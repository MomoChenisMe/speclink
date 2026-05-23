//! Section 8 integration tests：state machine evaluator hook 透過 `LocalConfigStore`
//! 讀取 review-flag policy。對應 spec scenarios:
//! - default config → proposing→ready (artifact DAG complete)
//! - require_artifact_review=true → proposing→reviewing
//! - require_code_review=true → in_progress→code_reviewing (last task done)
//! - mid-cycle flip：set 完旗標立刻 artifact.write evaluator 看到新 config
//! - malformed config → 走 fallback、warning pass-through

#![allow(clippy::doc_markdown)]

use std::path::Path;
use std::process::Command;

use speclink_provider::{ArtifactKind, ChangeState, ExpectedEtag};
use speclink_runtime::{
    ArtifactOperations, Bootstrap, ChangeOperations, ConfigOperations, RealGitProbe,
    TaskOperations, bootstrap,
};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "command failed: {cmd:?}");
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

fn write_config_with_flags(
    working: &Path,
    require_artifact_review: bool,
    require_code_review: bool,
) {
    let speclink = working.join(".speclink");
    std::fs::create_dir_all(&speclink).expect("create .speclink");
    let body = format!(
        "rules:\n  require_artifact_review: {require_artifact_review}\n  require_code_review: {require_code_review}\n"
    );
    std::fs::write(speclink.join("config.yaml"), body).expect("write config.yaml");
}

async fn setup_change_with_dag(working: &Path, name: &str) {
    let cops = ChangeOperations::new(RealGitProbe);
    cops.create_change(working, name).await.expect("create");

    // Write all three DAG members: proposal.md, tasks.md, spec.md.
    let ops = ArtifactOperations::new(RealGitProbe);
    let _ = ops
        .write_artifact(
            working,
            name,
            ArtifactKind::Proposal,
            None,
            b"## Why\n\nbecause\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write proposal");
    let _ = ops
        .write_artifact(
            working,
            name,
            ArtifactKind::Tasks,
            None,
            b"# Tasks\n\n- [ ] task one\n- [ ] task two\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write tasks");
    let _ = ops
        .write_artifact(
            working,
            name,
            ArtifactKind::Spec,
            Some("foo-cap"),
            b"## ADDED Requirements\n\n### Requirement: x SHALL y\n\n#### Scenario: z\n\n- a\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write spec");
}

async fn fresh_init_with_config(
    require_artifact_review: bool,
    require_code_review: bool,
) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    write_config_with_flags(&working, require_artifact_review, require_code_review);
    (tmp, working)
}

// ----- (a) default config → proposing → ready -----

#[tokio::test]
async fn dag_complete_with_default_config_transitions_proposing_to_ready() {
    let (_tmp, working) = fresh_init_with_config(false, false).await;
    setup_change_with_dag(&working, "demo").await;

    let cops = ChangeOperations::new(RealGitProbe);
    let view = cops
        .show_change(&working, "demo")
        .await
        .expect("show change");
    assert_eq!(
        parse_state(&view.change.state),
        ChangeState::Ready,
        "default config SHALL transition proposing→ready after DAG complete"
    );
}

// ----- (b) require_artifact_review=true → proposing → reviewing -----

#[tokio::test]
async fn dag_complete_with_require_artifact_review_transitions_proposing_to_reviewing() {
    let (_tmp, working) = fresh_init_with_config(true, false).await;
    setup_change_with_dag(&working, "demo").await;

    let cops = ChangeOperations::new(RealGitProbe);
    let view = cops
        .show_change(&working, "demo")
        .await
        .expect("show change");
    assert_eq!(
        parse_state(&view.change.state),
        ChangeState::Reviewing,
        "require_artifact_review=true SHALL hold at reviewing"
    );
}

// ----- (c) require_code_review=true → in_progress → code_reviewing on last task done -----

#[tokio::test]
async fn last_task_done_with_require_code_review_transitions_to_code_reviewing() {
    let (_tmp, working) = fresh_init_with_config(false, true).await;
    setup_change_with_dag(&working, "demo").await;

    // proposing → ready（DAG complete + require_artifact_review=false）
    let cops = ChangeOperations::new(RealGitProbe);
    let view = cops.show_change(&working, "demo").await.expect("show");
    assert_eq!(parse_state(&view.change.state), ChangeState::Ready);

    // apply.start → in_progress.
    let apply = speclink_runtime::ApplyOperations::new(RealGitProbe);
    let _ = apply
        .start(&working, "demo", None)
        .await
        .expect("apply start");

    let tops = TaskOperations::new(RealGitProbe);
    let (_t1, _) = tops.done(&working, "demo", 1).await.expect("done 1");
    let (last, warnings) = tops.done(&working, "demo", 2).await.expect("done 2");

    assert_eq!(
        last.state,
        ChangeState::CodeReviewing,
        "require_code_review=true SHALL transition to code_reviewing after last task done"
    );
    assert!(last.auto_transitioned);
    // No fallback warning (config exists, no external edit).
    assert!(
        warnings.is_empty(),
        "no warnings expected for clean read, got {warnings:?}"
    );
}

