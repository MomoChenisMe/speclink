//! 溯源鏈組裝：`speclink trace <capability>` 的領域層。
//!
//! 只讀既存事實——封存目錄列舉（D1）、`.openspec.yaml` 的 from_discussion、
//! 討論 frontmatter 的 promoted_to、`.evidence.json` 的逐 task 檔案清單、
//! 正典規格的 @trace 歸屬。進行中 change 不入鏈（D2）；evidence 是逐 change
//! 的存在性偵測，缺檔為 None、絕不回讀舊 @trace 的 code 清單（D3）；單環
//! 髒資料不使整鏈失敗。不含 ANSI、不假設儲存媒介（D4）。

use crate::model::ChangeMeta;
use crate::store::Store;
use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TraceReport {
    pub capability: String,
    pub requirements: Vec<RequirementTrace>,
    pub changes: Vec<ChangeTrace>,
    pub discussions: Vec<DiscussionTrace>,
}

/// 正典規格單條 Requirement 的現行 @trace 歸屬。
#[derive(Debug, Clone, Serialize)]
pub struct RequirementTrace {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeTrace {
    pub name: String,
    #[serde(rename = "archivedDir")]
    pub archived_dir: String,
    /// `.openspec.yaml` 的 from_discussion 原文（逗號累加器照錄）；缺欄為 null。
    #[serde(rename = "fromDiscussion")]
    pub from_discussion: Option<String>,
    /// `.evidence.json` 的逐 task 檔案清單；檔案不存在（或不可解析）為 null。
    pub evidence: Option<Vec<EvidenceTask>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceTask {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscussionTrace {
    pub slug: String,
    pub archived: bool,
    #[serde(rename = "promotedTo")]
    pub promoted_to: Vec<PromotedChange>,
}

/// 討論扇出的兄弟變更與其觸及的 capability 名集合。
#[derive(Debug, Clone, Serialize)]
pub struct PromotedChange {
    pub change: String,
    pub capabilities: Vec<String>,
}

/// 組裝 capability 的封存演進鏈。capability 無正典規格時回錯誤（近似建議
/// 見 `not_found_message`）；鏈內單環缺漏一律寬容輸出。
pub fn run(store: &dyn Store, capability: &str) -> Result<TraceReport> {
    let Some(spec_text) = store.read_canonical_spec(capability) else {
        bail!(not_found_message(store, capability));
    };
    let requirements = requirement_sources(&spec_text);

    // D1：封存目錄含該 capability 的 delta 子目錄即入鏈；`YYYY-MM-DD-` 前綴
    // 的字典序即時序，遞增排序＝由舊至新。
    let mut dated: Vec<String> = store
        .list_archived_changes()
        .into_iter()
        .filter(|d| store.archived_delta_capabilities(d).iter().any(|c| c == capability))
        .collect();
    dated.sort();

    let mut changes = Vec::new();
    let mut slugs: Vec<String> = Vec::new();
    for dated_name in &dated {
        let meta_text = store.read_archived_meta(dated_name);
        let meta = ChangeMeta::from_text(meta_text.as_deref()).unwrap_or_default();
        for slug in meta.from_discussions() {
            if !slugs.contains(&slug) {
                slugs.push(slug);
            }
        }
        let evidence = store
            .read_archived_artifact(dated_name, ".evidence.json")
            .and_then(|text| parse_evidence(&text));
        changes.push(ChangeTrace {
            name: strip_date_prefix(dated_name).to_string(),
            archived_dir: dated_name.clone(),
            from_discussion: meta.from_discussion,
            evidence,
        });
    }

    let mut discussions = Vec::new();
    for slug in slugs {
        let Some(doc) = store.read_discussion(&slug) else {
            continue;
        };
        let promoted_to = crate::discuss::promoted_to(store, &slug)
            .into_iter()
            .map(|change| {
                let capabilities = sibling_capabilities(store, &change);
                PromotedChange { change, capabilities }
            })
            .collect();
        discussions.push(DiscussionTrace { slug, archived: doc.archived, promoted_to });
    }

    Ok(TraceReport {
        capability: capability.to_string(),
        requirements,
        changes,
        discussions,
    })
}

/// D6：不存在的 capability 依 naming guard 慣例給至多三筆近似名。
fn not_found_message(store: &dyn Store, cap: &str) -> String {
    let suggestions = crate::capname::suggest(cap, &crate::capname::suggestion_pool(store));
    let mut msg = format!("Capability '{cap}' is not in the canonical specs.\n");
    if suggestions.is_empty() {
        msg.push_str("No similar capability names found.");
    } else {
        msg.push_str("Similar existing names:\n");
        for s in &suggestions {
            msg.push_str(&format!("  - {}\n", crate::capname::suggestion_line(s)));
        }
        msg.push_str("Re-run with one of these exact names.");
    }
    msg
}

/// 兄弟變更觸及的 capability：優先取其最新封存目錄的 delta 集合，未封存
/// 時退回進行中 delta；兩者皆無為空集合（寬容）。
fn sibling_capabilities(store: &dyn Store, change: &str) -> Vec<String> {
    if let Some(dated) = store
        .list_archived_changes()
        .into_iter()
        .find(|d| strip_date_prefix(d) == change)
    {
        return store.archived_delta_capabilities(&dated);
    }
    store.delta_capabilities(change)
}

/// 正典規格每條 Requirement 的 @trace source；無 @trace 的 Requirement 無
/// 歸屬可列，略過。@trace 內 source 以外的鍵（含歷史 code 清單）一律不讀。
fn requirement_sources(text: &str) -> Vec<RequirementTrace> {
    let (_, blocks) = crate::archive::parse_canonical(text);
    blocks
        .into_iter()
        .filter_map(|(name, block)| trace_source(&block).map(|source| RequirementTrace { name, source }))
        .collect()
}

fn trace_source(block: &str) -> Option<String> {
    let mut in_trace = false;
    for line in block.lines() {
        let t = line.trim();
        if t.starts_with("<!-- @trace") {
            in_trace = true;
            continue;
        }
        if in_trace {
            if t.ends_with("-->") {
                in_trace = false;
                continue;
            }
            if let Some(rest) = t.strip_prefix("source:") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

/// v2 的 entries 優先，退回 v1 的 touched；不可解析視同無記錄（單環寬容）。
fn parse_evidence(text: &str) -> Option<Vec<EvidenceTask>> {
    let record: crate::tasks::TouchedRecord = serde_json::from_str(text).ok()?;
    let tasks = if record.entries.is_empty() {
        record
            .touched
            .into_iter()
            .map(|t| EvidenceTask { task_id: t.task_id, files: t.files })
            .collect()
    } else {
        record
            .entries
            .into_iter()
            .map(|e| EvidenceTask { task_id: e.task_id, files: e.touched_files })
            .collect()
    };
    Some(tasks)
}

/// 封存目錄名 `YYYY-MM-DD-<name>` 剝日期前綴；不合格式原樣回傳（寬容）。
fn strip_date_prefix(name: &str) -> &str {
    let b = name.as_bytes();
    let dated = b.len() > 11
        && b[10] == b'-'
        && b[..10]
            .iter()
            .enumerate()
            .all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() });
    if dated {
        &name[11..]
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    const META_WITH_DISCUSSION: &str =
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: origin-talk\n";
    const META_PLAIN: &str = "schema: spec-driven\ncreated: 2026-07-01\n";

    fn spec_with_traces() -> String {
        "# x Specification\n\n## Purpose\n\nX lifecycle.\n\n## Requirements\n\n\
         ### Requirement: R1\n\nIt SHALL work.\n\n<!-- @trace\nsource: alpha\nupdated: 2026-07-10\n-->\n\n\
         ### Requirement: R2\n\nIt SHALL also work.\n\n<!-- @trace\nsource: beta\nupdated: 2026-08-02\n-->\n"
            .to_string()
    }

    /// 封存一個含 capability delta 的 change 到測試 store。
    fn put_archived(store: &TestStore, dated: &str, meta: &str, caps: &[&str]) {
        store.archived_metas.borrow_mut().insert(dated.to_string(), meta.to_string());
        for cap in caps {
            store
                .archived_artifacts
                .borrow_mut()
                .insert((dated.to_string(), format!("specs/{cap}/spec.md")), "delta".to_string());
        }
    }

    fn store_with_canon() -> TestStore {
        let store = TestStore::default();
        store.canonical.borrow_mut().insert("x".to_string(), spec_with_traces());
        store
    }

    // --- 溯源鏈組裝：列舉、排序、進行中排除 ---

    #[test]
    fn archived_dirs_with_the_capability_delta_enter_the_chain_oldest_first() {
        let store = store_with_canon();
        put_archived(&store, "2026-08-02-beta", META_PLAIN, &["x"]);
        put_archived(&store, "2026-07-10-alpha", META_PLAIN, &["x", "y"]);
        put_archived(&store, "2026-08-05-other", META_PLAIN, &["y"]);

        let report = run(&store, "x").expect("assembles");
        let names: Vec<&str> = report.changes.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["alpha", "beta"], "含 x delta 的封存依日期由舊至新");
        assert_eq!(report.changes[0].archived_dir, "2026-07-10-alpha");
    }

    #[test]
    fn an_in_flight_change_with_the_capability_delta_stays_out_of_the_chain() {
        let store = store_with_canon();
        store.metas.borrow_mut().insert("wip".to_string(), META_PLAIN.to_string());
        store.put_artifact("wip", "specs/x/spec.md", "delta");
        put_archived(&store, "2026-07-10-alpha", META_PLAIN, &["x"]);

        let report = run(&store, "x").expect("assembles");
        let names: Vec<&str> = report.changes.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["alpha"], "進行中 change 不入鏈");
    }

    // --- Requirement 歸屬 ---

    #[test]
    fn each_requirement_reports_its_current_trace_source() {
        let store = store_with_canon();
        let report = run(&store, "x").expect("assembles");
        let pairs: Vec<(&str, &str)> = report
            .requirements
            .iter()
            .map(|r| (r.name.as_str(), r.source.as_str()))
            .collect();
        assert_eq!(pairs, [("R1", "alpha"), ("R2", "beta")]);
    }

    #[test]
    fn a_requirement_without_a_trace_block_is_omitted_from_attribution() {
        let store = TestStore::default();
        store.canonical.borrow_mut().insert(
            "x".to_string(),
            "## Requirements\n\n### Requirement: R1\n\nNo trace here.\n".to_string(),
        );
        let report = run(&store, "x").expect("assembles");
        assert!(report.requirements.is_empty(), "無 @trace 的 Requirement 不列歸屬");
    }

    #[test]
    fn a_trace_source_pointing_to_a_missing_archive_dir_is_listed_without_change_detail() {
        // 歸屬照列於 requirements；changes 清單缺其明細（寬容組裝）。
        let store = store_with_canon();
        put_archived(&store, "2026-08-02-beta", META_PLAIN, &["x"]);
        // alpha 無封存目錄。

        let report = run(&store, "x").expect("dirty link must not fail the chain");
        assert!(report.requirements.iter().any(|r| r.source == "alpha"));
        let names: Vec<&str> = report.changes.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["beta"], "找不到封存目錄的歸屬不進 changes");
    }

    // --- evidence 存在性偵測 ---

    #[test]
    fn an_evidence_file_yields_per_task_files_and_absence_yields_none() {
        let store = store_with_canon();
        put_archived(&store, "2026-07-10-alpha", META_PLAIN, &["x"]);
        put_archived(&store, "2026-08-02-beta", META_PLAIN, &["x"]);
        store.archived_artifacts.borrow_mut().insert(
            ("2026-07-10-alpha".to_string(), ".evidence.json".to_string()),
            r#"{"version":2,"change":"alpha","touched":[{"task_id":"1","task_desc":"d","files":["old.rs"]}],"entries":[{"taskId":"tsk_1","taskDesc":"d","touchedFiles":["a.rs","b.rs"],"recordedAt":"2026-07-10T00:00:00Z"}]}"#
                .to_string(),
        );

        let report = run(&store, "x").expect("assembles");
        let alpha = &report.changes[0];
        let tasks = alpha.evidence.as_ref().expect("evidence 存在則解析");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_id, "tsk_1", "v2 entries 優先");
        assert_eq!(tasks[0].files, ["a.rs", "b.rs"]);
        assert!(report.changes[1].evidence.is_none(), "無 .evidence.json 為 None");
    }

