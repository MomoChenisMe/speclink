//! Backup, verify and restore (server-backup capability). A backup is a single
//! self-describing tar: a manifest with per-member digests, one export bundle per
//! scope (through the TeamStore export contract, never a database-file copy), and
//! a time-point-consistent identity snapshot that holds only hashes.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::EventSettings;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use speclink_store_sqlite::SqliteTeamStore;

use chrono::{Duration, Utc};

/// Read every member of the tar at `path` into name → bytes.
fn read_tar(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let file = std::fs::File::open(path).expect("open backup");
    let mut archive = tar::Archive::new(file);
    let mut members = BTreeMap::new();
    for entry in archive.entries().expect("tar entries") {
        let mut entry = entry.expect("tar entry");
        let name = entry.path().expect("entry path").to_string_lossy().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).expect("read entry");
        members.insert(name, bytes);
    }
    members
}

/// Seed two registry scopes with a couple of documents each, a member user with
/// a PAT, and return the store, identity, the PAT plaintext and the password.
/// The two seed scopes: (project, repo, change).
const SEED_SCOPES: [(&str, &str, &str); 2] =
    [("demo", "backend", "add-auth"), ("acme", "web", "add-billing")];

const SEED_PASSWORD: &str = "seed-password-9f3a";

/// Write two documents into each seed scope of `store`.
fn seed_store_docs(store: &dyn TeamStore) {
    for (project, repo, doc) in SEED_SCOPES {
        let scope = Scope::new(ProjectId::new(project), RepoId::new(repo));
        let ctx = CommandContext { command: "seed".into(), actor: "seed".into() };
        let mut uow = store.begin_unit_of_work(&scope, ctx).expect("uow");
        uow.create(DocumentId::ChangeMeta { change: doc.into() }, "schema: spec-driven\n");
        uow.create(
            DocumentId::ChangeArtifact { change: doc.into(), artifact: "proposal.md".into() },
            "## Why\nseed\n",
        );
        store.commit(uow, Vec::new()).expect("commit");
    }
}

