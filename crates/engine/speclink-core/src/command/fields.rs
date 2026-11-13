//! The free-field edit commands (host-free-fields D3): `change field set` /
//! `unset` and their discussion twins.
//!
//! One command carries both lists, so a caller with no argv (the HTTP route)
//! can set and unset in one write. The rules and the write are
//! [`crate::fields`]; this module finds the document and types the failures.

use super::{classify, Command, CommandError, CommandOutcome, ErrorCode};
use crate::fields::Fields;
use crate::store::Store;

impl Command {
    /// The free fields this command creates its change or discussion with;
    /// `None` for a command that creates neither. The three create commands
    /// are named here and in [`Command::creation_fields_mut`] only.
    pub fn creation_fields(&self) -> Option<&Fields> {
        match self {
            Command::NewChange { fields, .. }
            | Command::DiscussNew { fields, .. }
            | Command::DiscussPromote { fields, .. } => Some(fields),
            _ => None,
        }
    }

    /// [`Command::creation_fields`], to add to them.
    pub fn creation_fields_mut(&mut self) -> Option<&mut Fields> {
        match self {
            Command::NewChange { fields, .. }
            | Command::DiscussNew { fields, .. }
            | Command::DiscussPromote { fields, .. } => Some(fields),
            _ => None,
        }
    }
}

/// The outcome of a field edit on a change or a discussion: every field
/// after the call, and the keys this call changed (in byte order). An empty
/// `changed` is a call that wrote nothing, and it reports no event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldsOutcome {
    /// The change's name or the discussion's slug.
    pub subject: String,
    pub fields: Fields,
    pub changed: Vec<String>,
}

fn outcome(subject: String, edited: crate::fields::Edited) -> FieldsOutcome {
    FieldsOutcome { subject, fields: edited.fields, changed: edited.changed }
}

pub(super) fn run_change(
    store: &dyn Store,
    name: String,
    set: &Fields,
    unset: &[String],
) -> Result<CommandOutcome, CommandError> {
    let not_found = || CommandError::new(ErrorCode::NotFound, format!("Change '{name}' not found."));
    if crate::model::find_change(store, &name).is_none() {
        return Err(not_found());
    }
    let edited = crate::fields::edit_change(store, &name, set, unset)
        .map_err(classify)?
        // A change the roster lists without a metadata document has nowhere
        // to keep fields.
        .ok_or_else(not_found)?;
    Ok(CommandOutcome::ChangeFieldsEdit(outcome(name, edited)))
}

pub(super) fn run_discussion(
    store: &dyn Store,
    slug: String,
    set: &Fields,
    unset: &[String],
) -> Result<CommandOutcome, CommandError> {
    let Some(edited) = crate::fields::edit_discussion(store, &slug, set, unset).map_err(classify)? else {
        let message = if store.archived_discussion_exists(&slug) {
            format!("discussion '{slug}' not found among the live discussions — it is archived, and an archived record does not change")
        } else {
            format!("discussion '{slug}' not found — run `speclink discuss new` first")
        };
        return Err(CommandError::new(ErrorCode::NotFound, message));
    };
    Ok(CommandOutcome::DiscussFieldsEdit(outcome(slug, edited)))
}

#[cfg(test)]
mod tests {
    use crate::command::{execute, Command, CommandError, CommandOutcome, DomainEvent, ErrorCode, ExecutionContext};
    use crate::fields::Fields;
    use crate::model::{ChangeMeta, ReviewedScopeEntry};
    use crate::plan::Side;
    use crate::store::Store;
    use crate::teststore::TestStore;
    use crate::workspace::Workspace;

    const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
    const RECORD: &str = "---\ntopic: Board search\nslug: board-search\nstatus: open\ncreated: 2026-11-01\n---\n\n\
                          # Discussion: Board search\n\n## Context\n\n## Rounds\n\n## Conclusion\n\nShip it.\n";

