//! The engine commands' stable names and serializable params (spec command-runtime:
//! 引擎命令有穩定的名稱與可序列化的參數). One sample per `Command` variant; the
//! name/params pairs are pinned by `golden/command-describe.json`, so renaming a
//! command or one of its param keys turns this suite red.
//!
//! Regenerate the golden deliberately with:
//! UPDATE_GOLDEN=1 cargo test -p speclink-core --test it command_describe::

use speclink_core::command::Command;
use speclink_core::config::{ReviewList, SubagentStation, ToolName, WorkflowConfigEdit};
use speclink_core::listing::ListWorktreeJson;
use speclink_core::model::ReviewedScopeEntry;
use std::collections::BTreeMap;

/// Every variant's position, matched with no wildcard arm: a new `Command`
/// variant does not compile here until it has a sample below.
fn variant_index(cmd: &Command) -> usize {
    match cmd {
        Command::List { .. } => 0,
        Command::Show { .. } => 1,
        Command::Status { .. } => 2,
        Command::Instructions { .. } => 3,
        Command::Validate { .. } => 4,
        Command::Analyze { .. } => 5,
        Command::Trace { .. } => 6,
        Command::Plan { .. } => 7,
        Command::PlanStrict => 8,
        Command::ArtifactCat { .. } => 9,
        Command::EvidenceShow { .. } => 10,
        Command::LanguageShow => 11,
        Command::DiscussList { .. } => 12,
        Command::DiscussShow { .. } => 13,
        Command::DiscussSearch { .. } => 14,
        Command::NewChange { .. } => 15,
        Command::NewArtifact { .. } => 16,
        Command::TaskDone { .. } => 17,
        Command::TaskUndone { .. } => 18,
        Command::TaskMove { .. } => 19,
        Command::Claim { .. } => 20,
        Command::InProgressAdd { .. } => 21,
        Command::InProgressRemove { .. } => 22,
        Command::ChangeDepends { .. } => 23,
        Command::ChangeSchema { .. } => 24,
        Command::ChangeRank { .. } => 25,
        Command::Archive { .. } => 26,
        Command::Discard { .. } => 27,
        Command::DiscussNew { .. } => 28,
        Command::DiscussContext { .. } => 29,
        Command::DiscussAddRound { .. } => 30,
        Command::DiscussConclude { .. } => 31,
        Command::DiscussPromote { .. } => 32,
        Command::DiscussLink { .. } => 33,
        Command::DiscussSeal { .. } => 34,
        Command::DiscussArchive { .. } => 35,
        Command::DiscussDiscard { .. } => 36,
        Command::ReviewAddRound { .. } => 37,
        Command::ReviewShow { .. } => 38,
        Command::ReviewStamp { .. } => 39,
        Command::ReviewDiscard { .. } => 40,
        Command::VerifyAddRound { .. } => 41,
        Command::VerifyShow { .. } => 42,
        Command::VerifyStamp { .. } => 43,
        Command::VerifyDiscard { .. } => 44,
        Command::WorkflowConfigShow => 45,
        Command::WorkflowConfigEdit { .. } => 46,
        Command::WorkflowConfigReplace { .. } => 47,
    }
}

const VARIANT_COUNT: usize = 48;

fn s(v: &str) -> String {
    v.to_string()
}

