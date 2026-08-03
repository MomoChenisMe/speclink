//! Single-writer exclusion and the failure model of a directory that goes
//! away underneath the store.
//!
//! The lock is an OS advisory lock, which is the whole reason a dead
//! holder's lock is not a problem: the kernel drops it when the process
//! ends, so there is no stale lock to time out, adopt, or break. The
//! takeover case is therefore tested with a real second process — a
//! same-process instance could never prove that the OS released anything.

use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Revision, Scope, StoreError, TeamStore};
use speclink_store_fs::FsTeamStore;
use std::io::BufRead;
use std::process::{Command, Stdio};

/// Printed by the helper process once it holds the lock and has written its
/// document, so the parent never races the child's startup.
const READY: &str = "speclink-fs-lock-holder-ready";

/// Names the data directory for the helper process.
const DIR_ENV: &str = "SPECLINK_FS_LOCK_DIR";

fn ctx(command: &str) -> CommandContext {
    CommandContext {
        command: command.into(),
        actor: "tester".into(),
    }
}

fn scope() -> Scope {
    Scope::new(ProjectId::new("acme"), RepoId::new("web"))
}

fn auth() -> DocumentId {
    DocumentId::CanonicalSpec {
        capability: "auth".into(),
    }
}

fn create(store: &FsTeamStore, content: &str) -> Revision {
    let mut uow = store.begin_unit_of_work(&scope(), ctx("create")).unwrap();
    uow.create(auth(), content);
    store.commit(uow, vec![]).unwrap()
}

#[test]
fn a_second_instance_of_a_held_directory_is_unavailable() {
    let dir = tempfile::tempdir().unwrap();
    let first = FsTeamStore::open(dir.path()).unwrap();
    create(&first, "v1");

    match FsTeamStore::open(dir.path()) {
        Err(StoreError::Unavailable) => {}
        Err(other) => panic!("expected unavailable, got {other:?}"),
        Ok(_) => panic!("two writers opened the same data directory"),
    }

    // The refusal does not wait: the second attempt is back immediately, and
    // the holder is untouched by it.
    assert_eq!(
        first.snapshot(&scope()).unwrap().read(&auth()).unwrap().unwrap().content,
        "v1"
    );

    // Once the holder is gone, the directory opens normally and whole.
    drop(first);
    let second = FsTeamStore::open(dir.path()).expect("open after the holder released");
    assert_eq!(
        second.snapshot(&scope()).unwrap().read(&auth()).unwrap().unwrap().content,
        "v1"
    );
}

#[test]
fn a_dead_holders_lock_is_released_and_the_directory_can_be_taken_over() {
    let dir = tempfile::tempdir().unwrap();

    // A real second process takes the lock and writes a document.
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "single_writer::lock_holder_child", "--ignored", "--nocapture"])
        .env(DIR_ENV, dir.path())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn the lock holder");
    let mut out = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        let read = out.read_line(&mut line).expect("read the holder's output");
        assert_ne!(read, 0, "the lock holder exited before it was ready");
        if line.trim() == READY {
            break;
        }
    }

    // While it lives, we are locked out — no waiting, no stealing.
    match FsTeamStore::open(dir.path()) {
        Err(StoreError::Unavailable) => {}
        Err(other) => panic!("expected unavailable while another process holds it, got {other:?}"),
        Ok(_) => panic!("opened a directory another process holds"),
    }

    // Kill it outright: no unlock, no cleanup, no chance to tidy up. The
    // kernel is what releases the lock, which is exactly the case a
    // hand-rolled lock file would fail — it would leave a lock nobody can
    // prove is dead.
    child.kill().expect("kill the lock holder");
    child.wait().expect("reap the lock holder");

    let store = FsTeamStore::open(dir.path()).expect("take over from the dead holder");
    assert_eq!(
        store.snapshot(&scope()).unwrap().read(&auth()).unwrap().unwrap().content,
        "written by the holder",
        "the dead holder's committed work survives the takeover"
    );
}

