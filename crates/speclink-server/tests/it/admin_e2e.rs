//! End-to-end management面 over the real server binary and CLI on SQLite
//! (server-admin spec, 決策 1-4). An operator completes /setup, then through
//! /admin invites a member and registers a second project/repo; the member walks
//! the web flow to a PAT and runs a CLI verb against the new project; the admin
//! force-revokes that PAT and the member's next CLI call fails 401; and the audit
//! page carries every action in reverse-chronological order.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};

const ADMIN_EMAIL: &str = "root@example.com";
const ADMIN_DISPLAY: &str = "Root <root@example.com>";
const ADMIN_PASSWORD: &str = "root-correct-horse";

const MEMBER_EMAIL: &str = "member@example.com";
const MEMBER_PASSWORD: &str = "member-correct-horse";

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
                panic!("no setup token on stdout");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
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

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").expect("bind free port").local_addr().unwrap().port()
}

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

// --- web helpers ---

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

/// Complete setup over the browser JSON API: create the first admin and the first
/// project/repo (demo/backend).
fn complete_setup(base: &str, token: &str) {
    let http = agent();
    let admin = http
        .post(&format!("{base}/api/speclink/v1/web/setup/admin?token={token}"))
        .send_json(json!({ "email": ADMIN_EMAIL, "display": ADMIN_DISPLAY, "password": ADMIN_PASSWORD }))
        .expect("setup admin");
    assert_eq!(admin.status(), 200, "setup admin section");
    let project = http
        .post(&format!("{base}/api/speclink/v1/web/setup/registry?token={token}"))
        .send_json(json!({ "projectKey": "demo", "projectName": "Demo", "repoKey": "backend", "repoName": "Backend" }))
        .expect("setup registry");
    assert_eq!(project.status(), 200, "setup registry section");
}

/// Log in over the browser JSON API and return the session cookie value.
fn login(base: &str, email: &str, password: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .send_json(json!({ "email": email, "password": password }))
        .expect("login");
    assert_eq!(resp.status(), 200, "login succeeds");
    resp.header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("session cookie")
        .to_string()
}

