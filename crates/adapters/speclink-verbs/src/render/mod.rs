//! The one definition of each spec verb's output.
//!
//! Every function here turns a result into text or a JSON value and returns
//! it. None of them writes to stdout or stderr, reads the terminal, or reads
//! an environment variable: whether to color and whether the caller wants the
//! JSON form both come in as parameters.
//!
//! One file per verb family, the same split as `families/`. A verb has up to
//! three functions: `<verb>_json` (its structured value, when it has one),
//! `<verb>_human` (its text) and `<verb>`, which puts the two together under
//! the caller's [`RenderOptions`].

use serde::Serialize;
use serde_json::Value;
use speclink_core::command::{CommandError, CommandOutcome};

use crate::families::discuss::DiscussCommands;
use crate::families::new::NewCommands;
use crate::families::plan::ChangeCommands;
use crate::{ParsedVerb, SpecVerb};

// The three writers every human-output function uses. They append to a
// `Text`; nothing reaches the process's own streams.

/// Append one line to `text.stdout`.
macro_rules! outln {
    ($text:expr) => {
        $text.stdout.push('\n')
    };
    ($text:expr, $($arg:tt)*) => {{
        $text.stdout.push_str(&format!($($arg)*));
        $text.stdout.push('\n');
    }};
}

/// Append to `text.stdout` with no newline added.
macro_rules! out {
    ($text:expr, $($arg:tt)*) => {
        $text.stdout.push_str(&format!($($arg)*))
    };
}

/// Append one line to `text.stderr`.
macro_rules! errln {
    ($text:expr, $($arg:tt)*) => {{
        $text.stderr.push_str(&format!($($arg)*));
        $text.stderr.push('\n');
    }};
}

pub mod checks;
pub mod discuss;
pub mod documents;
pub mod evidence;
pub mod fields;
pub mod instructions;
pub mod lifecycle;
pub mod new;
pub mod paint;
pub mod plan;
pub mod progress;
pub mod query;
pub mod station;
pub mod trace;
pub mod workflow_config;

pub use paint::Paint;

/// What the caller wants from an output function.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderOptions {
    /// The caller asked for the JSON form.
    pub json: bool,
    /// Color the human form.
    pub color: bool,
}

/// A verb's human output: the two streams' text, each complete with its
/// final newline.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text {
    pub stdout: String,
    pub stderr: String,
}

/// One verb's whole output.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rendered {
    /// The text for stdout. With `RenderOptions::json` set, this is the JSON
    /// form's text.
    pub stdout: String,
    /// The text for stderr; empty for most verbs.
    pub stderr: String,
    /// The verb's structured value, whether or not the caller asked for the
    /// JSON form; `None` for a verb that has none.
    pub json: Option<Value>,
}

impl Rendered {
    /// Put a verb's two forms together: the JSON text when the caller asked
    /// for it and the verb has one, else the human text.
    pub(crate) fn of(
        options: RenderOptions,
        json: Option<JsonForm>,
        human: impl FnOnce(&Paint) -> Text,
    ) -> Rendered {
        match json {
            Some(form) if options.json => {
                Rendered { stdout: form.text, stderr: String::new(), json: Some(form.value) }
            }
            form => {
                let text = human(&Paint::new(options.color));
                Rendered { stdout: text.stdout, stderr: text.stderr, json: form.map(|f| f.value) }
            }
        }
    }
}

/// A verb's JSON form: the value, and the exact text the CLI prints for it.
/// The text is serialized from the source itself, not from the value — a
/// struct keeps its field order in the text, while a `Value` sorts its keys.
pub(crate) struct JsonForm {
    value: Value,
    text: String,
}

// The sources are the engine's result types (derived `Serialize`, string
// keys) and `Value`s built here: serialization has no failure case for them.
const SERIALIZES: &str = "a result type serializes to JSON";

impl JsonForm {
    /// Indented JSON and a newline — what most verbs print.
    pub(crate) fn pretty<T: Serialize + ?Sized>(source: &T) -> JsonForm {
        let mut text = serde_json::to_string_pretty(source).expect(SERIALIZES);
        text.push('\n');
        JsonForm { value: value_of(source), text }
    }

    /// One-line JSON and a newline — the frozen form of `new artifact` and
    /// the task flips.
    pub(crate) fn compact<T: Serialize + ?Sized>(source: &T) -> JsonForm {
        let mut text = serde_json::to_string(source).expect(SERIALIZES);
        text.push('\n');
        JsonForm { value: value_of(source), text }
    }
}

/// A result type as a JSON value.
pub(crate) fn value_of<T: Serialize + ?Sized>(source: &T) -> Value {
    serde_json::to_value(source).expect(SERIALIZES)
}

/// The notice a verb gives in place of its output when it was given no
/// change name and the project has no change at all. Both entry points use
/// this one text.
pub fn no_active_changes() -> Rendered {
    let mut t = Text::default();
    outln!(t, "{}", speclink_core::command::NO_ACTIVE_CHANGES);
    Rendered { stdout: t.stdout, stderr: t.stderr, json: None }
}

/// The single exit: one verb's result as its whole output.
///
/// `verb` gives the facts only argv holds (the `--on` names of
/// `change depends`, `--archived` of `discuss list`, …).
///
/// The error case is a result that both entry points report as a failure
/// with no output: a task already in the asked state.
///
/// This entry is for a caller with a local folder (the CLI, an engine on a
/// project folder): a `Path:` line names a real location. A host that embeds
/// the engine calls [`render_embedded`]. The two have the same signature, and
/// a caller that takes the wrong one gets or loses three path lines and
/// nothing else: no JSON form changes, and nothing the caller is allowed to
/// read changes.
///
/// Sharp-edges notes on `list --json --detail`, whose items this entry
/// serializes as the engine filled them:
/// - The flag's two checks (it needs `--json`; it does not go with `--specs`)
///   are in the verb definition every entry point parses with, so the CLI and
///   a host refuse the same argv with the same text, before any store call.
/// - Without the flag no detail field is set, and none is serialized: the
///   frozen output is the default, not something a caller must ask for.
/// - A change whose metadata does not read is listed with `metaError` and no
///   detail key. It never fails the list.
/// - A missing key means "no record here", and against an older server "the
///   server did not say". It is never a statement that a station did not run.
/// - `claimed.byId` is the identifier the host gave for the holder. Whoever
///   can list the scope sees it, as with `show --json`; the user documents say
///   so. The list adds no read the caller could not already make.
pub fn render(
    verb: &ParsedVerb,
    outcome: &CommandOutcome,
    options: RenderOptions,
) -> Result<Rendered, CommandError> {
    render_with(verb, outcome, options, PathLines::Printed)
}

