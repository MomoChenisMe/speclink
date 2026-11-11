//! Output of `instructions`: one artifact's guidance, or the apply payload.

use speclink_core::command::InstructionsOutcome;
use speclink_core::instructions::{ApplyInstructions, ArtifactInstructions};

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// `instructions`: the artifact form or the apply form, as the outcome says.
pub fn instructions_human(outcome: &InstructionsOutcome, paint: &Paint) -> Text {
    match outcome {
        InstructionsOutcome::Apply(p) => apply_human(p, paint),
        InstructionsOutcome::Artifact(p) => artifact_human(p, paint),
    }
}

/// `instructions --json` is the payload as the engine serializes it.
pub fn instructions(outcome: &InstructionsOutcome, options: RenderOptions) -> Rendered {
    let json = match outcome {
        InstructionsOutcome::Apply(p) => JsonForm::pretty(p),
        InstructionsOutcome::Artifact(p) => JsonForm::pretty(p),
    };
    Rendered::of(options, Some(json), |paint| instructions_human(outcome, paint))
}

/// One artifact's guidance: the header, then each section that has content.
pub fn artifact_human(p: &ArtifactInstructions, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{}: {}", paint.bold("Artifact"), p.artifact_id);
    outln!(t, "{}: {}", paint.bold("Output"), p.output_path);
    outln!(t, "{}: {}", paint.bold("Description"), p.description);
    // Each section is preceded by one blank separator and rendered only when non-empty
    // (a custom schema may have no instruction and an empty template) — frozen output shape.
    if let Some(instr) = &p.instruction {
        outln!(t);
        outln!(t, "{}", paint.bold("Instruction:"));
        out!(t, "{instr}"); // ends with a newline
        outln!(t);
    }
    if !p.dependencies.is_empty() {
        outln!(t);
        outln!(t, "{}", paint.bold("Dependencies:"));
        // Dependency symbols stay plain (probed — unlike status's colored ones).
        for d in &p.dependencies {
            let sym = if d.done { "✓" } else { "○" };
            outln!(t, "  {sym} {} ({})", d.id, d.path);
        }
    }
    if !p.unlocks.is_empty() {
        outln!(t);
        outln!(t, "{}", paint.bold("Unlocks:"));
        for u in &p.unlocks {
            outln!(t, "  - {u}");
        }
    }
    if !p.template.is_empty() {
        outln!(t);
        outln!(t, "{}", paint.bold("Template:"));
        out!(t, "{}", p.template);
        outln!(t);
    }
    t
}

