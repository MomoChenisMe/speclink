//! `change schema`：切換作用中變更的 schema（change-schema-verb design D1–D3）。
//! 守門依序全過之後，以一次 `edit_meta` 手術改 `schema:` 行並清掉驗證章。

use crate::schema::Schema;
use crate::station::{NotFound, VERIFY};
use crate::store::Store;

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
/// specs 時變更不得帶 delta、沒有未結的驗證工單；查無是 [`NotFound`]，其餘失敗是
/// Refusal（壞 meta 是 MetaError），一律零寫入。守門全過且與現值相同時不寫檔。切換以
/// 一次寫入完成：取代或新增 `schema:` 行，並移除驗證章（依舊 schema 的標準蓋的）。
pub fn set_schema(
    store: &dyn Store,
    name: &str,
    schema: &str,
    resolve: &dyn Fn(&str) -> Option<Result<Schema, String>>,
) -> anyhow::Result<SchemaWrite> {
    if !crate::model::list_changes(store).iter().any(|c| c.name == name) {
        return Err(NotFound(format!("Change '{name}' not found.")).into());
    }
    let refuse = |msg: String| anyhow::Error::from(crate::command::Refusal(msg));
    let previous =
        crate::model::check_meta_text(store, name, store.read_change_meta(name).as_deref())?.schema_name();
    // 名稱守門在解析之前：專案層解析把名字接成路徑，寫入把它原樣接進 meta。
    if !crate::schema::is_valid_schema_name(schema) {
        return Err(refuse(crate::schema::invalid_name_msg(schema)));
    }
    let target = match resolve(schema) {
        Some(Ok(s)) => s,
        Some(Err(e)) => return Err(refuse(e)),
        None => return Err(refuse(crate::schema::not_found_msg(schema))),
    };
    if !target.has_specs() {
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
    let meta_file = store.change_meta_display(name);
    let cleared = crate::model::edit_meta(store, name, |m| {
        m.set("schema", schema)?;
        // 清章的欄位清單由 station 管；`verified_rounds` 不在內，流程成本跨章累加（D3）。
        let stamp = VERIFY.stamp_keys();
        let cleared = stamp.iter().any(|key| m.get(key).is_some());
        for key in &stamp {
            m.remove(key);
        }
        // KeyLines 只認第 0 欄的 `schema:`：`schema : x` 會讓手術補出重複鍵、縮排續行
        // 會被併進新值。寫入前以解析結果把關，讀不出新值就零寫入。
        let reparsed = crate::model::check_meta_text(store, name, Some(&m.text()));
        if !reparsed.is_ok_and(|meta| meta.schema_name() == schema) {
            return Err(refuse(format!(
                "cannot switch '{name}': {meta_file} has a `schema:` line this command cannot \
                 rewrite — make it a plain `schema: <name>` line first"
            )));
        }
        Ok(cleared)
    })?
    // 名冊列得出目錄卻讀不到文件：這個 change 根本沒有 `.meta.yaml`。
    .ok_or_else(|| refuse(format!("cannot write schema for '{name}': {meta_file} is missing")))?;
    Ok(write(true, cleared))
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
        // Scenario「變更不存在」：名稱逐字比對作用中名冊——大小寫不同（不分大小寫的
        // 檔案系統上目錄看得到）也是查無，錯誤型別同其他動詞的查無。
        let store = TestStore::with_meta("c", SPEC_META);
        for name in ["ghost", "C"] {
            let err = switch(&store, name, "spec-driven").unwrap_err();
            assert_eq!(err.to_string(), format!("Change '{name}' not found."));
            assert!(err.downcast_ref::<crate::station::NotFound>().is_some(), "typed not-found: {err}");
        }
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_schema_line_the_edit_cannot_reach_refuses_without_writing() {
        // KeyLines 只認第 0 欄的 `schema:`：`schema : x`（多一個空白）會讓手術補出
        // 重複鍵，縮排續行會被併進新值——兩者都在寫入前被擋下。
        for meta in [
            "schema : spec-driven\ncreated: 2026-10-01\n",
            "schema:\n  spec-driven\ncreated: 2026-10-01\n",
        ] {
            let store = TestStore::with_meta("c", meta);
            let err = switch(&store, "c", "no-spec-change").expect_err(meta);
            assert!(err.to_string().contains(".meta.yaml"), "{err}");
            assert_refused(&store, &err, meta);
        }
    }

    #[test]
    fn a_change_without_a_meta_file_refuses() {
        // 名冊列得出目錄卻讀不到 `.meta.yaml`：拒絕，不憑空建出 meta。
        let store = TestStore::with_meta("c", SPEC_META);
        *store.missing_meta.borrow_mut() = Some("c".into());
        let err = switch(&store, "c", "no-spec-change").unwrap_err();
        assert_eq!(
            err.to_string(),
            "cannot write schema for 'c': speclink/changes/c/.meta.yaml is missing"
        );
        assert_refused(&store, &err, SPEC_META);
    }
}
