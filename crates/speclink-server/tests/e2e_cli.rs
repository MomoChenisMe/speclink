//! End-to-end onboarding: the real CLI binary against the real server over
//! SQLite, driven from a fresh database through the whole /setup flow
//! (server-setup spec「setup 流程完成開箱四要素」/「完成 setup 即可邀請與連線」,
//! 決策 3/4/5). An operator takes the one-time token off the server's stdout and
//! completes /setup (first admin + first project/repo); mints an invitation with
//! the `invite` subcommand; the invitee walks the web forms to set a password,
//! log in and create a PAT; that PAT configures the real CLI, whose remote verbs
//! match fs mode; a restart confirms /setup is closed and the data persists; and
//! revoking the PAT makes the very next CLI call fail authentication.

use speclink_protocol::command::CreateChangeRequest;
use speclink_protocol::context::ContextSnapshotRequest;
use speclink_remote::client::{Client, ContextSnapshotOutcome};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};

const PROPOSAL: &str = "## Why\n\nDemo change.\n\n## What Changes\n\n- a thing\n";
const DESIGN: &str = "## Context\n\nDemo design.\n";
const TASKS: &str = "## 1. Work\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n";
const SPEC: &str = "## ADDED Requirements\n\n### Requirement: Demo\nDemo SHALL work.\n\n#### Scenario: works\n- **WHEN** run\n- **THEN** ok\n";
const CANON_SPEC: &str = "### Requirement: Cap A\nCap A SHALL work.\n";
const LANGUAGE_DOC: &str = "# Shared Vocabulary\n\n- Change: a proposed edit.\n";

const EMAIL: &str = "dev@example.com";
const DISPLAY: &str = "E2E <e2e@example.com>";
const PASSWORD: &str = "e2e-correct-horse";

// The first admin, created through /setup (not via invitation).
const ADMIN_EMAIL: &str = "root@example.com";
const ADMIN_DISPLAY: &str = "Root <root@example.com>";
const ADMIN_PASSWORD: &str = "root-correct-horse";

// --- binaries ---

static BUILD_CLI: Once = Once::new();

/// Build the CLI binary once, then return its path (a sibling of the server
/// binary in the same target profile).
fn cli_bin() -> PathBuf {
    BUILD_CLI.call_once(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .args(["build", "-p", "speclink-cli", "--bin", "speclink"])
            .status()
            .expect("spawn cargo build for the CLI");
        assert!(status.success(), "building the speclink CLI failed");
    });
    let server = PathBuf::from(env!("CARGO_BIN_EXE_speclink-server"));
    let dir = server.parent().expect("target dir");
    let exe = if cfg!(windows) { "speclink.exe" } else { "speclink" };
    dir.join(exe)
}

fn server_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_speclink-server"))
}

// --- a running server child, killed on drop ---

struct Server {
    child: Child,
    addr: String,
    /// The server's captured stdout lines — where the one-time setup token
    /// appears on a fresh start.
    stdout: Arc<Mutex<Vec<String>>>,
}

