//! Typed command runtime — the single execution layer for every Store-touching
//! domain verb (design engine-typed-core 決策一/二). [`execute`] is the one entry
//! point: a typed [`Command`] in, a typed [`CommandOutcome`] plus domain events
//! out, typed [`CommandError`]s with stable codes on failure. Orchestration only
//! (change resolution, schema resolution, event construction) — flow logic stays
//! in the existing core modules; the runtime is the front door, not a rewrite.
//!
//! Workspace bootstrap and peripheral tool verbs (init, update, config, schema
//! tools, completion, templates, feedback, demo) and remote connection
//! management (link, unlink, auth) intentionally do NOT appear in [`Command`].

use crate::config::ConfigError;
use crate::model::Change;
use crate::schema::Schema;
use crate::store::Store;
use crate::workspace::Workspace;

/// Stable error-code registry (design 決策三). The string values are the wire
/// contract shared by every entry point (Node envelope codes, future HTTP
/// mapping) — they never change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Arguments are invalid or ambiguous (bad flag combination, multiple
    /// changes matched an auto-detect).
    InvalidArgv,
    /// The addressed subject (change, spec, discussion, artifact) does not exist.
    NotFound,
    /// A config file exists but cannot be parsed (fail-closed, never defaults).
    InvalidConfig,
    /// A precondition refused the command — retry with --force or complete the
    /// prerequisite first.
    Refused,
    /// Every other failure.
    Error,
}

impl ErrorCode {
    /// The stable registry string for this code.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidArgv => "invalid_argv",
            ErrorCode::NotFound => "not_found",
            ErrorCode::InvalidConfig => "invalid_config",
            ErrorCode::Refused => "refused",
            ErrorCode::Error => "error",
        }
    }
}

/// A typed command failure: `code` classifies it for programmatic handling,
/// `message` is the semantic text — the SAME string the CLI prints (frozen by
/// the regression baseline), so hosts can hand it to users or agents verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
}

