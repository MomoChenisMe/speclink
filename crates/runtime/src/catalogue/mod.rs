//! SpecLink operation catalogue — 37 個 operation 的 metadata single source of truth。
//!
//! 鏡像 `doc/protocol/operations.md` 的 Index 表。CI snapshot test
//! `crates/runtime/tests/catalogue_doc_sync.rs` 守 doc ↔ Rust 同步。
//!
//! 設計參考：
//! - `doc/speclink-design.md` §21 — Operation Catalogue 角色與映射規則
//! - `doc/speclink-design.md` §22.2 — Layer 1 curated 12 ops
//! - `doc/protocol/operations.md` — 單一 op 完整規格

pub mod schemas;

use serde_json::Value;

/// Operation idempotency 屬性。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Idempotency {
    /// Non-idempotent：重試會產生新效果。
    NonIdempotent,
    /// Idempotent：重試無副作用差異。
    Idempotent,
    /// Idempotent-with-version：重試需帶 etag CAS。
    IdempotentWithVersion,
}

/// Operation lock requirement（design.md §12.2.2 lock hierarchy）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockRequirement {
    /// 不需要 lock。
    None,
    /// Global short lock（讀寫小段元資料）。
    GlobalShort,
    /// Global exclusive lock。
    GlobalExclusive,
    /// Per-change exclusive lock。
    ChangeExclusive,
    /// Per-discussion exclusive lock。
    DiscussExclusive,
    /// Compound: per-change exclusive + global short。
    ChangeExclusivePlusGlobalShort,
}

/// Skill phase（design.md §4.4 / §22.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Discuss,
    Propose,
    Apply,
    Archive,
    Ingest,
}

/// Catalogue 單一 operation 的完整 metadata。
///
/// 欄位來源見 `doc/speclink-design.md` §21.1 catalogue 角色表。
pub struct Operation {
    /// Canonical id（如 `change.create`）。
    pub id: &'static str,
    /// Category bucket（如 `change`、`config`、`tool`）。
    pub category: &'static str,
    /// CLI binding 描述串（如 `new change <name>`）。
    pub cli: &'static str,
    /// Tool binding name（snake_case），純 meta-op 用 `"n/a"`。
    pub tool_binding: &'static str,
    /// SDK method（camelCase namespace，如 `speclink.changes.create`）。
    pub sdk_method: &'static str,
    /// HTTP endpoint（method + path，純本機 op 用 `"n/a"`）。
    pub http_endpoint: &'static str,
    /// MVP 是否必做。
    pub mvp: bool,
    /// 是否為 destructive op（CLI 需 confirmation；AI 永不主動帶 `--force`）。
    pub destructive: bool,
    pub idempotency: Idempotency,
    pub lock: LockRequirement,
    pub phases: &'static [Phase],
    /// design.md §22.2 Layer 1 curated subset 標記。
    pub curated: bool,
    /// 一句話描述（給 `describe-tools --format text` / SDK descriptor 用）。
    pub description: &'static str,
    /// 該 op 的 inputs JSON Schema 函式指標。回傳 `serde_json::Value` 必為 Object
    /// 且 `type == "object"`。AI tool function-call descriptor 走此欄位。
    pub inputs_schema: fn() -> Value,
    /// 該 op 的 outputs JSON Schema 函式指標。回傳 `serde_json::Value` 必為 Object。
    /// 還未實作的 op 使用 `empty_object_outputs_schema()` stub；對應 SDD slice 真做時
    /// 補完整 schema。`JsonRenderer` 印此欄位；`CopilotSdkRenderer` 不印（AI tool
    /// function-call convention inputs-only）。
    pub outputs_schema: fn() -> Value,
}

