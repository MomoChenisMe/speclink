//! The atomicity of the FS driver: the index rename is the only moment a
//! commit becomes true, and nothing on disk means anything the index does
//! not say.
//!
//! These are the driver-specific boundaries the shared conformance suite
//! cannot express: what the directory *itself* looks like after a crash, and
//! that file timestamps carry no semantics at all.

use speclink_store::{
    CommandContext, DocumentId, EventRecord, ExpectedRevision, FaultPoint, OutboxCursor,
    ProjectId, RepoId, Revision, Scope, StoreError, TeamStore,
};
use speclink_store_fs::layout;
use speclink_store_fs::FsTeamStore;
use std::path::{Path, PathBuf};

fn ctx(command: &str) -> CommandContext {
    CommandContext {
        command: command.into(),
        actor: "tester".into(),
    }
}

fn scope(repo: &str) -> Scope {
    Scope::new(ProjectId::new("acme"), RepoId::new(repo))
}

fn spec(capability: &str) -> DocumentId {
    DocumentId::CanonicalSpec {
        capability: capability.into(),
    }
}

fn event(name: &str) -> EventRecord {
    EventRecord {
        name: name.into(),
        payload: serde_json::json!({ "event": name }),
        actor: "tester".into(),
        at: chrono::Utc::now(),
    }
}

fn create(store: &FsTeamStore, scope: &Scope, doc: &DocumentId, content: &str) -> Revision {
    let mut uow = store.begin_unit_of_work(scope, ctx("create")).unwrap();
    uow.create(doc.clone(), content);
    store.commit(uow, vec![event("created")]).unwrap()
}

fn update(
    store: &FsTeamStore,
    scope: &Scope,
    doc: &DocumentId,
    content: &str,
    read_at: Revision,
) -> Result<Revision, StoreError> {
    let mut uow = store.begin_unit_of_work(scope, ctx("update"))?;
    uow.update(doc.clone(), content, read_at);
    store.commit(uow, vec![event("edited")])
}

/// File names directly inside `dir`, sorted. Absent directory reads as empty.
fn names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

/// Every file under `root`, recursively.
fn all_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

// --- CAS ------------------------------------------------------------------

#[test]
fn concurrent_commits_on_the_same_revision_conflict_and_the_loser_leaves_no_trace() {
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    let scope = scope("web");
    let auth = spec("auth");
    let billing = spec("billing");
    let seeded_at = create(&store, &scope, &auth, "base");

    // Two writers read the same revision. The loser also stages a second,
    // independent op: a rejected commit is rejected whole, so that op must
    // not survive either.
    let mut winner = store.begin_unit_of_work(&scope, ctx("win")).unwrap();
    winner.update(auth.clone(), "winner", seeded_at);
    let mut loser = store.begin_unit_of_work(&scope, ctx("lose")).unwrap();
    loser.update(auth.clone(), "loser", seeded_at);
    loser.create(billing.clone(), "loser-only");

    let winning_rev = store.commit(winner, vec![]).unwrap();
    match store.commit(loser, vec![]) {
        Err(StoreError::RevisionConflict {
            doc,
            expected,
            actual,
        }) => {
            assert_eq!(doc.doc, auth);
            assert_eq!(expected, ExpectedRevision::At(seeded_at));
            assert_eq!(actual, Some(winning_rev));
        }
        other => panic!("expected revision conflict, got {other:?}"),
    }

    let snap = store.snapshot(&scope).unwrap();
    assert_eq!(snap.read(&auth).unwrap().unwrap().content, "winner");
    assert_eq!(
        snap.read(&billing).unwrap(),
        None,
        "the loser's other op must not have landed"
    );
    assert_eq!(snap.revision(), winning_rev);
}

// --- the four fault points ------------------------------------------------

#[test]
fn a_crash_at_any_fault_point_leaves_the_commit_never_happened_and_no_orphans() {
    for point in [
        FaultPoint::AfterDocWrites,
        FaultPoint::AfterHistoryAppend,
        FaultPoint::BeforeOutboxAppend,
        FaultPoint::AfterOutboxAppend,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let scope = scope("web");
        let auth = spec("auth");

        let seeded_at = {
            let store = FsTeamStore::open(dir.path()).unwrap();
            let seeded_at = create(&store, &scope, &auth, "v1");

            store.crash_at(point);
            assert!(
                update(&store, &scope, &auth, "v2", seeded_at).is_err(),
                "{point:?}: commit reported success while armed to crash"
            );
            seeded_at
        };

        // Reopen the same directory: the crashed commit never happened, in
        // every facet at once.
        let store = FsTeamStore::open(dir.path()).unwrap();
        let snap = store.snapshot(&scope).unwrap();
        assert_eq!(
            snap.read(&auth).unwrap().unwrap().content,
            "v1",
            "{point:?}: document content moved"
        );
        assert_eq!(snap.revision(), seeded_at, "{point:?}: revision moved");
        assert_eq!(
            snap.history(&auth).unwrap().len(),
            1,
            "{point:?}: history leaked a record"
        );
        let outbox: Vec<String> = store
            .read_outbox(&scope, OutboxCursor(0))
            .unwrap()
            .into_iter()
            .map(|e| e.record.name)
            .collect();
        assert_eq!(outbox, ["created"], "{point:?}: outbox leaked an entry");

        // …and the files the doomed commit wrote are swept away, not left to
        // accumulate. Only the seeded revision's records remain.
        let paths = layout::ScopePaths::new(dir.path(), &scope);
        assert_eq!(
            names(&paths.documents()),
            ["cs.auth.1"],
            "{point:?}: orphan content file survived"
        );
        assert_eq!(
            names(&paths.history()),
            ["cs.auth.1.json"],
            "{point:?}: orphan history file survived"
        );
        assert_eq!(
            names(&paths.outbox()),
            ["1.json"],
            "{point:?}: orphan outbox file survived"
        );
        assert!(
            !paths.index_staging().exists(),
            "{point:?}: staged index survived"
        );
    }
}