/// Seed the registry (both scopes) plus a member user with a PAT. Returns the
/// PAT plaintext.
fn seed_identity(identity: &IdentitySqlite) -> String {
    for (project, repo, _) in SEED_SCOPES {
        identity.create_project(project, project).expect("project");
        identity.create_repo(project, repo, repo).expect("repo");
    }
    let token = identity
        .create_invitation(NewInvitation {
            email: "member@example.com".into(),
            display: "Member".into(),
            memberships: vec!["demo".into()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, SEED_PASSWORD).expect("accept");
    let (_, pat_plaintext) = identity.create_pat(&user_id, "tok", None).expect("pat");
    pat_plaintext
}

/// The in-memory seed fixture: store, identity, PAT plaintext and password.
fn seed() -> (Arc<MemoryStore>, IdentitySqlite, String, String) {
    let store = Arc::new(MemoryStore::new());
    seed_store_docs(store.as_ref());
    let identity = IdentitySqlite::open_memory().expect("identity");
    let pat_plaintext = seed_identity(&identity);
    (store, identity, pat_plaintext, SEED_PASSWORD.to_string())
}

/// Seed a sqlite store + identity under `dir`, produce a backup and return its
/// path. Source handles are closed before returning.
fn seed_and_backup(dir: &Path) -> std::path::PathBuf {
    let backup = dir.join("backup.tar");
    {
        let store = SqliteTeamStore::open(dir.join("src-store.db")).expect("src store");
        seed_store_docs(&store);
        let identity = IdentitySqlite::open(dir.join("src-identity.db")).expect("src identity");
        seed_identity(&identity);
        speclink_server::backup::create(&store, &identity, &backup).expect("backup");
    }
    backup
}

/// A restore target config over fresh sqlite paths under `dir`.
fn target_config(dir: &Path) -> ServerConfig {
    ServerConfig {
        store: StoreConfig::Sqlite { path: dir.join("tgt-store.db") },
        identity: IdentityConfig::Sqlite { path: dir.join("tgt-identity.db") },
        public_url: "http://127.0.0.1".into(),
        events: EventSettings::default(),
    }
}

#[test]
fn backup_is_self_describing_and_verifiable_per_member() {
    let (store, identity, pat_plaintext, password) = seed();
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("backup.tar");

    let summary =
        speclink_server::backup::create(store.as_ref(), &identity, &output).expect("backup");
    assert_eq!(summary.backup_format_version, speclink_server::backup::BACKUP_FORMAT_VERSION);
    assert_eq!(summary.scope_count, 2, "both registry scopes are backed up");

    let members = read_tar(&output);

    // Manifest present and self-describing.
    let manifest_bytes = members.get("manifest.json").expect("manifest present");
    let manifest: serde_json::Value =
        serde_json::from_slice(manifest_bytes).expect("manifest is json");
    assert_eq!(manifest["backup_format_version"], speclink_server::backup::BACKUP_FORMAT_VERSION);
    assert!(manifest["created_at"].as_str().unwrap().ends_with('Z'), "UTC creation time");
    assert!(!manifest["engine_version"].as_str().unwrap().is_empty(), "engine version recorded");
    assert!(manifest["identity_schema_version"].as_u64().is_some(), "identity schema version");
    let scopes = manifest["scopes"].as_array().expect("scope list");
    assert_eq!(scopes.len(), 2, "manifest lists both scopes");

    // The manifest self-digest side file exists (决策 1).
    assert!(members.contains_key("manifest.json.sha256"), "manifest self-digest side file");

    // One bundle per scope, produced through export — a parseable Bundle JSON,
    // not an opaque database file. Each scope's documents are present.
    let mut bundle_docs = 0usize;
    for scope in scopes {
        let member = scope["member"].as_str().expect("scope member name");
        let bundle_bytes = members.get(member).expect("scope bundle present");
        let bundle: serde_json::Value =
            serde_json::from_slice(bundle_bytes).expect("bundle is json (export, not db copy)");
        bundle_docs += bundle["documents"].as_array().expect("bundle documents").len();
    }
    assert_eq!(bundle_docs, 4, "both scopes' documents are exported");

    // The identity snapshot member is present.
    let identity_member = manifest["identity"]["member"].as_str().expect("identity member name");
    assert!(members.contains_key(identity_member), "identity snapshot present");

    // Every member carries a digest in the manifest (per-member verifiability).
    let listed: Vec<&str> = manifest["members"]
        .as_array()
        .expect("member digest list")
        .iter()
        .map(|m| m["name"].as_str().expect("member name"))
        .collect();
    for scope in scopes {
        assert!(listed.contains(&scope["member"].as_str().unwrap()), "bundle has a digest");
    }
    assert!(listed.contains(&identity_member), "identity snapshot has a digest");

    // No credential plaintext anywhere in the archive: the identity database
    // holds only hashes, so neither the full PAT plaintext nor the password
    // appears in the raw backup bytes.
    let all_bytes: Vec<u8> = members.values().flatten().copied().collect();
    assert!(
        !contains(&all_bytes, pat_plaintext.as_bytes()),
        "the full PAT plaintext must not appear in the backup"
    );
    assert!(
        !contains(&all_bytes, password.as_bytes()),
        "the password must not appear in the backup"
    );
}

/// Whether `haystack` contains `needle` as a contiguous byte subsequence.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Rewrite a tar at `path` from name → bytes members.
fn write_tar(path: &Path, members: &BTreeMap<String, Vec<u8>>) {
    let file = std::fs::File::create(path).expect("create tar");
    let mut builder = tar::Builder::new(file);
    for (name, bytes) in members {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_cksum();
        builder.append_data(&mut header, name, bytes.as_slice()).expect("append");
    }
    builder.finish().expect("finish tar");
}

/// Back up the seed fixture to a fresh file and return its path (holding the
/// tempdir alive).
fn fresh_backup() -> (tempfile::TempDir, std::path::PathBuf) {
    let (store, identity, _, _) = seed();
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("backup.tar");
    speclink_server::backup::create(store.as_ref(), &identity, &output).expect("backup");
    (dir, output)
}

#[test]
fn verify_accepts_a_fresh_backup() {
    let (_dir, output) = fresh_backup();
    let report = speclink_server::backup::verify(&output).expect("a fresh backup verifies");
    assert_eq!(report.backup_format_version, speclink_server::backup::BACKUP_FORMAT_VERSION);
    assert_eq!(report.scope_count, 2);
}

#[test]
fn tamper_of_any_member_is_rejected_and_names_the_member() {
    let (_dir, output) = fresh_backup();

    // Flip one bit of a scope's bundle member.
    let mut members = read_tar(&output);
    let target = members
        .keys()
        .find(|k| k.starts_with("bundles/"))
        .expect("a bundle member")
        .clone();
    members.get_mut(&target).expect("member")[0] ^= 0x01;
    write_tar(&output, &members);

    let err = speclink_server::backup::verify(&output).expect_err("a tampered member is rejected");
    let shown = err.to_string();
    assert!(shown.contains(&target), "the error names the tampered member: {shown}");
}

#[test]
fn tamper_of_the_manifest_is_rejected() {
    let (_dir, output) = fresh_backup();
    let mut members = read_tar(&output);
    // Corrupt the manifest body; its self-digest side file no longer matches.
    members.get_mut("manifest.json").expect("manifest")[10] ^= 0x01;
    write_tar(&output, &members);

    let err = speclink_server::backup::verify(&output).expect_err("a tampered manifest is rejected");
    assert!(err.to_string().contains("manifest"), "the error points at the manifest: {err}");
}

#[test]
fn an_unknown_format_version_is_rejected() {
    let (_dir, output) = fresh_backup();
    let mut members = read_tar(&output);

    // A legitimate future backup: bump the format version and re-root the
    // manifest self-digest so the tamper chain is intact but the version unknown.
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&members["manifest.json"]).expect("manifest json");
    manifest["backup_format_version"] = serde_json::json!(9999);
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("reserialize");
    let digest = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&manifest_bytes);
        format!("sha256:{:x}", h.finalize())
    };
    members.insert("manifest.json".into(), manifest_bytes);
    members.insert("manifest.json.sha256".into(), digest.into_bytes());
    write_tar(&output, &members);

    let err = speclink_server::backup::verify(&output).expect_err("an unknown version is rejected");
    assert!(err.to_string().contains("version"), "the reason names the version: {err}");
}

