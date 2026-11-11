//! Output of the creation verbs: `new change` and `new artifact`.

use serde_json::{json, Value};
use speclink_core::command::{NewArtifactOutcome, NewChangeOutcome};

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// The optional lines of `new change`, by name. `path` is absent in remote
/// mode (a store-side directory means nothing on the caller's machine);
/// `schema` prints only when the source gives one.
#[derive(Debug, Clone, Copy)]
pub struct NewChangeLines<'a> {
    pub path: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub from_discussion: Option<&'a str>,
}

/// `new change`: the success line, then the lines that were given.
pub fn new_change_human(name: &str, lines: NewChangeLines<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Created change: {name}", paint.green("✓"));
    if let Some(path) = lines.path {
        outln!(t, "  Path: {path}");
    }
    if let Some(schema) = lines.schema {
        outln!(t, "  Schema: {schema}");
    }
    if let Some(slug) = lines.from_discussion {
        outln!(t, "  From discussion: {slug}");
    }
    t
}

/// `new change` has no JSON form.
pub fn new_change(name: &str, lines: NewChangeLines<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |paint| new_change_human(name, lines, paint))
}

/// `new change`, from the engine's result. The discussion's name is not in
/// the result: it comes from the call.
pub fn change_created(
    o: &NewChangeOutcome,
    from_discussion: Option<&str>,
    options: RenderOptions,
) -> Rendered {
    new_change(
        &o.name,
        NewChangeLines { path: Some(&o.dir.to_string_lossy()), schema: Some(&o.schema), from_discussion },
        options,
    )
}

/// What `new artifact` reports. `artifact_type` is the word the caller gave
/// (`spec`), not the schema's artifact id.
#[derive(Debug, Clone, Copy)]
pub struct NewArtifact<'a> {
    pub artifact_type: &'a str,
    pub change: &'a str,
    pub path: &'a str,
    /// Content came with the call and passed the content checks.
    pub validated: bool,
}

/// `new artifact --json`.
pub fn new_artifact_json(a: &NewArtifact<'_>) -> Value {
    json!({
        "artifact": a.artifact_type,
        "change": a.change,
        "path": a.path,
        "status": "created",
        "validated": a.validated,
        "warnings": [],
    })
}

/// `new artifact`: the success line, and one more when content was checked.
pub fn new_artifact_human(a: &NewArtifact<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Created {}: {}", paint.green("✓"), a.artifact_type, a.path);
    if a.validated {
        outln!(t, "  Content validated ✓");
    }
    t
}

/// The JSON form is one line (frozen), not indented.
pub fn new_artifact(a: &NewArtifact<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::compact(&new_artifact_json(a))), |paint| {
        new_artifact_human(a, paint)
    })
}

/// `new artifact`, from the engine's result. `artifact_type` is the word the
/// caller gave.
pub fn artifact_created(
    o: &NewArtifactOutcome,
    artifact_type: &str,
    options: RenderOptions,
) -> Rendered {
    new_artifact(
        &NewArtifact {
            artifact_type,
            change: &o.change,
            path: &o.path.to_string_lossy(),
            validated: o.had_content,
        },
        options,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    #[test]
    fn new_change_human_prints_only_the_lines_that_were_given() {
        let all = NewChangeLines {
            path: Some("/p/speclink/changes/fresh"),
            schema: Some("spec-driven"),
            from_discussion: Some("topic-live"),
        };
        assert_eq!(
            new_change_human("fresh", all, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Created change: fresh\n  Path: /p/speclink/changes/fresh\n  Schema: spec-driven\n  From discussion: topic-live\n"
        );
        let none = NewChangeLines { path: None, schema: None, from_discussion: None };
        assert_eq!(new_change_human("fresh", none, &PLAIN).stdout, "✓ Created change: fresh\n");
        let remote = NewChangeLines { path: None, schema: Some("spec-driven"), from_discussion: None };
        assert_eq!(
            new_change_human("fresh", remote, &PLAIN).stdout,
            "✓ Created change: fresh\n  Schema: spec-driven\n"
        );
    }

    #[test]
    fn new_change_has_no_json_form() {
        let lines = NewChangeLines { path: None, schema: None, from_discussion: None };
        let out = new_change("fresh", lines, RenderOptions { json: true, color: false });
        assert_eq!(out.stdout, "✓ Created change: fresh\n");
        assert_eq!(out.json, None);
    }

    fn created(validated: bool) -> NewArtifact<'static> {
        NewArtifact {
            artifact_type: "proposal",
            change: "empty",
            path: "/p/speclink/changes/empty/proposal.md",
            validated,
        }
    }

    #[test]
    fn new_artifact_human_adds_a_line_when_content_was_checked() {
        assert_eq!(
            new_artifact_human(&created(true), &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Created proposal: /p/speclink/changes/empty/proposal.md\n  Content validated ✓\n"
        );
        assert_eq!(
            new_artifact_human(&created(false), &PLAIN).stdout,
            "✓ Created proposal: /p/speclink/changes/empty/proposal.md\n"
        );
    }

    #[test]
    fn new_artifact_json_is_printed_on_one_line() {
        assert_eq!(
            new_artifact_json(&created(true)),
            json!({
                "artifact": "proposal", "change": "empty",
                "path": "/p/speclink/changes/empty/proposal.md",
                "status": "created", "validated": true, "warnings": [],
            })
        );
        assert_eq!(
            new_artifact(&created(true), RenderOptions { json: true, color: true }).stdout,
            "{\"artifact\":\"proposal\",\"change\":\"empty\",\"path\":\"/p/speclink/changes/empty/proposal.md\",\"status\":\"created\",\"validated\":true,\"warnings\":[]}\n"
        );
    }
}
