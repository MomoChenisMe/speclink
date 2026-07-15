//! The server over the `postgres` store driver (postgres-team-store spec
//! 「連線失敗分類與版本守門」, 「密碼來源紀律」).
//!
//! The point of a store contract is that the layer above cannot tell which
//! driver is underneath. So this drives the same verb flow through a
//! postgres-backed server and a sqlite-backed one and compares what came out —
//! a difference here means the abstraction leaks, whichever side is "right".
//!
//! Every test needs a real PostgreSQL instance named by
//! `SPECLINK_TEST_POSTGRES_URL`, so this target runs its own harness: absent
//! one, it must report `skipped` rather than the `passed` that libtest's only
//! runtime outcome would give it.

mod common;
mod pg;

use speclink_protocol::command::CreateChangeRequest;
use speclink_remote::client::Client;
use speclink_server::build_store;
use speclink_server::config::StoreConfig;
use speclink_server::state::SharedStore;
use speclink_store::{OutboxCursor, ProjectId, RepoId, Scope};
use std::process::{Command, Stdio};

const PROPOSAL: &str = "## Why\n\npostgres drives the same flow.\n";
const TASKS: &str = "## 1. Work\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n";

/// Names the connection URL for the helper process of the restart test.
const URL_ENV: &str = "SPECLINK_POSTGRES_E2E_URL";
/// Asks the helper process to play the password-warning role instead.
const WARN_ENV: &str = "SPECLINK_POSTGRES_E2E_WARN_URL";

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

/// Blank out the task ids the engine mints. They are ULIDs — new on every run,
/// by design — so comparing two independent flows verbatim would only ever
/// prove that time passed.
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
/// That is why the restart test below needs a real second process.
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
        proposal: client
            .get_artifact("demo", "proposal")
            .expect("read proposal")
            .content,
        tasks: scrub_task_ids(
            &client
                .get_artifact("demo", "tasks")
                .expect("read tasks")
                .content,
        ),
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

fn a_postgres_config_serves_the_verb_flow_exactly_as_sqlite_does() {
    let db = pg::TestDb::new();
    let postgres_store = build_store(&StoreConfig::Postgres {
        url: db.url().to_string(),
    })
    .expect("build the postgres store");

    let sqlite_dir = tempfile::tempdir().expect("tempdir");
    let sqlite_store = build_store(&StoreConfig::Sqlite {
        path: sqlite_dir.path().join("store.db"),
    })
    .expect("build the sqlite store");

    let over_postgres = drive(postgres_store);
    let over_sqlite = drive(sqlite_store);

    assert_eq!(
        over_postgres, over_sqlite,
        "the verb flow must not be able to tell which driver is underneath"
    );
    // Guard the comparison itself: two outcomes of empty strings would match
    // just as happily.
    assert_eq!(over_postgres.proposal, PROPOSAL);
    assert!(
        over_postgres.tasks.contains("- [x] 1.1"),
        "the flow really ran: {}",
        over_postgres.tasks
    );
    assert_eq!(over_postgres.changes, ["demo"]);
    assert_eq!(
        over_postgres.events,
        [
            "change-created",
            "artifact-created",
            "artifact-created",
            "task-completed"
        ]
    );
}

fn a_postgres_store_serves_the_same_data_after_a_server_restart() {
    let db = pg::TestDb::new();

    // The first server runs in its own process and drives the verb flow.
    // Ending a process is the only honest restart available: the in-process
    // harness hands the store to a thread that never ends.
    let child = Command::new(std::env::current_exe().unwrap())
        .env(URL_ENV, db.url())
        .stdout(Stdio::piped())
        .output()
        .expect("run the first server");
    assert!(
        child.status.success(),
        "the first server's flow failed: {}{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let before: serde_json::Value = String::from_utf8_lossy(&child.stdout)
        .lines()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("the first server reports what it wrote");

    // The second server opens the very same database.
    let store = build_store(&StoreConfig::Postgres {
        url: db.url().to_string(),
    })
    .expect("reopen the postgres store after the restart");
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    assert_eq!(
        client
            .get_artifact("demo", "proposal")
            .expect("read the proposal back")
            .content,
        before["proposal"].as_str().expect("proposal"),
    );
    assert_eq!(
        scrub_task_ids(
            &client
                .get_artifact("demo", "tasks")
                .expect("read the tasks back")
                .content
        ),
        before["tasks"].as_str().expect("tasks"),
        "the completed task's checkbox survived the restart"
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

/// A password may come from the environment, and a password left in the config
/// is called out — but neither shape stops the server from starting.
///
/// The password is moved out of the configured URL rather than assumed absent
/// from it: CI reaches its database with a password in the URL and a laptop on
/// trust auth does not, so a test written against either shape would pass on
/// one machine and fail on the other. Moving it also means that where the
/// server really demands a password, the passwordless case can only connect if
/// the environment variable did its job.
fn the_password_source_is_a_warning_not_a_gate() {
    let db = pg::TestDb::new();
    let (bare_url, password) = db.url_without_password();

    let mut passwordless = Command::new(std::env::current_exe().unwrap());
    passwordless.env(WARN_ENV, &bare_url);
    if let Some(password) = &password {
        passwordless.env(speclink_store_postgres::PASSWORD_VAR, password);
    }
    let quiet = passwordless.output().expect("run the passwordless case");
    let stderr = String::from_utf8_lossy(&quiet.stderr);
    assert!(
        quiet.status.success(),
        "a url without a password must still start, on the environment's password: {stderr}"
    );
    assert!(
        !stderr.contains("carries a password"),
        "a url without a password was warned about: {stderr}"
    );

    // The same store, reached by a connection string that carries the password
    // inline: it still starts, and stderr says where the password belongs.
    let loud = Command::new(std::env::current_exe().unwrap())
        .env(
            WARN_ENV,
            db.url_with_password(password.as_deref().unwrap_or("ignored-under-trust-auth")),
        )
        .output()
        .expect("run the embedded-password case");
    let stderr = String::from_utf8_lossy(&loud.stderr);
    assert!(
        loud.status.success(),
        "a url with an embedded password must still start: {stderr}"
    );
    assert!(
        stderr.contains("carries a password") && stderr.contains("SPECLINK_POSTGRES_PASSWORD"),
        "stderr should advise the environment variable: {stderr}"
    );
}

fn main() {
    // The first server of the restart test: drive the verb flow, report what it
    // wrote, and exit — releasing the database the way a real shutdown does.
    if let Ok(url) = std::env::var(URL_ENV) {
        let store = build_store(&StoreConfig::Postgres { url }).expect("the first server opens");
        let outcome = drive(store);
        println!(
            "{}",
            serde_json::json!({
                "proposal": outcome.proposal,
                "tasks": outcome.tasks,
                "events": outcome.events,
            })
        );
        return;
    }

    // The startup-warning role: open the store the way startup does, so the
    // parent can read what reached stderr.
    if let Ok(url) = std::env::var(WARN_ENV) {
        build_store(&StoreConfig::Postgres { url }).expect("the store opens");
        return;
    }

    pg::run(&[
        (
            "a_postgres_config_serves_the_verb_flow_exactly_as_sqlite_does",
            a_postgres_config_serves_the_verb_flow_exactly_as_sqlite_does,
        ),
        (
            "a_postgres_store_serves_the_same_data_after_a_server_restart",
            a_postgres_store_serves_the_same_data_after_a_server_restart,
        ),
        (
            "the_password_source_is_a_warning_not_a_gate",
            the_password_source_is_a_warning_not_a_gate,
        ),
    ]);
}
