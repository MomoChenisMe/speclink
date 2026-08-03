//! Local bundle → typed import → server read-surface round trip.

use crate::common;

use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_desktop_lib::remote::{self, TokenManager};
use speclink_remote::client::Client;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture parent")).unwrap();
    std::fs::write(path, content).unwrap();
}

fn complete_workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for (relative, content) in [
        (
            "openspec/config.yaml",
            "schema: spec-driven\nlocale: tw\n",
        ),
        ("openspec/LANGUAGE.md", "TeamStore: 團隊儲存\n"),
        (
            "openspec/changes/active-change/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-21\n",
        ),
        (
            "openspec/changes/active-change/proposal.md",
            "## Why\n\nActive proposal.\n",
        ),
        (
            "openspec/changes/active-change/design.md",
            "## Context\n\nActive design.\n",
        ),
        (
            "openspec/changes/active-change/tasks.md",
            "- [ ] 1.1 Active task\n",
        ),
        (
            "openspec/changes/active-change/specs/payments/spec.md",
            "## ADDED Requirements\n\n### Requirement: Active payment\n",
        ),
        (
            "openspec/specs/accounts/spec.md",
            "# accounts Specification\n\nCanonical accounts.\n",
        ),
        (
            "openspec/discussions/live-plan.md",
            "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n",
        ),
        (
            "openspec/discussions/archive/2026-07-20-old-plan.md",
            "---\ntopic: Old plan\nslug: old-plan\nstatus: concluded\ncreated: 2026-07-20\n---\n\nArchived discussion.\n",
        ),
        (
            "openspec/changes/archive/2026-07-20-old-change/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-20\n",
        ),
        (
            "openspec/changes/archive/2026-07-20-old-change/proposal.md",
            "## Why\n\nArchived proposal.\n",
        ),
        (
            "openspec/changes/archive/2026-07-20-old-change/tasks.md",
            "- [x] 1.1 Archived task\n",
        ),
        (
            "openspec/changes/archive/2026-07-20-old-change/specs/payments/spec.md",
            "## ADDED Requirements\n\n### Requirement: Archived payment\n",
        ),
    ] {
        write(root.path(), relative, content);
    }
    root
}

fn file_snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(base: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(current).expect("read fixture tree") {
            let entry = entry.expect("fixture entry");
            let path = entry.path();
            if path.is_dir() {
                visit(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base)
                        .expect("relative fixture path")
                        .to_string_lossy()
                        .to_string(),
                    std::fs::read(path).expect("fixture bytes"),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn complete_local_workspace_round_trips_through_import_and_every_read_surface() {
    let harness = common::harness();
    let root = complete_workspace();
    let credentials = MemoryCredentialStore::new();
    let pat = common::pat_of(&harness);
    credentials
        .set(&harness.origin, CredentialKind::Pat, &pat)
        .expect("set PAT");
    let manager = Arc::new(TokenManager::new(&harness.origin));

    let result = remote::migrate_workspace(
        root.path(),
        &harness.origin,
        "demo",
        "backend",
        &manager,
        &credentials,
    )
    .expect("bundle uploads into the empty scope");
    assert_eq!(result.report.project_revision, 1);
    assert_eq!(result.report.documents.len(), 14);
    assert_eq!(result.checkout_root, root.path().display().to_string());
    assert!(!root.path().join("openspec").exists());
    assert!(Path::new(&result.backup_path).is_dir());
    let marker = std::fs::read_to_string(root.path().join(".speclink.yaml"))
        .expect("migration writes marker");
    assert!(marker.contains("/api/speclink/v1/projects/demo"));
    assert!(marker.contains("repo: backend"));

    let client = Client::new(
        &format!("{}/api/speclink/v1/projects/demo", harness.origin),
        &pat,
        Some("backend"),
    );
    let changes = client.list_changes().expect("active list");
    assert_eq!(changes.changes[0].name, "active-change");
    assert_eq!(
        client
            .get_artifact("active-change", "proposal")
            .expect("active proposal")
            .content,
        "## Why\n\nActive proposal.\n"
    );
    assert_eq!(
        client
            .get_artifact("active-change", "specs/payments")
            .expect("active delta spec")
            .content,
        "## ADDED Requirements\n\n### Requirement: Active payment\n"
    );
    assert_eq!(
        client
            .spec_document("accounts")
            .expect("canonical spec")
            .content,
        "# accounts Specification\n\nCanonical accounts.\n"
    );
    assert_eq!(
        client
            .show_discussion("live-plan")
            .expect("live discussion")
            .content,
        "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n"
    );
    assert_eq!(
        client
            .list_discussions(true)
            .expect("archived discussions")
            .discussions[0]
            .slug,
        "old-plan"
    );
    let archived = client.archived_list().expect("archived changes");
    assert_eq!(archived.archived[0].dated_name, "2026-07-20-old-change");
    assert_eq!(
        client
            .archived_artifact("2026-07-20-old-change", "proposal.md")
            .expect("archived proposal")
            .content,
        "## Why\n\nArchived proposal.\n"
    );
    assert_eq!(
        client
            .archived_capabilities("2026-07-20-old-change")
            .expect("archived capabilities"),
        ["payments"]
    );
    assert_eq!(
        client.config().expect("workflow config").content.as_deref(),
        Some("schema: spec-driven\nlocale: tw\n")
    );
    assert_eq!(
        client.language().expect("language").content,
        "TeamStore: 團隊儲存\n"
    );
}

#[test]
fn rejected_import_leaves_every_local_file_byte_identical() {
    let harness = common::harness();
    common::seed_change(harness.store.as_ref(), "- [ ] 1.1 Existing\n");
    let root = complete_workspace();
    let before = file_snapshot(root.path());
    let credentials = MemoryCredentialStore::new();
    let pat = common::pat_of(&harness);
    credentials
        .set(&harness.origin, CredentialKind::Pat, &pat)
        .expect("set PAT");
    let manager = Arc::new(TokenManager::new(&harness.origin));

    let error = remote::migrate_workspace(
        root.path(),
        &harness.origin,
        "demo",
        "backend",
        &manager,
        &credentials,
    )
    .expect_err("CreateNew refuses a non-empty scope");
    assert!(error.contains("create-new"));
    assert_eq!(file_snapshot(root.path()), before);
    assert!(root.path().join("openspec").is_dir());
    assert!(!root.path().join(".speclink.yaml").exists());
    assert!(
        std::fs::read_dir(root.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("openspec.migrated-")),
        "no backup is created before import succeeds"
    );
}