/// 37 個 operation 的鏡像 const slice。對齊 `doc/protocol/operations.md` Index 表。
const OPERATIONS: &[Operation] = &[
    // ----- project -----
    Operation {
        id: "project.init",
        category: "project",
        cli: "init <name>",
        tool_binding: "project_init",
        sdk_method: "speclink.init",
        http_endpoint: "POST /api/projects",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::GlobalShort,
        phases: &[],
        curated: false,
        description: "Initialize a SpecLink project inside a git working tree.",
        inputs_schema: schemas::project_init,
        outputs_schema: schemas::project_init_outputs,
    },
    Operation {
        id: "project.link",
        category: "project",
        cli: "link <url>",
        tool_binding: "project_link",
        sdk_method: "speclink.link",
        http_endpoint: "n/a",
        mvp: false,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "Bind the current working tree to an existing project_id (deferred).",
        inputs_schema: schemas::project_link,
        outputs_schema: schemas::project_link_outputs,
    },
    Operation {
        id: "project.unlink",
        category: "project",
        cli: "unlink",
        tool_binding: "project_unlink",
        sdk_method: "speclink.unlink",
        http_endpoint: "n/a",
        mvp: false,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "Remove the binding between working tree and project_id (deferred).",
        inputs_schema: schemas::project_unlink,
        outputs_schema: schemas::project_unlink_outputs,
    },
    Operation {
        id: "project.status",
        category: "project",
        cli: "status",
        tool_binding: "project_status",
        sdk_method: "speclink.status",
        http_endpoint: "GET /api/projects/{id}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Apply, Phase::Propose],
        curated: false,
        description: "Show project status including artifact DAG and active changes.",
        inputs_schema: schemas::project_status,
        outputs_schema: schemas::project_status_outputs,
    },
    // ----- config -----
    Operation {
        id: "config.read",
        category: "config",
        cli: "config show",
        tool_binding: "read_config",
        sdk_method: "speclink.config.read",
        http_endpoint: "GET /api/projects/{id}/config",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "Read .speclink/config.yaml and return Versioned<Config>.",
        inputs_schema: schemas::config_read,
        outputs_schema: schemas::config_read_outputs,
    },
    Operation {
        id: "config.write",
        category: "config",
        cli: "config set / config edit",
        tool_binding: "write_config",
        sdk_method: "speclink.config.write",
        http_endpoint: "PATCH /api/projects/{id}/config",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::IdempotentWithVersion,
        lock: LockRequirement::GlobalShort,
        phases: &[],
        curated: false,
        description: "Write config.yaml via key/value set or full-file edit with etag CAS.",
        inputs_schema: schemas::config_write,
        outputs_schema: schemas::config_write_outputs,
    },
    // ----- schema -----
    Operation {
        id: "schema.list",
        category: "schema",
        cli: "schemas",
        tool_binding: "list_schemas",
        sdk_method: "speclink.schemas.list",
        http_endpoint: "GET /api/projects/{id}/schemas",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "List available SDD schemas (built-in plus user forks).",
        inputs_schema: schemas::schema_list,
        outputs_schema: schemas::schema_list_outputs,
    },
    Operation {
        id: "schema.show",
        category: "schema",
        cli: "schema show <id>",
        tool_binding: "show_schema",
        sdk_method: "speclink.schemas.get",
        http_endpoint: "GET /api/projects/{id}/schemas/{id}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "Read a single schema definition by id.",
        inputs_schema: schemas::schema_show,
        outputs_schema: schemas::schema_show_outputs,
    },
    Operation {
        id: "schema.fork",
        category: "schema",
        cli: "schema fork",
        tool_binding: "fork_schema",
        sdk_method: "speclink.schemas.fork",
        http_endpoint: "POST /api/projects/{id}/schemas/{dst}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::GlobalShort,
        phases: &[],
        curated: false,
        description: "Fork a schema into a user-editable copy.",
        inputs_schema: schemas::schema_fork,
        outputs_schema: schemas::schema_fork_outputs,
    },
    Operation {
        id: "schema.delete",
        category: "schema",
        cli: "schema delete <id>",
        tool_binding: "delete_schema",
        sdk_method: "speclink.schemas.delete",
        http_endpoint: "DELETE /api/projects/{id}/schemas/{id}",
        mvp: true,
        destructive: true,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::GlobalShort,
        phases: &[],
        curated: false,
        description: "Delete a user-forked schema (destructive).",
        inputs_schema: schemas::schema_delete,
        outputs_schema: schemas::schema_delete_outputs,
    },
    // ----- discuss -----
    Operation {
        id: "discuss.new",
        category: "discuss",
        cli: "discuss new \"<topic>\"",
        tool_binding: "new_discussion",
        sdk_method: "speclink.discussions.create",
        http_endpoint: "POST /api/projects/{id}/discussions",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::DiscussExclusive,
        phases: &[Phase::Discuss],
        curated: true,
        description: "Create a new discussion thread for structured deliberation.",
        inputs_schema: schemas::discuss_new,
        outputs_schema: schemas::discuss_new_outputs,
    },
    Operation {
        id: "discuss.list",
        category: "discuss",
        cli: "discuss list",
        tool_binding: "list_discussions",
        sdk_method: "speclink.discussions.list",
        http_endpoint: "GET /api/projects/{id}/discussions",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Discuss],
        curated: false,
        description: "List all discussion threads.",
        inputs_schema: schemas::discuss_list,
        outputs_schema: schemas::discuss_list_outputs,
    },
    Operation {
        id: "discuss.show",
        category: "discuss",
        cli: "discuss show <id>",
        tool_binding: "show_discussion",
        sdk_method: "speclink.discussions.get",
        http_endpoint: "GET /api/projects/{id}/discussions/{id}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Discuss],
        curated: false,
        description: "Show a single discussion's full content.",
        inputs_schema: schemas::discuss_show,
        outputs_schema: schemas::discuss_show_outputs,
    },
    Operation {
        id: "discuss.patch",
        category: "discuss",
        cli: "discuss patch <id>",
        tool_binding: "patch_discussion",
        sdk_method: "speclink.discussions.patch",
        http_endpoint: "PATCH /api/projects/{id}/discussions/{id}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::IdempotentWithVersion,
        lock: LockRequirement::DiscussExclusive,
        phases: &[Phase::Discuss],
        curated: true,
        description: "Patch one section of a discussion (append round, edit conclusion).",
        inputs_schema: schemas::discuss_patch,
        outputs_schema: schemas::discuss_patch_outputs,
    },
    Operation {
        id: "discuss.conclude",
        category: "discuss",
        cli: "discuss conclude <id>",
        tool_binding: "conclude_discussion",
        sdk_method: "speclink.discussions.conclude",
        http_endpoint: "POST /api/projects/{id}/discussions/{id}/conclude",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::DiscussExclusive,
        phases: &[Phase::Discuss],
        curated: true,
        description: "Lock a discussion as concluded; freeze its conclusion section.",
        inputs_schema: schemas::discuss_conclude,
        outputs_schema: schemas::discuss_conclude_outputs,
    },
    Operation {
        id: "discuss.delete",
        category: "discuss",
        cli: "discuss delete <id>",
        tool_binding: "delete_discussion",
        sdk_method: "speclink.discussions.delete",
        http_endpoint: "DELETE /api/projects/{id}/discussions/{id}",
        mvp: true,
        destructive: true,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::DiscussExclusive,
        phases: &[Phase::Discuss],
        curated: false,
        description: "Delete a discussion thread (destructive).",
        inputs_schema: schemas::discuss_delete,
        outputs_schema: schemas::discuss_delete_outputs,
    },
    // ----- change -----
    Operation {
        id: "change.create",
        category: "change",
        cli: "new change <name>",
        tool_binding: "new_change",
        sdk_method: "speclink.changes.create",
        http_endpoint: "POST /api/projects/{id}/changes",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Propose],
        curated: true,
        description: "Create a new change in proposing state.",
        inputs_schema: schemas::change_create,
        outputs_schema: schemas::change_create_outputs,
    },
    Operation {
        id: "change.list",
        category: "change",
        cli: "list --changes",
        tool_binding: "list_changes",
        sdk_method: "speclink.changes.list",
        http_endpoint: "GET /api/projects/{id}/changes",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Apply, Phase::Ingest],
        curated: false,
        description: "List all changes regardless of state.",
        inputs_schema: schemas::change_list,
        outputs_schema: schemas::change_list_outputs,
    },
    Operation {
        id: "change.show",
        category: "change",
        cli: "show change <id>",
        tool_binding: "show_change",
        sdk_method: "speclink.changes.get",
        http_endpoint: "GET /api/projects/{id}/changes/{id}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Apply, Phase::Ingest],
        curated: false,
        description: "Show a change's metadata and artifact roster.",
        inputs_schema: schemas::change_show,
        outputs_schema: schemas::change_show_outputs,
    },
    Operation {
        id: "change.delete",
        category: "change",
        cli: "delete change <id>",
        tool_binding: "delete_change",
        sdk_method: "speclink.changes.delete",
        http_endpoint: "DELETE /api/projects/{id}/changes/{id}",
        mvp: true,
        destructive: true,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[],
        curated: false,
        description: "Delete a change row and its filesystem directory (destructive).",
        inputs_schema: schemas::change_delete,
        outputs_schema: schemas::change_delete_outputs,
    },
    // ----- artifact -----
    Operation {
        id: "artifact.write",
        category: "artifact",
        cli: "new artifact <kind>",
        tool_binding: "write_artifact",
        sdk_method: "speclink.artifacts.write",
        http_endpoint: "PUT /api/projects/{id}/changes/{cid}/artifacts/{kind}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::IdempotentWithVersion,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Propose, Phase::Apply, Phase::Ingest],
        curated: true,
        description: "Write an artifact (proposal/design/tasks/spec) with etag CAS.",
        inputs_schema: schemas::artifact_write,
        outputs_schema: schemas::artifact_write_outputs,
    },
    Operation {
        id: "artifact.read",
        category: "artifact",
        cli: "artifact read <kind>",
        tool_binding: "read_artifact",
        sdk_method: "speclink.artifacts.read",
        http_endpoint: "GET /api/projects/{id}/changes/{cid}/artifacts/{kind}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Apply, Phase::Ingest, Phase::Archive],
        curated: true,
        description: "Read an artifact's content and current etag.",
        inputs_schema: schemas::artifact_read,
        outputs_schema: schemas::artifact_read_outputs,
    },
    // ----- apply -----
    Operation {
        id: "apply.start",
        category: "apply",
        cli: "apply start <id>",
        tool_binding: "apply_start",
        sdk_method: "speclink.apply.start",
        http_endpoint: "POST /api/projects/{id}/changes/{cid}/apply/start",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Apply],
        curated: true,
        description: "Transition a change to in_progress and assign an actor.",
        inputs_schema: schemas::apply_start,
        outputs_schema: schemas::apply_start_outputs,
    },
    Operation {
        id: "apply.pause",
        category: "apply",
        cli: "apply pause <id>",
        tool_binding: "apply_pause",
        sdk_method: "speclink.apply.pause",
        http_endpoint: "POST /api/projects/{id}/changes/{cid}/apply/pause",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Apply],
        curated: false,
        description: "Pause an in-progress change back to ready and clear actor.",
        inputs_schema: schemas::apply_pause,
        outputs_schema: schemas::apply_pause_outputs,
    },
    Operation {
        id: "task.done",
        category: "apply",
        cli: "task done <task-id>",
        tool_binding: "task_done",
        sdk_method: "speclink.tasks.done",
        http_endpoint: "POST /api/projects/{id}/changes/{cid}/tasks/{tid}/done",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Apply],
        curated: true,
        description: "Mark a task as done; auto-transition when all tasks complete.",
        inputs_schema: schemas::task_done,
        outputs_schema: schemas::task_done_outputs,
    },
    // ----- review -----
    Operation {
        id: "review.approve",
        category: "review",
        cli: "review approve",
        tool_binding: "review_approve",
        sdk_method: "speclink.review.approve",
        http_endpoint: "POST /api/projects/{id}/changes/{cid}/review/approve",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Apply],
        curated: true,
        description: "Approve a review phase (artifact or code) and advance state.",
        inputs_schema: schemas::review_approve,
        outputs_schema: schemas::review_approve_outputs,
    },
    Operation {
        id: "review.reject",
        category: "review",
        cli: "review reject",
        tool_binding: "review_reject",
        sdk_method: "speclink.review.reject",
        http_endpoint: "POST /api/projects/{id}/changes/{cid}/review/reject",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::ChangeExclusive,
        phases: &[Phase::Apply],
        curated: true,
        description: "Reject a review phase with a reason; add synthetic feedback tasks.",
        inputs_schema: schemas::review_reject,
        outputs_schema: schemas::review_reject_outputs,
    },
    Operation {
        id: "review.history",
        category: "review",
        cli: "review history",
        tool_binding: "review_history",
        sdk_method: "speclink.review.history",
        http_endpoint: "GET /api/projects/{id}/changes/{cid}/review/history",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Apply, Phase::Archive],
        curated: false,
        description: "List all review events for a change in chronological order.",
        inputs_schema: schemas::review_history,
        outputs_schema: schemas::review_history_outputs,
    },
    // ----- archive -----
    Operation {
        id: "archive.run",
        category: "archive",
        cli: "archive <id>",
        tool_binding: "archive_change",
        sdk_method: "speclink.archive.run",
        http_endpoint: "POST /api/projects/{id}/changes/{cid}/archive",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::NonIdempotent,
        lock: LockRequirement::ChangeExclusivePlusGlobalShort,
        phases: &[Phase::Archive],
        curated: true,
        description: "Archive a completed change with spec delta merge.",
        inputs_schema: schemas::archive_run,
        outputs_schema: schemas::archive_run_outputs,
    },
    // ----- spec -----
    Operation {
        id: "spec.list",
        category: "spec",
        cli: "list --specs",
        tool_binding: "list_specs",
        sdk_method: "speclink.specs.list",
        http_endpoint: "GET /api/projects/{id}/specs",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Propose, Phase::Ingest],
        curated: false,
        description: "List canonical capability specs (post-archive merge targets).",
        inputs_schema: schemas::spec_list,
        outputs_schema: schemas::spec_list_outputs,
    },
    Operation {
        id: "spec.show",
        category: "spec",
        cli: "show spec <cap>",
        tool_binding: "show_spec",
        sdk_method: "speclink.specs.get",
        http_endpoint: "GET /api/projects/{id}/specs/{cap}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Propose, Phase::Ingest],
        curated: false,
        description: "Read a canonical capability spec by id.",
        inputs_schema: schemas::spec_show,
        outputs_schema: schemas::spec_show_outputs,
    },
    // ----- instructions / analyze / doctor / tool -----
    Operation {
        id: "instructions.get",
        category: "meta",
        cli: "instructions <kind>",
        tool_binding: "get_instructions",
        sdk_method: "speclink.instructions.get",
        http_endpoint: "GET /api/projects/{id}/changes/{cid}/instructions/{kind}",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[
            Phase::Discuss,
            Phase::Propose,
            Phase::Apply,
            Phase::Ingest,
            Phase::Archive,
        ],
        curated: true,
        description: "Get AI prompt and template for an artifact or workflow step.",
        inputs_schema: schemas::instructions_get,
        outputs_schema: schemas::instructions_get_outputs,
    },
    Operation {
        id: "analyze.run",
        category: "meta",
        cli: "analyze <id>",
        tool_binding: "analyze_change",
        sdk_method: "speclink.analyze.run",
        http_endpoint: "GET /api/projects/{id}/changes/{cid}/analyze",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Propose, Phase::Ingest],
        curated: false,
        description: "Run cross-artifact analyze (Coverage, Consistency, Ambiguity, Gaps).",
        inputs_schema: schemas::analyze_run,
        outputs_schema: schemas::analyze_run_outputs,
    },
    Operation {
        id: "validate.run",
        category: "meta",
        cli: "validate <id>",
        tool_binding: "validate_change",
        sdk_method: "speclink.validate.run",
        http_endpoint: "GET /api/projects/{id}/changes/{cid}/validate",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Propose, Phase::Apply, Phase::Archive],
        curated: false,
        description: "Validate a change's artifacts against its schema.",
        inputs_schema: schemas::validate_run,
        outputs_schema: schemas::validate_run_outputs,
    },
    Operation {
        id: "drift.run",
        category: "meta",
        cli: "drift <id>",
        tool_binding: "drift_change",
        sdk_method: "speclink.drift.run",
        http_endpoint: "GET /api/projects/{id}/changes/{cid}/drift",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[Phase::Apply, Phase::Ingest],
        curated: false,
        description: "Detect drift between change artifacts and current codebase state.",
        inputs_schema: schemas::drift_run,
        outputs_schema: schemas::drift_run_outputs,
    },
    Operation {
        id: "doctor.run",
        category: "doctor",
        cli: "doctor",
        tool_binding: "run_doctor",
        sdk_method: "speclink.doctor.run",
        http_endpoint: "n/a",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "Run 9 health-check categories with optional auto-fix.",
        inputs_schema: schemas::doctor_run,
        outputs_schema: schemas::doctor_run_outputs,
    },
    Operation {
        id: "tool.describe",
        category: "tool",
        cli: "describe-tools",
        tool_binding: "n/a",
        sdk_method: "speclink.describeTools",
        http_endpoint: "GET /api/projects/{id}/tool-catalogue",
        mvp: true,
        destructive: false,
        idempotency: Idempotency::Idempotent,
        lock: LockRequirement::None,
        phases: &[],
        curated: false,
        description: "Describe the operation catalogue in machine or human format.",
        inputs_schema: schemas::tool_describe,
        outputs_schema: schemas::tool_describe_outputs,
    },
];

