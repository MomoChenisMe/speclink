//! Disaster-recovery drill (server-backup spec「災難演練閉環」, 決策 3): seed a real
//! server over SQLite (setup、members、PAT、a change verb flow、a discussion、audit),
//! back it up offline with the real binary, restore into a brand-new target,
//! confirm validation is green, then start the restored server and prove the
//! round trip preserved everything — the member's original PAT still connects,
//! CLI query output is byte-for-byte identical to before the backup, the audit
//! history is intact, and /setup stays closed.

use speclink_protocol::command::CreateChangeRequest;
use speclink_remote::client::Client;
use speclink_server::identity::{IdentitySqlite, IdentityStore};
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

const EMAIL: &str = "dev@example.com";
const DISPLAY: &str = "E2E <e2e@example.com>";
const PASSWORD: &str = "e2e-correct-horse";
const ADMIN_EMAIL: &str = "root@example.com";
const ADMIN_DISPLAY: &str = "Root <root@example.com>";
const ADMIN_PASSWORD: &str = "root-correct-horse";

// --- binaries ---

static BUILD_CLI: Once = Once::new();

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
    stdout: Arc<Mutex<Vec<String>>>,
}

impl Server {
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

    fn printed_setup_token(&self) -> bool {
        self.stdout.lock().expect("stdout lock").iter().any(|l| parse_setup_token(l).is_some())
    }
}

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
    TcpListener::bind("127.0.0.1:0").expect("bind free port").local_addr().unwrap().port()
}

// --- config ---

/// Write a server config with a SQLite store and identity at the given paths.
fn write_config(config_path: &Path, store_db: &Path, identity_db: &Path) {
    let mut file = std::fs::File::create(config_path).expect("create config");
    write!(
        file,
        "store:\n  driver: sqlite\n  path: {}\nidentity:\n  driver: sqlite\n  path: {}\n",
        store_db.display(),
        identity_db.display()
    )
    .expect("write config");
}

// --- identity flow over the web ---

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

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

fn create_pat_via_web(base: &str, token: &str) -> String {
    let http = agent();
    let accept = http
        .post(&format!("{base}/api/speclink/v1/web/invite/{token}"))
        .send_json(serde_json::json!({ "password": PASSWORD }))
        .expect("accept invitation");
    assert_eq!(accept.status(), 200, "accepting the invitation succeeds");
    let login = http
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .send_json(serde_json::json!({ "email": EMAIL, "password": PASSWORD }))
        .expect("login");
    assert_eq!(login.status(), 200, "login succeeds");
    let session = login
        .header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("a session cookie")
        .to_string();
    let created: serde_json::Value = http
        .post(&format!("{base}/api/speclink/v1/web/account/tokens"))
        .set("Cookie", &format!("speclink_session={session}"))
        .send_json(serde_json::json!({ "name": "cli" }))
        .expect("create PAT")
        .into_json()
        .expect("create PAT json");
    created["data"]["plaintext"]
        .as_str()
        .unwrap_or_else(|| panic!("PAT plaintext in response: {created}"))
        .to_string()
}

fn complete_setup(base: &str, token: &str) {
    let http = agent();
    let admin = http
        .post(&format!("{base}/api/speclink/v1/web/setup/admin?token={token}"))
        .send_json(serde_json::json!({ "email": ADMIN_EMAIL, "display": ADMIN_DISPLAY, "password": ADMIN_PASSWORD }))
        .expect("setup: create the first admin");
    assert_eq!(admin.status(), 200, "the setup admin section succeeds");
    let project = http
        .post(&format!("{base}/api/speclink/v1/web/setup/registry?token={token}"))
        .send_json(serde_json::json!({
            "projectKey": "demo",
            "projectName": "Demo",
            "repoKey": "backend",
            "repoName": "Backend",
        }))
        .expect("setup: register the first project/repo");
    assert_eq!(project.status(), 200, "the setup project/repo section succeeds");
}

/// Seed change `demo` and a discussion through the typed client with `pat`.
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
    for (artifact, content) in
        [("proposal", PROPOSAL), ("design", DESIGN), ("tasks", TASKS), ("specs/cap-a", SPEC)]
    {
        client.put_artifact("demo", artifact, content, 0).unwrap_or_else(|e| panic!("put {artifact}: {e:?}"));
    }
    client.new_discussion("Rate limiting approach", None).expect("seed discussion");
}

fn remote_project(dir: &Path, name: &str, project_url: &str) -> PathBuf {
    let project = dir.join(name);
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join(".speclink.yaml"),
        format!("remote:\n  url: {project_url}\n  repo: backend\n"),
    )
    .unwrap();
    project
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

