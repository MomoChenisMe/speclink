//! Skill registry, embedded bodies, and rendering (frontmatter + placeholder substitution).

use crate::init::MARKER_VERSION;

/// A tool target for generated skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Claude,
    Codex,
    Cursor,
    Gemini,
    Windsurf,
}

impl Tool {
    pub fn parse(s: &str) -> Option<Tool> {
        match s.trim().to_ascii_lowercase().as_str() {
            "claude" => Some(Tool::Claude),
            "codex" | "agents" => Some(Tool::Codex),
            "cursor" => Some(Tool::Cursor),
            "gemini" => Some(Tool::Gemini),
            "windsurf" => Some(Tool::Windsurf),
            _ => None,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
            Tool::Cursor => "cursor",
            Tool::Gemini => "gemini",
            Tool::Windsurf => "windsurf",
        }
    }
    /// Directory (relative to project root) that holds generated skill files.
    pub fn skills_dir(&self) -> &'static str {
        match self {
            Tool::Claude => ".claude/skills",
            Tool::Codex => ".agents/skills",
            Tool::Cursor => ".cursor/skills",
            Tool::Gemini => ".gemini/skills",
            Tool::Windsurf => ".windsurf/skills",
        }
    }
    /// Where the tool keeps plan-mode files ({{PLAN_DIR}}); empty when the tool has none.
    fn plan_dir(&self) -> &'static str {
        match self {
            Tool::Claude => "~/.claude/plans/",
            Tool::Codex => "",
            Tool::Cursor => ".cursor/plans/",
            Tool::Gemini => "",
            Tool::Windsurf => "~/.windsurf/plans/",
        }
    }
    /// The prefix that `/speclink:` becomes for this tool. Cursor/Gemini/Windsurf keep the
    /// colon form verbatim (matches Spectra).
    fn slash_replacement(&self) -> &'static str {
        match self {
            Tool::Claude => "/speclink-",
            Tool::Codex => "$speclink-",
            Tool::Cursor | Tool::Gemini | Tool::Windsurf => "/speclink:",
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
const B_DISCUSS: &str = include_str!("../assets/skills/discuss.md");
const B_DRIFT: &str = include_str!("../assets/skills/drift.md");
const B_INGEST: &str = include_str!("../assets/skills/ingest.md");
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
        Skill { name: "audit", description: "Audit changed code for security sharp edges — dangerous defaults, type confusion, and silent failures", fork: true, disallow_edit: true, for_codex: true, body: B_AUDIT },
        Skill { name: "commit", description: "Commit files related to a specific Speclink change", fork: false, disallow_edit: false, for_codex: true, body: B_COMMIT },
        Skill { name: "discuss", description: "Have a focused discussion that is recorded to a discussion document", fork: false, disallow_edit: true, for_codex: true, body: B_DISCUSS },
        Skill { name: "drift", description: "Detect drift between a Speclink change and the current codebase state", fork: true, disallow_edit: true, for_codex: true, body: B_DRIFT },
        Skill { name: "ingest", description: "Update an existing Speclink change from external context", fork: false, disallow_edit: false, for_codex: true, body: B_INGEST },
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

/// Render a complete SKILL.md (frontmatter + substituted body) for a tool. The fork/agent and
/// disallowedTools lines are Claude-only (matches Spectra).
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
    fm.push_str("  version: \"1.0\"\n");
    fm.push_str("  generatedBy: \"Speclink\"\n");
    fm.push_str("---\n\n");
    fm.push_str(&substitute(skill.body, tool, spec_dir));
    fm
}

fn marker_start() -> String {
    format!("<!-- SPECLINK:START {MARKER_VERSION} -->")
}
const MARKER_END: &str = "<!-- SPECLINK:END -->";

fn wrapped_body(skill: &Skill, tool: Tool, spec_dir: &str) -> String {
    let body = substitute(skill.body, tool, spec_dir);
    format!("{}\n\n{}\n\n{MARKER_END}\n", marker_start(), body.trim_end())
}

