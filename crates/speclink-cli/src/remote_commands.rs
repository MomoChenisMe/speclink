// Included into main.rs. Remote-mode detection and remote verb handlers.
//
// Routing rule: each dual-mode command checks `remote_ctx()` first; `None`
// falls through to the existing fs body untouched (fs behavior is the
// regression-protected baseline). Handlers here are a thin translation:
// argv → typed protocol client → the same rendering fs mode uses. Server
// extras (`repo`/`lifecycle`) live on the protocol DTOs and never leak into
// the parity view.

use speclink_protocol::command::CreateChangeRequest;
use speclink_protocol::query as protocol_query;
use speclink_remote::auth as remote_auth;
use speclink_remote::client::{Client as RemoteClient, ContextSnapshotOutcome};
use speclink_remote::credentials as remote_credentials;
use speclink_remote::credentials::CredentialStore as _;
use speclink_remote::login as remote_login;
use std::process::Stdio;

struct RemoteCtx {
    client: RemoteClient,
}

/// Resolve remote mode for the current workspace. `Ok(None)` = fs mode
/// (including "no workspace at all" — the fs path owns that error).
fn remote_ctx() -> Result<Option<RemoteCtx>> {
    // A broken .speclink.yaml fails here — before either the remote or the fs
    // path runs (fail-closed: no local reads, no remote requests).
    let Some(ws) = core::workspace::Workspace::discover_cwd()? else {
        return Ok(None);
    };
    let resolution = speclink_host::context::resolve_store_mode(&ws)?;
    let conn = match resolution.mode {
        core::workspace::StoreMode::Fs => return Ok(None),
        core::workspace::StoreMode::Remote(conn) => conn,
    };
    if resolution.coexists {
        eprintln!(
            "speclink: warning: the remote section in .speclink.yaml and {}/ both exist — remote mode takes effect",
            ws.spec_dir_name
        );
    }
    let origin = remote_auth::origin_of(&conn.url);
    // The binding handshake precedes every verb (fail closed): an
    // incompatible API version or a missing/ambiguous binding stops here —
    // no verb request leaves the client. It doubles as the credential's first
    // use, so a cached access token the server has aged out is caught here and
    // rotated once, invisibly.
    let authenticated = remote_auth::with_resolved_credential(&origin, |bearer| {
        RemoteClient::new(&conn.url, bearer, conn.repo.as_deref())
            .handshake()
            .map(|binding| (binding, bearer.to_string()))
    })
    .map_err(|e| anyhow::anyhow!(e.message(&origin)))?;
    let (binding, token) = authenticated.value;
    // The confirmed repo identity rides every subsequent request: a declared
    // `remote.repo` keeps its value; an undeclared one adopts the server's
    // unambiguous binding.
    let repo = if conn.repo.is_none() && !binding.repo.key.is_empty() {
        Some(binding.repo.key.as_str())
    } else {
        conn.repo.as_deref()
    };
    Ok(Some(RemoteCtx { client: RemoteClient::new(&conn.url, &token, repo) }))
}

// --- list ---

/// The wire summary reshaped into the core listing item — the same rendering
/// path then keeps stdout byte-identical to fs mode.
fn to_list_change_json(c: &protocol_query::ChangeSummary) -> ListChangeJson {
    ListChangeJson {
        completed_tasks: c.completed_tasks,
        name: c.name.clone(),
        status: c.status.clone(),
        summary: c.summary.clone(),
        total_tasks: c.total_tasks,
        restale_from: c.restale_from.clone(),
        meta_error: c.meta_error.clone(),
        // remote 恆缺席：worktree 是本機主 checkout 的觀察面，
        // server 端沒有這回事（spec scenario remote list 恆無 worktree 欄位）。
        worktree: None,
    }
}

fn remote_list(ctx: &RemoteCtx, a: &ListArgs) -> Result<()> {
    // 組出 fs 模式同一個 ListOutcome 再交給共用渲染：`--specs` 單獨給定時
    // changes 為 None，與引擎的 List outcome 同構。
    let specs_only = a.specs && !a.changes;
    let changes = if specs_only {
        None
    } else {
        let mut items: Vec<ListChangeJson> =
            ctx.client.list_changes()?.changes.iter().map(to_list_change_json).collect();
        // `--sort name` 在本地重排對齊 fs；`created`／`modified` 的排序鍵
        // （meta created、store mtime）不在 wire 上，維持 server 回傳序。
        if a.sort == "name" {
            items.sort_by(|x, y| x.name.cmp(&y.name));
        }
        Some(items)
    };
    let specs = if a.specs { Some(ctx.client.list_specs()?.specs) } else { None };
    render_list(&remote_list_outcome(changes, specs)?, a.json)
}

// --- change resolution (mirrors the fs wording) ---

/// Explicit name passes through; otherwise a single active change
/// auto-selects, several is an error, and none prints the info line and
/// returns `Ok(None)` (exit 0, matching fs `info_if_no_changes`).
fn remote_resolve_change(
    ctx: &RemoteCtx,
    name: Option<&str>,
    specify: &str,
) -> Result<Option<String>> {
    if let Some(n) = name {
        return Ok(Some(n.to_string()));
    }
    let mut names: Vec<String> = ctx
        .client
        .list_changes()?
        .changes
        .into_iter()
        .map(|c| c.name)
        .collect();
    match names.len() {
        0 => {
            println!("No active changes. Create one with: speclink new change <name>");
            Ok(None)
        }
        1 => Ok(Some(names.remove(0))),
        _ => bail!("Multiple changes found. {specify} {}", names.join(", ")),
    }
}

// --- status ---

/// The wire status reshaped into the fs report type — the same rendering
/// path keeps stdout byte-identical to fs mode; server extras (repo,
/// lifecycle, versions) simply don't cross this boundary.
fn to_status_report(status: protocol_query::ChangeStatus) -> core::status::StatusReport {
    core::status::StatusReport {
        change_name: status.change_name,
        schema_name: status.schema_name,
        is_complete: status.is_complete,
        apply_requires: status.apply_requires,
        artifacts: status
            .artifacts
            .into_iter()
            .map(|a| core::status::ArtifactStatusJson {
                id: a.id,
                output_path: a.output_path,
                status: a.status,
                blocked_by: a.missing_deps,
            })
            .collect(),
    }
}

fn remote_status(ctx: &RemoteCtx, a: &StatusArgs) -> Result<()> {
    if a.schema.is_some() {
        bail!("--schema is not supported in remote mode — the server's workflow config decides the schema");
    }
    let Some(name) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    let report = to_status_report(ctx.client.get_change(&name)?);
    if a.json {
        return print_json(&report);
    }
    render_status_human(&report);
    Ok(())
}