/// [`render`] for a caller whose documents are in a store and not in a local
/// folder: a host that embeds the engine. The human output of `new change`,
/// `discuss new` and `discuss promote` leaves out the `Path:` line (and, for
/// `discuss promote`, the propose hint that goes with it), because the path
/// names no file the reader can open. Every other verb, and the JSON form of
/// every verb (its `path` key included), is the one of [`render`].
///
/// Sharp-edges notes:
/// - The difference from [`render`] is those three human outputs only. The
///   tests compare the two entries over every result type, in the human form,
///   the colored form and the JSON form.
/// - The JSON `path` stays, so a caller that reads it keeps working; it is the
///   document's key in the store. The Agent tool's description tells the model
///   that it is not a file, and names the verbs that read a document.
/// - The choice is a separate entry and not a field of [`RenderOptions`]: a
///   field would default to "print the path" in every literal that omits
///   thinking about it, and only one kind of caller needs the other answer.
pub fn render_embedded(
    verb: &ParsedVerb,
    outcome: &CommandOutcome,
    options: RenderOptions,
) -> Result<Rendered, CommandError> {
    render_with(verb, outcome, options, PathLines::Omitted)
}

/// Whether the three verbs whose human output names a path print that line:
/// `new change`, `discuss new` and `discuss promote`. It never changes a JSON
/// form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathLines {
    /// The caller has a local folder: the path is a real location.
    Printed,
    /// The documents are in a store: the path names no file the reader can
    /// open, so the line is left out.
    Omitted,
}

/// The one definition behind the two entries. The match on the result has no
/// wildcard arm: a new result type does not compile until it has an output
/// here.
fn render_with(
    verb: &ParsedVerb,
    outcome: &CommandOutcome,
    options: RenderOptions,
    paths: PathLines,
) -> Result<Rendered, CommandError> {
    use progress::TaskFlip;
    use speclink_core::station::{REVIEW, VERIFY};
    use station::StationAction;
    use CommandOutcome as O;

    Ok(match outcome {
        O::List(o) => query::list(o, options),
        O::Show(o) => query::show(o, matches!(verb.spec(), SpecVerb::Show(a) if a.detail), options),
        O::Status(o) => query::status(o, options),
        O::Instructions(o) => instructions::instructions(o, options),
        O::Validate(o) => checks::validate(&o.results, options),
        O::Analyze(o) => checks::analyze(o, options),
        O::Trace(o) => trace::trace(o, options),
        O::Plan(o) => plan::plan(o, options),
        O::ArtifactCat(read) => documents::document(&read.content, options),
        O::Language(content) => documents::document(content, options),
        O::LanguageReplace(o) => documents::replaced(o, options),
        O::EvidenceShow(o) => evidence::show(o, options),
        O::DiscussList(items) => {
            let archived =
                matches!(discuss_command(verb), Some(DiscussCommands::List { archived: true, .. }));
            discuss::list(items, archived, options)
        }
        O::DiscussShow(o) => discuss::show(o, options),
        O::DiscussSearch(hits) => {
            let terms: &[String] = match discuss_command(verb) {
                Some(DiscussCommands::Search { terms, .. }) => terms,
                _ => &[],
            };
            discuss::search(terms, hits, options)
        }
        O::NewChange(o) => {
            let from_discussion = match new_command(verb) {
                Some(NewCommands::Change(c)) => c.from_discussion.as_deref(),
                _ => None,
            };
            new::change_created(o, from_discussion, paths, options)
        }
        O::NewArtifact(o) => {
            // The output names the artifact with the word the caller gave.
            let artifact_type = match new_command(verb) {
                Some(NewCommands::Artifact(c)) => c.artifact_type.as_str(),
                _ => o.artifact.as_str(),
            };
            new::artifact_created(o, artifact_type, options)
        }
        O::TaskDone(o) => progress::task_flipped(TaskFlip::Done, o, options)?,
        O::TaskUndone(o) => progress::task_flipped(TaskFlip::Undone, o, options)?,
        // No verb text reaches a task move, and it has no output.
        O::TaskMove(_) => Rendered::default(),
        O::Claim(o) => lifecycle::claim(&o.name, o.claimed_by.as_deref(), options),
        O::ClaimRelease(o) => lifecycle::claim_release(o, options),
        // `in-progress add` prints nothing.
        O::InProgressAdd(_) => Rendered::default(),
        O::InProgressRemove(o) => progress::in_progress_remove(&o.name, o.removed, options),
        O::ChangeDepends(o) => {
            let (on, remove): (&[String], bool) = match change_command(verb) {
                Some(ChangeCommands::Depends { on, remove, .. }) => (on, *remove),
                _ => (&[], false),
            };
            plan::depends_declared(o, on, remove, options)
        }
        O::ChangeSchema(o) => plan::schema(o, options),
        O::ChangeRank(o) => plan::rank(o, options),
        O::ChangeLanded(o) => plan::landed(o, options),
        O::ChangeFieldsEdit(o) => fields::edited(fields::FieldKind::Change, o, options),
        O::Archive(o) => lifecycle::archive(o, options),
        O::Discard(o) => lifecycle::discard(&o.change_name, &o.unlinked_discussions, options),
        O::DiscussNew(info) => discuss::new(info, paths, options),
        O::DiscussContext(o) => discuss::context(&o.slug, options),
        O::DiscussAddRound(o) => discuss::round_recorded(o, options),
        // The conclusion landed even when the closing step failed, so this is
        // an output: the value carries the reason. An error would read as
        // "nothing was written", and a retry would conclude a second time.
        O::DiscussConclude(o) => discuss::conclude(
            &discuss::Concluded {
                closing_error: o.closing_error.as_deref(),
                ..discuss::Concluded::from(o)
            },
            options,
        ),
        O::DiscussPromote(o) => discuss::promoted(o, paths, options),
        O::DiscussLink(o) => discuss::bind(discuss::DiscussBind::Link, &o.slug, &o.change, options),
        O::DiscussSeal(o) => discuss::bind(discuss::DiscussBind::Seal, &o.slug, &o.change, options),
        O::DiscussArchive(o) => discuss::archived(o, options),
        O::DiscussRestore(o) => discuss::restored(o, options),
        O::DiscussDiscard(o) => discuss::discard(&o.slug, options),
        O::DiscussFieldsEdit(o) => fields::edited(fields::FieldKind::Discussion, o, options),
        O::ReviewAddRound(o) => {
            station::action(&REVIEW, StationAction::AddRound(o.round), &o.change, options)
        }
        O::ReviewShow(o) => station::show(&REVIEW, &o.change, &o.ticket, o.content.as_deref(), options),
        O::ReviewStamp(o) => station::action(&REVIEW, StationAction::Stamp, &o.change, options),
        O::ReviewDiscard(o) => station::action(&REVIEW, StationAction::Discard, &o.change, options),
        O::VerifyAddRound(o) => {
            station::action(&VERIFY, StationAction::AddRound(o.round), &o.change, options)
        }
        O::VerifyShow(o) => station::show(&VERIFY, &o.change, &o.ticket, o.content.as_deref(), options),
        O::VerifyStamp(o) => station::action(&VERIFY, StationAction::Stamp, &o.change, options),
        O::VerifyDiscard(o) => station::action(&VERIFY, StationAction::Discard, &o.change, options),
        O::WorkflowConfigShow(o) => workflow_config::show(o, options),
        O::WorkflowConfigEdit(o) => match verb.spec() {
            SpecVerb::WorkflowConfig(a) => workflow_config::written(&a.command, o, options),
            // Only a workflow-config verb converts to this command.
            _ => Rendered::default(),
        },
        // No verb text reaches a whole-document replace, and it has no output.
        O::WorkflowConfigReplace(_) => Rendered::default(),
    })
}

