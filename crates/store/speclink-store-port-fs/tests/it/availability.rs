//! Requirement「可用性檢查與失敗的分類」.

use crate::support::{commit, new_store, put, scope};
use speclink_store_port::{Expected, PortError, StorePort};
use speclink_store_port_fs::FsPort;

#[test]
fn probe_succeeds_on_an_open_store() {
    let (_dir, store) = new_store();
    assert_eq!(store.probe(), Ok(()));
}

/// Unix only: Windows does not let a directory move while a file in it is
/// open, so the situation cannot occur there.
#[cfg(unix)]
#[test]
fn probe_fails_after_the_directory_is_moved_away() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = FsPort::open(&root).unwrap();
    let s = scope("s");
    store
        .commit(
            &s,
            &commit(vec![put("k", "x", Expected::Absent)], Vec::new()),
        )
        .unwrap();

    std::fs::rename(&root, dir.path().join("moved")).unwrap();

    let result = store.probe();
    assert!(
        matches!(
            result,
            Err(PortError::Backend { .. } | PortError::Unavailable { .. })
        ),
        "{result:?}"
    );
}

/// Whether permission bits keep this process out. A process that runs as
/// root ignores them, and the test below would then prove nothing.
#[cfg(unix)]
fn permissions_bind() -> bool {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let kept_out = std::fs::write(dir.path().join("probe"), "x").is_err();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    kept_out
}

#[cfg(unix)]
#[test]
fn probe_fails_when_the_directory_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    if !permissions_bind() {
        eprintln!("note: this process ignores permission bits, so nothing is checked");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let store = FsPort::open(&root).unwrap();

    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = store.probe();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();

    assert!(
        matches!(result, Err(PortError::Backend { .. })),
        "{result:?}"
    );
    assert_eq!(store.probe(), Ok(()));
}
