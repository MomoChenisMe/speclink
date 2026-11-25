//! The terminal activity report through the real binary (spec
//! terminal-activity-report): a fake herdr-core on a Unix socket, and the same
//! verb run twice — once with the Herdr variables, once without — in the same
//! project path, so the two outputs compare byte for byte.
//!
//! Only variables Herdr and this feature already have are set:
//! `HC_TERMINAL_ID`, `HC_TERMINAL_SOCKET` and `SPECLINK_TERMINAL_REPORT`
//! (which `.cargo/config.toml` sets to `false` for every cargo test, so each
//! run here removes it).
#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 One\n- [ ] 1.2 Two\n- [ ] 1.3 Three\n";
const OK: &str = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n";

fn short_dir(tag: &str) -> PathBuf {
    // macOS caps a socket path at 104 bytes: keep it short.
    std::env::temp_dir().join(format!("sl-trit-{tag}-{}", std::process::id()))
}

/// A fake herdr-core: it accepts every connection on its socket, keeps the
/// first line of each and answers it (or never answers).
struct FakeCore {
    dir: PathBuf,
    socket: PathBuf,
    lines: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl FakeCore {
    fn start(tag: &str, answers: bool) -> FakeCore {
        let dir = short_dir(&format!("core-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let socket = dir.join("c.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let lines: Arc<Mutex<Vec<String>>> = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (sink, stopped) = (lines.clone(), stop.clone());
        let worker = std::thread::spawn(move || {
            let mut held = Vec::new();
            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        // On a busy machine the CLI may have given up and closed the
                        // connection already (macOS then refuses setsockopt); the line
                        // it wrote is still there to read, so errors are ignored.
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                        let mut line = String::new();
                        let _ = BufReader::new(&stream).read_line(&mut line);
                        sink.lock().unwrap().push(line);
                        if answers {
                            let _ = (&stream).write_all(OK.as_bytes());
                        } else {
                            held.push(stream);
                        }
                    }
                    // Stop only once nothing waits in the backlog.
                    Err(_) if stopped.load(Ordering::SeqCst) => return,
                    Err(_) => std::thread::sleep(Duration::from_millis(2)),
                }
            }
        });
        FakeCore { dir, socket, lines, stop, worker: Some(worker) }
    }

    /// Every request line the core got, once it stopped.
    fn finish(mut self) -> Vec<String> {
        std::thread::sleep(Duration::from_millis(20));
        self.stop.store(true, Ordering::SeqCst);
        self.worker.take().unwrap().join().unwrap();
        self.lines.lock().unwrap().clone()
    }
}

impl Drop for FakeCore {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A throwaway local project, rebuilt at the same path for each run.
struct Project {
    dir: PathBuf,
    setup: fn(&Project),
}

impl Project {
    fn new(tag: &str, setup: fn(&Project)) -> Project {
        let p = Project { dir: short_dir(&format!("p-{tag}")), setup };
        p.reset();
        p
    }

