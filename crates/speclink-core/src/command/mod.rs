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

mod typed;
pub use typed::WrongOutcome;

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
    } else if let Some(b) = e.downcast_ref::<crate::inprogress::RevertBlocked>() {
        (ErrorCode::Refused, b.to_string())
    } else if let Some(m) = e.downcast_ref::<crate::model::MetaError>() {
        (ErrorCode::InvalidConfig, m.to_string())
    } else if let Some(n) = e.downcast_ref::<crate::review::NotFound>() {
        (ErrorCode::NotFound, n.to_string())
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
    /// `task_id` is the task's stable ID; undone on an unstamped task falls
    /// back to the ordinal string (undone never stamps).
    TaskCompleted {
        change: String,
        task_id: String,
        /// The files this completion recorded as its evidence. Empty when
        /// nothing was attributable — never a guess.
        touched_files: Vec<String>,
        occurred_at: chrono::DateTime<chrono::Utc>,
    },
    TaskUncompleted { change: String, task_id: String, occurred_at: chrono::DateTime<chrono::Utc> },
    TaskMoved { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    /// Only stores that adjudicate ownership reach this — the fs store's
    /// `claim` still refuses, so a local checkout never produces it.
    ChangeClaimed { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeMarkedInProgress { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeInProgressRemoved { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeArchived { change: String, dated_name: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ChangeDiscarded { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ReviewRoundAdded { change: String, round: usize, occurred_at: chrono::DateTime<chrono::Utc> },
    ReviewStamped { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    ReviewDiscarded { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    VerifyRoundAdded { change: String, round: usize, occurred_at: chrono::DateTime<chrono::Utc> },
    VerifyStamped { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
    VerifyDiscarded { change: String, occurred_at: chrono::DateTime<chrono::Utc> },
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
            DomainEvent::TaskMoved { .. } => "task-moved",
            DomainEvent::ChangeClaimed { .. } => "change-claimed",
            DomainEvent::ChangeMarkedInProgress { .. } => "change-marked-in-progress",
            DomainEvent::ChangeInProgressRemoved { .. } => "change-in-progress-removed",
            DomainEvent::ChangeArchived { .. } => "change-archived",
            DomainEvent::ChangeDiscarded { .. } => "change-discarded",
            DomainEvent::ReviewRoundAdded { .. } => "review-round-added",
            DomainEvent::ReviewStamped { .. } => "review-stamped",
            DomainEvent::ReviewDiscarded { .. } => "review-discarded",
            DomainEvent::VerifyRoundAdded { .. } => "verify-round-added",
            DomainEvent::VerifyStamped { .. } => "verify-stamped",
            DomainEvent::VerifyDiscarded { .. } => "verify-discarded",
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
        /// Local worktree observation facts, keyed by change name. Only the CLI's
        /// fs main-checkout path ever fills this; every other entry point passes
        /// an empty map, whose output is byte-identical to the frozen baseline.
        worktrees: std::collections::BTreeMap<String, crate::listing::ListWorktreeJson>,
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
    /// `validate [item] [--all] [--changes] [--specs] [--strict]`. Target selection
    /// is a union: `--specs` alone validates only the canonical specs, `--all`
    /// validates both sides, and both flags absent keeps the frozen change-only
    /// behavior.
    Validate {
        item: Option<String>,
        all: bool,
        changes: bool,
        specs: bool,
        strict: bool,
    },
    /// `analyze [change]`
    Analyze { change: Option<String> },
    /// `trace <capability>`
    Trace { capability: String },
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
    /// `discuss search <keyword>...` — keywords are matched any-of, case-insensitive.
    DiscussSearch { terms: Vec<String> },
    // --- 變更群 ---
    /// `new change <name> [--description] [--schema] [--agent] [--from-discussion]`
    NewChange {
        name: String,
        description: Option<String>,
        schema: Option<String>,
        agent: Option<String>,
        from_discussion: Option<String>,
    },
    /// `new artifact <type> [capability] [--change <name>] [--force] [--new]`;
    /// `content` is the CLI's `--stdin` payload, `new_capability` the `--new`
    /// confirmation for a canonically-unlisted spec capability.
    NewArtifact {
        kind: String,
        capability: Option<String>,
        change: Option<String>,
        content: Option<String>,
        force: bool,
        new_capability: bool,
    },
    /// `task done <task_id> [--change <name>]` (`task_id` stays the raw argv
    /// token — validation and its frozen messages live in the runtime).
    /// `touched_files` is the Host-resolved touched-file candidate list, not an
    /// argv surface: `Some` means the Host already resolved it at its boundary
    /// (the server route's wire request), `None` means fall back to probing the
    /// local workspace (design 決策一). `head_commit` rides along the same way:
    /// the commit the sender observed its candidates on, absent when unreported.
    TaskDone {
        task_id: String,
        change: Option<String>,
        touched_files: Option<Vec<String>>,
        head_commit: Option<String>,
    },
    /// `task undone <task_id> [--change <name>]`
    TaskUndone {
        task_id: String,
        change: Option<String>,
    },
    /// 任務搬移（無 CLI argv 形；desktop 拖排與 server 端點的共用動詞）。
    /// `from`/`to` 為 1-based checkbox ordinal、`before` 為可省略側別，
    /// 鏡射 UI moveTask 簽名（design 決策 4）。
    TaskMove {
        change: String,
        from: usize,
        to: usize,
        before: Option<bool>,
    },
    /// `claim <name>` — remote-store only; the plain-store path refuses.
    Claim { name: String },
    /// `in-progress add <name>` — silent and idempotent (unknown names included).
    InProgressAdd { name: String },
    /// `in-progress remove <name>` — the reverse verb, gated on zero work
    /// traces; unknown names error loudly (deliberately asymmetric with add).
    InProgressRemove { name: String },
    /// `archive [change] [--skip-specs] [--no-validate] [--mark-tasks-complete]`
    /// (single change; the CLI's `--all`/bulk loop stays in the entry point).
    Archive {
        change: Option<String>,
        skip_specs: bool,
        no_validate: bool,
        mark_tasks_complete: bool,
        carry_review: bool,
        carry_verify: bool,
    },
    /// `discard <change> [--force]`
    Discard { change: String, force: bool },
    /// `discuss new <topic> [--slug <slug>] [--kind <kind>]`
    DiscussNew { topic: String, slug: Option<String>, kind: Option<String> },
    /// `discuss context <slug>` with stdin content
    DiscussContext { slug: String, content: String },
    /// `discuss add-round <slug> --mode <mode>` with stdin content
    DiscussAddRound {
        slug: String,
        mode: String,
        content: String,
    },
    /// `discuss conclude <slug> [--hold]` with stdin content
    DiscussConclude { slug: String, content: String, hold: bool },
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
    /// `review add-round <change> --stdin`
    ReviewAddRound { change: String, content: String },
    /// `review show <change>`（查詢——不產生事件）
    ReviewShow { change: String },
    /// `review stamp <change> [--accept] [--agent]`。`scope` 為工作樹持有者
    /// 預算的指紋清單（design D4a）：server 無工作樹，唯一蓋章路徑是提交
    /// 端算好 (path, hash) 上 wire；`missing` 明示宣告聯集中已不存在的檔，
    /// 分割「scope ∪ missing ＝工單聯集且不相交」不成立即拒。
    ReviewStamp {
        change: String,
        accept: bool,
        tool: Option<String>,
        scope: Vec<crate::model::ReviewedScopeEntry>,
        missing: Vec<String>,
    },
    /// `review discard <change>`
    ReviewDiscard { change: String },
    /// `verify add-round <change> --stdin`（引擎守門：任務未全完成即拒絕）
    VerifyAddRound { change: String, content: String },
    /// `verify show <change>`（查詢——不產生事件）
    VerifyShow { change: String },
    /// `verify stamp <change> [--accept] [--agent]`：`scope`／`missing` 的分割
    /// 語意與 [`Command::ReviewStamp`] 完全相同（design D4a 的驗證面）。
    VerifyStamp {
        change: String,
        accept: bool,
        tool: Option<String>,
        scope: Vec<crate::model::ReviewedScopeEntry>,
        missing: Vec<String>,
    },
    /// `verify discard <change>`
    VerifyDiscard { change: String },
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
/// one unit: unless BOTH are present, neither is reported (frozen output shape).
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
/// `task_id` is the resolved 1-based ordinal the mutation used. `stable_id`
/// is the task's stable ID when it has one (done stamps its target, so only
/// undone on an unstamped task leaves it None). `already` = nothing changed
/// (zero file effects) — presentation stays with the entry point.
#[derive(Debug)]
pub struct TaskFlipOutcome {
    pub change: String,
    pub task_id: usize,
    pub task_id_arg: String,
    pub description: String,
    pub already: bool,
    pub stable_id: Option<String>,
    /// Files this completion recorded as evidence (empty for undone and for a
    /// completion with nothing new to attribute).
    pub touched_files: Vec<String>,
}

/// `task move` outcome: the subject change and the moved task's cleaned
/// description after the move (prefixes already renumbered).
#[derive(Debug)]
pub struct TaskMoveOutcome {
    pub change: String,
    pub description: String,
}

/// `claim` outcome: the change and its owner after the call, plus whether THIS
/// call stamped it (false for the same-actor idempotent pass — no event then).
#[derive(Debug)]
pub struct ClaimOutcome {
    pub name: String,
    pub claimed_by: Option<String>,
    pub claimed: bool,
}

/// `in-progress add` outcome: whether this call stamped the marker (false for
/// the idempotent/unknown-name silent successes — no event then).
#[derive(Debug)]
pub struct InProgressOutcome {
    pub name: String,
    pub stamped: bool,
}

/// `in-progress remove` outcome: whether this call removed the marker (false
/// for the idempotent not-started success — no event then).
#[derive(Debug)]
pub struct InProgressRemoveOutcome {
    pub name: String,
    pub removed: bool,
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

/// `discuss conclude` outcome: changes flagged stale by a re-conclude, whether the
/// closing step auto-archived the record (its spun-out changes were all archived),
/// and the closing step's failure reason when it could not archive — the conclusion
/// itself is committed either way.
#[derive(Debug)]
pub struct DiscussConcludeOutcome {
    pub slug: String,
    pub restale_flagged: Vec<String>,
    pub auto_archived: bool,
    pub closing_error: Option<String>,
    /// Whether the record carries the hold flag after this write.
    pub held: bool,
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

/// `review add-round` outcome.
#[derive(Debug)]
pub struct ReviewRoundOutcome {
    pub change: String,
    pub round: usize,
}

/// `review show` outcome. `content` is the ticket document verbatim — the
/// human-readable path prints it, so every caller renders the same text
/// instead of reassembling it from the structured rounds. `show` has already
/// verified the ticket exists, so it is only ever absent for a store that
/// dropped the document between the two reads.
#[derive(Debug)]
pub struct ReviewShowOutcome {
    pub change: String,
    pub ticket: crate::review::Ticket,
    pub content: Option<String>,
}

/// `review stamp` / `review discard` outcome (subject only).
#[derive(Debug)]
pub struct ReviewSubjectOutcome {
    pub change: String,
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
    Trace(crate::trace::TraceReport),
    /// Raw artifact content (`artifact cat`).
    ArtifactCat(String),
    /// Raw LANGUAGE document content (`language show`).
    Language(String),
    DiscussList(Vec<crate::discuss::DiscussionInfo>),
    DiscussShow(DiscussShowOutcome),
    /// `discuss search` hits, already in the spec's order.
    DiscussSearch(Vec<crate::discuss::DiscussionHit>),
    NewChange(NewChangeOutcome),
    NewArtifact(NewArtifactOutcome),
    TaskDone(TaskFlipOutcome),
    TaskUndone(TaskFlipOutcome),
    TaskMove(TaskMoveOutcome),
    Claim(ClaimOutcome),
    InProgressAdd(InProgressOutcome),
    InProgressRemove(InProgressRemoveOutcome),
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
    ReviewAddRound(ReviewRoundOutcome),
    ReviewShow(ReviewShowOutcome),
    ReviewStamp(ReviewSubjectOutcome),
    ReviewDiscard(ReviewSubjectOutcome),
    /// 驗證站的四個結果沿用審查站的 outcome 型別——工單形狀站別無關，
    /// 差異只在寫哪份文件（[`crate::station`] 的常數組承載）。
    VerifyAddRound(ReviewRoundOutcome),
    VerifyShow(ReviewShowOutcome),
    VerifyStamp(ReviewSubjectOutcome),
    VerifyDiscard(ReviewSubjectOutcome),
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
    /// Repo binding key resolved at the Host boundary (local fs mode: the
    /// default binding). Recorded in completion evidence; `None` stays absent.
    pub repo: Option<String>,
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
        Command::List { sort, specs, changes, worktrees } => {
            run_list(store, &sort, specs, changes, &worktrees)
        }
        Command::Show { item, item_type } => run_show(store, item.as_deref(), item_type.as_deref()),
        Command::Status { change, schema } => {
            run_status(store, ws, ctx.user_config_dir.as_deref(), change.as_deref(), schema.as_deref())
        }
        Command::Instructions { artifact, change, schema } => {
            run_instructions(store, ws, ctx.user_config_dir.as_deref(), &ctx.env, artifact.as_deref(), change.as_deref(), schema.as_deref())
        }
        Command::Validate { item, all, changes, specs, strict } => {
            run_validate(store, item.as_deref(), all, changes, specs, strict)
        }
        Command::Analyze { change } => run_analyze(store, change.as_deref()),
        Command::Trace { capability } => run_trace(store, &capability),
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
        Command::DiscussSearch { terms } => crate::discuss::search(store, &terms)
            .map(CommandOutcome::DiscussSearch)
            // The engine's only refusal here is an empty/blank keyword list —
            // an argv defect (the server maps it to 400 invalid_argument).
            .map_err(|e| CommandError::new(ErrorCode::InvalidArgv, e.to_string())),
        Command::NewChange { name, description, schema, agent, from_discussion } => {
            run_new_change(store, ctx.actor.as_deref(), name, description, schema, agent, from_discussion)
        }
        Command::NewArtifact { kind, capability, change, content, force, new_capability } => {
            run_new_artifact(store, ws, ctx.user_config_dir.as_deref(), &kind, capability.as_deref(), change.as_deref(), content.as_deref(), force, new_capability)
        }
        Command::TaskDone { task_id, change, touched_files, head_commit } => {
            run_task_flip(
                store,
                ws,
                ctx,
                &task_id,
                change.as_deref(),
                TaskFlip::Done { touched_files, head_commit },
            )
        }
        Command::TaskUndone { task_id, change } => {
            run_task_flip(store, ws, ctx, &task_id, change.as_deref(), TaskFlip::Undone)
        }
        Command::TaskMove { change, from, to, before } => {
            run_task_move(store, &change, from, to, before)
        }
        Command::Claim { name } => run_claim(store, ctx.actor.as_deref(), &name),
        Command::InProgressAdd { name } => run_in_progress_add(store, ctx.actor.as_deref(), &name),
        Command::InProgressRemove { name } => run_in_progress_remove(store, &name),
        Command::Archive { change, skip_specs, no_validate, mark_tasks_complete, carry_review, carry_verify } => run_archive(
            store,
            ws,
            ctx.actor.as_deref(),
            change.as_deref(),
            crate::archive::ArchiveOptions { skip_specs, no_validate, mark_tasks_complete, carry_review, carry_verify },
        ),
        Command::Discard { change, force } => run_discard(store, ws, &change, force),
        Command::DiscussNew { topic, slug, kind } => {
            run_discuss_new(store, ctx.actor.as_deref(), &topic, slug.as_deref(), kind.as_deref())
        }
        Command::DiscussContext { slug, content } => {
            crate::discuss::set_context(store, &slug, &content).map_err(classify)?;
            Ok(CommandOutcome::DiscussContext(DiscussSubjectOutcome { slug }))
        }
        Command::DiscussAddRound { slug, mode, content } => {
            let round = crate::discuss::add_round(store, &slug, &mode, &content).map_err(classify)?;
            Ok(CommandOutcome::DiscussAddRound(DiscussRoundOutcome { slug, mode, round }))
        }
        Command::DiscussConclude { slug, content, hold } => {
            let outcome =
                crate::discuss::conclude(store, &slug, &content, hold).map_err(classify)?;
            Ok(CommandOutcome::DiscussConclude(DiscussConcludeOutcome {
                slug,
                restale_flagged: outcome.restale_flagged,
                auto_archived: outcome.auto_archived,
                closing_error: outcome.closing_error,
                held: outcome.held,
            }))
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
        Command::ReviewAddRound { change, content } => {
            let round = crate::review::add_round(store, &change, &content).map_err(classify)?;
            Ok(CommandOutcome::ReviewAddRound(ReviewRoundOutcome { change, round }))
        }
        Command::ReviewShow { change } => {
            let (ticket, content) =
                crate::review::show_with_content(store, &change).map_err(classify)?;
            Ok(CommandOutcome::ReviewShow(ReviewShowOutcome { change, ticket, content }))
        }
        Command::ReviewStamp { change, accept, tool, scope, missing } => {
            crate::review::stamp_with_scope(
                store,
                &change,
                accept,
                ctx.actor.as_deref(),
                tool.as_deref(),
                scope,
                missing,
            )
            .map_err(classify)?;
            Ok(CommandOutcome::ReviewStamp(ReviewSubjectOutcome { change }))
        }
        Command::ReviewDiscard { change } => {
            crate::review::discard(store, &change).map_err(classify)?;
            Ok(CommandOutcome::ReviewDiscard(ReviewSubjectOutcome { change }))
        }
        Command::VerifyAddRound { change, content } => {
            let round = crate::verify::add_round(store, &change, &content).map_err(classify)?;
            Ok(CommandOutcome::VerifyAddRound(ReviewRoundOutcome { change, round }))
        }
        Command::VerifyShow { change } => {
            let (ticket, content) =
                crate::verify::show_with_content(store, &change).map_err(classify)?;
            Ok(CommandOutcome::VerifyShow(ReviewShowOutcome { change, ticket, content }))
        }
        Command::VerifyStamp { change, accept, tool, scope, missing } => {
            crate::verify::stamp_with_scope(
                store,
                &change,
                accept,
                ctx.actor.as_deref(),
                tool.as_deref(),
                scope,
                missing,
            )
            .map_err(classify)?;
            Ok(CommandOutcome::VerifyStamp(ReviewSubjectOutcome { change }))
        }
        Command::VerifyDiscard { change } => {
            crate::verify::discard(store, &change).map_err(classify)?;
            Ok(CommandOutcome::VerifyDiscard(ReviewSubjectOutcome { change }))
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
        | CommandOutcome::Trace(_)
        | CommandOutcome::ArtifactCat(_)
        | CommandOutcome::Language(_)
        | CommandOutcome::DiscussList(_)
        | CommandOutcome::DiscussShow(_)
        | CommandOutcome::DiscussSearch(_) => Vec::new(),
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
            task_id: o.stable_id.clone().unwrap_or_else(|| o.task_id.to_string()),
            touched_files: o.touched_files.clone(),
            occurred_at: at,
        }],
        CommandOutcome::TaskUndone(o) if o.already => Vec::new(),
        CommandOutcome::TaskUndone(o) => vec![DomainEvent::TaskUncompleted {
            change: o.change.clone(),
            task_id: o.stable_id.clone().unwrap_or_else(|| o.task_id.to_string()),
            occurred_at: at,
        }],
        CommandOutcome::TaskMove(o) => vec![DomainEvent::TaskMoved {
            change: o.change.clone(),
            occurred_at: at,
        }],
        CommandOutcome::Claim(o) if !o.claimed => Vec::new(),
        CommandOutcome::Claim(o) => vec![DomainEvent::ChangeClaimed {
            change: o.name.clone(),
            occurred_at: at,
        }],
        CommandOutcome::InProgressAdd(o) if !o.stamped => Vec::new(),
        CommandOutcome::InProgressAdd(o) => vec![DomainEvent::ChangeMarkedInProgress {
            change: o.name.clone(),
            occurred_at: at,
        }],
        CommandOutcome::InProgressRemove(o) if !o.removed => Vec::new(),
        CommandOutcome::InProgressRemove(o) => vec![DomainEvent::ChangeInProgressRemoved {
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
        CommandOutcome::ReviewShow(_) => Vec::new(),
        CommandOutcome::ReviewAddRound(o) => vec![DomainEvent::ReviewRoundAdded {
            change: o.change.clone(),
            round: o.round,
            occurred_at: at,
        }],
        CommandOutcome::ReviewStamp(o) => vec![DomainEvent::ReviewStamped {
            change: o.change.clone(),
            occurred_at: at,
        }],
        CommandOutcome::ReviewDiscard(o) => vec![DomainEvent::ReviewDiscarded {
            change: o.change.clone(),
            occurred_at: at,
        }],
        CommandOutcome::VerifyShow(_) => Vec::new(),
        CommandOutcome::VerifyAddRound(o) => vec![DomainEvent::VerifyRoundAdded {
            change: o.change.clone(),
            round: o.round,
            occurred_at: at,
        }],
        CommandOutcome::VerifyStamp(o) => vec![DomainEvent::VerifyStamped {
            change: o.change.clone(),
            occurred_at: at,
        }],
        CommandOutcome::VerifyDiscard(o) => vec![DomainEvent::VerifyDiscarded {
            change: o.change.clone(),
            occurred_at: at,
        }],
    }
}

/// Specify-wording of the multi-change auto-detect error: flag-style verbs.
const SPECIFY_FLAG: &str = "Use --change to specify one:";
/// Positional-style verbs (analyze, drift) say just this (frozen wording).
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
    worktrees: &std::collections::BTreeMap<String, crate::listing::ListWorktreeJson>,
) -> Result<CommandOutcome, CommandError> {
    // --specs alone omits the changes section; combined with --changes both appear.
    let changes = if specs && !changes_flag {
        None
    } else {
        let mut changes = crate::model::list_changes(store);
        crate::listing::sort_changes(store, &mut changes, sort);
        Some(crate::listing::changes_json_with(store, &changes, worktrees))
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
    // every artifact exists (frozen behavior).
    let default_artifact = crate::status::first_incomplete_artifact(store, &change, &schema)
        .unwrap_or_else(|| "apply".to_string());
    let artifact = artifact.unwrap_or(&default_artifact);
    if artifact == "apply" {
        let host = host_workspace(ws);
        let payload = crate::instructions::build_apply(&host, store, env, &change, &schema)?;
        return Ok(CommandOutcome::Instructions(InstructionsOutcome::Apply(payload)));
    }
    let payload = crate::instructions::build_artifact(store, env, &change, &schema, artifact)?
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
    specs_flag: bool,
    strict: bool,
) -> Result<CommandOutcome, CommandError> {
    // 目標集由 validate 的單一旗標語意解出（design D4），remote 分流讀同一支；
    // --specs 與 item 同傳在此以參數錯誤拒絕。
    let targets = crate::validate::validate_targets(item, all, changes_flag, specs_flag)
        .map_err(|m| CommandError::new(ErrorCode::InvalidArgv, m))?;
    if !targets.changes {
        return Ok(CommandOutcome::Validate(ValidateOutcome {
            results: crate::validate::validate_specs(store, strict),
        }));
    }
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
        // back to validating everything (frozen behavior).
        match resolve_change(store, None, SPECIFY_FLAG) {
            Ok(c) => {
                guard_meta(&c)?;
                vec![c]
            }
            Err(_) => crate::model::list_changes(store),
        }
    };
    // Multi-change runs are ordered newest-modified first (frozen ordering).
    crate::listing::sort_changes(store, &mut changes, "modified");
    // validate never resolves the change's schema (an unresolvable
    // one still validates).
    let schema = crate::schema::spec_driven();
    let mut results: Vec<crate::validate::ValidationResult> = changes
        .iter()
        .map(|c| crate::validate::validate_change(store, c, &schema, strict))
        .collect();
    if targets.specs {
        results.extend(crate::validate::validate_specs(store, strict));
    }
    Ok(CommandOutcome::Validate(ValidateOutcome { results }))
}

fn run_analyze(store: &dyn Store, change: Option<&str>) -> Result<CommandOutcome, CommandError> {
    let change = resolve_change(store, change, SPECIFY_POSITIONAL)?;
    guard_meta(&change)?;
    // The analyzer is schema-agnostic and never resolves the change's schema.
    let schema = crate::schema::spec_driven();
    Ok(CommandOutcome::Analyze(crate::analyzer::analyze(
        store, &change, &schema,
    )))
}

fn run_trace(store: &dyn Store, capability: &str) -> Result<CommandOutcome, CommandError> {
    Ok(CommandOutcome::Trace(
        crate::trace::run(store, capability).map_err(classify)?,
    ))
}

/// Resolve and meta-guard a change for the drift verb, which the CLI now
/// orchestrates outside `execute` (Host collects the workspace facts, the
/// Engine computes each side, the merger assembles the report). Uses the same
/// positional resolution and fail-closed meta guard the engine query verbs do,
/// so drift's not-found / ambiguous / corrupt-meta behaviour is unchanged.
pub fn resolve_guarded_change(
    store: &dyn Store,
    change: Option<&str>,
) -> Result<Change, CommandError> {
    let change = resolve_change(store, change, SPECIFY_POSITIONAL)?;
    guard_meta(&change)?;
    Ok(change)
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
    // here (downstream commands fail on resolution).
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
    new_capability: bool,
) -> Result<CommandOutcome, CommandError> {
    let type_ok = ["proposal", "design", "tasks", "spec"].contains(&kind);
    let type_err = || {
        CommandError::new(
            ErrorCode::InvalidArgv,
            format!("Unknown artifact type '{kind}'. Valid types: proposal, design, tasks, spec"),
        )
    };
    // Frozen order: with an explicit --change, validate the type before
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
    // creates the artifact (no template → empty file) — frozen behavior.
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
        crate::newcmd::new_artifact(store, &change, &schema, kind, capability, content, force, new_capability)
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
    /// Completion, carrying the Host-resolved touched-file candidates (`None`
    /// = probe the local workspace instead) and the commit they were observed
    /// on (absent when the sender did not report one).
    Done {
        touched_files: Option<Vec<String>>,
        head_commit: Option<String>,
    },
    Undone,
}

fn run_task_flip(
    store: &dyn Store,
    ws: Option<&Workspace>,
    ctx: &ExecutionContext,
    task_id: &str,
    change: Option<&str>,
    flip: TaskFlip,
) -> Result<CommandOutcome, CommandError> {
    // `task done`/`task undone` do not require the change to exist — they go
    // straight to tasks.md, and its existence is checked BEFORE the id
    // (frozen order).
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
    // Closed dual value domain: pure digits → ordinal (frozen behavior),
    // tsk_ prefix → stable-ID lookup, anything else refuses.
    let addr = if task_id.starts_with("tsk_") {
        crate::tasks::TaskAddr::Stable(task_id.to_string())
    } else {
        let id: usize = task_id.parse().map_err(|_| {
            CommandError::new(
                ErrorCode::InvalidArgv,
                format!("Invalid task ID '{task_id}': must be a number or a tsk_-prefixed stable ID"),
            )
        })?;
        if id < 1 {
            return Err(CommandError::new(ErrorCode::InvalidArgv, "Task ID must be >= 1"));
        }
        crate::tasks::TaskAddr::Ordinal(id)
    };
    let host = host_workspace(ws);
    let outcome_of = |description, already, ordinal, stable_id, touched_files| TaskFlipOutcome {
        change: change_name.clone(),
        task_id: ordinal,
        task_id_arg: task_id.to_string(),
        description,
        already,
        stable_id,
        touched_files,
    };
    Ok(match flip {
        TaskFlip::Done { touched_files, head_commit } => {
            let candidates = match &touched_files {
                Some(files) => crate::tasks::TouchedCandidates::Injected {
                    files,
                    head_commit: head_commit.as_deref(),
                },
                None => crate::tasks::TouchedCandidates::ProbeWorkspace(&host),
            };
            let o = crate::tasks::complete(
                store,
                &change_name,
                &addr,
                &crate::tasks::CompleteAttribution {
                    identity: ctx.actor.as_deref(),
                    agent: None,
                    repo: ctx.repo.as_deref(),
                },
                candidates,
            )
            .map_err(classify)?;
            CommandOutcome::TaskDone(outcome_of(
                o.description,
                o.already,
                o.ordinal,
                o.stable_id,
                o.touched_files,
            ))
        }
        TaskFlip::Undone => {
            let o = crate::tasks::uncomplete(store, &change_name, &addr).map_err(classify)?;
            CommandOutcome::TaskUndone(outcome_of(
                o.description,
                o.already,
                o.ordinal,
                o.stable_id,
                Vec::new(),
            ))
        }
    })
}

fn run_task_move(
    store: &dyn Store,
    change: &str,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<CommandOutcome, CommandError> {
    // tasks.md 的存在先於索引檢查（與 task done/undone 同序）。
    if !store.artifact_exists(change, "tasks.md") {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("tasks.md not found for change '{change}'"),
        ));
    }
    let o = crate::tasks::move_task(store, change, from, to, before).map_err(classify)?;
    Ok(CommandOutcome::TaskMove(TaskMoveOutcome {
        change: change.to_string(),
        description: o.description,
    }))
}

/// `claim` — ownership adjudication, split on what the backend can do (design
/// D2). A plain fs store has nobody to coordinate with and refuses with the
/// frozen text; a team-mode store stamps `claimed_at` / `claimed_by` into the
/// change metadata the same read-append-write way the started stamp does.
fn run_claim(
    store: &dyn Store,
    actor: Option<&str>,
    name: &str,
) -> Result<CommandOutcome, CommandError> {
    // Fail-closed gate first: claiming a change whose metadata is corrupt must
    // name the broken file, not the missing remote store.
    if let Some(change) = crate::model::find_change(store, name) {
        guard_meta(&change)?;
    }
    if !store.supports_ownership() {
        return Err(CommandError::new(
            ErrorCode::Error,
            "claim requires a remote store — this project uses the local fs store",
        ));
    }
    let Some(mut meta) = store.read_change_meta(name) else {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("Change '{name}' not found."),
        ));
    };
    // An owner is the whole point of the verb: with nobody to record, a stamp
    // would make the change unclaimable while naming no one to coordinate with.
    let Some(actor) = actor else {
        return Err(CommandError::new(
            ErrorCode::Refused,
            format!("cannot claim '{name}': no identity to record as its owner"),
        ));
    };
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        classify(crate::model::MetaError { change: name.to_string(), reason }.into())
    })?;
    // The stamp is a pair: both fields are written together, so both are judged
    // together. A meta carrying `claimed_at` alone is inconsistent — appending
    // the pair again would leave a duplicate key and make the change
    // permanently unparseable, so the half stamp refuses instead.
    match (parsed.claimed_by, parsed.claimed_at) {
        (Some(holder), _) if holder == actor => {
            return Ok(CommandOutcome::Claim(ClaimOutcome {
                name: name.to_string(),
                claimed_by: Some(holder),
                claimed: false,
            }));
        }
        (Some(holder), _) => {
            return Err(CommandError::new(
                ErrorCode::Refused,
                format!(
                    "change '{name}' is already claimed by {holder} — coordinate with them, or ask them to release it"
                ),
            ));
        }
        (None, Some(_)) => {
            return Err(CommandError::new(
                ErrorCode::Refused,
                format!(
                    "cannot claim '{name}': its metadata carries claimed_at with no claimed_by — restore or remove that line in openspec/changes/{name}/.openspec.yaml"
                ),
            ));
        }
        (None, None) => {}
    }
    if !meta.ends_with('\n') && !meta.is_empty() {
        meta.push('\n');
    }
    meta.push_str(&format!("claimed_at: {}\n", crate::util::today()));
    meta.push_str(&format!("claimed_by: {}\n", crate::util::yaml_scalar(actor)));
    store.write_change_meta(name, &meta).map_err(classify)?;
    Ok(CommandOutcome::Claim(ClaimOutcome {
        name: name.to_string(),
        claimed_by: Some(actor.to_string()),
        claimed: true,
    }))
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

fn run_in_progress_remove(
    store: &dyn Store,
    name: &str,
) -> Result<CommandOutcome, CommandError> {
    // Loud not-found before the flow (the engine errors too — this classifies
    // it under the stable code the entry points key on).
    if crate::model::find_change(store, name).is_none() {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("Change '{name}' not found."),
        ));
    }
    let removed = crate::inprogress::remove(store, name).map_err(classify)?;
    Ok(CommandOutcome::InProgressRemove(InProgressRemoveOutcome {
        name: name.to_string(),
        removed,
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
    let host = host_workspace(ws);
    crate::archive::guard_linked_worktree(&host).map_err(classify)?;
    guard_meta(&change)?;
    crate::archive::guard_open_tickets(store, &change.name, opts.carry_review, opts.carry_verify)
        .map_err(classify)?;
    // The merge gate too: a refused archive must leave tasks.md untouched
    // (spec archive-merge「兩階段合併計畫與零半套寫入」). Pure Store reads.
    if !opts.skip_specs {
        let violations = crate::archive::merge_violations(store, &change.name);
        if !violations.is_empty() {
            return Err(classify(crate::archive::merge_refusal(&change.name, &violations)));
        }
    }
    if opts.mark_tasks_complete {
        // 章失效守門先於前置全勾寫入(同上方諸守門的拒絕路徑零寫入):stale
        // 拒絕不得留下被代勾的 [M] 任務。無旗標路徑不在此判——維持任務完成度
        // 守門先拒的順序契約;核心 archive 內的同一守門供直接呼叫的入口沿用。
        crate::archive::guard_stale_stamps(&host, store, &change).map_err(classify)?;
        if let Some(text) = store.read_artifact(&change.name, "tasks.md") {
            // Star-bullet checkboxes are tasks too (frozen rule).
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
    // The in-progress marker stays untouched on archive (frozen behavior).
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
    kind: Option<&str>,
) -> Result<CommandOutcome, CommandError> {
    let info = crate::discuss::new_discussion(store, topic, slug, actor, kind).map_err(classify)?;
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
            worktrees: Default::default(),
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
                specs: false,
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

    // --- validate 的 --specs／--all 旗標語意（design D4；spec spec-validation
    //     「validate --specs 驗證正典規格」）---

    /// 一個 change（demo）＋一份 Purpose 缺席的正典規格（auth）的專案。
    fn validate_flags_store() -> TestStore {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
        store.canonical.borrow_mut().insert(
            "auth".to_string(),
            "# auth Specification\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n"
                .to_string(),
        );
        store
    }

    fn validated_names(store: &TestStore, all: bool, changes: bool, specs: bool) -> Vec<String> {
        let (outcome, _) = execute(
            store,
            &ExecutionContext::default(),
            Command::Validate { item: None, all, changes, specs, strict: false },
        )
        .expect("validate executes");
        let CommandOutcome::Validate(v) = outcome else { panic!("expected a validate outcome") };
        v.results.iter().map(|r| r.change.clone()).collect()
    }

    #[test]
    fn validate_specs_flag_alone_validates_only_specs() {
        // spec：`--specs` 單獨傳入時僅驗規格。
        let store = validate_flags_store();
        assert_eq!(validated_names(&store, false, false, true), vec!["auth".to_string()]);
    }

    #[test]
    fn validate_all_validates_changes_and_specs() {
        // spec：`--all` 同時驗 changes 與 specs。
        let store = validate_flags_store();
        assert_eq!(
            validated_names(&store, true, false, false),
            vec!["demo".to_string(), "auth".to_string()]
        );
        // `--specs --changes` 同傳的聯集語意與 `--all` 等效（design 風險項）。
        assert_eq!(
            validated_names(&store, false, true, true),
            vec!["demo".to_string(), "auth".to_string()]
        );
    }

    #[test]
    fn validate_without_flags_still_validates_only_changes() {
        // spec Scenario「預設行為不變」：兩旗標皆缺席時只驗 changes。
        let store = validate_flags_store();
        assert_eq!(validated_names(&store, false, false, false), vec!["demo".to_string()]);
        assert_eq!(validated_names(&store, false, true, false), vec!["demo".to_string()]);
    }

    #[test]
    fn validate_item_with_specs_flag_is_rejected() {
        // spec Scenario「--specs 與 change 名稱同傳被拒」：--specs 驗的是正典
        // 規格、無法指定單一規格，與名稱同傳是參數錯誤——大聲拒絕，不做聯集。
        let store = validate_flags_store();
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::Validate {
                item: Some("demo".to_string()),
                all: false,
                changes: false,
                specs: true,
                strict: false,
            },
        )
        .expect_err("--specs with an item must refuse");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
        assert!(err.message.contains("--specs"), "the flag is named: {}", err.message);
        assert!(err.message.contains("--all"), "the way out is named: {}", err.message);
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
                    specs: false,
                    strict: false,
                },
            ),
            ("analyze", Command::Analyze { change: Some("demo".to_string()) }),
            // drift no longer routes through `execute` — it is CLI-orchestrated
            // (Host-collected facts → compute_* → merge); its corrupt-meta
            // fail-closed is covered at the CLI layer (meta_fail_closed).
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
            Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
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
    fn archive_on_open_review_ticket_refuses_before_marking_tasks_complete() {
        // 未結工單守門與壞 metadata 守門同級：拒絕路徑不得留下副作用。
        // --mark-tasks-complete 的前置寫入排在守門之前的話，被拒的封存仍會把
        // tasks.md 全勾成完成——任務狀態被污染且不回滾。
        const TASKS: &str = "- [ ] 1.1 open\n";
        const TICKET: &str = "# Review — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n";
        let store = TestStore::with_meta("demo", "schema: spec-driven\n");
        store.put_artifact("demo", "tasks.md", TASKS);
        store.put_artifact("demo", crate::review::REVIEW_DOC, TICKET);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::Archive {
                change: Some("demo".to_string()),
                skip_specs: true,
                no_validate: true,
                mark_tasks_complete: true,
                carry_review: false,
                carry_verify: false,
            },
        )
        .expect_err("open review ticket must refuse archive");
        assert!(err.message.contains("--carry-review"), "three disposals listed: {}", err.message);
        assert_eq!(
            store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
            TASKS,
            "tasks.md must stay byte-identical when the gate refuses"
        );
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
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
                    carry_review: false,
                carry_verify: false,
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
                new_capability: false,
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

    #[test]
    fn newcmd_gate_keeps_the_change_not_found_error() {
        // spec Scenario「change 不存在時維持既有錯誤」：主閘不得改變
        // 找不到 change 的錯誤碼與訊息。
        let store = TestStore::with_meta("demo", META);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::NewArtifact {
                kind: "spec".to_string(),
                capability: Some("brand-new-cap".to_string()),
                change: Some("no-such-change".to_string()),
                content: Some("## ADDED Requirements\n\n### Requirement: R1\n\nOk.\n".to_string()),
                force: false,
                new_capability: false,
            },
        )
        .expect_err("missing change must fail");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "Change 'no-such-change' not found", "frozen CLI text");
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
                specs: false,
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
    fn discuss_search_is_a_query_that_returns_hits_and_no_events() {
        // Spec「動詞覆蓋與跨入口一致性」：discuss search 為唯讀查詢動詞，不發領域事件。
        let store = TestStore::with_live_discussion(
            "drawer-scope",
            "---\ntopic: Drawer scope\nslug: drawer-scope\nstatus: open\ncreated: 2026-07-01\n---\n\n\
             ## Rounds\n\n### Round 1 — interview (2026-07-01)\n\n**Ruled out**: drawer flag\n\n## Conclusion\n",
        );
        let (outcome, events) =
            ok(&store, Command::DiscussSearch { terms: vec!["drawer".to_string()] });
        let hits: Vec<crate::discuss::DiscussionHit> =
            outcome.try_into().expect("search outcome carries hits");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].info.slug, "drawer-scope");
        assert_eq!(hits[0].matches.len(), 3, "topic, slug and the ruled-out line");
        assert!(events.is_empty(), "search is a query and never produces events");

        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::DiscussSearch { terms: vec![" ".to_string()] },
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidArgv, "blank keywords are refused: {err}");
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
                new_capability: false,
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
                touched_files: None,
                head_commit: None,
            },
        );
        assert_eq!(kinds(&events), ["task-completed"]);
        match &events[0] {
            DomainEvent::TaskCompleted { change, task_id, .. } => {
                assert_eq!(change, "demo");
                assert!(task_id.starts_with("tsk_"), "event carries the stamped stable id: {task_id}");
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
                touched_files: None,
                head_commit: None,
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

    // --- 蓋章時機（spec task-identity: 產出全檔蓋章、task done 單行補章）---

    #[test]
    fn new_artifact_tasks_stamps_every_task_line() {
        let store = TestStore::with_meta("demo", META);
        ok(
            &store,
            Command::NewArtifact {
                kind: "tasks".to_string(),
                capability: None,
                change: Some("demo".to_string()),
                content: Some(
                    "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n- [x] 1.3 third\n"
                        .to_string(),
                ),
                force: false,
                new_capability: false,
            },
        );
        let written = store.read_artifact("demo", "tasks.md").unwrap();
        let tasks = crate::tasks::parse(&written);
        assert_eq!(tasks.len(), 3);
        assert!(tasks.iter().all(|t| t.stable_id.is_some()), "every task carries an id: {written}");
        let mut ids: Vec<String> = tasks.iter().filter_map(|t| t.stable_id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3, "ids must be unique");
        assert_eq!(tasks[0].description, "1.1 first");
        assert_eq!(tasks[1].description, "1.2 second");
        assert_eq!(tasks[2].description, "1.3 third");
    }

    #[test]
    fn task_done_on_unstamped_file_touches_only_the_target_line() {
        const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n- [ ] 1.3 third\n";
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", TASKS);
        ok(&store, Command::TaskDone { task_id: "3".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None });
        let written = store.read_artifact("demo", "tasks.md").unwrap();
        let orig: Vec<&str> = TASKS.lines().collect();
        let new: Vec<&str> = written.lines().collect();
        assert_eq!(orig.len(), new.len());
        for (i, (o, n)) in orig.iter().zip(&new).enumerate() {
            if i == 4 {
                assert!(
                    n.starts_with("- [x] 1.3 third <!-- speclink-task:tsk_"),
                    "target must be checked and stamped: {n}"
                );
            } else {
                assert_eq!(o, n, "line {i} must stay byte-identical");
            }
        }
        let stamped_id =
            crate::tasks::parse(&written)[2].stable_id.clone().expect("target gained an id");
        assert!(stamped_id.starts_with("tsk_"));
    }

    #[test]
    fn task_undone_never_stamps() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 first\n");
        ok(&store, Command::TaskUndone { task_id: "1".to_string(), change: Some("demo".to_string()) });
        assert_eq!(
            store.read_artifact("demo", "tasks.md").unwrap(),
            "- [ ] 1.1 first\n",
            "undone flips without stamping"
        );
    }

    // --- 雙值域定址、重複拒絕與事件載荷（spec task-identity）---

    const TID_A: &str = "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV";
    const TID_B: &str = "tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ";

    #[test]
    fn stable_id_addressing_hits_the_original_task_after_reorder() {
        let store = TestStore::with_meta("demo", META);
        // 「1.2 beta」原為第 2 任務、帶 ID；重排後移到末位，第 2 位改為 gamma。
        store.put_artifact(
            "demo",
            "tasks.md",
            &format!("- [ ] 1.1 alpha\n- [ ] 1.3 gamma\n- [ ] 1.2 beta <!-- speclink-task:{TID_B} -->\n"),
        );
        let (outcome, _) = ok(
            &store,
            Command::TaskDone { task_id: TID_B.to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
        );
        match outcome {
            CommandOutcome::TaskDone(o) => {
                assert_eq!(o.description, "1.2 beta", "stable id must hit the original task");
                assert!(!o.already);
            }
            other => panic!("expected a task-done outcome, got {other:?}"),
        }
        let (outcome, _) = ok(
            &store,
            Command::TaskDone { task_id: "2".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
        );
        match outcome {
            CommandOutcome::TaskDone(o) => {
                assert_eq!(o.description, "1.3 gamma", "ordinal must hit the task now in slot 2");
            }
            other => panic!("expected a task-done outcome, got {other:?}"),
        }
    }

    #[test]
    fn unknown_stable_id_errors_symmetric_to_out_of_range() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n- [x] 1.2 b\n");
        let ctx = ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() };
        let ordinal_err = execute(
            &store,
            &ctx,
            Command::TaskDone { task_id: "9".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
        )
        .expect_err("out-of-range ordinal must fail");
        assert_eq!(ordinal_err.message, "Task 9 not found (total: 2)");
        let id_err = execute(
            &store,
            &ctx,
            Command::TaskDone {
                task_id: "tsk_01NOPE0000000000000000000ial".to_string(),
                change: Some("demo".to_string()),
                touched_files: None,
                head_commit: None,
            },
        )
        .expect_err("unknown stable id must fail");
        assert_eq!(
            id_err.message,
            "Task tsk_01NOPE0000000000000000000ial not found (total: 2)",
            "unknown id error is symmetric to out-of-range"
        );
        assert_eq!(id_err.code, ordinal_err.code, "same error shape as out-of-range");
        let undone_err = execute(
            &store,
            &ctx,
            Command::TaskUndone {
                task_id: "tsk_01NOPE0000000000000000000ial".to_string(),
                change: Some("demo".to_string()),
            },
        )
        .expect_err("unknown stable id must fail for undone too");
        assert_eq!(undone_err.message, "Task tsk_01NOPE0000000000000000000ial not found (total: 2)");
        assert_eq!(*store.artifact_writes.borrow(), 0, "failed addressing writes nothing");
    }

    #[test]
    fn duplicate_stable_ids_refuse_task_verbs_naming_the_value() {
        let dup_md = format!(
            "- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n- [x] 1.2 b <!-- speclink-task:{TID_A} -->\n"
        );
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", &dup_md);
        let ctx = ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() };
        let done_err = execute(
            &store,
            &ctx,
            Command::TaskDone { task_id: TID_A.to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
        )
        .expect_err("duplicate ids must refuse task done");
        assert!(
            done_err.message.contains(TID_A),
            "error must name the duplicate value: {}",
            done_err.message
        );
        let undone_err = execute(
            &store,
            &ctx,
            Command::TaskUndone { task_id: "2".to_string(), change: Some("demo".to_string()) },
        )
        .expect_err("duplicate ids must refuse task undone regardless of addressing");
        assert!(undone_err.message.contains(TID_A));
        assert_eq!(
            store.read_artifact("demo", "tasks.md").unwrap(),
            dup_md,
            "refusal leaves tasks.md byte-identical"
        );
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal writes nothing");
    }

    #[test]
    fn task_completed_event_carries_the_stable_id() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", &format!("- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n"));
        let (_, events) = ok(
            &store,
            Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
        );
        match &events[0] {
            DomainEvent::TaskCompleted { task_id, .. } => assert_eq!(task_id, TID_A),
            other => panic!("expected task-completed, got {other:?}"),
        }
    }

    #[test]
    fn task_done_carries_host_injected_touched_files_into_the_event() {
        // spec verify-evidence「遠端 task done 攜檔案後 evidence 可查」的事件面：
        // Host 在邊界解析好的候選一路走到 task-completed payload，且同一份清單
        // 落進 evidence 記錄——事件是流、記錄是狀態，兩條都得有。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", &format!("- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n"));
        let (_, events) = ok(
            &store,
            Command::TaskDone {
                task_id: "1".to_string(),
                change: Some("demo".to_string()),
                touched_files: Some(vec!["src/wire.rs".to_string()]),
                head_commit: None,
            },
        );
        match &events[0] {
            DomainEvent::TaskCompleted { touched_files, .. } => {
                assert_eq!(touched_files, &vec!["src/wire.rs".to_string()]);
            }
            other => panic!("expected task-completed, got {other:?}"),
        }
        let rec = crate::tasks::TouchedRecord::load(&store, "demo");
        assert_eq!(rec.all_files(), vec!["src/wire.rs".to_string()]);
    }

    #[test]
    fn task_done_without_injected_files_carries_no_touched_files() {
        // 無候選不偽造：payload 未攜帶時事件的 touchedFiles 為空，而非補一份
        // 猜來的清單。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", &format!("- [ ] 1.1 a <!-- speclink-task:{TID_A} -->\n"));
        let (_, events) = ok(
            &store,
            Command::TaskDone {
                task_id: "1".to_string(),
                change: Some("demo".to_string()),
                touched_files: None,
                head_commit: None,
            },
        );
        match &events[0] {
            DomainEvent::TaskCompleted { touched_files, .. } => {
                assert!(touched_files.is_empty(), "nothing to attribute, nothing recorded");
            }
            other => panic!("expected task-completed, got {other:?}"),
        }
        assert_eq!(store.read_evidence("demo"), None);
    }

    #[test]
    fn task_completed_event_on_unstamped_task_carries_the_fresh_id() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
        let (_, events) = ok(
            &store,
            Command::TaskDone { task_id: "1".to_string(), change: Some("demo".to_string()), touched_files: None, head_commit: None },
        );
        let written = store.read_artifact("demo", "tasks.md").unwrap();
        let fresh = crate::tasks::parse(&written)[0]
            .stable_id
            .clone()
            .expect("done stamps the target line");
        match &events[0] {
            DomainEvent::TaskCompleted { task_id, .. } => {
                assert_eq!(task_id, &fresh, "event carries the id stamped by this very write");
            }
            other => panic!("expected task-completed, got {other:?}"),
        }
    }

    #[test]
    fn task_uncompleted_event_id_is_stable_id_or_ordinal_string() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact(
            "demo",
            "tasks.md",
            &format!("- [x] 1.1 a <!-- speclink-task:{TID_A} -->\n- [x] 1.2 b\n"),
        );
        let (_, events) = ok(
            &store,
            Command::TaskUndone { task_id: "1".to_string(), change: Some("demo".to_string()) },
        );
        match &events[0] {
            DomainEvent::TaskUncompleted { task_id, .. } => assert_eq!(task_id, TID_A),
            other => panic!("expected task-uncompleted, got {other:?}"),
        }
        let (_, events) = ok(
            &store,
            Command::TaskUndone { task_id: "2".to_string(), change: Some("demo".to_string()) },
        );
        match &events[0] {
            DomainEvent::TaskUncompleted { task_id, .. } => {
                assert_eq!(task_id, "2", "undone on an unstamped task falls back to the ordinal string");
            }
            other => panic!("expected task-uncompleted, got {other:?}"),
        }
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

    // --- claim:團隊模式 store 的認領語意(change-lifecycle「認領標記欄位」)---

    /// 認領測試共用的既有 meta:含建立欄位與 board_rank,用來釘「既有欄位
    /// 逐字元保留」。
    const CLAIM_META: &str =
        "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\nboard_rank: n\n";

    fn claim_ctx(actor: &str) -> ExecutionContext {
        ExecutionContext {
            actor: Some(actor.to_string()),
            workspace: Some(ghost_ws()),
            ..Default::default()
        }
    }

    fn claim(store: &TestStore, actor: &str) -> Result<(CommandOutcome, Vec<DomainEvent>), CommandError> {
        execute(store, &claim_ctx(actor), Command::Claim { name: "demo".to_string() })
    }

    #[test]
    fn first_claim_on_a_team_store_stamps_the_owner_and_reports_change_claimed() {
        let store = TestStore::team_with_meta("demo", CLAIM_META);
        let (outcome, events) = claim(&store, "Alice <a@example.com>").expect("first claim succeeds");

        match &outcome {
            CommandOutcome::Claim(o) => {
                assert_eq!(o.name, "demo");
                assert_eq!(o.claimed_by.as_deref(), Some("Alice <a@example.com>"));
                assert!(o.claimed, "the first claim is the one that stamps");
            }
            other => panic!("expected a claim outcome, got {other:?}"),
        }
        assert_eq!(kinds(&events), ["change-claimed"]);

        let meta = store.meta("demo");
        assert!(
            meta.starts_with(CLAIM_META),
            "existing meta fields survive byte-for-byte: {meta}"
        );
        let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).expect("stamped meta parses");
        assert_eq!(parsed.claimed_by.as_deref(), Some("Alice <a@example.com>"));
        assert_eq!(parsed.claimed_at.as_deref(), Some(crate::util::today().as_str()));
        assert_eq!(*store.meta_writes.borrow(), 1, "exactly one meta write");
    }

    #[test]
    fn repeat_claim_by_the_same_actor_is_idempotent_with_no_write_and_no_event() {
        let store = TestStore::team_with_meta("demo", CLAIM_META);
        claim(&store, "Alice <a@example.com>").expect("first claim succeeds");
        let first_stamp = store.meta("demo");

        let (outcome, events) = claim(&store, "Alice <a@example.com>").expect("repeat claim succeeds");
        match &outcome {
            CommandOutcome::Claim(o) => {
                assert_eq!(o.claimed_by.as_deref(), Some("Alice <a@example.com>"));
                assert!(!o.claimed, "the repeat claim reports no new stamp");
            }
            other => panic!("expected a claim outcome, got {other:?}"),
        }
        assert!(events.is_empty(), "an idempotent pass states no mutation");
        assert_eq!(store.meta("demo"), first_stamp, "the first stamp survives verbatim");
        assert_eq!(*store.meta_writes.borrow(), 1, "no second meta write");
    }

    #[test]
    fn claim_by_another_actor_is_refused_naming_the_holder_and_writes_nothing() {
        let store = TestStore::team_with_meta("demo", CLAIM_META);
        claim(&store, "Alice <a@example.com>").expect("first claim succeeds");
        let held = store.meta("demo");

        let err = claim(&store, "Bob <b@example.com>").expect_err("a held change refuses another claimant");
        assert_eq!(err.code, ErrorCode::Refused);
        assert!(
            err.message.contains("Alice <a@example.com>"),
            "the refusal must name the current holder: {}",
            err.message
        );
        assert_eq!(store.meta("demo"), held, "the holder's stamp is untouched");
        assert_eq!(*store.meta_writes.borrow(), 1, "the refused claim writes nothing");
    }

    /// 只剩 claimed_at 的 meta:目前沒有 release 動詞,人工「釋出」最直覺
    /// 就是手刪 claimed_by 那一行。
    const HALF_CLAIMED_META: &str =
        "schema: spec-driven\ncreated: 2026-07-01\nclaimed_at: 2026-07-02\n";

    #[test]
    fn claim_on_a_half_stamped_meta_is_refused_instead_of_duplicating_the_key() {
        // 認領章的兩個欄位一起寫,也必須一起判。半章 meta 若再追加一次,
        // claimed_at 會成為重複鍵而讓這個 change 永久無法解析——寧可拒絕。
        let store = TestStore::team_with_meta("demo", HALF_CLAIMED_META);
        let err = claim(&store, "Alice <a@example.com>").expect_err("a half stamp refuses");
        assert_eq!(err.code, ErrorCode::Refused);
        assert_eq!(store.meta("demo"), HALF_CLAIMED_META, "meta byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "no write on a refusal");
        crate::model::ChangeMeta::from_text(Some(&store.meta("demo")))
            .expect("the refused meta stays parseable");
    }

    #[test]
    fn claim_on_a_team_store_with_corrupt_meta_fails_closed_without_writing() {
        let store = TestStore::team_with_meta("demo", BAD_META);
        let err = claim(&store, "Alice <a@example.com>").expect_err("corrupt meta refuses");
        assert_eq!(err.code, ErrorCode::InvalidConfig);
        assert!(
            err.message.contains("openspec/changes/demo/.openspec.yaml"),
            "the error must name the metadata file: {}",
            err.message
        );
        assert_eq!(store.meta("demo"), BAD_META, "meta byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "no write on a fail-closed refusal");
    }

    #[test]
    fn claim_of_an_unknown_change_on_a_team_store_is_not_found() {
        let store = TestStore::team_with_meta("demo", CLAIM_META);
        let err = execute(
            &store,
            &claim_ctx("Alice <a@example.com>"),
            Command::Claim { name: "ghost".to_string() },
        )
        .expect_err("an unknown change cannot be claimed");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn claim_without_an_actor_is_refused_rather_than_stamping_an_anonymous_owner() {
        // 認領是「誰在做」的宣告——無身分可歸屬時寧可拒絕,也不留一筆
        // 沒有持有人的認領章(那會讓別人永遠撞衝突卻不知該找誰)。
        let store = TestStore::team_with_meta("demo", CLAIM_META);
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::Claim { name: "demo".to_string() },
        )
        .expect_err("an anonymous caller cannot claim");
        assert_eq!(err.code, ErrorCode::Refused);
        assert_eq!(store.meta("demo"), CLAIM_META);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn in_progress_add_reports_change_marked_in_progress() {
        let store = TestStore::with_meta("demo", META);
        let (_, events) = ok(&store, Command::InProgressAdd { name: "demo".to_string() });
        assert_eq!(kinds(&events), ["change-marked-in-progress"]);
    }

    // --- in-progress remove:退回動詞的 outcome、事件與守門錯誤分類 ---

    /// 帶開工戳記、無任何工作痕跡的 meta。
    const STARTED_META: &str = "schema: spec-driven\nstarted_at: 2026-07-10\nstarted_by: T <t@example.com>\n";

    #[test]
    fn in_progress_remove_reports_change_in_progress_removed() {
        let store = TestStore::with_meta("demo", STARTED_META);
        let (outcome, events) = ok(&store, Command::InProgressRemove { name: "demo".to_string() });
        match outcome {
            CommandOutcome::InProgressRemove(o) => {
                assert_eq!(o.name, "demo");
                assert!(o.removed, "the marker must actually be removed");
            }
            other => panic!("expected an in-progress-remove outcome, got {other:?}"),
        }
        assert_eq!(kinds(&events), ["change-in-progress-removed"]);
        assert!(!store.meta("demo").contains("started_"), "started_* lines stripped");
    }

    #[test]
    fn in_progress_remove_idempotent_pass_reports_no_event() {
        let store = TestStore::with_meta("demo", META);
        let (outcome, events) = ok(&store, Command::InProgressRemove { name: "demo".to_string() });
        match outcome {
            CommandOutcome::InProgressRemove(o) => {
                assert!(!o.removed, "nothing to remove must report removed: false");
            }
            other => panic!("expected an in-progress-remove outcome, got {other:?}"),
        }
        assert!(events.is_empty(), "an idempotent pass mutates nothing — no event");
    }

    #[test]
    fn in_progress_remove_gate_refusal_is_refused_with_evidence_on_the_error() {
        let store = TestStore::with_meta("demo", STARTED_META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 a\n- [x] 1.2 b\n- [ ] 1.3 c\n");
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::InProgressRemove { name: "demo".to_string() },
        )
        .expect_err("work traces must refuse the removal");
        assert_eq!(err.code, ErrorCode::Refused);
        let evidence = err
            .source
            .as_ref()
            .expect("the refusal keeps the flow error as source")
            .downcast_ref::<crate::inprogress::RevertBlocked>()
            .expect("structured evidence rides the error");
        assert_eq!(evidence.checked_tasks, 2);
        assert!(evidence.touched_files.is_empty());
        assert!(store.meta("demo").contains("started_at:"), "refusal must not strip the marker");
    }

    #[test]
    fn in_progress_remove_unknown_change_is_not_found() {
        // 與 add 的 parity 靜默刻意不對稱:修正動作打錯名字必須明確報錯。
        let store = TestStore::default();
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::InProgressRemove { name: "ghost".to_string() },
        )
        .expect_err("unknown change must error");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert!(err.message.contains("ghost"), "error names the change: {}", err.message);
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
                carry_review: false,
                carry_verify: false,
            },
        );
        assert_eq!(kinds(&events), ["change-archived"]);
        match &events[0] {
            DomainEvent::ChangeArchived { change, .. } => assert_eq!(change, "demo"),
            other => panic!("expected change-archived, got {other:?}"),
        }
    }

    #[test]
    fn archive_never_consults_evidence_wherever_it_runs() {
        // 討論 evidence-gate-false-blocks：封存不再讀證據判生死,所以有沒有 host
        // workspace 都一樣通過——server bridge／Node SDK 的合成 workspace 曾是
        // 「一律判缺席」的地雷,現在連判的動作都不存在。
        let cmd = Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
                carry_verify: false,
        };

        for workspace in [None, Some(ghost_ws())] {
            let store = TestStore::with_meta("demo", META);
            store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
            execute(&store, &ExecutionContext { workspace, ..Default::default() }, cmd.clone())
                .expect("an evidence-less change archives with or without a host workspace");
            assert!(!store.change_exists("demo"), "the change moved into the archive");
        }
    }

    #[test]
    fn archive_of_incomplete_change_is_refused() {
        // spec change-lifecycle「單筆封存的任務完成度守門」：任務未完成的單筆
        // Archive 經 runtime 分類為 refused，change 原地不動、無事件。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
        let err = execute(
            &store,
            &ExecutionContext { workspace: Some(ghost_ws()), ..Default::default() },
            Command::Archive {
                change: Some("demo".to_string()),
                skip_specs: false,
                no_validate: false,
                mark_tasks_complete: false,
                carry_review: false,
                carry_verify: false,
            },
        )
        .expect_err("incomplete change must refuse archive");
        assert_eq!(err.code, ErrorCode::Refused);
        assert!(err.message.contains("1/3"), "evidence rides the message: {}", err.message);
        assert!(store.change_exists("demo"), "nothing moved on refusal");
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
    fn review_verbs_execute_over_commands_with_events_and_actor() {
        // design D4a：review 動詞家族經 Command 分派（server 承載的動詞契約）——
        // 一條真實生命週期；stamp 取 ctx.actor 落 reviewed_by、scope 由提交端預算；
        // 查無映 NotFound（server 404）。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1 a\n");
        let (outcome, ev) = ok(
            &store,
            Command::ReviewAddRound {
                change: "demo".to_string(),
                content: "**Scope**: src/lib.rs\n".to_string(),
            },
        );
        match outcome {
            CommandOutcome::ReviewAddRound(o) => {
                assert_eq!((o.change.as_str(), o.round), ("demo", 1));
            }
            other => panic!("expected review add-round outcome, got {other:?}"),
        }
        assert_eq!(kinds(&ev), ["review-round-added"]);

        let (outcome, ev) = ok(&store, Command::ReviewShow { change: "demo".to_string() });
        match outcome {
            CommandOutcome::ReviewShow(o) => {
                assert_eq!(o.ticket.last_round().scope, ["src/lib.rs"]);
            }
            other => panic!("expected review show outcome, got {other:?}"),
        }
        assert!(ev.is_empty(), "show is a query and never produces events");

        let ctx = ExecutionContext {
            actor: Some("Rev <r@example.com>".to_string()),
            ..Default::default()
        };
        let hash = crate::review::content_fingerprint("fn lib() {}\n");
        let (_, ev) = execute(
            &store,
            &ctx,
            Command::ReviewStamp {
                change: "demo".to_string(),
                accept: false,
                tool: Some("claude".to_string()),
                scope: vec![crate::model::ReviewedScopeEntry {
                    path: "src/lib.rs".to_string(),
                    hash,
                }],
                missing: vec![],
            },
        )
        .expect("stamp executes");
        assert_eq!(kinds(&ev), ["review-stamped"]);
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo")))
            .expect("meta parses");
        assert_eq!(meta.reviewed_by.as_deref(), Some("Rev <r@example.com>"));
        assert_eq!(meta.reviewed_with.as_deref(), Some("claude"));

        let (_, ev) = ok(
            &store,
            Command::ReviewAddRound {
                change: "demo".to_string(),
                content: "**Scope**: src/lib.rs\n".to_string(),
            },
        );
        assert_eq!(kinds(&ev), ["review-round-added"]);
        let (_, ev) = ok(&store, Command::ReviewDiscard { change: "demo".to_string() });
        assert_eq!(kinds(&ev), ["review-discarded"]);

        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::ReviewShow { change: "demo".to_string() },
        )
        .expect_err("no ticket left after discard");
        assert_eq!(err.code, ErrorCode::NotFound, "missing ticket maps to 404: {}", err.message);
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
                kind: None,
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
                hold: false,
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
    fn discuss_archive_ignores_the_hold_flag() {
        // 手動封存是「放棄後續刀」的明示出口：帶 hold 的記錄照常封存。
        let store = TestStore::with_live_discussion(
            "staged",
            "---\nslug: staged\nstatus: promoted\npromoted_to: cut-a\ncreated: 2026-07-10\nhold: true\n---\n\n# Discussion: staged\n\n## Conclusion\n\n**Decision**: cut-b later\n",
        );

        let (_, events) = ok(&store, Command::DiscussArchive { slug: "staged".to_string() });

        assert_eq!(kinds(&events), ["discussion-archived"]);
        assert!(store.archived_discussion_exists("staged"));
        assert!(!store.live_discussion_exists("staged"));
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
                kind: None,
            },
        );
        ok(
            &store,
            Command::DiscussConclude {
                slug: "api-auth".to_string(),
                content: "結論：轉正。\n".to_string(),
                hold: false,
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

    // --- TaskMove 經 gateway（spec「任務搬移端點與重編號效果」；design 決策 4）---

    #[test]
    fn task_move_rewrites_tasks_and_reports_task_moved_event() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact(
            "demo",
            "tasks.md",
            "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n",
        );
        let (outcome, events) = ok(
            &store,
            Command::TaskMove { change: "demo".to_string(), from: 1, to: 3, before: None },
        );
        match outcome {
            CommandOutcome::TaskMove(o) => {
                assert_eq!(o.change, "demo");
                assert_eq!(o.description, "2.2 甲", "outcome carries the post-move description");
            }
            other => panic!("expected a task-move outcome, got {other:?}"),
        }
        assert_eq!(kinds(&events), ["task-moved"]);
        match &events[0] {
            DomainEvent::TaskMoved { change, .. } => assert_eq!(change, "demo"),
            other => panic!("expected task-moved, got {other:?}"),
        }
        let text = store
            .artifacts
            .borrow()
            .get(&("demo".to_string(), "tasks.md".to_string()))
            .unwrap()
            .clone();
        assert!(text.contains("- [ ] 2.2 甲"), "moved line renumbered into group 2: {text}");
    }

    #[test]
    fn task_move_out_of_range_refuses_without_writes_or_events() {
        const TASKS: &str = "- [ ] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n";
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", TASKS);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::TaskMove { change: "demo".to_string(), from: 5, to: 1, before: None },
        )
        .expect_err("out-of-range move must refuse");
        assert!(err.message.contains("out of range"), "must name the refusal: {}", err.message);
        assert_eq!(
            store.artifacts.borrow().get(&("demo".to_string(), "tasks.md".to_string())).unwrap(),
            TASKS,
            "tasks.md byte-identical"
        );
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    #[test]
    fn task_move_missing_tasks_md_is_not_found() {
        let store = TestStore::with_meta("demo", META);
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::TaskMove { change: "demo".to_string(), from: 1, to: 2, before: None },
        )
        .expect_err("move without tasks.md must refuse");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "tasks.md not found for change 'demo'");
    }

    #[test]
    fn event_kind_table_matches_the_spec_coverage_table() {
        // spec Example 變更型動詞與事件種類對應——18 列逐一斷言（含 execute 中
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
                DomainEvent::TaskCompleted {
                    change: s("c"),
                    task_id: s("tsk_1"),
                    touched_files: Vec::new(),
                    occurred_at: at,
                },
                "task-completed",
            ),
            (
                DomainEvent::TaskUncompleted { change: s("c"), task_id: s("tsk_1"), occurred_at: at },
                "task-uncompleted",
            ),
            (DomainEvent::TaskMoved { change: s("c"), occurred_at: at }, "task-moved"),
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
            (
                DomainEvent::ReviewRoundAdded { change: s("c"), round: 1, occurred_at: at },
                "review-round-added",
            ),
            (DomainEvent::ReviewStamped { change: s("c"), occurred_at: at }, "review-stamped"),
            (DomainEvent::ReviewDiscarded { change: s("c"), occurred_at: at }, "review-discarded"),
            (
                DomainEvent::VerifyRoundAdded { change: s("c"), round: 1, occurred_at: at },
                "verify-round-added",
            ),
            (DomainEvent::VerifyStamped { change: s("c"), occurred_at: at }, "verify-stamped"),
            (DomainEvent::VerifyDiscarded { change: s("c"), occurred_at: at }, "verify-discarded"),
        ];
        assert_eq!(table.len(), 24, "the coverage table has 24 mutating verbs");
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
            Command::List { sort: _, specs: _, changes: _, worktrees: _ } => {}
            Command::Show { item: _, item_type: _ } => {}
            Command::Status { change: _, schema: _ } => {}
            Command::Instructions { artifact: _, change: _, schema: _ } => {}
            Command::Validate { item: _, all: _, changes: _, specs: _, strict: _ } => {}
            Command::Analyze { change: _ } => {}
            Command::Trace { capability: _ } => {}
            Command::ArtifactCat { artifact: _, change: _ } => {}
            Command::LanguageShow => {}
            Command::DiscussList { archived: _ } => {}
            Command::DiscussShow { slug: _ } => {}
            Command::DiscussSearch { terms: _ } => {}
            Command::NewChange {
                name: _,
                description: _,
                schema: _,
                agent: _,
                from_discussion: _,
            } => {}
            Command::NewArtifact { kind: _, capability: _, change: _, content: _, force: _, new_capability: _ } => {}
            Command::TaskDone { task_id: _, change: _, touched_files: _, head_commit: _ } => {}
            Command::TaskUndone { task_id: _, change: _ } => {}
            Command::TaskMove { change: _, from: _, to: _, before: _ } => {}
            Command::Claim { name: _ } => {}
            Command::InProgressAdd { name: _ } => {}
            Command::InProgressRemove { name: _ } => {}
            Command::Archive { change: _, skip_specs: _, no_validate: _, mark_tasks_complete: _, carry_review: _, carry_verify: _ } => {}
            Command::Discard { change: _, force: _ } => {}
            Command::DiscussNew { topic: _, slug: _, kind: _ } => {}
            Command::DiscussContext { slug: _, content: _ } => {}
            Command::DiscussAddRound { slug: _, mode: _, content: _ } => {}
            Command::DiscussConclude { slug: _, content: _, hold: _ } => {}
            Command::DiscussPromote { slug: _, name: _ } => {}
            Command::DiscussLink { slug: _, change: _ } => {}
            Command::DiscussSeal { slug: _, change: _ } => {}
            Command::DiscussArchive { slug: _ } => {}
            Command::DiscussDiscard { slug: _, force: _ } => {}
            Command::ReviewAddRound { change: _, content: _ } => {}
            Command::ReviewShow { change: _ } => {}
            // `tool` 是工具名（CLI `--agent`，同 NewChange 的 agent），非身分；
            // 蓋章者身分仍只來自 ExecutionContext.actor。
            Command::ReviewStamp { change: _, accept: _, tool: _, scope: _, missing: _ } => {}
            Command::ReviewDiscard { change: _ } => {}
            Command::VerifyAddRound { change: _, content: _ } => {}
            Command::VerifyShow { change: _ } => {}
            Command::VerifyStamp {
                change: _,
                accept: _,
                tool: _,
                scope: _,
                missing: _,
            } => {}
            Command::VerifyDiscard { change: _ } => {}
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
            Command::DiscussNew { topic: "Identity probe".to_string(), slug: None, kind: None },
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