// The subcommand a verb parsed to, for the three families whose output needs
// a fact from argv. A verb of another family has none.

fn discuss_command(verb: &ParsedVerb) -> Option<&DiscussCommands> {
    match verb.spec() {
        SpecVerb::Discuss(a) => Some(&a.command),
        _ => None,
    }
}

fn new_command(verb: &ParsedVerb) -> Option<&NewCommands> {
    match verb.spec() {
        SpecVerb::New(a) => Some(&a.command),
        _ => None,
    }
}

fn change_command(verb: &ParsedVerb) -> Option<&ChangeCommands> {
    match verb.spec() {
        SpecVerb::Change(a) => Some(&a.command),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::command as cmd;
    use speclink_core::command::ErrorCode;
    use speclink_core::station::{Round, Ticket};

    const HUMAN: RenderOptions = RenderOptions { json: false, color: false };
    const JSON: RenderOptions = RenderOptions { json: true, color: false };

    /// Each result type's place in the table below. The match has no wildcard
    /// arm, so a new result type must get a place — and then a row.
    fn place(outcome: &CommandOutcome) -> usize {
        match outcome {
            CommandOutcome::List(_) => 0,
            CommandOutcome::Show(_) => 1,
            CommandOutcome::Status(_) => 2,
            CommandOutcome::Instructions(_) => 3,
            CommandOutcome::Validate(_) => 4,
            CommandOutcome::Analyze(_) => 5,
            CommandOutcome::Trace(_) => 6,
            CommandOutcome::Plan(_) => 7,
            CommandOutcome::ArtifactCat(_) => 8,
            CommandOutcome::Language(_) => 9,
            CommandOutcome::DiscussList(_) => 10,
            CommandOutcome::DiscussShow(_) => 11,
            CommandOutcome::DiscussSearch(_) => 12,
            CommandOutcome::NewChange(_) => 13,
            CommandOutcome::NewArtifact(_) => 14,
            CommandOutcome::TaskDone(_) => 15,
            CommandOutcome::TaskUndone(_) => 16,
            CommandOutcome::TaskMove(_) => 17,
            CommandOutcome::Claim(_) => 18,
            CommandOutcome::InProgressAdd(_) => 19,
            CommandOutcome::InProgressRemove(_) => 20,
            CommandOutcome::ChangeDepends(_) => 21,
            CommandOutcome::ChangeSchema(_) => 22,
            CommandOutcome::Archive(_) => 23,
            CommandOutcome::Discard(_) => 24,
            CommandOutcome::DiscussNew(_) => 25,
            CommandOutcome::DiscussContext(_) => 26,
            CommandOutcome::DiscussAddRound(_) => 27,
            CommandOutcome::DiscussConclude(_) => 28,
            CommandOutcome::DiscussPromote(_) => 29,
            CommandOutcome::DiscussLink(_) => 30,
            CommandOutcome::DiscussSeal(_) => 31,
            CommandOutcome::DiscussArchive(_) => 32,
            CommandOutcome::DiscussDiscard(_) => 33,
            CommandOutcome::ReviewAddRound(_) => 34,
            CommandOutcome::ReviewShow(_) => 35,
            CommandOutcome::ReviewStamp(_) => 36,
            CommandOutcome::ReviewDiscard(_) => 37,
            CommandOutcome::VerifyAddRound(_) => 38,
            CommandOutcome::VerifyShow(_) => 39,
            CommandOutcome::VerifyStamp(_) => 40,
            CommandOutcome::VerifyDiscard(_) => 41,
            CommandOutcome::WorkflowConfigShow(_) => 42,
            CommandOutcome::WorkflowConfigEdit(_) => 43,
            CommandOutcome::WorkflowConfigReplace(_) => 44,
            CommandOutcome::ChangeRank(_) => 45,
            CommandOutcome::EvidenceShow(_) => 46,
            CommandOutcome::ChangeLanded(_) => 47,
            CommandOutcome::ChangeFieldsEdit(_) => 48,
            CommandOutcome::DiscussFieldsEdit(_) => 49,
            CommandOutcome::ClaimRelease(_) => 50,
            CommandOutcome::LanguageReplace(_) => 51,
            CommandOutcome::DiscussRestore(_) => 52,
        }
    }
    const RESULT_TYPES: usize = 53;

    fn s(text: &str) -> String {
        text.to_string()
    }

    fn info() -> speclink_core::discuss::DiscussionInfo {
        speclink_core::discuss::DiscussionInfo {
            slug: s("t"),
            topic: s("T"),
            status: s("open"),
            rounds: 0,
            created: s("2026-01-01"),
            created_by: None,
            kind: None,
            path: s("/p/t.md"),
            archived: false,
            fields: Default::default(),
            concluded: false,
            head: Default::default(),
        }
    }

    fn flipped(already: bool) -> cmd::TaskFlipOutcome {
        cmd::TaskFlipOutcome {
            change: s("c"),
            task_id: 1,
            task_id_arg: s("tsk_1"),
            description: s("d"),
            already,
            stable_id: None,
            touched_files: Vec::new(),
        }
    }

    fn concluded(closing_error: Option<&str>) -> cmd::DiscussConcludeOutcome {
        cmd::DiscussConcludeOutcome {
            slug: s("t"),
            restale_flagged: Vec::new(),
            auto_archived: false,
            closing_error: closing_error.map(s),
            held: false,
        }
    }

    fn config_written() -> cmd::WorkflowConfigWriteOutcome {
        use speclink_core::config::{ChangedKey, PolicyKey, WorkflowConfig};
        cmd::WorkflowConfigWriteOutcome {
            file: s("speclink/config.yaml"),
            before: None,
            after: s("tdd: true\n"),
            config: WorkflowConfig { tdd: Some(true), ..Default::default() },
            changed_keys: vec![ChangedKey::Policy(PolicyKey::Tdd)],
            written: true,
        }
    }

    fn shown_ticket() -> cmd::ReviewShowOutcome {
        cmd::ReviewShowOutcome {
            change: s("c"),
            ticket: Ticket {
                rounds: vec![Round {
                    index: 1,
                    phase: None,
                    patch_hash: None,
                    rules: None,
                    scope: vec![s("src/a.rs")],
                    findings: Vec::new(),
                }],
            },
            content: Some(s("# Ticket — c\n")),
        }
    }

    /// One row per result type: the argv, the result, whether the verb has a
    /// structured value, and its human stdout.
    fn rows() -> Vec<(&'static [&'static str], CommandOutcome, bool, &'static str)> {
        use speclink_core::analyzer::AnalyzeReport;
        use speclink_core::archive::ArchiveOutcome;
        use speclink_core::discard::DiscardOutcome;
        use speclink_core::instructions::ArtifactInstructions;
        use speclink_core::status::StatusReport;
        use speclink_core::trace::TraceReport;
        use speclink_core::validate::ValidationResult;
        use CommandOutcome as O;
        let subject = || cmd::ReviewSubjectOutcome { change: s("c") };
        let stamped = || cmd::StampOutcome { change: s("c"), commit: None };
        let round = || cmd::ReviewRoundOutcome { change: s("c"), round: 1 };
        let bound = || cmd::DiscussBindOutcome { slug: s("t"), change: s("c") };
        vec![
            (&["list"], O::List(cmd::ListOutcome { changes: Some(Vec::new()), specs: None }), true, "No active changes.\n"),
            (
                &["show", "auth"],
                O::Show(cmd::ShowOutcome::Spec { name: s("auth"), content: s("# auth\n") }),
                true,
                "Spec: auth\n\n--- spec.md ---\n# auth\n\n",
            ),
            (
                &["status"],
                O::Status(StatusReport {
                    change_name: s("c"),
                    schema_name: s("spec-driven"),
                    is_complete: false,
                    apply_requires: Vec::new(),
                    artifacts: Vec::new(),
                }),
                true,
                "Change: c\nSchema: spec-driven\n\n\n",
            ),
            (
                &["instructions", "design"],
                O::Instructions(cmd::InstructionsOutcome::Artifact(ArtifactInstructions {
                    change_name: s("c"),
                    artifact_id: s("design"),
                    schema_name: s("spec-driven"),
                    change_dir: s("/p/c"),
                    output_path: s("design.md"),
                    description: s("The design"),
                    instruction: None,
                    context: None,
                    rules: None,
                    locale: s("English"),
                    template: String::new(),
                    dependencies: Vec::new(),
                    unlocks: Vec::new(),
                })),
                true,
                "Artifact: design\nOutput: design.md\nDescription: The design\n",
            ),
            (
                &["validate"],
                O::Validate(cmd::ValidateOutcome {
                    results: vec![ValidationResult {
                        change: s("c"),
                        errors: Vec::new(),
                        valid: true,
                        warnings: Vec::new(),
                    }],
                }),
                true,
                "✓ c — valid\n",
            ),
            (
                &["analyze"],
                O::Analyze(AnalyzeReport {
                    change_id: s("c"),
                    dimensions: Vec::new(),
                    findings: Vec::new(),
                    artifacts_analyzed: Vec::new(),
                    artifacts_missing: Vec::new(),
                }),
                true,
                "Change: c\n\n\n  ✓ No issues found\n",
            ),
            (
                &["trace", "auth"],
                O::Trace(TraceReport {
                    capability: s("auth"),
                    requirements: Vec::new(),
                    changes: Vec::new(),
                    discussions: Vec::new(),
                }),
                true,
                "Capability: auth\n\nChanges (oldest first):\n  (none)\n\nDiscussions:\n  (none)\n\nRequirements:\n  (none)\n",
            ),
            (
                &["plan"],
                O::Plan(cmd::PlanReport {
                    waves: Vec::new(),
                    changes: Vec::new(),
                    next: None,
                    skipped: Vec::new(),
                    unpromoted_cuts: Vec::new(),
                }),
                true,
                "next: none\n",
            ),
            (
                &["artifact", "cat", "tasks"],
                O::ArtifactCat(cmd::ArtifactCatOutcome { content: s("- [ ] 1.1 x\n"), archived_as: None }),
                false,
                "- [ ] 1.1 x\n",
            ),
            (&["language", "show"], O::Language(s("# Language\n")), false, "# Language\n"),
            (
                &["evidence", "show", "c"],
                O::EvidenceShow(cmd::EvidenceShowOutcome { change: s("c"), archived_as: None, entries: Vec::new() }),
                true,
                "No evidence recorded for 'c'.\n",
            ),
            (&["discuss", "list", "--archived"], O::DiscussList(Vec::new()), true, "No archived discussions found.\n"),
            (
                &["discuss", "show", "t"],
                O::DiscussShow(cmd::DiscussShowOutcome { info: None, content: s("the record\n") }),
                true,
                "the record\n",
            ),
            (&["discuss", "search", "a", "b"], O::DiscussSearch(Vec::new()), true, "No discussions match \"a b\".\n"),
            (
                &["new", "change", "c", "--from-discussion", "t"],
                O::NewChange(cmd::NewChangeOutcome { name: s("c"), dir: "/p/c".into(), schema: s("spec-driven") }),
                false,
                "✓ Created change: c\n  Path: /p/c\n  Schema: spec-driven\n  From discussion: t\n",
            ),
            (
                &["new", "artifact", "spec", "auth"],
                O::NewArtifact(cmd::NewArtifactOutcome {
                    artifact: s("specs"),
                    change: s("c"),
                    path: "/p/c/specs/auth/spec.md".into(),
                    had_content: false,
                }),
                true,
                "✓ Created spec: /p/c/specs/auth/spec.md\n",
            ),
            (&["task", "done", "tsk_1"], O::TaskDone(flipped(false)), true, "✓ Task tsk_1 marked as done: d\n"),
            (&["task", "undone", "tsk_1"], O::TaskUndone(flipped(false)), true, "✓ Task tsk_1 marked as not done: d\n"),
            // No verb text reaches a task move.
            (&["list"], O::TaskMove(cmd::TaskMoveOutcome { change: s("c"), description: s("d") }), false, ""),
            (
                &["claim", "c"],
                O::Claim(cmd::ClaimOutcome { name: s("c"), claimed_by: Some(s("Alice")), claimed: true }),
                true,
                "✓ Claimed change: c\n",
            ),
            (
                &["claim", "c", "--release"],
                O::ClaimRelease(cmd::ClaimReleaseOutcome { name: s("c"), released: true, previous_holder: Some(s("Alice")) }),
                true,
                "✓ Released claim on change: c (was claimed by Alice)\n",
            ),
            (
                &["in-progress", "add", "c"],
                O::InProgressAdd(cmd::InProgressOutcome { name: s("c"), stamped: true }),
                false,
                "",
            ),
            (
                &["in-progress", "remove", "c"],
                O::InProgressRemove(cmd::InProgressRemoveOutcome { name: s("c"), removed: true }),
                false,
                "✓ Removed the in-progress marker from 'c' — back to proposed\n",
            ),
            (
                &["change", "depends", "c", "--on", "b", "--remove"],
                O::ChangeDepends(cmd::DependsOutcome { change: s("c"), depends_on: vec![s("a")], changed: true }),
                true,
                "✓ c no longer depends on: b\n",
            ),
            (
                &["change", "schema", "c", "spec-driven"],
                O::ChangeSchema(cmd::SchemaOutcome {
                    change: s("c"),
                    schema: s("spec-driven"),
                    previous: s("spec-driven"),
                    changed: false,
                    verify_stamp_cleared: false,
                }),
                true,
                "✓ c schema: spec-driven (unchanged)\n",
            ),
            (
                &["archive", "c"],
                O::Archive(ArchiveOutcome {
                    change_name: s("c"),
                    dated_name: s("2026-01-01-c"),
                    caps: Vec::new(),
                    snapshot_created: false,
                    skipped_specs: false,
                    archived_discussions: Vec::new(),
                    evidence_recorded: true,
                }),
                false,
                "✓ Archived: c → 2026-01-01-c\n",
            ),
            (
                &["discard", "c"],
                O::Discard(DiscardOutcome { change_name: s("c"), unlinked_discussions: Vec::new() }),
                true,
                "✓ Discarded change: c\n",
            ),
            (
                &["discuss", "new", "T"],
                O::DiscussNew(info()),
                true,
                "✓ Created discussion: t\n  Topic: T\n  Path: /p/t.md\n",
            ),
            (
                &["discuss", "context", "t"],
                O::DiscussContext(cmd::DiscussSubjectOutcome { slug: s("t") }),
                true,
                "✓ Set context for discussion 't'\n",
            ),
            (
                &["discuss", "add-round", "t"],
                O::DiscussAddRound(cmd::DiscussRoundOutcome { slug: s("t"), mode: s("interview"), round: 1 }),
                true,
                "✓ Recorded round 1 (interview) to discussion 't'\n",
            ),
            (&["discuss", "conclude", "t"], O::DiscussConclude(concluded(None)), true, "✓ Concluded discussion 't'\n"),
            (
                &["discuss", "promote", "t"],
                O::DiscussPromote(cmd::DiscussPromoteOutcome { slug: s("t"), change: s("c"), path: "/p/c".into() }),
                true,
                "✓ Promoted discussion 't' → change 'c'\n  Path: /p/c\n  Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts\n",
            ),
            (&["discuss", "link", "t", "c"], O::DiscussLink(bound()), true, "✓ Linked discussion 't' → change 'c'\n"),
            (
                &["discuss", "seal", "t", "c"],
                O::DiscussSeal(bound()),
                true,
                "✓ Sealed discussion 't' → change 'c' (marked promoted)\n",
            ),
            (
                &["discuss", "archive", "t"],
                O::DiscussArchive(cmd::DiscussArchiveOutcome { slug: s("t"), archived_file: s("2026-01-01-t.md") }),
                true,
                "✓ Archived discussion: t → discussions/archive/2026-01-01-t.md\n",
            ),
            (
                &["discuss", "discard", "t"],
                O::DiscussDiscard(cmd::DiscussSubjectOutcome { slug: s("t") }),
                true,
                "✓ Discarded discussion: t\n",
            ),
            (&["review", "add-round", "c"], O::ReviewAddRound(round()), true, "✓ Recorded review Round 1 for change 'c'\n"),
            (&["review", "show", "c"], O::ReviewShow(shown_ticket()), true, "# Ticket — c\n"),
            (&["review", "stamp", "c"], O::ReviewStamp(stamped()), true, "✓ Stamped review for change 'c'\n"),
            (&["review", "discard", "c"], O::ReviewDiscard(subject()), false, "✓ Discarded review ticket for change 'c'\n"),
            (
                &["verify", "add-round", "c"],
                O::VerifyAddRound(round()),
                true,
                "✓ Recorded verification Round 1 for change 'c'\n",
            ),
            (&["verify", "show", "c"], O::VerifyShow(shown_ticket()), true, "# Ticket — c\n"),
            (&["verify", "stamp", "c"], O::VerifyStamp(stamped()), true, "✓ Stamped verification for change 'c'\n"),
            (
                &["verify", "discard", "c"],
                O::VerifyDiscard(subject()),
                false,
                "✓ Discarded verification ticket for change 'c'\n",
            ),
            (
                &["workflow-config", "show"],
                O::WorkflowConfigShow(cmd::WorkflowConfigShowOutcome {
                    file: s("speclink/config.yaml"),
                    config: Default::default(),
                    exists: false,
                }),
                true,
                "Workflow config: speclink/config.yaml\n\n  schema       unset (spec-driven)\n  locale       unset (English)\n  spec_locale  unset (specs in English)\n  tdd          unset (off)\n  audit        unset (off)\n  worktree     unset (off)\n  context      none\n  rules        none\n  review       none\n",
            ),
            (
                &["workflow-config", "set", "tdd", "true"],
                O::WorkflowConfigEdit(config_written()),
                false,
                "✓ tdd = true\n",
            ),
            // No verb text reaches a whole-document replace.
            (&["list"], O::WorkflowConfigReplace(config_written()), false, ""),
            (
                &["change", "rank", "c", "--before", "a"],
                O::ChangeRank(cmd::ChangeRankOutcome {
                    change: s("c"),
                    anchor: s("a"),
                    position: speclink_core::plan::Side::Before,
                }),
                true,
                "✓ c ranked before a\n",
            ),
            (
                &["change", "landed", "c", "--commit", "0123456789abcdef"],
                O::ChangeLanded(cmd::ChangeLandedOutcome { change: s("c"), commit: s("0123456789abcdef"), branch: None }),
                true,
                "✓ c landed at 0123456789ab\n",
            ),
            (
                &["change", "field", "set", "c", "ticket=PROJ-1", "source=portal"],
                O::ChangeFieldsEdit(cmd::FieldsOutcome {
                    subject: s("c"),
                    fields: [(s("source"), s("portal")), (s("ticket"), s("PROJ-1"))].into(),
                    changed: vec![s("source"), s("ticket")],
                }),
                true,
                "✓ c: set source, ticket\n",
            ),
            (
                &["discuss", "field", "unset", "t", "owner", "ghost"],
                O::DiscussFieldsEdit(cmd::FieldsOutcome {
                    subject: s("t"),
                    fields: Default::default(),
                    changed: vec![s("owner")],
                }),
                true,
                "✓ t: unset owner\n",
            ),
            (
                &["language", "replace", "--stdin"],
                O::LanguageReplace(cmd::LanguageReplaceOutcome { created: false }),
                true,
                "✓ Replaced the LANGUAGE document\n",
            ),
            (
                &["discuss", "restore", "t"],
                O::DiscussRestore(cmd::DiscussRestoreOutcome { slug: s("t"), restored_from: s("2026-01-01-t.md") }),
                true,
                "✓ Restored discussion: t\n  From: 2026-01-01-t.md\n",
            ),
        ]
    }

    fn parsed(argv: &[&str]) -> ParsedVerb {
        crate::parse(argv.iter().copied()).unwrap_or_else(|e| panic!("{argv:?}: {}", e.message))
    }

    #[test]
    fn the_table_has_one_row_for_each_result_type() {
        let mut places: Vec<usize> = rows().iter().map(|(_, outcome, _, _)| place(outcome)).collect();
        places.sort_unstable();
        assert_eq!(places, (0..RESULT_TYPES).collect::<Vec<_>>());
    }

    #[test]
    fn every_result_type_has_its_human_output() {
        for (argv, outcome, has_json, stdout) in rows() {
            let out = render(&parsed(argv), &outcome, HUMAN).unwrap_or_else(|e| panic!("{argv:?}: {e}"));
            assert_eq!(out.stdout, stdout, "{argv:?}");
            assert_eq!(out.stderr, "", "{argv:?}");
            assert_eq!(out.json.is_some(), has_json, "{argv:?}");
        }
    }

    // The JSON form is indented and ends with one newline; `new artifact` and
    // the task flips print theirs on one line (frozen). A verb with no JSON
    // form prints its human text whatever the caller asked for.
    #[test]
    fn the_json_form_is_indented_json_with_a_final_newline() {
        const ONE_LINE: [usize; 3] = [14, 15, 16];
        for (argv, outcome, has_json, human) in rows() {
            let out = render(&parsed(argv), &outcome, JSON).unwrap_or_else(|e| panic!("{argv:?}: {e}"));
            let Some(value) = &out.json else {
                assert!(!has_json, "{argv:?}");
                assert_eq!(out.stdout, human, "{argv:?}");
                continue;
            };
            assert_eq!(&serde_json::from_str::<Value>(&out.stdout).expect("stdout is JSON"), value, "{argv:?}");
            let body = out.stdout.strip_suffix('\n').expect("one final newline");
            assert!(!body.ends_with('\n'), "{argv:?}");
            let indented = serde_json::to_string_pretty(&serde_json::from_str::<Value>(body).unwrap()).unwrap();
            if ONE_LINE.contains(&place(&outcome)) {
                assert!(!body.contains('\n'), "{argv:?}: {body}");
            } else {
                assert_eq!(body.lines().count(), indented.lines().count(), "{argv:?}: {body}");
            }
            // The value is the same whether or not the JSON form was asked for.
            let human_out = render(&parsed(argv), &outcome, HUMAN).unwrap();
            assert_eq!(human_out.json.as_ref(), Some(value), "{argv:?}");
        }
    }

    // --- the embedded entry (verb-output-for-embedded D4) ---

    /// The three result types whose human output has a path line: `new
    /// change`, `discuss new` and `discuss promote`.
    fn has_path_line(outcome: &CommandOutcome) -> bool {
        matches!(
            outcome,
            CommandOutcome::NewChange(_) | CommandOutcome::DiscussNew(_) | CommandOutcome::DiscussPromote(_)
        )
    }

    fn row(argv: &[&str]) -> (ParsedVerb, CommandOutcome) {
        let (_, outcome, _, _) =
            rows().into_iter().find(|(row, ..)| *row == argv).unwrap_or_else(|| panic!("no row {argv:?}"));
        (parsed(argv), outcome)
    }

    #[test]
    fn the_embedded_entry_prints_no_path_line_for_the_three_verbs() {
        // spec verb-contract「內嵌宿主的人眼輸出不印路徑行」Example「三個動詞在兩種執行
        // 方式的輸出」：其餘的行與 `render` 的逐位元相同。
        let embedded = |argv: &[&str]| {
            let (verb, outcome) = row(argv);
            render_embedded(&verb, &outcome, HUMAN).unwrap_or_else(|e| panic!("{argv:?}: {e}"))
        };
        assert_eq!(
            embedded(&["new", "change", "c", "--from-discussion", "t"]).stdout,
            "✓ Created change: c\n  Schema: spec-driven\n  From discussion: t\n"
        );
        assert_eq!(embedded(&["discuss", "new", "T"]).stdout, "✓ Created discussion: t\n  Topic: T\n");
        // The Path line and the propose hint go together.
        assert_eq!(embedded(&["discuss", "promote", "t"]).stdout, "✓ Promoted discussion 't' → change 'c'\n");

        // The rows of the spec's table, with the table's own values.
        let table: [(&[&str], CommandOutcome, &str); 3] = [
            (
                &["new", "change", "demo"],
                CommandOutcome::NewChange(cmd::NewChangeOutcome {
                    name: s("demo"),
                    dir: "speclink/changes/demo".into(),
                    schema: s("spec-driven"),
                }),
                "✓ Created change: demo\n  Schema: spec-driven\n",
            ),
            (
                &["discuss", "new", "Topic", "--slug", "topic"],
                CommandOutcome::DiscussNew(speclink_core::discuss::DiscussionInfo {
                    slug: s("topic"),
                    topic: s("Topic"),
                    path: s("speclink/discussions/topic.md"),
                    ..info()
                }),
                "✓ Created discussion: topic\n  Topic: Topic\n",
            ),
            (
                &["discuss", "promote", "topic"],
                CommandOutcome::DiscussPromote(cmd::DiscussPromoteOutcome {
                    slug: s("topic"),
                    change: s("topic"),
                    path: "speclink/changes/topic".into(),
                }),
                "✓ Promoted discussion 'topic' → change 'topic'\n",
            ),
        ];
        for (argv, outcome, stdout) in table {
            let verb = parsed(argv);
            assert_eq!(render_embedded(&verb, &outcome, HUMAN).unwrap().stdout, stdout, "{argv:?}");
            // The local column of the same row: the same lines plus the path.
            let local = render(&verb, &outcome, HUMAN).unwrap().stdout;
            assert!(local.starts_with(stdout.lines().next().unwrap()), "{argv:?}: {local}");
            assert!(local.contains("\n  Path: speclink/"), "{argv:?}: {local}");
        }
        for argv in [&["new", "change", "c", "--from-discussion", "t"][..], &["discuss", "new", "T"], &["discuss", "promote", "t"]] {
            let out = embedded(argv);
            assert!(!out.stdout.lines().any(|line| line.trim_start().starts_with("Path:")), "{argv:?}: {}", out.stdout);
            assert_eq!(out.stderr, "", "{argv:?}");
        }
    }

    #[test]
    fn the_embedded_entry_keeps_every_json_form_and_its_path_key() {
        for (argv, outcome, has_json, _) in rows() {
            let verb = parsed(argv);
            let embedded = render_embedded(&verb, &outcome, JSON).unwrap_or_else(|e| panic!("{argv:?}: {e}"));
            let plain = render(&verb, &outcome, JSON).unwrap();
            if has_json {
                assert_eq!(embedded, plain, "{argv:?}");
            }
            // The structured value is the same when the human form was asked for.
            assert_eq!(
                render_embedded(&verb, &outcome, HUMAN).unwrap().json,
                render(&verb, &outcome, HUMAN).unwrap().json,
                "{argv:?}"
            );
        }
        let json_of = |argv: &[&str]| {
            let (verb, outcome) = row(argv);
            render_embedded(&verb, &outcome, JSON).unwrap().json.expect("a JSON form")
        };
        assert_eq!(json_of(&["discuss", "new", "T"])["path"], "/p/t.md");
        assert_eq!(json_of(&["discuss", "promote", "t"])["path"], "/p/c");
    }

    #[test]
    fn the_embedded_entry_changes_no_other_verb() {
        let mut same = 0;
        for (argv, outcome, _, _) in rows() {
            if has_path_line(&outcome) {
                continue;
            }
            let verb = parsed(argv);
            for options in [HUMAN, JSON, RenderOptions { json: false, color: true }] {
                assert_eq!(
                    render_embedded(&verb, &outcome, options).unwrap(),
                    render(&verb, &outcome, options).unwrap(),
                    "{argv:?} {options:?}"
                );
            }
            same += 1;
        }
        assert_eq!(same, RESULT_TYPES - 3, "every result type but the three with a path line");
        // The two samples the task names, by their own argv.
        for argv in [&["list"][..], &["archive", "c"]] {
            let (verb, outcome) = row(argv);
            assert_eq!(render_embedded(&verb, &outcome, HUMAN).unwrap(), render(&verb, &outcome, HUMAN).unwrap());
        }
    }

    #[test]
    fn the_embedded_entry_reports_the_same_errors() {
        let (verb, outcome) = (parsed(&["task", "done", "1"]), CommandOutcome::TaskDone(flipped(true)));
        assert_eq!(
            render_embedded(&verb, &outcome, HUMAN).unwrap_err().message,
            render(&verb, &outcome, HUMAN).unwrap_err().message
        );
    }

    #[test]
    fn claim_and_its_release_have_a_json_form() {
        let claimed = CommandOutcome::Claim(cmd::ClaimOutcome { name: s("c"), claimed_by: Some(s("Alice")), claimed: false });
        let out = render(&parsed(&["claim", "c", "--json"]), &claimed, JSON).unwrap();
        assert_eq!(out.json, Some(json!({ "change": "c", "claimedBy": "Alice" })));
        assert_eq!(serde_json::from_str::<Value>(&out.stdout).unwrap(), json!({ "change": "c", "claimedBy": "Alice" }));

        let nothing = CommandOutcome::ClaimRelease(cmd::ClaimReleaseOutcome { name: s("c"), released: false, previous_holder: None });
        let out = render(&parsed(&["claim", "c", "--release", "--json"]), &nothing, JSON).unwrap();
        assert_eq!(out.json, Some(json!({ "change": "c", "released": false })));
        let out = render(&parsed(&["claim", "c", "--release"]), &nothing, HUMAN).unwrap();
        assert_eq!(out.stdout, "Change 'c' is not claimed — nothing to release\n");
    }

    #[test]
    fn a_field_edit_names_its_subject_by_kind_in_json() {
        let edit = |subject: &str| cmd::FieldsOutcome {
            subject: s(subject),
            fields: [(s("ticket"), s("PROJ-1"))].into(),
            changed: vec![s("ticket")],
        };
        let out = render(&parsed(&["change", "field", "set", "demo", "ticket=PROJ-1", "--json"]), &CommandOutcome::ChangeFieldsEdit(edit("demo")), JSON)
            .unwrap();
        assert_eq!(out.json, Some(json!({ "change": "demo", "fields": { "ticket": "PROJ-1" } })));
        let out = render(&parsed(&["discuss", "field", "set", "t", "ticket=PROJ-1"]), &CommandOutcome::DiscussFieldsEdit(edit("t")), JSON).unwrap();
        assert_eq!(out.json, Some(json!({ "discussion": "t", "fields": { "ticket": "PROJ-1" } })));
    }

    #[test]
    fn argv_gives_the_facts_the_result_does_not_carry() {
        let added = CommandOutcome::ChangeDepends(cmd::DependsOutcome {
            change: s("c"),
            depends_on: vec![s("a"), s("b")],
            changed: true,
        });
        let out = render(&parsed(&["change", "depends", "c", "--on", "b"]), &added, HUMAN).unwrap();
        assert_eq!(out.stdout, "✓ c depends on: a, b\n");
        assert_eq!(out.json, Some(json!({ "change": "c", "dependsOn": ["a", "b"] })));

        let live = render(&parsed(&["discuss", "list"]), &CommandOutcome::DiscussList(Vec::new()), HUMAN).unwrap();
        assert_eq!(live.stdout, "No discussions found.\n");

        let created = CommandOutcome::NewArtifact(cmd::NewArtifactOutcome {
            artifact: s("proposal"),
            change: s("c"),
            path: "/p/c/proposal.md".into(),
            had_content: true,
        });
        let out = render(&parsed(&["new", "artifact", "proposal", "--stdin"]), &created, JSON).unwrap();
        assert_eq!(
            out.stdout,
            "{\"artifact\":\"proposal\",\"change\":\"c\",\"path\":\"/p/c/proposal.md\",\"status\":\"created\",\"validated\":true,\"warnings\":[]}\n"
        );
    }

    #[test]
    fn a_task_already_in_the_asked_state_is_an_error() {
        for (argv, outcome, message) in [
            (&["task", "done", "tsk_1"], CommandOutcome::TaskDone(flipped(true)), "Task 1 is already done"),
            (&["task", "undone", "tsk_1"], CommandOutcome::TaskUndone(flipped(true)), "Task 1 is already not done"),
        ] {
            let err = render(&parsed(argv), &outcome, HUMAN).expect_err("an error, not an output");
            assert_eq!(err.message, message);
            assert_eq!(err.code, ErrorCode::Error);
        }
    }

    // The conclusion landed, so this is an output: the value says why the
    // record was not archived. An error would read as "nothing was written".
    #[test]
    fn a_conclusion_whose_closing_step_failed_is_an_output_that_says_so() {
        let outcome = CommandOutcome::DiscussConclude(concluded(Some("the file is locked")));
        let out = render(&parsed(&["discuss", "conclude", "t"]), &outcome, HUMAN)
            .expect("an output, not an error");
        assert_eq!(
            out.json,
            Some(json!({ "slug": "t", "status": "concluded", "closingError": "the file is locked" }))
        );
        assert_eq!(out.stdout, "✓ Concluded discussion 't'\n");
    }

    #[test]
    fn the_no_change_notice_is_one_line_with_no_json_form() {
        let notice = no_active_changes();
        assert_eq!(notice.stdout, "No active changes. Create one with: speclink new change <name>\n");
        assert_eq!(notice.stderr, "");
        assert_eq!(notice.json, None);
    }

    #[test]
    fn the_archive_note_goes_to_stderr() {
        let outcome = CommandOutcome::Archive(speclink_core::archive::ArchiveOutcome {
            change_name: s("c"),
            dated_name: s("2026-01-01-c"),
            caps: Vec::new(),
            snapshot_created: false,
            skipped_specs: false,
            archived_discussions: Vec::new(),
            evidence_recorded: false,
        });
        let out = render(&parsed(&["archive", "c"]), &outcome, HUMAN).unwrap();
        assert_eq!(out.stdout, "✓ Archived: c → 2026-01-01-c\n");
        assert!(out.stderr.starts_with("note: no task evidence recorded for change 'c'"), "{}", out.stderr);
    }
}