impl Server {
    /// Start the server binary with `config` bound to a free loopback port,
    /// capturing stdout, and wait until `/healthz` answers.
    fn start(config: &Path) -> Server {
        let port = free_port();
        let addr = format!("127.0.0.1:{port}");
        let mut child = Command::new(server_bin())
            .args(["--config", config.to_str().unwrap(), "--addr", &addr])
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn speclink-server");
        let out = child.stdout.take().expect("piped stdout");
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let sink = stdout.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                sink.lock().expect("stdout lock").push(line);
            }
        });
        let server = Server { child, addr, stdout };
        server.wait_ready();
        server
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn project_url(&self) -> String {
        format!("{}/api/speclink/v1/projects/demo", self.base())
    }

    fn wait_ready(&self) {
        let url = format!("{}/healthz", self.base());
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if ureq::get(&url).call().map(|r| r.status() == 200).unwrap_or(false) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("server did not become ready at {url}");
    }

    /// The one-time setup token parsed from the `/setup?token=…` guidance line
    /// the server prints on a fresh start.
    fn setup_token(&self) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(token) = self
                .stdout
                .lock()
                .expect("stdout lock")
                .iter()
                .find_map(|line| parse_setup_token(line))
            {
                return token;
            }
            if Instant::now() > deadline {
                panic!("no setup token on stdout: {:?}", self.stdout.lock().expect("stdout lock"));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Whether the server printed a setup-token line (a fresh start does; a
    /// restart over a completed database does not).
    fn printed_setup_token(&self) -> bool {
        self.stdout.lock().expect("stdout lock").iter().any(|l| parse_setup_token(l).is_some())
    }
}

/// Extract the `token=` value from a `/setup?token=…` guidance line.
fn parse_setup_token(line: &str) -> Option<String> {
    if !line.contains("/setup") {
        return None;
    }
    let token: String = line
        .split("token=")
        .nth(1)?
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    (!token.is_empty()).then_some(token)
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind free port")
        .local_addr()
        .unwrap()
        .port()
}

// --- config ---

/// Write a server config: SQLite store and SQLite identity in `dir`, a `demo`
/// project, and no bootstrap tokens (retired).
fn write_config(dir: &Path, db: &Path) -> PathBuf {
    let path = dir.join("server.yaml");
    let identity_db = dir.join("identity.db");
    let mut file = std::fs::File::create(&path).expect("create config");
    write!(
        file,
        "store:\n  driver: sqlite\n  path: {}\nidentity:\n  driver: sqlite\n  path: {}\n",
        db.display(),
        identity_db.display()
    )
    .expect("write config");
    path
}

// --- identity flow over the web (invite → accept → login → PAT) ---

/// A ureq agent that does not follow redirects.
fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

/// Run the `invite` subcommand against `config` and return the one-time token
/// parsed from the printed URL.
fn invite(config: &Path) -> String {
    let out = Command::new(server_bin())
        .arg("invite")
        .args(["--config", config.to_str().unwrap()])
        .args(["--email", EMAIL, "--display", DISPLAY, "--project", "demo"])
        .output()
        .expect("run invite subcommand");
    assert!(out.status.success(), "invite failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .split_whitespace()
        .find(|w| w.contains("/invite/"))
        .and_then(|url| url.rsplit("/invite/").next())
        .unwrap_or_else(|| panic!("invite printed a URL: {stdout}"))
        .trim()
        .to_string()
}

/// Walk the web forms: accept the invitation, log in, create a PAT. Returns the
/// PAT plaintext and the session cookie value (for the later revoke).
fn create_pat_via_web(base: &str, token: &str) -> (String, String) {
    let http = agent();

    // Accept the invitation (set the password) — creates the active user.
    let accept = http
        .post(&format!("{base}/invite/{token}"))
        .send_form(&[("password", PASSWORD)])
        .expect("accept invitation");
    assert!((300..400).contains(&accept.status()), "accepting the invitation succeeds");

    // Log in.
    let login = http
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login");
    assert!((300..400).contains(&login.status()), "login succeeds");
    let session = login
        .header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("a session cookie")
        .to_string();

    // Create a PAT; the plaintext is shown once in the response.
    let created = http
        .post(&format!("{base}/account/tokens"))
        .set("Cookie", &format!("speclink_session={session}"))
        .send_form(&[("name", "cli"), ("expires", "")])
        .expect("create PAT");
    let body = created.into_string().unwrap_or_default();
    let pat = body
        .match_indices("spk_pat_")
        .map(|(i, _)| {
            body[i..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect::<String>()
        })
        .find(|t| t.len() == 72)
        .unwrap_or_else(|| panic!("PAT plaintext in response: {body}"));
    (pat, session)
}

/// Revoke the (single) PAT via the account page.
fn revoke_pat(base: &str, session: &str) {
    let http = agent();
    let cookie = format!("speclink_session={session}");
    let account = http
        .get(&format!("{base}/account"))
        .set("Cookie", &cookie)
        .call()
        .expect("load account page")
        .into_string()
        .unwrap_or_default();
    let marker = "/account/tokens/";
    let start = account.find(marker).expect("a revoke form") + marker.len();
    let pat_id = &account[start..][..account[start..].find("/revoke").expect("revoke suffix")];
    let resp = http
        .post(&format!("{base}/account/tokens/{pat_id}/revoke"))
        .set("Cookie", &cookie)
        .send_form(&[])
        .expect("revoke PAT");
    assert!((200..400).contains(&resp.status()), "revoke succeeds");
}

// --- store seeding, project layout ---

/// Seed change `demo` through the command routes with the typed client using
/// `pat`.
fn seed(project_url: &str, pat: &str) {
    let client = Client::new(project_url, pat, Some("backend"));
    client
        .create_change(CreateChangeRequest {
            name: "demo".to_string(),
            schema: Some("spec-driven".to_string()),
            description: None,
            agent: None,
            from_discussion: None,
        })
        .expect("create change");
    for (artifact, content) in [
        ("proposal", PROPOSAL),
        ("design", DESIGN),
        ("tasks", TASKS),
        ("specs/cap-a", SPEC),
    ] {
        client
            .put_artifact("demo", artifact, content, 0)
            .unwrap_or_else(|e| panic!("put {artifact}: {e:?}"));
    }
    client
        .new_discussion("Rate limiting approach")
        .expect("seed discussion");
}

/// A CLI project directory in `remote` mode pointed at `project_url`.
fn remote_project(dir: &Path, project_url: &str) -> PathBuf {
    let project = dir.join("remote");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join(".speclink.yaml"),
        format!("remote:\n  url: {project_url}\n  repo: backend\n"),
    )
    .unwrap();
    project
}

/// An fs-mode project directory holding the same change content — the shape
/// authority the remote output is compared against.
fn fs_project(dir: &Path) -> PathBuf {
    let project = dir.join("fs");
    let change = project.join("openspec").join("changes").join("demo");
    std::fs::create_dir_all(change.join("specs").join("cap-a")).unwrap();
    std::fs::write(change.join(".openspec.yaml"), "schema: spec-driven\n").unwrap();
    std::fs::write(change.join("proposal.md"), PROPOSAL).unwrap();
    std::fs::write(change.join("design.md"), DESIGN).unwrap();
    std::fs::write(change.join("tasks.md"), TASKS).unwrap();
    std::fs::write(change.join("specs").join("cap-a").join("spec.md"), SPEC).unwrap();
    project
}

/// Parse apply-instructions JSON and drop the fields whose values differ by
/// mode (local projection paths and the fs-only preflight block), leaving the
/// store-determined content for comparison.
fn content_only(stdout: &[u8]) -> serde_json::Value {
    let mut value: serde_json::Value =
        serde_json::from_slice(stdout).expect("apply stdout is JSON");
    if let Some(obj) = value.as_object_mut() {
        obj.remove("changeDir");
        obj.remove("contextFiles");
        obj.remove("preflight");
    }
    value
}

fn run_cli(project: &Path, args: &[&str], token: Option<&str>) -> Output {
    let mut cmd = Command::new(cli_bin());
    cmd.args(args)
        .current_dir(project)
        .env_remove("SPECLINK_STORE_URL")
        .env_remove("SPECLINK_TOKEN");
    if let Some(t) = token {
        cmd.env("SPECLINK_TOKEN", t);
    }
    cmd.output().expect("run speclink CLI")
}

/// Walk the /setup flow with the one-time token: create the first admin, then
/// register the first project (`demo`) and repo (`backend`). After this the
/// token is consumed and /setup is closed.
fn complete_setup(base: &str, token: &str) {
    let http = agent();
    let admin = http
        .post(&format!("{base}/setup?token={token}"))
        .send_form(&[("email", ADMIN_EMAIL), ("display", ADMIN_DISPLAY), ("password", ADMIN_PASSWORD)])
        .expect("setup: create the first admin");
    assert_eq!(admin.status(), 200, "the setup admin section succeeds");
    let project = http
        .post(&format!("{base}/setup?token={token}"))
        .send_form(&[
            ("project_key", "demo"),
            ("project_name", "Demo"),
            ("repo_key", "backend"),
            ("repo_name", "Backend"),
        ])
        .expect("setup: register the first project/repo");
    assert_eq!(project.status(), 200, "the setup project/repo section succeeds");
}

/// GET `url` and return its HTTP status, treating a protocol error as its code.
fn get_status(url: &str) -> u16 {
    match ureq::get(url).call() {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("transport error for {url}: {e}"),
    }
}

// --- the e2e ---

#[test]
fn setup_flow_onboards_a_team_and_a_restart_closes_it() {
    let workdir = tempfile::tempdir().expect("workdir");
    let db = workdir.path().join("store.db");
    let config = write_config(workdir.path(), &db);

    // A fresh server prints a one-time setup token; complete /setup to create the
    // first admin and register the first project/repo (開箱四要素).
    let server = Server::start(&config);
    let setup_token = server.setup_token();
    complete_setup(&server.base(), &setup_token);
    assert_eq!(
        get_status(&format!("{}/setup?token={}", server.base(), setup_token)),
        404,
        "/setup closes once setup completes",
    );

    // Invite a member into the just-registered project, then walk the web flow to
    // a PAT (完成 setup 即可邀請與連線).
    let token = invite(&config);
    let (pat, session) = create_pat_via_web(&server.base(), &token);

    // Seed the store with that PAT, then run remote verbs with it.
    seed(&server.project_url(), &pat);
    let remote = remote_project(workdir.path(), &server.project_url());
    let fs = fs_project(workdir.path());

    let list = run_cli(&remote, &["list", "--json"], Some(&pat));
    assert!(list.status.success(), "remote list failed: {}", String::from_utf8_lossy(&list.stderr));
    assert!(
        String::from_utf8_lossy(&list.stdout).contains("demo"),
        "remote list names the seeded change: {}",
        String::from_utf8_lossy(&list.stdout)
    );

    // Byte-identical parity against fs mode (fs is the shape authority).
    let remote_status = run_cli(&remote, &["status", "--change", "demo", "--json"], Some(&pat));
    let fs_status = run_cli(&fs, &["status", "--change", "demo", "--json"], None);
    assert!(
        remote_status.status.success() && fs_status.status.success(),
        "status runs on both paths (remote stderr: {}) (fs stderr: {})",
        String::from_utf8_lossy(&remote_status.stderr),
        String::from_utf8_lossy(&fs_status.stderr),
    );
    assert_eq!(
        remote_status.stdout, fs_status.stdout,
        "remote status --json is byte-identical to fs mode",
    );

    let remote_apply = run_cli(&remote, &["instructions", "apply", "--change", "demo", "--json"], Some(&pat));
    let fs_apply = run_cli(&fs, &["instructions", "apply", "--change", "demo", "--json"], None);
    assert!(remote_apply.status.success() && fs_apply.status.success(), "apply runs on both paths");
    assert_eq!(
        content_only(&remote_apply.stdout),
        content_only(&fs_apply.stdout),
        "remote and fs apply agree on the store-determined content",
    );

    // Discussions replay end to end.
    let discuss = run_cli(&remote, &["discuss", "list", "--json"], Some(&pat));
    assert!(discuss.status.success(), "remote discuss list failed: {}", String::from_utf8_lossy(&discuss.stderr));
    assert!(
        String::from_utf8_lossy(&discuss.stdout).contains("Rate limiting approach"),
        "remote discuss list names the seeded discussion",
    );

    // Restart over the same database: /setup stays closed (no token printed, 404)
    // and the seeded data persists — the member's PAT still lists the change
    // (重啟確認關門且資料完整).
    drop(server);
    let restarted = Server::start(&config);
    assert!(!restarted.printed_setup_token(), "a restart over a completed database prints no setup token");
    assert_eq!(get_status(&format!("{}/setup", restarted.base())), 404, "/setup stays closed after restart");
    let restarted_remote = remote_project(workdir.path(), &restarted.project_url());
    let after_restart = run_cli(&restarted_remote, &["list", "--json"], Some(&pat));
    assert!(
        after_restart.status.success(),
        "the PAT still authenticates after restart: {}",
        String::from_utf8_lossy(&after_restart.stderr),
    );
    assert!(
        String::from_utf8_lossy(&after_restart.stdout).contains("demo"),
        "the seeded change persists across the restart",
    );

    // Revoke the PAT: the very next CLI call fails authentication (401).
    revoke_pat(&restarted.base(), &session);
    let after = run_cli(&restarted_remote, &["list", "--json"], Some(&pat));
    assert!(!after.status.success(), "the revoked PAT no longer authorizes CLI calls");
    let stderr = String::from_utf8_lossy(&after.stderr);
    assert!(
        stderr.contains("authentication failed"),
        "the revoked call maps to the 401 authentication message: {stderr}"
    );
}

// --- Context API projection e2e (server-context-api / context-projection) ---

/// Seed the store scope directly, before the server opens it, with documents
/// the command routes cannot create: a canonical spec, the workflow config, and
/// the LANGUAGE document (the store's new shared-vocabulary kind). Dropping the
/// store closes the connection before the server starts.
fn preseed_scope(db: &Path) {
    use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
    let store = speclink_store_sqlite::SqliteTeamStore::open(db).expect("open store for preseed");
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "preseed".into(), actor: "preseed".into() },
        )
        .expect("begin preseed uow");
    uow.create(DocumentId::CanonicalSpec { capability: "cap-a".into() }, CANON_SPEC);
    uow.create(DocumentId::WorkflowConfig, "schema: spec-driven\n");
    uow.create(DocumentId::Language, LANGUAGE_DOC);
    store.commit(uow, Vec::new()).expect("preseed commit");
}