// --- instructions ---

fn to_apply_instructions(
    p: protocol_query::ApplyInstructions,
) -> core::instructions::ApplyInstructions {
    core::instructions::ApplyInstructions {
        change_name: p.change_name,
        change_dir: p.change_dir,
        schema_name: p.schema_name,
        context_files: p.context_files,
        progress: core::instructions::Progress {
            total: p.progress.total,
            complete: p.progress.complete,
            remaining: p.progress.remaining,
        },
        tasks: p
            .tasks
            .into_iter()
            .map(|t| core::instructions::TaskJson {
                id: t.id,
                description: t.description,
                done: t.done,
                parallel: t.parallel,
            })
            .collect(),
        state: p.state,
        missing_artifacts: p.missing_artifacts,
        locale: p.locale,
        instruction: p.instruction,
        // Deliberately fs-only (local file checks) — the wire contract
        // omits it, so the remote payload never renders one.
        preflight: None,
    }
}

fn to_artifact_instructions(
    p: protocol_query::ArtifactInstructions,
) -> core::instructions::ArtifactInstructions {
    core::instructions::ArtifactInstructions {
        change_name: p.change_name,
        artifact_id: p.artifact_id,
        schema_name: p.schema_name,
        change_dir: p.change_dir,
        output_path: p.output_path,
        description: p.description,
        instruction: p.instruction,
        context: p.context,
        rules: p.rules,
        locale: p.locale,
        template: p.template,
        dependencies: p
            .dependencies
            .into_iter()
            .map(|d| core::instructions::Dependency {
                id: d.id,
                done: d.done,
                path: d.path,
                description: d.description,
            })
            .collect(),
        unlocks: p.unlocks,
    }
}

/// Snapshot source for the projection materializer: one consistent snapshot
/// already fetched from the Context API. Flow narrowing is the materializer's
/// job (design 決策三), so the provider returns the fetched snapshot verbatim —
/// the provider seam keeps the Context API call out of the materializer.
struct VerbContextProvider {
    snapshot: speclink_protocol::context::ContextSnapshot,
}

impl speclink_host::projection::SnapshotProvider for VerbContextProvider {
    fn snapshot(
        &self,
        _request: &speclink_protocol::context::ContextSnapshotRequest,
    ) -> Result<speclink_protocol::context::ContextSnapshot> {
        Ok(self.snapshot.clone())
    }
}

/// The remote verb flow's projection refresh: fetch one consistent Context API
/// snapshot for this change's apply flow, then materialize it and point
/// contextFiles into the projection. The manifest's current snapshot id travels
/// as `If-None-Match`, so an unchanged scope returns 304 and the rewrite is
/// skipped (免重寫). Projection trouble is a loud warning, never a verb failure —
/// the instructions payload is intact either way, and a failed fetch marks the
/// existing projection stale rather than serving it silently.
fn point_context_files_at_projection(
    ctx: &RemoteCtx,
    name: &str,
    context_files: &mut std::collections::BTreeMap<String, String>,
) {
    let Some(ws) = core::workspace::Workspace::discover_cwd().ok().flatten() else {
        return;
    };
    let request = speclink_protocol::context::ContextSnapshotRequest {
        change: Some(name.to_string()),
        flow: Some("apply".to_string()),
    };
    let known = speclink_host::projection::current_snapshot_id(&ws);
    match ctx.client.context_snapshot(&request, known.as_deref()) {
        // Unchanged since the projection's snapshot id: leave it untouched.
        Ok(ContextSnapshotOutcome::Unchanged) => {}
        Ok(ContextSnapshotOutcome::Fresh(snapshot)) => {
            let provider = VerbContextProvider { snapshot };
            match speclink_host::projection::materialize(&ws, &provider, &request) {
                Ok(out) => {
                    for w in &out.warnings {
                        eprintln!("speclink: warning: {w}");
                    }
                }
                Err(e) => eprintln!("speclink: warning: context projection not refreshed: {e:#}"),
            }
        }
        Err(e) => {
            eprintln!("speclink: warning: context projection not refreshed: {e}");
            // Keep the existing projection but flag it stale. No projection yet
            // is a no-op (mark_stale bails, which we ignore).
            let _ = speclink_host::projection::mark_stale(&ws);
        }
    }
    core::instructions::project_context_files(
        context_files,
        &speclink_host::projection::projection_dir(&ws).join("openspec"),
        name,
    );
}

fn remote_instructions(ctx: &RemoteCtx, a: &InstructionsArgs) -> Result<()> {
    if a.schema.is_some() {
        bail!("--schema is not supported in remote mode — the server's workflow config decides the schema");
    }
    let Some(name) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    // No-arg default mirrors fs mode: the first incomplete artifact (the
    // server's artifact list is already in display order), else "apply".
    let artifact = match a.artifact.as_deref() {
        Some(s) => s.to_string(),
        None => ctx
            .client
            .get_change(&name)?
            .artifacts
            .iter()
            .find(|x| x.status != "done")
            .map(|x| x.id.clone())
            .unwrap_or_else(|| "apply".to_string()),
    };
    if artifact == "apply" {
        let mut p = to_apply_instructions(ctx.client.apply_instructions(&name)?);
        point_context_files_at_projection(ctx, &name, &mut p.context_files);
        if a.json {
            return print_json(&p);
        }
        render_apply_human(&p);
    } else {
        let p = to_artifact_instructions(ctx.client.artifact_instructions(&name, &artifact)?);
        if a.json {
            return print_json(&p);
        }
        render_artifact_human(&p);
    }
    Ok(())
}

// --- init / link / unlink / auth ---

/// `tools` arrives already resolved and validated by `cmd_init` — filesystem and remote
/// init share that one entry, so a remote checkout is bootstrapped from the same selection
/// rules (and the same non-empty guarantee) as a local one.
fn cmd_init_remote(
    a: &InitArgs,
    root: &std::path::Path,
    display_base: &str,
    tools: &[core::skills::Tool],
) -> Result<()> {
    let Some(url) = a.url.as_deref() else {
        bail!("--store remote requires --url <project-scoped url>");
    };
    if a.dir.is_some() {
        bail!("--dir has no meaning with --store remote (documents live on the server)");
    }
    core::init::init_remote(root, tools, a.force, url, a.repo.as_deref())?;
    println!("{} Initialized at {display_base} (remote store)", color::green("✓"));
    let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    println!("Generated files for: {}", names.join(", "));
    // Validate the declared repo now when credentials exist; defer otherwise
    // (offline init must not block — the first verb still validates).
    validate_or_defer(root, url, a.repo.as_deref())
}

