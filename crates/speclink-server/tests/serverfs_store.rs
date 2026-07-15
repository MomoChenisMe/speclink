//! The server over the `serverfs` store driver (serverfs-team-store spec
//! 「serverfs 組態可服務」).
//!
//! The point of a store contract is that the layer above cannot tell which
//! driver is underneath. So this drives the same verb flow through a
//! serverfs-backed server and a sqlite-backed one and compares what came
//! out — a difference here means the abstraction leaks, whichever side is
//! "right".

mod common;

use speclink_protocol::command::CreateChangeRequest;
use speclink_remote::client::Client;
use speclink_server::build_store;
use speclink_server::config::StoreConfig;
use speclink_server::state::SharedStore;
use speclink_store::{OutboxCursor, ProjectId, RepoId, Scope};
use std::process::{Command, Stdio};

const PROPOSAL: &str = "## Why\n\nserverfs drives the same flow.\n";
const TASKS: &str = "## 1. Work\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n";

/// Names the data directory for the helper process of the restart test.
const DIR_ENV: &str = "SPECLINK_SERVERFS_E2E_DIR";

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

fn client(base: &str, token: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        token,
        Some("backend"),
    )
}

/// `build_store` hands back a trait object that has no `Debug`, so a failed
/// open cannot be unwrapped with `expect_err`. Reduce it to its message.
fn open_error(config: &StoreConfig) -> String {
    match build_store(config) {
        Ok(_) => panic!("expected the open to be refused"),
        Err(e) => e.to_string(),
    }
}

/// Blank out the task ids the engine mints. They are ULIDs — new on every
/// run, by design — so comparing two independent flows verbatim would only
/// ever prove that time passed.
fn scrub_task_ids(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("tsk_") {
        out.push_str(&rest[..at + "tsk_".len()]);
        rest = &rest[at + "tsk_".len()..];
        let id_len = rest
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(rest.len());
        out.push_str("<id>");
        rest = &rest[id_len..];
    }
    out.push_str(rest);
    out
}

/// What the verb flow produced, in terms the protocol defines — no store
/// internals, so the two drivers are comparable at all.
#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    schema: Option<String>,
    proposal: String,
    tasks: String,
    changes: Vec<String>,
    task_already_done: bool,
    events: Vec<String>,
}

/// Drive the standard verb flow against a server over `store` and report what
/// the client observed.
///
/// Note the state — and with it `store` — outlives this call: `common::start`
/// hands the server a detached thread that lives for the rest of the process.
/// That is why the restart test below needs a real second process rather than
/// a drop.
fn drive(store: SharedStore) -> Outcome {
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    client.handshake().expect("handshake");
    let created = client
        .create_change(CreateChangeRequest {
            name: "demo".into(),
            schema: None,
            description: None,
            agent: Some("claude".into()),
            from_discussion: None,
        })
        .expect("create change");

    client
        .put_artifact("demo", "proposal", PROPOSAL, 0)
        .expect("write the proposal");
    client
        .put_artifact("demo", "tasks", TASKS, 0)
        .expect("write the tasks");

    let status = client.get_change("demo").expect("change status");
    assert_eq!(status.change_name, "demo");
    let done = client.task_done("demo", "1", &[]).expect("task done");

    Outcome {
        schema: created.schema,
        proposal: client.get_artifact("demo", "proposal").expect("read proposal").content,
        tasks: scrub_task_ids(&client.get_artifact("demo", "tasks").expect("read tasks").content),
        changes: client
            .list_changes()
            .expect("list changes")
            .changes
            .into_iter()
            .map(|c| c.name)
            .collect(),
        task_already_done: done.already_done,
        events: store
            .read_outbox(&scope(), OutboxCursor(0))
            .expect("read outbox")
            .into_iter()
            .map(|entry| entry.record.name)
            .collect(),
    }
}