impl CommandError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> CommandError {
        CommandError {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandError {}

impl From<ConfigError> for CommandError {
    fn from(e: ConfigError) -> CommandError {
        CommandError::new(ErrorCode::InvalidConfig, e.to_string())
    }
}

/// Domain events reported by mutating verbs (design 決策四). Experimental
/// contract: payloads may change incompatibly until event persistence lands.
/// Variants arrive with the mutating-verb execution (階段 3); queries never
/// produce events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {}

/// The closed verb set of the command runtime, grouped per the 決策二 coverage
/// table. Inputs mirror the CLI argv vocabulary one-to-one; rendering concerns
/// (`--json`, color) stay in the entry points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // --- 查詢群 ---
    /// `list [--specs] [--changes] [--sort <key>]`
    List {
        sort: String,
        specs: bool,
        changes: bool,
    },
    /// `show <item> [--item-type change|spec]`
    Show {
        item: Option<String>,
        item_type: Option<String>,
    },
    /// `status [--change <name>] [--schema <name>]`
    Status {
        change: Option<String>,
        schema: Option<String>,
    },
    /// `instructions [artifact|apply] [--change <name>] [--schema <name>]`
    /// (`--skill` never reaches the runtime: skill bodies live outside the Store)
    Instructions {
        artifact: Option<String>,
        change: Option<String>,
        schema: Option<String>,
    },
    /// `validate [item] [--all] [--changes] [--strict]`. The CLI's `--specs`
    /// flag is accepted-and-ignored there (Spectra parity) and deliberately has
    /// no field here — carrying a dead input would read as if it selected
    /// spec validation.
    Validate {
        item: Option<String>,
        all: bool,
        changes: bool,
        strict: bool,
    },
    /// `analyze [change]`
    Analyze { change: Option<String> },
    /// `drift [change]`
    Drift { change: Option<String> },
    /// `artifact cat <artifact> [--change <name>]`
    ArtifactCat {
        artifact: String,
        change: Option<String>,
    },
    /// `language show`
    LanguageShow,
    /// `discuss list [--archived]`
    DiscussList { archived: bool },
    /// `discuss show <slug>`
    DiscussShow { slug: String },
}

/// `list` outcome: the changes section (sorted per the requested key, absent
/// for `--specs` alone) and the specs section (present when specs were
/// requested; the `list --specs --json` item shape, human rendering reads `id`).
#[derive(Debug)]
pub struct ListOutcome {
    pub changes: Option<Vec<crate::listing::ListChangeJson>>,
    pub specs: Option<serde_json::Value>,
}

/// `show` outcome — the item resolved to a canonical spec or a change.
#[derive(Debug)]
pub enum ShowOutcome {
    Spec { name: String, content: String },
    Change(ShowChange),
}

/// `show <change>` payload. `schema`/`created` echo the metadata verbatim as
/// one unit: unless BOTH are present, neither is reported (matches Spectra).
#[derive(Debug)]
pub struct ShowChange {
    pub name: String,
    pub schema: Option<String>,
    pub created: Option<String>,
    pub proposal: Option<String>,
    pub design: Option<String>,
    pub tasks: Option<String>,
    /// Delta capability names (renderers append "/spec.md").
    pub delta_capabilities: Vec<String>,
    pub from_discussions: Vec<String>,
    pub restale_from: Vec<String>,
}

/// `instructions` outcome: the apply view or one artifact's instructions.
#[derive(Debug)]
pub enum InstructionsOutcome {
    Apply(crate::instructions::ApplyInstructions),
    Artifact(crate::instructions::ArtifactInstructions),
}

/// `validate` outcome. Overall validity derives from the results — no
/// duplicated flag.
#[derive(Debug)]
pub struct ValidateOutcome {
    pub results: Vec<crate::validate::ValidationResult>,
}

/// `discuss show` outcome: the raw document plus its parsed info header.
#[derive(Debug)]
pub struct DiscussShowOutcome {
    pub info: Option<crate::discuss::DiscussionInfo>,
    pub content: String,
}

/// Typed result of one command execution.
#[derive(Debug)]
pub enum CommandOutcome {
    List(ListOutcome),
    Show(ShowOutcome),
    Status(crate::status::StatusReport),
    Instructions(InstructionsOutcome),
    Validate(ValidateOutcome),
    Analyze(crate::analyzer::AnalyzeReport),
    Drift(crate::drift::DriftReport),
    /// Raw artifact content (`artifact cat`).
    ArtifactCat(String),
    /// Raw LANGUAGE document content (`language show`).
    Language(String),
    DiscussList(Vec<crate::discuss::DiscussionInfo>),
    DiscussShow(DiscussShowOutcome),
}

/// Execute one command against the store. `ws` is the host workspace when the
/// entry point has one (CLI); hosts without a local workspace (Node SDK) pass
/// `None` and get a synthetic workspace for the flows that only use it for
/// host-side lookups. Returns the typed outcome plus the domain events the
/// execution produced (always empty for queries).
pub fn execute(
    store: &dyn Store,
    ws: Option<&Workspace>,
    cmd: Command,
) -> Result<(CommandOutcome, Vec<DomainEvent>), CommandError> {
    let outcome = match cmd {
        Command::List { sort, specs, changes } => run_list(store, &sort, specs, changes),
        Command::Show { item, item_type } => run_show(store, item.as_deref(), item_type.as_deref()),
        Command::Status { change, schema } => {
            run_status(store, ws, change.as_deref(), schema.as_deref())
        }
        Command::Instructions { artifact, change, schema } => {
            run_instructions(store, ws, artifact.as_deref(), change.as_deref(), schema.as_deref())
        }
        Command::Validate { item, all, changes, strict } => {
            run_validate(store, item.as_deref(), all, changes, strict)
        }
        Command::Analyze { change } => run_analyze(store, change.as_deref()),
        Command::Drift { change } => run_drift(store, ws, change.as_deref()),
        Command::ArtifactCat { artifact, change } => {
            run_artifact_cat(store, &artifact, change.as_deref())
        }
        Command::LanguageShow => run_language_show(store),
        Command::DiscussList { archived } => Ok(CommandOutcome::DiscussList(if archived {
            crate::discuss::list_archived(store)
        } else {
            crate::discuss::list_discussions(store)
        })),
        Command::DiscussShow { slug } => run_discuss_show(store, &slug),
    }?;
    // Queries carry no events; mutating verbs (階段 3) construct theirs from the
    // typed outcome right here — the single emission point.
    Ok((outcome, Vec::new()))
}

/// Specify-wording of the multi-change auto-detect error: flag-style verbs.
const SPECIFY_FLAG: &str = "Use --change to specify one:";
/// Positional-style verbs (analyze, drift) say just this (matches Spectra).
const SPECIFY_POSITIONAL: &str = "Specify one:";

/// Resolve a change by name, or auto-detect when no name is given (exactly one
/// active change). Message strings are the frozen CLI texts.
fn resolve_change(
    store: &dyn Store,
    name: Option<&str>,
    specify: &str,
) -> Result<Change, CommandError> {
    if let Some(n) = name {
        return crate::model::find_change(store, n).ok_or_else(|| {
            CommandError::new(ErrorCode::NotFound, format!("Change '{n}' not found."))
        });
    }
    let mut changes = crate::model::list_changes(store);
    match changes.len() {
        0 => Err(CommandError::new(
            ErrorCode::NotFound,
            "No active changes. Create one with: speclink new change <name>",
        )),
        1 => Ok(changes.remove(0)),
        _ => {
            crate::listing::sort_changes(store, &mut changes, "modified");
            let names: Vec<&str> = changes.iter().map(|c| c.name.as_str()).collect();
            Err(CommandError::new(
                ErrorCode::InvalidArgv,
                format!("Multiple changes found. {specify} {}", names.join(", ")),
            ))
        }
    }
}

/// Resolve a schema by name (project → user → built-in), frozen CLI messages.
fn resolve_schema(ws: Option<&Workspace>, name: &str) -> Result<Schema, CommandError> {
    match crate::schema::resolve_with(ws, name) {
        Some(Ok(s)) => Ok(s),
        Some(Err(e)) => Err(CommandError::new(ErrorCode::Error, e)),
        None => Err(CommandError::new(
            ErrorCode::NotFound,
            crate::schema::not_found_msg(name),
        )),
    }
}

/// The host workspace, or the synthetic no-host one (empty root — matches the
/// Node SDK's existing synthetic workspace). CAUTION: an empty root makes
/// host-side paths RELATIVE TO THE PROCESS CWD, so flows that read host files
/// through it (instructions' policy lookup, drift's git calls) are only ever
/// dispatched by entry points that hold a real workspace (the CLI).
fn host_workspace(ws: Option<&Workspace>) -> Workspace {
    ws.cloned().unwrap_or(Workspace {
        root: std::path::PathBuf::new(),
        spec_dir_name: "openspec".to_string(),
    })
}

fn run_list(
    store: &dyn Store,
    sort: &str,
    specs: bool,
    changes_flag: bool,
) -> Result<CommandOutcome, CommandError> {
    // --specs alone omits the changes section; combined with --changes both appear.
    let changes = if specs && !changes_flag {
        None
    } else {
        let mut changes = crate::model::list_changes(store);
        crate::listing::sort_changes(store, &mut changes, sort);
        Some(crate::listing::changes_json(store, &changes))
    };
    let specs = specs.then(|| crate::listing::specs_json_items(store));
    Ok(CommandOutcome::List(ListOutcome { changes, specs }))
}

fn run_show(
    store: &dyn Store,
    item: Option<&str>,
    item_type: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let Some(item) = item else {
        return Err(CommandError::new(
            ErrorCode::InvalidArgv,
            "Please specify an item name.",
        ));
    };
    if let Some(t) = item_type {
        if t != "change" && t != "spec" {
            return Err(CommandError::new(
                ErrorCode::InvalidArgv,
                format!("Unknown type: {t}. Use 'change' or 'spec'."),
            ));
        }
    }
    let is_spec = store.canonical_spec_exists(item);
    let change = crate::model::find_change(store, item);
    let show_spec =
        item_type == Some("spec") || (item_type != Some("change") && change.is_none() && is_spec);
    if show_spec {
        if !is_spec {
            if item_type == Some("spec") {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    format!("Spec '{item}' not found."),
                ));
            }
            return Err(CommandError::new(
                ErrorCode::NotFound,
                format!("Item '{item}' not found as a change or spec."),
            ));
        }
        let content = store.read_canonical_spec(item).unwrap_or_default();
        return Ok(CommandOutcome::Show(ShowOutcome::Spec {
            name: item.to_string(),
            content,
        }));
    }
    let Some(change) = change else {
        if item_type == Some("change") {
            return Err(CommandError::new(
                ErrorCode::NotFound,
                format!("Change '{item}' not found."),
            ));
        }
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("Item '{item}' not found as a change or spec."),
        ));
    };
    // The metadata pair reports as one unit: both present or neither.
    let (schema, created) = match (&change.meta.schema, &change.meta.created) {
        (Some(s), Some(c)) => (Some(s.clone()), Some(c.clone())),
        _ => (None, None),
    };
    Ok(CommandOutcome::Show(ShowOutcome::Change(ShowChange {
        schema,
        created,
        proposal: store.read_artifact(&change.name, "proposal.md"),
        design: store.read_artifact(&change.name, "design.md"),
        tasks: store.read_artifact(&change.name, "tasks.md"),
        delta_capabilities: store.delta_capabilities(&change.name),
        from_discussions: change.meta.from_discussions(),
        restale_from: change.meta.restale_from(),
        name: change.name,
    })))
}