/// One command per variant, with representative params.
fn samples() -> Vec<Command> {
    let scope = vec![ReviewedScopeEntry { path: s("src/lib.rs"), hash: s("ab12") }];
    vec![
        Command::List {
            sort: s("name"),
            specs: false,
            changes: true,
            worktrees: BTreeMap::from([(
                s("demo"),
                ListWorktreeJson { path: s("/work/repo.worktrees/demo"), branch: s("speclink/demo") },
            )]),
        },
        Command::Show { item: Some(s("demo")), item_type: Some(s("change")) },
        Command::Status { change: Some(s("demo")), schema: None },
        Command::Instructions { artifact: Some(s("apply")), change: Some(s("demo")), schema: None },
        Command::Validate { item: Some(s("demo")), all: false, changes: false, specs: false, strict: true },
        Command::Analyze { change: Some(s("demo")) },
        Command::Trace { capability: s("host-kit") },
        Command::Plan { ranks: Some(BTreeMap::from([(s("demo"), s("a0"))])) },
        Command::PlanStrict,
        Command::ArtifactCat { artifact: s("proposal"), change: Some(s("demo")) },
        Command::EvidenceShow { change: s("demo") },
        Command::LanguageShow,
        Command::DiscussList { archived: true },
        Command::DiscussShow { slug: s("topic") },
        Command::DiscussSearch { terms: vec![s("host"), s("kit")] },
        Command::NewChange {
            name: s("demo"),
            description: Some(s("A demo change")),
            schema: Some(s("spec-driven")),
            agent: Some(s("claude")),
            from_discussion: Some(s("topic")),
            last: true,
        },
        Command::NewArtifact {
            kind: s("specs"),
            capability: Some(s("host-kit")),
            change: Some(s("demo")),
            content: Some(s("## ADDED Requirements\n")),
            force: false,
            new_capability: true,
        },
        Command::TaskDone {
            task_id: s("1.1"),
            change: Some(s("demo")),
            touched_files: Some(vec![s("src/lib.rs")]),
            head_commit: Some(s("abc123")),
        },
        Command::TaskUndone { task_id: s("1.1"), change: Some(s("demo")) },
        Command::TaskMove { change: s("demo"), from: 1, to: 3, before: Some(true) },
        Command::Claim { name: s("demo") },
        Command::InProgressAdd { name: s("demo") },
        Command::InProgressRemove { name: s("demo") },
        Command::ChangeDepends { name: s("demo"), on: vec![s("base")], remove: false },
        Command::ChangeSchema { name: s("demo"), schema: s("spec-driven") },
        Command::ChangeRank {
            name: s("demo"),
            anchor: s("base"),
            position: speclink_core::plan::Side::After,
            force: false,
        },
        Command::Archive {
            change: Some(s("demo")),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: true,
            carry_verify: false,
        },
        Command::Discard { change: s("demo"), force: true },
        Command::DiscussNew { topic: s("Host kit"), slug: Some(s("host-kit")), kind: Some(s("feature")) },
        Command::DiscussContext { slug: s("topic"), content: s("context text") },
        Command::DiscussAddRound { slug: s("topic"), mode: s("explore"), content: s("round text") },
        Command::DiscussConclude { slug: s("topic"), content: s("conclusion"), hold: false },
        Command::DiscussPromote { slug: s("topic"), name: Some(s("demo")), last: false },
        Command::DiscussLink { slug: s("topic"), change: s("demo") },
        Command::DiscussSeal { slug: s("topic"), change: s("demo"), last: true },
        Command::DiscussArchive { slug: s("topic") },
        Command::DiscussDiscard { slug: s("topic"), force: false },
        Command::ReviewAddRound { change: s("demo"), content: s("{\"findings\":[]}") },
        Command::ReviewShow { change: s("demo") },
        Command::ReviewStamp {
            change: s("demo"),
            accept: false,
            tool: Some(s("claude")),
            scope: scope.clone(),
            missing: vec![s("src/gone.rs")],
        },
        Command::ReviewDiscard { change: s("demo") },
        Command::VerifyAddRound { change: s("demo"), content: s("{\"findings\":[]}") },
        Command::VerifyShow { change: s("demo") },
        Command::VerifyStamp {
            change: s("demo"),
            accept: true,
            tool: None,
            scope,
            missing: vec![],
        },
        Command::VerifyDiscard { change: s("demo") },
        Command::WorkflowConfigShow,
        Command::WorkflowConfigEdit {
            edit: WorkflowConfigEdit::Policy { key: s("tdd"), value: s("true") },
            dry_run: false,
        },
        Command::WorkflowConfigReplace { content: s("schema: spec-driven\n") },
    ]
}

