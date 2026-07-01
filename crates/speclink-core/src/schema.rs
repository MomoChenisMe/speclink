//! Embedded workflow schema (`spec-driven`), artifacts, and their assets.

/// A single artifact definition within a schema.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: &'static str,
    /// Output path pattern relative to the change directory.
    pub output_path: &'static str,
    /// Template file name (shown by `templates`).
    pub template_name: &'static str,
    pub description: &'static str,
    /// Artifact ids that must be done before this one is ready.
    pub requires: &'static [&'static str],
    pub instruction: &'static str,
    pub template: &'static str,
}

/// A workflow schema.
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: &'static str,
    pub description: &'static str,
    pub source: &'static str,
    pub artifacts: Vec<Artifact>,
    /// Artifact ids required before `apply`.
    pub apply_requires: &'static [&'static str],
    pub apply_instruction: &'static str,
}

impl Schema {
    pub fn artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }
    pub fn artifact_ids(&self) -> Vec<&'static str> {
        self.artifacts.iter().map(|a| a.id).collect()
    }
}

const A_PROPOSAL_INSTR: &str =
    include_str!("../assets/schema/spec-driven/proposal.instruction.md");
const A_PROPOSAL_TMPL: &str = include_str!("../assets/schema/spec-driven/proposal.template.md");
const A_SPECS_INSTR: &str = include_str!("../assets/schema/spec-driven/specs.instruction.md");
const A_SPECS_TMPL: &str = include_str!("../assets/schema/spec-driven/specs.template.md");
const A_DESIGN_INSTR: &str = include_str!("../assets/schema/spec-driven/design.instruction.md");
const A_DESIGN_TMPL: &str = include_str!("../assets/schema/spec-driven/design.template.md");
const A_TASKS_INSTR: &str = include_str!("../assets/schema/spec-driven/tasks.instruction.md");
const A_TASKS_TMPL: &str = include_str!("../assets/schema/spec-driven/tasks.template.md");
const A_APPLY_INSTR: &str = include_str!("../assets/schema/spec-driven/apply.instruction.md");

/// The built-in `spec-driven` schema.
pub fn spec_driven() -> Schema {
    Schema {
        name: "spec-driven",
        description: "Default OpenSpec workflow - proposal → specs → design → tasks",
        source: "package",
        apply_requires: &["tasks"],
        apply_instruction: A_APPLY_INSTR,
        artifacts: vec![
            Artifact {
                id: "proposal",
                output_path: "proposal.md",
                template_name: "proposal.md",
                description: "Initial proposal document outlining the change",
                requires: &[],
                instruction: A_PROPOSAL_INSTR,
                template: A_PROPOSAL_TMPL,
            },
            Artifact {
                id: "specs",
                output_path: "specs/**/*.md",
                template_name: "spec.md",
                description: "Detailed specifications for the change",
                requires: &["proposal"],
                instruction: A_SPECS_INSTR,
                template: A_SPECS_TMPL,
            },
            Artifact {
                id: "design",
                output_path: "design.md",
                template_name: "design.md",
                description: "Technical design document with implementation details",
                requires: &["proposal"],
                instruction: A_DESIGN_INSTR,
                template: A_DESIGN_TMPL,
            },
            Artifact {
                id: "tasks",
                output_path: "tasks.md",
                template_name: "tasks.md",
                description: "Implementation checklist with trackable tasks",
                requires: &["specs"],
                instruction: A_TASKS_INSTR,
                template: A_TASKS_TMPL,
            },
        ],
    }
}

/// Resolve a schema by name. Only `spec-driven` is built in.
pub fn resolve(name: &str) -> Option<Schema> {
    match name {
        "spec-driven" => Some(spec_driven()),
        _ => None,
    }
}

/// All available schemas.
pub fn all() -> Vec<Schema> {
    vec![spec_driven()]
}