/// The helper process of [`a_dead_holders_lock_is_released_and_the_directory_can_be_taken_over`]:
/// holds the data directory and waits to be killed. Ignored so a normal test
/// run never picks it up; the parent runs it by name.
#[test]
#[ignore = "helper process, launched by the takeover test"]
fn lock_holder_child() {
    let dir = std::env::var(DIR_ENV).expect("the parent names the data directory");
    let store = FsTeamStore::open(&dir).expect("the holder opens the directory");
    create(&store, "written by the holder");

    use std::io::Write;
    println!("{READY}");
    std::io::stdout().flush().unwrap();

    // Wait to be killed. The timeout only bounds a leaked process if the
    // parent dies first; the test itself never reaches it.
    std::thread::sleep(std::time::Duration::from_secs(120));
}

// --- the directory goes away ----------------------------------------------

/// Whether this process can be kept out by mode bits at all. Running as root
/// (as CI containers often do) ignores them, which would make a
/// permission-denied test assert nothing.
#[cfg(unix)]
fn mode_bits_bind() -> bool {
    use std::os::unix::fs::PermissionsExt;
    let probe = tempfile::tempdir().unwrap();
    let barred = probe.path().join("barred");
    std::fs::create_dir(&barred).unwrap();
    std::fs::set_permissions(&barred, std::fs::Permissions::from_mode(0o000)).unwrap();
    let kept_out = std::fs::write(barred.join("probe"), b"x").is_err();
    std::fs::set_permissions(&barred, std::fs::Permissions::from_mode(0o755)).unwrap();
    kept_out
}

#[cfg(unix)]
#[test]
fn a_directory_that_goes_unreachable_fails_the_commit_without_damaging_anything() {
    use std::os::unix::fs::PermissionsExt;

    if !mode_bits_bind() {
        eprintln!("skipped: this process is not bound by mode bits (running as root?)");
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let paths = speclink_store_fs::layout::ScopePaths::new(dir.path(), &scope());
    let seeded_at = {
        let store = FsTeamStore::open(dir.path()).unwrap();
        let seeded_at = create(&store, "v1");

        // The scope's content directory becomes unreachable mid-life, the
        // way a dropped NAS mount or a botched chown does.
        std::fs::set_permissions(&paths.documents(), std::fs::Permissions::from_mode(0o000))
            .unwrap();

        let mut uow = store.begin_unit_of_work(&scope(), ctx("update")).unwrap();
        uow.update(auth(), "v2", seeded_at);
        match store.commit(uow, vec![]) {
            // A refused path needs an operator, not a retry: it is a backend
            // failure, and never `permission_denied`, which in this contract
            // is about the caller's authorization, not the disk's.
            Err(StoreError::Backend { .. }) => {}
            Err(other) => panic!("expected a backend failure, got {other:?}"),
            Ok(_) => panic!("commit reported success with an unwritable directory"),
        }
        seeded_at
    };

    // Access comes back. Nothing was damaged and nothing half-landed: the
    // store reopens on exactly the state it had before the outage.
    std::fs::set_permissions(&paths.documents(), std::fs::Permissions::from_mode(0o755)).unwrap();
    let store = FsTeamStore::open(dir.path()).expect("reopen after the outage");
    let snap = store.snapshot(&scope()).unwrap();
    assert_eq!(snap.read(&auth()).unwrap().unwrap().content, "v1");
    assert_eq!(snap.revision(), seeded_at);
    assert_eq!(snap.history(&auth()).unwrap().len(), 1);
    drop(snap);

    // And it is a working store again, not a wounded one.
    let mut uow = store.begin_unit_of_work(&scope(), ctx("update")).unwrap();
    uow.update(auth(), "v2", seeded_at);
    let after = store.commit(uow, vec![]).expect("commit after recovery");
    assert_eq!(after, Revision(seeded_at.0 + 1));
    assert_eq!(
        store.snapshot(&scope()).unwrap().read(&auth()).unwrap().unwrap().content,
        "v2"
    );
}
