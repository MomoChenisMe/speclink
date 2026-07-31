//! Skill registry, embedded bodies, and rendering (frontmatter + placeholder substitution).

use crate::config::{CustomTool, Invocation};
use crate::init::MARKER_VERSION;

/// The three render targets: built-in claude, built-in codex, or a custom descriptor.
/// Descriptors render the NEUTRAL body: no tool-specific slash prefix, no plan-mode
/// references, verb wording decided by the descriptor's `invocation`.
#[derive(Clone, Copy)]
pub enum RenderTarget<'a> {
    Builtin(Tool),
    Custom(&'a CustomTool),
}

/// A tool target for generated skills. Speclink deliberately scopes the tool matrix to
/// claude + codex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Claude,
    Codex,
}

impl Tool {
    pub fn parse(s: &str) -> Option<Tool> {
        match s.trim().to_ascii_lowercase().as_str() {
            "claude" => Some(Tool::Claude),
            "codex" | "agents" => Some(Tool::Codex),
            _ => None,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
        }
    }
    /// Directory (relative to project root) that holds generated skill files.
    pub fn skills_dir(&self) -> &'static str {
        match self {
            Tool::Claude => ".claude/skills",
            Tool::Codex => ".agents/skills",
        }
    }
    /// Where the tool keeps plan-mode files ({{PLAN_DIR}}); empty when the tool has none.
    fn plan_dir(&self) -> &'static str {
        match self {
            Tool::Claude => "~/.claude/plans/",
            Tool::Codex => "",
        }
    }
    /// The prefix that `/speclink:` becomes for this tool.
    fn slash_replacement(&self) -> &'static str {
        match self {
            Tool::Claude => "/speclink-",
            Tool::Codex => "$speclink-",
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
    /// Part of the command subset generated for non-Claude tools (codex/cursor/gemini/windsurf
    /// skills, and the cursor/gemini/windsurf command files).
    pub for_codex: bool,
    pub body: &'static str,
}

// Embedded bodies.
const B_ANALYZE: &str = include_str!("../assets/skills/analyze.md");
const B_APPLY: &str = include_str!("../assets/skills/apply.md");
const B_ARCHIVE: &str = include_str!("../assets/skills/archive.md");
const B_AUDIT: &str = include_str!("../assets/skills/audit.md");
const B_COMMIT: &str = include_str!("../assets/skills/commit.md");
const B_CONFIG: &str = include_str!("../assets/skills/config.md");
const B_DISCUSS: &str = include_str!("../assets/skills/discuss.md");
const B_DRIFT: &str = include_str!("../assets/skills/drift.md");
const B_INGEST: &str = include_str!("../assets/skills/ingest.md");
const B_ONBOARD: &str = include_str!("../assets/skills/onboard.md");
const B_PROPOSE: &str = include_str!("../assets/skills/propose.md");
const B_VERIFY: &str = include_str!("../assets/skills/verify.md");
const B_SYNC: &str = include_str!("../assets/skills/sync.md");
const B_CLARIFY: &str = include_str!("../assets/skills/clarify.md");
const B_TDD: &str = include_str!("../assets/skills/tdd.md");

