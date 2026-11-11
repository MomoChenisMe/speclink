//! The workflow-config commands: read the document, apply one edit to it, or
//! replace it whole (workflow-config-commands D1).
//!
//! A write command reads the document once, computes the new content,
//! validates it, and writes at most once — so a failure leaves the store as it
//! was. What needs a checkout (the worktree teardown check, the skill sync, the
//! stale-skill hint) is the entry point's job, before and after the command.

use super::{classify, resolve_schema, CommandError, ErrorCode, ExecutionContext};
use crate::config::{
    changed_keys, update_workflow_config_text, validate_policy_locales, ChangedKey, ConfigError,
    ContentEdit, ContextEdit, PolicyKey, ReviewConfig, SubagentModelConfig, WorkflowConfig,
    WorkflowConfigEdit, WorkflowPolicyFields,
};
use crate::store::Store;
use crate::workspace::Workspace;
use std::collections::BTreeMap;
use std::path::Path;

/// `workflow-config show` outcome. The values are canonical — what the
/// document says — never the env-resolved policy.
#[derive(Debug)]
pub struct WorkflowConfigShowOutcome {
    /// Display path of the document (`<spec dir>/config.yaml`).
    pub file: String,
    pub config: WorkflowConfig,
    /// Whether the store has the document; a missing one reads as defaults.
    pub exists: bool,
}

/// Outcome of the two write commands.
#[derive(Debug)]
pub struct WorkflowConfigWriteOutcome {
    /// Display path of the document (`<spec dir>/config.yaml`).
    pub file: String,
    /// The document before the command; `None` when the store had none.
    pub before: Option<String>,
    /// The document after the command — also when nothing was written
    /// (a dry run, or content that did not change).
    pub after: String,
    /// `after`, parsed.
    pub config: WorkflowConfig,
    /// The keys whose value differs between `before` and `after`
    /// ([`changed_keys`]). Empty when only layout or comments changed.
    pub changed_keys: Vec<ChangedKey>,
    /// Whether the document was written.
    pub written: bool,
}

fn config_file(store: &dyn Store) -> String {
    format!("{}/config.yaml", store.spec_dir_display())
}

/// Parse a document, naming `file` in the error.
fn parse(text: Option<&str>, file: &str) -> Result<WorkflowConfig, ConfigError> {
    WorkflowConfig::from_text(text).map_err(|e| ConfigError { file: file.to_string(), ..e })
}

pub(super) fn show(store: &dyn Store) -> Result<WorkflowConfigShowOutcome, CommandError> {
    let file = config_file(store);
    let text = store.read_workflow_config();
    let config = parse(text.as_deref(), &file)?;
    Ok(WorkflowConfigShowOutcome { file, config, exists: text.is_some() })
}

pub(super) fn edit(
    store: &dyn Store,
    ctx: &ExecutionContext,
    edit: &WorkflowConfigEdit,
    dry_run: bool,
) -> Result<WorkflowConfigWriteOutcome, CommandError> {
    let plan = plan_workflow_config_edit(
        edit,
        &config_file(store),
        store.read_workflow_config(),
        ctx.workspace.as_ref(),
        ctx.user_config_dir.as_deref(),
    )?;
    write_unless_same(store, plan, dry_run)
}

pub(super) fn replace(
    store: &dyn Store,
    content: String,
) -> Result<WorkflowConfigWriteOutcome, CommandError> {
    let file = config_file(store);
    let before = store.read_workflow_config();
    // A broken document must stay replaceable — that is how it gets repaired.
    // Its keys count as unset, so every key the new content sets is reported.
    let current = parse(before.as_deref(), &file).unwrap_or_default();
    write_unless_same(store, planned(file, before, &current, content)?, false)
}