#[test]
fn a_failed_commits_records_are_never_adopted_by_a_later_one() {
    // A commit can fail without crashing the process: the store absorbs the
    // error and stays usable, so nothing reopens and nothing sweeps. Its
    // records are already on disk at the revision it would have published —
    // and that revision is still unused, so the *next* commit takes the very
    // same number. Unless the abandoned records are gone by then, publishing
    // that number vouches for work that never happened.
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    let scope = scope("web");
    let auth = spec("auth");
    let billing = spec("billing");
    let seeded_at = create(&store, &scope, &auth, "v1");

    store.fail_outbox_append();
    assert!(
        update(&store, &scope, &auth, "v2", seeded_at).is_err(),
        "the armed commit fails"
    );

    // The next commit touches a *different* document, so nothing overwrites
    // the abandoned records by coincidence.
    create(&store, &scope, &billing, "billing v1");

    let snap = store.snapshot(&scope).unwrap();
    let document = snap.read(&auth).unwrap().unwrap();
    assert_eq!(document.content, "v1");
    assert_eq!(document.revision, seeded_at);
    assert_eq!(
        snap.history(&auth).unwrap().len(),
        1,
        "history must not gain a record from the commit that failed"
    );
    assert_eq!(
        names(&layout::ScopePaths::new(dir.path(), &scope).history()),
        ["cs.auth.1.json", "cs.billing.2.json"],
        "the abandoned history file is gone, not merely hidden"
    );

    // The same holds across a reopen: no record of the failed commit anywhere.
    drop(snap);
    drop(store);
    let store = FsTeamStore::open(dir.path()).unwrap();
    let snap = store.snapshot(&scope).unwrap();
    assert_eq!(snap.read(&auth).unwrap().unwrap().content, "v1");
    assert_eq!(snap.history(&auth).unwrap().len(), 1);
}

#[test]
fn sweeping_a_failed_commit_leaves_an_open_snapshots_fixed_point_readable() {
    // Sweeping mid-life is not the same job as sweeping at open. At open
    // nobody is reading, so every superseded revision can go. While the store
    // is live, a snapshot may still be resolving an older revision through
    // the index it read — those files are published history of the store, not
    // litter, and taking them away would report a perfectly intact store as
    // corrupt.
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    let scope = scope("web");
    let auth = spec("auth");
    let billing = spec("billing");
    let seeded_at = create(&store, &scope, &auth, "v1");

    let snap = store.snapshot(&scope).unwrap();
    update(&store, &scope, &auth, "v2", seeded_at).unwrap();

    // A failure marks the scope for sweeping; the next commit does the sweep.
    store.fail_outbox_append();
    assert!(update(&store, &scope, &auth, "v3", Revision(2)).is_err());
    create(&store, &scope, &billing, "billing v1");

    // The snapshot still reads its own fixed point.
    let document = snap
        .read(&auth)
        .expect("the snapshot's revision is still readable")
        .expect("the document is still there");
    assert_eq!(document.content, "v1");
    assert_eq!(document.revision, seeded_at);
    assert_eq!(snap.revision(), seeded_at);

    // …and the abandoned commit's records are still gone.
    assert_eq!(
        names(&layout::ScopePaths::new(dir.path(), &scope).history()),
        ["cs.auth.1.json", "cs.auth.2.json", "cs.billing.3.json"],
    );
}

// --- timestamps carry no meaning ------------------------------------------

