//! Requirement「兩個儲存物件開同一份資料」.

use crate::support::{
    all, child_dir, child_report, commit, key, open_error, put, report, scope, spawn_child, tree, v,
};
use speclink_store_port::{Expected, StorePort};
use speclink_store_port_fs::{FsPort, OpenError};

#[test]
#[ignore = "helper process of a_second_process_finds_the_directory_in_use"]
fn child_tries_to_open() {
    match FsPort::open(child_dir()) {
        Ok(_) => report("opened"),
        Err(OpenError::InUse) => report("in-use"),
        Err(other) => report(&format!("other: {other:?}")),
    }
}

#[test]
#[ignore = "helper process of a_killed_holder_does_not_keep_the_directory"]
fn child_holds_the_directory() {
    let store = FsPort::open(child_dir()).expect("the holder opens the directory");
    store
        .commit(
            &scope("s"),
            &commit(
                vec![put("k", "written by the holder", Expected::Absent)],
                Vec::new(),
            ),
        )
        .unwrap();
    report("ready");
    // Wait for the parent to kill this process. The limit only bounds a
    // process that the parent did not stop.
    std::thread::sleep(std::time::Duration::from_secs(120));
}

#[test]
fn a_second_process_finds_the_directory_in_use() {
    let dir = tempfile::tempdir().unwrap();
    let s = scope("s");
    let store = FsPort::open(dir.path()).unwrap();
    store
        .commit(
            &s,
            &commit(vec![put("k", "v1", Expected::Absent)], Vec::new()),
        )
        .unwrap();
    let before = tree(dir.path());

    let mut child = spawn_child("two_objects::child_tries_to_open", dir.path());
    let outcome = child_report(&mut child);
    child.wait().unwrap();

    assert_eq!(outcome, "in-use");
    assert_eq!(tree(dir.path()), before);
    // The first object continues to work.
    assert_eq!(
        store.commit(
            &s,
            &commit(vec![put("k", "v2", Expected::At(v(1)))], Vec::new())
        ),
        Ok(v(2))
    );
}

#[test]
fn a_second_object_in_the_same_process_finds_the_directory_in_use() {
    let dir = tempfile::tempdir().unwrap();
    let _store = FsPort::open(dir.path()).unwrap();
    assert_eq!(open_error(FsPort::open(dir.path())), OpenError::InUse);
}

#[test]
fn a_killed_holder_does_not_keep_the_directory() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = spawn_child("two_objects::child_holds_the_directory", dir.path());
    assert_eq!(child_report(&mut child), "ready");
    assert_eq!(open_error(FsPort::open(dir.path())), OpenError::InUse);

    // No unlock and no cleanup: the operating system releases the lock.
    child.kill().expect("kill the holder");
    child.wait().expect("reap the holder");

    let store = FsPort::open(dir.path()).expect("open after the holder ended");
    assert_eq!(
        store.read(&scope("s"), &[key("k")]).unwrap()[0]
            .as_ref()
            .unwrap()
            .content,
        "written by the holder"
    );
}

#[test]
fn scope_names_stay_inside_the_directory_and_do_not_mix() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = FsPort::open(&root).unwrap();
    let names = [
        "abc",
        "ABC",
        "a.b",
        "..",
        "../../escape",
        "Org/Repo",
        "/tmp/x",
    ];

    for name in names {
        store
            .commit(
                &scope(name),
                &commit(vec![put("k", name, Expected::Absent)], Vec::new()),
            )
            .unwrap();
    }

    for name in names {
        let listing = store.list(&scope(name), &all()).unwrap();
        assert_eq!(listing.version, v(1), "scope {name}");
        assert_eq!(
            store.read(&scope(name), &[key("k")]).unwrap()[0]
                .as_ref()
                .unwrap()
                .content,
            name
        );
    }
    // Nothing was written next to the store's directory.
    let beside: Vec<_> = std::fs::read_dir(parent.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(beside, ["store"]);
    // One directory per scope, directly below `scopes`.
    assert_eq!(
        std::fs::read_dir(root.join("scopes")).unwrap().count(),
        names.len()
    );
}