fn run_status(
    store: &dyn Store,
    ws: Option<&Workspace>,
    change: Option<&str>,
    schema: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    let schema_name = match schema {
        Some(s) => s.to_string(),
        None => change.meta.schema_name(),
    };
    let schema = resolve_schema(ws, &schema_name)?;
    Ok(CommandOutcome::Status(crate::status::build(
        store, &change, &schema,
    )))
}

fn run_instructions(
    store: &dyn Store,
    ws: Option<&Workspace>,
    artifact: Option<&str>,
    change: Option<&str>,
    schema: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    let schema = match schema {
        Some(s) => resolve_schema(ws, s)?,
        None => resolve_schema(ws, &change.meta.schema_name())?,
    };
    // No-arg default: the first incomplete artifact, or the apply view once
    // every artifact exists (matches Spectra).
    let default_artifact = crate::status::first_incomplete_artifact(store, &change, &schema)
        .unwrap_or_else(|| "apply".to_string());
    let artifact = artifact.unwrap_or(&default_artifact);
    let host = host_workspace(ws);
    if artifact == "apply" {
        let payload = crate::instructions::build_apply(&host, store, &change, &schema)?;
        return Ok(CommandOutcome::Instructions(InstructionsOutcome::Apply(payload)));
    }
    let payload = crate::instructions::build_artifact(&host, store, &change, &schema, artifact)?
        .ok_or_else(|| {
            CommandError::new(
                ErrorCode::NotFound,
                format!("Artifact '{artifact}' not found in schema"),
            )
        })?;
    Ok(CommandOutcome::Instructions(InstructionsOutcome::Artifact(payload)))
}