/// What one edit does to the document `before` (`None`: there is none): the
/// complete new document, validated, and the keys it changes. Nothing is
/// written — `written` is false. The edit command writes the result to its
/// store; an entry point that holds the document itself (the CLI's remote
/// arm) sends it where the document lives, and so states the same result.
///
/// `file` names the document in messages. A document that does not parse is
/// refused: a read-modify-write on it would destroy the user's content.
/// `ws` and `user_config_dir` locate the schema whose artifact ids `rules`
/// accepts (project → user → built-in).
pub fn plan_workflow_config_edit(
    edit: &WorkflowConfigEdit,
    file: &str,
    before: Option<String>,
    ws: Option<&Workspace>,
    user_config_dir: Option<&Path>,
) -> Result<WorkflowConfigWriteOutcome, CommandError> {
    let original = before.as_deref().unwrap_or_default();
    let current = WorkflowConfig::from_text(Some(original)).map_err(|e| {
        CommandError::new(
            ErrorCode::InvalidConfig,
            format!("invalid {file}: {} — write refused", e.reason),
        )
    })?;
    let after = rewrite(edit, original, &current, ws, user_config_dir)?;
    planned(file.to_string(), before, &current, after)
}

/// Validate the new content and name the keys it changes against `current`
/// (the parsed `before`).
fn planned(
    file: String,
    before: Option<String>,
    current: &WorkflowConfig,
    after: String,
) -> Result<WorkflowConfigWriteOutcome, CommandError> {
    let config = parse(Some(&after), &file)?;
    validate_policy_locales(&WorkflowPolicyFields {
        locale: config.locale.clone(),
        spec_locale: config.spec_locale.clone(),
        ..Default::default()
    })
    .map_err(invalid_config)?;
    let changed_keys = changed_keys(current, &config);
    Ok(WorkflowConfigWriteOutcome { file, before, after, config, changed_keys, written: false })
}

/// Write the planned document once — unless it is byte-for-byte what the
/// store has (a missing document equals an empty one), or this is a dry run.
fn write_unless_same(
    store: &dyn Store,
    mut plan: WorkflowConfigWriteOutcome,
    dry_run: bool,
) -> Result<WorkflowConfigWriteOutcome, CommandError> {
    let same = plan.before.as_deref().unwrap_or_default() == plan.after;
    if !same && !dry_run {
        store.write_workflow_config(&plan.after).map_err(|e| CommandError {
            message: format!("{}: write failed: {e}", plan.file),
            ..classify(e)
        })?;
        plan.written = true;
    }
    Ok(plan)
}

fn invalid_config(e: anyhow::Error) -> CommandError {
    CommandError::new(ErrorCode::InvalidConfig, e.to_string())
}

fn invalid_argv(message: String) -> CommandError {
    CommandError::new(ErrorCode::InvalidArgv, message)
}