    /// The project as `setup` leaves it: a change `add-auth` with three open
    /// tasks, and whatever `setup` adds.
    fn reset(&self) {
        let _ = std::fs::remove_dir_all(&self.dir);
        let change = self.dir.join("speclink").join("changes").join("add-auth");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".meta.yaml"), META).unwrap();
        std::fs::write(change.join("tasks.md"), TASKS).unwrap();
        for args in [&["init", "-q"][..], &["config", "user.name", "Sandbox Tester"], &["config", "user.email", "sandbox@example.com"]] {
            let ok = Command::new("git").args(args).current_dir(&self.dir).output().unwrap().status.success();
            assert!(ok, "git {args:?}");
        }
        (self.setup)(self);
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = outside_herdr(&self.dir, args);
        cmd.env_remove("SPECLINK_TOKEN")
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE");
        cmd
    }

    fn quiet(&self, args: &[&str], stdin: &str) {
        let out = run(self.cmd(args), stdin);
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The binary in `dir`, with no Herdr variable, the report switch unset and
/// no store URL from the environment.
fn outside_herdr(dir: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
    cmd.args(args)
        .current_dir(dir)
        .env_remove("HC_TERMINAL_ID")
        .env_remove("HC_TERMINAL_SOCKET")
        .env_remove("SPECLINK_TERMINAL_REPORT")
        .env_remove("SPECLINK_STORE_URL");
    cmd
}

fn run(mut cmd: Command, stdin: &str) -> Output {
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn no_setup(_: &Project) {}

fn with_discussion(p: &Project) {
    p.quiet(&["discuss", "new", "登入流程", "--slug", "auth-flow"], "");
}

fn in_herdr(cmd: &mut Command, socket: &Path) {
    cmd.env("HC_TERMINAL_ID", "t2p").env("HC_TERMINAL_SOCKET", socket);
}

/// The verb run without Herdr, then in the terminal `t2p` whose socket is
/// `socket`, each run on a fresh copy at the same path. `herdr` can change
/// that run's environment further.
fn twice(p: &Project, args: &[&str], stdin: &str, socket: &Path, herdr: impl Fn(&mut Command)) -> (Output, Output) {
    p.reset();
    let plain = run(p.cmd(args), stdin);
    p.reset();
    let mut cmd = p.cmd(args);
    in_herdr(&mut cmd, socket);
    herdr(&mut cmd);
    (plain, run(cmd, stdin))
}

/// Exit code, stdout and stderr byte for byte; the text form is only for the
/// failure message.
fn assert_same(plain: &Output, herdr: &Output) {
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    assert_eq!(herdr.status.code(), plain.status.code());
    assert!(herdr.stdout == plain.stdout, "stdout:\n{}\n--- without Herdr:\n{}", text(&herdr.stdout), text(&plain.stdout));
    assert!(herdr.stderr == plain.stderr, "stderr:\n{}\n--- without Herdr:\n{}", text(&herdr.stderr), text(&plain.stderr));
}

fn label_request(change: &str, discussion: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "terminal.label",
        "params": {"terminal": "t2p", "labels": {"speclink.change": change, "speclink.discussion": discussion}},
    })
}

/// The core got exactly `expected`. A missing request can also mean a machine
/// so busy that the CLI did not connect within its own 100 ms limit.
fn assert_requests(lines: &[String], expected: &[serde_json::Value]) {
    let got: Vec<serde_json::Value> =
        lines.iter().map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))).collect();
    assert_eq!(got, expected, "requests the fake core got (timing-dependent: the CLI waits 100 ms at most)");
}

#[test]
fn a_finished_task_labels_the_change_and_changes_no_output() {
    let p = Project::new("done", no_setup);
    let core = FakeCore::start("done", true);
    let (plain, herdr) = twice(&p, &["task", "done", "1", "--change", "add-auth"], "", &core.socket, |_| {});
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    assert_same(&plain, &herdr);
    let lines = core.finish();
    assert!(lines.iter().all(|line| line.ends_with('\n')), "{lines:?}");
    assert_requests(&lines, &[label_request("add-auth", "")]);
}

#[test]
fn json_and_no_color_output_stay_the_same() {
    let p = Project::new("json", no_setup);
    let core = FakeCore::start("json", true);
    let (plain, herdr) = twice(&p, &["task", "done", "2", "--change", "add-auth", "--json"], "", &core.socket, |_| {});
    assert_same(&plain, &herdr);
    let payload: serde_json::Value = serde_json::from_slice(&herdr.stdout).unwrap();
    let keys: Vec<&str> = payload.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys, ["change", "status", "task_desc", "task_id"]);
    assert_eq!(payload["task_id"], "2");
    let (plain, herdr) = twice(&p, &["task", "done", "3", "--change", "add-auth", "--no-color"], "", &core.socket, |_| {});
    assert_same(&plain, &herdr);
    assert_requests(&core.finish(), &[label_request("add-auth", ""), label_request("add-auth", "")]);
}