/// Operation catalogue 查詢入口。
pub struct Catalogue;

impl Catalogue {
    /// 回傳全部 37 個 operation 的 const slice。
    #[must_use]
    pub fn all() -> &'static [Operation] {
        OPERATIONS
    }

    /// 以 canonical id 查詢 operation。Case-sensitive。
    #[must_use]
    pub fn get(id: &str) -> Option<&'static Operation> {
        OPERATIONS.iter().find(|op| op.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalogue_all_returns_static_slice() {
        // smoke：呼叫不 panic、回傳的是同一 reference。
        let a = Catalogue::all();
        let b = Catalogue::all();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn catalogue_count_is_37() {
        assert_eq!(Catalogue::all().len(), 37);
    }

    #[test]
    fn catalogue_get_existing_id_returns_some() {
        assert!(Catalogue::get("change.create").is_some());
        assert!(Catalogue::get("tool.describe").is_some());
    }

    #[test]
    fn catalogue_get_unknown_id_returns_none() {
        assert!(Catalogue::get("no.such.op").is_none());
        assert!(Catalogue::get("").is_none());
    }

    #[test]
    fn catalogue_get_is_case_sensitive() {
        assert!(Catalogue::get("CHANGE.CREATE").is_none());
        assert!(Catalogue::get("Change.Create").is_none());
    }

    #[test]
    fn catalogue_ids_match_operations_md_set() {
        let expected: HashSet<&str> = [
            "project.init",
            "project.link",
            "project.unlink",
            "project.status",
            "config.read",
            "config.write",
            "schema.list",
            "schema.show",
            "schema.fork",
            "schema.delete",
            "discuss.new",
            "discuss.list",
            "discuss.show",
            "discuss.patch",
            "discuss.conclude",
            "discuss.delete",
            "change.create",
            "change.list",
            "change.show",
            "change.delete",
            "artifact.write",
            "artifact.read",
            "apply.start",
            "apply.pause",
            "task.done",
            "review.approve",
            "review.reject",
            "review.history",
            "archive.run",
            "spec.list",
            "spec.show",
            "instructions.get",
            "analyze.run",
            "validate.run",
            "drift.run",
            "doctor.run",
            "tool.describe",
        ]
        .into_iter()
        .collect();
        let actual: HashSet<&str> = Catalogue::all().iter().map(|op| op.id).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn catalogue_destructive_set_is_three() {
        let actual: HashSet<&str> = Catalogue::all()
            .iter()
            .filter(|op| op.destructive)
            .map(|op| op.id)
            .collect();
        let expected: HashSet<&str> = ["change.delete", "discuss.delete", "schema.delete"]
            .into_iter()
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn catalogue_tool_binding_na_only_for_tool_describe() {
        let na: Vec<&str> = Catalogue::all()
            .iter()
            .filter(|op| op.tool_binding == "n/a")
            .map(|op| op.id)
            .collect();
        assert_eq!(na, vec!["tool.describe"]);
    }

    #[test]
    fn catalogue_every_op_has_non_empty_basic_fields() {
        for op in Catalogue::all() {
            assert!(!op.id.is_empty(), "id empty for some op");
            assert!(!op.category.is_empty(), "category empty for {}", op.id);
            assert!(!op.cli.is_empty(), "cli empty for {}", op.id);
            assert!(!op.sdk_method.is_empty(), "sdk_method empty for {}", op.id);
            assert!(
                !op.description.is_empty(),
                "description empty for {}",
                op.id
            );
            assert!(
                !op.tool_binding.is_empty(),
                "tool_binding empty for {}",
                op.id
            );
            assert!(
                !op.http_endpoint.is_empty(),
                "http_endpoint empty for {}",
                op.id
            );
        }
    }

    #[test]
    fn catalogue_schema_is_object_type() {
        for op in Catalogue::all() {
            let schema = (op.inputs_schema)();
            assert!(schema.is_object(), "schema not object for {}", op.id);
            let typ = schema
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("schema missing string type for {}", op.id));
            assert_eq!(typ, "object", "schema type != object for {}", op.id);
        }
    }

    #[test]
    fn catalogue_schema_is_deterministic() {
        for op in Catalogue::all() {
            let a = (op.inputs_schema)();
            let b = (op.inputs_schema)();
            assert_eq!(a, b, "schema not deterministic for {}", op.id);
        }
    }

    #[test]
    fn catalogue_curated_count_is_12() {
        let count = Catalogue::all().iter().filter(|op| op.curated).count();
        assert_eq!(count, 12);
    }

    #[test]
    fn catalogue_curated_set_matches_design_22_2() {
        let actual: HashSet<&str> = Catalogue::all()
            .iter()
            .filter(|op| op.curated)
            .map(|op| op.id)
            .collect();
        let expected: HashSet<&str> = [
            "discuss.new",
            "discuss.patch",
            "discuss.conclude",
            "change.create",
            "artifact.write",
            "artifact.read",
            "apply.start",
            "task.done",
            "review.approve",
            "review.reject",
            "archive.run",
            "instructions.get",
        ]
        .into_iter()
        .collect();
        assert_eq!(actual, expected);
    }

    // ----- outputs_schema (B 方案) -----

    #[test]
    fn catalogue_outputs_schema_pointers_all_non_panic_and_object() {
        for op in Catalogue::all() {
            let schema = (op.outputs_schema)();
            assert!(schema.is_object(), "outputs_schema not object for {}", op.id);
            let typ = schema
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("outputs_schema missing string type for {}", op.id));
            assert_eq!(
                typ, "object",
                "outputs_schema type != object for {}",
                op.id
            );
        }
    }

    #[test]
    fn catalogue_outputs_schema_is_deterministic() {
        for op in Catalogue::all() {
            let a = (op.outputs_schema)();
            let b = (op.outputs_schema)();
            assert_eq!(a, b, "outputs_schema not deterministic for {}", op.id);
        }
    }

    #[test]
    fn catalogue_project_status_outputs_required_has_six_names() {
        let op = Catalogue::get("project.status").expect("project.status in catalogue");
        let schema = (op.outputs_schema)();
        let required: HashSet<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .expect("project.status outputs has required array")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        let expected: HashSet<&str> = [
            "provider_type",
            "project_id",
            "working_dir",
            "changes_count",
            "discussions_count",
            "schema_active",
        ]
        .into_iter()
        .collect();
        assert_eq!(required, expected);
    }

    #[test]
    fn catalogue_change_show_outputs_properties_has_all_tasks_done_and_next_actions() {
        let op = Catalogue::get("change.show").expect("change.show in catalogue");
        let schema = (op.outputs_schema)();
        let props = schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("change.show outputs has properties");
        for required_key in ["change", "artifacts", "all_tasks_done", "next_actions"] {
            assert!(
                props.contains_key(required_key),
                "change.show outputs properties missing key: {}",
                required_key
            );
        }
        // 型別 sanity
        assert_eq!(
            props["all_tasks_done"]["type"], "boolean",
            "all_tasks_done not boolean"
        );
        assert_eq!(
            props["next_actions"]["type"], "array",
            "next_actions not array"
        );
    }
}