fn cmd_link(a: LinkArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let origin = remote_auth::origin_of(&a.url);
    // The full ladder, not just the file: a desktop login already covers this
    // origin, and reporting "no credentials yet" at it would be a lie.
    match remote_auth::resolve_credential(&origin).ok().flatten() {
        Some(resolved) => {
            let token = resolved.token;
            let whoami = RemoteClient::new(&a.url, &token, None).whoami()?;
            if let Some(repo) = a.repo.as_deref() {
                ensure_repo_registered(&whoami, repo)?;
                println!("{} Repo '{repo}' is registered in this project", color::green("✓"));
            }
            core::init::write_remote_section(&root, &a.url, a.repo.as_deref())?;
            println!("{} Linked to {}", color::green("✓"), a.url);
            git_reference_warning(&root, a.repo.as_deref(), &whoami);
            Ok(())
        }
        None => {
            core::init::write_remote_section(&root, &a.url, a.repo.as_deref())?;
            println!("{} Linked to {}", color::green("✓"), a.url);
            println!("  No credentials yet — run `speclink auth login` to connect");
            Ok(())
        }
    }
}

fn cmd_unlink() -> Result<()> {
    let root = core::workspace::Workspace::discover_cwd()?
        .map(|ws| ws.root)
        .unwrap_or(std::env::current_dir()?);
    if !core::init::remove_remote_section(&root)? {
        bail!("No remote connection to remove (no `remote:` section in .speclink.yaml)");
    }
    println!(
        "{} Removed the remote section from .speclink.yaml — back to the local fs store",
        color::green("✓")
    );
    Ok(())
}

fn cmd_auth(a: AuthArgs) -> Result<()> {
    // Both auth verbs need the connection (its origin keys the credentials).
    let Some(ws) = core::workspace::Workspace::discover_cwd()? else {
        bail!("Not connected to a remote store — run `speclink link <url>` first");
    };
    let conn = match speclink_host::context::resolve_store_mode(&ws)?.mode {
        core::workspace::StoreMode::Remote(conn) => conn,
        core::workspace::StoreMode::Fs => {
            bail!("Not connected to a remote store — run `speclink link <url>` first")
        }
    };
    let origin = remote_auth::origin_of(&conn.url);
    match a.command {
        AuthCommands::Login { token_stdin, pat } => {
            if token_stdin || pat {
                return login_with_pat(&conn, &origin, token_stdin);
            }
            // Device authorization needs a terminal to show the code in and a
            // keyring to put the credential in. Missing either is a hard stop
            // with the flag that works instead — never a silent downgrade.
            if !std::io::stdin().is_terminal() {
                bail!(
                    "`speclink auth login` needs a terminal to authorize this device — use `--token-stdin` to pipe a personal access token instead"
                );
            }
            login_with_device(&conn, &origin)
        }
        AuthCommands::Status { json } => {
            let resolved = remote_auth::resolve_credential(&origin)
                .map_err(|e| anyhow::anyhow!(e.message()))?;
            let Some(resolved) = resolved else {
                bail!("Not logged in to {origin} — run `speclink auth login`");
            };
            let whoami = RemoteClient::new(&conn.url, &resolved.token, None).whoami()?;
            if json {
                return print_json(&serde_json::json!({
                    "user": {
                        "name": whoami.user.name,
                        "handle": whoami.user.handle,
                    },
                    "credentialSource": resolved.source.as_str(),
                }));
            }
            print_identity(&whoami);
            println!("  Credential from: {}", resolved.source.describe());
            if let Some(repo) = conn.repo.as_deref() {
                match ensure_repo_registered(&whoami, repo) {
                    Ok(()) => println!("Repo '{repo}' is registered in this project"),
                    Err(e) => println!("! {e}"),
                }
            }
            git_reference_warning(&ws.root, conn.repo.as_deref(), &whoami);
            Ok(())
        }
        AuthCommands::Logout => cmd_auth_logout(&origin),
    }
}

/// The PAT tracks, unchanged from before device authorization existed: paste
/// interactively, or pipe from stdin for CI. Both land in the credentials
/// file, and both validate before storing so a rejected token is never
/// written.
fn login_with_pat(
    conn: &core::workspace::RemoteConnection,
    origin: &str,
    token_stdin: bool,
) -> Result<()> {
    if !token_stdin {
        eprint!("Paste your personal access token: ");
    }
    let token = read_stdin().trim().to_string();
    if token.is_empty() {
        bail!("empty token — nothing stored");
    }
    let whoami = RemoteClient::new(&conn.url, &token, None).whoami()?;
    remote_auth::save_token_at(&remote_auth::speclink_config_dir(), origin, &token)?;
    print_identity(&whoami);
    Ok(())
}

/// Terminal, browser and clock for a device login.
struct CliDeviceIo;

impl remote_login::DeviceLoginIo for CliDeviceIo {
    fn announce(&self, verification_uri: &str, user_code: &str) {
        println!("Open {verification_uri} and enter code: {user_code}");
        println!("Waiting for approval…");
    }

    fn open_browser(&self, url: &str) -> bool {
        #[cfg(target_os = "macos")]
        let mut cmd = std::process::Command::new("open");
        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = std::process::Command::new("cmd");
            c.args(["/C", "start", ""]);
            c
        };
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        let mut cmd = std::process::Command::new("xdg-open");