#[test]
fn a_change_from_a_discussion_keeps_the_discussion_label() {
    let p = Project::new("from", with_discussion);
    let core = FakeCore::start("from", true);
    let args = ["new", "change", "add-login", "--from-discussion", "auth-flow"];
    let (plain, herdr) = twice(&p, &args, "", &core.socket, |_| {});
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    assert_same(&plain, &herdr);
    assert_requests(&core.finish(), &[label_request("add-login", "auth-flow")]);
}

#[test]
fn a_discussion_round_labels_the_discussion_and_clears_the_change() {
    let p = Project::new("round", with_discussion);
    let core = FakeCore::start("round", true);
    let (plain, herdr) = twice(&p, &["discuss", "add-round", "auth-flow", "--stdin"], "round text\n", &core.socket, |_| {});
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    assert_same(&plain, &herdr);
    assert_requests(&core.finish(), &[label_request("", "auth-flow")]);
}

#[test]
fn read_only_verbs_do_not_connect() {
    let p = Project::new("read", no_setup);
    let core = FakeCore::start("read", true);
    // Only the connection is checked: `list --json` carries file times, which
    // a rebuilt project does not keep.
    for args in [&["show", "add-auth"][..], &["status", "--change", "add-auth", "--json"], &["list", "--json"]] {
        let mut cmd = p.cmd(args);
        in_herdr(&mut cmd, &core.socket);
        let out = run(cmd, "");
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert_eq!(core.finish(), Vec::<String>::new());
}

#[test]
fn a_failed_verb_does_not_connect() {
    let p = Project::new("fail", no_setup);
    let core = FakeCore::start("fail", true);
    let (plain, herdr) = twice(&p, &["task", "done", "99", "--change", "add-auth"], "", &core.socket, |_| {});
    assert!(!plain.status.success());
    assert_same(&plain, &herdr);
    assert_eq!(core.finish(), Vec::<String>::new());
}

/// A directory holding an `hc` that leaves `marker` when it runs.
fn fake_hc(dir: &Path, marker: &Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let hc = bin.join("hc");
    std::fs::write(&hc, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).unwrap();
    std::fs::set_permissions(&hc, std::fs::Permissions::from_mode(0o755)).unwrap();
    format!("{}:{}", bin.display(), std::env::var("PATH").unwrap_or_default())
}

#[test]
fn a_missing_socket_never_runs_hc() {
    let p = Project::new("nosock", no_setup);
    let marker = p.dir.join("hc-ran");
    let missing = short_dir("gone").join("c.sock");
    let path = fake_hc(&short_dir("nosock-bin"), &marker);
    let (plain, herdr) = twice(&p, &["task", "done", "1", "--change", "add-auth"], "", &missing, |cmd| {
        cmd.env("PATH", &path);
    });
    assert_same(&plain, &herdr);
    assert!(!marker.exists(), "hc ran");
    assert!(!missing.exists(), "something was created at the socket path");
    let _ = std::fs::remove_dir_all(short_dir("nosock-bin"));
}

#[test]
fn a_terminal_id_without_a_socket_never_runs_hc() {
    let p = Project::new("idonly", no_setup);
    let marker = p.dir.join("hc-ran");
    let path = fake_hc(&short_dir("idonly-bin"), &marker);
    let plain = run(p.cmd(&["task", "done", "1", "--change", "add-auth"]), "");
    p.reset();
    let mut cmd = p.cmd(&["task", "done", "1", "--change", "add-auth"]);
    cmd.env("HC_TERMINAL_ID", "t2p").env("PATH", &path);
    let herdr = run(cmd, "");
    assert_same(&plain, &herdr);
    assert!(!marker.exists(), "hc ran");
    let _ = std::fs::remove_dir_all(short_dir("idonly-bin"));
}

#[test]
fn a_core_that_never_answers_does_not_hold_the_verb() {
    let p = Project::new("silent", no_setup);
    let core = FakeCore::start("silent", false);
    let args = ["task", "done", "1", "--change", "add-auth"];
    let start = Instant::now();
    let plain = run(p.cmd(&args), "");
    let plain_took = start.elapsed();
    p.reset();
    let mut cmd = p.cmd(&args);
    in_herdr(&mut cmd, &core.socket);
    let start = Instant::now();
    let herdr = run(cmd, "");
    // Only catches a hang: the 100 ms cap itself is pinned at function level.
    assert!(start.elapsed() < plain_took + Duration::from_secs(1), "{:?} vs {plain_took:?}", start.elapsed());
    assert_same(&plain, &herdr);
    assert_eq!(core.finish().len(), 1, "requests the fake core got (timing-dependent: the CLI waits 100 ms at most)");
}

#[test]
fn speclink_terminal_report_false_turns_the_report_off() {
    let p = Project::new("off", no_setup);
    let core = FakeCore::start("off", true);
    let (plain, herdr) = twice(&p, &["task", "done", "1", "--change", "add-auth"], "", &core.socket, |cmd| {
        cmd.env("SPECLINK_TERMINAL_REPORT", "false");
    });
    assert_same(&plain, &herdr);
    assert_eq!(core.finish(), Vec::<String>::new());
}

// --- remote store mode ---

const PROJECT: &str = "/api/speclink/v1/projects/demo";
const BINDING: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[],"polling":{"url":"/sync-state","etag":true}}}}"#;
const DONE: &str = r#"{"change":"add-auth","taskId":"1","taskDesc":"1.1 One","status":"done","alreadyDone":false,"tasksVersion":7}"#;

