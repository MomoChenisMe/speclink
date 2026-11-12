//! Skill registry, embedded bodies, and rendering (frontmatter + placeholder substitution).

use crate::config::{CustomTool, Invocation};
use crate::init::ASSET_VERSION;

/// The render targets: a built-in tool (claude, codex, copilot, pi, cursor) or a custom
/// descriptor.
/// Descriptors render the NEUTRAL body: no tool-specific slash prefix, no plan-mode
/// references, verb wording decided by the descriptor's `invocation`.
#[derive(Clone, Copy)]
pub enum RenderTarget<'a> {
    Builtin(Tool),
    Custom(&'a CustomTool),
}

/// A tool target for generated skills. Speclink deliberately scopes the tool matrix to
/// claude, codex, copilot, pi and cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Claude,
    Codex,
    Copilot,
    Pi,
    Cursor,
}

impl Tool {
    /// Every built-in tool, in the order selections and reports list them. The engine
    /// walks it for generation, deselection, the reserved descriptor names and
    /// directories, the asset lock and its error messages; the adapters (CLI hint, Node
    /// SDK, desktop) spell the set out in their own wording, pinned by their tests.
    pub const ALL: [Tool; 5] = [Tool::Claude, Tool::Codex, Tool::Copilot, Tool::Pi, Tool::Cursor];

    /// The built-in names as error messages list them: `claude, codex, copilot, pi, cursor`.
    pub fn supported_names() -> String {
        Tool::ALL.map(|t| t.name()).join(", ")
    }

    pub fn parse(s: &str) -> Option<Tool> {
        match s.trim().to_ascii_lowercase().as_str() {
            "claude" => Some(Tool::Claude),
            "codex" | "agents" => Some(Tool::Codex),
            "copilot" => Some(Tool::Copilot),
            "pi" => Some(Tool::Pi),
            "cursor" => Some(Tool::Cursor),
            _ => None,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
            Tool::Copilot => "copilot",
            Tool::Pi => "pi",
            Tool::Cursor => "cursor",
        }
    }
    /// Directory (relative to project root) that holds generated skill files.
    pub fn skills_dir(&self) -> &'static str {
        match self {
            Tool::Claude => ".claude/skills",
            Tool::Codex => ".agents/skills",
            Tool::Copilot => ".github/skills",
            Tool::Pi => ".pi/skills",
            Tool::Cursor => ".cursor/skills",
        }
    }
    /// Where the tool keeps plan-mode files ({{PLAN_DIR}}); empty when the tool has none.
    fn plan_dir(&self) -> &'static str {
        match self {
            Tool::Claude => "~/.claude/plans/",
            Tool::Codex | Tool::Copilot | Tool::Pi | Tool::Cursor => "",
        }
    }
    /// The prefix that `/speclink:` becomes for this tool.
    fn slash_replacement(&self) -> &'static str {
        match self {
            Tool::Claude | Tool::Copilot | Tool::Cursor => "/speclink-",
            Tool::Codex => "$speclink-",
            Tool::Pi => "/skill:speclink-",
        }
    }
}

/// A registered skill.
pub struct Skill {
    pub name: &'static str,
    pub description: &'static str,
    /// Read-only fork skill (context: fork + agent: Explore) — Claude frontmatter only.
    pub fork: bool,
    /// Whether to add `disallowedTools: [Edit, Write]` — Claude frontmatter only.
    pub disallow_edit: bool,
    /// Generated for Claude only. Every other target (codex, copilot, pi, cursor and
    /// custom descriptors) gets the rest of the registry.
    pub claude_only: bool,
    /// Generated only when the workflow config's `worktree` policy is on. The registry
    /// itself stays complete regardless — the gate applies to the GENERATION set, so
    /// `skill_body`, the SDK's render API and the golden locks keep seeing every skill.
    pub worktree_gated: bool,
    pub body: &'static str,
}

// Embedded bodies.
const B_ANALYZE: &str = include_str!("../../assets/skills/analyze.md");
const B_APPLY: &str = include_str!("../../assets/skills/apply.md");
/// Composed at COMPILE TIME so the body stays a `&'static str` and the apply
/// flow keeps a single source: the worktree preamble, the whole apply body
/// (same asset the plain apply skill embeds), then the worktree wrap-up.
const B_APPLY_WITH_WORKTREE: &str = concat!(
    include_str!("../../assets/skills/apply-worktree-pre.md"),
    include_str!("../../assets/skills/apply.md"),
    include_str!("../../assets/skills/apply-worktree-post.md"),
);
const B_WORKTREE_MERGE: &str = include_str!("../../assets/skills/worktree-merge.md");
const B_ARCHIVE: &str = include_str!("../../assets/skills/archive.md");
const B_AUDIT: &str = include_str!("../../assets/skills/audit.md");
const B_COMMIT: &str = include_str!("../../assets/skills/commit.md");
const B_CONFIG: &str = include_str!("../../assets/skills/config.md");
const B_DISCUSS: &str = include_str!("../../assets/skills/discuss.md");
const B_DRIFT: &str = include_str!("../../assets/skills/drift.md");
const B_IMPROVE: &str = include_str!("../../assets/skills/improve.md");
const B_INGEST: &str = include_str!("../../assets/skills/ingest.md");
const B_MANUAL: &str = include_str!("../../assets/skills/manual.md");
const B_BASELINE: &str = include_str!("../../assets/skills/baseline.md");
const B_PROPOSE: &str = include_str!("../../assets/skills/propose.md");
const B_QUALITY: &str = include_str!("../../assets/skills/quality.md");
/// The two quality stations share their checking rules (brief discipline,
/// verification gate) from one source, placed between each station's intro
/// and its steps.
const B_REVIEW: &str = concat!(
    include_str!("../../assets/skills/review-intro.md"),
    include_str!("../../assets/skills/station-shared.md"),
    include_str!("../../assets/skills/review.md"),
);
const B_TRACE: &str = include_str!("../../assets/skills/trace.md");
const B_VERIFY: &str = concat!(
    include_str!("../../assets/skills/verify-intro.md"),
    include_str!("../../assets/skills/station-shared.md"),
    include_str!("../../assets/skills/verify.md"),
);
const B_SYNC: &str = include_str!("../../assets/skills/sync.md");
const B_CLARIFY: &str = include_str!("../../assets/skills/clarify.md");
const B_TDD: &str = include_str!("../../assets/skills/tdd.md");