    #[test]
    fn a_v1_only_record_falls_back_to_the_touched_channel() {
        let store = store_with_canon();
        put_archived(&store, "2026-07-10-alpha", META_PLAIN, &["x"]);
        store.archived_artifacts.borrow_mut().insert(
            ("2026-07-10-alpha".to_string(), ".evidence.json".to_string()),
            r#"{"change":"alpha","touched":[{"task_id":"1","task_desc":"d","files":["legacy.rs"]}]}"#.to_string(),
        );

        let report = run(&store, "x").expect("assembles");
        let tasks = report.changes[0].evidence.as_ref().expect("v1 記錄仍解析");
        assert_eq!(tasks[0].task_id, "1");
        assert_eq!(tasks[0].files, ["legacy.rs"]);
    }

    #[test]
    fn old_trace_code_lists_are_never_read_into_the_report() {
        let store = TestStore::default();
        store.canonical.borrow_mut().insert(
            "x".to_string(),
            "## Requirements\n\n### Requirement: R1\n\nBody.\n\n<!-- @trace\nsource: alpha\nupdated: 2026-08-03\ncode:\n  - poisoned/path.rs\n-->\n"
                .to_string(),
        );
        put_archived(&store, "2026-08-03-alpha", META_PLAIN, &["x"]);
        // alpha 無 .evidence.json：evidence 必須是 None，而非 @trace 的 code 清單。

        let report = run(&store, "x").expect("assembles");
        assert!(report.changes[0].evidence.is_none(), "@trace code 清單永不回讀");
        assert_eq!(report.requirements[0].source, "alpha", "source 照常解析");
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("poisoned"), "code 清單不得出現在任何輸出欄位");
    }