/// A checkout linked to the remote project at `url`.
struct Checkout {
    dir: PathBuf,
}

impl Checkout {
    fn new(tag: &str, url: &str) -> Checkout {
        let dir = short_dir(&format!("r-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), format!("remote:\n  url: {url}\n  repo: backend\n")).unwrap();
        Checkout { dir }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = outside_herdr(&self.dir, args);
        cmd.env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env("SPECLINK_TOKEN", "tok");
        cmd
    }
}

impl Drop for Checkout {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const TASK_DONE: [&str; 5] = ["task", "done", "1", "--change", "add-auth"];

#[test]
fn remote_mode_labels_after_the_server_accepted() {
    let fake = crate::fake_server::Fake::new();
    fake.route(&format!("{PROJECT}/binding"), 200, BINDING);
    fake.route(&format!("{PROJECT}/changes/add-auth/tasks/1/done"), 200, DONE);
    let checkout = Checkout::new("ok", &fake.project_url());
    let core = FakeCore::start("rok", true);
    let plain = run(checkout.cmd(&TASK_DONE), "");
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    let asked = fake.paths();
    let mut cmd = checkout.cmd(&TASK_DONE);
    in_herdr(&mut cmd, &core.socket);
    let herdr = run(cmd, "");
    assert_same(&plain, &herdr);
    // The report sends the Server nothing more.
    assert_eq!(fake.paths()[asked.len()..], asked[..]);
    assert_requests(&core.finish(), &[label_request("add-auth", "")]);
}

#[test]
fn remote_mode_does_not_label_when_the_server_refuses_or_is_gone() {
    let closed = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}{PROJECT}", listener.local_addr().unwrap())
    };
    let refusing = |status: u16, body: &str| {
        let fake = crate::fake_server::Fake::new();
        fake.route(&format!("{PROJECT}/binding"), 200, BINDING);
        fake.route(&format!("{PROJECT}/changes/add-auth/tasks/1/done"), status, body);
        fake
    };
    let conflict = refusing(409, r#"{"status":409,"reason":"refused","message":"change is busy — wait for the in-flight operation to finish, then retry"}"#);
    let expired = refusing(401, r#"{"status":401,"reason":"unauthenticated","message":"token expired"}"#);
    let core = FakeCore::start("rno", true);
    for (tag, url) in [("409", conflict.project_url()), ("gone", closed), ("401", expired.project_url())] {
        let checkout = Checkout::new(tag, &url);
        let plain = run(checkout.cmd(&TASK_DONE), "");
        assert!(!plain.status.success(), "{tag}");
        let mut cmd = checkout.cmd(&TASK_DONE);
        in_herdr(&mut cmd, &core.socket);
        assert_same(&plain, &run(cmd, ""));
    }
    assert_eq!(core.finish(), Vec::<String>::new());
}
