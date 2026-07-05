// Included into main.rs. Remote-mode detection and remote verb handlers.
//
// Routing rule: each dual-mode command checks `remote_ctx()` first; `None`
// falls through to the existing fs body untouched (fs behavior is the
// regression-protected baseline). Handlers here re-shape server payloads
// into the exact fs-parity stdout (human and --json) — server extras like
// `repo`/`lifecycle` never leak into the parity view.

use speclink_remote::auth as remote_auth;
use speclink_remote::client::Client as RemoteClient;

struct RemoteCtx {
    client: RemoteClient,
}

/// Resolve remote mode for the current workspace. `Ok(None)` = fs mode
/// (including "no workspace at all" — the fs path owns that error).
fn remote_ctx() -> Result<Option<RemoteCtx>> {
    let Some(ws) = core::workspace::Workspace::discover_cwd() else {
        return Ok(None);
    };
    let resolution = ws.resolve_mode()?;
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
    Ok(Some(RemoteCtx {
        client: RemoteClient::new(&conn.url, &token, conn.repo.as_deref()),
    }))
}

/// A 2xx body that doesn't match the contract shape is a client/server
/// version skew, reported like any other unexpected response.
fn remote_shape_err(e: serde_json::Error) -> anyhow::Error {
    anyhow::anyhow!("unexpected server response — update speclink or report a bug ({e})")
}

fn v_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}

fn v_usize(v: &serde_json::Value, key: &str) -> usize {
    v.get(key).and_then(|x| x.as_u64()).unwrap_or(0) as usize
}

fn v_array(v: &serde_json::Value, key: &str) -> Vec<serde_json::Value> {
    v.get(key)
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default()
}

// --- list ---

fn remote_list(ctx: &RemoteCtx, a: &ListArgs) -> Result<()> {
    if a.specs && !a.changes {
        return remote_list_specs(ctx, a.json);
    }
    let payload = ctx.client.list_changes()?;
    let items = v_array(&payload, "changes");
    if a.json {
        let parity: Vec<ListChangeJson> = items
            .iter()
            .map(|c| ListChangeJson {
                completed_tasks: v_usize(c, "completedTasks"),
                name: v_str(c, "name"),
                status: v_str(c, "status"),
                summary: v_str(c, "summary"),
                total_tasks: v_usize(c, "totalTasks"),
            })
            .collect();
        if a.specs {
            let mut out = serde_json::Map::new();
            out.insert("changes".into(), serde_json::to_value(&parity)?);
            out.insert("specs".into(), remote_specs_value(ctx)?);
            return print_json(&serde_json::Value::Object(out));
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
        let complete = v_usize(c, "completedTasks");
        let total = v_usize(c, "totalTasks");
        let summary = v_str(c, "summary");
        let marker = if total > 0 {
            format!(" [{complete}/{total}]")
        } else {
            String::new()
        };
        let suffix = if summary.is_empty() {
            String::new()
        } else {
            format!(" — {summary}")
        };
        println!("  {} {}{marker}{}", color::cyan("•"), v_str(c, "name"), color::dim(&suffix));
    }
    if a.specs {
        println!();
        remote_list_specs(ctx, false)?;
    }
    Ok(())
}

fn remote_specs_value(ctx: &RemoteCtx) -> Result<serde_json::Value> {
    let payload = ctx.client.list_specs()?;
    let items = v_array(&payload, "specs");
    Ok(serde_json::Value::Array(
        items
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": v_str(s, "id"),
                    "path": v_str(s, "path"),
                })
            })
            .collect(),
    ))
}

