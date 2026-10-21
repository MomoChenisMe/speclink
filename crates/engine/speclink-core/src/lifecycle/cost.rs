//! 一個封存變更的流程成本（process-cost-metrics design D4）：只依賴 [`Store`]，
//! desktop 快取建表時逐項呼叫；中位數與 P90 的彙總屬呈現層，不在這裡。

use crate::store::Store;

/// 單一封存變更的流程成本。來源缺席或無法解析的欄位為 `None`——呼叫端據此
/// 不輸出該欄位，不以 0 佔位。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessCost {
    pub doc_lines: Option<u64>,
    pub lead_days: Option<i64>,
    pub lead_hours: Option<f64>,
    pub review_rounds: Option<u64>,
    pub verify_rounds: Option<u64>,
}

/// 計算封存目錄 `dated_name`（`YYYY-MM-DD-<name>`）的流程成本。歸檔 metadata
/// 壞檔不在 fail-closed 範圍（同 desktop 快取）——讀作沒有 metadata。
pub fn archived_cost(store: &dyn Store, dated_name: &str) -> ProcessCost {
    let meta = crate::model::ChangeMeta::from_text(store.read_archived_meta(dated_name).as_deref())
        .unwrap_or_default();
    let archived_date =
        dated_name.get(..10).and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let created_date = meta
        .created
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let lead_days = archived_date
        .zip(created_date)
        .map(|(archived, created)| (archived - created).num_days().max(0));
    let stamp = |v: Option<&str>| v.and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok());
    let lead_hours = stamp(meta.archived_at.as_deref())
        .zip(stamp(meta.created_stamp.as_deref()))
        .map(|(archived, created)| {
            let hours = (archived - created).num_seconds().max(0) as f64 / 3600.0;
            (hours * 10.0).round() / 10.0
        });
    ProcessCost {
        doc_lines: doc_lines(store, dated_name),
        lead_days,
        lead_hours,
        review_rounds: meta.reviewed_rounds,
        verify_rounds: meta.verified_rounds,
    }
}

/// proposal.md、design.md、tasks.md 與每份 delta spec 的行數總和；四類都不存在時 `None`。
fn doc_lines(store: &dyn Store, dated_name: &str) -> Option<u64> {
    let specs = store
        .archived_delta_capabilities(dated_name)
        .into_iter()
        .map(|cap| crate::model::delta_spec_artifact(&cap));
    let docs: Vec<String> = ["proposal.md", "design.md", "tasks.md"]
        .map(String::from)
        .into_iter()
        .chain(specs)
        .filter_map(|artifact| store.read_archived_artifact(dated_name, &artifact))
        .collect();
    (!docs.is_empty()).then(|| docs.iter().map(|text| line_count(text)).sum())
}

/// 換行字元數；最後一行沒有換行時也算一行。
fn line_count(text: &str) -> u64 {
    let newlines = text.matches('\n').count() as u64;
    newlines + u64::from(!text.is_empty() && !text.ends_with('\n'))
}

#[cfg(test)]
mod tests {
    use super::{archived_cost, ProcessCost};
    use crate::teststore::TestStore;

    fn lines(n: usize) -> String {
        (1..=n).map(|i| format!("line {i}\n")).collect()
    }

    fn put(store: &TestStore, dated: &str, artifact: &str, text: &str) {
        store
            .archived_artifacts
            .borrow_mut()
            .insert((dated.to_string(), artifact.to_string()), text.to_string());
    }

    #[test]
    fn new_format_item_carries_every_cost_field() {
        // spec client-protocol scenario「新格式封存項帶齊流程成本欄位」。delta spec 的
        // 最後一行沒有換行，也算一行。
        let store = TestStore::default();
        let dated = "2026-10-22-demo";
        store.archived_metas.borrow_mut().insert(
            dated.to_string(),
            "schema: spec-driven\ncreated: 2026-10-21T10:00:00+08:00\nreviewed_rounds: 2\nverified_rounds: 3\narchived_at: 2026-10-22T16:30:00+08:00\n".to_string(),
        );
        put(&store, dated, "proposal.md", &lines(120));
        put(&store, dated, "tasks.md", &lines(80));
        put(&store, dated, "specs/auth/spec.md", lines(50).trim_end_matches('\n'));
        assert_eq!(
            archived_cost(&store, dated),
            ProcessCost {
                doc_lines: Some(250),
                lead_days: Some(1),
                lead_hours: Some(30.5),
                review_rounds: Some(2),
                verify_rounds: Some(3),
            }
        );
    }

    #[test]
    fn old_format_item_carries_only_what_it_has() {
        // spec client-protocol scenario「舊格式封存項只帶得出的欄位」。
        let store = TestStore::default();
        let dated = "2026-08-14-old";
        store
            .archived_metas
            .borrow_mut()
            .insert(dated.to_string(), "schema: spec-driven\ncreated: 2026-08-12\narchived_at: 2026-08-14\n".to_string());
        put(&store, dated, "proposal.md", &lines(10));
        put(&store, dated, "tasks.md", &lines(5));
        assert_eq!(
            archived_cost(&store, dated),
            ProcessCost { doc_lines: Some(15), lead_days: Some(2), ..ProcessCost::default() }
        );
    }

    #[test]
    fn item_without_metadata_or_documents_has_no_cost_fields() {
        // spec client-protocol scenario「沒有 metadata 與文件時欄位全部缺席」。
        let store = TestStore::default();
        assert_eq!(archived_cost(&store, "2026-08-15-empty"), ProcessCost::default());
    }

    #[test]
    fn lead_time_never_goes_negative_and_skips_unrecognised_dates() {
        let store = TestStore::default();
        store.archived_metas.borrow_mut().insert(
            "2026-08-10-early".to_string(),
            "created: 2026-08-12T09:00:00+08:00\narchived_at: 2026-08-12T08:00:00+08:00\n".to_string(),
        );
        store
            .archived_metas
            .borrow_mut()
            .insert("2026-08-10-odd".to_string(), "created: someday\n".to_string());
        let early = archived_cost(&store, "2026-08-10-early");
        assert_eq!((early.lead_days, early.lead_hours), (Some(0), Some(0.0)));
        assert_eq!(archived_cost(&store, "2026-08-10-odd"), ProcessCost::default());
    }
}