/// The variant name as `Debug` prints it (`NewChange { .. }` → `NewChange`).
fn variant_name(cmd: &Command) -> String {
    format!("{cmd:?}").chars().take_while(|c| c.is_ascii_alphanumeric()).collect()
}

fn kebab(camel: &str) -> String {
    let mut out = String::new();
    for (i, c) in camel.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn is_camel_case(key: &str) -> bool {
    key.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && key.chars().all(|c| c.is_ascii_alphanumeric())
}

#[test]
fn every_variant_has_exactly_one_sample() {
    let mut seen: Vec<usize> = samples().iter().map(variant_index).collect();
    seen.sort_unstable();
    assert_eq!(seen, (0..VARIANT_COUNT).collect::<Vec<_>>());
}

#[test]
fn names_are_unique_kebab_case_of_the_variant() {
    let samples = samples();
    let mut names: Vec<&str> = samples.iter().map(Command::name).collect();
    for cmd in &samples {
        assert_eq!(cmd.name(), kebab(&variant_name(cmd)), "{cmd:?}");
    }
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), VARIANT_COUNT, "two commands share a name");
}

#[test]
fn the_name_depends_only_on_the_kind() {
    let plain = Command::NewChange {
        name: s("a"),
        description: None,
        schema: None,
        agent: None,
        from_discussion: None,
        last: false,
    };
    let with_schema = Command::NewChange {
        name: s("b"),
        description: None,
        schema: Some(s("spec-driven")),
        agent: None,
        from_discussion: None,
        last: false,
    };
    assert_eq!(plain.name(), "new-change");
    assert_eq!(with_schema.name(), "new-change");
}

#[test]
fn params_are_objects_with_camel_case_keys() {
    for cmd in samples() {
        let params = cmd.params();
        let object = params.as_object().unwrap_or_else(|| panic!("{} params not an object", cmd.name()));
        for key in object.keys() {
            assert!(is_camel_case(key), "{}: key '{key}' is not camelCase", cmd.name());
        }
    }
}

#[test]
fn a_command_without_fields_is_an_empty_object() {
    for cmd in [Command::LanguageShow, Command::PlanStrict, Command::WorkflowConfigShow] {
        assert_eq!(cmd.params(), serde_json::json!({}), "{}", cmd.name());
    }
}

#[test]
fn archive_params_carry_the_carry_review_flag() {
    let cmd = Command::Archive {
        change: Some(s("demo")),
        skip_specs: false,
        no_validate: false,
        mark_tasks_complete: false,
        carry_review: true,
        carry_verify: false,
    };
    let params = cmd.params();
    assert_eq!(params["change"], "demo");
    assert_eq!(params["carryReview"], true);
}

#[test]
fn names_and_params_match_the_golden() {
    let entries: Vec<serde_json::Value> = samples()
        .iter()
        .map(|cmd| serde_json::json!({ "name": cmd.name(), "params": cmd.params() }))
        .collect();
    let actual = serde_json::to_string_pretty(&entries).unwrap() + "\n";
    crate::render_golden::assert_matches_golden("command-describe.json", &actual);
}

/// The nested workflow-config edit keeps camelCase keys too.
#[test]
fn workflow_config_edit_params_use_camel_case_inside() {
    let cmd = Command::WorkflowConfigEdit {
        edit: WorkflowConfigEdit::SubagentCell {
            station: SubagentStation::Review,
            tool: ToolName::parse("claude").unwrap(),
            model: s("opus"),
        },
        dry_run: true,
    };
    let params = cmd.params();
    assert_eq!(params["dryRun"], true);
    assert_eq!(
        params["edit"],
        serde_json::json!({ "subagentCell": { "station": "review", "tool": "claude", "model": "opus" } })
    );
    let list = Command::WorkflowConfigEdit {
        edit: WorkflowConfigEdit::ReviewList { list: ReviewList::Critical, body: s("rule") },
        dry_run: false,
    };
    assert_eq!(
        list.params()["edit"],
        serde_json::json!({ "reviewList": { "list": "critical", "body": "rule" } })
    );
}