    // --- from_discussion 與討論扇出 ---

    #[test]
    fn from_discussion_flows_through_and_a_missing_field_is_null() {
        let store = store_with_canon();
        put_archived(&store, "2026-07-10-alpha", META_WITH_DISCUSSION, &["x"]);
        put_archived(&store, "2026-08-02-beta", META_PLAIN, &["x"]);

        let report = run(&store, "x").expect("assembles");
        assert_eq!(report.changes[0].from_discussion.as_deref(), Some("origin-talk"));
        assert!(report.changes[1].from_discussion.is_none(), "缺欄為 null");
    }

    #[test]
    fn an_unparseable_archived_meta_degrades_to_null_without_failing() {
        let store = store_with_canon();
        put_archived(&store, "2026-07-10-alpha", "{{not yaml", &["x"]);

        let report = run(&store, "x").expect("髒 meta 不使整鏈失敗");
        assert!(report.changes[0].from_discussion.is_none());
    }

    #[test]
    fn the_source_discussion_carries_its_fanout_with_sibling_capabilities() {
        let store = store_with_canon();
        put_archived(&store, "2026-07-10-alpha", META_WITH_DISCUSSION, &["x"]);
        put_archived(&store, "2026-07-12-cousin", META_PLAIN, &["y", "z"]);
        store.archived_discussions.borrow_mut().insert(
            "origin-talk".to_string(),
            "---\ntopic: t\nslug: origin-talk\nstatus: promoted\npromoted_to: alpha, cousin\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: go\n"
                .to_string(),
        );

        let report = run(&store, "x").expect("assembles");
        assert_eq!(report.discussions.len(), 1);
        let d = &report.discussions[0];
        assert_eq!(d.slug, "origin-talk");
        assert!(d.archived, "封存討論照樣讀出");
        let fanout: Vec<(&str, &[String])> = d
            .promoted_to
            .iter()
            .map(|p| (p.change.as_str(), p.capabilities.as_slice()))
            .collect();
        assert_eq!(fanout.len(), 2);
        assert_eq!(fanout[0].0, "alpha");
        assert_eq!(fanout[0].1, ["x"]);
        assert_eq!(fanout[1].0, "cousin");
        assert_eq!(fanout[1].1, ["y", "z"]);
    }