/// The review skill's smell baseline names, in template order — the single
/// source presentation surfaces (the desktop review-rules page) list, so no
/// second copy can drift. A unit test pins it to the review skill template.
pub const REVIEW_BASELINE_SMELLS: [&str; 12] = [
    "Mysterious Name",
    "Duplicated Code",
    "Feature Envy",
    "Data Clumps",
    "Primitive Obsession",
    "Repeated Switches",
    "Shotgun Surgery",
    "Divergent Change",
    "Speculative Generality",
    "Message Chains",
    "Middle Man",
    "Refused Bequest",
];

/// The skills that generate SKILL.md files.
pub fn registry() -> Vec<Skill> {
    vec![
        Skill { name: "analyze", description: "Use when a change's artifacts should be cross-checked before implementing — coverage, consistency, ambiguity and gaps across proposal, design, specs and tasks; reports findings without editing anything.", fork: true, disallow_edit: true, claude_only: true, worktree_gated: false, body: B_ANALYZE },
        Skill { name: "apply", description: "Use when a change's tasks are ready to implement, or when resuming one left half-done — works through the task list, marking each checkbox as it lands.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_APPLY },
        // Same permissions as `apply` — it IS apply, wrapped in worktree setup
        // and hand-off; the extra steps are git commands, not a narrower role.
        Skill { name: "apply-with-worktree", description: "Use when several independent changes are being implemented at once — runs apply inside an isolated git worktree per change so parallel work never collides.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: true, body: B_APPLY_WITH_WORKTREE },
        Skill { name: "archive", description: "Use when a change is finished and its quality stations are settled — folds the deltas into the specs and moves the change into the archive.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_ARCHIVE },
        // Not a fork skill: the rewritten standalone mode fans out three
        // parallel audit agents, which the fork's Explore agent cannot spawn.
        Skill { name: "audit", description: "Use when changed code needs a security pass — hunts dangerous defaults, type confusion and silent failures, and reports the sharp edges it finds.", fork: false, disallow_edit: true, claude_only: false, worktree_gated: false, body: B_AUDIT },
        Skill { name: "baseline", description: "Use when adopting Speclink on a codebase that already has behavior but no specs — establishes the baseline by generating the initial specs from what the code does today.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_BASELINE },
        Skill { name: "commit", description: "Use when only the files belonging to one specific change should be committed — selects that change's files and writes the commit.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_COMMIT },
        // Writes only through `speclink workflow-config` (never a direct file
        // edit), so Edit/Write stay disallowed; not a fork skill because the
        // policy fields must be asked for interactively.
        Skill { name: "config", description: "Use when the workflow config's project context or per-artifact rules need composing or refreshing from the codebase — lands them through an approved diff.", fork: false, disallow_edit: true, claude_only: false, worktree_gated: false, body: B_CONFIG },
        Skill { name: "discuss", description: "Use when requirements are fuzzy, contested, or worth debating before any change exists — records the exchange as a discussion document that can later be promoted into a change.", fork: false, disallow_edit: true, claude_only: false, worktree_gated: false, body: B_DISCUSS },
        Skill { name: "drift", description: "Use when picking a change back up after it sat idle, before touching its tasks — reports how far the codebase has moved from the delta's assumptions.", fork: true, disallow_edit: true, claude_only: false, worktree_gated: false, body: B_DRIFT },
        // discuss 的鏡像入口（模型帶 candidates），權限比照 discuss：記錄一律
        // 經 CLI 寫入，技能本身不得實作程式碼，故 Edit/Write 禁用；要互動挑
        // candidate 並逐輪 grill，不是 fork。
        Skill { name: "improve", description: "Use when improvements are asked for without naming a topic — user-initiated only, never on the model's own initiative; scans the codebase and records the candidates as a discussion.", fork: false, disallow_edit: true, claude_only: false, worktree_gated: false, body: B_IMPROVE },
        Skill { name: "ingest", description: "Use when requirements change mid-work, including after a separate planning session — folds the new context into an existing change's artifacts so apply can resume.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_INGEST },
        // Tool skill with two modes (spec manual-skill): generation writes the
        // manual pages under the spec dir, so Edit/Write stay allowed; tour mode
        // asks the user one role question interactively, so not a fork.
        Skill { name: "manual", description: "Use when a human-readable operating manual is needed, or when someone wants to be walked through how to operate the system — generates a wiki-style Markdown manual under {{SPEC_DIR}}manual/ from the canonical specs, or gives an in-conversation tour with every answer sourced.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_MANUAL },
        Skill { name: "propose", description: "Use when a change needs planning, proposing or designing — creates the change with every required artifact; seed it from a concluded discussion with --from-discussion.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_PROPOSE },
        // Pure ordering over the two stations plus the per-round pause（design
        // D1）：不是 fork——它要在主線依序呼叫兩站技能、停下問使用者，且步驟 4
        // 的修正就在主線改檔，故 Edit 不禁。
        Skill { name: "quality", description: "Use when both quality stations should run over one change, pausing after every round for the user's call: both check without stamping, their findings are reported together and the skill stops — for only one station, call review or verify directly.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_QUALITY },
        // Not a fork skill（design D7 替代案否決 fork）：主線 orchestrator 要
        // fan-out 兩個平行 sub-agent 並互動詢問三選項；修正回主線，故 Edit 不禁。
        Skill { name: "review", description: "Use when an implementation should be checked for craft quality before archiving — parallel standards and correctness axes, recorded to a review ticket.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_REVIEW },
        // 唯讀問答（比照 drift 的 fork 形狀）：走 speclink verbs 與 git 組敘事
        // 答案，永不改檔，Edit/Write 禁用。
        Skill { name: "trace", description: "Use when someone asks how a capability came to be or why it works this way — walks its provenance chain across archived changes, source discussions, evidence and live code.", fork: true, disallow_edit: true, claude_only: false, worktree_gated: false, body: B_TRACE },
        // Not a fork skill any more（design D6）：主線要取得 frozen scope、落
        // structured 工單、互動詢問三選項並在本地依 TDD 修正；檢查段本身仍以
        // 單一唯讀 sub-agent 隔離。codex 變體以純文字詢問，兩工具同步生成。
        Skill { name: "verify", description: "Use when an implementation should be checked against its artifacts before archiving — confirms the specs and tasks are actually satisfied, recorded to a verify ticket.", fork: false, disallow_edit: false, claude_only: false, worktree_gated: false, body: B_VERIFY },
        // Runs git commands only — a conflict stops the flow for the user to
        // resolve, so the agent never edits a file: Edit/Write stay disallowed.
        Skill { name: "worktree-merge", description: "Use when a worktree change is committed and ready to land — merges the branch back into the main branch, then cleans the worktree up.", fork: false, disallow_edit: true, claude_only: false, worktree_gated: true, body: B_WORKTREE_MERGE },
    ]
}

/// Lookup any embedded skill body (including internal ones) by name: the registry
/// is the single table for every rendered skill; only the three internal skills
/// (taken via `speclink instructions --skill`, never rendered — spec skill-routing
/// 「內部技能不參與路由」) live outside it.
pub fn skill_body(name: &str) -> Option<&'static str> {
    if let Some(skill) = registry().into_iter().find(|s| s.name == name) {
        return Some(skill.body);
    }
    Some(match name {
        "sync" => B_SYNC,
        "clarify" => B_CLARIFY,
        "tdd" => B_TDD,
        _ => return None,
    })
}

/// The lead sentence every "Next steps" hand-off section opens with (spec
/// skill-routing「出口交棒由技能結尾承載」的 guidance-only 條款). Injected at
/// render time via `{{NEXT_STEPS_LEAD}}` so the eleven-plus assets that carry a
/// hand-off section cannot drift apart one wording tweak at a time.
const NEXT_STEPS_LEAD: &str = "Suggestions only. This skill NEVER invokes any of them — report where things stand and stop; the user decides what runs next.";

/// The write-back section every skill that changes an existing artifact shares: how
/// the artifact is read whole and written back whole. Injected via
/// `{{ARTIFACT_WRITE_BACK}}` on a line of its own, so the copies cannot drift apart.
const ARTIFACT_WRITE_BACK: &str = include_str!("../../assets/skills/artifact-write-back.md");

/// The conversation-language paragraph every rendered product skill opens with
/// (spec skill-authoring「技能共用的對話語言段落」). Inserted by the renderer rather
/// than embedded in each body, so `instructions --skill` keeps printing the raw body.
const CONVERSATION_LANGUAGE: &str = include_str!("../../assets/skills/conversation-language.md");

/// Where a project's spec folder is, seen from the checkout a generated skill runs in
/// (spec workspace-tools「技能依規格是否在 checkout 渲染」). It is the one thing a skill
/// source can branch on, and it decides which of the source's conditional blocks stay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecLocation {
    /// In this checkout — a local project. Git steps on spec files and spec paths apply.
    InCheckout,
    /// Not in this checkout — a remote project keeps its specs on the Server, and a host
    /// that embeds the engine has no local spec files.
    Elsewhere,
}

impl SpecLocation {
    /// Both locations, the local one first.
    pub const BOTH: [SpecLocation; 2] = [SpecLocation::InCheckout, SpecLocation::Elsewhere];
}

/// The name of that one condition in a skill source (spec skill-authoring「原稿的條件區塊」).
const BLOCK_CONDITION: &str = "SPEC_IN_CHECKOUT";

/// A malformed conditional block in a skill source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockError {
    /// The source the marker is in: a file under assets/skills, or a skill name (whose
    /// lines then count through its composed body).
    pub source: String,
    /// 1-based line of the marker at fault.
    pub line: usize,
    pub problem: String,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.source, self.line, self.problem)
    }
}

impl std::error::Error for BlockError {}

/// Resolve the conditional blocks of a skill source: the lines between
/// `{{#SPEC_IN_CHECKOUT}}` and `{{/SPEC_IN_CHECKOUT}}` stay only when the spec folder is
/// in the checkout, those between `{{^SPEC_IN_CHECKOUT}}` and `{{/SPEC_IN_CHECKOUT}}` only
/// when it is not. A marker stands alone on its line (indentation allowed) and never
/// reaches the output; every other line keeps its own line ending.
///
/// Errors — each names `source` and the marker's line: a block that is never closed, a
/// block opened inside another, a closing marker with no open block, a condition other
/// than `SPEC_IN_CHECKOUT`, and a marker that shares its line with other text.
pub fn resolve_blocks(source: &str, text: &str, location: SpecLocation) -> Result<String, BlockError> {
    let fail = |line: usize, problem: String| BlockError { source: source.to_string(), line, problem };
    let mut out = String::with_capacity(text.len());
    // The open block: the line of its opening marker, and whether its lines stay.
    let mut open: Option<(usize, bool)> = None;
    for (index, raw) in text.split_inclusive('\n').enumerate() {
        let line = index + 1;
        if !["{{#", "{{^", "{{/"].iter().any(|marker| raw.contains(marker)) {
            if open.is_none_or(|(_, keep)| keep) {
                out.push_str(raw);
            }
            continue;
        }
        let marker = raw
            .trim()
            .strip_prefix("{{")
            .and_then(|rest| rest.strip_suffix("}}"))
            .filter(|inner| !inner.contains(['{', '}']))
            .ok_or_else(|| fail(line, "a block marker must be alone on its line".to_string()))?;
        let (sigil, condition) = marker.split_at(1);
        if condition != BLOCK_CONDITION {
            return Err(fail(
                line,
                format!("unknown block condition `{condition}` — the only one is `{BLOCK_CONDITION}`"),
            ));
        }
        open = match (sigil, open) {
            ("/", Some(_)) => None,
            ("/", None) => return Err(fail(line, "closing marker with no open block".to_string())),
            (_, Some((opened, _))) => {
                return Err(fail(
                    line,
                    format!("blocks cannot be nested — the block opened on line {opened} is still open"),
                ))
            }
            ("#", None) => Some((line, location == SpecLocation::InCheckout)),
            (_, None) => Some((line, location == SpecLocation::Elsewhere)),
        };
    }
    match open {
        Some((opened, _)) => Err(fail(opened, "this block is never closed".to_string())),
        None => Ok(out),
    }
}

/// [`resolve_blocks`] for an embedded source. A malformed block there is a defect in the
/// build, not a runtime condition — the unit tests render every skill in both cases — so
/// the render stops loudly instead of writing half a skill.
fn embedded_blocks(source: &str, text: &str, location: SpecLocation) -> String {
    resolve_blocks(source, text, location)
        .unwrap_or_else(|err| panic!("malformed conditional block in an embedded skill source — {err}"))
}

/// `body` with a shared fragment standing where `placeholder` is. The fragment's own
/// conditional blocks are resolved first, under its file name, so an error points at
/// the fragment and not at the skill that inserts it.
fn with_fragment(body: &str, placeholder: &str, file: &str, fragment: &str, location: SpecLocation) -> String {
    body.replace(placeholder, embedded_blocks(file, fragment, location).trim_end())
}

/// The placeholder values one render target substitutes into a skill body. The target
/// decides the wording: built-ins keep their slash prefix and plan directory; descriptors
/// drop plan mode and name another skill by its skill name (`speclink-apply`), which a
/// verb (`speclink archive`) can never be mistaken for. `location` is the project's
/// side: it decides which conditional blocks stay.
struct Substitutions<'a> {
    spec_dir_slash: String,
    location: SpecLocation,
    plan_dir: &'a str,
    tool_name: &'a str,
    slash_replacement: &'a str,
    /// Descriptors have no plan directory, so lines that mention plan mode are dropped.
    drop_plan_mode_lines: bool,
}

impl Substitutions<'_> {
    /// Render `body`. Conditional blocks go first, before any placeholder; `source` is
    /// the name a block error reports.
    fn apply(&self, source: &str, body: &str) -> String {
        let body = embedded_blocks(source, body, self.location);
        let body = if self.drop_plan_mode_lines {
            body.lines()
                .filter(|l| !l.to_ascii_lowercase().contains("plan mode"))
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            body
        };
        with_fragment(
            &body,
            "{{ARTIFACT_WRITE_BACK}}",
            "artifact-write-back.md",
            ARTIFACT_WRITE_BACK,
            self.location,
        )
        .replace("{{SPEC_DIR}}", &self.spec_dir_slash)
        .replace("{{PLAN_DIR}}", self.plan_dir)
        .replace("{{TOOL}}", self.tool_name)
        .replace("{{NEXT_STEPS_LEAD}}", NEXT_STEPS_LEAD)
        .replace("/speclink:", self.slash_replacement)
    }
}

/// `spec_dir` with exactly one trailing slash — the `{{SPEC_DIR}}` rendering.
fn spec_dir_slash(spec_dir: &str) -> String {
    if spec_dir.ends_with('/') {
        spec_dir.to_string()
    } else {
        format!("{spec_dir}/")
    }
}

/// A skill's description with `{{SPEC_DIR}}` rendered — the only substitution a
/// description gets: it is one frozen line.
pub fn description_for(skill: &Skill, spec_dir: &str) -> String {
    skill.description.replace("{{SPEC_DIR}}", &spec_dir_slash(spec_dir))
}

fn substitutions<'a>(target: RenderTarget<'a>, spec_dir: &str, location: SpecLocation) -> Substitutions<'a> {
    let spec_dir_slash = spec_dir_slash(spec_dir);
    match target {
        RenderTarget::Builtin(tool) => Substitutions {
            spec_dir_slash,
            location,
            plan_dir: tool.plan_dir(),
            tool_name: tool.name(),
            slash_replacement: tool.slash_replacement(),
            drop_plan_mode_lines: false,
        },
        RenderTarget::Custom(custom) => Substitutions {
            spec_dir_slash,
            location,
            plan_dir: "",
            tool_name: &custom.name,
            slash_replacement: "speclink-",
            drop_plan_mode_lines: true,
        },
    }
}

/// Render a skill body alone — no frontmatter, no preamble — for one target: conditional
/// blocks resolved for `location`, then the placeholders substituted.
pub fn substitute(body: &str, target: RenderTarget, spec_dir: &str, location: SpecLocation) -> String {
    substitutions(target, spec_dir, location).apply("skill body", body)
}

/// Claude-only fork preamble: fork auto-select rules that take
/// precedence over the shared skill body.
fn fork_context(skill_name: &str) -> Option<String> {
    let rule = match skill_name {
        "analyze" | "drift" => format!(
            "When no change name is provided, run `speclink list --json`. Auto-select only \
when there is exactly one active change. If there are zero active changes or more than one \
active change, return the candidate list or empty-state message and ask the main thread to \
rerun `/speclink-{skill_name} <change-name>`. Do NOT ask an interactive selection question \
inside the fork."
        ),
        "verify" => "When no change name is provided, run `speclink list --json` and \
consider only active changes with implementation tasks. Auto-select only when exactly one \
matching active change exists. If there are zero matching active changes or more than one \
matching active change, return the candidate list or empty-state message and ask the main \
thread to rerun `/speclink-verify <change-name>`. Do NOT ask an interactive selection \
question inside the fork."
            .to_string(),
        _ => return None,
    };
    Some(format!(
        "## Claude fork context\n\nThis generated Claude Code skill runs with `context: fork`. \
The rules in this section take precedence over the shared `{skill_name}` body below.\n\n{rule}\n\n---\n\n"
    ))
}

/// The invocation preamble that tells a custom harness how `speclink <verb>` references in
/// the body are meant to be executed — and that `speclink-<name>` is not one of them: the
/// body names another skill that way, and a skill sent as a verb either fails or, where a
/// verb has the same name (archive, review, verify), runs the verb.
fn invocation_note(invocation: Invocation) -> &'static str {
    match invocation {
        Invocation::Cli => {
            "## Invocation\n\nThis harness executes speclink verbs as shell commands: \
run `speclink <verb> [arguments]`. A name written `speclink-<name>` is another skill, not a \
verb: load that skill and follow it — do not run it as a command.\n\n---\n\n"
        }
        Invocation::ToolCall => {
            "## Invocation\n\nThis harness executes speclink verbs by calling the speclink \
tool with an argv array (e.g. [\"list\", \"--json\"]). Wherever this document says \
`speclink <verb> [arguments]`, it means calling the speclink tool with those arguments \
as argv. A name written `speclink-<name>` is another skill, not a verb: load that skill and \
follow it — do not send it to the speclink tool as arguments.\n\n---\n\n"
        }
    }
}