        cmd.arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn sleep_secs(&self, secs: u64) {
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
}

fn login_with_device(conn: &core::workspace::RemoteConnection, origin: &str) -> Result<()> {
    let store = remote_credentials::KeyringCredentialStore;
    // Probe before starting: approving in a browser only to discover there is
    // nowhere to keep the credential wastes the user's time.
    if store.get(origin, remote_credentials::CredentialKind::Refresh).is_err() {
        bail!(
            "no system keychain available on this machine — a device login's refresh credential is never written to a plain file; use `speclink auth login --pat` or set SPECLINK_TOKEN"
        );
    }

    let outcome = remote_login::device_login(
        origin,
        &store,
        &remote_auth::speclink_config_dir(),
        &CliDeviceIo,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    match outcome {
        remote_login::DeviceLoginOutcome::Approved { display } => {
            println!("{} Logged in as {display}", color::green("✓"));
            if let Some(repo) = conn.repo.as_deref() {
                println!("Connected to repo '{repo}'");
            }
            Ok(())
        }
        remote_login::DeviceLoginOutcome::Denied => {
            bail!("authorization denied — nothing was stored")
        }
        remote_login::DeviceLoginOutcome::Expired => {
            bail!("authorization expired before it was approved — run `speclink auth login` again")
        }
        remote_login::DeviceLoginOutcome::Unsupported => {
            bail!("this server does not offer device authorization — use `speclink auth login --pat`")
        }
    }
}

/// Logout revokes the credential family on the server, then clears every local
/// credential for the origin. The family is shared with the desktop app, so
/// this logs the whole machine out — the symmetric cost of one login covering
/// both. The server-side PAT is left alone: it is a separate credential the
/// user manages, possibly in use from another machine.
fn cmd_auth_logout(origin: &str) -> Result<()> {
    let store = remote_credentials::KeyringCredentialStore;
    let dir = remote_auth::speclink_config_dir();
    let refresh = store
        .get(origin, remote_credentials::CredentialKind::Refresh)
        .ok()
        .flatten();
    let keyring_pat = store
        .get(origin, remote_credentials::CredentialKind::Pat)
        .ok()
        .flatten();
    let had_file_token = remote_auth::load_token_at(&dir, origin).is_some();

    if refresh.is_none() && keyring_pat.is_none() && !had_file_token {
        bail!("Not logged in to {origin}");
    }

    // A revoke that cannot reach the server still leaves nothing usable here;
    // the family outliving us is worth a warning, not a failed logout.
    if let Some(refresh) = refresh.as_deref() {
        if let Err(e) = speclink_remote::device::revoke(origin, refresh) {
            eprintln!(
                "speclink: warning: could not revoke the credential family on the server ({e}) — it stays live until it expires"
            );
        }
    }
    remote_login::clear_all_local_credentials(origin, &store);
    remote_auth::remove_token_at(&dir, origin)?;
    println!("{} Logged out of {origin}", color::green("✓"));
    Ok(())
}

fn print_identity(whoami: &protocol_query::WhoamiResponse) {
    if whoami.user.handle.is_empty() {
        println!("{} Logged in as {}", color::green("✓"), whoami.user.name);
    } else {
        println!(
            "{} Logged in as {} (@{})",
            color::green("✓"),
            whoami.user.name,
            whoami.user.handle
        );
    }
}

/// Check the declared repo against the server's registry (`whoami.repos[]`).
fn ensure_repo_registered(whoami: &protocol_query::WhoamiResponse, repo: &str) -> Result<()> {
    let names: Vec<&str> = whoami.repos.iter().map(|r| r.name.as_str()).collect();
    if names.iter().any(|n| *n == repo) {
        return Ok(());
    }
    bail!(
        "repo '{repo}' is not registered in this project (available: {})",
        names.join(", ")
    )
}

/// Post-init/link validation: immediate when credentials exist, deferred with
/// a login hint otherwise.
fn validate_or_defer(root: &std::path::Path, url: &str, repo: Option<&str>) -> Result<()> {
    let origin = remote_auth::origin_of(url);
    // The full ladder, not just the file: a desktop login already covers this
    // origin, and reporting "no credentials yet" at it would be a lie.
    match remote_auth::resolve_credential(&origin).ok().flatten() {
        Some(resolved) => {
            let token = resolved.token;
            let whoami = RemoteClient::new(url, &token, None).whoami()?;
            if let Some(repo) = repo {
                ensure_repo_registered(&whoami, repo)?;
                println!("{} Repo '{repo}' is registered in this project", color::green("✓"));
            }
            git_reference_warning(root, repo, &whoami);
            Ok(())
        }
        None => {
            println!("  No credentials yet — run `speclink auth login` to connect");
            Ok(())
        }
    }
}

/// Advisory fork/mirror hint: compare the local `git remote origin` URL with
/// the server's reference value for this repo. One stderr line on mismatch;
/// silence when either side has no value; never affects results or exit code.
fn git_reference_warning(
    root: &std::path::Path,
    repo: Option<&str>,
    whoami: &protocol_query::WhoamiResponse,
) {
    let Some(repo) = repo else { return };
    let reference = whoami
        .repos
        .iter()
        .find(|r| r.name == repo)
        .map(|r| r.git_url.clone())
        .unwrap_or_default();
    if reference.is_empty() {
        return;
    }
    let local = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if local.is_empty() {
        return; // not a git dir / no origin — silently skip
    }
    let norm = |u: &str| u.trim().trim_end_matches('/').trim_end_matches(".git").to_string();
    if norm(&local) != norm(&reference) {
        eprintln!(
            "speclink: warning: local git remote ({local}) differs from the project's reference ({reference}) — you may be working on a fork or mirror"
        );
    }
}

// --- drift ---

/// Remote drift: the Server supplies the spec side, the basis it was computed
/// against, and the change's store-side inputs; the workspace side is collected
/// and computed here off the local checkout; the Engine's one merger assembles
/// the report and the renderer fs mode uses prints it — so there is no second
/// merge and no second output shape.
fn remote_drift(ctx: &RemoteCtx, a: &ChangeArg) -> Result<()> {
    // Positional-style wording, matching the fs verb's auto-detect error.
    let Some(name) = remote_resolve_change(ctx, a.change.as_deref(), "Specify one:")? else {
        return Ok(());
    };
    // A server failure stops here: no report is rendered from half the facts.
    let response = ctx.client.spec_drift(&name)?;

    let docs = speclink_host::drift::RemoteDriftStore::new(
        &name,
        response.change.created,
        response.change.design,
        response.change.tasks,
    );
    let change = docs.change();
    let spec = speclink_host::drift::spec_drift_from_wire(&response.spec_drift);

    // The workspace side needs a checkout. Remote mode always has a workspace
    // (the .speclink.yaml the mode was resolved from), so git availability is
    // what actually distinguishes "there is code here" from "there is not".
    // Without it the facts are absent — which the Engine reports as
    // unavailable. Collecting anyway would stat the anchors against a
    // codeless directory and report every one of them broken: an absent
    // checkout would read as deleted code.
    let ws = core::workspace::Workspace::discover_cwd()?
        .filter(|w| core::util::git_available(&w.root));
    let facts = ws
        .as_ref()
        .map(|w| speclink_host::drift::collect_workspace_facts(w, &docs, &change));
    let workspace = core::drift::compute_workspace_drift(&docs, &change, facts.as_ref());

    // The spec side and its basis come from one server snapshot, so expected
    // and current are the same fixed point and the report is never stale —
    // matching fs mode, where the bundle and the current digests are read
    // back-to-back off one store.
    let digests = speclink_host::drift::basis_from_wire(&response.basis);
    let basis = core::drift::DriftBasis { expected: digests.clone(), current: digests };
    let report = core::drift::merge_drift_reports(&change, spec, workspace, Some(&basis));
    if a.json {
        return print_json(&report);
    }
    render_drift(&report.report);
    Ok(())
}

// --- artifact cat (remote) ---

fn remote_artifact_cat(ctx: &RemoteCtx, artifact: &str, change: Option<&str>) -> Result<()> {
    // Validate the id shape locally so both modes reject the same inputs.
    let _ = artifact_rel_path(artifact)?;
    let change = match change {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    print!("{}", ctx.client.get_artifact(&change, artifact)?.content);
    Ok(())
}

// --- write path: changes ---

fn remote_new_change(ctx: &RemoteCtx, a: &NewChangeArgs) -> Result<()> {
    let resp = ctx.client.create_change(CreateChangeRequest {
        name: a.name.clone(),
        schema: a.schema.clone(),
        description: a.description.clone(),
        agent: a.agent.clone(),
        from_discussion: a.from_discussion.clone(),
    })?;
    // Path 行是明文分歧（design D5）：server 端目錄對本機使用者無意義。
    render_new_change(
        &a.name,
        NewChangeLines {
            path: None,
            schema: resp.schema.as_deref().filter(|s| !s.is_empty()),
            from_discussion: a.from_discussion.as_deref(),
        },
    );
    Ok(())
}

/// Map the fs artifact TYPE argument onto the contract's artifact path.
fn remote_artifact_path(artifact_type: &str, capability: Option<&str>) -> Result<(String, &'static str)> {
    match artifact_type {
        "proposal" => Ok(("proposal".to_string(), "proposal")),
        "design" => Ok(("design".to_string(), "design")),
        "tasks" => Ok(("tasks".to_string(), "tasks")),
        "spec" => {
            let cap = capability
                .ok_or_else(|| anyhow::anyhow!("Capability name required for spec artifacts"))?;
            Ok((format!("specs/{cap}"), "specs"))
        }
        other => bail!("Unknown artifact type '{other}'. Valid types: proposal, design, tasks, spec"),
    }
}

fn remote_new_artifact(ctx: &RemoteCtx, a: &NewArtifactArgs) -> Result<()> {
    let (artifact_path, schema_artifact_id) =
        remote_artifact_path(&a.artifact_type, a.capability.as_deref())?;
    let content = if a.stdin {
        read_stdin()
    } else {
        // Template comes from the server's workflow schema, rendered by the
        // embedded engine (built-in/user schema definitions are engine-local).
        let schema_name = ctx.client.config()?.schema;
        let name = if schema_name.is_empty() { "spec-driven".to_string() } else { schema_name };
        match core::schema::resolve_with(None, Some(&speclink_host::context::global_config_dir()), &name) {
            Some(Ok(schema)) => schema
                .artifact(schema_artifact_id)
                .and_then(|art| art.template.clone())
                .unwrap_or_default(),
            _ => String::new(),
        }
    };
    let change = match a.change.as_deref() {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    // --force overwrites: re-read the current version so the write still
    // asserts what it replaces; plain create asserts absence (If-Match: 0).
    let version = if a.force {
        ctx.client
            .get_artifact(&change, &artifact_path)
            .map(|got| got.version)
            .unwrap_or(0)
    } else {
        0
    };
    ctx.client.put_artifact(&change, &artifact_path, &content, version)?;
    if a.json {
        let v = serde_json::json!({
            "artifact": a.artifact_type,
            "change": change,
            "path": artifact_path,
            "status": "created",
            "validated": a.stdin,
            "warnings": [],
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Created {}: {}", color::green("✓"), a.artifact_type, artifact_path);
    if a.stdin {
        println!("  Content validated ✓");
    }
    Ok(())
}

fn remote_task_done(
    ctx: &RemoteCtx,
    task_id: &str,
    change: Option<&str>,
    json: bool,
) -> Result<()> {
    let change = match change {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    // Attribution: same git-derived touched-file set the fs path records.
    // Best-effort — remote mode already resolved, so a config error here is
    // unreachable; an empty set is the existing no-workspace behavior.
    let ws = core::workspace::Workspace::discover_cwd().ok().flatten();
    let touched: Vec<String> = ws
        .map(|w| core::tasks::git_changed_files(&w))
        .unwrap_or_default();
    let resp = ctx.client.task_done(&change, task_id, &touched)?;
    // remote 只有 argv 一種識別，拒絕訊息與 stdout 兩處都餵它。
    render_task_flip(
        TaskFlip::Done,
        &change,
        TaskIdentity { refused: task_id, arg: task_id },
        &resp.task_desc,
        resp.already_done,
        json,
    )
}

fn remote_task_undone(
    ctx: &RemoteCtx,
    task_id: &str,
    change: Option<&str>,
    json: bool,
) -> Result<()> {
    let change = match change {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    let resp = ctx.client.task_undone(&change, task_id)?;
    render_task_flip(
        TaskFlip::Undone,
        &change,
        TaskIdentity { refused: task_id, arg: task_id },
        &resp.task_desc,
        resp.already_undone,
        json,
    )
}

fn remote_claim(ctx: &RemoteCtx, name: &str) -> Result<()> {
    let _ = ctx.client.claim(name)?;
    println!("{} Claimed change: {name}", color::green("✓"));
    Ok(())
}

fn remote_in_progress_remove(ctx: &RemoteCtx, name: &str) -> Result<()> {
    // 回應的 removed 區分實際移除與未開工冪等,兩者印不同的行(舊 server 的
    // 裸 Ack 讀作已移除,即其原本的意思);守門 409 與 404 的 message 為引擎
    // 凍結文字,經 `?` 轉發後 stderr 與 fs 模式逐位元一致。
    let removed = ctx.client.in_progress_remove(name)?.removed;
    render_in_progress_remove(name, removed);
    Ok(())
}

// --- validate / analyze：唯讀衍生查詢的 remote 分流（remote-verb-parity）---

/// 聚合語意（無參數／--all／--changes）由 client 組合：先 list 再逐 change 打
/// 單 change 端點（design 決策 2）；DTO 轉回本地型別後走 fs 同一渲染（決策 6）。
fn remote_validate(ctx: &RemoteCtx, a: &ValidateArgs) -> Result<()> {
    let names: Vec<String> = if let Some(item) = &a.item {
        vec![item.clone()]
    } else {
        // 無參數與 --all/--changes 的目標集在 remote 皆為 scope 的全部
        // changes（fs 的「恰一個 active change 單驗」是同集合的特例）。
        ctx.client.list_changes()?.changes.into_iter().map(|c| c.name).collect()
    };
    let results: Vec<core::validate::ValidationResult> = names
        .iter()
        .map(|n| ctx.client.validate_change(n).map(speclink_remote::convert::validation_result))
        .collect::<Result<Vec<_>, _>>()?;
    render_validate_results(&results, a.json)
}

fn remote_analyze(ctx: &RemoteCtx, a: &ChangeArg) -> Result<()> {
    let Some(name) = remote_resolve_change(ctx, a.change.as_deref(), "Specify one:")? else {
        return Ok(());
    };
    let report = speclink_remote::convert::analyze_report(ctx.client.analyze_change(&name)?);
    if a.json {
        return print_json(&report);
    }
    render_analyze(&report);
    Ok(())
}

/// remote discard：直通 DELETE 端點（--force 為 query 參數）。guard 拒絕由
/// server 以引擎凍結訊息（含「pass --force」指引）回來，經標準錯誤翻譯原文
/// 呈現——與 fs 模式同語意（design 決策 3／6）。
fn remote_discard(ctx: &RemoteCtx, a: &DiscardArgs) -> Result<()> {
    let outcome = ctx.client.discard(&a.change, a.force)?;
    let unlinked: Vec<(String, String)> = outcome
        .unlinked_discussions
        .into_iter()
        .map(|d| (d.slug, d.status))
        .collect();
    render_discard(&outcome.change, &unlinked, a.json)
}

fn remote_archive(ctx: &RemoteCtx, a: &ArchiveArgs) -> Result<()> {
    if a.all || a.changes.len() > 1 {
        bail!("bulk archive is not supported in remote mode — archive changes one at a time");
    }
    let Some(name) = a.changes.first().cloned().map(Some).unwrap_or(None) else {
        bail!("Please specify a change to archive.");
    };
    let resp = ctx.client.archive(&name, a.carry_review, a.carry_verify)?;
    // datedName 是哨兵：新 server 給得出完整封存結果，就走 fs 同一支渲染；
    // 舊 server 什麼都沒帶，退回既有的兩行輸出而不是半套。
    match to_archive_outcome(name, resp) {
        Ok(outcome) => print_archive_outcome(&outcome),
        Err((name, resp)) => {
            println!("{} Archived change: {name}", color::green("✓"));
            if !resp.specs.is_empty() {
                let caps: Vec<&str> = resp.specs.iter().map(|s| s.capability.as_str()).collect();
                println!("  Specs updated: {}", caps.join(", "));
            }
        }
    }
    Ok(())
}

/// The wire archive response reshaped into the engine's outcome, so the fs
/// rendering path prints it as-is. `Err` hands back the inputs when the
/// sentinel (`datedName`) is absent — an old server whose response cannot
/// honestly fill the outcome.
fn to_archive_outcome(
    change_name: String,
    resp: speclink_protocol::command::ArchiveResponse,
) -> std::result::Result<
    core::archive::ArchiveOutcome,
    (String, speclink_protocol::command::ArchiveResponse),
> {
    let Some(dated_name) = resp.dated_name.clone() else {
        return Err((change_name, resp));
    };
    Ok(core::archive::ArchiveOutcome {
        change_name,
        dated_name,
        caps: resp
            .specs
            .into_iter()
            .map(|s| core::archive::CapCounts {
                capability: s.capability,
                added: s.added,
                modified: s.modified,
                removed: s.removed,
                renamed: s.renamed,
            })
            .collect(),
        snapshot_created: resp.snapshot_created.unwrap_or(false),
        // remote 的封存永遠不跳過 specs（端點無此旗標）——欄位對渲染無影響。
        skipped_specs: false,
        archived_discussions: resp
            .archived_discussions
            .into_iter()
            .map(|d| (d.slug, d.file))
            .collect(),
        // 缺席讀作「有證據」，零證據提示因此不會憑空冒出來。
        evidence_recorded: resp.evidence_recorded.unwrap_or(true),
    })
}

// --- show ---

/// remote show 的讀取組合（design D4）：以既有讀 API 組出與 fs 模式相同的
/// ShowOutcome，交給共用的 render_show 渲染。item 與 --item-type 的判別序
/// 對齊引擎 run_show：type==spec 直取規格；否則 change 優先、缺席時規格
/// 遞補（type==change 不遞補）；錯誤訊息沿引擎凍結文本。
fn remote_show_outcome(
    ctx: &RemoteCtx,
    item: Option<&str>,
    item_type: Option<&str>,
) -> Result<core::command::ShowOutcome> {
    let Some(item) = item else {
        bail!("Please specify an item name.");
    };
    if let Some(t) = item_type {
        if t != "change" && t != "spec" {
            bail!("Unknown type: {t}. Use 'change' or 'spec'.");
        }
    }
    let is_spec = ctx.client.list_specs()?.specs.iter().any(|s| s.id == item);
    let status = if item_type == Some("spec") {
        None
    } else {
        match ctx.client.get_change(item) {
            Ok(status) => Some(status),
            Err(e) if e.status == Some(404) => None,
            Err(e) => return Err(e.into()),
        }
    };
    let show_spec =
        item_type == Some("spec") || (item_type != Some("change") && status.is_none() && is_spec);
    if show_spec {
        if !is_spec {
            bail!("Spec '{item}' not found.");
        }
        let content = ctx.client.spec_document(item)?.content;
        return Ok(core::command::ShowOutcome::Spec { name: item.to_string(), content });
    }
    let Some(status) = status else {
        if item_type == Some("change") {
            bail!("Change '{item}' not found.");
        }
        bail!("Item '{item}' not found as a change or spec.");
    };
    // 成對規則由 server 套用：created 出現即代表 meta 的 schema+created 成對。
    let (schema, created) = match status.created {
        Some(created) => (Some(status.schema_name.clone()), Some(created)),
        None => (None, None),
    };
    let read_artifact = |artifact: &str| -> Result<Option<String>> {
        match ctx.client.get_artifact(item, artifact) {
            Ok(content) => Ok(Some(content.content)),
            Err(e) if e.status == Some(404) => Ok(None),
            Err(e) => Err(e.into()),
        }
    };
    // restaleFrom 走清單摘要（設計上 show 組合的「兩份清單」之一）。
    let restale_from = ctx
        .client
        .list_changes()?
        .changes
        .into_iter()
        .find(|c| c.name == item)
        .map(|c| c.restale_from)
        .unwrap_or_default();
    Ok(core::command::ShowOutcome::Change(core::command::ShowChange {
        name: status.change_name,
        schema,
        created,
        proposal: read_artifact("proposal")?,
        design: read_artifact("design")?,
        tasks: read_artifact("tasks")?,
        delta_capabilities: status.delta_capabilities,
        from_discussions: status.from_discussions,
        restale_from,
    }))
}

// --- 品質站（design D4a：動詞端點；spec「remote 模式下的動詞行為」）---

/// `review prepare` 的 remote 面：sidecar 仍在本地 checkout，先做 remote read
/// （存在＋startedAt），失敗即零 sidecar effects。listing 的 status 只由任務
/// 完成度推導，不能當「已開工」讀。
fn remote_review_prepare(ctx: &RemoteCtx, change: String) -> Result<()> {
    let summary = ctx
        .client
        .list_changes()?
        .changes
        .into_iter()
        .find(|c| c.name == change)
        .ok_or_else(|| anyhow::anyhow!("change not found: {change}"))?;
    let ws = require_workspace()?;
    run_review_prepare(&ws, &change, summary.started_at.is_some())
}

/// 兩個品質站的 remote 動詞（唯一實作落點）：工單經 typed client 的站別端點
/// 讀寫，scope 仍由本地 checkout 的 Host resolver 解析——server 不收 patch、
/// 不收 snapshot，也沒有 Git endpoint。
fn remote_station(ctx: &RemoteCtx, cli: &StationCli, verb: StationVerb) -> Result<()> {
    let st = cli.station;
    let noun = st.noun;
    match verb {
        StationVerb::Scope { change, json, base, candidate_hash, include_hunk } => {
            // 同一 Host resolver：remote 只提供 active changes 與 ticket 事實，
            // Git、baseline、touched、snapshot 全在本地 checkout。
            let changes = ctx.client.list_changes()?.changes;
            if !changes.iter().any(|c| c.name == change) {
                anyhow::bail!("change not found: {change}");
            }
            let ws = require_workspace()?;
            let ticket = ctx.client.station_ticket_if_any(noun, &change)?.map(|t| {
                speclink_host::change_diff::TicketBinding {
                    patch_hash_chain: patch_hash_chain(
                        t.rounds.iter().map(|r| r.patch_hash.as_deref()),
                    ),
                    finding_paths: t
                        .last_round
                        .findings
                        .iter()
                        .map(|f| f.path.clone())
                        .collect(),
                }
            });
            let names = changes.into_iter().map(|c| c.name).collect();
            let req = build_scope_request(
                &ws,
                change,
                names,
                ticket,
                base,
                candidate_hash,
                include_hunk,
                cli.ns,
            );
            run_station_scope(&ws, st, &req, json)
        }
        StationVerb::AddRound { change, stdin } => {
            let content = read_stdin_content(stdin);
            let round = ctx.client.station_add_round(noun, &change, &content)?.round;
            // u64→usize：支援平台皆 64-bit，無損（不為不可能的情境設防）。
            render_station_action(st, StationAction::AddRound(round as usize), &change);
            Ok(())
        }
        StationVerb::Show { change, json } => {
            let resp = ctx.client.station_ticket(noun, &change)?;
            // 人眼＋有原文＝純轉印，不碰結構化欄位——這正是原文上 wire 的目的：
            // server 詞彙比 CLI 新（未知 token、新形狀）也不影響印出工單本文。
            // 解析（與其 fail-loud）只留給真正讀 token 的兩條路：--json 與退化摘要。
            if !json {
                if let Some(doc) = &resp.content {
                    print!("{doc}");
                    return Ok(());
                }
            }
            let ticket = to_station_ticket(resp)?;
            render_station_show(st, &change, &ticket, None, json)
        }
        StationVerb::Stamp { change, accept, agent } => {
            // 指紋歸屬（design D4a）：工作樹持有者是這裡——先取工單算 Scope
            // 聯集（鏡射引擎的正規化：`\`→`/`、去重、排序），逐檔讀 checkout
            // 內容算雜湊，隨請求上 wire；server 驗集合相等、不重算。
            let ticket = ctx.client.station_ticket(noun, &change)?;
            let Some(ws) = core::workspace::Workspace::discover_cwd()? else {
                anyhow::bail!(
                    "{noun} stamp needs a workspace checkout to fingerprint scope files"
                );
            };
            let paths = core::station::scope_union(
                ticket.rounds.iter().flat_map(|r| r.scope.iter().map(String::as_str)),
            );
            // 修正可能刪除／改名早輪檢查過的檔：仍存在者算雜湊，已消失者以
            // missing 明示宣告——server 無工作樹，分割由這裡的存在性判定。
            let (present, missing): (Vec<String>, Vec<String>) =
                paths.into_iter().partition(|p| ws.root.join(p).is_file());
            let read_file = |p: &str| std::fs::read_to_string(ws.root.join(p)).ok();
            let scope: Vec<_> = core::station::fingerprint_scope(&present, &read_file)?
                .into_iter()
                .map(|(path, hash)| speclink_protocol::command::ReviewScopeEntryDto { path, hash })
                .collect();
            ctx.client.station_stamp(noun, &change, accept, agent.as_deref(), &scope, &missing)?;
            render_station_action(st, StationAction::Stamp, &change);
            Ok(())
        }
        StationVerb::Discard { change } => {
            ctx.client.station_discard(noun, &change)?;
            render_station_action(st, StationAction::Discard, &change);
            Ok(())
        }
    }
}

/// The wire ticket reshaped into the engine's ticket, so `--json` goes through
/// the one payload assembly whose field set is the public contract (and the
/// document body, which is not part of that contract, cannot leak into it).
///
/// An unrecognized phase token is an error, not a silent `None`: it means the
/// server speaks a round vocabulary this CLI does not, and rendering it as a
/// legacy round would state something false about the ticket.
fn to_station_ticket(
    resp: speclink_protocol::command::ReviewTicketResponse,
) -> Result<core::station::Ticket> {
    // 引擎工單的不變量是至少一輪；wire 形狀上允許空陣列，所以這裡是明確
    // 錯誤，不是留給 last_round() 的 expect 去炸。
    if resp.rounds.is_empty() {
        anyhow::bail!(
            "the server returned a ticket with no rounds for change '{}' — cannot render it",
            resp.change
        );
    }
    let rounds = resp
        .rounds
        .into_iter()
        .map(to_station_round)
        .collect::<Result<Vec<_>>>()?;
    Ok(core::station::Ticket { rounds })
}

/// server 端詞彙比這支 CLI 新——靜默吞掉會渲染出錯誤事實，一律報錯。
fn unknown_ticket_token(kind: &str, token: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "unknown {kind} '{token}' from the server — this CLI is older than the server's ticket format"
    )
}

fn to_station_round(
    r: speclink_protocol::command::ReviewRoundDto,
) -> Result<core::station::Round> {
    let phase = match r.phase.as_deref() {
        None => None,
        Some(token) => Some(
            core::station::RoundPhase::parse(token)
                .ok_or_else(|| unknown_ticket_token("round phase", token))?,
        ),
    };
    let findings = r
        .findings
        .into_iter()
        .map(|f| {
            let severity = core::station::Severity::parse(&f.severity)
                .ok_or_else(|| unknown_ticket_token("finding severity", &f.severity))?;
            Ok(core::station::Finding { severity, path: f.path, text: f.text })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(core::station::Round {
        index: r.index as usize,
        phase,
        patch_hash: r.patch_hash,
        scope: r.scope,
        findings,
    })
}

// --- discuss ---

fn remote_discuss(ctx: &RemoteCtx, a: DiscussArgs) -> Result<()> {
    match a.command {
        DiscussCommands::List { archived, json } => {
            let items: Vec<_> = ctx
                .client
                .list_discussions(archived)?
                .discussions
                .iter()
                .map(to_discussion_info)
                .collect();
            render_discuss_list(&items, archived, json)
        }
        DiscussCommands::Show { slug, json } => {
            let payload = ctx.client.show_discussion(&slug)?;
            render_discuss_show(
                &core::command::DiscussShowOutcome {
                    info: Some(to_discussion_info(&payload.info)),
                    content: payload.content,
                },
                json,
            )
        }
        DiscussCommands::New { topic, slug, kind, json } => {
            // --slug 與 --kind 隨請求上 wire；驗證的單一事實來源在引擎（server 端），
            // CLI 不預驗（design D1）。
            let resp = ctx.client.new_discussion(&topic, slug.as_deref(), kind.as_deref())?;
            if json {
                // remote 的 --json 契約是 wire 回應原樣（slug／topic／path）——
                // 組 core 型別會捏造 server 沒說的欄位，形狀凍結不允許。
                return print_json(&resp);
            }
            render_discuss_new_human(NewDiscussionLines {
                slug: &resp.slug,
                topic: &resp.topic,
                path: &resp.path,
            });
            Ok(())
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            ctx.client.discussion_context(&slug, &content)?;
            render_discuss_context(&slug, json)
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = read_stdin_content(stdin);
            let round = ctx.client.discussion_add_round(&slug, &mode, &content)?.round;
            render_discuss_add_round(&slug, round as usize, &mode, json)
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            let flagged = ctx.client.discussion_conclude(&slug, &content)?.restale_flagged;
            render_discuss_conclude(&slug, &flagged, json)
        }
        DiscussCommands::Archive { slug, json } => {
            let archived_to = ctx.client.discussion_archive(&slug)?.archived_to;
            render_discuss_archive(&slug, &archived_to, json)
        }
        DiscussCommands::Promote { slug, name, json } => {
            let change = ctx.client.discussion_promote(&slug, name.as_deref())?.change;
            // 明文分歧（design D5）：新變更目錄是 store 端位置，remote 不印，
            // 與 `new change` 的 Path 行同一條裁定。
            render_discuss_promote(&slug, &change, None, json)
        }
        DiscussCommands::Discard { slug, force, json } => {
            let slug = ctx.client.discard_discussion(&slug, force)?.slug;
            render_discuss_discard(&slug, json)
        }
        DiscussCommands::Link { slug, change, json } => {
            let bound = ctx.client.link_discussion(&slug, &change)?;
            render_discuss_bind(&bound.slug, &bound.change, DiscussBind::Link, json)
        }
        DiscussCommands::Seal { slug, change, json } => {
            let bound = ctx.client.seal_discussion(&slug, &change)?;
            render_discuss_bind(&bound.slug, &bound.change, DiscussBind::Seal, json)
        }
    }
}

/// The wire discussion summary reshaped into the engine's info type, so both
/// modes render (and serialize) it through one path.
fn to_discussion_info(d: &protocol_query::DiscussionInfo) -> core::discuss::DiscussionInfo {
    core::discuss::DiscussionInfo {
        slug: d.slug.clone(),
        topic: d.topic.clone(),
        status: d.status.clone(),
        rounds: d.rounds,
        created: d.created.clone(),
        created_by: d.created_by.clone(),
        kind: d.kind.clone(),
        path: d.path.clone(),
        archived: d.archived,
    }
}

// --- workflow-config ---

/// The document label in remote mode: the server holds one workflow-config
/// document per scope, with no local path to name.
const REMOTE_CONFIG_LABEL: &str = "config.yaml";

/// Remote workflow-config: read the server document (content plus the scope
/// revision), apply the SAME core rewrite fs mode uses, write back guarded by
/// the revision just read. The revision never reaches the command surface — a
/// CAS refusal simply means someone else wrote in the read→write window, and
/// re-running the command is the whole fix.
fn remote_workflow_config(
    ctx: &RemoteCtx,
    write: Option<(WorkflowConfigWrite, bool)>,
    json: bool,
) -> Result<()> {
    let current = ctx.client.config()?;
    let original = current.content.unwrap_or_default();
    let Some((write, dry_run)) = write else {
        return print_workflow_config(&original, REMOTE_CONFIG_LABEL, json);
    };
    let edit = plan_workflow_config_edit(&write, &original, REMOTE_CONFIG_LABEL, None)?;
    if dry_run {
        print!("{}", unified_diff(REMOTE_CONFIG_LABEL, &original, &edit.new_text));
        return Ok(());
    }
    ctx.client
        .put_config(&edit.new_text, current.revision)
        .map_err(remote_config_write_error)?;
    println!("{} {}", color::green("✓"), edit.summary);
    Ok(())
}

/// The CAS refusal restated in this verb's own terms — the generic
/// "re-read and re-apply" wording does not say what the user should do with a
/// single-shot command. Every other failure keeps its translated message.
fn remote_config_write_error(e: speclink_remote::RemoteError) -> anyhow::Error {
    if e.reason.as_deref() == Some("revision_conflict") {
        return anyhow::anyhow!(
            "the workflow config was updated by someone else — re-run this command to apply your change on top"
        );
    }
    anyhow::Error::new(e)
}