/// POST JSON to a browser admin path with the admin session cookie; returns the response.
fn post_admin(base: &str, path: &str, session: &str, body: Value) -> ureq::Response {
    agent()
        .post(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .send_json(body)
        .unwrap_or_else(|e| match e {
            ureq::Error::Status(_, resp) => resp,
            e => panic!("transport error posting {path}: {e}"),
        })
}

/// GET JSON from a browser admin path with the admin session cookie; returns the parsed body.
fn get_admin(base: &str, path: &str, session: &str) -> Value {
    agent()
        .get(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .call()
        .expect("admin GET")
        .into_json()
        .expect("admin GET json")
}

/// Walk the member web flow over the JSON API: accept the invitation, log in,
/// create a PAT. Returns the PAT plaintext.
fn member_pat(base: &str, invite_token: &str) -> String {
    let http = agent();
    let accept = http
        .post(&format!("{base}/api/speclink/v1/web/invite/{invite_token}"))
        .send_json(json!({ "password": MEMBER_PASSWORD }))
        .expect("accept invitation");
    assert_eq!(accept.status(), 200, "accept invitation");
    let session = login(base, MEMBER_EMAIL, MEMBER_PASSWORD);
    let created: Value = http
        .post(&format!("{base}/api/speclink/v1/web/account/tokens"))
        .set("Cookie", &format!("speclink_session={session}"))
        .send_json(json!({ "name": "cli" }))
        .expect("create PAT")
        .into_json()
        .expect("create PAT json");
    created["data"]["plaintext"]
        .as_str()
        .unwrap_or_else(|| panic!("PAT plaintext in response: {created}"))
        .to_string()
}

// --- CLI helpers ---

/// A remote-mode project dir pointed at `project_url` with repo `api`.
fn remote_project(dir: &Path, project_url: &str) -> PathBuf {
    let project = dir.join("remote");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join(".speclink.yaml"),
        format!("remote:\n  url: {project_url}\n  repo: api\n"),
    )
    .unwrap();
    project
}

fn run_cli(project: &Path, args: &[&str], token: &str) -> Output {
    Command::new(cli_bin())
        .args(args)
        .current_dir(project)
        .env_remove("SPECLINK_STORE_URL")
        .env("SPECLINK_TOKEN", token)
        .output()
        .expect("run speclink CLI")
}

// --- the e2e ---

#[test]
fn the_admin_manages_a_team_end_to_end_over_the_real_binaries() {
    let _gate = crate::common::acquire_process_gate();
    let workdir = tempfile::tempdir().expect("workdir");
    let db = workdir.path().join("store.db");
    let config = write_config(workdir.path(), &db);

    // 1. Fresh server → complete /setup (first admin + demo/backend).
    let server = Server::start(&config);
    let setup_token = server.setup_token();
    complete_setup(&server.base(), &setup_token);
    let base = server.base();
    let admin = login(&base, ADMIN_EMAIL, ADMIN_PASSWORD);

    // 2. Through the browser admin API, register a second project/repo and invite the member.
    assert_eq!(
        post_admin(&base, "/api/speclink/v1/web/admin/registry/projects", &admin, json!({ "key": "team", "name": "Team" })).status(),
        200
    );
    assert_eq!(
        post_admin(&base, "/api/speclink/v1/web/admin/registry/repos", &admin, json!({ "projectKey": "team", "key": "api", "name": "API" })).status(),
        200
    );
    let invite = post_admin(&base, "/api/speclink/v1/web/admin/users/invite", &admin, json!({ "email": MEMBER_EMAIL, "display": "Member", "memberships": ["team"] }));
    assert_eq!(invite.status(), 200, "the invite API returns the acceptance token");
    let invite_body: Value = invite.into_json().expect("invite json");
    let invite_token = invite_body["data"]["token"].as_str().expect("an invite token").to_string();

    // 3. The member accepts, gets a PAT, and runs a CLI verb against team/api.
    let pat = member_pat(&base, &invite_token);
    let project_url = format!("{base}/api/speclink/v1/projects/team");
    let remote = remote_project(workdir.path(), &project_url);
    let before = run_cli(&remote, &["list", "--json"], &pat);
    assert!(before.status.success(), "the member's PAT runs a CLI verb: {}", String::from_utf8_lossy(&before.stderr));

    // 4. The admin force-revokes that PAT from the credentials view.
    let creds = get_admin(&base, "/api/speclink/v1/web/admin/credentials", &admin);
    let pats = creds["data"]["pats"].as_array().expect("the credentials view lists PATs");
    let pat_id = pats
        .iter()
        .find(|p| p["name"] == "cli")
        .expect("the member's PAT is listed")["id"]
        .as_str()
        .expect("a PAT id")
        .to_string();
    assert_eq!(
        post_admin(&base, &format!("/api/speclink/v1/web/admin/credentials/tokens/{pat_id}/revoke"), &admin, json!({})).status(),
        200
    );

    // 5. The member's next CLI call fails to authenticate.
    let after = run_cli(&remote, &["list", "--json"], &pat);
    assert!(!after.status.success(), "the revoked PAT no longer authenticates");

    // 6. The audit carries every action, newest first.
    let audit = get_admin(&base, "/api/speclink/v1/web/admin/audit", &admin);
    let entries = audit["data"]["entries"].as_array().expect("audit entries");
    let order = |needle: &str| {
        entries
            .iter()
            .position(|e| e["action"] == needle)
            .unwrap_or_else(|| panic!("audit missing {needle}: {audit}"))
    };
    let revoked = order("token-revoked");
    let invited = order("user-invited");
    let repo_created = order("repo-created");
    let project_created = order("project-created");
    let setup_done = order("setup-completed");
    assert!(
        revoked < invited && invited < repo_created && repo_created < project_created && project_created < setup_done,
        "audit is reverse-chronological: revoke, invite, repo, project, setup"
    );
}