#[test]
fn a_serverfs_config_serves_the_verb_flow_exactly_as_sqlite_does() {
    let fs_dir = tempfile::tempdir().expect("tempdir");
    let fs_store = build_store(&StoreConfig::ServerFs {
        path: fs_dir.path().join("store"),
    })
    .expect("build the serverfs store");

    let sqlite_dir = tempfile::tempdir().expect("tempdir");
    let sqlite_store = build_store(&StoreConfig::Sqlite {
        path: sqlite_dir.path().join("store.db"),
    })
    .expect("build the sqlite store");

    let over_fs = drive(fs_store);
    let over_sqlite = drive(sqlite_store);

    assert_eq!(
        over_fs, over_sqlite,
        "the verb flow must not be able to tell which driver is underneath"
    );
    // Guard the comparison itself: two outcomes of empty strings would match
    // just as happily.
    assert_eq!(over_fs.proposal, PROPOSAL);
    assert!(over_fs.tasks.contains("- [x] 1.1"), "the flow really ran: {}", over_fs.tasks);
    assert_eq!(over_fs.changes, ["demo"]);
    assert_eq!(
        over_fs.events,
        ["change-created", "artifact-created", "artifact-created", "task-completed"]
    );
}

#[test]
fn a_serverfs_store_serves_the_same_data_after_a_server_restart() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("store");

    // The first server runs in its own process and drives the verb flow.
    // Ending a process is the only honest restart available: the in-process
    // harness hands the store to a thread that never ends, so nothing short
    // of a real exit releases the data directory.
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "serverfs_verb_flow_child", "--ignored", "--nocapture"])
        .env(DIR_ENV, &path)
        .stdout(Stdio::piped())
        .output()
        .expect("run the first server");
    assert!(
        child.status.success(),
        "the first server's flow failed: {}",
        String::from_utf8_lossy(&child.stdout)
    );
    let before: serde_json::Value = String::from_utf8_lossy(&child.stdout)
        .lines()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("the first server reports what it wrote");

    // The second server opens the very same data directory.
    let store = build_store(&StoreConfig::ServerFs { path })
        .expect("reopen the serverfs store after the restart");
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    assert_eq!(
        client.get_artifact("demo", "proposal").expect("read the proposal back").content,
        before["proposal"].as_str().expect("proposal"),
    );
    assert_eq!(
        scrub_task_ids(&client.get_artifact("demo", "tasks").expect("read the tasks back").content),
        before["tasks"].as_str().expect("tasks"),
        "the completed task's checkbox survived the restart"
    );
    assert_eq!(
        client
            .list_changes()
            .expect("list changes")
            .changes
            .into_iter()
            .map(|c| c.name)
            .collect::<Vec<_>>(),
        ["demo"]
    );
    let events: Vec<String> = store
        .read_outbox(&scope(), OutboxCursor(0))
        .expect("read outbox")
        .into_iter()
        .map(|entry| entry.record.name)
        .collect();
    assert_eq!(
        serde_json::to_value(&events).unwrap(),
        before["events"],
        "the outbox is durable, not a buffer that died with the first process"
    );
}

/// The first server of [`a_serverfs_store_serves_the_same_data_after_a_server_restart`]:
/// drives the verb flow over a serverfs store, reports what it wrote, and
/// exits — releasing the data directory the way a real shutdown does.
/// Ignored so a normal run never picks it up; the parent runs it by name.
#[test]
#[ignore = "helper process, launched by the restart test"]
fn serverfs_verb_flow_child() {
    let path = std::env::var(DIR_ENV).expect("the parent names the data directory");
    let store = build_store(&StoreConfig::ServerFs { path: path.into() })
        .expect("the first server opens the directory");
    let outcome = drive(store);
    println!(
        "{}",
        serde_json::json!({
            "proposal": outcome.proposal,
            "tasks": outcome.tasks,
            "events": outcome.events,
        })
    );
}

#[test]
fn a_second_server_on_the_same_data_directory_is_refused() {
    // Two servers pointed at one directory is a misconfiguration; the driver
    // is single-node by design, so the second must fail to start rather than
    // interleave writes into the first one's data.
    let dir = tempfile::tempdir().expect("tempdir");
    let config = StoreConfig::ServerFs {
        path: dir.path().join("store"),
    };
    let _first = build_store(&config).expect("the first server opens the directory");

    let shown = open_error(&config);
    assert!(
        shown.contains("serverfs store") && shown.contains("unavailable"),
        "the reason names the store and why it could not open: {shown}"
    );
}
