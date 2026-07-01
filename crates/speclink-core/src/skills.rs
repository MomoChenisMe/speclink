//! Skill registry, embedded bodies, and rendering (frontmatter + placeholder substitution).

/// A tool target for generated skills.
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
    /// Read-only fork skill (context: fork + agent: Explore).
    pub fork: bool,
    /// Whether to add `disallowedTools: [Edit, Write]`.
    pub disallow_edit: bool,
    /// Whether generated for the Codex (.agents) target.
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

/// Render a complete SKILL.md (frontmatter + substituted body) for a tool.
pub fn render_skill_file(skill: &Skill, tool: Tool, spec_dir: &str) -> String {
    let mut fm = String::from("---\n");
    fm.push_str(&format!("name: speclink-{}\n", skill.name));
    fm.push_str(&format!("description: \"{}\"\n", skill.description));
    if skill.fork {
        fm.push_str("context: fork\n");
        fm.push_str("agent: Explore\n");
    }
    if skill.disallow_edit {
        fm.push_str("disallowedTools: [Edit, Write]\n");
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
