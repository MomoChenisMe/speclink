//! Backups made by an older version (server-backup「舊版的備份檔通過檢查」
//! 「舊版的備份還原到這個版本」「舊版的備份裡有換不成鍵的文件」).
//!
//! `tests/fixtures/backup-format-1.tar` is a backup in format version 1, made
//! once by speclink-server 0.12.0. `backup-format-1.md` next to it says what
//! it holds; the numbers and names asserted here come from that file.

use super::backup_restore::{read_tar, target_config, write_tar};
use crate::common;

use speclink_remote::client::Client;
use speclink_server::events::EventHub;
use speclink_server::identity::{IdentitySqlite, IdentityStore};
use speclink_server::state::AppState;
use speclink_store_port::{KeyPrefix, ScopeId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;

const NOTES: &str = include_str!("../../fixtures/backup-format-1.md");

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backup-format-1.tar")
}

/// The plaintext of the fixture user's access key, from the notes.
fn fixture_pat() -> &'static str {
    NOTES
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("spk_pat_"))
        .expect("the notes give the access key")
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// Put the manifest back as the root of trust after a member or the manifest
/// itself was changed on purpose: every member's digest, then the manifest's.
fn reroot(members: &mut BTreeMap<String, Vec<u8>>, change: impl FnOnce(&mut serde_json::Value)) {
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&members["manifest.json"]).expect("manifest json");
    change(&mut manifest);
    for member in manifest["members"].as_array_mut().expect("member digests") {
        let name = member["name"].as_str().expect("member name").to_string();
        member["digest"] = sha256(&members[&name]).into();
    }
    let bytes = serde_json::to_vec_pretty(&manifest).expect("manifest bytes");
    members.insert("manifest.json.sha256".into(), sha256(&bytes).into_bytes());
    members.insert("manifest.json".into(), bytes);
}

/// A copy of the fixture in `dir`, changed by `change`.
fn changed_fixture(dir: &Path, change: impl FnOnce(&mut BTreeMap<String, Vec<u8>>)) -> PathBuf {
    let mut members = read_tar(&fixture());
    change(&mut members);
    let path = dir.join("changed.tar");
    write_tar(&path, &members);
    path
}

/// A tar with one more member, under a name the tar builder itself would
/// refuse: the name goes into the header as it is.
fn with_raw_member(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join("raw.tar");
    let mut builder = tar::Builder::new(std::fs::File::create(&path).expect("create tar"));
    for (member, bytes) in read_tar(&fixture()) {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, member, bytes.as_slice()).expect("append");
    }
    let mut header = tar::Header::new_gnu();
    header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
    header.set_size(1);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, &b"x"[..]).expect("append the raw member");
    builder.finish().expect("finish tar");
    path
}

fn verify_backup(input: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_speclink-server"))
        .arg("verify-backup")
        .arg("--input")
        .arg(input)
        .output()
        .expect("run verify-backup")
}

/// The restore target of `target_config(dir)` was not written: its identity
/// database was not made. (The store file can exist, empty: the check that
/// the target is empty opens it.)
fn no_identity_was_placed(dir: &Path) {
    assert!(!dir.join("tgt-identity.db").exists(), "the identity database was not made");
}

/// Nothing at all was made for the restore target of `target_config(dir)`:
/// the restore stopped before it opened the target.
fn nothing_was_written(dir: &Path) {
    assert!(!dir.join("tgt-store.db").exists(), "the store was not made");
    no_identity_was_placed(dir);
}

#[test]
fn verify_backup_accepts_the_old_format_and_reports_version_1() {
    let _gate = common::acquire_process_gate();
    let out = verify_backup(&fixture());
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("格式版本 1"), "the report names format version 1: {stdout}");

    let report = speclink_server::backup::verify(&fixture()).expect("the old backup verifies");
    assert_eq!((report.backup_format_version, report.scope_count), (1, 2));
}

#[test]
fn verify_backup_refuses_format_version_3() {
    let _gate = common::acquire_process_gate();
    let dir = tempfile::tempdir().expect("tempdir");
    let input = changed_fixture(dir.path(), |members| {
        reroot(members, |manifest| manifest["backup_format_version"] = 3.into());
    });
    let out = verify_backup(&input);
    assert!(!out.status.success(), "a format version this build does not know is refused");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("version 3"), "the reason names the version: {stderr}");

    let err = speclink_server::backup::restore(&target_config(dir.path()), &input).expect_err("restore refuses it too");
    assert!(err.to_string().contains("version"), "{err}");
    nothing_was_written(dir.path());
}

