//! Remote-mode plumbing shared by every family's remote arm.
//!
//! The handshake lives here and is reached only from dispatch's mode shapes
//! (design D3): `remote_ctx()` resolves the store mode once, and the families
//! receive the already-connected client.

use anyhow::{bail, Result};
use speclink_core as core;
use speclink_remote::auth as remote_auth;
use speclink_remote::client::Client as RemoteClient;

use crate::color;
use speclink_protocol::query as protocol_query;

pub(crate) struct RemoteCtx {
    pub(crate) client: RemoteClient,
}

/// Resolve remote mode for the current workspace. `Ok(None)` = fs mode
/// (including "no workspace at all" — the fs path owns that error).
pub(crate) fn remote_ctx() -> Result<Option<RemoteCtx>> {
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

/// Explicit name passes through; otherwise a single active change
/// auto-selects, several is an error, and none prints the info line and
/// returns `Ok(None)` (exit 0, matching fs `info_if_no_changes`).
pub(crate) fn remote_resolve_change(
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

// --- repo 歸屬驗證：init 的 remote 初始化與 connection 的 link／auth 共用 ---

/// Check the declared repo against the server's registry (`whoami.repos[]`).
pub(crate) fn ensure_repo_registered(whoami: &protocol_query::WhoamiResponse, repo: &str) -> Result<()> {
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
pub(crate) fn validate_or_defer(root: &std::path::Path, url: &str, repo: Option<&str>) -> Result<()> {
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
pub(crate) fn git_reference_warning(
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
