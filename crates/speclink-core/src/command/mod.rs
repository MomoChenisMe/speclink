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
#[derive(Debug)]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
    /// The original flow error when one exists. Hosts with their own error
    /// taxonomy (the Node SDK's store-bridge failures) downcast it to refine
    /// their envelope — that taxonomy stays at the envelope layer (決策三).
    pub source: Option<anyhow::Error>,
}

impl CommandError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> CommandError {
        CommandError {
            code,
            message: message.into(),
            source: None,
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

/// Typed refusal carried inside anyhow errors from core guard points (discard's
/// started-work guard, discuss discard's rounds guard) so the runtime classifies
/// them `refused` without string matching. Display is the exact frozen CLI text,
/// so every existing anyhow-printing path is byte-identical.
#[derive(Debug)]
pub struct Refusal(pub String);

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refusal {}

/// Classify a core-flow anyhow error: a [`Refusal`] marker → `refused`, a
/// [`crate::model::MetaError`] (corrupt `.openspec.yaml`) → `invalid_config`,
/// everything else → `error`. Message text passes through verbatim; the
/// original error rides along as `source` for host-side refinement.
fn classify(e: anyhow::Error) -> CommandError {
    let (code, message) = if let Some(r) = e.downcast_ref::<Refusal>() {
        (ErrorCode::Refused, r.0.clone())
    } else if let Some(m) = e.downcast_ref::<crate::model::MetaError>() {
        (ErrorCode::InvalidConfig, m.to_string())
    } else {
        (ErrorCode::Error, e.to_string())
    };
    CommandError {
        code,
        message,
        source: Some(e),
    }
}

/// Command-layer fail-closed gate: a resolved change whose metadata is corrupt
/// refuses as `invalid_config` before the verb's flow runs (spec「單一 change
/// 查詢對壞 metadata fail closed」).
fn guard_meta(change: &Change) -> Result<(), CommandError> {
    crate::model::require_valid_meta(change).map_err(|e| classify(e.into()))
}

/// Domain events reported by mutating verbs (design 決策四). Payload = subject
/// identity (change name / discussion slug) + the minimal fact of the mutation
/// + the UTC execution timestamp. No actor and no revision yet (binding and
/// teamstore knives). Experimental contract: payloads may change incompatibly
/// until event persistence lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    ChangeCreated { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ArtifactCreated { change: String, artifact: String, occurred_at: chrono::DateTime<chrono::Utc> },
    TaskCompleted { change: String, task_id: usize, occurred_at: chrono::DateTime<chrono::Utc> },
    TaskUncompleted { change: String, task_id: usize, occurred_at: chrono::DateTime<chrono::Utc> },
    /// No fs-store success path today — the mapping is contract for the remote store.
    ChangeClaimed { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeMarkedInProgress { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeArchived { change: String, dated_name: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeDiscarded { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionCreated { slug: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionContextSet { slug: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionRoundAdded { slug: String, round: usize, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionConcluded { slug: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionPromoted { slug: String, change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionLinked { slug: String, change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionSealed { slug: String, change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionArchived { slug: String, occurred_at: chrono::DateTime<chrono::Utc> },
    DiscussionDiscarded { slug: String, occurred_at: chrono::DateTime<chrono::Utc> },
}

impl DomainEvent {
    /// The event's stable kind name (the spec coverage table's wire strings).
    pub fn kind(&self) -> &'static str {
        match self {
            DomainEvent::ChangeCreated { .. } => "change-created",
            DomainEvent::ArtifactCreated { .. } => "artifact-created",
            DomainEvent::TaskCompleted { .. } => "task-completed",
            DomainEvent::TaskUncompleted { .. } => "task-uncompleted",
            DomainEvent::ChangeClaimed { .. } => "change-claimed",
            DomainEvent::ChangeMarkedInProgress { .. } => "change-marked-in-progress",
            DomainEvent::ChangeArchived { .. } => "change-archived",
            DomainEvent::ChangeDiscarded { .. } => "change-discarded",
            DomainEvent::DiscussionCreated { .. } => "discussion-created",
            DomainEvent::DiscussionContextSet { .. } => "discussion-context-set",
            DomainEvent::DiscussionRoundAdded { .. } => "discussion-round-added",
            DomainEvent::DiscussionConcluded { .. } => "discussion-concluded",
            DomainEvent::DiscussionPromoted { .. } => "discussion-promoted",
            DomainEvent::DiscussionLinked { .. } => "discussion-linked",
            DomainEvent::DiscussionSealed { .. } => "discussion-sealed",
            DomainEvent::DiscussionArchived { .. } => "discussion-archived",
            DomainEvent::DiscussionDiscarded { .. } => "discussion-discarded",
        }
    }
}

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
    // --- 變更群 ---
    /// `new change <name> [--description] [--schema] [--agent] [--from-discussion]`
    NewChange {
        name: String,
        description: Option<String>,
        schema: Option<String>,
        agent: Option<String>,
        from_discussion: Option<String>,
    },
    /// `new artifact <type> [capability] [--change <name>] [--force]`;
    /// `content` is the CLI's `--stdin` payload.
    NewArtifact {
        kind: String,
        capability: Option<String>,
        change: Option<String>,
        content: Option<String>,
        force: bool,
    },
    /// `task done <task_id> [--change <name>]` (`task_id` stays the raw argv
    /// token — validation and its frozen messages live in the runtime).
    TaskDone {
        task_id: String,
        change: Option<String>,
    },
    /// `task undone <task_id> [--change <name>]`
    TaskUndone {
        task_id: String,
        change: Option<String>,
    },
    /// `claim <name>` — remote-store only; the plain-store path refuses.
    Claim { name: String },
    /// `in-progress add <name>` — silent and idempotent (unknown names included).
    InProgressAdd { name: String },
    /// `archive [change] [--skip-specs] [--no-validate] [--mark-tasks-complete]`
    /// (single change; the CLI's `--all`/bulk loop stays in the entry point).
    Archive {
        change: Option<String>,
        skip_specs: bool,
        no_validate: bool,
        mark_tasks_complete: bool,
    },
    /// `discard <change> [--force]`
    Discard { change: String, force: bool },
    /// `discuss new <topic> [--slug <slug>]`
    DiscussNew { topic: String, slug: Option<String> },
    /// `discuss context <slug>` with stdin content
    DiscussContext { slug: String, content: String },
    /// `discuss add-round <slug> --mode <mode>` with stdin content
    DiscussAddRound {
        slug: String,
        mode: String,
        content: String,
    },
    /// `discuss conclude <slug>` with stdin content
    DiscussConclude { slug: String, content: String },
    /// `discuss promote <slug> [--name <change>]`
    DiscussPromote { slug: String, name: Option<String> },
    /// `discuss link <slug> --change <change>`
    DiscussLink { slug: String, change: String },
    /// `discuss seal <slug> --change <change>`
    DiscussSeal { slug: String, change: String },
    /// `discuss archive <slug>`
    DiscussArchive { slug: String },
    /// `discuss discard <slug> [--force]`
    DiscussDiscard { slug: String, force: bool },
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

/// `new change` outcome.
#[derive(Debug)]
pub struct NewChangeOutcome {
    pub name: String,
    pub dir: std::path::PathBuf,
    /// The schema the change was created with (explicit or config default).
    pub schema: String,
}

/// `new artifact` outcome.
#[derive(Debug)]
pub struct NewArtifactOutcome {
    pub artifact: String,
    pub change: String,
    pub path: std::path::PathBuf,
    /// True when caller content was written (vs. the schema template/empty file).
    pub had_content: bool,
}

/// `task done` / `task undone` outcome. `task_id_arg` preserves the raw argv
/// token for rendering (the CLI echoes the input verbatim, e.g. "01");
/// `task_id` is the parsed index the mutation used. `already` = nothing
/// changed (zero file effects) — presentation stays with the entry point.
#[derive(Debug)]
pub struct TaskFlipOutcome {
    pub change: String,
    pub task_id: usize,
    pub task_id_arg: String,
    pub description: String,
    pub already: bool,
}

/// `in-progress add` outcome: whether this call stamped the marker (false for
/// the idempotent/unknown-name silent successes — no event then).
#[derive(Debug)]
pub struct InProgressOutcome {
    pub name: String,
    pub stamped: bool,
}

/// `discuss context` / `discuss discard` outcome (subject only).
#[derive(Debug)]
pub struct DiscussSubjectOutcome {
    pub slug: String,
}

/// `discuss add-round` outcome.
#[derive(Debug)]
pub struct DiscussRoundOutcome {
    pub slug: String,
    pub mode: String,
    pub round: usize,
}

/// `discuss conclude` outcome: changes flagged stale by a re-conclude.
#[derive(Debug)]
pub struct DiscussConcludeOutcome {
    pub slug: String,
    pub restale_flagged: Vec<String>,
}

/// `discuss promote` outcome.
#[derive(Debug)]
pub struct DiscussPromoteOutcome {
    pub slug: String,
    pub change: String,
    pub path: std::path::PathBuf,
}

/// `discuss link` / `discuss seal` outcome.
#[derive(Debug)]
pub struct DiscussBindOutcome {
    pub slug: String,
    pub change: String,
}

/// `discuss archive` outcome.
#[derive(Debug)]
pub struct DiscussArchiveOutcome {
    pub slug: String,
    /// Dated file name inside discussions/archive/.
    pub archived_file: String,
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
    NewChange(NewChangeOutcome),
    NewArtifact(NewArtifactOutcome),
    TaskDone(TaskFlipOutcome),
    TaskUndone(TaskFlipOutcome),
    InProgressAdd(InProgressOutcome),
    Archive(crate::archive::ArchiveOutcome),
    Discard(crate::discard::DiscardOutcome),
    DiscussNew(crate::discuss::DiscussionInfo),
    DiscussContext(DiscussSubjectOutcome),
    DiscussAddRound(DiscussRoundOutcome),
    DiscussConclude(DiscussConcludeOutcome),
    DiscussPromote(DiscussPromoteOutcome),
    DiscussLink(DiscussBindOutcome),
    DiscussSeal(DiscussBindOutcome),
    DiscussArchive(DiscussArchiveOutcome),
    DiscussDiscard(DiscussSubjectOutcome),
}

/// The Host-resolved engine-side execution context — resolved once at the
/// Host boundary and consumed by every flow downstream. Command inputs carry
/// no actor or policy fields, so this context is the only identity source;
/// the Engine itself never reads process env or git identity.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Display identity ("Name <email>") stamping flows record. `None` =
    /// anonymous: stamping flows stamp nothing (the current local behavior
    /// when git is absent or user.name is unset).
    pub actor: Option<String>,
    /// The SPECLINK_* env-override layer of policy resolution, read at the
    /// Host boundary and injected here.
    pub env: crate::config::EnvOverrides,
    /// Host workspace for host-side lookups (schema resolution, app config,
    /// drift's git probes). `None` = no local workspace (Node host store):
    /// flows that need host files are only dispatched by entry points that
    /// hold a real workspace (the CLI).
    pub workspace: Option<Workspace>,
    /// The machine-level speclink directory (user schemas), resolved at the
    /// Host boundary (speclink-host's `global_config_dir`). `None` skips the
    /// user schema location.
    pub user_config_dir: Option<std::path::PathBuf>,
}

/// Execute one command against the store under the Host-resolved context.
/// Returns the typed outcome plus the domain events the execution produced
/// (always empty for queries).
pub fn execute(
    store: &dyn Store,
    ctx: &ExecutionContext,
    cmd: Command,
) -> Result<(CommandOutcome, Vec<DomainEvent>), CommandError> {
    let ws = ctx.workspace.as_ref();
    let outcome = match cmd {
        Command::List { sort, specs, changes } => run_list(store, &sort, specs, changes),
        Command::Show { item, item_type } => run_show(store, item.as_deref(), item_type.as_deref()),
        Command::Status { change, schema } => {
            run_status(store, ws, ctx.user_config_dir.as_deref(), change.as_deref(), schema.as_deref())
        }
        Command::Instructions { artifact, change, schema } => {
            run_instructions(store, ws, ctx.user_config_dir.as_deref(), &ctx.env, artifact.as_deref(), change.as_deref(), schema.as_deref())
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
        Command::NewChange { name, description, schema, agent, from_discussion } => {
            run_new_change(store, ctx.actor.as_deref(), name, description, schema, agent, from_discussion)
        }
        Command::NewArtifact { kind, capability, change, content, force } => {
            run_new_artifact(store, ws, ctx.user_config_dir.as_deref(), &kind, capability.as_deref(), change.as_deref(), content.as_deref(), force)
        }
        Command::TaskDone { task_id, change } => {
            run_task_flip(store, ws, ctx.actor.as_deref(), &task_id, change.as_deref(), TaskFlip::Done)
        }
        Command::TaskUndone { task_id, change } => {
            run_task_flip(store, ws, ctx.actor.as_deref(), &task_id, change.as_deref(), TaskFlip::Undone)
        }
        Command::Claim { name } => {
            // Fail-closed gate first: claiming a change whose metadata is
            // corrupt must name the broken file, not the missing remote store.
            if let Some(change) = crate::model::find_change(store, &name) {
                guard_meta(&change)?;
            }
            Err(CommandError::new(
                ErrorCode::Error,
                "claim requires a remote store — this project uses the local fs store",
            ))
        }
        Command::InProgressAdd { name } => run_in_progress_add(store, ctx.actor.as_deref(), &name),
        Command::Archive { change, skip_specs, no_validate, mark_tasks_complete } => run_archive(
            store,
            ws,
            ctx.actor.as_deref(),
            change.as_deref(),
            crate::archive::ArchiveOptions { skip_specs, no_validate, mark_tasks_complete },
        ),
        Command::Discard { change, force } => run_discard(store, ws, &change, force),
        Command::DiscussNew { topic, slug } => run_discuss_new(store, ctx.actor.as_deref(), &topic, slug.as_deref()),
        Command::DiscussContext { slug, content } => {
            crate::discuss::set_context(store, &slug, &content).map_err(classify)?;
            Ok(CommandOutcome::DiscussContext(DiscussSubjectOutcome { slug }))
        }
        Command::DiscussAddRound { slug, mode, content } => {
            let round = crate::discuss::add_round(store, &slug, &mode, &content).map_err(classify)?;
            Ok(CommandOutcome::DiscussAddRound(DiscussRoundOutcome { slug, mode, round }))
        }
        Command::DiscussConclude { slug, content } => {
            let restale_flagged = crate::discuss::conclude(store, &slug, &content).map_err(classify)?;
            Ok(CommandOutcome::DiscussConclude(DiscussConcludeOutcome { slug, restale_flagged }))
        }
        Command::DiscussPromote { slug, name } => {
            let o = crate::discuss::promote(store, &slug, name.as_deref(), ctx.actor.as_deref())
                .map_err(classify)?;
            Ok(CommandOutcome::DiscussPromote(DiscussPromoteOutcome {
                slug,
                change: o.change,
                path: o.path,
            }))
        }
        Command::DiscussLink { slug, change } => {
            crate::discuss::link(store, &slug, &change).map_err(classify)?;
            Ok(CommandOutcome::DiscussLink(DiscussBindOutcome { slug, change }))
        }
        Command::DiscussSeal { slug, change } => {
            crate::discuss::seal(store, &slug, &change).map_err(classify)?;
            Ok(CommandOutcome::DiscussSeal(DiscussBindOutcome { slug, change }))
        }
        Command::DiscussArchive { slug } => {
            match crate::discuss::archive_discussion(store, &slug).map_err(classify)? {
                Some(archived_file) => {
                    Ok(CommandOutcome::DiscussArchive(DiscussArchiveOutcome { slug, archived_file }))
                }
                None => Err(CommandError::new(
                    ErrorCode::NotFound,
                    format!("discussion '{slug}' not found"),
                )),
            }
        }
        Command::DiscussDiscard { slug, force } => {
            crate::discuss::discard_discussion(store, &slug, force).map_err(classify)?;
            Ok(CommandOutcome::DiscussDiscard(DiscussSubjectOutcome { slug }))
        }
    }?;
    let events = events_of(&outcome);
    Ok((outcome, events))
}

/// The single event-emission point (design 決策四): events derive from the
/// typed outcome after the core flow succeeded. Queries yield none; outcomes
/// that report "nothing changed" (already-flipped task, unstamped in-progress)
/// yield none either — an event states a mutation that actually happened.
fn events_of(outcome: &CommandOutcome) -> Vec<DomainEvent> {
    let at = chrono::Utc::now();
    match outcome {
        CommandOutcome::List(_)
        | CommandOutcome::Show(_)
        | CommandOutcome::Status(_)
        | CommandOutcome::Instructions(_)
        | CommandOutcome::Validate(_)
        | CommandOutcome::Analyze(_)
        | CommandOutcome::Drift(_)
        | CommandOutcome::ArtifactCat(_)
        | CommandOutcome::Language(_)
        | CommandOutcome::DiscussList(_)
        | CommandOutcome::DiscussShow(_) => Vec::new(),
        CommandOutcome::NewChange(o) => vec![DomainEvent::ChangeCreated {
            change: o.name.clone(),
            occurred_at: at,
        }],
        CommandOutcome::NewArtifact(o) => vec![DomainEvent::ArtifactCreated {
            change: o.change.clone(),
            artifact: o.artifact.clone(),
            occurred_at: at,
        }],
        CommandOutcome::TaskDone(o) if o.already => Vec::new(),
        CommandOutcome::TaskDone(o) => vec![DomainEvent::TaskCompleted {
            change: o.change.clone(),
            task_id: o.task_id,
            occurred_at: at,
        }],
        CommandOutcome::TaskUndone(o) if o.already => Vec::new(),
        CommandOutcome::TaskUndone(o) => vec![DomainEvent::TaskUncompleted {
            change: o.change.clone(),
            task_id: o.task_id,
            occurred_at: at,
        }],
        CommandOutcome::InProgressAdd(o) if !o.stamped => Vec::new(),
        CommandOutcome::InProgressAdd(o) => vec![DomainEvent::ChangeMarkedInProgress {
            change: o.name.clone(),
            occurred_at: at,
        }],
        CommandOutcome::Archive(o) => vec![DomainEvent::ChangeArchived {
            change: o.change_name.clone(),
            dated_name: o.dated_name.clone(),
            occurred_at: at,
        }],
        CommandOutcome::Discard(o) => vec![DomainEvent::ChangeDiscarded {
            change: o.change_name.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussNew(info) => vec![DomainEvent::DiscussionCreated {
            slug: info.slug.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussContext(o) => vec![DomainEvent::DiscussionContextSet {
            slug: o.slug.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussAddRound(o) => vec![DomainEvent::DiscussionRoundAdded {
            slug: o.slug.clone(),
            round: o.round,
            occurred_at: at,
        }],
        CommandOutcome::DiscussConclude(o) => vec![DomainEvent::DiscussionConcluded {
            slug: o.slug.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussPromote(o) => vec![
            DomainEvent::DiscussionPromoted {
                slug: o.slug.clone(),
                change: o.change.clone(),
                occurred_at: at,
            },
            DomainEvent::ChangeCreated {
                change: o.change.clone(),
                occurred_at: at,
            },
        ],
        CommandOutcome::DiscussLink(o) => vec![DomainEvent::DiscussionLinked {
            slug: o.slug.clone(),
            change: o.change.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussSeal(o) => vec![DomainEvent::DiscussionSealed {
            slug: o.slug.clone(),
            change: o.change.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussArchive(o) => vec![DomainEvent::DiscussionArchived {
            slug: o.slug.clone(),
            occurred_at: at,
        }],
        CommandOutcome::DiscussDiscard(o) => vec![DomainEvent::DiscussionDiscarded {
            slug: o.slug.clone(),
            occurred_at: at,
        }],
    }
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
fn resolve_schema(
    ws: Option<&Workspace>,
    user_dir: Option<&std::path::Path>,
    name: &str,
) -> Result<Schema, CommandError> {
    match crate::schema::resolve_with(ws, user_dir, name) {
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
    user_dir: Option<&std::path::Path>,
    change: Option<&str>,
    schema: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    guard_meta(&change)?;
    let schema_name = match schema {
        Some(s) => s.to_string(),
        None => change.meta.schema_name(),
    };
    let schema = resolve_schema(ws, user_dir, &schema_name)?;
    Ok(CommandOutcome::Status(crate::status::build(
        store, &change, &schema,
    )))
}

fn run_instructions(
    store: &dyn Store,
    ws: Option<&Workspace>,
    user_dir: Option<&std::path::Path>,
    env: &crate::config::EnvOverrides,
    artifact: Option<&str>,
    change: Option<&str>,
    schema: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    guard_meta(&change)?;
    let schema = match schema {
        Some(s) => resolve_schema(ws, user_dir, s)?,
        None => resolve_schema(ws, user_dir, &change.meta.schema_name())?,
    };
    // No-arg default: the first incomplete artifact, or the apply view once
    // every artifact exists (matches Spectra).
    let default_artifact = crate::status::first_incomplete_artifact(store, &change, &schema)
        .unwrap_or_else(|| "apply".to_string());
    let artifact = artifact.unwrap_or(&default_artifact);
    let host = host_workspace(ws);
    if artifact == "apply" {
        let payload = crate::instructions::build_apply(&host, store, env, &change, &schema)?;
        return Ok(CommandOutcome::Instructions(InstructionsOutcome::Apply(payload)));
    }
    let payload = crate::instructions::build_artifact(&host, store, env, &change, &schema, artifact)?
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
    // The fail-closed gate covers the single-change target paths only — a
    // multi-change sweep must not die on one corrupt item (mirrors list).
    let mut changes = if let Some(item) = item {
        let c = crate::model::find_change(store, item).ok_or_else(|| {
            CommandError::new(ErrorCode::NotFound, format!("Change '{item}' not found."))
        })?;
        guard_meta(&c)?;
        vec![c]
    } else if all || changes_flag {
        crate::model::list_changes(store)
    } else {
        // No item: exactly one change validates alone; zero or several fall
        // back to validating everything (matches Spectra).
        match resolve_change(store, None, SPECIFY_FLAG) {
            Ok(c) => {
                guard_meta(&c)?;
                vec![c]
            }
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
    guard_meta(&change)?;
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
    guard_meta(&change)?;
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
    guard_meta(&change)?;
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

fn run_new_change(
    store: &dyn Store,
    actor: Option<&str>,
    name: String,
    description: Option<String>,
    schema: Option<String>,
    agent: Option<String>,
    from_discussion: Option<String>,
) -> Result<CommandOutcome, CommandError> {
    // Default schema comes from openspec/config.yaml; the name is NOT validated
    // here (downstream commands fail on resolution, matching Spectra).
    let schema = match schema {
        Some(s) => s,
        None => crate::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())?
            .schema_name(),
    };
    if let Some(slug) = from_discussion.as_deref() {
        if crate::discuss::info(store, slug).is_none() {
            return Err(CommandError::new(
                ErrorCode::NotFound,
                format!("discussion '{slug}' not found — run `speclink discuss new` first"),
            ));
        }
    }
    let dir = crate::newcmd::new_change(
        store,
        &name,
        description.as_deref(),
        &schema,
        agent.as_deref(),
        from_discussion.as_deref(),
        actor,
    )
    .map_err(classify)?;
    // A change born of a discussion marks that discussion promoted — both
    // entry points already did this inline; it is part of the verb's meaning.
    if let Some(slug) = from_discussion.as_deref() {
        crate::discuss::mark_promoted(store, slug, &name).map_err(classify)?;
    }
    Ok(CommandOutcome::NewChange(NewChangeOutcome { name, dir, schema }))
}

fn run_new_artifact(
    store: &dyn Store,
    ws: Option<&Workspace>,
    user_dir: Option<&std::path::Path>,
    kind: &str,
    capability: Option<&str>,
    change: Option<&str>,
    content: Option<&str>,
    force: bool,
) -> Result<CommandOutcome, CommandError> {
    let type_ok = ["proposal", "design", "tasks", "spec"].contains(&kind);
    let type_err = || {
        CommandError::new(
            ErrorCode::InvalidArgv,
            format!("Unknown artifact type '{kind}'. Valid types: proposal, design, tasks, spec"),
        )
    };
    // Spectra's order: with an explicit --change, validate the type before
    // existence; when auto-detecting, resolve the change first (so "No active
    // changes" wins over a bad type). Change-not-found here has NO trailing period.
    let change = match change {
        Some(name) => {
            if !type_ok {
                return Err(type_err());
            }
            crate::model::find_change(store, name).ok_or_else(|| {
                CommandError::new(ErrorCode::NotFound, format!("Change '{name}' not found"))
            })?
        }
        None => {
            let c = resolve_change(store, None, SPECIFY_FLAG)?;
            if !type_ok {
                return Err(type_err());
            }
            c
        }
    };
    // Best-effort schema resolution: an unresolvable/broken schema still
    // creates the artifact (no template → empty file), matching Spectra.
    let schema = match crate::schema::resolve_with(ws, user_dir, &change.meta.schema_name()) {
        Some(Ok(s)) => s,
        _ => Schema {
            name: change.meta.schema_name(),
            display_name: change.meta.schema_name(),
            description: None,
            source: "project".to_string(),
            artifacts: Vec::new(),
            apply_requires: Vec::new(),
            apply_tracks: None,
            apply_instruction: None,
        },
    };
    let had_content = content.is_some();
    let (artifact_id, path) =
        crate::newcmd::new_artifact(store, &change, &schema, kind, capability, content, force)
            .map_err(classify)?;
    Ok(CommandOutcome::NewArtifact(NewArtifactOutcome {
        artifact: artifact_id,
        change: change.name,
        path,
        had_content,
    }))
}

/// Which way a task checkbox flips.
enum TaskFlip {
    Done,
    Undone,
}

fn run_task_flip(
    store: &dyn Store,
    ws: Option<&Workspace>,
    actor: Option<&str>,
    task_id: &str,
    change: Option<&str>,
    flip: TaskFlip,
) -> Result<CommandOutcome, CommandError> {
    // `task done`/`task undone` do not require the change to exist — they go
    // straight to tasks.md, and its existence is checked BEFORE the id
    // (matching Spectra's order).
    let change_name = match change {
        Some(name) => name.to_string(),
        None => resolve_change(store, None, SPECIFY_FLAG)?.name,
    };
    if !store.artifact_exists(&change_name, "tasks.md") {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("tasks.md not found for change '{change_name}'"),
        ));
    }
    let id: usize = task_id.parse().map_err(|_| {
        CommandError::new(
            ErrorCode::InvalidArgv,
            format!("Invalid task ID '{task_id}': must be a number"),
        )
    })?;
    if id < 1 {
        return Err(CommandError::new(ErrorCode::InvalidArgv, "Task ID must be >= 1"));
    }
    let host = host_workspace(ws);
    let (description, already) = match flip {
        TaskFlip::Done => {
            let o = crate::tasks::complete(
                store,
                &host,
                &change_name,
                id,
                actor,
                None,
            )
            .map_err(classify)?;
            (o.description, o.already)
        }
        TaskFlip::Undone => {
            let o = crate::tasks::uncomplete(store, &change_name, id).map_err(classify)?;
            (o.description, o.already)
        }
    };
    let outcome = TaskFlipOutcome {
        change: change_name,
        task_id: id,
        task_id_arg: task_id.to_string(),
        description,
        already,
    };
    Ok(match flip {
        TaskFlip::Done => CommandOutcome::TaskDone(outcome),
        TaskFlip::Undone => CommandOutcome::TaskUndone(outcome),
    })
}

fn run_in_progress_add(
    store: &dyn Store,
    actor: Option<&str>,
    name: &str,
) -> Result<CommandOutcome, CommandError> {
    let stamped = crate::inprogress::add(store, name, actor, None).map_err(classify)?;
    Ok(CommandOutcome::InProgressAdd(InProgressOutcome {
        name: name.to_string(),
        stamped,
    }))
}

fn run_archive(
    store: &dyn Store,
    ws: Option<&Workspace>,
    actor: Option<&str>,
    change: Option<&str>,
    opts: crate::archive::ArchiveOptions,
) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_FLAG)?;
    // Gate before --mark-tasks-complete's pre-write; the core archive flow
    // gates again for entry points that call it directly (desktop).
    guard_meta(&change)?;
    if opts.mark_tasks_complete {
        if let Some(text) = store.read_artifact(&change.name, "tasks.md") {
            // Star-bullet checkboxes are tasks too (matches Spectra).
            let done = text
                .replace("- [ ] ", "- [x] ")
                .replace("- [ ]\t", "- [x]\t")
                .replace("* [ ] ", "* [x] ")
                .replace("* [ ]\t", "* [x]\t");
            store
                .write_artifact(&change.name, "tasks.md", &done)
                .map_err(classify)?;
        }
    }
    let host = host_workspace(ws);
    // The in-progress marker stays untouched on archive (matches Spectra).
    let outcome = crate::archive::archive(&host, store, &change, &opts, actor).map_err(classify)?;
    Ok(CommandOutcome::Archive(outcome))
}

fn run_discard(
    store: &dyn Store,
    ws: Option<&Workspace>,
    change: &str,
    force: bool,
) -> Result<CommandOutcome, CommandError> {
    if crate::model::find_change(store, change).is_none() {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("Change '{change}' not found."),
        ));
    }
    let host = host_workspace(ws);
    let outcome = crate::discard::discard(&host, store, change, force).map_err(classify)?;
    Ok(CommandOutcome::Discard(outcome))
}

fn run_discuss_new(
    store: &dyn Store,
    actor: Option<&str>,
    topic: &str,
    slug: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let info = crate::discuss::new_discussion(store, topic, slug, actor).map_err(classify)?;
    Ok(CommandOutcome::DiscussNew(info))
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
        let (outcome, events) = execute(&store, &ExecutionContext::default(), list_cmd()).expect("list executes");
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
            &ExecutionContext::default(),
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
            &ExecutionContext::default(),
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

    // --- 壞 metadata 的查詢群 fail closed 與 list 診斷（spec command-runtime）---

    #[test]
    fn single_change_queries_on_corrupt_meta_are_invalid_config_with_path_and_reason() {
        // spec「單一 change 查詢對壞 metadata fail closed」＋「穩定錯誤碼註冊表」
        // Example 新行：.openspec.yaml 存在但解析失敗 → invalid_config，訊息
        // 沿 ConfigError 形式指出 workspace 相對路徑與解析原因。
        let store = TestStore::with_meta("demo", BAD_META);
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
        let cmds: Vec<(&str, Command)> = vec![
            ("status", Command::Status { change: Some("demo".to_string()), schema: None }),
            (
                "instructions",
                Command::Instructions { artifact: None, change: Some("demo".to_string()), schema: None },
            ),
            (
                "validate",
                Command::Validate {
                    item: Some("demo".to_string()),
                    all: false,
                    changes: false,
                    strict: false,
                },
            ),
            ("analyze", Command::Analyze { change: Some("demo".to_string()) }),
            ("drift", Command::Drift { change: Some("demo".to_string()) }),
            (
                "artifact cat",
                Command::ArtifactCat { artifact: "tasks".to_string(), change: Some("demo".to_string()) },
            ),
        ];
        for (verb, cmd) in cmds {
            let err = execute(&store, &ExecutionContext::default(), cmd)
                .expect_err(&format!("{verb} on corrupt meta must refuse"));
            assert_eq!(
                err.code,
                ErrorCode::InvalidConfig,
                "{verb} must classify invalid_config, got {:?}: {}",
                err.code,
                err.message
            );
            assert!(
                err.message.starts_with("invalid openspec/changes/demo/.openspec.yaml: "),
                "{verb} message must name path then reason: {}",
                err.message
            );
            assert!(
                err.message.len() > "invalid openspec/changes/demo/.openspec.yaml: ".len(),
                "{verb} message must carry the parse reason"
            );
        }
    }

    #[test]
    fn list_on_corrupt_meta_flags_the_item_and_keeps_valid_items_intact() {
        // spec「list 對壞 metadata 標 invalid 而不失效」：壞檔項目帶診斷，
        // 有效項目不帶且內容與無壞檔時一致。
        let store = TestStore::with_meta("good", META);
        store.metas.borrow_mut().insert("broken".to_string(), BAD_META.to_string());
        store.put_artifact("good", "tasks.md", "- [x] 1.1 a\n");
        let (outcome, _) = execute(&store, &ExecutionContext::default(), list_cmd()).expect("list must stay available");
        let CommandOutcome::List(list) = outcome else {
            panic!("expected a list outcome");
        };
        let changes = list.changes.expect("changes section present");
        assert_eq!(changes.len(), 2, "corrupt meta must not drop the change");
        let broken = changes.iter().find(|c| c.name == "broken").expect("broken listed");
        let reason = broken.meta_error.as_deref().expect("corrupt item carries the diagnostic");
        assert!(!reason.is_empty());
        let good = changes.iter().find(|c| c.name == "good").expect("good listed");
        assert!(good.meta_error.is_none(), "valid item carries no diagnostic");
        assert_eq!((good.completed_tasks, good.total_tasks), (1, 1));
        assert_eq!(good.status, "done");
    }

    // --- 壞 change metadata 的生命週期寫入 fail closed（spec change-lifecycle）---

    /// `.openspec.yaml` 存在但 YAML 解析失敗的固定樣本（與 store_fs 測試同款）。
    const BAD_META: &str = ": : :\n\t bad yaml [unclosed\n";

    #[test]
    fn task_flip_on_corrupt_meta_refuses_and_leaves_files_untouched() {
        // spec「task done 因蘊含開工標記而拒絕」：done 與 undone 皆拒，
        // tasks.md 與 .openspec.yaml 逐位元不變。
        const TASKS: &str = "- [ ] 1.1 a\n- [x] 1.2 b\n";
        let store = TestStore::with_meta("demo", BAD_META);
        store.put_artifact("demo", "tasks.md", TASKS);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()) },
        )
        .expect_err("task done on corrupt meta must refuse");
        assert!(
            err.message.contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {}",
            err.message
        );
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::TaskUndone { task_id: "2".to_string(), change: Some("demo".to_string()) },
        )
        .expect_err("task undone on corrupt meta must refuse");
        assert!(err.message.contains("openspec/changes/demo/.openspec.yaml"));
        assert_eq!(
            store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
            TASKS,
            "tasks.md byte-identical"
        );
        assert_eq!(store.meta("demo"), BAD_META, "meta byte-identical");
        assert_eq!(*store.artifact_writes.borrow(), 0, "no artifact write on refusal");
        assert_eq!(*store.meta_writes.borrow(), 0, "no meta write on refusal");
    }

    #[test]
    fn claim_on_corrupt_meta_refuses_naming_the_file() {
        let store = TestStore::with_meta("demo", BAD_META);
        let err = execute(&store, &ExecutionContext::default(), Command::Claim { name: "demo".to_string() })
            .expect_err("claim on corrupt meta must refuse");
        assert!(
            err.message.contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {}",
            err.message
        );
        assert_eq!(store.meta("demo"), BAD_META);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn archive_on_corrupt_meta_refuses_without_moving_or_merging() {
        // spec「archive 對壞 metadata 拒絕」：正典未併入、目錄未移動；
        // --mark-tasks-complete 的前置寫入也不得發生。
        const TASKS: &str = "- [ ] 1.1 open\n";
        for mark in [false, true] {
            let store = TestStore::with_meta("demo", BAD_META);
            store.put_artifact("demo", "tasks.md", TASKS);
            let err = execute(
                &store,
                &ExecutionContext::default(),
                Command::Archive {
                    change: Some("demo".to_string()),
                    skip_specs: false,
                    no_validate: false,
                    mark_tasks_complete: mark,
                },
            )
            .expect_err("archive on corrupt meta must refuse");
            assert!(
                err.message.contains("openspec/changes/demo/.openspec.yaml"),
                "error must name the metadata file (mark={mark}): {}",
                err.message
            );
            assert!(store.metas.borrow().contains_key("demo"), "change not moved (mark={mark})");
            assert!(store.archived_metas.borrow().is_empty(), "archive untouched (mark={mark})");
            assert!(store.canonical.borrow().is_empty(), "canon untouched (mark={mark})");
            assert_eq!(
                store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
                TASKS,
                "tasks.md byte-identical (mark={mark})"
            );
            assert_eq!(*store.artifact_writes.borrow(), 0, "no write (mark={mark})");
        }
    }

    #[test]
    fn new_artifact_on_corrupt_meta_refuses_without_default_schema_fallback() {
        // spec：壞 metadata 不得被解讀為預設 schema 而照常產出 artifact。
        let store = TestStore::with_meta("demo", BAD_META);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::NewArtifact {
                kind: "design".to_string(),
                capability: None,
                change: Some("demo".to_string()),
                content: None,
                force: false,
            },
        )
        .expect_err("new artifact on corrupt meta must refuse");
        assert!(
            err.message.contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {}",
            err.message
        );
        assert!(store.artifacts.borrow().is_empty(), "no artifact created via default schema");
        assert_eq!(*store.artifact_writes.borrow(), 0);
    }

    // --- 不存在的主體：not_found，訊息沿用現行 CLI 文字 ---

    #[test]
    fn status_of_missing_change_is_not_found() {
        let store = TestStore::with_meta("demo", META);
        let err = execute(
            &store,
            &ExecutionContext::default(),
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
            &ExecutionContext::default(),
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
        let err = execute(&store, &ExecutionContext::default(), Command::Status { change: None, schema: None })
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
        let err = execute(&store, &ExecutionContext::default(), Command::Status { change: None, schema: None })
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

    // === 變更型動詞的領域事件（spec: 變更型動詞的領域事件） ===

    /// Ghost workspace: nonexistent root — git probes fail soft, no snapshot
    /// or touched-record files are written (same pattern as archive/discard tests).
    fn ghost_ws() -> Workspace {
        Workspace {
            root: std::env::temp_dir().join("speclink-command-test-ghost-root"),
            spec_dir_name: "openspec".to_string(),
        }
    }

    /// Execute expecting success; returns (outcome, events).
    fn ok(
        store: &TestStore,
        cmd: Command,
    ) -> (CommandOutcome, Vec<DomainEvent>) {
        let ctx = ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() };
        execute(store, &ctx, cmd).expect("command succeeds")
    }

    fn kinds(events: &[DomainEvent]) -> Vec<&'static str> {
        events.iter().map(|e| e.kind()).collect()
    }

    #[test]
    fn new_change_reports_exactly_one_change_created_event() {
        // Spec scenario 建立變更回報 change-created.
        let store = TestStore::default();
        let (_, events) = ok(
            &store,
            Command::NewChange {
                name: "add-auth".to_string(),
                description: None,
                schema: None,
                agent: None,
                from_discussion: None,
            },
        );
        assert_eq!(events.len(), 1, "exactly one event");
        match &events[0] {
            DomainEvent::ChangeCreated { change, occurred_at } => {
                assert_eq!(change, "add-auth", "subject is the change name");
                assert!(
                    *occurred_at <= chrono::Utc::now(),
                    "occurredAt is a UTC timestamp of the execution"
                );
            }
            other => panic!("expected change-created, got {other:?}"),
        }
    }

    #[test]
    fn failed_new_change_produces_no_events_and_frozen_message() {
        // Spec scenario 失敗的命令不產生事件 (the Err arm carries no events by type).
        let store = TestStore::with_meta("demo", META);
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::NewChange {
                name: "demo".to_string(),
                description: None,
                schema: None,
                agent: None,
                from_discussion: None,
            },
        )
        .expect_err("duplicate name must fail");
        assert_eq!(err.code, ErrorCode::Error);
        assert_eq!(err.message, "Change 'demo' already exists.");
    }

    #[test]
    fn new_artifact_reports_artifact_created() {
        let store = TestStore::with_meta("demo", META);
        let (_, events) = ok(
            &store,
            Command::NewArtifact {
                kind: "proposal".to_string(),
                capability: None,
                change: Some("demo".to_string()),
                content: Some("## Why\n\nDemo.\n".to_string()),
                force: false,
            },
        );
        assert_eq!(kinds(&events), ["artifact-created"]);
        match &events[0] {
            DomainEvent::ArtifactCreated { change, artifact, .. } => {
                assert_eq!(change, "demo");
                assert_eq!(artifact, "proposal");
            }
            other => panic!("expected artifact-created, got {other:?}"),
        }
    }

    #[test]
    fn task_done_reports_task_completed() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 Do the thing\n");
        let (_, events) = ok(
            &store,
            Command::TaskDone {
                task_id: "1".to_string(),
                change: Some("demo".to_string()),
            },
        );
        assert_eq!(kinds(&events), ["task-completed"]);
        match &events[0] {
            DomainEvent::TaskCompleted { change, task_id, .. } => {
                assert_eq!(change, "demo");
                assert_eq!(*task_id, 1);
            }
            other => panic!("expected task-completed, got {other:?}"),
        }
    }

    #[test]
    fn already_done_task_produces_no_event() {
        // No state changed → no event; the outcome carries the `already` fact
        // for the entry point's own presentation (CLI error, GUI idempotence).
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 Done already\n");
        let (outcome, events) = ok(
            &store,
            Command::TaskDone {
                task_id: "1".to_string(),
                change: Some("demo".to_string()),
            },
        );
        assert!(events.is_empty(), "no event when nothing changed");
        match outcome {
            CommandOutcome::TaskDone(o) => assert!(o.already),
            other => panic!("expected a task-done outcome, got {other:?}"),
        }
    }

    #[test]
    fn task_undone_reports_task_uncompleted() {
        // 覆蓋表: task undone → task-uncompleted.
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 Do the thing\n");
        let (_, events) = ok(
            &store,
            Command::TaskUndone {
                task_id: "1".to_string(),
                change: Some("demo".to_string()),
            },
        );
        assert_eq!(kinds(&events), ["task-uncompleted"]);
    }

    #[test]
    fn claim_on_the_fs_store_is_an_error_without_events() {
        // claim is remote-store-only; the fs path refuses with the frozen text.
        // Its event mapping (change-claimed) is asserted in the kind table below.
        let store = TestStore::with_meta("demo", META);
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::Claim { name: "demo".to_string() },
        )
        .expect_err("claim must refuse on a plain store");
        assert_eq!(err.code, ErrorCode::Error);
        assert_eq!(
            err.message,
            "claim requires a remote store — this project uses the local fs store"
        );
    }

    #[test]
    fn in_progress_add_reports_change_marked_in_progress() {
        let store = TestStore::with_meta("demo", META);
        let (_, events) = ok(&store, Command::InProgressAdd { name: "demo".to_string() });
        assert_eq!(kinds(&events), ["change-marked-in-progress"]);
    }

    #[test]
    fn archive_reports_change_archived() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        let (_, events) = ok(
            &store,
            Command::Archive {
                change: Some("demo".to_string()),
                skip_specs: false,
                no_validate: false,
                mark_tasks_complete: false,
            },
        );
        assert_eq!(kinds(&events), ["change-archived"]);
        match &events[0] {
            DomainEvent::ChangeArchived { change, .. } => assert_eq!(change, "demo"),
            other => panic!("expected change-archived, got {other:?}"),
        }
    }

    #[test]
    fn discard_reports_change_discarded() {
        let store = TestStore::with_meta("demo", META);
        let (_, events) = ok(
            &store,
            Command::Discard { change: "demo".to_string(), force: false },
        );
        assert_eq!(kinds(&events), ["change-discarded"]);
    }

    #[test]
    fn discard_of_started_change_is_refused_without_force() {
        // Spec scenario 需 --force 的拒絕: started work refuses without --force,
        // classified refused, no files deleted, no events.
        let store = TestStore::with_meta("demo", "schema: spec-driven\nstarted_at: 2026-07-10\n");
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::Discard { change: "demo".to_string(), force: false },
        )
        .expect_err("started change must refuse discard");
        assert_eq!(err.code, ErrorCode::Refused);
        assert!(store.change_exists("demo"), "nothing deleted on refusal");
    }

    #[test]
    fn discuss_verbs_report_their_events() {
        // 覆蓋表 discuss 全系列（new/context/add-round/conclude/link/seal/
        // archive）：一條真實生命週期，每步斷言恰一筆對應事件。
        let store = TestStore::default();
        let (_, ev) = ok(
            &store,
            Command::DiscussNew {
                topic: "API auth".to_string(),
                slug: Some("api-auth".to_string()),
            },
        );
        assert_eq!(kinds(&ev), ["discussion-created"]);
        match &ev[0] {
            DomainEvent::DiscussionCreated { slug, .. } => assert_eq!(slug, "api-auth"),
            other => panic!("expected discussion-created, got {other:?}"),
        }

        let (_, ev) = ok(
            &store,
            Command::DiscussContext {
                slug: "api-auth".to_string(),
                content: "框架：假設訪談模式。\n".to_string(),
            },
        );
        assert_eq!(kinds(&ev), ["discussion-context-set"]);

        let (_, ev) = ok(
            &store,
            Command::DiscussAddRound {
                slug: "api-auth".to_string(),
                mode: "assumptions".to_string(),
                content: "第一輪內容。\n".to_string(),
            },
        );
        assert_eq!(kinds(&ev), ["discussion-round-added"]);

        let (_, ev) = ok(
            &store,
            Command::DiscussConclude {
                slug: "api-auth".to_string(),
                content: "結論：做。\n".to_string(),
            },
        );
        assert_eq!(kinds(&ev), ["discussion-concluded"]);

        // link + seal need a change to bind to.
        ok(
            &store,
            Command::NewChange {
                name: "auth-change".to_string(),
                description: None,
                schema: None,
                agent: None,
                from_discussion: None,
            },
        );
        let (_, ev) = ok(
            &store,
            Command::DiscussLink {
                slug: "api-auth".to_string(),
                change: "auth-change".to_string(),
            },
        );
        assert_eq!(kinds(&ev), ["discussion-linked"]);

        let (_, ev) = ok(
            &store,
            Command::DiscussSeal {
                slug: "api-auth".to_string(),
                change: "auth-change".to_string(),
            },
        );
        assert_eq!(kinds(&ev), ["discussion-sealed"]);

        let (_, ev) = ok(&store, Command::DiscussArchive { slug: "api-auth".to_string() });
        assert_eq!(kinds(&ev), ["discussion-archived"]);
    }

    #[test]
    fn discuss_discard_reports_discussion_discarded() {
        let store = TestStore::with_live_discussion(
            "scrap-idea",
            "---\nslug: scrap-idea\nstatus: open\ncreated: 2026-07-10\n---\n\n# Discussion: scrap\n",
        );
        let (_, events) = ok(
            &store,
            Command::DiscussDiscard { slug: "scrap-idea".to_string(), force: false },
        );
        assert_eq!(kinds(&events), ["discussion-discarded"]);
        assert!(!store.live_discussion_exists("scrap-idea"));
    }

    #[test]
    fn promote_reports_promoted_and_change_created() {
        // Spec scenario 複合動詞回報多筆事件.
        let store = TestStore::default();
        ok(
            &store,
            Command::DiscussNew {
                topic: "API auth".to_string(),
                slug: Some("api-auth".to_string()),
            },
        );
        ok(
            &store,
            Command::DiscussConclude {
                slug: "api-auth".to_string(),
                content: "結論：轉正。\n".to_string(),
            },
        );
        let (_, events) = ok(
            &store,
            Command::DiscussPromote { slug: "api-auth".to_string(), name: None },
        );
        assert_eq!(kinds(&events), ["discussion-promoted", "change-created"]);
        match (&events[0], &events[1]) {
            (
                DomainEvent::DiscussionPromoted { slug, change, .. },
                DomainEvent::ChangeCreated { change: created, .. },
            ) => {
                assert_eq!(slug, "api-auth");
                assert_eq!(change, "api-auth");
                assert_eq!(created, "api-auth");
            }
            other => panic!("expected promoted+created, got {other:?}"),
        }
    }

    #[test]
    fn event_kind_table_matches_the_spec_coverage_table() {
        // spec Example 變更型動詞與事件種類對應——17 列逐一斷言（含 execute 中
        // 無成功路徑的 change-claimed：對應表本身是契約）。
        let at = chrono::Utc::now();
        let s = |v: &str| v.to_string();
        let table: Vec<(DomainEvent, &str)> = vec![
            (DomainEvent::ChangeCreated { change: s("c"), occurred_at: at }, "change-created"),
            (
                DomainEvent::ArtifactCreated { change: s("c"), artifact: s("proposal"), occurred_at: at },
                "artifact-created",
            ),
            (
                DomainEvent::TaskCompleted { change: s("c"), task_id: 1, occurred_at: at },
                "task-completed",
            ),
            (
                DomainEvent::TaskUncompleted { change: s("c"), task_id: 1, occurred_at: at },
                "task-uncompleted",
            ),
            (DomainEvent::ChangeClaimed { change: s("c"), occurred_at: at }, "change-claimed"),
            (
                DomainEvent::ChangeMarkedInProgress { change: s("c"), occurred_at: at },
                "change-marked-in-progress",
            ),
            (
                DomainEvent::ChangeArchived { change: s("c"), dated_name: s("2026-07-12-c"), occurred_at: at },
                "change-archived",
            ),
            (DomainEvent::ChangeDiscarded { change: s("c"), occurred_at: at }, "change-discarded"),
            (DomainEvent::DiscussionCreated { slug: s("d"), occurred_at: at }, "discussion-created"),
            (
                DomainEvent::DiscussionContextSet { slug: s("d"), occurred_at: at },
                "discussion-context-set",
            ),
            (
                DomainEvent::DiscussionRoundAdded { slug: s("d"), round: 1, occurred_at: at },
                "discussion-round-added",
            ),
            (
                DomainEvent::DiscussionConcluded { slug: s("d"), occurred_at: at },
                "discussion-concluded",
            ),
            (
                DomainEvent::DiscussionPromoted { slug: s("d"), change: s("c"), occurred_at: at },
                "discussion-promoted",
            ),
            (
                DomainEvent::DiscussionLinked { slug: s("d"), change: s("c"), occurred_at: at },
                "discussion-linked",
            ),
            (
                DomainEvent::DiscussionSealed { slug: s("d"), change: s("c"), occurred_at: at },
                "discussion-sealed",
            ),
            (DomainEvent::DiscussionArchived { slug: s("d"), occurred_at: at }, "discussion-archived"),
            (
                DomainEvent::DiscussionDiscarded { slug: s("d"), occurred_at: at },
                "discussion-discarded",
            ),
        ];
        assert_eq!(table.len(), 17, "the coverage table has 17 mutating verbs");
        for (event, kind) in &table {
            assert_eq!(event.kind(), *kind, "for {event:?}");
        }
    }

    // === ExecutionContext 由 Host 解析且不可覆寫（spec: command 無從攜帶 identity／本地 actor 語意不變） ===

    #[test]
    fn command_inputs_carry_no_actor_or_policy_fields() {
        // Command 封閉 enum 的完整欄位解構：任何 variant 若新增 actor 或
        // policy 欄位，這裡缺欄位的解構就編譯失敗——蓋章身分與政策只能
        // 來自 ExecutionContext，呼叫端與模型無從經 command 參數覆寫。
        let probe = list_cmd();
        match probe {
            Command::List { sort: _, specs: _, changes: _ } => {}
            Command::Show { item: _, item_type: _ } => {}
            Command::Status { change: _, schema: _ } => {}
            Command::Instructions { artifact: _, change: _, schema: _ } => {}
            Command::Validate { item: _, all: _, changes: _, strict: _ } => {}
            Command::Analyze { change: _ } => {}
            Command::Drift { change: _ } => {}
            Command::ArtifactCat { artifact: _, change: _ } => {}
            Command::LanguageShow => {}
            Command::DiscussList { archived: _ } => {}
            Command::DiscussShow { slug: _ } => {}
            Command::NewChange {
                name: _,
                description: _,
                schema: _,
                agent: _,
                from_discussion: _,
            } => {}
            Command::NewArtifact { kind: _, capability: _, change: _, content: _, force: _ } => {}
            Command::TaskDone { task_id: _, change: _ } => {}
            Command::TaskUndone { task_id: _, change: _ } => {}
            Command::Claim { name: _ } => {}
            Command::InProgressAdd { name: _ } => {}
            Command::Archive { change: _, skip_specs: _, no_validate: _, mark_tasks_complete: _ } => {}
            Command::Discard { change: _, force: _ } => {}
            Command::DiscussNew { topic: _, slug: _ } => {}
            Command::DiscussContext { slug: _, content: _ } => {}
            Command::DiscussAddRound { slug: _, mode: _, content: _ } => {}
            Command::DiscussConclude { slug: _, content: _ } => {}
            Command::DiscussPromote { slug: _, name: _ } => {}
            Command::DiscussLink { slug: _, change: _ } => {}
            Command::DiscussSeal { slug: _, change: _ } => {}
            Command::DiscussArchive { slug: _ } => {}
            Command::DiscussDiscard { slug: _, force: _ } => {}
        }
    }

    fn actor_ctx(actor: Option<&str>) -> ExecutionContext {
        ExecutionContext {
            actor: actor.map(str::to_string),
            workspace: Some(ghost_ws()),
            ..Default::default()
        }
    }

    fn new_change_cmd(name: &str) -> Command {
        Command::NewChange {
            name: name.to_string(),
            description: None,
            schema: None,
            agent: None,
            from_discussion: None,
        }
    }

    #[test]
    fn created_by_stamp_follows_context_actor_only() {
        // new change 的 created_by 章只隨 context actor 改變；無身分時
        // 沿用現行無章行為。
        let store = TestStore::default();
        execute(
            &store,
            &actor_ctx(Some("Ctx Actor <ctx@example.com>")),
            new_change_cmd("stamped"),
        )
        .expect("new change succeeds");
        assert!(
            store.meta("stamped").contains("created_by: Ctx Actor <ctx@example.com>\n"),
            "created_by follows the context actor, meta: {}",
            store.meta("stamped")
        );

        execute(&store, &actor_ctx(None), new_change_cmd("anon")).expect("new change succeeds");
        assert!(
            !store.meta("anon").contains("created_by:"),
            "anonymous context keeps the current no-stamp behavior, meta: {}",
            store.meta("anon")
        );
    }

    #[test]
    fn started_by_stamp_follows_context_actor_only() {
        let store = TestStore::with_meta("demo", META);
        execute(
            &store,
            &actor_ctx(Some("Ctx Actor <ctx@example.com>")),
            Command::InProgressAdd { name: "demo".to_string() },
        )
        .expect("in-progress add succeeds");
        assert!(
            store.meta("demo").contains("started_by: Ctx Actor <ctx@example.com>\n"),
            "started_by follows the context actor, meta: {}",
            store.meta("demo")
        );
    }

    #[test]
    fn discussion_created_by_follows_context_actor_only() {
        let store = TestStore::default();
        let (outcome, _) = execute(
            &store,
            &actor_ctx(Some("Ctx Actor <ctx@example.com>")),
            Command::DiscussNew { topic: "Identity probe".to_string(), slug: None },
        )
        .expect("discuss new succeeds");
        match outcome {
            CommandOutcome::DiscussNew(info) => assert_eq!(
                info.created_by.as_deref(),
                Some("Ctx Actor <ctx@example.com>"),
                "the discussion creator stamp follows the context actor"
            ),
            other => panic!("expected a discuss-new outcome, got {other:?}"),
        }
    }
}