/// The snapshot id recorded in a projection's manifest.
fn manifest_snapshot_id(projection: &Path) -> String {
    let text = std::fs::read_to_string(projection.join("manifest.json")).expect("manifest.json");
    let json: serde_json::Value = serde_json::from_str(&text).expect("manifest json");
    json["snapshotId"].as_str().expect("snapshotId string").to_string()
}

#[test]
fn remote_apply_materializes_a_consistent_context_projection_from_the_real_server() {
    let workdir = tempfile::tempdir().expect("workdir");
    let db = workdir.path().join("store.db");

    // Seed the scope with the canonical spec / config / LANGUAGE the command
    // routes cannot create, then bring the server up over the same database.
    preseed_scope(&db);
    let config = write_config(workdir.path(), &db);
    let server = Server::start(&config);
    complete_setup(&server.base(), &server.setup_token());
    let (pat, _session) = create_pat_via_web(&server.base(), &invite(&config));

    // Seed the change (artifacts + a delta spec on cap-a) through the API.
    seed(&server.project_url(), &pat);
    let remote = remote_project(workdir.path(), &server.project_url());
    let projection = remote
        .canonicalize()
        .expect("canonicalize remote dir")
        .join(".speclink")
        .join("context");

    // Run the apply-stage verb: it materializes the projection from one Context
    // API snapshot.
    let apply = run_cli(&remote, &["instructions", "apply", "--change", "demo", "--json"], Some(&pat));
    assert!(apply.status.success(), "remote apply failed: {}", String::from_utf8_lossy(&apply.stderr));

    // The projection mirrors the change's artifacts, its delta spec, and the
    // matching canonical spec (apply flow narrows to the delta's capability),
    // plus INDEX and manifest.
    for rel in [
        "openspec/changes/demo/proposal.md",
        "openspec/changes/demo/design.md",
        "openspec/changes/demo/tasks.md",
        "openspec/changes/demo/specs/cap-a/spec.md",
        "openspec/specs/cap-a/spec.md",
        "INDEX.md",
        "manifest.json",
    ] {
        assert!(projection.join(rel).is_file(), "{rel} is in the projection");
    }

    // The manifest snapshot id is the server's real scope token, and the whole
    // projection verifies fail-closed.
    let ws = speclink_core::workspace::Workspace {
        root: remote.canonicalize().expect("canonicalize remote dir"),
        spec_dir_name: "openspec".to_string(),
    };
    speclink_host::projection::verify_projection(&ws).expect("the projection verifies");

    // The server's change-narrowed snapshot carries config and LANGUAGE (the new
    // store kind), and its id matches the materialized manifest.
    let client = Client::new(&server.project_url(), &pat, Some("backend"));
    let request = ContextSnapshotRequest { change: Some("demo".to_string()), flow: None };
    let snapshot = match client.context_snapshot(&request, None).expect("context snapshot") {
        ContextSnapshotOutcome::Fresh(s) => s,
        ContextSnapshotOutcome::Unchanged => panic!("a fresh request is never unchanged"),
    };
    let doc_paths: Vec<&str> = snapshot.documents.iter().map(|d| d.path.as_str()).collect();
    assert!(doc_paths.contains(&"openspec/config.yaml"), "server response carries config: {doc_paths:?}");
    assert!(doc_paths.contains(&"openspec/LANGUAGE.md"), "server response carries LANGUAGE: {doc_paths:?}");
    let id_before = manifest_snapshot_id(&projection);
    assert_eq!(id_before, snapshot.snapshot_id, "the manifest id is the server's snapshot id");

    // contextFiles: every value is a path under the projection that exists (glob
    // values excluded — they are patterns, not single files).
    let payload: serde_json::Value = serde_json::from_slice(&apply.stdout).expect("apply json");
    let files = payload["contextFiles"].as_object().expect("contextFiles object");
    for (key, value) in files {
        let value = value.as_str().unwrap();
        assert!(PathBuf::from(value).starts_with(&projection), "{key} points into the projection: {value}");
        if !value.contains('*') {
            assert!(PathBuf::from(value).is_file(), "{key} exists under the projection: {value}");
        }
    }

    // Repeating the same verb with no commit in between does not rewrite the
    // projection: a sentinel dropped into the directory survives (免重寫).
    let sentinel = projection.join("SENTINEL");
    std::fs::write(&sentinel, "probe").unwrap();
    let again = run_cli(&remote, &["instructions", "apply", "--change", "demo", "--json"], Some(&pat));
    assert!(again.status.success(), "second apply failed: {}", String::from_utf8_lossy(&again.stderr));
    assert!(sentinel.is_file(), "an unchanged scope skips the rewrite");
    assert_eq!(manifest_snapshot_id(&projection), id_before, "the snapshot id is unchanged");

    // Another write advances the scope; the next apply updates the projection to
    // the new snapshot (and the rewrite removes the sentinel).
    let proposal = client.get_artifact("demo", "proposal").expect("read proposal");
    client
        .put_artifact("demo", "proposal", "## Why\n\nEdited.\n", proposal.version)
        .expect("edit proposal");
    let after = run_cli(&remote, &["instructions", "apply", "--change", "demo", "--json"], Some(&pat));
    assert!(after.status.success(), "third apply failed: {}", String::from_utf8_lossy(&after.stderr));
    assert!(!sentinel.exists(), "a fresh snapshot rewrites the projection");
    assert_ne!(
        manifest_snapshot_id(&projection),
        id_before,
        "the projection updated to the new scope snapshot",
    );
    speclink_host::projection::verify_projection(&ws).expect("the refreshed projection verifies");
}