/// Apply one edit to the document `original`, whose parsed form is `current`,
/// and return the complete new document.
fn rewrite(
    edit: &WorkflowConfigEdit,
    original: &str,
    current: &WorkflowConfig,
    ws: Option<&Workspace>,
    user_config_dir: Option<&Path>,
) -> Result<String, CommandError> {
    // The rewrite takes the COMPLETE target state of the policy keys, so the
    // current values are read back first and only the edited key moves.
    let mut fields = WorkflowPolicyFields {
        locale: current.locale.clone(),
        spec_locale: current.spec_locale.clone(),
        tdd: current.tdd.unwrap_or(false),
        audit: current.audit.unwrap_or(false),
        worktree: current.worktree.unwrap_or(false),
    };
    let mut context = ContextEdit::Keep;
    let mut rules: Option<Vec<(String, Vec<String>)>> = None;
    // review and subagent_model are handed over whole: the current block with
    // only the edited key moved.
    let mut review: Option<ReviewConfig> = None;
    let mut subagent_model: Option<SubagentModelConfig> = None;
    match edit {
        WorkflowConfigEdit::Policy { key, value } => set_policy_field(&mut fields, key, value)?,
        WorkflowConfigEdit::Context(text) => context = ContextEdit::Set(text.clone()),
        WorkflowConfigEdit::Rules { artifact, body } => {
            let artifacts = schema_artifacts(ws, user_config_dir, current)?;
            if !artifacts.iter().any(|id| id == artifact) {
                return Err(invalid_argv(format!(
                    "Unknown artifact '{artifact}' for schema '{}'. Use one of: {}",
                    current.schema_name(),
                    artifacts.join(", ")
                )));
            }
            rules = Some(merged_rules(&current.rules, &artifacts, artifact, list_entries(body)));
        }
        WorkflowConfigEdit::ReviewList { list, body } => {
            let mut target = current.review();
            *list.entries(&mut target) = list_entries(body);
            review = Some(target);
        }
        WorkflowConfigEdit::ReviewBaseline(baseline) => {
            let mut target = current.review();
            target.baseline = *baseline;
            review = Some(target);
        }
        WorkflowConfigEdit::SubagentCell { station, tool, model } => {
            let mut target = current.subagent_model();
            match model.trim() {
                "" => station.cells(&mut target).remove(tool.as_str()),
                model => station.cells(&mut target).insert(tool.clone(), model.to_string()),
            };
            subagent_model = moved(target, current);
        }
        WorkflowConfigEdit::SubagentChoices { tool, body } => {
            let mut target = current.subagent_model();
            match list_entries(body) {
                entries if entries.is_empty() => target.choices.remove(tool.as_str()),
                entries => target.choices.insert(tool.clone(), entries),
            };
            subagent_model = moved(target, current);
        }
    }
    validate_policy_locales(&fields).map_err(invalid_config)?;
    update_workflow_config_text(
        original,
        &fields,
        &ContentEdit {
            context,
            rules: rules.as_deref(),
            review: review.as_ref(),
            subagent_model: subagent_model.as_ref(),
        },
    )
    .map_err(classify)
}

/// A `subagent_model` edit reaches the rewrite only when it moves something: a
/// write that leaves the block as it is (a cell removed that was not set, the
/// same model again) keeps the block untouched, comments and hand-written
/// layout included.
fn moved(target: SubagentModelConfig, current: &WorkflowConfig) -> Option<SubagentModelConfig> {
    (target != current.subagent_model()).then_some(target)
}

/// A stdin body as list entries: one per line, trimmed, blank lines skipped.
fn list_entries(body: &str) -> Vec<String> {
    body.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

/// Map one `set <key> <value>` onto the complete-target-state fields. An empty
/// locale value and `false` both mean "back to default" — the rewrite then
/// removes the key, keeping unset-means-default intact.
fn set_policy_field(
    fields: &mut WorkflowPolicyFields,
    key: &str,
    value: &str,
) -> Result<(), CommandError> {
    let text = || {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    };
    let Some(policy) = PolicyKey::parse(key) else {
        return Err(invalid_argv(format!(
            "Unknown key '{key}'. Use one of: {}",
            PolicyKey::list()
        )));
    };
    match policy {
        PolicyKey::Locale => fields.locale = text(),
        PolicyKey::SpecLocale => fields.spec_locale = text(),
        PolicyKey::Tdd => fields.tdd = policy_bool(key, value)?,
        PolicyKey::Audit => fields.audit = policy_bool(key, value)?,
        PolicyKey::Worktree => fields.worktree = policy_bool(key, value)?,
    }
    Ok(())
}

/// Toggles accept only `true`/`false` — "1", "yes" and friends are refused
/// rather than guessed at.
fn policy_bool(key: &str, value: &str) -> Result<bool, CommandError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(invalid_argv(format!("Value for '{key}' must be true or false (got '{value}')"))),
    }
}

/// The active schema's artifact ids in display order — both the accepted keys
/// for `rules <artifact>` and the section order written back. A schema that
/// cannot be resolved (missing, or present but broken) is the not-found error.
fn schema_artifacts(
    ws: Option<&Workspace>,
    user_config_dir: Option<&Path>,
    cfg: &WorkflowConfig,
) -> Result<Vec<String>, CommandError> {
    let name = cfg.schema_name();
    let not_found = || CommandError::new(ErrorCode::NotFound, crate::schema::not_found_msg(&name));
    let schema = resolve_schema(ws, user_config_dir, &name).map_err(|_| not_found())?;
    let artifacts: Vec<String> =
        crate::status::display_order(&schema).into_iter().map(|a| a.id.clone()).collect();
    if artifacts.is_empty() {
        return Err(not_found());
    }
    Ok(artifacts)
}

