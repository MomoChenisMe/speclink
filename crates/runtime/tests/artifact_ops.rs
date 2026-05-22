//! Integration tests for `runtime::artifact_ops::ArtifactOperations`.

use std::path::Path;
use std::process::Command;

use speclink_provider::{ArtifactKind, Etag, ExpectedEtag};
use speclink_runtime::{
    ArtifactOperations, Bootstrap, ChangeOperations, RealGitProbe, RuntimeError,
};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        cmd,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

async fn fresh_project_with_change(name: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    let cops = ChangeOperations::new(RealGitProbe);
    cops.create_change(&working, name).await.expect("create");
    (tmp, working)
}

#[tokio::test]
async fn write_new_then_read_returns_bytes_and_sha256_etag() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let body = b"## Why\n\nfoo\n";
    let w = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Proposal,
            None,
            body,
            ExpectedEtag::None,
        )
        .await
        .expect("write");
    let r = ops
        .read_artifact(&working, "foo", ArtifactKind::Proposal, None)
        .await
        .expect("read");
    assert_eq!(r.value, body);
    assert_eq!(r.etag, w.0.etag);
    assert_eq!(r.etag, Etag::from_bytes(body));
}

#[tokio::test]
async fn overwrite_with_matching_etag_succeeds() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let v0 = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Proposal,
            None,
            b"B0",
            ExpectedEtag::None,
        )
        .await
        .expect("first write");
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Proposal,
        None,
        b"B1",
        ExpectedEtag::Some(v0.0.etag),
    )
    .await
    .expect("matching overwrite");
    let r = ops
        .read_artifact(&working, "foo", ArtifactKind::Proposal, None)
        .await
        .expect("read");
    assert_eq!(r.value, b"B1");
}

#[tokio::test]
async fn overwrite_without_etag_returns_version_conflict() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Proposal,
        None,
        b"B0",
        ExpectedEtag::None,
    )
    .await
    .expect("first write");
    let err = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Proposal,
            None,
            b"B1",
            ExpectedEtag::None,
        )
        .await
        .expect_err("conflict");
    assert!(matches!(err, RuntimeError::ArtifactVersionConflict { .. }));
}

#[tokio::test]
async fn write_new_with_etag_returns_not_found() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let phantom = Etag::from_bytes(b"phantom");
    let err = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Proposal,
            None,
            b"x",
            ExpectedEtag::Some(phantom),
        )
        .await
        .expect_err("not_found");
    assert!(matches!(err, RuntimeError::ArtifactNotFound { .. }));
}

#[tokio::test]
async fn write_spec_without_capability_returns_capability_required() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let err = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Spec,
            None,
            b"x",
            ExpectedEtag::None,
        )
        .await
        .expect_err("spec needs capability");
    assert!(matches!(err, RuntimeError::ArtifactCapabilityRequired));
}

#[tokio::test]
async fn write_with_invalid_capability_returns_kind_invalid() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let err = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Spec,
            Some("User_Auth"),
            b"x",
            ExpectedEtag::None,
        )
        .await
        .expect_err("invalid capability");
    assert!(matches!(err, RuntimeError::ArtifactKindInvalid { .. }));
}

#[tokio::test]
async fn read_artifact_unknown_change_returns_not_found() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let err = ops
        .read_artifact(&working, "unknown", ArtifactKind::Proposal, None)
        .await
        .expect_err("unknown change");
    assert!(matches!(err, RuntimeError::ChangeNotFound { .. }));
}

#[tokio::test]
async fn list_specs_empty_initially() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let caps = ops
        .list_spec_capabilities(&working, "foo")
        .await
        .expect("list");
    assert!(caps.is_empty());
}

#[tokio::test]
async fn list_specs_after_writes_sorted() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Spec,
        Some("user-auth"),
        b"x",
        ExpectedEtag::None,
    )
    .await
    .expect("a");
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Spec,
        Some("rate-limiting"),
        b"x",
        ExpectedEtag::None,
    )
    .await
    .expect("b");
    let caps = ops
        .list_spec_capabilities(&working, "foo")
        .await
        .expect("list");
    assert_eq!(caps, vec!["rate-limiting", "user-auth"]);
}

// --- slice A3 DAG evaluator hook tests --------------------------------------

use speclink_provider::{ChangeState, StateMachineStore};
use speclink_provider_local::LocalStateMachineStore;
use speclink_runtime::resolve_state_root;

async fn current_state(working: &Path, change: &str) -> ChangeState {
    let state_root = resolve_state_root(&RealGitProbe, working).expect("state root");
    let sm = LocalStateMachineStore::new(state_root);
    sm.get_change_state(change).await.expect("read").state
}

#[tokio::test]
async fn hook_noop_when_dag_incomplete() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    let (_v, warnings) = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Proposal,
            None,
            b"## Why\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write proposal only");
    assert!(
        warnings.iter().all(|w| w.code != "state_transitioned"),
        "incomplete DAG SHALL NOT trigger transition"
    );
    assert_eq!(current_state(&working, "foo").await, ChangeState::Proposing);
}

#[tokio::test]
async fn hook_transitions_proposing_to_ready_when_dag_complete() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Proposal,
        None,
        b"## Why\n",
        ExpectedEtag::None,
    )
    .await
    .expect("proposal");
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Spec,
        Some("auth"),
        b"## ADDED Requirements\n",
        ExpectedEtag::None,
    )
    .await
    .expect("spec");
    let (_v, warnings) = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Tasks,
            None,
            b"- [ ] task one\n",
            ExpectedEtag::None,
        )
        .await
        .expect("tasks");
    assert!(
        warnings.iter().any(|w| w.code == "state_transitioned"),
        "DAG complete SHALL trigger state_transitioned warning"
    );
    assert_eq!(current_state(&working, "foo").await, ChangeState::Ready);
}

#[tokio::test]
async fn hook_noop_for_non_proposing_states() {
    let (_tmp, working) = fresh_project_with_change("foo").await;
    let ops = ArtifactOperations::new(RealGitProbe);
    // Get to ready via DAG completion
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Proposal,
        None,
        b"## Why\n",
        ExpectedEtag::None,
    )
    .await
    .expect("proposal");
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Spec,
        Some("auth"),
        b"## ADDED Requirements\n",
        ExpectedEtag::None,
    )
    .await
    .expect("spec");
    ops.write_artifact(
        &working,
        "foo",
        ArtifactKind::Tasks,
        None,
        b"- [ ] t\n",
        ExpectedEtag::None,
    )
    .await
    .expect("tasks → ready");
    assert_eq!(current_state(&working, "foo").await, ChangeState::Ready);

    // Now write a design.md; state is `ready`, hook SHALL no-op.
    let (_v, warnings) = ops
        .write_artifact(
            &working,
            "foo",
            ArtifactKind::Design,
            None,
            b"## Decision\n",
            ExpectedEtag::None,
        )
        .await
        .expect("design write");
    assert!(
        warnings.iter().all(|w| w.code != "state_transitioned"),
        "non-proposing state SHALL skip transition"
    );
    assert_eq!(current_state(&working, "foo").await, ChangeState::Ready);
}
