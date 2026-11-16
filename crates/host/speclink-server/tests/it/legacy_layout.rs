//! The store location holds data of another layout (reference-server「啟動組態
//! fail closed」), and what a fresh store of this version gives (「寫入原子提交
//! 且 CAS 衝突可辨」; design D1, D2 of server-on-port-examples).
//!
//! speclink-server 0.12 and earlier kept its data in another layout. This
//! version does not open it: the server and the `backup` subcommand stop
//! before any port is bound, leave the data as it is, and say how to move
//! it. The tests make the old layouts' markers by hand: the old drivers are
//! not a dependency of the server.

use crate::common;

use speclink_protocol::command::CreateChangeRequest;
use speclink_remote::client::Client;
use speclink_server::build_store;
use speclink_server::config::StoreConfig;
use speclink_store_port::{KeyPrefix, ScopeId};
use std::collections::BTreeMap;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What a run of the server binary gave.
struct Run {
    /// `None` when the process did not end by itself and was killed.
    exit: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Run `speclink-server` with `args` and wait for it to end. A server that
/// starts to serve never ends: it is killed after a while, and `exit` is
/// `None`.
fn run(args: &[&str]) -> Run {
    let mut child = Command::new(env!("CARGO_BIN_EXE_speclink-server"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn speclink-server");
    let deadline = Instant::now() + Duration::from_secs(10);
    let exit = loop {
        match child.try_wait().expect("wait for the server") {
            Some(status) => break status.code(),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    child.stdout.take().expect("stdout").read_to_string(&mut stdout).expect("read stdout");
    child.stderr.take().expect("stderr").read_to_string(&mut stderr).expect("read stderr");
    Run { exit, stdout, stderr }
}

/// A loopback port this test holds for as long as the listener lives. A
/// server told to bind it cannot: a server that tries says so on stderr.
fn held_port() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a port");
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

/// A config file in `dir` for `driver` at `store`, with a SQLite identity
/// database that does not exist yet.
fn write_config(dir: &Path, driver: &str, store: &Path) -> PathBuf {
    let config = dir.join("server.yaml");
    std::fs::write(
        &config,
        format!(
            "store:\n  driver: {driver}\n  path: {}\nidentity:\n  driver: sqlite\n  path: {}\n",
            store.display(),
            dir.join("identity.db").display()
        ),
    )
    .expect("write the config");
    config
}

/// A SQLite file as speclink-server 0.12 left it: the `meta` table with the
/// old driver's marker, and a table with a row.
fn old_sqlite_store(path: &Path) {
    let conn = rusqlite::Connection::open(path).expect("create the old database");
    conn.execute_batch(
        "CREATE TABLE meta (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);
         INSERT INTO meta (key, value) VALUES ('format', 'speclink-team-store'), ('schema_version', '1');
         CREATE TABLE documents (id TEXT PRIMARY KEY NOT NULL, content TEXT NOT NULL);
         INSERT INTO documents (id, content) VALUES ('config', 'schema: spec-driven');",
    )
    .expect("fill the old database");
}

/// A directory as speclink-server 0.12 left it: `meta.json` with the old
/// driver's marker, and a data file.
fn old_serverfs_store(dir: &Path) {
    std::fs::create_dir_all(dir.join("scopes")).expect("create the old directory");
    std::fs::write(dir.join("meta.json"), r#"{"format":"speclink-team-store-fs","version":1}"#).expect("meta");
    std::fs::write(dir.join("scopes").join("data"), "schema: spec-driven").expect("data");
}

/// Every file under `root`, by its path from `root`, with its bytes.
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, found: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).expect("read dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk(root, &path, found);
            } else {
                found.insert(path.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(&path).expect("read file"));
            }
        }
    }
    let mut found = BTreeMap::new();
    if root.is_dir() {
        walk(root, root, &mut found);
    } else {
        found.insert(PathBuf::new(), std::fs::read(root).expect("read file"));
    }
    found
}

/// The run stopped before it served: a failure exit, the message for data of
/// an older version with its three steps, no setup link, and no try to bind
/// a port (the tests give the server a port that is taken: a try to bind it
/// is an "address in use" failure on stderr).
fn assert_refused_with_the_steps(run: &Run, config: &Path) {
    assert!(
        matches!(run.exit, Some(code) if code != 0),
        "the process ends with a failure, it does not serve: exit {:?}, stderr: {}",
        run.exit,
        run.stderr
    );
    let config = config.display().to_string();
    for line in [
        "0.12 or earlier",
        "did not change your data",
        &format!("speclink-server backup --config {config} --output <file>"),
        "Point the config at an empty store location and an empty identity database.",
        &format!("speclink-server restore --config {config} --input <file>"),
    ] {
        assert!(run.stderr.contains(line), "stderr has {line:?}: {}", run.stderr);
    }
    assert_eq!(run.stdout, "", "nothing was printed to stdout: no setup link");
    assert_did_not_bind(run);
}

fn assert_did_not_bind(run: &Run) {
    let stderr = run.stderr.to_lowercase();
    assert!(!stderr.contains("in use") && !stderr.contains("os error"), "the server did not try to bind: {}", run.stderr);
}

#[test]
fn the_server_refuses_a_sqlite_store_of_an_older_version_and_leaves_it_as_it_was() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("store.db");
    old_sqlite_store(&store);
    let config = write_config(dir.path(), "sqlite", &store);
    let before = files(&store);
    let (_listener, port) = held_port();

    let run = run(&["--config", config.to_str().unwrap(), "--addr", &format!("127.0.0.1:{port}")]);

    assert_refused_with_the_steps(&run, &config);
    assert!(run.stderr.contains(&store.display().to_string()), "the message names the location: {}", run.stderr);
    assert_eq!(files(&store), before, "the database file is the same, byte for byte");
    assert_eq!(files(dir.path()).len(), 2, "no file was made next to it: only the config and the store");
}

#[test]
fn the_server_refuses_a_serverfs_store_of_an_older_version_and_leaves_it_as_it_was() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("store");
    old_serverfs_store(&store);
    let config = write_config(dir.path(), "serverfs", &store);
    let before = files(&store);
    let (_listener, port) = held_port();