/// The complete rules map the rewrite replaces wholesale: the target section
/// swapped in, every other section carried over. Ordered by the schema's
/// artifact display order (the layout the desktop settings page also writes),
/// with any section outside the schema appended after so nothing is dropped.
/// Empty sections are removed by the rewrite.
fn merged_rules(
    current: &BTreeMap<String, Vec<String>>,
    artifacts: &[String],
    target: &str,
    entries: Vec<String>,
) -> Vec<(String, Vec<String>)> {
    let mut keys: Vec<String> = artifacts.to_vec();
    for key in current.keys() {
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }
    keys.into_iter()
        .map(|key| {
            let section = if key == target {
                entries.clone()
            } else {
                current.get(&key).cloned().unwrap_or_default()
            };
            (key, section)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::command::{
        execute, Command, CommandError, CommandOutcome, ErrorCode, ExecutionContext,
        WorkflowConfigShowOutcome, WorkflowConfigWriteOutcome,
    };
    use crate::config::{
        ReviewBaseline, ReviewList, SubagentStation, ToolName, WorkflowConfigEdit,
    };
    use crate::model::Change;
    use crate::store::{DiscussionDoc, Store};
    use crate::teststore::TestStore;
    use std::path::PathBuf;

    const DOC: &str = "\
# team config
schema: spec-driven
locale: tw
tdd: false

context: |
  Line one.
rules:
  tasks:
  - Use checkbox format
";

    fn run(store: &dyn Store, cmd: Command) -> Result<CommandOutcome, CommandError> {
        let (outcome, events) = execute(store, &ExecutionContext::default(), cmd)?;
        assert!(events.is_empty(), "the workflow-config commands produce no domain event");
        Ok(outcome)
    }

    fn show(store: &dyn Store) -> Result<WorkflowConfigShowOutcome, CommandError> {
        Ok(WorkflowConfigShowOutcome::try_from(run(store, Command::WorkflowConfigShow)?)
            .expect("show carries the show outcome"))
    }

    fn write(store: &dyn Store, cmd: Command) -> Result<WorkflowConfigWriteOutcome, CommandError> {
        Ok(WorkflowConfigWriteOutcome::try_from(run(store, cmd)?).expect("a write carries the write outcome"))
    }

    fn edit(store: &dyn Store, edit: WorkflowConfigEdit) -> Result<WorkflowConfigWriteOutcome, CommandError> {
        write(store, Command::WorkflowConfigEdit { edit, dry_run: false })
    }

    fn policy(key: &str, value: &str) -> WorkflowConfigEdit {
        WorkflowConfigEdit::Policy { key: key.into(), value: value.into() }
    }

    fn tool(name: &str) -> ToolName {
        ToolName::parse(name).expect("a kebab-case tool name")
    }

    /// The changed keys by the names the spec lists.
    fn keys(done: &WorkflowConfigWriteOutcome) -> Vec<String> {
        done.changed_keys.iter().map(ToString::to_string).collect()
    }

    fn stored(store: &TestStore) -> Option<String> {
        store.workflow_config.borrow().clone()
    }

    fn writes(store: &TestStore) -> u32 {
        *store.workflow_config_writes.borrow()
    }

    // --- show ---

    #[test]
    fn show_returns_the_parsed_config() {
        let store = TestStore::with_workflow_config(DOC);
        let shown = show(&store).expect("show executes");
        assert_eq!(shown.config.locale.as_deref(), Some("tw"));
        assert_eq!(shown.config.tdd, Some(false));
        assert_eq!(shown.config.rules_for("tasks"), Some(vec!["Use checkbox format".to_string()]));
        assert!(shown.exists);
        assert_eq!(shown.file, "speclink/config.yaml");
        assert_eq!(writes(&store), 0);
    }

    #[test]
    fn show_of_a_missing_document_is_the_defaults() {
        let shown = show(&TestStore::default()).expect("show executes");
        assert!(!shown.exists);
        assert_eq!(shown.config.schema, None);
        assert_eq!(shown.config.schema_name(), "spec-driven");
    }

    #[test]
    fn show_of_a_broken_document_is_invalid_config() {
        let store = TestStore::with_workflow_config(": not yaml : [\n");
        let err = show(&store).expect_err("a broken document fails closed");
        assert_eq!(err.code, ErrorCode::InvalidConfig);
        assert!(err.message.starts_with("invalid speclink/config.yaml: "), "{}", err.message);
    }

    // --- edit ---

    #[test]
    fn set_reports_the_changed_key_and_leaves_the_other_lines_alone() {
        // Spec scenario「set 回報變動的鍵」。
        let store = TestStore::with_workflow_config(DOC);
        let done = edit(&store, policy("tdd", "true")).expect("the edit executes");
        assert_eq!(keys(&done), ["tdd"]);
        assert!(done.written);
        assert_eq!(done.file, "speclink/config.yaml");
        assert_eq!(done.before.as_deref(), Some(DOC));
        assert_eq!(done.after, DOC.replace("tdd: false\n", "tdd: true\n"));
        assert_eq!(stored(&store).as_deref(), Some(done.after.as_str()));
        assert_eq!(writes(&store), 1);
    }

    #[test]
    fn every_kind_of_edit_writes_once_and_names_its_key() {
        // Spec Example「各種編輯的變動鍵」（逐列），再加模型清單。
        let rows: Vec<(WorkflowConfigEdit, &str, &str)> = vec![
            (policy("locale", "ja"), "locale", "locale: ja\n"),
            (WorkflowConfigEdit::Context("New text.\n".into()), "context", "  New text.\n"),
            (
                WorkflowConfigEdit::Rules { artifact: "proposal".into(), body: "one\ntwo\n".into() },
                "rules.proposal",
                "  proposal:\n  - one\n  - two\n",
            ),
            (
                WorkflowConfigEdit::ReviewList { list: ReviewList::Critical, body: "one rule\n".into() },
                "review.critical",
                "review:\n  critical:\n  - one rule\n",
            ),
            (
                WorkflowConfigEdit::ReviewBaseline(ReviewBaseline::Replace),
                "review.baseline",
                "review:\n  baseline: replace\n",
            ),
            (
                WorkflowConfigEdit::SubagentCell {
                    station: SubagentStation::Review,
                    tool: tool("claude"),
                    model: "opus".into(),
                },
                "subagent_model.review",
                "subagent_model:\n  review:\n    claude: opus\n",
            ),
            (
                WorkflowConfigEdit::SubagentChoices { tool: tool("codex"), body: "a\nb\n".into() },
                "subagent_model.choices",
                "subagent_model:\n  choices:\n    codex:\n    - a\n    - b\n",
            ),
        ];
        for (an_edit, key, fragment) in rows {
            let store = TestStore::with_workflow_config(DOC);
            let done = edit(&store, an_edit.clone()).unwrap_or_else(|e| panic!("{an_edit:?}: {e}"));
            assert_eq!(keys(&done), [key], "{an_edit:?}");
            assert!(done.written, "{an_edit:?}");
            assert_eq!(done.before.as_deref(), Some(DOC), "{an_edit:?}");
            assert!(done.after.contains(fragment), "{an_edit:?}: {}", done.after);
            assert_eq!(stored(&store).as_deref(), Some(done.after.as_str()), "{an_edit:?}");
            assert_eq!(writes(&store), 1, "{an_edit:?}: one command writes at most once");
        }
    }

    #[test]
    fn an_edit_that_changes_nothing_does_not_write() {
        // Spec scenario「值沒有變時不寫入」。
        let doc = DOC.replace("tdd: false\n", "tdd: true\n");
        let store = TestStore::with_workflow_config(&doc);
        let done = edit(&store, policy("tdd", "true")).expect("the edit executes");
        assert_eq!(keys(&done), [""; 0]);
        assert!(!done.written);
        assert_eq!(done.after, doc);
        assert_eq!(writes(&store), 0);
    }

    #[test]
    fn a_dry_run_does_not_write_but_names_the_changed_keys() {
        let store = TestStore::with_workflow_config(DOC);
        let done = write(&store, Command::WorkflowConfigEdit { edit: policy("audit", "true"), dry_run: true })
            .expect("the dry run executes");
        assert_eq!(keys(&done), ["audit"]);
        assert!(!done.written);
        assert_eq!(done.before.as_deref(), Some(DOC));
        assert!(done.after.contains("audit: true\n"), "{}", done.after);
        assert_eq!(stored(&store).as_deref(), Some(DOC));
        assert_eq!(writes(&store), 0);
    }

    #[test]
    fn an_edit_of_a_missing_document_creates_it() {
        let store = TestStore::default();
        let done = edit(&store, policy("tdd", "true")).expect("the edit executes");
        assert_eq!(done.before, None);
        assert_eq!(done.after, "tdd: true\n");
        assert_eq!(keys(&done), ["tdd"]);
        assert!(done.written);
        assert_eq!(stored(&store).as_deref(), Some("tdd: true\n"));
        // 沒有內容可寫時不建立空檔。
        let store = TestStore::default();
        let done = edit(&store, policy("tdd", "false")).expect("the edit executes");
        assert!(!done.written);
        assert_eq!(stored(&store), None);
    }

    #[test]
    fn an_edit_is_refused_before_any_write() {
        let bad_doc = ": not yaml : [\n";
        let rows: Vec<(&str, WorkflowConfigEdit, ErrorCode, &str)> = vec![
            (DOC, policy("nope", "1"), ErrorCode::InvalidArgv, "Unknown key 'nope'. Use one of: locale, spec_locale, tdd, audit, worktree"),
            (DOC, policy("tdd", "yes"), ErrorCode::InvalidArgv, "Value for 'tdd' must be true or false (got 'yes')"),
            (DOC, policy("locale", "fr"), ErrorCode::InvalidConfig, "Value for 'locale' must be one of tw, ja, en (got 'fr')"),
            (
                DOC,
                WorkflowConfigEdit::Rules { artifact: "nope".into(), body: "x\n".into() },
                ErrorCode::InvalidArgv,
                "Unknown artifact 'nope' for schema 'spec-driven'. Use one of: proposal, design, specs, tasks",
            ),
            (
                "schema: no-such-schema\n",
                WorkflowConfigEdit::Rules { artifact: "proposal".into(), body: "x\n".into() },
                ErrorCode::NotFound,
                "Schema not found: Schema 'no-such-schema' not found in project, user, or built-in locations",
            ),
            (bad_doc, policy("tdd", "true"), ErrorCode::InvalidConfig, "invalid speclink/config.yaml: "),
        ];
        for (doc, an_edit, code, message) in rows {
            let store = TestStore::with_workflow_config(doc);
            let err = edit(&store, an_edit.clone()).expect_err("the edit is refused");
            assert_eq!(err.code, code, "{an_edit:?}: {}", err.message);
            assert!(err.message.starts_with(message), "{an_edit:?}: {}", err.message);
            assert_eq!(stored(&store).as_deref(), Some(doc), "{an_edit:?}: the store is unchanged");
            assert_eq!(writes(&store), 0, "{an_edit:?}");
        }
        let store = TestStore::with_workflow_config(bad_doc);
        let err = edit(&store, policy("tdd", "true")).expect_err("a broken document is not rewritten");
        assert!(err.message.ends_with(" — write refused"), "{}", err.message);
    }

    // --- replace ---

    fn replace(store: &dyn Store, content: &str) -> Result<WorkflowConfigWriteOutcome, CommandError> {
        write(store, Command::WorkflowConfigReplace { content: content.to_string() })
    }

    #[test]
    fn replace_writes_the_given_document_once() {
        let store = TestStore::with_workflow_config(DOC);
        let next = "schema: spec-driven\nlocale: ja\naudit: true\n";
        let done = replace(&store, next).expect("the replace executes");
        assert_eq!(done.before.as_deref(), Some(DOC));
        assert_eq!(done.after, next);
        assert_eq!(keys(&done), ["locale", "audit", "context", "rules.tasks"]);
        assert!(done.written);
        assert_eq!(stored(&store).as_deref(), Some(next));
        assert_eq!(writes(&store), 1);
    }

    #[test]
    fn replace_refuses_invalid_content_and_leaves_the_store_alone() {
        // Spec scenario「不合法的內容在寫入前被拒絕」。
        for (content, needle) in [
            ("schema: [unterminated", "invalid speclink/config.yaml: "),
            ("schema: spec-driven\nlocale: 繁體中文\n", "Value for 'locale' must be one of tw, ja, en"),
            ("schema: spec-driven\nspec_locale: zh-Hant\n", "Value for 'spec_locale' must be one of tw, ja, en, auto"),
            ("review:\n  critcal:\n  - x\n", "critcal"),
        ] {
            let store = TestStore::with_workflow_config(DOC);
            let err = replace(&store, content).expect_err("invalid content is refused");
            assert_eq!(err.code, ErrorCode::InvalidConfig, "{content:?}: {}", err.message);
            assert!(err.message.contains(needle), "{content:?}: {}", err.message);
            assert_eq!(stored(&store).as_deref(), Some(DOC), "{content:?}");
            assert_eq!(writes(&store), 0, "{content:?}");
        }
    }

    #[test]
    fn replace_repairs_a_document_that_does_not_parse() {
        // design D1：現有內容無法解析時整份取代照常執行；舊內容視為沒有任何設定，
        // 新內容設定的每個鍵都列進 changed_keys。編輯命令對同一份文件則拒絕。
        let broken = ": not yaml : [\n";
        let store = TestStore::with_workflow_config(broken);
        let next = "schema: spec-driven\nlocale: ja\ntdd: true\n";
        let done = replace(&store, next).expect("a broken document stays replaceable");
        assert!(done.written);
        assert_eq!(done.before.as_deref(), Some(broken));
        assert_eq!(keys(&done), ["schema", "locale", "tdd"]);
        assert_eq!(stored(&store).as_deref(), Some(next));
        assert_eq!(writes(&store), 1);
    }

    #[test]
    fn replace_with_the_same_bytes_does_not_write() {
        let store = TestStore::with_workflow_config(DOC);
        let done = replace(&store, DOC).expect("the replace executes");
        assert!(!done.written);
        assert_eq!(keys(&done), [""; 0]);
        assert_eq!(writes(&store), 0);
    }

    #[test]
    fn replace_that_only_changes_comments_writes_with_no_changed_key() {
        let store = TestStore::with_workflow_config(DOC);
        let next = DOC.replace("# team config\n", "# our config\n");
        let done = replace(&store, &next).expect("the replace executes");
        assert!(done.written);
        assert_eq!(keys(&done), [""; 0]);
        assert_eq!(stored(&store).as_deref(), Some(next.as_str()));
    }

    // --- a store without the write method ---

    /// A store that only answers the workflow-config read: every other method
    /// is one the three commands must never call, and the write method is the
    /// interface's default.
    struct ReadOnly(Option<String>);

    #[rustfmt::skip]
    impl Store for ReadOnly {
        fn read_workflow_config(&self) -> Option<String> { self.0.clone() }
        fn list_changes(&self) -> Vec<Change> { unreachable!() }
        fn find_change(&self, _: &str) -> Option<Change> { unreachable!() }
        fn change_exists(&self, _: &str) -> bool { unreachable!() }
        fn create_change(&self, _: &str, _: &str) -> anyhow::Result<PathBuf> { unreachable!() }
        fn updated_at_secs(&self, _: &str) -> u64 { unreachable!() }
        fn read_change_meta(&self, _: &str) -> Option<String> { unreachable!() }
        fn write_change_meta(&self, _: &str, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn delete_change(&self, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn read_artifact(&self, _: &str, _: &str) -> Option<String> { unreachable!() }
        fn write_artifact(&self, _: &str, _: &str, _: &str) -> anyhow::Result<PathBuf> { unreachable!() }
        fn artifact_exists(&self, _: &str, _: &str) -> bool { unreachable!() }
        fn read_evidence(&self, _: &str) -> Option<String> { unreachable!() }
        fn write_evidence(&self, _: &str, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn delta_capabilities(&self, _: &str) -> Vec<String> { unreachable!() }
        fn has_capability_dirs(&self, _: &str) -> bool { unreachable!() }
        fn list_canonical_capabilities(&self) -> Vec<String> { unreachable!() }
        fn canonical_spec_exists(&self, _: &str) -> bool { unreachable!() }
        fn read_canonical_spec(&self, _: &str) -> Option<String> { unreachable!() }
        fn write_canonical_spec(&self, _: &str, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn canonical_spec_path(&self, _: &str) -> PathBuf { unreachable!() }
        fn archived_change_exists(&self, _: &str) -> bool { unreachable!() }
        fn archive_change(&self, _: &str, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn read_archived_meta(&self, _: &str) -> Option<String> { unreachable!() }
        fn write_archived_meta(&self, _: &str, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn live_discussion_exists(&self, _: &str) -> bool { unreachable!() }
        fn archived_discussion_exists(&self, _: &str) -> bool { unreachable!() }
        fn live_discussion_path(&self, _: &str) -> PathBuf { unreachable!() }
        fn read_live_discussion(&self, _: &str) -> Option<String> { unreachable!() }
        fn write_live_discussion(&self, _: &str, _: &str) -> anyhow::Result<PathBuf> { unreachable!() }
        fn delete_live_discussion(&self, _: &str) -> anyhow::Result<()> { unreachable!() }
        fn read_discussion(&self, _: &str) -> Option<DiscussionDoc> { unreachable!() }
        fn list_live_discussions(&self) -> Vec<DiscussionDoc> { unreachable!() }
        fn list_archived_discussions(&self) -> Vec<DiscussionDoc> { unreachable!() }
        fn archive_discussion(&self, _: &str, _: &str) -> anyhow::Result<Option<String>> { unreachable!() }
        fn read_language(&self) -> Option<String> { unreachable!() }
    }

    #[test]
    fn a_store_without_the_write_method_reports_that_it_cannot_write() {
        let store = ReadOnly(Some(DOC.to_string()));
        for cmd in [
            Command::WorkflowConfigEdit { edit: policy("tdd", "true"), dry_run: false },
            Command::WorkflowConfigReplace { content: "schema: spec-driven\n".into() },
        ] {
            let err = write(&store, cmd.clone()).expect_err("the default write refuses");
            assert_eq!(err.code, ErrorCode::Error, "{cmd:?}");
            assert!(
                err.message.contains("does not support writing the workflow config"),
                "{cmd:?}: {}",
                err.message
            );
            assert!(err.message.contains("ReadOnly"), "the message names the store: {}", err.message);
        }
        // 讀取、dry-run 與沒有變動的寫入不需要寫入方法。
        show(&store).expect("show needs no write method");
        let dry = write(&store, Command::WorkflowConfigEdit { edit: policy("tdd", "true"), dry_run: true })
            .expect("a dry run needs no write method");
        assert_eq!(keys(&dry), ["tdd"]);
        write(&store, Command::WorkflowConfigReplace { content: DOC.into() })
            .expect("an unchanged document needs no write method");
    }
}
