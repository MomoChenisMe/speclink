//! `new change` and `new artifact`.

use crate::capname::{self, Source};
use crate::model::{self, Change};
use crate::schema::Schema;
use crate::store::Store;
use crate::util;
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Create a new change with its metadata document. `actor` is the
/// Host-resolved display identity — None (anonymous) stamps no created_by.
pub fn new_change(
    store: &dyn Store,
    name: &str,
    _description: Option<&str>,
    schema: &str,
    agent: Option<&str>,
    from_discussion: Option<&str>,
    actor: Option<&str>,
) -> Result<PathBuf> {
    if !is_kebab_case(name) {
        bail!("Invalid change name '{name}'. Must be kebab-case (e.g., 'add-feature').");
    }
    if store.change_exists(name) {
        bail!("Change '{name}' already exists.");
    }
    let created = util::today();
    let mut meta = format!("schema: {schema}\ncreated: {created}\n");
    if let Some(id) = actor {
        meta.push_str(&format!("created_by: {id}\n"));
    }
    if let Some(agent) = agent {
        meta.push_str(&format!("created_with: {agent}\n"));
    }
    if let Some(slug) = from_discussion {
        meta.push_str(&format!("from_discussion: {slug}\n"));
    }
    store.create_change(name, &meta)
}

/// Resolve the artifact type token to (artifact_id, relative_output_path).
fn resolve_output(kind: &str, capability: Option<&str>) -> Result<(String, String)> {
    match kind {
        "proposal" => Ok(("proposal".into(), "proposal.md".into())),
        "design" => Ok(("design".into(), "design.md".into())),
        "tasks" => Ok(("tasks".into(), "tasks.md".into())),
        "spec" => {
            let cap = capability.ok_or_else(|| {
                anyhow::anyhow!(
                    "Capability name is required for spec type. Usage: speclink new artifact spec <capability> --change <name>"
                )
            })?;
            Ok(("specs".into(), format!("specs/{cap}/spec.md")))
        }
        other => bail!("Unknown artifact type '{other}'. Valid types: proposal, design, tasks, spec"),
    }
}

/// Create (write) an artifact for a change. `new_capability` is the `--new`
/// confirmation: a spec whose capability the canonical specs do not carry is
/// refused unless it is set (capability-naming-guard 主閘).
pub fn new_artifact(
    store: &dyn Store,
    change: &Change,
    schema: &Schema,
    kind: &str,
    capability: Option<&str>,
    content: Option<&str>,
    force: bool,
    new_capability: bool,
) -> Result<(String, PathBuf)> {
    // Fail-closed gate: corrupt metadata must not be read as the default
    // schema and produce an artifact from its templates.
    crate::model::require_valid_meta(change)?;
    let (artifact_id, rel) = resolve_output(kind, capability)?;
    // 命名主閘（capability-naming-guard design D1/D2）：路徑解析後、任何寫入
    // 前，正典未收錄的 spec capability 未經 --new 顯性宣告即拒絕。閘門條件是
    // 二元事實（正典有無收錄）；近似名只進建議、不進判斷。兩側判定都走清單
    // 逐字比對、不走檔案系統存在性——大小寫不敏感的 fs 會讓 `Auth` 冒充
    // `auth` 靜默放行。本 change 已有同名 delta（曾以 --new 宣告過）時重寫
    // 放行，交給下游既有的覆寫保護。
    if artifact_id == "specs" && !new_capability {
        let cap = capability.expect("resolve_output guarantees a capability for specs");
        let in_canon = store.list_canonical_capabilities().iter().any(|c| c == cap);
        let declared = store.delta_capabilities(&change.name).iter().any(|c| c == cap);
        if !in_canon && !declared {
            return Err(crate::command::Refusal(naming_gate_message(store, cap)).into());
        }
    }
    if store.artifact_exists(&change.name, &rel) && !force {
        // The display path is joined component-by-component so the native
        // separator is used throughout (matches the created file's path).
        let out_path = rel.split('/').fold(change.dir.clone(), |p, c| p.join(c));
        bail!("Artifact already exists: {}. Use --force to overwrite", out_path.to_string_lossy());
    }

    let body = match content {
        Some(c) => c.to_string(),
        // Template from the schema; a missing template file (or an artifact the schema doesn't
        // define) yields an empty file (frozen behavior).
        None => schema
            .artifact(&artifact_id)
            .and_then(|a| a.template.clone())
            .unwrap_or_default(),
    };

    // Validate supplied content structurally (only when content is provided).
    if content.is_some() {
        validate_artifact_content(&artifact_id, &rel, &body)?;
    }

    // Engine-produced tasks carry stable IDs on every task line (spec task-identity).
    let body = if artifact_id == "tasks" { crate::tasks::stamp_all(&body) } else { body };

    let out_path = store.write_artifact(&change.name, &rel, &body)?;
    Ok((artifact_id, out_path))
}

