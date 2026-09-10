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
use crate::station::{self, Station, REVIEW, VERIFY};
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
    } else if let Some(n) = e.downcast_ref::<station::NotFound>() {
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

/// 兩個品質站的工單動詞執行體：站別只差常數、outcome 結構兩站共用，Command
/// 臂只剩選站別與包 `CommandOutcome` 變體。`stamp` 臂不抽——抽出去要 8 個參數。
fn station_add_round(
    st: &Station,
    store: &dyn Store,
    change: String,
    content: &str,
) -> Result<ReviewRoundOutcome, CommandError> {
    let round = station::add_round(st, store, &change, content).map_err(classify)?;
    Ok(ReviewRoundOutcome { change, round })
}

fn station_show(
    st: &Station,
    store: &dyn Store,
    change: String,
) -> Result<ReviewShowOutcome, CommandError> {
    let (ticket, content) = station::show_with_content(st, store, &change).map_err(classify)?;
    Ok(ReviewShowOutcome { change, ticket, content })
}

fn station_discard(
    st: &Station,
    store: &dyn Store,
    change: String,
) -> Result<ReviewSubjectOutcome, CommandError> {
    station::discard(st, store, &change).map_err(classify)?;
    Ok(ReviewSubjectOutcome { change })
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
    pub ticket: station::Ticket,
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
            Ok(CommandOutcome::ReviewAddRound(station_add_round(&REVIEW, store, change, &content)?))
        }
        Command::ReviewShow { change } => {
            Ok(CommandOutcome::ReviewShow(station_show(&REVIEW, store, change)?))
        }
        Command::ReviewStamp { change, accept, tool, scope, missing } => {
            station::stamp_with_scope(
                &REVIEW,
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
            Ok(CommandOutcome::ReviewDiscard(station_discard(&REVIEW, store, change)?))
        }
        Command::VerifyAddRound { change, content } => {
            Ok(CommandOutcome::VerifyAddRound(station_add_round(&VERIFY, store, change, &content)?))
        }
        Command::VerifyShow { change } => {
            Ok(CommandOutcome::VerifyShow(station_show(&VERIFY, store, change)?))
        }
        Command::VerifyStamp { change, accept, tool, scope, missing } => {
            station::stamp_with_scope(
                &VERIFY,
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
            Ok(CommandOutcome::VerifyDiscard(station_discard(&VERIFY, store, change)?))
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
    let mut linked = None;
    if let Some(slug) = from_discussion.as_deref() {
        if crate::discuss::info(store, slug).is_none() {
            return Err(CommandError::new(
                ErrorCode::NotFound,
                format!("discussion '{slug}' not found — run `speclink discuss new` first"),
            ));
        }
        // The discussion-side link is computed before the change lands (same order as
        // `discuss::promote`): a record that cannot take it fails with nothing written.
        linked = crate::discuss::promoted_text(store, slug, &name).map_err(classify)?;
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
    if let (Some(slug), Some(out)) = (from_discussion.as_deref(), linked) {
        store.write_live_discussion(slug, &out).map_err(classify)?;
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
/// change metadata through [`crate::model::edit_meta`], like the started stamp.
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
    let claimed = crate::model::edit_meta(store, name, |m| {
        // An owner is the whole point of the verb: with nobody to record, a
        // stamp would make the change unclaimable while naming no one to
        // coordinate with. Judged inside the closure so an unknown change still
        // reads as not-found before the missing identity is reported.
        let Some(actor) = actor else {
            return Err(Refusal(format!(
                "cannot claim '{name}': no identity to record as its owner"
            ))
            .into());
        };
        // The stamp is a pair: both fields are written together, so both are
        // judged together. A meta carrying `claimed_at` alone is inconsistent —
        // writing the pair again would leave a duplicate key and make the change
        // permanently unparseable, so the half stamp refuses instead.
        // `claimed_by` is written through `yaml_scalar`, so read it back through
        // its inverse before comparing — a quoted holder is still the same person.
        let holder = m.get("claimed_by").map(crate::util::yaml_unscalar);
        match (holder, m.get("claimed_at")) {
            (Some(holder), _) if holder == actor => return Ok(false),
            (Some(holder), _) => {
                return Err(Refusal(format!(
                    "change '{name}' is already claimed by {holder} — coordinate with them, or ask them to release it"
                ))
                .into())
            }
            (None, Some(_)) => {
                return Err(Refusal(format!(
                    "cannot claim '{name}': its metadata carries claimed_at with no claimed_by — restore or remove that line in openspec/changes/{name}/.openspec.yaml"
                ))
                .into())
            }
            (None, None) => {}
        }
        m.set("claimed_at", &crate::util::today())?;
        m.set("claimed_by", &crate::util::yaml_scalar(actor))?;
        Ok(true)
    })
    .map_err(classify)?;
    let Some(claimed) = claimed else {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            format!("Change '{name}' not found."),
        ));
    };
    Ok(CommandOutcome::Claim(ClaimOutcome {
        name: name.to_string(),
        claimed_by: actor.map(str::to_string),
        claimed,
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
    // Every archive gate — environment, meta, open tickets, task readiness, stale
    // stamps, name collision, structural validation, merge plan — lives inside
    // `archive()` in one order, and its --mark-tasks-complete pre-write lands
    // after all of them. `guard_meta` is covered too: `require_valid_meta` yields
    // the same MetaError, classified to the same invalid_config.
    let host = host_workspace(ws);
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
mod tests;