fn run_validate(
    store: &dyn Store,
    item: Option<&str>,
    all: bool,
    changes_flag: bool,
    strict: bool,
) -> Result<CommandOutcome, CommandError> {
    let mut changes = if let Some(item) = item {
        vec![crate::model::find_change(store, item).ok_or_else(|| {
            CommandError::new(ErrorCode::NotFound, format!("Change '{item}' not found."))
        })?]
    } else if all || changes_flag {
        crate::model::list_changes(store)
    } else {
        // No item: exactly one change validates alone; zero or several fall
        // back to validating everything (matches Spectra).
        match resolve_change(store, None, SPECIFY_FLAG) {
            Ok(c) => vec![c],
            Err(_) => crate::model::list_changes(store),
        }
    };
    // Multi-change runs are ordered newest-modified first (matches Spectra).
    crate::listing::sort_changes(store, &mut changes, "modified");
    // Spectra's validate never resolves the change's schema (an unresolvable
    // one still validates).
    let schema = crate::schema::spec_driven();
    let results = changes
        .iter()
        .map(|c| crate::validate::validate_change(store, c, &schema, strict))
        .collect();
    Ok(CommandOutcome::Validate(ValidateOutcome { results }))
}

fn run_analyze(store: &dyn Store, change: Option<&str>) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_POSITIONAL)?;
    // Spectra's analyzer is schema-agnostic and never resolves the change's schema.
    let schema = crate::schema::spec_driven();
    Ok(CommandOutcome::Analyze(crate::analyzer::analyze(
        store, &change, &schema,
    )))
}