/// The skills that generate SKILL.md files.
pub fn registry() -> Vec<Skill> {
    vec![
        Skill { name: "analyze", description: "Analyze artifact consistency for a change", fork: true, disallow_edit: true, for_codex: false, body: B_ANALYZE },
        Skill { name: "apply", description: "Implement or resume tasks from a Speclink change", fork: false, disallow_edit: false, for_codex: true, body: B_APPLY },
        Skill { name: "archive", description: "Archive a completed change", fork: false, disallow_edit: false, for_codex: true, body: B_ARCHIVE },
        // Not a fork skill: the rewritten standalone mode fans out three
        // parallel audit agents, which the fork's Explore agent cannot spawn.
        Skill { name: "audit", description: "Audit changed code for security sharp edges — dangerous defaults, type confusion, and silent failures", fork: false, disallow_edit: true, for_codex: true, body: B_AUDIT },
        Skill { name: "commit", description: "Commit files related to a specific Speclink change", fork: false, disallow_edit: false, for_codex: true, body: B_COMMIT },
        // Writes only through `speclink workflow-config` (never a direct file
        // edit), so Edit/Write stay disallowed; not a fork skill because the
        // policy fields must be asked for interactively.
        Skill { name: "config", description: "Compose the workflow config's context and rules from the codebase, landed through an approved diff", fork: false, disallow_edit: true, for_codex: true, body: B_CONFIG },
        Skill { name: "discuss", description: "Have a focused discussion that is recorded to a discussion document", fork: false, disallow_edit: true, for_codex: true, body: B_DISCUSS },
        Skill { name: "drift", description: "Detect drift between a Speclink change and the current codebase state", fork: true, disallow_edit: true, for_codex: true, body: B_DRIFT },
        Skill { name: "ingest", description: "Update an existing Speclink change from external context", fork: false, disallow_edit: false, for_codex: true, body: B_INGEST },
        Skill { name: "onboard", description: "Adopt Speclink on an existing codebase by generating initial specs from current behavior", fork: false, disallow_edit: false, for_codex: true, body: B_ONBOARD },
        Skill { name: "propose", description: "Create a change proposal with all required artifacts", fork: false, disallow_edit: false, for_codex: true, body: B_PROPOSE },
        Skill { name: "verify", description: "Verify implementation matches artifacts", fork: true, disallow_edit: true, for_codex: false, body: B_VERIFY },
    ]
}

/// Lookup any embedded skill body (including internal ones) by name.
pub fn skill_body(name: &str) -> Option<&'static str> {
    Some(match name {
        "analyze" => B_ANALYZE,
        "apply" => B_APPLY,
        "archive" => B_ARCHIVE,
        "audit" => B_AUDIT,
        "commit" => B_COMMIT,
        "config" => B_CONFIG,
        "discuss" => B_DISCUSS,
        "drift" => B_DRIFT,
        "ingest" => B_INGEST,
        "propose" => B_PROPOSE,
        "verify" => B_VERIFY,
        "sync" => B_SYNC,
        "clarify" => B_CLARIFY,
        "tdd" => B_TDD,
        _ => return None,
    })
}

/// Substitute placeholders in a skill body for a given tool and spec dir.
pub fn substitute(body: &str, tool: Tool, spec_dir: &str) -> String {
    let spec_dir_slash = if spec_dir.ends_with('/') {
        spec_dir.to_string()
    } else {
        format!("{spec_dir}/")
    };
    body.replace("{{SPEC_DIR}}", &spec_dir_slash)
        .replace("{{PLAN_DIR}}", tool.plan_dir())
        .replace("{{TOOL}}", tool.name())
        .replace("/speclink:", tool.slash_replacement())
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

/// Unified skill rendering across the three targets.
pub fn render_skill_file_for(target: RenderTarget, skill: &Skill, spec_dir: &str) -> String {
    match target {
        RenderTarget::Builtin(tool) => render_skill_file(skill, tool, spec_dir),
        RenderTarget::Custom(custom) => render_skill_file_custom(skill, custom, spec_dir),
    }
}

/// Substitute placeholders for the neutral (descriptor) target: `/speclink:apply` reads as
/// `speclink apply` (no slash prefix), lines referencing plan mode are dropped (descriptors
/// have no plan directory), and `{{TOOL}}` is the descriptor name.
pub fn substitute_neutral(body: &str, tool: &CustomTool, spec_dir: &str) -> String {
    let spec_dir_slash = if spec_dir.ends_with('/') {
        spec_dir.to_string()
    } else {
        format!("{spec_dir}/")
    };
    let without_plan_mode: Vec<&str> = body
        .lines()
        .filter(|l| !l.to_ascii_lowercase().contains("plan mode"))
        .collect();
    without_plan_mode
        .join("\n")
        .replace("{{SPEC_DIR}}", &spec_dir_slash)
        .replace("{{PLAN_DIR}}", "")
        .replace("{{TOOL}}", &tool.name)
        .replace("/speclink:", "speclink ")
        // Some bodies carry literal claude-style skill references; neutrally they are
        // plain skill names (`speclink-ingest`), never slash commands.
        .replace("/speclink-", "speclink-")
}

/// The invocation preamble that tells a custom harness how `speclink <verb>` references in
/// the body are meant to be executed.
fn invocation_note(invocation: Invocation) -> &'static str {
    match invocation {
        Invocation::Cli => {
            "## Invocation\n\nThis harness executes speclink verbs as shell commands: \
run `speclink <verb> [arguments]`.\n\n---\n\n"
        }
        Invocation::ToolCall => {
            "## Invocation\n\nThis harness executes speclink verbs by calling the speclink \
tool with an argv array (e.g. [\"apply\", \"add-auth\"]). Wherever this document says \
`speclink <verb> [arguments]`, it means calling the speclink tool with those arguments \
as argv.\n\n---\n\n"
        }
    }
}