/// Render a complete SKILL.md (frontmatter + preamble + conversation-language paragraph +
/// substituted body) for one target.
/// The fork/agent and disallowedTools lines are Claude-only; descriptors get the neutral
/// frontmatter plus an invocation preamble (frozen output shape).
///
/// `location` says where the spec folder is, seen from the checkout the skill file is
/// for: it decides which of the source's conditional blocks stay.
pub fn render_skill_file_for(
    target: RenderTarget,
    skill: &Skill,
    spec_dir: &str,
    location: SpecLocation,
) -> String {
    let claude = matches!(target, RenderTarget::Builtin(Tool::Claude));
    let subs = substitutions(target, spec_dir, location);
    let mut fm = String::from("---\n");
    fm.push_str(&format!("name: speclink-{}\n", skill.name));
    // A YAML double-quoted scalar: a spec_dir may bring a backslash or a quote.
    let description = description_for(skill, spec_dir).replace('\\', "\\\\").replace('"', "\\\"");
    fm.push_str(&format!("description: \"{description}\"\n"));
    if claude {
        if skill.fork {
            fm.push_str("context: fork\n");
            fm.push_str("agent: Explore\n");
        }
        if skill.disallow_edit {
            fm.push_str("disallowedTools: [Edit, Write]\n");
        }
    }
    fm.push_str("license: MIT\n");
    fm.push_str("compatibility: Requires speclink CLI.\n");
    fm.push_str("metadata:\n");
    fm.push_str("  author: speclink\n");
    fm.push_str(&format!("  version: \"{ASSET_VERSION}\"\n"));
    fm.push_str("  generatedBy: \"Speclink\"\n");
    fm.push_str("---\n\n");
    match target {
        RenderTarget::Builtin(_) => {
            if claude && skill.fork {
                if let Some(preamble) = fork_context(skill.name) {
                    fm.push_str(&preamble);
                }
            }
        }
        RenderTarget::Custom(custom) => fm.push_str(invocation_note(custom.invocation)),
    }
    fm.push_str(subs.apply("conversation-language.md", CONVERSATION_LANGUAGE).trim_end());
    fm.push_str("\n\n");
    fm.push_str(&subs.apply(skill.name, skill.body));
    // Exactly one trailing newline (frozen output shape), regardless of asset file endings.
    while fm.ends_with("\n\n") {
        fm.pop();
    }
    if !fm.ends_with('\n') {
        fm.push('\n');
    }
    fm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spec_dir_with_quotes_or_backslashes_keeps_the_description_valid_yaml() {
        let manual = registry().into_iter().find(|s| s.name == "manual").unwrap();
        let rendered = render_skill_file_for(RenderTarget::Builtin(Tool::Claude), &manual, r#"docs\"specs"#, SpecLocation::InCheckout);
        let front = rendered.split("---\n").nth(1).expect("frontmatter");
        let yaml: serde_yaml::Value = serde_yaml::from_str(front).expect("frontmatter parses");
        let description = yaml["description"].as_str().unwrap();
        assert!(description.contains(r#"docs\"specs/manual/"#), "{description}");
    }
    use crate::config::Invocation;
    use crate::init::ASSET_VERSION;

    fn version_line(rendered: &str) -> &str {
        rendered
            .lines()
            .find(|l| l.trim_start().starts_with("version:"))
            .unwrap_or_else(|| panic!("no version line in frontmatter:\n{rendered}"))
    }

    fn custom_target() -> CustomTool {
        CustomTool {
            name: "my-harness".to_string(),
            skills_dir: ".my-harness/skills".to_string(),
            instructions_file: None,
            invocation: Invocation::Cli,
        }
    }

    /// Spec requirement: 產物層版本戳同源 — the skill frontmatter version is the
    /// ASSET_VERSION string, never a hardcoded "1.0", across all three render targets.
    #[test]
    fn skill_frontmatter_version_is_asset_version() {
        let expected = format!("  version: \"{ASSET_VERSION}\"");
        let custom = custom_target();
        for skill in registry() {
            for rendered in [
                render_skill_file_for(RenderTarget::Builtin(Tool::Claude), &skill, "openspec", SpecLocation::InCheckout),
                render_skill_file_for(RenderTarget::Builtin(Tool::Codex), &skill, "openspec", SpecLocation::InCheckout),
                render_skill_file_for(RenderTarget::Custom(&custom), &skill, "openspec", SpecLocation::InCheckout),
            ] {
                assert_eq!(
                    version_line(&rendered),
                    expected,
                    "skill '{}' frontmatter version must equal ASSET_VERSION",
                    skill.name
                );
            }
        }
    }

    #[test]
    fn skill_frontmatter_has_no_hardcoded_version() {
        let custom = custom_target();
        for skill in registry() {
            for rendered in [
                render_skill_file_for(RenderTarget::Builtin(Tool::Claude), &skill, "openspec", SpecLocation::InCheckout),
                render_skill_file_for(RenderTarget::Builtin(Tool::Codex), &skill, "openspec", SpecLocation::InCheckout),
                render_skill_file_for(RenderTarget::Custom(&custom), &skill, "openspec", SpecLocation::InCheckout),
            ] {
                assert!(
                    !rendered.contains("  version: \"1.0\""),
                    "skill '{}' still carries the hardcoded \"1.0\" version stamp",
                    skill.name
                );
            }
        }
    }

    /// spec skill-authoring「技能原稿的跨模型寫法規範」Example「同一個引用在六種渲染
    /// 目標」的內建五列：原稿的 `/speclink:<名稱>` 依工具換成各自的叫用寫法。
    #[test]
    fn a_skill_reference_renders_per_builtin_tool() {
        let rendered = Tool::ALL.map(|tool| substitute("/speclink:ingest", RenderTarget::Builtin(tool), "speclink", SpecLocation::InCheckout));
        assert_eq!(
            rendered,
            [
                "/speclink-ingest",
                "$speclink-ingest",
                "/speclink-ingest",
                "/skill:speclink-ingest",
                "/speclink-ingest",
            ]
        );
    }

    /// `skill_body` 是 registry 的查表加三個內部技能（tdd、clarify、sync）：每個對外
    /// 技能都解析到 registry 同一份 body，內部技能可取、未知名稱回 None。
    #[test]
    fn skill_body_resolves_every_registered_skill_and_the_internal_ones() {
        for skill in registry() {
            assert_eq!(
                skill_body(skill.name),
                Some(skill.body),
                "skill '{}' must resolve to its registry body",
                skill.name
            );
        }
        for internal in ["tdd", "clarify", "sync"] {
            assert!(skill_body(internal).is_some(), "internal skill '{internal}' must resolve");
        }
        assert_eq!(skill_body("no-such-skill"), None);
    }

    /// spec manual-skill「技能檔的渲染」＋ skill-routing「入口路由由技能描述承載」：
    /// manual 是工具技能——三目標皆渲染、可寫檔（生成模式要寫 manual 頁）、不 fork、
    /// 不受 worktree 政策閘門；description 以觸發情境句開場。
    #[test]
    fn manual_is_registered_as_a_writable_tool_skill_for_every_target() {
        let reg = registry();
        let manual = reg
            .iter()
            .find(|s| s.name == "manual")
            .expect("registry carries the manual skill");
        assert!(
            !manual.claude_only,
            "manual renders for codex (and thus neutral) targets"
        );
        assert!(!manual.fork, "generation mode writes files — not a fork skill");
        assert!(!manual.disallow_edit, "generation mode writes files — Edit/Write allowed");
        assert!(!manual.worktree_gated, "manual is not part of the worktree flow");
        assert!(
            manual.description.starts_with("Use when "),
            "description opens with the trigger situation: {}",
            manual.description
        );
        assert!(
            manual.description.contains("manual") && manual.description.contains("walk"),
            "description names both entry situations (manual wanted / wants a walk-through): {}",
            manual.description
        );
        assert_eq!(
            skill_body("manual"),
            Some(manual.body),
            "skill_body resolves manual to the same embedded asset"
        );
    }

    /// spec review-skill「Standards 軸的 smell baseline」：引擎提供的 12 個名稱與每個內建工具
    /// 生成的審查技能檔內嵌的 smell 清單（`- **<name>** — … → …` 項目）逐名、逐序一致——
    /// 任一端改名或增刪即失敗。
    #[test]
    fn review_baseline_smells_match_the_generated_review_skill_files() {
        let reg = registry();
        let review = reg.iter().find(|s| s.name == "review").expect("registry carries review");
        for tool in [Tool::Claude, Tool::Codex, Tool::Copilot] {
            let rendered = render_skill_file_for(RenderTarget::Builtin(tool), review, "openspec", SpecLocation::InCheckout);
            let names: Vec<&str> = rendered
                .lines()
                .map(str::trim_start)
                .filter(|l| l.starts_with("- **") && l.contains("** — ") && l.contains(" → "))
                .map(|l| l["- **".len()..].split("**").next().unwrap())
                .collect();
            assert_eq!(names, REVIEW_BASELINE_SMELLS, "generated review skill for {tool:?}");
        }
    }

    /// spec review-skill「首輪工單記錄套用的規則」Scenario 技能檔載明記錄規則：三種工具
    /// 生成的審查技能檔，記錄輪次的步驟在 Patch 與 Scope 之間寫 `**Rules**:` 與
    /// `**Rule**:`，並寫明 validation 輪不寫。
    #[test]
    fn review_skill_records_applied_rules_in_discovery_rounds() {
        let reg = registry();
        let review = reg.iter().find(|s| s.name == "review").expect("registry carries review");
        for tool in [Tool::Claude, Tool::Codex, Tool::Copilot] {
            let rendered = render_skill_file_for(RenderTarget::Builtin(tool), review, "openspec", SpecLocation::InCheckout);
            let step: Vec<&str> = rendered
                .lines()
                .skip_while(|l| !l.contains("**Record the round**"))
                .take_while(|l| !l.contains("**Branch on the blocking set**"))
                .collect();
            let bullet = |needle: &str| {
                step.iter()
                    .position(|l| l.trim_start().starts_with("- ") && l.contains(needle))
                    .unwrap_or_else(|| panic!("{tool:?}: no stdin bullet for {needle}"))
            };
            let (patch, rules, scope) = (bullet("`**Patch**:`"), bullet("`**Rules**:`"), bullet("`**Scope**:`"));
            assert!(patch < rules && rules < scope, "{tool:?}: Rules sits between Patch and Scope");
            assert!(step[rules].contains("`**Rule**:"), "{tool:?}: the Rules bullet names the Rule lines");
            let text = step.join("\n");
            assert!(
                text.contains("A validation round writes no `**Rules**:` or `**Rule**:` line"),
                "{tool:?}: the step says validation rounds skip the record"
            );
        }
    }

    // --- 條件區塊（spec skill-authoring「原稿的條件區塊」） ---

    const BLOCK_SAMPLE: &str = "plain\n{{#SPEC_IN_CHECKOUT}}\nin the checkout\n{{/SPEC_IN_CHECKOUT}}\n{{^SPEC_IN_CHECKOUT}}\non the server\n{{/SPEC_IN_CHECKOUT}}\n";

    /// Scenario「兩種情況各留下自己的區塊」：記號行不輸出。
    #[test]
    fn skills_blocks_keep_only_the_side_that_matches() {
        assert_eq!(resolve_blocks("sample.md", BLOCK_SAMPLE, SpecLocation::InCheckout).unwrap(), "plain\nin the checkout\n");
        assert_eq!(resolve_blocks("sample.md", BLOCK_SAMPLE, SpecLocation::Elsewhere).unwrap(), "plain\non the server\n");
    }

    /// 區塊可以包住清單項目與程式碼區塊：記號行可以縮排。
    #[test]
    fn skills_blocks_may_be_indented_and_wrap_code_fences() {
        let source = "1. Step\n   {{#SPEC_IN_CHECKOUT}}\n   ```bash\n   git add x\n   ```\n   {{/SPEC_IN_CHECKOUT}}\n2. Next\n";
        assert_eq!(
            resolve_blocks("sample.md", source, SpecLocation::InCheckout).unwrap(),
            "1. Step\n   ```bash\n   git add x\n   ```\n2. Next\n"
        );
        assert_eq!(resolve_blocks("sample.md", source, SpecLocation::Elsewhere).unwrap(), "1. Step\n2. Next\n");
    }

    /// Scenario「沒有關閉的區塊」「巢狀的區塊」「不認得的條件名稱」，加上沒有開啟的
    /// 關閉記號與不獨佔一行的記號：每一種都回報錯誤，指出原稿與行號。
    #[test]
    fn skills_blocks_report_every_malformed_marker_with_source_and_line() {
        let cases = [
            ("unclosed", "a\n{{#SPEC_IN_CHECKOUT}}\nb\n", 2, "never closed"),
            (
                "nested",
                "{{#SPEC_IN_CHECKOUT}}\na\n{{^SPEC_IN_CHECKOUT}}\nb\n{{/SPEC_IN_CHECKOUT}}\n{{/SPEC_IN_CHECKOUT}}\n",
                3,
                "nested",
            ),
            ("unknown name", "a\nb\n{{#REMOTE}}\nc\n{{/REMOTE}}\n", 3, "REMOTE"),
            ("close without open", "a\n{{/SPEC_IN_CHECKOUT}}\n", 2, "no open block"),
            ("marker inside a sentence", "Do it {{#SPEC_IN_CHECKOUT}} here\n", 1, "alone on its line"),
        ];
        for (label, source, line, problem) in cases {
            for location in SpecLocation::BOTH {
                let err = resolve_blocks("sample.md", source, location)
                    .expect_err(&format!("{label}: must be an error"));
                assert_eq!((err.source.as_str(), err.line), ("sample.md", line), "{label}: {err}");
                assert!(err.problem.contains(problem), "{label}: {err}");
                assert!(err.to_string().starts_with(&format!("sample.md:{line}: ")), "{label}: {err}");
            }
        }
    }

    /// Scenario「CRLF 的原稿」：兩種行結尾得到相同的結果。
    #[test]
    fn skills_blocks_give_the_same_result_for_lf_and_crlf() {
        let crlf = BLOCK_SAMPLE.replace('\n', "\r\n");
        for location in SpecLocation::BOTH {
            assert_eq!(
                resolve_blocks("sample.md", &crlf, location).unwrap().replace("\r\n", "\n"),
                resolve_blocks("sample.md", BLOCK_SAMPLE, location).unwrap()
            );
        }
    }

    /// 「區塊的處理同樣套用於被插入的共用片段」：片段自己的區塊也只留下相符的那一邊。
    #[test]
    fn skills_blocks_apply_to_an_inserted_fragment() {
        let body = "before\n{{FRAGMENT}}\nafter\n";
        assert_eq!(
            with_fragment(body, "{{FRAGMENT}}", "fragment.md", BLOCK_SAMPLE, SpecLocation::InCheckout),
            "before\nplain\nin the checkout\nafter\n"
        );
        assert_eq!(
            with_fragment(body, "{{FRAGMENT}}", "fragment.md", BLOCK_SAMPLE, SpecLocation::Elsewhere),
            "before\nplain\non the server\nafter\n"
        );
    }

    /// 「渲染結果不殘留 `{{`」：每一支技能、每一種目標、兩種情況都渲染一次——原稿的
    /// 區塊寫壞時，這裡的渲染直接失敗並指出原稿與行號。
    #[test]
    fn skills_render_in_both_cases_without_a_leftover_placeholder() {
        let cli = custom_target();
        let tool_call = CustomTool { invocation: Invocation::ToolCall, ..custom_target() };
        for skill in registry() {
            let builtins = Tool::ALL.into_iter().map(RenderTarget::Builtin);
            for target in builtins.chain([RenderTarget::Custom(&cli), RenderTarget::Custom(&tool_call)]) {
                for location in SpecLocation::BOTH {
                    let rendered = render_skill_file_for(target, &skill, "openspec", location);
                    let leftover = rendered.lines().find(|l| l.contains("{{"));
                    assert_eq!(leftover, None, "skill '{}' ({location:?})", skill.name);
                }
            }
        }
    }

    /// 鎖定基準（重構前後皆須成立）：三種 render target 的輸出外形——恰一個結尾
    /// 換行、frontmatter 版號等於 `ASSET_VERSION`、前言只出現在該出現的 target。
    #[test]
    fn every_render_target_keeps_its_frontmatter_and_preamble_shape() {
        let reg = registry();
        let drift = reg.iter().find(|s| s.name == "drift").expect("registry carries drift");
        let apply = reg.iter().find(|s| s.name == "apply").expect("registry carries apply");
        let cli = custom_target();
        let tool_call = CustomTool { invocation: Invocation::ToolCall, ..custom_target() };

        let claude_fork = render_skill_file_for(RenderTarget::Builtin(Tool::Claude), drift, "openspec", SpecLocation::InCheckout);
        let codex = render_skill_file_for(RenderTarget::Builtin(Tool::Codex), apply, "openspec", SpecLocation::InCheckout);
        let custom_cli = render_skill_file_for(RenderTarget::Custom(&cli), apply, "openspec", SpecLocation::InCheckout);
        let custom_tool_call =
            render_skill_file_for(RenderTarget::Custom(&tool_call), apply, "openspec", SpecLocation::InCheckout);

        for (label, rendered) in [
            ("claude/drift", &claude_fork),
            ("codex/apply", &codex),
            ("custom-cli/apply", &custom_cli),
            ("custom-tool-call/apply", &custom_tool_call),
        ] {
            assert!(rendered.ends_with('\n'), "{label} must end with a newline");
            assert!(!rendered.ends_with("\n\n"), "{label} must end with exactly one newline");
            assert_eq!(
                version_line(rendered),
                format!("  version: \"{ASSET_VERSION}\""),
                "{label} frontmatter version must equal ASSET_VERSION"
            );
        }

        assert!(
            claude_fork.contains("## Claude fork context"),
            "the Claude render of a fork skill carries the fork preamble"
        );
        for (label, rendered) in [
            ("codex/apply", &codex),
            ("custom-cli/apply", &custom_cli),
            ("custom-tool-call/apply", &custom_tool_call),
        ] {
            assert!(
                !rendered.contains("## Claude fork context"),
                "{label} must not carry the Claude-only fork preamble"
            );
        }

        for (label, rendered) in [("claude/drift", &claude_fork), ("codex/apply", &codex)] {
            assert!(
                !rendered.contains("## Invocation"),
                "{label} is a built-in render — no invocation preamble"
            );
        }
        for (label, rendered) in
            [("custom-cli/apply", &custom_cli), ("custom-tool-call/apply", &custom_tool_call)]
        {
            assert!(
                rendered.contains("## Invocation"),
                "{label} carries the descriptor invocation preamble"
            );
        }
    }
}