fn get_status(url: &str) -> u16 {
    match ureq::get(url).call() {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("transport error for {url}: {e}"),
    }
}

/// The recorded audit action list of an identity database (opened offline).
fn audit_actions(identity_db: &Path) -> Vec<String> {
    let identity = IdentitySqlite::open(identity_db).expect("open identity for inspection");
    identity.list_audit(u32::MAX, 0).expect("audit").into_iter().map(|e| e.action).collect()
}

// --- the drill ---

#[test]
fn disaster_recovery_round_trip_preserves_everything() {
    let _gate = crate::common::acquire_process_gate();
    let workdir = tempfile::tempdir().expect("workdir");
    let src_store = workdir.path().join("store.db");
    let src_identity = workdir.path().join("identity.db");
    let src_config = workdir.path().join("server.yaml");
    write_config(&src_config, &src_store, &src_identity);

    // Seed a real server: setup (admin + demo/backend), a member with a PAT, a
    // change verb flow and a discussion.
    let server = Server::start(&src_config);
    let setup_token = server.setup_token();
    complete_setup(&server.base(), &setup_token);
    let invite_token = invite(&src_config);
    let pat = create_pat_via_web(&server.base(), &invite_token);
    seed(&server.project_url(), &pat);

    // Capture the pre-backup CLI query output (byte-comparison baseline).
    let pre = remote_project(workdir.path(), "pre", &server.project_url());
    let pre_status = run_cli(&pre, &["status", "--change", "demo", "--json"], Some(&pat));
    assert!(pre_status.status.success(), "pre-backup status: {}", String::from_utf8_lossy(&pre_status.stderr));
    let pre_list = run_cli(&pre, &["list", "--json"], Some(&pat));
    assert!(String::from_utf8_lossy(&pre_list.stdout).contains("demo"), "seeded change is listed");

    // Stop the server — backup runs offline (决策 2).
    drop(server);
    let src_audit = audit_actions(&src_identity);
    assert!(src_audit.contains(&"setup-completed".to_string()), "setup was audited");
    assert!(src_audit.contains(&"user-invited".to_string()), "the invite was audited");

    // Back up with the real binary.
    let backup_tar = workdir.path().join("backup.tar");
    let backup = Command::new(server_bin())
        .arg("backup")
        .args(["--config", src_config.to_str().unwrap()])
        .args(["--output", backup_tar.to_str().unwrap()])
        .output()
        .expect("run backup");
    assert!(backup.status.success(), "backup failed: {}", String::from_utf8_lossy(&backup.stderr));

    // Restore into a brand-new target; validation must be green (exit 0).
    let tgt_store = workdir.path().join("tgt-store.db");
    let tgt_identity = workdir.path().join("tgt-identity.db");
    let tgt_config = workdir.path().join("target.yaml");
    write_config(&tgt_config, &tgt_store, &tgt_identity);
    let restore = Command::new(server_bin())
        .arg("restore")
        .args(["--config", tgt_config.to_str().unwrap()])
        .args(["--input", backup_tar.to_str().unwrap()])
        .output()
        .expect("run restore");
    assert!(
        restore.status.success(),
        "restore validation must be green (exit 0): {}",
        String::from_utf8_lossy(&restore.stderr)
    );

    // Start the restored server and prove the round trip preserved everything.
    let restored = Server::start(&tgt_config);
    assert!(!restored.printed_setup_token(), "the restored server prints no setup token — /setup stays closed");
    assert_eq!(get_status(&format!("{}/api/speclink/v1/web/setup", restored.base())), 404, "/setup is closed on the restored server");

    // The member's original PAT still connects, and CLI query output is
    // byte-for-byte identical to before the backup.
    let post = remote_project(workdir.path(), "post", &restored.project_url());
    let post_status = run_cli(&post, &["status", "--change", "demo", "--json"], Some(&pat));
    assert!(
        post_status.status.success(),
        "the original PAT authenticates against the restored server: {}",
        String::from_utf8_lossy(&post_status.stderr)
    );
    assert_eq!(post_status.stdout, pre_status.stdout, "status --json is byte-identical after restore");

    // The seeded discussion survived the round trip.
    let discuss = run_cli(&post, &["discuss", "list", "--json"], Some(&pat));
    assert!(
        String::from_utf8_lossy(&discuss.stdout).contains("Rate limiting approach"),
        "the discussion survived the restore: {}",
        String::from_utf8_lossy(&discuss.stderr)
    );

    // Stop the restored server, then compare the audit history offline: identical.
    drop(restored);
    let tgt_audit = audit_actions(&tgt_identity);
    assert_eq!(tgt_audit, src_audit, "the full audit history is intact after restore");
}