    let run = run(&["--config", config.to_str().unwrap(), "--addr", &format!("127.0.0.1:{port}")]);

    assert_refused_with_the_steps(&run, &config);
    assert_eq!(files(&store), before, "the directory holds the same files with the same bytes");
}

#[test]
fn the_backup_subcommand_refuses_a_store_of_an_older_version_and_writes_no_backup() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("store.db");
    old_sqlite_store(&store);
    let config = write_config(dir.path(), "sqlite", &store);
    let before = files(&store);
    let output = dir.path().join("backup.tar");

    let run = run(&["backup", "--config", config.to_str().unwrap(), "--output", output.to_str().unwrap()]);

    assert_refused_with_the_steps(&run, &config);
    assert!(!output.exists(), "no backup file was made");
    assert_eq!(files(&store), before, "the database file is the same, byte for byte");
}

/// design D2: a restore whose target holds data of an older version is
/// refused with the same message, the config path filled in.
#[test]
fn the_restore_subcommand_refuses_a_target_of_an_older_version() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("store.db");
    old_sqlite_store(&store);
    let config = write_config(dir.path(), "sqlite", &store);
    let before = files(&store);
    let backup = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backup-format-1.tar");

    let run = run(&["restore", "--config", config.to_str().unwrap(), "--input", backup.to_str().unwrap()]);

    assert_refused_with_the_steps(&run, &config);
    assert_eq!(files(&store), before, "the database file is the same, byte for byte");
    assert!(!dir.path().join("identity.db").exists(), "no identity snapshot was placed");
}

#[test]
fn the_server_refuses_a_directory_that_is_not_a_speclink_store_without_the_steps() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let store = dir.path().join("notes");
    std::fs::create_dir_all(&store).expect("create the directory");
    std::fs::write(store.join("todo.txt"), "buy milk").expect("another program's file");
    let config = write_config(dir.path(), "serverfs", &store);
    let before = files(&store);
    let (_listener, port) = held_port();

    let run = run(&["--config", config.to_str().unwrap(), "--addr", &format!("127.0.0.1:{port}")]);

    assert!(matches!(run.exit, Some(code) if code != 0), "exit {:?}, stderr: {}", run.exit, run.stderr);
    assert!(run.stderr.contains("is not a speclink store"), "{}", run.stderr);
    assert!(run.stderr.contains(&store.display().to_string()), "the message names the location: {}", run.stderr);
    for step in ["0.12 or earlier", "speclink-server backup", "speclink-server restore"] {
        assert!(!run.stderr.contains(step), "the steps are for data of an older version only: {}", run.stderr);
    }
    assert_eq!(run.stdout, "");
    assert_did_not_bind(&run);
    assert_eq!(files(&store), before, "no file was added to the directory");
}

/// A path with spaces and characters outside ASCII: the location is shown as
/// it is, and the config path is one argument of the commands.
#[test]
fn the_steps_quote_a_config_path_with_spaces() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path().join("我的 資料 夾");
    std::fs::create_dir_all(&home).expect("create the directory");
    let store = home.join("store.db");
    old_sqlite_store(&store);
    let config = write_config(&home, "sqlite", &store);
    let before = files(&store);
    let (_listener, port) = held_port();

    let run = run(&["--config", config.to_str().unwrap(), "--addr", &format!("127.0.0.1:{port}")]);

    assert!(matches!(run.exit, Some(code) if code != 0), "exit {:?}, stderr: {}", run.exit, run.stderr);
    let quoted = format!("\"{}\"", config.display());
    for line in [
        format!("the store at {} was created by speclink-server 0.12 or earlier", store.display()),
        format!("speclink-server backup --config {quoted} --output <file>"),
        format!("speclink-server restore --config {quoted} --input <file>"),
    ] {
        assert!(run.stderr.contains(&line), "stderr has {line:?}: {}", run.stderr);
    }
    assert_eq!(files(&store), before);
}