fn run_drift(
    store: &dyn Store,
    ws: Option<&Workspace>,
    change: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_POSITIONAL)?;
    let host = host_workspace(ws);
    Ok(CommandOutcome::Drift(crate::drift::analyze(
        &host, store, &change,
    )))
}

/// Artifact id → change-relative path (the `artifact cat` vocabulary).
fn artifact_rel_path(artifact: &str) -> Result<String, CommandError> {
    match artifact {
        "proposal" => Ok("proposal.md".to_string()),
        "design" => Ok("design.md".to_string()),
        "tasks" => Ok("tasks.md".to_string()),
        _ => match artifact.strip_prefix("specs/") {
            Some(cap) if !cap.is_empty() && !cap.contains('/') => {
                Ok(format!("specs/{cap}/spec.md"))
            }
            _ => Err(CommandError::new(
                ErrorCode::InvalidArgv,
                format!(
                    "Unknown artifact '{artifact}'. Use proposal, design, tasks, or specs/<capability>"
                ),
            )),
        },
    }
}

fn run_artifact_cat(
    store: &dyn Store,
    artifact: &str,
    change: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    let rel = artifact_rel_path(artifact)?;
    match store.read_artifact(&change.name, &rel) {
        Some(content) => Ok(CommandOutcome::ArtifactCat(content)),
        None => Err(CommandError::new(
            ErrorCode::NotFound,
            format!("artifact '{artifact}' not found for change '{}'", change.name),
        )),
    }
}

fn run_language_show(store: &dyn Store) -> Result<CommandOutcome, CommandError> {
    match store.read_language() {
        Some(content) => Ok(CommandOutcome::Language(content)),
        None => Err(CommandError::new(
            ErrorCode::NotFound,
            "this project has no LANGUAGE document (shared vocabulary)",
        )),
    }
}

