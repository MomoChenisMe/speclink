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
use speclink_remote::client::Client as RemoteClient;

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
    let Some(token) = remote_auth::resolve_token(&origin) else {
        bail!("not logged in to {origin} — run `speclink auth login`");
    };
    // The binding handshake precedes every verb (fail closed): an
    // incompatible API version or a missing/ambiguous binding stops here —
    // no verb request leaves the client.
    let client = RemoteClient::new(&conn.url, &token, conn.repo.as_deref());
    let binding = client.handshake()?;
    // The confirmed repo identity rides every subsequent request: a declared
    // `remote.repo` keeps its value; an undeclared one adopts the server's
    // unambiguous binding.
    let client = if conn.repo.is_none() && !binding.repo.key.is_empty() {
        RemoteClient::new(&conn.url, &token, Some(&binding.repo.key))
    } else {
        client
    };
    Ok(Some(RemoteCtx { client }))
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

/// Phase-1 snapshot source over the existing verb contract: the change's own
/// artifacts (proposal/design/tasks) are fetchable today; delta and canonical
/// specs arrive with the Phase 2 Context API. The provider seam keeps that
/// upgrade out of the verb flow.
struct VerbContextProvider<'c> {
    client: &'c RemoteClient,
    change: String,
}

impl speclink_host::projection::SnapshotProvider for VerbContextProvider<'_> {
    fn snapshot(
        &self,
        _request: &speclink_protocol::context::ContextSnapshotRequest,
    ) -> Result<speclink_protocol::context::ContextSnapshot> {
        use speclink_host::projection::content_digest;
        use speclink_protocol::context::{ContextDocument, ContextSnapshot};
        let mut documents = Vec::new();
        for artifact in ["proposal", "design", "tasks"] {
            match self.client.get_artifact(&self.change, artifact) {
                Ok(got) => {
                    let digest = content_digest(&got.content);
                    documents.push(ContextDocument {
                        path: format!("openspec/changes/{}/{artifact}.md", self.change),
                        content: got.content,
                        revision: Some(got.version),
                        digest,
                    });
                }
                // An absent artifact is a normal shape (the schema may not
                // require it yet) — the projection simply omits it.
                Err(e) if e.reason.as_deref() == Some("not_found") => {}
                Err(e) => return Err(e.into()),
            }
        }
        let combined: Vec<&str> = documents.iter().map(|d| d.digest.as_str()).collect();
        let digest = content_digest(&combined.join("\n"));
        // Deterministic content-derived identity — the verb contract carries
        // no snapshot ids until the Phase 2 Context API.
        let snapshot_id = format!("verb-{}", &digest.trim_start_matches("sha256:")[..12]);
        Ok(ContextSnapshot { snapshot_id, policy_revision: None, digest, documents })
    }
}

/// The remote verb flow's materialize trigger: refresh the projection for
/// this change's apply flow and point contextFiles into it. Projection
/// trouble is a loud warning, never a verb failure — the instructions
/// payload itself is intact either way.
fn point_context_files_at_projection(
    ctx: &RemoteCtx,
    name: &str,
    context_files: &mut std::collections::BTreeMap<String, String>,
) {
    let Some(ws) = core::workspace::Workspace::discover_cwd().ok().flatten() else {
        return;
    };
    let provider = VerbContextProvider { client: &ctx.client, change: name.to_string() };
    let request = speclink_protocol::context::ContextSnapshotRequest {
        change: Some(name.to_string()),
        flow: Some("apply".to_string()),
    };
    match speclink_host::projection::materialize(&ws, &provider, &request) {
        Ok(out) => {
            for w in &out.warnings {
                eprintln!("speclink: warning: {w}");
            }
        }
        Err(e) => eprintln!("speclink: warning: context projection not refreshed: {e:#}"),
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

fn cmd_init_remote(a: &InitArgs, root: &std::path::Path, display_base: &str) -> Result<()> {
    let Some(url) = a.url.as_deref() else {
        bail!("--store remote requires --url <project-scoped url>");
    };
    if a.dir.is_some() {
        bail!("--dir has no meaning with --store remote (documents live on the server)");
    }
    let tools = match a.tools.as_deref() {
        Some(spec) => core::init::parse_tools(spec)?,
        None => core::init::detect_tools(root),
    };
    core::init::init_remote(root, &tools, a.force, url, a.repo.as_deref())?;
    println!("{} Initialized at {display_base} (remote store)", color::green("✓"));
    if !tools.is_empty() {
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        println!("Generated files for: {}", names.join(", "));
    }
    // Validate the declared repo now when credentials exist; defer otherwise
    // (offline init must not block — the first verb still validates).
    validate_or_defer(root, url, a.repo.as_deref())
}

fn cmd_link(a: LinkArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let origin = remote_auth::origin_of(&a.url);
    match remote_auth::resolve_token(&origin) {
        Some(token) => {
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
        AuthCommands::Login { token_stdin } => {
            let token = if token_stdin {
                read_stdin().trim().to_string()
            } else {
                eprint!("Paste your personal access token: ");
                read_stdin().trim().to_string()
            };
            if token.is_empty() {
                bail!("empty token — nothing stored");
            }
            // Validate before storing: a rejected token is never written.
            let whoami = RemoteClient::new(&conn.url, &token, None).whoami()?;
            remote_auth::save_token_at(&remote_auth::speclink_config_dir(), &origin, &token)?;
            print_identity(&whoami);
            Ok(())
        }
        AuthCommands::Status => {
            let Some(token) = remote_auth::resolve_token(&origin) else {
                bail!("Not logged in to {origin} — run `speclink auth login`");
            };
            let whoami = RemoteClient::new(&conn.url, &token, None).whoami()?;
            print_identity(&whoami);
            if let Some(repo) = conn.repo.as_deref() {
                match ensure_repo_registered(&whoami, repo) {
                    Ok(()) => println!("Repo '{repo}' is registered in this project"),
                    Err(e) => println!("! {e}"),
                }
            }
            git_reference_warning(&ws.root, conn.repo.as_deref(), &whoami);
            Ok(())
        }
    }
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
    match remote_auth::resolve_token(&origin) {
        Some(token) => {
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

// Destructive change removal stays host-governed in remote mode — the local-only
// verb does not cross the wire (mirrors `discuss discard`'s remote refusal).
fn remote_discard(_ctx: &RemoteCtx, _a: &DiscardArgs) -> Result<()> {
    bail!("discard is not available in remote mode — remove changes in the team system")
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