/// Render a SKILL.md for a custom descriptor target: neutral frontmatter (no Claude-only
/// fork/disallowedTools lines), an invocation preamble, and the neutral body.
pub fn render_skill_file_custom(skill: &Skill, tool: &CustomTool, spec_dir: &str) -> String {
    let mut fm = String::from("---\n");
    fm.push_str(&format!("name: speclink-{}\n", skill.name));
    fm.push_str(&format!("description: \"{}\"\n", skill.description));
    fm.push_str("license: MIT\n");
    fm.push_str("compatibility: Requires speclink CLI.\n");
    fm.push_str("metadata:\n");
    fm.push_str("  author: speclink\n");
    fm.push_str(&format!("  version: \"{MARKER_VERSION}\"\n"));
    fm.push_str("  generatedBy: \"Speclink\"\n");
    fm.push_str("---\n\n");
    fm.push_str(invocation_note(tool.invocation));
    fm.push_str(&substitute_neutral(skill.body, tool, spec_dir));
    // Exactly one trailing newline, matching the built-in renders.
    while fm.ends_with("\n\n") {
        fm.pop();
    }
    if !fm.ends_with('\n') {
        fm.push('\n');
    }
    fm
}

/// Render a complete SKILL.md (frontmatter + substituted body) for a tool. The fork/agent and
/// disallowedTools lines are Claude-only (frozen output shape).
pub fn render_skill_file(skill: &Skill, tool: Tool, spec_dir: &str) -> String {
    let mut fm = String::from("---\n");
    fm.push_str(&format!("name: speclink-{}\n", skill.name));
    fm.push_str(&format!("description: \"{}\"\n", skill.description));
    if tool == Tool::Claude {
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
    fm.push_str(&format!("  version: \"{MARKER_VERSION}\"\n"));
    fm.push_str("  generatedBy: \"Speclink\"\n");
    fm.push_str("---\n\n");
    if tool == Tool::Claude && skill.fork {
        if let Some(preamble) = fork_context(skill.name) {
            fm.push_str(&preamble);
        }
    }
    fm.push_str(&substitute(skill.body, tool, spec_dir));
    // Exactly one trailing newline (frozen output shape), regardless of asset file endings.
    while fm.ends_with("\n\n") {
        fm.pop();
    }
    fm
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Invocation;
    use crate::init::MARKER_VERSION;

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
            instructions_file: "HARNESS.md".to_string(),
            invocation: Invocation::Cli,
        }
    }

    /// Spec requirement: 產物層版本戳同源 — the skill frontmatter version is the
    /// MARKER_VERSION string, never a hardcoded "1.0", across all three render targets.
    #[test]
    fn skill_frontmatter_version_is_marker_version() {
        let expected = format!("  version: \"{MARKER_VERSION}\"");
        let custom = custom_target();
        for skill in registry() {
            for rendered in [
                render_skill_file_for(RenderTarget::Builtin(Tool::Claude), &skill, "openspec"),
                render_skill_file_for(RenderTarget::Builtin(Tool::Codex), &skill, "openspec"),
                render_skill_file_for(RenderTarget::Custom(&custom), &skill, "openspec"),
            ] {
                assert_eq!(
                    version_line(&rendered),
                    expected,
                    "skill '{}' frontmatter version must equal MARKER_VERSION",
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
                render_skill_file_for(RenderTarget::Builtin(Tool::Claude), &skill, "openspec"),
                render_skill_file_for(RenderTarget::Builtin(Tool::Codex), &skill, "openspec"),
                render_skill_file_for(RenderTarget::Custom(&custom), &skill, "openspec"),
            ] {
                assert!(
                    !rendered.contains("  version: \"1.0\""),
                    "skill '{}' still carries the hardcoded \"1.0\" version stamp",
                    skill.name
                );
            }
        }
    }
}