// ----- (d) mid-cycle flip → next artifact.write evaluator picks up new config -----

#[tokio::test]
async fn mid_cycle_config_flip_reflects_in_next_dag_evaluation() {
    // 起手 require_artifact_review=false → proposing→ready 後翻 flag、再 artifact.write
    // → evaluator 仍走 walking-skeleton path（state 已是 ready，DAG hook 只對 proposing
    // 觸發）。本測試只驗 mid-cycle 翻旗對「下次」 DAG evaluation 生效 → 換個 change
    // 重跑，確認新 change 走 require_artifact_review=true path。
    let (_tmp, working) = fresh_init_with_config(false, false).await;
    setup_change_with_dag(&working, "first").await;
    let cops = ChangeOperations::new(RealGitProbe);
    assert_eq!(
        parse_state(
            &cops
                .show_change(&working, "first")
                .await
                .unwrap()
                .change
                .state
        ),
        ChangeState::Ready
    );

    // Flip config via ConfigOperations.set_config (verify the runtime path itself works).
    use speclink_provider::{ConfigValue, JsonPath, JsonPathSegment};
    let cfg_ops = ConfigOperations::new(RealGitProbe);
    cfg_ops
        .set_config(
            &working,
            JsonPath::from_segments(vec![
                JsonPathSegment::Field("rules".into()),
                JsonPathSegment::Field("require_artifact_review".into()),
            ]),
            ConfigValue::Bool(true),
            None,
            None,
        )
        .expect("set config");

    // New change SHALL see the flipped flag.
    setup_change_with_dag(&working, "second").await;
    assert_eq!(
        parse_state(
            &cops
                .show_change(&working, "second")
                .await
                .unwrap()
                .change
                .state
        ),
        ChangeState::Reviewing,
        "mid-cycle config flip SHALL be visible to new DAG evaluator firings"
    );
}

// ----- 8.3 malformed warning pass-through -----

#[tokio::test]
async fn state_machine_passes_through_malformed_warning() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    // Malformed YAML in config.yaml.
    let speclink = working.join(".speclink");
    std::fs::create_dir_all(&speclink).expect("create");
    std::fs::write(
        speclink.join("config.yaml"),
        "rules:\n  require_artifact_review: [broken",
    )
    .expect("write malformed");

    // Walking-skeleton change + DAG complete via artifact_ops.
    let cops = ChangeOperations::new(RealGitProbe);
    cops.create_change(&working, "demo").await.expect("create");

    let ops = ArtifactOperations::new(RealGitProbe);
    let _ = ops
        .write_artifact(
            &working,
            "demo",
            ArtifactKind::Proposal,
            None,
            b"## Why\n\nfoo\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write proposal");
    let _ = ops
        .write_artifact(
            &working,
            "demo",
            ArtifactKind::Tasks,
            None,
            b"# Tasks\n\n- [ ] one\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write tasks");
    let (_v, warnings) = ops
        .write_artifact(
            &working,
            "demo",
            ArtifactKind::Spec,
            Some("foo-cap"),
            b"## ADDED Requirements\n\n### Requirement: x SHALL y\n\n#### Scenario: z\n\n- a\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write spec");

    // DAG completes after spec write → evaluator reads malformed config → fallback +
    // warning passed through envelope.
    let codes: Vec<&str> = warnings.iter().map(|w| w.code.as_str()).collect();
    assert!(
        codes.contains(&"config.malformed_using_defaults"),
        "expected config.malformed_using_defaults warning, got {codes:?}"
    );
    assert!(
        codes.contains(&"state_transitioned"),
        "expected state_transitioned warning, got {codes:?}"
    );
}

fn parse_state(s: &str) -> ChangeState {
    s.parse::<ChangeState>().expect("state SHALL parse")
}

// Hush bootstrap unused-import lint when running on no-bootstrap test workflows.
#[allow(dead_code)]
fn _unused_bootstrap() -> bootstrap::Bootstrap<RealGitProbe> {
    bootstrap::Bootstrap::new(RealGitProbe)
}