#[test]
fn tampered_mtimes_change_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let scope = scope("web");
    let auth = spec("auth");
    let billing = spec("billing");

    let (before_content, before_history, before_outbox) = {
        let store = FsTeamStore::open(dir.path()).unwrap();
        let r1 = create(&store, &scope, &auth, "v1");
        let r2 = update(&store, &scope, &auth, "v2", r1).unwrap();
        update(&store, &scope, &auth, "v3", r2).unwrap();
        create(&store, &scope, &billing, "billing v1");

        let snap = store.snapshot(&scope).unwrap();
        let content = snap.read(&auth).unwrap().unwrap();
        let history: Vec<Revision> = snap
            .history(&auth)
            .unwrap()
            .into_iter()
            .map(|r| r.revision)
            .collect();
        let outbox: Vec<u64> = store
            .read_outbox(&scope, OutboxCursor(0))
            .unwrap()
            .into_iter()
            .map(|e| e.seq)
            .collect();
        (content, history, outbox)
    };

    // Rewrite every timestamp in the directory, oldest file newest, so that
    // any ordering that leaned on mtime would come out reversed.
    let files = all_files(dir.path());
    let base = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    for (i, path) in files.iter().enumerate() {
        let when = base + std::time::Duration::from_secs((files.len() - i) as u64 * 3600);
        let handle = std::fs::File::options().write(true).open(path).unwrap();
        handle
            .set_times(std::fs::FileTimes::new().set_modified(when).set_accessed(when))
            .unwrap();
    }

    let store = FsTeamStore::open(dir.path()).unwrap();
    let snap = store.snapshot(&scope).unwrap();
    assert_eq!(snap.read(&auth).unwrap().unwrap(), before_content);
    assert_eq!(snap.revision(), Revision(4));
    assert_eq!(
        snap.history(&auth)
            .unwrap()
            .into_iter()
            .map(|r| r.revision)
            .collect::<Vec<_>>(),
        before_history,
        "history order comes from revisions, not timestamps"
    );
    assert_eq!(
        store
            .read_outbox(&scope, OutboxCursor(0))
            .unwrap()
            .into_iter()
            .map(|e| e.seq)
            .collect::<Vec<_>>(),
        before_outbox,
        "outbox order comes from sequence numbers, not timestamps"
    );
}

// --- one index read is the consistency boundary ---------------------------

#[test]
fn a_snapshot_is_the_one_index_read_it_started_from() {
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    let scope = scope("web");
    let auth = spec("auth");
    let billing = spec("billing");

    let mut uow = store.begin_unit_of_work(&scope, ctx("seed")).unwrap();
    uow.create(auth.clone(), "auth v1");
    uow.create(billing.clone(), "billing v1");
    let seeded_at = store.commit(uow, vec![]).unwrap();

    let snap = store.snapshot(&scope).unwrap();

    // A later commit republishes the index under the reader. Because the
    // snapshot resolved its references from the index it read, and content
    // files are immutable, it keeps seeing its own fixed point — in full,
    // never a mix of the two commits.
    let mut uow = store.begin_unit_of_work(&scope, ctx("edit")).unwrap();
    uow.update(auth.clone(), "auth v2", seeded_at);
    uow.update(billing.clone(), "billing v2", seeded_at);
    let after = store.commit(uow, vec![]).unwrap();

    assert_eq!(snap.read(&auth).unwrap().unwrap().content, "auth v1");
    assert_eq!(snap.read(&billing).unwrap().unwrap().content, "billing v1");
    assert_eq!(snap.revision(), seeded_at);
    assert_eq!(
        snap.history(&auth).unwrap().len(),
        1,
        "history of the fixed point excludes the later commit"
    );

    // A snapshot taken now sees the new fixed point.
    let fresh = store.snapshot(&scope).unwrap();
    assert_eq!(fresh.read(&auth).unwrap().unwrap().content, "auth v2");
    assert_eq!(fresh.revision(), after);
}

// --- the project revision is the project's, not the scope's ---------------

#[test]
fn the_project_revision_advances_across_the_repos_of_one_project() {
    // The contract's revision is a *project* revision: every committed unit
    // of work in the project advances it by one, whichever repo it lands in.
    // The FS layout keeps one index per scope, so this is the case where a
    // per-scope counter would silently diverge from the other drivers.
    let dir = tempfile::tempdir().unwrap();
    let store = FsTeamStore::open(dir.path()).unwrap();
    let web = scope("web");
    let api = scope("api");
    let auth = spec("auth");

    assert_eq!(create(&store, &web, &auth, "web v1"), Revision(1));
    assert_eq!(create(&store, &api, &auth, "api v1"), Revision(2));
    assert_eq!(
        update(&store, &web, &auth, "web v2", Revision(1)).unwrap(),
        Revision(3)
    );

    // Both scopes report the project's revision, and each still reads only
    // its own document.
    assert_eq!(store.snapshot(&web).unwrap().revision(), Revision(3));
    assert_eq!(store.snapshot(&api).unwrap().revision(), Revision(3));
    assert_eq!(
        store.snapshot(&api).unwrap().read(&auth).unwrap().unwrap().content,
        "api v1"
    );

    // It survives a reopen: the revision is derived from what is on disk,
    // never from a counter held in memory.
    drop(store);
    let store = FsTeamStore::open(dir.path()).unwrap();
    assert_eq!(store.snapshot(&web).unwrap().revision(), Revision(3));
    assert_eq!(
        update(&store, &api, &auth, "api v2", Revision(2)).unwrap(),
        Revision(4)
    );
}