fn remote_list_specs(ctx: &RemoteCtx, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({ "specs": remote_specs_value(ctx)? }));
    }
    let payload = ctx.client.list_specs()?;
    let items = v_array(&payload, "specs");
    if items.is_empty() {
        println!("No specs.");
        return Ok(());
    }
    println!("{}", color::bold("Specs:"));
    for s in &items {
        println!("  {} {}", color::cyan("•"), v_str(s, "id"));
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
    let payload = ctx.client.list_changes()?;
    let mut names: Vec<String> = v_array(&payload, "changes")
        .iter()
        .map(|c| v_str(c, "name"))
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

fn remote_status(ctx: &RemoteCtx, a: &StatusArgs) -> Result<()> {
    if a.schema.is_some() {
        bail!("--schema is not supported in remote mode — the server's workflow config decides the schema");
    }
    let Some(name) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    let payload = ctx.client.get_change(&name)?;
    // Deserializing into the fs report type keeps stdout byte-identical to fs
    // mode; server extras (repo, lifecycle, versions) are dropped by serde.
    let report: core::status::StatusReport =
        serde_json::from_value(payload).map_err(remote_shape_err)?;
    if a.json {
        return print_json(&report);
    }
    render_status_human(&report);
    Ok(())
}

// --- instructions ---

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
        None => {
            let status_payload = ctx.client.get_change(&name)?;
            let report: core::status::StatusReport =
                serde_json::from_value(status_payload).map_err(remote_shape_err)?;
            report
                .artifacts
                .iter()
                .find(|x| x.status != "done")
                .map(|x| x.id.clone())
                .unwrap_or_else(|| "apply".to_string())
        }
    };
    let payload = ctx.client.instructions(&name, &artifact)?;
    if artifact == "apply" {
        let p: core::instructions::ApplyInstructions =
            serde_json::from_value(payload).map_err(remote_shape_err)?;
        if a.json {
            return print_json(&p);
        }
        render_apply_human(&p);
    } else {
        let p: core::instructions::ArtifactInstructions =
            serde_json::from_value(payload).map_err(remote_shape_err)?;
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
    let root = core::workspace::Workspace::discover_cwd()
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
    let Some(ws) = core::workspace::Workspace::discover_cwd() else {
        bail!("Not connected to a remote store — run `speclink link <url>` first");
    };
    let conn = match ws.resolve_mode()?.mode {
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

fn print_identity(whoami: &serde_json::Value) {
    let user = whoami.get("user").cloned().unwrap_or_default();
    let name = v_str(&user, "name");
    let handle = v_str(&user, "handle");
    if handle.is_empty() {
        println!("{} Logged in as {name}", color::green("✓"));
    } else {
        println!("{} Logged in as {name} (@{handle})", color::green("✓"));
    }
}

/// Check the declared repo against the server's registry (`whoami.repos[]`).
fn ensure_repo_registered(whoami: &serde_json::Value, repo: &str) -> Result<()> {
    let repos = v_array(whoami, "repos");
    let names: Vec<String> = repos.iter().map(|r| v_str(r, "name")).collect();
    if names.iter().any(|n| n == repo) {
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
fn git_reference_warning(root: &std::path::Path, repo: Option<&str>, whoami: &serde_json::Value) {
    let Some(repo) = repo else { return };
    let reference = v_array(whoami, "repos")
        .iter()
        .find(|r| v_str(r, "name") == repo)
        .map(|r| v_str(r, "gitUrl"))
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
    let payload = ctx.client.get_artifact(&change, artifact)?;
    print!("{}", v_str(&payload, "content"));
    Ok(())
}

// --- write path: changes ---

fn remote_new_change(ctx: &RemoteCtx, a: &NewChangeArgs) -> Result<()> {
    let mut body = serde_json::json!({ "name": a.name });
    if let Some(s) = &a.schema {
        body["schema"] = serde_json::json!(s);
    }
    if let Some(d) = &a.description {
        body["description"] = serde_json::json!(d);
    }
    if let Some(agent) = &a.agent {
        body["agent"] = serde_json::json!(agent);
    }
    if let Some(slug) = &a.from_discussion {
        body["fromDiscussion"] = serde_json::json!(slug);
    }
    let resp = ctx.client.create_change(body)?;
    println!("{} Created change: {}", color::green("✓"), a.name);
    let schema = v_str(&resp, "schema");
    if !schema.is_empty() {
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
        let schema_name = v_str(&ctx.client.config()?, "schema");
        let name = if schema_name.is_empty() { "spec-driven".to_string() } else { schema_name };
        match core::schema::resolve_with(None, &name) {
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
        match ctx.client.get_artifact(&change, &artifact_path) {
            Ok(v) => v.get("version").and_then(|x| x.as_u64()).unwrap_or(0),
            Err(_) => 0,
        }
    } else {
        0
    };
    let resp = ctx.client.put_artifact(&change, &artifact_path, &content, version)?;
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
    let _ = resp;
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
    let ws = core::workspace::Workspace::discover_cwd();
    let touched: Vec<String> = ws
        .map(|w| core::tasks::git_changed_files(&w.root))
        .unwrap_or_default();
    let resp = ctx.client.task_done(&change, task_id, &touched)?;
    let desc = v_str(&resp, "taskDesc");
    if resp.get("alreadyDone").and_then(|v| v.as_bool()).unwrap_or(false) {
        bail!("Task {task_id} is already done");
    }
    if json {
        // Compact single-line JSON with the fs-mode keys.
        let v = serde_json::json!({
            "change": change,
            "status": "done",
            "task_desc": desc,
            "task_id": task_id,
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Task {task_id} marked as done: {desc}", color::green("✓"));
    Ok(())
}

fn remote_claim(ctx: &RemoteCtx, name: &str) -> Result<()> {
    let resp = ctx.client.claim(name)?;
    let _ = resp;
    println!("{} Claimed change: {name}", color::green("✓"));
    Ok(())
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
    let specs = v_array(&resp, "specs");
    if !specs.is_empty() {
        let caps: Vec<String> = specs.iter().map(|s| v_str(s, "capability")).collect();
        println!("  Specs updated: {}", caps.join(", "));
    }
    Ok(())
}

// --- discuss ---

fn remote_discuss(ctx: &RemoteCtx, a: DiscussArgs) -> Result<()> {
    match a.command {
        DiscussCommands::List { archived, json } => {
            let payload = ctx.client.list_discussions(archived)?;
            let items: Vec<core::discuss::DiscussionInfo> = serde_json::from_value(
                payload.get("discussions").cloned().unwrap_or_else(|| serde_json::json!([])),
            )
            .map_err(remote_shape_err)?;
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
                let info = payload.get("info").cloned().unwrap_or(serde_json::Value::Null);
                let content = v_str(&payload, "content");
                return print_json(&serde_json::json!({ "info": info, "content": content }));
            }
            print!("{}", v_str(&payload, "content"));
            Ok(())
        }
        DiscussCommands::New { topic, json } => {
            let resp = ctx.client.new_discussion(&topic)?;
            if json {
                return print_json(&resp);
            }
            println!("{} Created discussion: {}", color::green("✓"), v_str(&resp, "slug"));
            println!("  Topic: {}", v_str(&resp, "topic"));
            println!("  Path: {}", v_str(&resp, "path"));
            Ok(())
        }
        DiscussCommands::Context { slug, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            ctx.client.discussion_context(&slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "context": "set" }));
            }
            println!("{} Set context for discussion '{slug}'", color::green("✓"));
            Ok(())
        }
        DiscussCommands::AddRound { slug, mode, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            let resp = ctx.client.discussion_add_round(&slug, &mode, &content)?;
            let round = resp.get("round").and_then(|v| v.as_u64()).unwrap_or(0);
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "round": round, "mode": mode }));
            }
            println!("{} Recorded round {round} ({mode}) to discussion '{slug}'", color::green("✓"));
            Ok(())
        }
        DiscussCommands::Conclude { slug, stdin, json } => {
            let content = if stdin { read_stdin() } else { String::new() };
            ctx.client.discussion_conclude(&slug, &content)?;
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "status": "concluded" }));
            }
            println!("{} Concluded discussion '{slug}'", color::green("✓"));
            Ok(())
        }
        DiscussCommands::Archive { slug, json } => {
            let resp = ctx.client.discussion_archive(&slug)?;
            let archived_to = v_str(&resp, "archivedTo");
            if json {
                return print_json(&serde_json::json!({ "slug": slug, "archived_to": archived_to }));
            }
            println!("{} Archived discussion: {slug} → {archived_to}", color::green("✓"));
            Ok(())
        }
        DiscussCommands::Promote { slug, name, json } => {
            let resp = ctx.client.discussion_promote(&slug, name.as_deref())?;
            let change = v_str(&resp, "change");
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
    }
}