// --- what a backup file can hold that this build did not write (audit) ---

#[test]
fn a_member_name_that_leaves_the_archive_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in ["../evil.json", "bundles/../../evil.json", "/etc/evil.json"] {
        let input = with_raw_member(dir.path(), name);
        let err = speclink_server::backup::verify(&input).expect_err("the member name is refused");
        assert!(err.to_string().contains("member name"), "{name}: {err}");
        let err = speclink_server::backup::restore(&target_config(dir.path()), &input).expect_err("restore refuses it too");
        assert!(err.to_string().contains("member name"), "{name}: {err}");
        nothing_was_written(dir.path());
    }
}

#[test]
fn a_manifest_field_this_build_does_not_know_is_ignored() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = changed_fixture(dir.path(), |members| {
        reroot(members, |manifest| manifest["added_by_a_later_version"] = serde_json::json!({ "n": 1 }));
    });
    let report = speclink_server::backup::verify(&input).expect("the backup still verifies");
    assert_eq!(report.backup_format_version, 1);
}

#[test]
fn a_format_version_that_is_not_a_small_whole_number_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    for version in [serde_json::json!("1"), serde_json::json!(-1), serde_json::json!(1.5), serde_json::Value::Null] {
        let shown = version.to_string();
        let input = changed_fixture(dir.path(), |members| {
            reroot(members, |manifest| manifest["backup_format_version"] = version);
        });
        let err = speclink_server::backup::verify(&input).expect_err("the format version is refused");
        assert!(err.to_string().contains("manifest.json"), "{shown}: {err}");
        assert!(speclink_server::backup::restore(&target_config(dir.path()), &input).is_err(), "{shown}");
        nothing_was_written(dir.path());
    }
}

#[test]
fn a_changed_bit_in_the_old_backup_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = changed_fixture(dir.path(), |members| {
        members.get_mut("bundles/1.json").expect("the second bundle")[0] ^= 0x01;
    });
    let err = speclink_server::backup::verify(&input).expect_err("a changed member is refused");
    assert!(err.to_string().contains("bundles/1.json"), "the error names the member: {err}");

    let err = speclink_server::backup::restore(&target_config(dir.path()), &input).expect_err("restore refuses it too");
    assert!(err.to_string().contains("bundles/1.json"), "{err}");
    nothing_was_written(dir.path());
}