/// The apply payload: the progress header, the tasks (or the missing
/// artifacts), then the instruction.
pub fn apply_human(p: &ApplyInstructions, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{}: {}", paint.bold("Change"), p.change_name);
    outln!(t, "{}: {}", paint.bold("Schema"), p.schema_name);
    outln!(t, "{}: {}", paint.bold("State"), p.state);
    outln!(t, "{}: {}/{} complete", paint.bold("Progress"), p.progress.complete, p.progress.total);
    outln!(t);
    if let Some(missing) = &p.missing_artifacts {
        outln!(t, "{}", paint.red("Missing artifacts:"));
        for m in missing {
            outln!(t, "  - {m}");
        }
    } else {
        outln!(t, "{}", paint.bold("Tasks:"));
        // Task symbols stay plain here (probed — unlike status's colored ones).
        for task in &p.tasks {
            let sym = if task.done { "✓" } else { "○" };
            outln!(t, "  {sym} {}", task.description);
        }
    }
    outln!(t);
    if let Some(instr) = &p.instruction {
        outln!(t, "{}", paint.bold("Instruction:"));
        out!(t, "{instr}");
        outln!(t);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::instructions::{Dependency, Progress, TaskJson};

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    fn apply(missing: Option<Vec<String>>) -> ApplyInstructions {
        let task = |id: &str, description: &str, done: bool| TaskJson {
            id: id.to_string(),
            description: description.to_string(),
            done,
            manual: false,
        };
        ApplyInstructions {
            change_name: "alpha".to_string(),
            change_dir: "/p/speclink/changes/alpha".to_string(),
            schema_name: "spec-driven".to_string(),
            context_files: [("tasks".to_string(), "/p/speclink/changes/alpha/tasks.md".to_string())].into(),
            progress: Progress {
                total: 2,
                complete: 1,
                remaining: 1,
                code_total: 2,
                code_complete: 1,
                code_remaining: 1,
            },
            tasks: vec![
                task("1", "1.1 Deliver behavior 1; verify with the unit test", true),
                task("2", "1.2 Deliver behavior 2; verify with the unit test", false),
            ],
            state: "ready".to_string(),
            missing_artifacts: missing,
            locale: "English".to_string(),
            tdd: false,
            audit: false,
            instruction: Some("Read context files, work through pending tasks.\n".to_string()),
            preflight: None,
        }
    }

    fn artifact() -> ArtifactInstructions {
        ArtifactInstructions {
            change_name: "alpha".to_string(),
            artifact_id: "design".to_string(),
            schema_name: "spec-driven".to_string(),
            change_dir: "/p/speclink/changes/alpha".to_string(),
            output_path: "design.md".to_string(),
            description: "Technical design document".to_string(),
            instruction: Some("Create the design document.\n".to_string()),
            context: None,
            rules: None,
            locale: "English".to_string(),
            template: "## Context\n".to_string(),
            dependencies: vec![
                Dependency {
                    id: "proposal".to_string(),
                    done: true,
                    path: "proposal.md".to_string(),
                    description: "Why".to_string(),
                },
                Dependency {
                    id: "specs".to_string(),
                    done: false,
                    path: "specs/**/*.md".to_string(),
                    description: "What".to_string(),
                },
            ],
            unlocks: vec!["tasks".to_string()],
        }
    }

    #[test]
    fn apply_human_prints_the_progress_the_tasks_and_the_instruction() {
        assert_eq!(
            apply_human(&apply(None), &PLAIN).stdout,
            "Change: alpha\nSchema: spec-driven\nState: ready\nProgress: 1/2 complete\n\nTasks:\n  ✓ 1.1 Deliver behavior 1; verify with the unit test\n  ○ 1.2 Deliver behavior 2; verify with the unit test\n\nInstruction:\nRead context files, work through pending tasks.\n\n"
        );
    }

    #[test]
    fn apply_human_lists_missing_artifacts_in_place_of_the_tasks() {
        let mut blocked = apply(Some(vec!["tasks".to_string()]));
        blocked.instruction = None;
        assert_eq!(
            apply_human(&blocked, &COLOR).stdout,
            "\x1b[1mChange\x1b[0m: alpha\n\x1b[1mSchema\x1b[0m: spec-driven\n\x1b[1mState\x1b[0m: ready\n\x1b[1mProgress\x1b[0m: 1/2 complete\n\n\x1b[31mMissing artifacts:\x1b[0m\n  - tasks\n\n"
        );
    }

    #[test]
    fn artifact_human_prints_each_section_that_has_content() {
        assert_eq!(
            artifact_human(&artifact(), &PLAIN).stdout,
            "Artifact: design\nOutput: design.md\nDescription: Technical design document\n\nInstruction:\nCreate the design document.\n\n\nDependencies:\n  ✓ proposal (proposal.md)\n  ○ specs (specs/**/*.md)\n\nUnlocks:\n  - tasks\n\nTemplate:\n## Context\n\n"
        );
        let mut bare = artifact();
        bare.instruction = None;
        bare.dependencies.clear();
        bare.unlocks.clear();
        bare.template.clear();
        assert_eq!(
            artifact_human(&bare, &COLOR).stdout,
            "\x1b[1mArtifact\x1b[0m: design\n\x1b[1mOutput\x1b[0m: design.md\n\x1b[1mDescription\x1b[0m: Technical design document\n"
        );
    }

    #[test]
    fn instructions_human_follows_the_outcome_kind() {
        let as_apply = InstructionsOutcome::Apply(apply(None));
        assert_eq!(instructions_human(&as_apply, &PLAIN), apply_human(&apply(None), &PLAIN));
        assert!(instructions_human(&as_apply, &PLAIN).stdout.starts_with("Change: alpha\n"));
        let as_artifact = InstructionsOutcome::Artifact(artifact());
        assert_eq!(instructions_human(&as_artifact, &PLAIN), artifact_human(&artifact(), &PLAIN));
        assert!(instructions_human(&as_artifact, &PLAIN).stdout.starts_with("Artifact: design\n"));
    }

    #[test]
    fn instructions_json_is_the_payload_and_its_text_keeps_the_field_order() {
        let value = instructions(&InstructionsOutcome::Apply(apply(None)), RenderOptions::default()).json.unwrap();
        assert_eq!(value["changeName"], "alpha");
        assert_eq!(
            value["progress"],
            json!({ "total": 2, "complete": 1, "remaining": 1, "codeTotal": 2, "codeComplete": 1, "codeRemaining": 1 })
        );
        assert_eq!(value["tasks"][1], json!({ "id": "2", "description": "1.2 Deliver behavior 2; verify with the unit test", "done": false, "manual": false }));
        assert!(value.get("missingArtifacts").is_none() && value.get("preflight").is_none(), "{value}");

        let value = instructions(&InstructionsOutcome::Artifact(artifact()), RenderOptions::default()).json.unwrap();
        assert_eq!(value["artifactId"], "design");
        assert_eq!(value["dependencies"][0], json!({ "id": "proposal", "done": true, "path": "proposal.md", "description": "Why" }));
        assert!(value.get("context").is_none() && value.get("rules").is_none(), "{value}");

        let text = instructions(&InstructionsOutcome::Artifact(artifact()), RenderOptions { json: true, color: false }).stdout;
        assert!(text.starts_with("{\n  \"changeName\": \"alpha\",\n  \"artifactId\": \"design\",\n  \"schemaName\": \"spec-driven\",\n"), "{text}");
        assert!(text.ends_with("  \"unlocks\": [\n    \"tasks\"\n  ]\n}\n"), "{text}");
    }
}
