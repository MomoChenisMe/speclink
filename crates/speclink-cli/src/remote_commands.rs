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

fn remote_list(ctx: &RemoteCtx, a: &ListArgs) -> Result<()> {
    if a.specs && !a.changes {
        return remote_list_specs(ctx, a.json);
    }
    let items = ctx.client.list_changes()?.changes;
    if a.json {
        let parity: Vec<ListChangeJson> = items
            .iter()
            .map(|c| ListChangeJson {
                completed_tasks: c.completed_tasks,
                name: c.name.clone(),
                status: c.status.clone(),
                summary: c.summary.clone(),
                total_tasks: c.total_tasks,
                restale_from: c.restale_from.clone(),
                meta_error: c.meta_error.clone(),
            })
            .collect();
        if a.specs {
            return print_json(&serde_json::json!({
                "changes": parity,
                "specs": remote_specs_parity(ctx)?,
            }));
        }
        return print_json(&serde_json::json!({ "changes": parity }));
    }
    if items.is_empty() {
        println!("No active changes.");
        if a.specs {
            println!();
            remote_list_specs(ctx, false)?;
        }
        return Ok(());
    }
    println!("{}", color::bold("Changes:"));
    for c in &items {
        let marker = if c.total_tasks > 0 {
            format!(" [{}/{}]", c.completed_tasks, c.total_tasks)
        } else {
            String::new()
        };
        let suffix = if c.summary.is_empty() {
            String::new()
        } else {
            format!(" — {}", c.summary)
        };
        println!("  {} {}{marker}{}", color::cyan("•"), c.name, color::dim(&suffix));
    }
    if a.specs {
        println!();
        remote_list_specs(ctx, false)?;
    }
    Ok(())
}

/// The parity view of the spec listing: id and path only.
fn remote_specs_parity(ctx: &RemoteCtx) -> Result<Vec<protocol_query::SpecSummary>> {
    Ok(ctx.client.list_specs()?.specs)
}

