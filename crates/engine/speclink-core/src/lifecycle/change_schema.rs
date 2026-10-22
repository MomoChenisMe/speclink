//! `change schema`：切換作用中變更的 schema（change-schema-verb design D1–D3）。
//! 守門依序全過之後，以一次 `KeyLines` 手術改 `schema:` 行並清掉驗證章。

use crate::keylines::KeyLines;
use crate::schema::Schema;
use crate::store::Store;

/// 驗證章的五個欄位。`verified_rounds` 刻意不在內：流程成本的輪數跨章累加（design D3）。
const VERIFY_STAMP_KEYS: [&str; 5] =
    ["verified_at", "verified_by", "verified_with", "verified_tasks_total", "verified_scope"];

/// 一次切換的結果：`changed` 為 false 時沒有寫檔（同值，不發事件）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaWrite {
    pub previous: String,
    pub schema: String,
    pub changed: bool,
    pub verify_stamp_cleared: bool,
}

/// 把作用中變更 `name` 的 schema 切到 `schema`。`resolve` 由呼叫端注入（design D2）：
/// fs 模式看專案層、user 層與內建，server 只看內建。守門依序：作用中變更（名稱逐字
/// 比對名冊）、metadata 可解析、schema 名稱是 kebab-case 且解析得到、目標 schema 沒有
/// specs 時變更不得帶 delta、沒有未結的驗證工單；任一失敗零寫入。守門全過且與現值
/// 相同時不寫檔。切換以一次寫入完成：取代或新增 `schema:` 行，並移除驗證章（依舊
/// schema 的標準蓋的）。
pub fn set_schema(
    store: &dyn Store,
    name: &str,
    schema: &str,
    resolve: &dyn Fn(&str) -> Option<Result<Schema, String>>,
) -> anyhow::Result<SchemaWrite> {
    if !crate::model::list_changes(store).iter().any(|c| c.name == name) {
        anyhow::bail!("Change '{name}' not found.");
    }
    let refuse = |msg: String| anyhow::Error::from(crate::command::Refusal(msg));
    let raw = store.read_change_meta(name);
    let previous = crate::model::check_meta_text(name, raw.as_deref())?.schema_name();
    // 名稱守門在解析之前：專案層解析把名字接成路徑，寫入把它原樣接進 meta。
    if !crate::schema::is_valid_schema_name(schema) {
        return Err(refuse(crate::schema::invalid_name_msg(schema)));
    }
    let target = match resolve(schema) {
        Some(Ok(s)) => s,
        Some(Err(e)) => return Err(refuse(e)),
        None => return Err(refuse(crate::schema::not_found_msg(schema))),
    };
    if target.artifact("specs").is_none() {
        let deltas = store.delta_capabilities(name);
        if !deltas.is_empty() {
            return Err(refuse(format!(
                "cannot switch '{name}' to '{schema}': '{schema}' has no specs, but the change has \
                 delta specs ({}) — remove them first",
                deltas.join(", ")
            )));
        }
    }
    if store.artifact_exists(name, crate::station::VERIFY_DOC) {
        return Err(refuse(format!(
            "cannot switch '{name}' while its verify ticket is open — abandon it first: \
             speclink verify discard {name}"
        )));
    }
    let write = |changed: bool, verify_stamp_cleared: bool| SchemaWrite {
        previous: previous.clone(),
        schema: schema.to_string(),
        changed,
        verify_stamp_cleared,
    };
    if previous == schema {
        return Ok(write(false, false));
    }
    // 名冊列得出目錄卻讀不到文件：這個 change 根本沒有 `.openspec.yaml`。
    let raw = raw.ok_or_else(|| {
        anyhow::anyhow!(
            "cannot write schema for '{name}': openspec/changes/{name}/.openspec.yaml is missing"
        )
    })?;
    let mut m = KeyLines::whole(&raw);
    m.set("schema", schema)?;
    let verify_stamp_cleared = VERIFY_STAMP_KEYS.iter().any(|key| m.get(key).is_some());
    for key in VERIFY_STAMP_KEYS {
        m.remove(key);
    }
    store.write_change_meta(name, &m.text())?;
    Ok(write(true, verify_stamp_cleared))
}