/// The same kind of path as the location of a new store: it opens, and a
/// document written to it reads back.
#[test]
fn a_new_store_at_a_path_with_spaces_and_non_ascii_characters_works() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path().join("我的 資料 夾");
    std::fs::create_dir_all(&home).expect("create the directory");
    for config in [
        StoreConfig::Sqlite { path: home.join("儲存 庫.db") },
        StoreConfig::ServerFs { path: home.join("儲存 庫") },
    ] {
        let store = match build_store(&config) {
            Ok(store) => store,
            Err(e) => panic!("{config:?}: {e}"),
        };
        store.probe().expect("the store answers");
        assert_eq!(store.list(&ScopeId::new("demo/backend").unwrap(), &KeyPrefix::new("").unwrap()).expect("list").version.get(), 0);
    }
}

/// The open as a library call gives the same refusal, with the config
/// file's place left for the caller to fill in.
#[test]
fn opening_a_store_of_an_older_version_fails_with_the_reason() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("store.db");
    old_sqlite_store(&path);
    let shown = match build_store(&StoreConfig::Sqlite { path }) {
        Ok(_) => panic!("the old layout was opened"),
        Err(e) => e.to_string(),
    };
    assert!(shown.contains("0.12 or earlier"), "{shown}");
}

// --- a fresh store of this version ---

fn create_change(client: &Client, name: &str) {
    client
        .create_change(CreateChangeRequest {
            name: name.to_string(),
            schema: Some("spec-driven".to_string()),
            description: None,
            agent: None,
            from_discussion: None,
            last: false,
            fields: Default::default(),
        })
        .expect("create change");
}

/// A server on a new SQLite store in `dir`, with the `demo` project and the
/// two-repo `multi` project. Returns the store, the base URL and an access
/// key of a member of both projects.
fn fresh_sqlite_server(dir: &Path) -> (common::TestStore, String, String) {
    let store = common::TestStore::sqlite(dir.join("store.db"));
    let state = common::state_with(store.shared());
    common::seed_multi_project(&*state.identity);
    let (pat, _) = common::seed_pat(&state.identity, &["demo", "multi"]);
    (store, common::start(state), pat)
}

/// The change list carries a real last-modified time (the store of 0.12
/// gave none).
#[test]
fn a_change_on_a_fresh_sqlite_store_has_a_last_modified_time() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (_store, base, pat) = fresh_sqlite_server(dir.path());
    let project = format!("{base}/api/speclink/v1/projects/demo");
    create_change(&Client::new(&project, &pat, Some("backend")), "demo");

    let list: serde_json::Value = ureq::get(&format!("{project}/changes"))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("GET /changes")
        .into_json()
        .expect("JSON");
    let modified_at = &list["changes"][0]["modifiedAt"];
    assert!(
        !modified_at.is_null() && modified_at != 0 && modified_at != "1970-01-01T00:00:00Z",
        "the change has a real last-modified time: {list}"
    );
}

/// Each scope has its own version: a write to one repo does not move the
/// other repo's.
#[test]
fn two_repos_of_one_project_have_their_own_scope_versions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (store, base, pat) = fresh_sqlite_server(dir.path());
    let project = format!("{base}/api/speclink/v1/projects/multi");
    create_change(&Client::new(&project, &pat, Some("web")), "in-web");
    create_change(&Client::new(&project, &pat, Some("api")), "in-api");

    assert_eq!(store.version("multi", "web"), 1, "one commit in web");
    assert_eq!(store.version("multi", "api"), 1, "one commit in api: web's commit did not move it");
}

/// spec reference-server「成功寫入事件同步可見」: the event of a task done is
/// in the scope's event log, from the same commit as the checked task.
#[test]
fn a_task_done_puts_its_event_in_the_event_log_with_the_same_commit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (store, base, pat) = fresh_sqlite_server(dir.path());
    let client = Client::new(&format!("{base}/api/speclink/v1/projects/demo"), &pat, Some("backend"));
    create_change(&client, "demo");
    client.put_artifact("demo", "tasks", "## 1. Work\n\n- [ ] 1.1 First\n", 0).expect("put tasks");
    client.task_done("demo", "1", &[], None).expect("task done");

    let events = store.events("demo", "backend");
    let done = events.iter().find(|event| event.name == "task-completed").expect("a task-completed event");
    let tasks = store.doc("demo", "backend", "changes/demo/tasks.md").expect("the tasks document");
    assert!(tasks.content.contains("- [x] 1.1"), "the task is checked: {}", tasks.content);
    assert_eq!(done.version, tasks.version, "the event and the checked task are of one commit");
}