fn remote_list_specs(ctx: &RemoteCtx, json: bool) -> Result<()> {
    let specs = remote_specs_parity(ctx)?;
    if json {
        return print_json(&serde_json::json!({ "specs": specs }));
    }
    if specs.is_empty() {
        println!("No specs.");
        return Ok(());
    }
    println!("{}", color::bold("Specs:"));
    for s in &specs {
        println!("  {} {}", color::cyan("•"), s.id);
    }
    Ok(())
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
    println!("{} Created change: {}", color::green("✓"), a.name);
    if let Some(schema) = resp.schema.filter(|s| !s.is_empty()) {
        println!("  Schema: {schema}");
    }
    if let Some(slug) = &a.from_discussion {
        println!("  From discussion: {slug}");
    }
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
        .map(|w| core::tasks::git_changed_files(&w.root))
        .unwrap_or_default();
    let resp = ctx.client.task_done(&change, task_id, &touched)?;
    if resp.already_done {
        bail!("Task {task_id} is already done");
    }
    if json {
        // Compact single-line JSON with the fs-mode keys.
        let v = serde_json::json!({
            "change": change,
            "status": "done",
            "task_desc": resp.task_desc,
            "task_id": task_id,
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Task {task_id} marked as done: {}", color::green("✓"), resp.task_desc);
    Ok(())
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
    if resp.already_undone {
        bail!("Task {task_id} is already not done");
    }
    if json {
        // Compact single-line JSON with the fs-mode keys.
        let v = serde_json::json!({
            "change": change,
            "status": "undone",
            "task_desc": resp.task_desc,
            "task_id": task_id,
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Task {task_id} marked as not done: {}", color::green("✓"), resp.task_desc);
    Ok(())
}

fn remote_claim(ctx: &RemoteCtx, name: &str) -> Result<()> {
    let _ = ctx.client.claim(name)?;
    println!("{} Claimed change: {name}", color::green("✓"));
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
    let resp = ctx.client.archive(&name)?;
    println!("{} Archived change: {name}", color::green("✓"));
    if !resp.specs.is_empty() {
        let caps: Vec<&str> = resp.specs.iter().map(|s| s.capability.as_str()).collect();
        println!("  Specs updated: {}", caps.join(", "));
    }
    Ok(())
}

// --- discuss ---

fn remote_discuss(ctx: &RemoteCtx, a: DiscussArgs) -> Result<()> {
    match a.command {
        DiscussCommands::List { archived, json } => {
            let items = ctx.client.list_discussions(archived)?.discussions;
            if json {
                return print_json(&serde_json::json!({ "discussions": items }));
            }
            if items.is_empty() {
                let what = if archived { "archived discussions" } else { "discussions" };
                println!("No {what} found.");
                return Ok(());
            }
            let heading = if archived { "Archived discussions:" } else { "Discussions:" };
            println!("{heading}");
            for d in &items {
                println!("  • {} [{}] ({} rounds) — {}", d.slug, d.status, d.rounds, d.topic);
            }
            Ok(())
        }
        DiscussCommands::Show { slug, json } => {
            let payload = ctx.client.show_discussion(&slug)?;
            if json {
                return print_json(&serde_json::json!({
                    "info": payload.info,
                    "content": payload.content,
                }));
            }
            print!("{}", payload.content);
            Ok(())
        }
        DiscussCommands::New { topic, slug, json } => {
            // The remote API has no slug field yet — reject loudly rather than
            // silently dropping the override.
            if slug.is_some() {
                anyhow::bail!("--slug is not supported for remote discussions yet");
            }
            let resp = ctx.client.new_discussion(&topic)?;
            if json {
                return print_json(&resp);
            }
            println!("{} Created discussion: {}", color::green("✓"), resp.slug);
            println!("  Topic: {}", resp.topic);
            println!("  Path: {}", resp.path);
            Ok(())
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            ctx.client.discussion_context(&slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "context": "set" }));
            }
            println!("{} Set context for discussion '{slug}'", color::green("✓"));
            Ok(())
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = read_stdin_content(stdin);
            let round = ctx.client.discussion_add_round(&slug, &mode, &content)?.round;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "round": round, "mode": mode }));
            }
            println!("{} Recorded round {round} ({mode}) to discussion '{slug}'", color::green("✓"));
            Ok(())
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = read_stdin_content(stdin);
            ctx.client.discussion_conclude(&slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "status": "concluded" }));
            }
            println!("{} Concluded discussion '{slug}'", color::green("✓"));
            Ok(())
        }
        DiscussCommands::Archive { slug, json } => {
            let archived_to = ctx.client.discussion_archive(&slug)?.archived_to;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "archived_to": archived_to }));
            }
            println!("{} Archived discussion: {slug} → {archived_to}", color::green("✓"));
            Ok(())
        }
        DiscussCommands::Promote { slug, name, json } => {
            let change = ctx.client.discussion_promote(&slug, name.as_deref())?.change;
            if json {
                return print_json(&serde_json::json!({
                    "change": change,
                    "slug": slug,
                    "status": "promoted",
                }));
            }
            println!("{} Promoted discussion '{slug}' → change '{change}'", color::green("✓"));
            Ok(())
        }
        // Destructive discussion removal stays host-governed in remote mode
        // (contract §5.7) — the local-only verb does not cross the wire.
        DiscussCommands::Discard { .. } => {
            bail!("discuss discard is not available in remote mode — remove discussions in the team system")
        }
        // The remote verb contract has no link operation yet — the chain is
        // local change-metadata surgery until the contract grows one.
        DiscussCommands::Link { .. } => {
            bail!("discuss link is not available in remote mode yet — link the discussion locally")
        }
        // Seal marks the discussion promoted once content lands — same local
        // change-metadata surgery as link, with no remote contract op yet.
        DiscussCommands::Seal { .. } => {
            bail!("discuss seal is not available in remote mode yet — seal the discussion locally")
        }
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