#[cfg(test)]
mod tests {
    use super::{set_schema, SchemaWrite};
    use crate::schema::{resolve_with, Schema};
    use crate::store::Store;
    use crate::teststore::TestStore;

    const SPEC_META: &str = "schema: spec-driven\ncreated: 2026-10-01\n";

    fn builtin(name: &str) -> Option<Result<Schema, String>> {
        resolve_with(None, None, name)
    }

    fn switch(store: &TestStore, name: &str, schema: &str) -> anyhow::Result<SchemaWrite> {
        set_schema(store, name, schema, &builtin)
    }

    /// 拒絕一律是 Refusal、meta 逐位元不變且零寫入。
    fn assert_refused(store: &TestStore, err: &anyhow::Error, meta: &str) {
        assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "a refusal: {err}");
        assert_eq!(store.meta("c"), meta, "meta unchanged");
        assert_eq!(*store.meta_writes.borrow(), 0, "zero writes");
    }

    #[test]
    fn switching_to_no_spec_change_rewrites_only_the_schema_line() {
        // Scenario「切到不改規格」。
        let store = TestStore::with_meta("tidy", SPEC_META);
        let write = switch(&store, "tidy", "no-spec-change").unwrap();
        assert_eq!(
            write,
            SchemaWrite {
                previous: "spec-driven".into(),
                schema: "no-spec-change".into(),
                changed: true,
                verify_stamp_cleared: false,
            }
        );
        assert_eq!(store.meta("tidy"), "schema: no-spec-change\ncreated: 2026-10-01\n");
        assert_eq!(*store.meta_writes.borrow(), 1, "one write");
    }

    #[test]
    fn a_meta_without_a_schema_line_gains_one() {
        let store = TestStore::with_meta("tidy", "created: 2026-10-01\n");
        let write = switch(&store, "tidy", "no-spec-change").unwrap();
        assert_eq!(write.previous, "spec-driven", "an absent schema line reads as spec-driven");
        assert_eq!(store.meta("tidy"), "created: 2026-10-01\nschema: no-spec-change\n");
    }

    #[test]
    fn a_change_with_deltas_cannot_switch_to_a_schema_without_specs() {
        // Scenario「有 delta 時拒絕切到不改規格」：訊息列出 capability。
        let store = TestStore::with_meta("c", SPEC_META);
        store.put_artifact("c", "specs/auth/spec.md", "## ADDED Requirements\n");
        let err = switch(&store, "c", "no-spec-change").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("auth") && msg.contains("no-spec-change"), "{msg}");
        assert_refused(&store, &err, SPEC_META);
    }

    #[test]
    fn an_unresolvable_schema_refuses_with_the_not_found_message() {
        // Scenario「解析不到的 schema 拒絕」。
        let store = TestStore::with_meta("c", SPEC_META);
        let err = switch(&store, "c", "no-such-flow").unwrap_err();
        assert_eq!(err.to_string(), crate::schema::not_found_msg("no-such-flow"));
        assert_refused(&store, &err, SPEC_META);
    }

    #[test]
    fn an_open_verify_ticket_refuses_and_points_at_discard() {
        // Scenario「未結驗證工單時拒絕」：同值也照樣拒絕（守門先於同值判定）。
        let store = TestStore::with_meta("c", SPEC_META);
        store.put_artifact("c", "verify.md", "# Verify\n");
        let err = switch(&store, "c", "spec-driven").unwrap_err();
        assert!(err.to_string().contains("speclink verify discard c"), "{err}");
        assert_refused(&store, &err, SPEC_META);
        assert!(store.artifact_exists("c", "verify.md"), "the ticket stays");
    }

    #[test]
    fn a_corrupt_meta_refuses_without_writing() {
        let bad = "schema: [\n";
        let store = TestStore::with_meta("c", bad);
        let err = switch(&store, "c", "no-spec-change").unwrap_err();
        assert!(err.downcast_ref::<crate::model::MetaError>().is_some(), "a meta error: {err}");
        assert_eq!(store.meta("c"), bad);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn switching_clears_the_verify_stamp_and_keeps_rounds_and_the_review_stamp() {
        // Scenario「切換時移除驗證章、保留審查章與輪數」：同一次寫入。
        let meta = "schema: no-spec-change\ncreated: 2026-10-01\n\
reviewed_at: 2026-10-02T10:00:00+08:00\nreviewed_by: R <r@example.com>\nreviewed_with: claude\n\
reviewed_tasks_total: 3\nreviewed_rounds: 1\nreviewed_scope:\n  - path: src/a.rs\n    hash: aa\n\
verified_at: 2026-10-02T11:00:00+08:00\nverified_by: V <v@example.com>\nverified_with: claude\n\
verified_tasks_total: 3\nverified_rounds: 2\nverified_scope:\n  - path: src/a.rs\n    hash: bb\n";
        let store = TestStore::with_meta("c", meta);
        let write = switch(&store, "c", "spec-driven").unwrap();
        assert!(write.changed && write.verify_stamp_cleared, "{write:?}");
        assert_eq!(
            store.meta("c"),
            "schema: spec-driven\ncreated: 2026-10-01\n\
reviewed_at: 2026-10-02T10:00:00+08:00\nreviewed_by: R <r@example.com>\nreviewed_with: claude\n\
reviewed_tasks_total: 3\nreviewed_rounds: 1\nreviewed_scope:\n  - path: src/a.rs\n    hash: aa\n\
verified_rounds: 2\n"
        );
        assert_eq!(*store.meta_writes.borrow(), 1, "one write");
    }

    #[test]
    fn the_same_schema_writes_nothing() {
        // Scenario「同一個 schema 不寫檔」。
        let store = TestStore::with_meta("c", SPEC_META);
        let write = switch(&store, "c", "spec-driven").unwrap();
        assert!(!write.changed && !write.verify_stamp_cleared, "{write:?}");
        assert_eq!(store.meta("c"), SPEC_META);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn crlf_line_endings_survive_the_switch() {
        let store = TestStore::with_meta("c", "schema: spec-driven\r\ncreated: 2026-10-01\r\n");
        switch(&store, "c", "no-spec-change").unwrap();
        assert_eq!(store.meta("c"), "schema: no-spec-change\r\ncreated: 2026-10-01\r\n");
    }

    #[test]
    fn a_schema_name_that_is_not_kebab_case_refuses_before_resolution() {
        // sharp-edges 稽核（task 6.1）：專案層解析把名字接成路徑、寫入把它接進 meta。
        // 解析函式即使什麼都認，路徑分隔符、`..`、空字串、大寫與換行也一律拒絕且零寫入。
        let store = TestStore::with_meta("c", SPEC_META);
        let anything = |_: &str| Some(Ok(crate::schema::spec_driven()));
        for bad in ["../evil", "a/b", "a\\b", "", "Spec-Driven", "spec driven", "x\nschema: y"] {
            let err = set_schema(&store, "c", bad, &anything).expect_err(&format!("{bad:?} must refuse"));
            assert!(err.to_string().contains("must be lowercase kebab-case"), "{bad:?}: {err}");
            assert_refused(&store, &err, SPEC_META);
        }
    }

    #[test]
    fn an_unknown_change_is_not_found_and_not_a_refusal() {
        // Scenario「變更不存在」：名稱逐字比對作用中名冊。
        let store = TestStore::with_meta("c", SPEC_META);
        for name in ["ghost", "C"] {
            let err = switch(&store, name, "spec-driven").unwrap_err();
            assert_eq!(err.to_string(), format!("Change '{name}' not found."));
            assert!(err.downcast_ref::<crate::command::Refusal>().is_none(), "not a refusal");
        }
        assert_eq!(*store.meta_writes.borrow(), 0);
    }
}