/// The command-file description — a few skills use a shorter/different wording than their
/// SKILL.md description (matches Spectra's command frontmatter).
fn command_description(skill: &Skill) -> &'static str {
    match skill.name {
        "apply" => "Implement tasks from a Speclink change",
        "ingest" => "Update an existing Speclink change from a plan file or conversation context",
        "propose" => "Create a complete change proposal with all artifacts in a single workflow",
        _ => skill.description,
    }
}

/// Command category — capitalized first tag (audit → Development, commit → Utility).
fn command_category(name: &str) -> &'static str {
    match name {
        "audit" => "Development",
        "commit" => "Utility",
        _ => "Workflow",
    }
}

/// Cursor slash-command file (`.cursor/commands/speclink-<name>.md`).
pub fn render_cursor_command(skill: &Skill, spec_dir: &str) -> String {
    format!(
        "---\nname: /speclink-{n}\nid: speclink-{n}\ncategory: {c}\ndescription: {d}\n---\n\n{body}",
        n = skill.name,
        c = command_category(skill.name),
        d = command_description(skill),
        body = wrapped_body(skill, Tool::Cursor, spec_dir),
    )
}

/// Gemini command TOML (`.gemini/commands/speclink/<name>.toml`).
pub fn render_gemini_toml(skill: &Skill, spec_dir: &str) -> String {
    format!(
        "description = \"{d}\"\n\nprompt = \"\"\"\n{body}\"\"\"\n",
        d = command_description(skill),
        body = wrapped_body(skill, Tool::Gemini, spec_dir),
    )
}

/// The short rules file written to `.cursorrules` / `.windsurfrules` (marker-wrapped).
pub fn render_rules_file(spec_dir: &str) -> String {
    let sd = spec_dir.trim_end_matches('/');
    format!(
        "{start}\n\n# Speclink Instructions\n\nThis project uses Speclink for Spec-Driven Development(SDD).\n\n## Directory Structure\n\n- **Specs**: `{sd}/specs/` - Current truth, what IS built\n- **Changes**: `{sd}/changes/` - Proposals, what SHOULD change\n- **Archive**: `{sd}/changes/archive/` - Completed changes\n- **Config**: `{sd}/config.yaml` - Project context and rules\n\n## Workflow\n\ndiscuss? → propose → apply ⇄ ingest → archive\n\n{end}\n",
        start = marker_start(),
        end = MARKER_END,
    )
}

/// Windsurf workflow tags per command skill (matches Spectra's tag sets).
fn windsurf_tags(name: &str) -> &'static str {
    match name {
        "apply" => "[\"workflow\", \"artifacts\"]",
        "archive" => "[\"workflow\", \"archive\"]",
        "audit" => "[\"development\", \"security\", \"audit\"]",
        "commit" => "[\"utility\", \"git\", \"commit\"]",
        "discuss" => "[\"workflow\", \"discuss\", \"thinking\"]",
        "drift" => "[\"workflow\", \"drift\", \"diagnose\"]",
        "ingest" => "[\"workflow\", \"import\", \"plan\", \"claude\"]",
        "propose" => "[\"workflow\", \"propose\", \"artifacts\"]",
        _ => "[\"workflow\"]",
    }
}

/// Windsurf workflow file (`.windsurf/workflows/speclink-<name>.md`).
pub fn render_windsurf_workflow(skill: &Skill, spec_dir: &str) -> String {
    let mut title: Vec<char> = skill.name.chars().collect();
    if let Some(c) = title.first_mut() {
        *c = c.to_ascii_uppercase();
    }
    let title: String = title.into_iter().collect();
    format!(
        "---\nname: Speclink: {title}\ndescription: {d}\ncategory: {c}\ntags: {tags}\n---\n\n{body}",
        d = command_description(skill),
        c = command_category(skill.name),
        tags = windsurf_tags(skill.name),
        body = wrapped_body(skill, Tool::Windsurf, spec_dir),
    )
}