fn validate_artifact_content(artifact_id: &str, rel: &str, body: &str) -> Result<()> {
    match artifact_id {
        "proposal" => {
            let ok = ["## Why", "## Problem", "## Summary"]
                .iter()
                .any(|h| body.lines().any(|l| l.trim_end() == *h || l.trim_start().starts_with(&format!("{h} "))));
            if !ok {
                bail!("Proposal must contain a ## Why, ## Problem, or ## Summary section");
            }
        }
        "design" => {
            if !body.contains("## Context") {
                bail!("Design must contain a ## Context section");
            }
        }
        "tasks" => {
            // At least one INCOMPLETE checkbox is required.
            let ok = body
                .lines()
                .any(|l| l.trim_start().starts_with("- [ ] "));
            if !ok {
                bail!("Tasks must contain at least one checkbox (- [ ])");
            }
        }
        "specs" => {
            let _ = rel;
            if !model::has_delta_operation(body) {
                bail!("Delta spec parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)");
            }
        }
        _ => {}
    }
    Ok(())
}

/// 主閘拒絕訊息：近似建議（至多三筆，各附來源標注與 Purpose 首行）＋兩條
/// 指引。無近似時略去清單，指引恆在。與候選完全同名的 in-flight delta 不算
/// 「相似名」——「沿用其確切名稱」的指引對它是死路（名字已經一樣），改以
/// 獨立句點名該 change 並指路 --new。
fn naming_gate_message(store: &dyn Store, cap: &str) -> String {
    let suggestions = capname::suggest(cap, &capname::suggestion_pool(store));
    let mut msg = format!("Capability '{cap}' is not in the canonical specs.\n");
    let (same_name, similar): (Vec<_>, Vec<_>) =
        suggestions.into_iter().partition(|k| k.name == cap);
    if let Some(k) = same_name.first() {
        if let Source::InFlight(other) = &k.source {
            msg.push_str(&format!(
                "It is already opened by in-flight change '{other}' — the same capability \
keeps this exact name; re-run with --new to add this change's delta for it.\n"
            ));
        }
    }
    if !similar.is_empty() {
        msg.push_str(&capname::suggestion_block(&similar));
    }
    msg.push_str(
        "To modify an existing capability, reuse its exact name.\n\
If this really is a new capability, re-run with --new.",
    );
    msg
}