#[test]
fn the_old_backup_restores_into_an_empty_target_and_serves_its_data() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = target_config(dir.path());

    let report = speclink_server::backup::restore(&config, &fixture()).expect("the restore succeeds");
    assert!(report.ok, "the restore validates green: {:?}", report.differences);
    assert_eq!(report.scopes_checked, 2);

    // Start a server on the restored target.
    let store = speclink_server::build_store(&config.store).expect("open the restored store");
    let identity = speclink_server::build_identity(&config.identity).expect("open the restored identity");
    let events = EventHub::new(store.clone(), config.events.clone());
    let base = common::start(AppState::new(store.clone(), identity, Arc::new(config), events));

    // Every document is under the key of its kind: the nine kinds of the
    // old format.
    let keys = |repo: &str| -> Vec<String> {
        let scope = ScopeId::new(format!("demo/{repo}")).unwrap();
        let listing = store.list(&scope, &KeyPrefix::new("").unwrap()).expect("list the scope");
        listing.entries.into_iter().map(|entry| entry.key.as_str().to_string()).collect()
    };
    assert_eq!(
        keys("backend"),
        [
            "LANGUAGE.md",
            "archive/changes/2026-10-25-initial-setup/.meta.yaml",
            "archive/changes/2026-10-25-initial-setup/proposal.md",
            "archive/changes/2026-10-25-initial-setup/specs/accounts/spec.md",
            "archive/changes/2026-10-25-initial-setup/tasks.md",
            "archive/discussions/2026-10-20-naming.md",
            "board/order",
            "changes/add-auth/.evidence.json",
            "changes/add-auth/.meta.yaml",
            "changes/add-auth/design.md",
            "changes/add-auth/proposal.md",
            "changes/add-auth/specs/auth/spec.md",
            "changes/add-auth/tasks.md",
            "changes/add-billing/.meta.yaml",
            "changes/add-billing/proposal.md",
            "changes/add-billing/tasks.md",
            "config.yaml",
            "discussions/rate-limiting.md",
            "specs/accounts/spec.md",
        ]
    );
    assert_eq!(
        keys("web"),
        ["changes/new-landing/.meta.yaml", "changes/new-landing/proposal.md", "config.yaml"]
    );

    // The user's access key from before the backup still opens the server.
    let project = format!("{base}/api/speclink/v1/projects/demo");
    let client = Client::new(&project, fixture_pat(), Some("backend"));

    // list, show and plan give the changes, the spec and the order the old
    // version gave.
    let changes = client.list_changes().expect("the old access key authenticates").changes;
    let listed: Vec<(&str, usize, usize)> =
        changes.iter().map(|c| (c.name.as_str(), c.completed_tasks, c.total_tasks)).collect();
    assert_eq!(listed, [("add-auth", 1, 2), ("add-billing", 0, 1)]);
    let specs: Vec<String> = client.list_specs().expect("list specs").specs.into_iter().map(|s| s.id).collect();
    assert_eq!(specs, ["accounts"]);
    assert_eq!(
        client.get_artifact("add-auth", "proposal").expect("show the proposal").content,
        "## Why\n\nUsers need to sign in.\n\n## What Changes\n\n- add a login route\n"
    );
    let plan: serde_json::Value = ureq::get(&format!("{project}/plan"))
        .set("Authorization", &format!("Bearer {}", fixture_pat()))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("GET /plan")
        .into_json()
        .expect("plan JSON");
    assert_eq!(plan["waves"][0]["changes"], serde_json::json!(["add-auth"]), "{plan}");
    assert_eq!(plan["waves"][1]["changes"], serde_json::json!(["add-billing"]), "{plan}");
    let board = client.board_order().expect("board order").content.expect("the board order was restored");
    assert!(board.contains("\"add-billing\":\"a1\""), "{board}");

    // The other repo of the project came back too.
    let web = Client::new(&project, fixture_pat(), Some("web"));
    let names: Vec<String> = web.list_changes().expect("list web").changes.into_iter().map(|c| c.name).collect();
    assert_eq!(names, ["new-landing"]);

    // The identity came from the snapshot: the one user of the notes.
    let users =
        IdentitySqlite::open(dir.path().join("tgt-identity.db")).expect("identity").list_users().expect("users");
    let emails: Vec<&str> = users.iter().map(|user| user.email.as_str()).collect();
    assert_eq!(emails, ["fixture-member@example.com"]);
}

#[test]
fn an_old_backup_with_a_document_of_an_unknown_kind_is_refused_before_any_write() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = changed_fixture(dir.path(), |members| {
        // The first bundle is demo/backend.
        let mut bundle: serde_json::Value =
            serde_json::from_slice(&members["bundles/0.json"]).expect("bundle json");
        bundle["documents"][0]["doc"]["kind"] = "mystery".into();
        members.insert("bundles/0.json".into(), serde_json::to_vec_pretty(&bundle).unwrap());
        reroot(members, |_| {});
    });

    let err = speclink_server::backup::restore(&target_config(dir.path()), &input)
        .expect_err("a document that has no key stops the restore");
    let shown = err.to_string();
    assert!(shown.contains("demo/backend"), "the message names the scope: {shown}");
    assert!(shown.contains("mystery"), "the message names the document: {shown}");
    nothing_was_written(dir.path());
}

/// A document the import itself would refuse (its digest is not its
/// content's) stops the restore before the identity snapshot is placed.
#[test]
fn a_document_the_import_would_refuse_stops_the_restore_before_any_write() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = changed_fixture(dir.path(), |members| {
        // The second bundle is demo/web: the first scope would import.
        let mut bundle: serde_json::Value =
            serde_json::from_slice(&members["bundles/1.json"]).expect("bundle json");
        bundle["documents"][0]["content"] = "changed after the digest was taken".into();
        members.insert("bundles/1.json".into(), serde_json::to_vec_pretty(&bundle).unwrap());
        reroot(members, |_| {});
    });

    let err = speclink_server::backup::restore(&target_config(dir.path()), &input)
        .expect_err("the restore stops");
    let shown = err.to_string();
    assert!(shown.contains("nothing was written"), "{shown}");
    assert!(shown.contains("demo/web") && shown.contains("digest"), "the scope and the reason: {shown}");
    nothing_was_written(dir.path());
}