    #[test]
    fn a_missing_discussion_document_drops_silently_while_the_chain_survives() {
        let store = store_with_canon();
        put_archived(&store, "2026-07-10-alpha", META_WITH_DISCUSSION, &["x"]);
        // origin-talk 討論檔不存在。

        let report = run(&store, "x").expect("討論檔缺失不使整鏈失敗");
        assert_eq!(report.changes[0].from_discussion.as_deref(), Some("origin-talk"));
        assert!(report.discussions.is_empty());
    }

    // --- 找不到 capability（近似建議細節屬 task 1.2） ---

    #[test]
    fn an_unknown_capability_fails_with_up_to_three_suggestions() {
        let store = TestStore::default();
        for cap in ["user-auth", "auth-session", "auth-token", "billing"] {
            store
                .canonical
                .borrow_mut()
                .insert(cap.to_string(), format!("# {cap}\n\n## Purpose\n\n{cap} purpose.\n"));
        }

        let err = run(&store, "auth").expect_err("無正典規格必須失敗");
        let msg = err.to_string();
        assert!(msg.contains("'auth' is not in the canonical specs"), "{msg}");
        let count = msg.matches("  - ").count();
        assert!(count >= 1 && count <= 3, "至多三筆建議: {msg}");
        assert!(!msg.contains("billing"), "無關名稱不入建議: {msg}");
    }

    #[test]
    fn an_unknown_capability_with_no_neighbors_reports_plainly() {
        let store = TestStore::default();
        let err = run(&store, "totally-new").expect_err("must fail");
        let msg = err.to_string();
        assert!(msg.contains("not in the canonical specs"), "{msg}");
        assert!(!msg.contains("  - "), "無近似時不列建議: {msg}");
    }

    // --- 日期前綴 ---

    #[test]
    fn date_prefix_stripping_tolerates_malformed_names() {
        assert_eq!(strip_date_prefix("2026-07-10-alpha"), "alpha");
        assert_eq!(strip_date_prefix("2026-07-10-a-b-c"), "a-b-c");
        assert_eq!(strip_date_prefix("not-dated"), "not-dated");
        assert_eq!(strip_date_prefix("2026-07-10"), "2026-07-10");
        assert_eq!(strip_date_prefix("20a6-07-10-alpha"), "20a6-07-10-alpha");
    }
}
