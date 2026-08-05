//! 動詞操作：validate / analyze / archive，經內嵌 core 執行。
//!
//! 可觀察結果（成功資料、失敗訊息與語意）對應 CLI 對應指令；失敗回傳 `Err` 附訊息，
//! 不靜默吞掉。park/unpark 不在此——該功能已從 speclink 移除（見 core inprogress.rs）。

use std::path::Path;

use serde_json::Value;
use speclink_core::store::Store;

use crate::init_core_context;

/// 對應 `speclink validate <change>`：回傳該 change 的 `ValidationResult`（valid/errors/warnings）。
pub fn validate_at(root: &Path, change: &str) -> Result<Value, String> {
    let ctx = crate::require_context_for_change(root, change)?;
    let store: &dyn Store = &ctx.store;
    let change = find(store, change)?;
    // 與 CLI 一致：validate 不解析 change 的 schema，一律用 spec_driven。
    let schema = speclink_core::schema::spec_driven();
    let result = speclink_core::validate::validate_change(store, &change, &schema, false);
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 對應 `speclink analyze <change> --json`：回傳 `AnalyzeReport`（含 findings 與各維度狀態）。
pub fn analyze_at(root: &Path, change: &str) -> Result<Value, String> {
    let ctx = crate::require_context_for_change(root, change)?;
    let store: &dyn Store = &ctx.store;
    let change = find(store, change)?;
    let schema = speclink_core::schema::spec_driven();
    let report = speclink_core::analyzer::analyze(store, &change, &schema);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

/// 對應 `speclink archive <change>`：以預設選項歸檔（先驗證）。前置未滿足時回傳 `Err`
/// 且不標記歸檔（core::archive 在失敗時不搬移 change）。
pub fn archive_at(root: &Path, change: &str) -> Result<Value, String> {
    archive_with(root, change, false)
}

/// 對應 `speclink archive <change> --carry-review`：帶未結工單封存（封存入口
/// 三選項的「照樣帶走」；spec「封存入口的未結工單三選項」）。
pub fn archive_carry_at(root: &Path, change: &str) -> Result<Value, String> {
    archive_with(root, change, true)
}

/// 對應 `speclink review discard <change>`：刪工單、不蓋章（三選項的「放棄審查」）。
pub fn discard_review_at(root: &Path, change: &str) -> Result<Value, String> {
    let ctx = crate::require_context_for_change(root, change)?;
    let store: &dyn Store = &ctx.store;
    speclink_core::review::discard(store, change).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "change": change, "discarded": true }))
}

fn archive_with(root: &Path, change: &str, carry_review: bool) -> Result<Value, String> {
    let ctx = open(root)?;
    crate::query::refuse_if_worktree_is_open(&ctx, change)?;
    let store: &dyn Store = &ctx.store;
    let change = find(store, change)?;
    let opts = speclink_core::archive::ArchiveOptions { carry_review, ..Default::default() };
    let actor = crate::manage::cached_git_identity(&ctx.workspace.root);
    let outcome = speclink_core::archive::archive(&ctx.workspace, store, &change, &opts, actor.as_deref())
        .map_err(|e| e.to_string())?;
    // ArchiveOutcome 未實作 Serialize；GUI 需要的結果欄位以 camelCase 手動組出。
    Ok(serde_json::json!({
        "changeName": outcome.change_name,
        "datedName": outcome.dated_name,
        "snapshotCreated": outcome.snapshot_created,
        "skippedSpecs": outcome.skipped_specs,
    }))
}

pub(crate) fn open(root: &Path) -> Result<crate::ProjectContext, String> {
    init_core_context(root).ok_or_else(|| format!("not a speclink project: {}", root.display()))
}