fn run_discuss_show(store: &dyn Store, slug: &str) -> Result<CommandOutcome, CommandError> {
    let content = crate::discuss::show_discussion(store, slug).ok_or_else(|| {
        CommandError::new(ErrorCode::NotFound, format!("discussion '{slug}' not found"))
    })?;
    Ok(CommandOutcome::DiscussShow(DiscussShowOutcome {
        info: crate::discuss::info(store, slug),
        content,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    /// Minimal spec-driven change meta accepted by ChangeMeta::from_text.
    const META: &str = "schema: spec-driven\n";

    fn list_cmd() -> Command {
        Command::List {
            sort: "name".to_string(),
            specs: false,
            changes: false,
        }
    }

    // --- 查詢群經唯一進入點回 typed outcome、不產生事件 ---

    #[test]
    fn list_returns_typed_outcome_without_events() {
        let store = TestStore::with_meta("demo", META);
        let (outcome, events) = execute(&store, None, list_cmd()).expect("list executes");
        match outcome {
            CommandOutcome::List(list) => {
                let changes = list.changes.expect("changes section present");
                assert!(
                    changes.iter().any(|c| c.name == "demo"),
                    "list outcome carries the change"
                );
                assert!(list.specs.is_none(), "specs section only when requested");
            }
            other => panic!("expected a list outcome, got {other:?}"),
        }
        assert!(events.is_empty(), "queries never produce events");
    }

    #[test]
    fn status_returns_typed_outcome_without_events() {
        let store = TestStore::with_meta("demo", META);
        let (outcome, events) = execute(
            &store,
            None,
            Command::Status {
                change: Some("demo".to_string()),
                schema: None,
            },
        )
        .expect("status executes");
        match outcome {
            CommandOutcome::Status(report) => {
                assert_eq!(report.change_name, "demo");
                assert_eq!(report.schema_name, "spec-driven");
            }
            other => panic!("expected a status outcome, got {other:?}"),
        }
        assert!(events.is_empty(), "queries never produce events");
    }

    #[test]
    fn validate_returns_typed_outcome_without_events() {
        let store = TestStore::with_meta("demo", META);
        let (outcome, events) = execute(
            &store,
            None,
            Command::Validate {
                item: Some("demo".to_string()),
                all: false,
                changes: false,
                strict: false,
            },
        )
        .expect("validate executes");
        match outcome {
            CommandOutcome::Validate(v) => {
                assert_eq!(v.results.len(), 1);
                assert_eq!(v.results[0].change, "demo");
            }
            other => panic!("expected a validate outcome, got {other:?}"),
        }
        assert!(events.is_empty(), "queries never produce events");
    }

    // --- 不存在的主體：not_found，訊息沿用現行 CLI 文字 ---

    #[test]
    fn status_of_missing_change_is_not_found() {
        let store = TestStore::with_meta("demo", META);
        let err = execute(
            &store,
            None,
            Command::Status {
                change: Some("ghost".to_string()),
                schema: None,
            },
        )
        .expect_err("missing change must fail");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.code.as_str(), "not_found", "stable registry string");
        assert_eq!(err.message, "Change 'ghost' not found.", "frozen CLI text");
    }

    #[test]
    fn validate_of_missing_change_is_not_found() {
        let store = TestStore::default();
        let err = execute(
            &store,
            None,
            Command::Validate {
                item: Some("ghost".to_string()),
                all: false,
                changes: false,
                strict: false,
            },
        )
        .expect_err("missing change must fail");
        assert_eq!(err.code, ErrorCode::NotFound);
    }

    // --- auto-detect 語意：零與多個 active change ---

    #[test]
    fn status_with_no_changes_is_not_found_with_the_informational_text() {
        let store = TestStore::default();
        let err = execute(&store, None, Command::Status { change: None, schema: None })
            .expect_err("no changes must fail resolution");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(
            err.message,
            "No active changes. Create one with: speclink new change <name>"
        );
    }

    #[test]
    fn status_with_multiple_changes_is_invalid_argv_naming_candidates() {
        let store = TestStore::with_meta("alpha", META);
        store.metas.borrow_mut().insert("beta".to_string(), META.to_string());
        let err = execute(&store, None, Command::Status { change: None, schema: None })
            .expect_err("ambiguous auto-detect must fail");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
        assert!(
            err.message.starts_with("Multiple changes found. Use --change to specify one:"),
            "frozen CLI wording: {}",
            err.message
        );
        assert!(err.message.contains("alpha") && err.message.contains("beta"));
    }

    // --- 穩定錯誤碼註冊表：碼字串值域固定 ---

    #[test]
    fn error_code_registry_strings_are_stable() {
        assert_eq!(ErrorCode::InvalidArgv.as_str(), "invalid_argv");
        assert_eq!(ErrorCode::NotFound.as_str(), "not_found");
        assert_eq!(ErrorCode::InvalidConfig.as_str(), "invalid_config");
        assert_eq!(ErrorCode::Refused.as_str(), "refused");
        assert_eq!(ErrorCode::Error.as_str(), "error");
    }
}
