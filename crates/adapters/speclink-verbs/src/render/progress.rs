//! Output of the work-progress verbs: task done/undone and the in-progress
//! marker.

use serde_json::{json, Value};
use speclink_core::command::{CommandError, ErrorCode, TaskFlipOutcome};

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// Which way a task flipped. `task done` and `task undone` differ only in
/// their status word and phrases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFlip {
    Done,
    Undone,
}

impl TaskFlip {
    /// The JSON status, the "already in that state" phrase, and the success
    /// line's verb phrase.
    fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            TaskFlip::Done => ("done", "is already done", "marked as done"),
            TaskFlip::Undone => ("undone", "is already not done", "marked as not done"),
        }
    }
}

/// One flipped task. `task_id` is the id as the caller gave it.
#[derive(Debug, Clone, Copy)]
pub struct FlippedTask<'a> {
    pub change: &'a str,
    pub task_id: &'a str,
    pub description: &'a str,
}

/// The error text for a task that already is in the asked state. Both entry
/// points fail with it and print nothing else.
pub fn already_message(flip: TaskFlip, task_id: &str) -> String {
    let (_, already_phrase, _) = flip.parts();
    format!("Task {task_id} {already_phrase}")
}

/// `task done --json` and `task undone --json`.
pub fn task_flip_json(flip: TaskFlip, task: &FlippedTask<'_>) -> Value {
    let (status, _, _) = flip.parts();
    json!({
        "change": task.change,
        "status": status,
        "task_desc": task.description,
        "task_id": task.task_id,
    })
}

/// `task done` and `task undone`: one success line.
pub fn task_flip_human(flip: TaskFlip, task: &FlippedTask<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    let (_, _, verb_phrase) = flip.parts();
    outln!(t, "{} Task {} {verb_phrase}: {}", paint.green("✓"), task.task_id, task.description);
    t
}

/// The JSON form is one line (frozen), not indented.
pub fn task_flip(flip: TaskFlip, task: &FlippedTask<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::compact(&task_flip_json(flip, task))), |paint| {
        task_flip_human(flip, task, paint)
    })
}

/// A task flip from the engine's result — or, for a task already in the
/// asked state, the error both entry points fail with (it names the task by
/// the index the engine resolved).
pub fn task_flipped(
    flip: TaskFlip,
    o: &TaskFlipOutcome,
    options: RenderOptions,
) -> Result<Rendered, CommandError> {
    if o.already {
        return Err(CommandError::new(ErrorCode::Error, already_message(flip, &o.task_id.to_string())));
    }
    Ok(task_flip(
        flip,
        &FlippedTask { change: &o.change, task_id: &o.task_id_arg, description: &o.description },
        options,
    ))
}

/// `in-progress remove`: a removed marker and "there was none" read
/// differently.
pub fn in_progress_remove_human(name: &str, removed: bool, paint: &Paint) -> Text {
    let mut t = Text::default();
    if removed {
        outln!(t, "{} Removed the in-progress marker from '{name}' — back to proposed", paint.green("✓"));
    } else {
        outln!(t, "Change '{name}' has no in-progress marker — already proposed");
    }
    t
}

/// `in-progress remove` has no JSON form.
pub fn in_progress_remove(name: &str, removed: bool, options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |paint| in_progress_remove_human(name, removed, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    const TASK: FlippedTask<'static> = FlippedTask {
        change: "alpha",
        task_id: "2",
        description: "1.2 Deliver behavior 2; verify with the unit test",
    };

    #[test]
    fn task_flip_human_names_the_new_state() {
        assert_eq!(
            task_flip_human(TaskFlip::Done, &TASK, &PLAIN).stdout,
            "✓ Task 2 marked as done: 1.2 Deliver behavior 2; verify with the unit test\n"
        );
        assert_eq!(
            task_flip_human(TaskFlip::Undone, &TASK, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Task 2 marked as not done: 1.2 Deliver behavior 2; verify with the unit test\n"
        );
    }

    #[test]
    fn task_flip_json_is_printed_on_one_line() {
        assert_eq!(
            task_flip_json(TaskFlip::Undone, &TASK),
            json!({
                "change": "alpha", "status": "undone",
                "task_desc": "1.2 Deliver behavior 2; verify with the unit test", "task_id": "2",
            })
        );
        assert_eq!(
            task_flip(TaskFlip::Done, &TASK, RenderOptions { json: true, color: false }).stdout,
            "{\"change\":\"alpha\",\"status\":\"done\",\"task_desc\":\"1.2 Deliver behavior 2; verify with the unit test\",\"task_id\":\"2\"}\n"
        );
    }

    #[test]
    fn already_message_names_the_task_and_its_state() {
        assert_eq!(already_message(TaskFlip::Done, "1"), "Task 1 is already done");
        assert_eq!(already_message(TaskFlip::Undone, "2"), "Task 2 is already not done");
    }

    #[test]
    fn in_progress_remove_human_tells_a_removal_from_nothing_to_remove() {
        assert_eq!(
            in_progress_remove_human("started", true, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Removed the in-progress marker from 'started' — back to proposed\n"
        );
        assert_eq!(
            in_progress_remove_human("beta", false, &COLOR).stdout,
            "Change 'beta' has no in-progress marker — already proposed\n"
        );
        let out = in_progress_remove("started", true, RenderOptions { json: true, color: false });
        assert_eq!(out.stdout, "✓ Removed the in-progress marker from 'started' — back to proposed\n");
        assert_eq!(out.json, None);
    }
}