#[test]
fn restore_into_empty_target_validates_green() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backup = seed_and_backup(dir.path());
    let config = target_config(dir.path());

    let report = speclink_server::backup::restore(&config, &backup).expect("restore succeeds");
    assert!(report.ok, "a fresh restore validates green: {:?}", report.differences);
    assert_eq!(report.scopes_checked, 2, "both scopes are validated");

    // The restored target carries the backed-up data: both scopes' documents and
    // the member user.
    let store = SqliteTeamStore::open(dir.path().join("tgt-store.db")).expect("target store");
    for (project, repo, _) in SEED_SCOPES {
        let scope = Scope::new(ProjectId::new(project), RepoId::new(repo));
        assert_eq!(store.export(&scope).expect("export").documents.len(), 2, "scope restored");
    }
    let identity = IdentitySqlite::open(dir.path().join("tgt-identity.db")).expect("target identity");
    assert_eq!(identity.list_users().expect("users").len(), 1, "member user restored");
}

#[test]
fn restore_refuses_a_non_empty_target_and_leaves_it_unchanged() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backup = seed_and_backup(dir.path());
    let config = target_config(dir.path());

    // Pre-populate the target identity with a user — a non-empty target.
    {
        let identity =
            IdentitySqlite::open(dir.path().join("tgt-identity.db")).expect("target identity");
        seed_identity(&identity);
    }
    let identity_before = std::fs::read(dir.path().join("tgt-identity.db")).expect("read before");

    let err = speclink_server::backup::restore(&config, &backup)
        .expect_err("a non-empty target is refused");
    assert!(err.to_string().to_lowercase().contains("empty"), "the reason cites emptiness: {err}");

    let identity_after = std::fs::read(dir.path().join("tgt-identity.db")).expect("read after");
    assert_eq!(identity_before, identity_after, "the refused target is byte-for-byte unchanged");
}

#[test]
fn restore_validation_reports_a_post_restore_mismatch() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backup = seed_and_backup(dir.path());
    let config = target_config(dir.path());

    let report = speclink_server::backup::restore(&config, &backup).expect("restore");
    assert!(report.ok, "fresh restore is green");

    // Tamper with the restored target: change one document's content, so the
    // scope no longer matches the backup's recorded digest and count.
    {
        let store = SqliteTeamStore::open(dir.path().join("tgt-store.db")).expect("target store");
        let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
        let snapshot = store.snapshot(&scope).expect("snapshot");
        let doc = DocumentId::ChangeMeta { change: "add-auth".into() };
        let rev = snapshot.read(&doc).expect("read").expect("present").revision;
        let ctx = CommandContext { command: "tamper".into(), actor: "tamper".into() };
        let mut uow = store.begin_unit_of_work(&scope, ctx).expect("uow");
        uow.update(doc, "schema: spec-driven\n# drift\n", rev);
        store.commit(uow, Vec::new()).expect("commit");
    }

    let report = speclink_server::backup::validate(&config, &backup).expect("validate runs");
    assert!(!report.ok, "validation catches the post-restore drift");
    assert!(!report.differences.is_empty(), "the report lists the differing item");
    assert!(
        report.differences.iter().any(|d| d.contains("demo")),
        "the difference names the drifted scope: {:?}",
        report.differences
    );
}
