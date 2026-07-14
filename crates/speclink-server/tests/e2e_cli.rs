//! End-to-end: the real CLI binary against the real server over SQLite
//! (reference-server spec「真實 CLI 端到端一致」). Data is seeded through the
//! command routes (never straight into the database); remote CLI output matches
//! fs mode; and the server restart keeps the SQLite data intact.

use speclink_protocol::command::CreateChangeRequest;
use speclink_remote::client::Client;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output};
use std::sync::Once;
use std::time::{Duration, Instant};

const TOKEN: &str = "e2e-token";

const PROPOSAL: &str = "## Why\n\nDemo change.\n\n## What Changes\n\n- a thing\n";
const DESIGN: &str = "## Context\n\nDemo design.\n";
const TASKS: &str = "## 1. Work\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n";
const SPEC: &str = "## ADDED Requirements\n\n### Requirement: Demo\nDemo SHALL work.\n\n#### Scenario: works\n- **WHEN** run\n- **THEN** ok\n";

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
}

impl Server {
    /// Start the server binary with `config` bound to a free loopback port and
    /// wait until `/healthz` answers.
    fn start(config: &Path) -> Server {
        let port = free_port();
        let addr = format!("127.0.0.1:{port}");
        let child = Command::new(server_bin())
            .args(["--config", config.to_str().unwrap(), "--addr", &addr])
            .spawn()
            .expect("spawn speclink-server");
        let server = Server { child, addr };
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

// --- config, seeding, project layout ---

fn write_config(dir: &Path, db: &Path) -> PathBuf {
    let path = dir.join("server.yaml");
    let mut file = std::fs::File::create(&path).expect("create config");
    write!(
        file,
        "store:\n  driver: sqlite\n  path: {}\nprojects:\n  - key: demo\n    name: Demo\n    repos:\n      - backend\ntokens:\n  - token: {TOKEN}\n    actor:\n      id: u_e2e\n      display: E2E <e2e@example.com>\n",
        db.display()
    )
    .expect("write config");
    path
}

/// Seed change `demo` through the command routes with the typed client.
fn seed(project_url: &str) {
    let client = Client::new(project_url, TOKEN, Some("backend"));
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

// --- the e2e ---

#[test]
fn real_cli_over_real_server_matches_fs_and_survives_restart() {
    let workdir = tempfile::tempdir().expect("workdir");
    let db = workdir.path().join("store.db");
    let config = write_config(workdir.path(), &db);

    let server = Server::start(&config);
    seed(&server.project_url());

    let remote = remote_project(workdir.path(), &server.project_url());
    let fs = fs_project(workdir.path());

    // Remote verbs succeed against the real server.
    let list = run_cli(&remote, &["list", "--json"], Some(TOKEN));
    assert!(list.status.success(), "remote list failed: {}", String::from_utf8_lossy(&list.stderr));
    assert!(
        String::from_utf8_lossy(&list.stdout).contains("demo"),
        "remote list names the seeded change: {}",
        String::from_utf8_lossy(&list.stdout)
    );

    // Byte-identical parity against fs mode (fs is the shape authority).
    let remote_status = run_cli(&remote, &["status", "--change", "demo", "--json"], Some(TOKEN));
    let fs_status = run_cli(&fs, &["status", "--change", "demo", "--json"], None);
    assert!(
        remote_status.status.success() && fs_status.status.success(),
        "status runs on both paths (remote stderr: {}) (fs stderr: {})",
        String::from_utf8_lossy(&remote_status.stderr),
        String::from_utf8_lossy(&fs_status.stderr),
    );
    assert_eq!(
        remote_status.stdout, fs_status.stdout,
        "remote status --json is byte-identical to fs mode\nremote: {}\nfs:     {}",
        String::from_utf8_lossy(&remote_status.stdout),
        String::from_utf8_lossy(&fs_status.stdout),
    );

    // Apply instructions match on their content once the fields that legitimately
    // differ between the two modes are set aside: `changeDir`/`contextFiles`
    // point into the remote projection (vs the fs paths), and `preflight` is a
    // deliberately fs-only local-file check the wire contract omits.
    let remote_apply = run_cli(&remote, &["instructions", "apply", "--change", "demo", "--json"], Some(TOKEN));
    let fs_apply = run_cli(&fs, &["instructions", "apply", "--change", "demo", "--json"], None);
    assert!(
        remote_apply.status.success() && fs_apply.status.success(),
        "apply runs on both paths"
    );
    assert_eq!(
        content_only(&remote_apply.stdout),
        content_only(&fs_apply.stdout),
        "remote and fs apply agree on the store-determined content\nremote: {}\nfs:     {}",
        String::from_utf8_lossy(&remote_apply.stdout),
        String::from_utf8_lossy(&fs_apply.stdout),
    );

    // Discussions replay end to end: the seeded discussion is listed.
    let discuss = run_cli(&remote, &["discuss", "list", "--json"], Some(TOKEN));
    assert!(discuss.status.success(), "remote discuss list failed: {}", String::from_utf8_lossy(&discuss.stderr));
    assert!(
        String::from_utf8_lossy(&discuss.stdout).contains("Rate limiting approach"),
        "remote discuss list names the seeded discussion: {}",
        String::from_utf8_lossy(&discuss.stdout)
    );

    // Restart the server against the same database: the data is still there.
    drop(server);
    let restarted = Server::start(&config);
    let remote2 = remote_project(&workdir.path().join("after"), &restarted.project_url());
    let list2 = run_cli(&remote2, &["list", "--json"], Some(TOKEN));
    assert!(list2.status.success(), "list after restart failed: {}", String::from_utf8_lossy(&list2.stderr));
    assert!(
        String::from_utf8_lossy(&list2.stdout).contains("demo"),
        "the change persisted across the restart: {}",
        String::from_utf8_lossy(&list2.stdout)
    );
}