    fn map(pairs: &[(&str, &str)]) -> Fields {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn strings(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    fn change_edit(name: &str, set: &[(&str, &str)], unset: &[&str]) -> Command {
        Command::ChangeFieldsEdit { name: name.to_string(), set: map(set), unset: strings(unset) }
    }

    fn discuss_edit(slug: &str, set: &[(&str, &str)], unset: &[&str]) -> Command {
        Command::DiscussFieldsEdit { slug: slug.to_string(), set: map(set), unset: strings(unset) }
    }

    /// A workspace whose root does not exist: the flows that look at the host
    /// (archive's snapshot, task done's git probe) find nothing and write nothing.
    fn ctx() -> ExecutionContext {
        ExecutionContext {
            workspace: Some(Workspace {
                root: std::env::temp_dir().join("speclink-fields-test-ghost-root"),
                spec_dir_name: "openspec".to_string(),
            }),
            ..Default::default()
        }
    }

    fn run(store: &TestStore, cmd: Command) -> Result<(CommandOutcome, Vec<DomainEvent>), CommandError> {
        execute(store, &ctx(), cmd)
    }

    fn ok(store: &TestStore, cmd: Command) -> (CommandOutcome, Vec<DomainEvent>) {
        let name = cmd.name();
        run(store, cmd).unwrap_or_else(|e| panic!("{name}: {}", e.message))
    }

    fn fields_of(store: &TestStore, name: &str) -> Fields {
        ChangeMeta::from_text(Some(&store.meta(name))).expect("the metadata parses").fields
    }

    #[test]
    fn only_the_three_create_commands_have_creation_fields() {
        let mut create = new_change("demo", &[("a", "1")]);
        assert_eq!(create.creation_fields(), Some(&map(&[("a", "1")])));
        create.creation_fields_mut().expect("a create command").insert("b".to_string(), "2".to_string());
        assert_eq!(create.creation_fields(), Some(&map(&[("a", "1"), ("b", "2")])));
        let promote = Command::DiscussPromote { slug: "t".to_string(), name: None, last: false, fields: Fields::new() };
        let new = Command::DiscussNew { topic: "T".to_string(), slug: None, kind: None, fields: Fields::new() };
        assert!(promote.creation_fields().is_some() && new.creation_fields().is_some());
        // 編輯命令寫的是既有的文件，不是建立時附帶。
        assert!(change_edit("demo", &[("a", "1")], &[]).creation_fields().is_none());
        assert!(Command::DiscussList { archived: false }.creation_fields().is_none());
    }

    // --- D3：套用順序、結果與事件 ---

    #[test]
    fn unset_applies_before_set_and_the_outcome_lists_every_field_and_the_changed_keys() {
        let store = TestStore::with_meta("demo", META);
        ok(&store, change_edit("demo", &[("a", "1"), ("b", "2"), ("keep", "k")], &[]));
        let (outcome, events) = ok(&store, change_edit("demo", &[("a", "9"), ("c", "3"), ("keep", "k")], &["b", "ghost"]));
        match outcome {
            CommandOutcome::ChangeFieldsEdit(o) => {
                assert_eq!(o.subject, "demo");
                assert_eq!(o.fields, map(&[("a", "9"), ("c", "3"), ("keep", "k")]));
                // 實際變動的鍵，依位元組順序；值相同的 keep 與不存在的 ghost 不算。
                assert_eq!(o.changed, ["a", "b", "c"]);
            }
            other => panic!("expected a fields outcome, got {other:?}"),
        }
        assert_eq!(fields_of(&store, "demo"), map(&[("a", "9"), ("c", "3"), ("keep", "k")]));
        match events.as_slice() {
            // 完整解構：事件只有名稱、鍵與時間。
            [DomainEvent::ChangeFieldsChanged { change, keys, occurred_at: _ }] => {
                assert_eq!(change, "demo");
                assert_eq!(keys, &["a", "b", "c"]);
            }
            other => panic!("expected one change-fields-changed event, got {other:?}"),
        }
        assert_eq!(events[0].kind(), "change-fields-changed");
    }

    #[test]
    fn an_edit_is_one_write_of_the_metadata() {
        let store = TestStore::with_meta("demo", META);
        ok(&store, change_edit("demo", &[("a", "1"), ("b", "2"), ("c", "3")], &[]));
        assert_eq!(*store.meta_writes.borrow(), 1);
    }

    #[test]
    fn one_bad_entry_refuses_the_whole_edit_with_zero_writes() {
        // 規格「全部寫入或全部不寫」：ok=1 Bad=2。
        let store = TestStore::with_meta("demo", META);
        let err = run(&store, change_edit("demo", &[("ok", "1"), ("Bad", "2")], &[])).expect_err("Bad is refused");
        assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
        assert_eq!(store.meta("demo"), META);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_key_both_set_and_unset_or_an_empty_edit_is_invalid_argv() {
        let store = TestStore::with_meta("demo", META);
        ok(&store, change_edit("demo", &[("a", "1")], &[]));
        let before = store.meta("demo");
        for cmd in [change_edit("demo", &[("a", "2")], &["a"]), change_edit("demo", &[], &[])] {
            let err = run(&store, cmd).expect_err("refused");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
        }
        assert_eq!(store.meta("demo"), before);
        assert_eq!(*store.meta_writes.borrow(), 1);

        let store = TestStore::with_live_discussion("board-search", RECORD);
        for cmd in [discuss_edit("board-search", &[("a", "2")], &["a"]), discuss_edit("board-search", &[], &[])] {
            let err = run(&store, cmd).expect_err("refused");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
        }
        assert_eq!(store.discussion("board-search"), RECORD);
    }

    #[test]
    fn unsetting_a_key_that_is_not_there_changes_nothing_and_reports_no_event() {
        // 規格「沒有變動」。
        let store = TestStore::with_meta("demo", META);
        let (outcome, events) = ok(&store, change_edit("demo", &[], &["ghost"]));
        match outcome {
            CommandOutcome::ChangeFieldsEdit(o) => {
                assert!(o.fields.is_empty() && o.changed.is_empty());
            }
            other => panic!("expected a fields outcome, got {other:?}"),
        }
        assert!(events.is_empty());
        assert_eq!(store.meta("demo"), META);
        assert_eq!(*store.meta_writes.borrow(), 0);

        // 寫同一個值也是沒有變動。
        ok(&store, change_edit("demo", &[("a", "1")], &[]));
        let (_, events) = ok(&store, change_edit("demo", &[("a", "1")], &[]));
        assert!(events.is_empty());
        assert_eq!(*store.meta_writes.borrow(), 1);
    }

    #[test]
    fn the_event_carries_the_keys_and_never_a_value() {
        // 規格「事件只帶鍵」：change field set demo secret=abc。
        let store = TestStore::with_meta("demo", META);
        let (_, events) = ok(&store, change_edit("demo", &[("secret", "abc")], &[]));
        assert_eq!(events.len(), 1);
        let text = format!("{:?}", events[0]);
        assert!(text.contains("demo") && text.contains("secret"), "{text}");
        assert!(!text.contains("abc"), "{text}");
    }

    #[test]
    fn a_refusal_message_never_carries_a_value() {
        let store = TestStore::with_meta("demo", META);
        let err = run(&store, change_edit("demo", &[("token", "s3cret\nline")], &[])).expect_err("refused");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
        assert!(err.message.contains("token") && !err.message.contains("s3cret"), "{}", err.message);
    }

    // --- 找不到 ---

    #[test]
    fn a_change_that_does_not_exist_is_not_found() {
        let store = TestStore::with_meta("demo", META);
        for name in ["ghost", "../demo", "archive"] {
            let err = run(&store, change_edit(name, &[("a", "1")], &[])).expect_err(name);
            assert_eq!(err.code, ErrorCode::NotFound, "{name}: {}", err.message);
        }
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_missing_or_archived_discussion_is_not_found() {
        let store = TestStore::default();
        store.archived_discussions.borrow_mut().insert("old-talk".to_string(), RECORD.to_string());
        for slug in ["ghost", "old-talk"] {
            let err = run(&store, discuss_edit(slug, &[("a", "1")], &[])).expect_err(slug);
            assert_eq!(err.code, ErrorCode::NotFound, "{slug}: {}", err.message);
            assert!(err.message.contains("not found"), "{}", err.message);
        }
        assert_eq!(store.archived_discussions.borrow()["old-talk"], RECORD, "an archived record never changes");
    }

    // --- 討論 ---

    #[test]
    fn a_discussion_edit_writes_the_frontmatter_and_reports_one_event() {
        let store = TestStore::with_live_discussion("board-search", RECORD);
        let (outcome, events) = ok(&store, discuss_edit("board-search", &[("owner", "alice")], &[]));
        match outcome {
            CommandOutcome::DiscussFieldsEdit(o) => {
                assert_eq!(o.subject, "board-search");
                assert_eq!(o.fields, map(&[("owner", "alice")]));
                assert_eq!(o.changed, ["owner"]);
            }
            other => panic!("expected a fields outcome, got {other:?}"),
        }
        match events.as_slice() {
            [DomainEvent::DiscussionFieldsChanged { slug, keys, occurred_at: _ }] => {
                assert_eq!(slug, "board-search");
                assert_eq!(keys, &["owner"]);
            }
            other => panic!("expected one discussion-fields-changed event, got {other:?}"),
        }
        assert_eq!(events[0].kind(), "discussion-fields-changed");
        let (head, body) = RECORD.split_once("---\n\n").unwrap();
        assert_eq!(store.discussion("board-search"), format!("{head}fields:\n  owner: \"alice\"\n---\n\n{body}"));

        let (_, events) = ok(&store, discuss_edit("board-search", &[], &["ghost"]));
        assert!(events.is_empty());
    }

    // --- 建立時附帶 ---

    fn new_change(name: &str, fields: &[(&str, &str)]) -> Command {
        Command::NewChange {
            name: name.to_string(),
            description: None,
            schema: Some("spec-driven".to_string()),
            agent: None,
            from_discussion: None,
            last: false,
            fields: map(fields),
        }
    }

    #[test]
    fn new_change_writes_the_fields_in_the_document_it_creates() {
        // 規格「建立時附帶」。
        let store = TestStore::default();
        let (_, events) = ok(&store, new_change("demo", &[("ticket", "PROJ-1"), ("source", "portal")]));
        assert!(
            store.meta("demo").ends_with("fields:\n  source: \"portal\"\n  ticket: \"PROJ-1\"\n"),
            "{}",
            store.meta("demo")
        );
        assert_eq!(fields_of(&store, "demo"), map(&[("source", "portal"), ("ticket", "PROJ-1")]));
        // 建立的那一份文件就有欄位：之後沒有第二次寫入，事件也只有建立。
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert_eq!(events.iter().map(DomainEvent::kind).collect::<Vec<_>>(), ["change-created"]);

        // 沒有欄位時，建立的文件沒有區塊。
        ok(&store, new_change("plain", &[]));
        assert!(!store.meta("plain").contains("fields"), "{}", store.meta("plain"));
    }

    #[test]
    fn new_change_with_a_field_outside_the_rules_creates_nothing() {
        // 規格「建立時欄位不合規則」。
        let store = TestStore::default();
        let too_many: Vec<(String, String)> = (0..33).map(|i| (format!("k{i:02}"), "v".to_string())).collect();
        let too_many: Vec<(&str, &str)> = too_many.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        for fields in [&[("Bad", "1")][..], &[("a", "x\ny")][..], &too_many[..]] {
            let err = run(&store, new_change("demo", fields)).expect_err("refused");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
            assert!(!store.change_exists("demo"));
        }
    }

    #[test]
    fn discuss_new_writes_the_fields_in_the_record_it_creates() {
        let store = TestStore::default();
        let cmd = |fields: &[(&str, &str)]| Command::DiscussNew {
            topic: "Board search".to_string(),
            slug: Some("board-search".to_string()),
            kind: None,
            fields: map(fields),
        };
        let err = run(&store, cmd(&[("Bad", "1")])).expect_err("refused");
        assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
        assert!(!store.live_discussion_exists("board-search"));

        let (outcome, events) = ok(&store, cmd(&[("owner", "alice"), ("n", "123")]));
        let text = store.discussion("board-search");
        assert!(text.contains("\nfields:\n  n: \"123\"\n  owner: \"alice\"\n---\n"), "{text}");
        assert_eq!(crate::discuss::DiscussionHead::parse(&text).fields, map(&[("n", "123"), ("owner", "alice")]));
        assert_eq!(events.iter().map(DomainEvent::kind).collect::<Vec<_>>(), ["discussion-created"]);
        match outcome {
            CommandOutcome::DiscussNew(info) => assert_eq!(info.head.fields, map(&[("n", "123"), ("owner", "alice")])),
            other => panic!("expected a discussion, got {other:?}"),
        }
    }

    #[test]
    fn discuss_promote_writes_the_fields_on_the_change_it_creates() {
        let store = TestStore::with_live_discussion("board-search", RECORD);
        let cmd = |fields: &[(&str, &str)]| Command::DiscussPromote {
            slug: "board-search".to_string(),
            name: Some("demo".to_string()),
            last: false,
            fields: map(fields),
        };
        let err = run(&store, cmd(&[("Bad", "1")])).expect_err("refused");
        assert_eq!(err.code, ErrorCode::InvalidArgv, "{}", err.message);
        assert!(!store.change_exists("demo"));
        assert_eq!(store.discussion("board-search"), RECORD, "the record is not marked promoted");

        ok(&store, cmd(&[("source", "portal")]));
        assert_eq!(fields_of(&store, "demo"), map(&[("source", "portal")]));
        assert_eq!(*store.meta_writes.borrow(), 0, "the fields are in the document the change was created with");
        // 欄位屬於變更，不寫進討論記錄。
        assert!(crate::discuss::DiscussionHead::parse(&store.discussion("board-search")).fields.is_empty());
    }

    // --- 規格「既有的寫入動詞保留欄位」 ---

    #[test]
    fn every_existing_write_verb_keeps_the_fields_and_the_archive_carries_them() {
        let store = TestStore::with_meta("demo", META);
        store.metas.borrow_mut().insert("base".to_string(), META.to_string());
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 only task\n");
        let fields = map(&[("source", "portal"), ("ticket", "PROJ-1"), ("n", "123")]);
        ok(&store, change_edit("demo", &[("source", "portal"), ("ticket", "PROJ-1"), ("n", "123")], &[]));

        let scope = || vec![ReviewedScopeEntry { path: "src/lib.rs".into(), hash: crate::station::content_fingerprint("fn lib() {}\n") }];
        let round = || "**Scope**: src/lib.rs\n".to_string();
        let demo = || "demo".to_string();
        let steps = vec![
            // 排序只在同一個階段裡排，所以放在開工之前。
            Command::ChangeRank { name: demo(), anchor: "base".to_string(), position: Side::Before, force: true },
            Command::InProgressAdd { name: demo(), branch: Some("speclink/demo".to_string()) },
            Command::TaskDone { task_id: "1".to_string(), change: Some(demo()), touched_files: Some(vec![]), head_commit: None },
            Command::ChangeDepends { name: demo(), on: vec!["base".to_string()], remove: false },
            Command::ChangeDepends { name: demo(), on: vec!["base".to_string()], remove: true },
            Command::ChangeSchema { name: demo(), schema: "no-spec-change".to_string() },
            Command::ChangeSchema { name: demo(), schema: "spec-driven".to_string() },
            Command::ChangeLanded { name: demo(), commit: "0123456789ab".to_string(), branch: None },
            Command::ReviewAddRound { change: demo(), content: round() },
            Command::ReviewStamp { change: demo(), accept: false, tool: None, head_commit: None, scope: scope(), missing: vec![] },
            Command::VerifyAddRound { change: demo(), content: round() },
            Command::VerifyStamp { change: demo(), accept: false, tool: None, head_commit: None, scope: scope(), missing: vec![] },
        ];
        for cmd in steps {
            let name = cmd.name();
            ok(&store, cmd);
            assert_eq!(fields_of(&store, "demo"), fields, "after {name}");
        }
        assert!(*store.meta_writes.borrow() > 6, "the steps did rewrite the metadata");

        // 封存會拿章的指紋對工作樹上的檔案，所以這一步給一個真的有那個檔案的工作樹。
        let root = std::env::temp_dir().join("speclink-fields-keep-on-archive");
        std::fs::create_dir_all(root.join("src")).expect("mkdir");
        std::fs::write(root.join("src/lib.rs"), "fn lib() {}\n").expect("write the scope file");
        let with_tree =
            ExecutionContext { workspace: Some(Workspace { root, spec_dir_name: "openspec".to_string() }), ..Default::default() };
        let (outcome, _) = execute(
            &store,
            &with_tree,
            Command::Archive {
                change: Some(demo()),
                skip_specs: false,
                no_validate: false,
                mark_tasks_complete: false,
                carry_review: false,
                carry_verify: false,
            },
        )
        .unwrap_or_else(|e| panic!("archive: {}", e.message));
        let CommandOutcome::Archive(archived) = outcome else { panic!("expected an archive outcome") };
        let meta = store.read_archived_meta(&archived.dated_name).expect("the archived metadata");
        assert_eq!(ChangeMeta::from_text(Some(&meta)).expect("it parses").fields, fields);
    }

    // --- 規格「引擎的行為不因欄位改變」 ---

    #[test]
    fn fields_named_like_engine_keys_change_no_plan_validation_or_archive() {
        let build = |fields: &[(&str, &str)]| {
            let store = TestStore::with_meta("demo", META);
            store.metas.borrow_mut().insert("other".to_string(), META.to_string());
            store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
            store.put_artifact("demo", "proposal.md", "## Why\n\nBecause.\n");
            if !fields.is_empty() {
                ok(&store, change_edit("demo", fields, &[]));
            }
            store
        };
        let plain = build(&[]);
        let loaded = build(&[
            ("schema", "no-such-schema"),
            ("depends_on", "other"),
            ("board_rank", "zzz"),
            ("started_at", "2026-01-01"),
            ("reviewed_at", "2026-01-01"),
            ("from_discussion", "ghost"),
            ("archived_at", "never"),
        ]);
        let archive = || Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        };
        let validate = || Command::Validate { item: Some("demo".to_string()), all: false, changes: false, specs: false, strict: true };
        let status = || Command::Status { change: Some("demo".to_string()), schema: None };
        for cmd in [Command::Plan { ranks: None }, validate(), status(), archive()] {
            let name = cmd.name();
            let (plain_outcome, plain_events) = ok(&plain, cmd.clone());
            let (loaded_outcome, loaded_events) = ok(&loaded, cmd);
            assert_eq!(format!("{loaded_outcome:?}"), format!("{plain_outcome:?}"), "{name}");
            assert_eq!(
                loaded_events.iter().map(DomainEvent::kind).collect::<Vec<_>>(),
                plain_events.iter().map(DomainEvent::kind).collect::<Vec<_>>(),
                "{name}"
            );
        }
    }
    // --- D5：讀取的結果帶出欄位（規格「自由欄位的讀取輸出」） ---

    fn list_items(store: &TestStore) -> Vec<serde_json::Value> {
        let cmd = Command::List { sort: "name".to_string(), specs: false, changes: true, worktrees: Default::default() };
        let (outcome, events) = ok(store, cmd);
        assert!(events.is_empty());
        let CommandOutcome::List(list) = outcome else { panic!("expected a list outcome") };
        list.changes.expect("the changes section").iter().map(|item| serde_json::to_value(item).unwrap()).collect()
    }

    #[test]
    fn a_list_item_has_fields_only_when_the_change_has_some() {
        // 規格「清單帶出欄位」：demo 有兩個欄位，other 沒有。
        let store = TestStore::with_meta("demo", META);
        store.metas.borrow_mut().insert("other".to_string(), META.to_string());
        let before = list_items(&store);
        ok(&store, change_edit("demo", &[("ticket", "PROJ-1"), ("source", "portal")], &[]));
        let after = list_items(&store);
        assert_eq!(after[0]["name"], "demo");
        assert_eq!(after[0]["fields"], serde_json::json!({ "source": "portal", "ticket": "PROJ-1" }));
        assert!(after[1].get("fields").is_none(), "{}", after[1]);
        // 沒有欄位的項目，和這個能力之前一樣：沒有多出任何鍵。
        assert!(before.iter().all(|item| item.get("fields").is_none()));
        assert_eq!(after[1], before[1]);
    }

    #[test]
    fn show_of_a_change_carries_its_fields() {
        let store = TestStore::with_meta("demo", META);
        let show = |store: &TestStore| {
            let cmd = Command::Show { item: Some("demo".to_string()), item_type: Some("change".to_string()) };
            match ok(store, cmd).0 {
                CommandOutcome::Show(crate::command::ShowOutcome::Change(change)) => change,
                other => panic!("expected a change, got {other:?}"),
            }
        };
        assert!(show(&store).fields.is_empty());
        ok(&store, change_edit("demo", &[("ticket", "PROJ-1")], &[]));
        assert_eq!(show(&store).fields, map(&[("ticket", "PROJ-1")]));
    }

    #[test]
    fn discussion_info_has_fields_only_when_the_record_has_some() {
        // 規格「討論資訊帶出欄位」：board-search 有欄位 owner。
        let store = TestStore::with_live_discussion("board-search", RECORD);
        let infos = |store: &TestStore| {
            let CommandOutcome::DiscussList(list) = ok(store, Command::DiscussList { archived: false }).0 else {
                panic!("expected a discussion list")
            };
            let CommandOutcome::DiscussShow(show) = ok(store, Command::DiscussShow { slug: "board-search".to_string() }).0 else {
                panic!("expected a discussion")
            };
            let CommandOutcome::DiscussSearch(hits) = ok(store, Command::DiscussSearch { terms: vec!["board".to_string()] }).0
            else {
                panic!("expected search hits")
            };
            [list[0].clone(), show.info.expect("the info"), hits[0].info.clone()]
        };
        for info in infos(&store) {
            assert!(info.fields.is_empty());
            assert!(serde_json::to_value(&info).unwrap().get("fields").is_none(), "an empty map is not serialized");
        }
        ok(&store, discuss_edit("board-search", &[("owner", "alice")], &[]));
        for info in infos(&store) {
            assert_eq!(info.fields, map(&[("owner", "alice")]));
            assert_eq!(serde_json::to_value(&info).unwrap()["fields"], serde_json::json!({ "owner": "alice" }));
        }
    }

    #[test]
    fn discussion_info_reads_back_from_json_with_or_without_fields() {
        // remote 模式把線上的資料轉回這個型別；沒有 fields 鍵的舊資料照樣讀得進來。
        let store = TestStore::with_live_discussion("board-search", RECORD);
        let CommandOutcome::DiscussList(list) = ok(&store, Command::DiscussList { archived: false }).0 else {
            panic!("expected a discussion list")
        };
        let mut json = serde_json::to_value(&list[0]).unwrap();
        let plain: crate::discuss::DiscussionInfo = serde_json::from_value(json.clone()).unwrap();
        assert!(plain.fields.is_empty());
        json["fields"] = serde_json::json!({ "owner": "alice" });
        let with: crate::discuss::DiscussionInfo = serde_json::from_value(json).unwrap();
        assert_eq!(with.fields, map(&[("owner", "alice")]));
    }
}