fn find(store: &dyn Store, change: &str) -> Result<speclink_core::model::Change, String> {
    store
        .find_change(change)
        .ok_or_else(|| format!("change not found: {change}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;

    fn valid_fixture(tag: &str) -> FixtureRoot {
        let fx = FixtureRoot::new(tag);
        fx.add_change("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write(
            "openspec/changes/demo/specs/cap-x/spec.md",
            "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: works\n\n- **WHEN** used\n- **THEN** it works\n",
        );
        fx
    }

    #[test]
    fn validate_reports_valid_for_a_valid_change() {
        let fx = valid_fixture("v-valid");
        let v = validate_at(fx.root(), "demo").expect("validate ok");
        assert_eq!(v["valid"], true);
        assert!(v.get("errors").is_some() && v.get("warnings").is_some());
    }

    #[test]
    fn validate_unknown_change_errors() {
        let fx = valid_fixture("v-unknown");
        assert!(validate_at(fx.root(), "no-such-change-xyz").is_err());
    }

    #[test]
    fn analyze_returns_report_with_findings_array() {
        let fx = valid_fixture("v-analyze");
        let v = analyze_at(fx.root(), "demo").expect("analyze ok");
        assert!(v["findings"].is_array(), "report exposes findings array");
    }

    #[test]
    fn archive_is_refused_while_the_change_has_a_worktree() {
        // spec worktree-overlay「worktree 掛著時的 desktop 動詞防護」Scenario
        // 「封存被擋」：動詞拒絕、訊息指出收尾方式、change 仍在原處。
        let fx = valid_fixture("v-archive-wt");
        fx.attach_worktree("demo");

        let err = archive_at(fx.root(), "demo").expect_err("有 worktree 映射時必須拒絕");
        assert!(err.contains("worktree-merge"), "須指出收尾方式: {err}");
        assert!(err.contains("demo"), "須指名 change: {err}");

        let ctx = crate::init_core_context(fx.root()).unwrap();
        assert!(
            speclink_core::store::Store::change_exists(&ctx.store, "demo"),
            "拒絕時 change 不得被搬走"
        );
        // 「照樣帶走」入口走同一守門。
        assert!(archive_carry_at(fx.root(), "demo").is_err());
    }

    #[test]
    fn the_guard_lifts_once_the_worktree_is_removed() {
        // Scenario「收尾後解禁」：worktree 移除後守門退場，動詞照常走既有前置。
        let fx = valid_fixture("v-archive-wt-gone");
        fx.attach_worktree("demo");
        assert!(archive_at(fx.root(), "demo").is_err());

        let out = std::process::Command::new("git")
            .args(["worktree", "remove", "--force", "wt"])
            .current_dir(fx.root())
            .output()
            .expect("run git");
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

        let err = archive_at(fx.root(), "demo").expect_err("仍受既有前置擋（任務未完成）");
        assert!(!err.contains("worktree-merge"), "worktree 守門須已退場: {err}");
    }

    #[test]
    fn validate_and_analyze_read_the_worktree_copy() {
        // spec worktree-overlay Scenario「分析報告反映 worktree 現值」：驗證與分析
        // 吃的是 worktree 副本的 artifact——主 checkout 的舊內容不該左右結果。
        let fx = valid_fixture("v-analyze-wt");
        // 主 checkout 的 delta spec 退化成無場景版（validate 判不合格、零 Ambiguity）。
        fx.write(
            "openspec/changes/demo/specs/cap-x/spec.md",
            "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n",
        );
        let wt = fx.attach_worktree("demo");
        std::fs::write(
            wt.change_dir("demo")
                .join("specs")
                .join("cap-x")
                .join("spec.md"),
            "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: works\n\n- **WHEN** used\n- **THEN** it works\n\n#### Scenario: also works\n\n- **WHEN** used again\n- **THEN** it still works\n",
        )
        .unwrap();

        let v = validate_at(fx.root(), "demo").expect("validate ok");
        assert_eq!(v["valid"], true, "驗證須讀副本的合格版本: {v}");

        let report = analyze_at(fx.root(), "demo").expect("analyze ok");
        let ambiguity = report["findings"]
            .as_array()
            .expect("findings array")
            .iter()
            .filter(|f| f["dimension"] == "Ambiguity")
            .count();
        assert_eq!(ambiguity, 2, "分析報告須依副本的兩個場景產出: {report}");
    }

    #[test]
    fn read_only_verbs_are_unaffected_by_a_worktree() {
        // 唯讀面不擋：抽屜與檢視在 worktree 掛著時照常。
        let fx = valid_fixture("v-readonly-wt");
        fx.attach_worktree("demo");
        assert!(validate_at(fx.root(), "demo").is_ok());
        assert!(analyze_at(fx.root(), "demo").is_ok());
    }

    #[test]
    fn archive_unmet_prerequisite_errors_and_does_not_archive() {
        // 不存在的 change 無法歸檔——安全地驗證失敗語意，不觸碰 fixture change 檔案。
        let fx = valid_fixture("v-archive");
        assert!(archive_at(fx.root(), "no-such-change-xyz").is_err());
        let ctx = crate::init_core_context(fx.root()).unwrap();
        assert!(speclink_core::store::Store::change_exists(&ctx.store, "demo"));
    }
}