/// Whether a change name is valid kebab-case (lowercase alphanumerics with single hyphens).
fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Convenience: find or error for an active change by name.
pub fn require_change(store: &dyn Store, name: &str) -> Result<Change> {
    model::find_change(store, name).ok_or_else(|| anyhow::anyhow!("Change '{name}' not found."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    // --- ExecutionContext 由 Host 解析且不可覆寫：new change 收明確 actor ---

    #[test]
    fn new_change_stamps_the_injected_actor_only() {
        let store = TestStore::default();
        new_change(&store, "with-actor", None, "spec-driven", None, None, Some("Alice <a@example.com>"))
            .expect("new change succeeds");
        assert!(
            store.meta("with-actor").contains("created_by: Alice <a@example.com>\n"),
            "created_by is the injected actor, meta: {}",
            store.meta("with-actor")
        );
    }

    #[test]
    fn new_change_without_actor_stamps_no_created_by() {
        // 無身分：沿用現行無章行為（同今日無 git／未設 user.name）。
        let store = TestStore::default();
        new_change(&store, "anon", None, "spec-driven", None, None, None)
            .expect("new change succeeds");
        assert!(
            !store.meta("anon").contains("created_by:"),
            "anonymous stays unstamped, meta: {}",
            store.meta("anon")
        );
    }

    // --- capability 命名主閘（capability-naming-guard「建立點主閘」）---

    /// 一份合法的 delta 內容（含操作區塊與合格 Purpose——新 capability 的
    /// Purpose 早檢查在 validate，這裡只需通過格式驗證）。
    const DELTA: &str = "## Purpose\n\n測試用。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n";
    const CANON_AUTH: &str = "# auth Specification\n\n## Purpose\n\nAuth session lifecycle.\n第二行不入建議。\n\n## Requirements\n";

    fn store_with_change(name: &str) -> TestStore {
        TestStore::with_meta(name, "schema: spec-driven\ncreated: 2026-08-20\n")
    }

    fn spec_artifact(
        store: &TestStore,
        change_name: &str,
        cap: &str,
        new_capability: bool,
    ) -> Result<(String, PathBuf)> {
        let change = model::find_change(store, change_name).expect("change resolves");
        new_artifact(
            store,
            &change,
            &crate::schema::spec_driven(),
            "spec",
            Some(cap),
            Some(DELTA),
            false,
            new_capability,
        )
    }

    #[test]
    fn gate_refuses_an_unlisted_capability_and_writes_nothing() {
        // spec Scenario「未收錄名稱未帶 --new 遭拒且不落盤」。
        let store = store_with_change("demo");
        store.canonical.borrow_mut().insert("auth".into(), CANON_AUTH.into());
        let err = spec_artifact(&store, "demo", "authentication", false)
            .expect_err("unlisted capability must refuse");
        assert!(
            err.downcast_ref::<crate::command::Refusal>().is_some(),
            "refusal 標記使 runtime 分類為 refused: {err}"
        );
        let msg = err.to_string();
        assert!(msg.contains("auth"), "訊息含近似建議: {msg}");
        assert!(msg.contains("Auth session lifecycle."), "建議附 Purpose 首行: {msg}");
        assert!(!msg.contains("第二行"), "只取 Purpose 首行: {msg}");
        assert!(msg.contains("--new"), "指引指路 --new 重跑: {msg}");
        assert!(msg.contains("exact name"), "指引指路沿用既有名: {msg}");
        assert_eq!(*store.artifact_writes.borrow(), 0, "拒絕不落盤");
        assert!(!store.artifact_exists("demo", "specs/authentication/spec.md"));
    }

    #[test]
    fn gate_passes_a_canonical_capability_unchanged() {
        // spec Scenario「命中正典名稱照常放行」：行為與輸出維持現狀。
        let store = store_with_change("demo");
        store.canonical.borrow_mut().insert("auth".into(), CANON_AUTH.into());
        let (artifact_id, path) =
            spec_artifact(&store, "demo", "auth", false).expect("canonical name passes");
        assert_eq!(artifact_id, "specs");
        assert_eq!(path, PathBuf::from("changes/demo/specs/auth/spec.md"));
        assert_eq!(store.read_artifact("demo", "specs/auth/spec.md").as_deref(), Some(DELTA));
    }

    #[test]
    fn gate_passes_with_the_new_confirmation() {
        // spec Scenario「帶 --new 建立新 capability 成功」。
        let store = store_with_change("demo");
        spec_artifact(&store, "demo", "token-rotation", true)
            .expect("--new declares the new capability");
        assert!(store.artifact_exists("demo", "specs/token-rotation/spec.md"));
    }

    #[test]
    fn the_confirmation_does_not_waive_delta_format_validation() {
        // spec Scenario「--new 不豁免 delta 格式驗證」：無操作區塊照拒。
        let store = store_with_change("demo");
        let change = model::find_change(&store, "demo").expect("change resolves");
        let err = new_artifact(
            &store,
            &change,
            &crate::schema::spec_driven(),
            "spec",
            Some("token-rotation"),
            Some("## Purpose\n\n沒有操作區塊。\n"),
            false,
            true,
        )
        .expect_err("format validation still applies");
        assert!(err.to_string().contains("at least one operation"), "既有格式錯誤不變: {err}");
        assert_eq!(*store.artifact_writes.borrow(), 0, "拒絕不落盤");
    }

    #[test]
    fn in_flight_deltas_of_other_changes_appear_in_the_suggestions() {
        // spec Scenario「進行中 change 的 delta 出現在名單」。
        let store = store_with_change("demo");
        store.metas.borrow_mut().insert(
            "add-sso".into(),
            "schema: spec-driven\ncreated: 2026-08-20\n".into(),
        );
        store.put_artifact(
            "add-sso",
            "specs/user-auth/spec.md",
            "## Purpose\n\n使用者驗證。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n",
        );
        let err = spec_artifact(&store, "demo", "user-authentication", false)
            .expect_err("unlisted capability must refuse");
        let msg = err.to_string();
        assert!(msg.contains("user-auth"), "in-flight delta 進名單: {msg}");
        assert!(msg.contains("add-sso"), "標注來源 change: {msg}");
        assert!(msg.contains("使用者驗證。"), "附 delta 的 Purpose 首行: {msg}");
    }

    #[test]
    fn a_declared_delta_rewrites_without_repeating_the_new_flag() {
        // --new 宣告過一次（本 change 已有同名 delta）後，重寫不再過閘：
        // 未帶 --force 得回既有的 already-exists 錯誤（恢復其可達性），
        // 帶 --force 照現行流程覆寫。
        let store = store_with_change("demo");
        store.put_artifact("demo", "specs/token-rotation/spec.md", DELTA);
        let err = spec_artifact(&store, "demo", "token-rotation", false)
            .expect_err("existing artifact without --force still errors");
        assert!(
            err.to_string().contains("already exists"),
            "回到既有覆寫保護而非主閘: {err}"
        );
        let change = model::find_change(&store, "demo").expect("change resolves");
        new_artifact(
            &store,
            &change,
            &crate::schema::spec_driven(),
            "spec",
            Some("token-rotation"),
            Some(DELTA),
            true,
            false,
        )
        .expect("--force rewrite passes without --new");
    }

    #[test]
    fn a_case_variant_of_a_canonical_name_is_refused_with_the_suggestion() {
        // 收錄與否是正典清單的逐字比對——`Auth` 不等於 `auth`，即使大小寫
        // 不敏感的檔案系統說同一個檔案存在；建議端折疊大小寫，auth 要出現。
        let store = store_with_change("demo");
        store.canonical.borrow_mut().insert("auth".into(), CANON_AUTH.into());
        let err = spec_artifact(&store, "demo", "Auth", false)
            .expect_err("case variant is not the canonical name");
        let msg = err.to_string();
        assert!(msg.contains("'Auth' is not in the canonical specs"), "逐字比對拒絕: {msg}");
        assert!(msg.contains("auth (canonical)"), "折疊後建議正典名: {msg}");
    }

    #[test]
    fn a_same_named_in_flight_delta_gets_the_dedicated_guidance() {
        // 與候選完全同名的 in-flight delta 不是「相似名」——指引沿用確切
        // 名稱對它是死路，訊息改點名該 change 並指路 --new。
        let store = store_with_change("demo");
        store.metas.borrow_mut().insert(
            "add-sso".into(),
            "schema: spec-driven\ncreated: 2026-08-20\n".into(),
        );
        store.put_artifact("add-sso", "specs/user-auth/spec.md", DELTA);
        let err = spec_artifact(&store, "demo", "user-auth", false)
            .expect_err("cross-change same-name still refuses");
        let msg = err.to_string();
        assert!(
            msg.contains("already opened by in-flight change 'add-sso'"),
            "點名開立它的 change: {msg}"
        );
        assert!(msg.contains("--new"), "指路 --new: {msg}");
        assert!(!msg.contains("Similar existing names"), "同名不列進相似清單: {msg}");
    }

    #[test]
    fn a_gate_refusal_without_near_names_keeps_the_guidance() {
        // spec Scenario「無近似名仍拒絕」：只少建議清單，兩條指引仍在。
        let store = store_with_change("demo");
        store.canonical.borrow_mut().insert("auth".into(), CANON_AUTH.into());
        let err = spec_artifact(&store, "demo", "zzz-unrelated", false)
            .expect_err("no similarity still refuses");
        let msg = err.to_string();
        assert!(!msg.contains("auth"), "無近似不列建議: {msg}");
        assert!(msg.contains("--new") && msg.contains("exact name"), "兩條指引仍在: {msg}");
    }
}
