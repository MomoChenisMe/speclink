//! The Agent tool definition (design D9): one tool named `speclink` whose input
//! is verb text, with a description that lists every verb an engine command
//! stands for. The list is walked from the same clap definition `parse` uses,
//! so a verb added there reaches the description too.

use crate::families::ToCommand;
use crate::inputs::{CommandTarget, VerbInputs};

/// One tool for a host's Agent framework: its name, the text the model reads,
/// and the JSON Schema of its input.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: String,
    pub input_schema: serde_json::Value,
}

const USAGE: &str = "Run one speclink verb on this project's specs, changes and discussions.

`argv` is the verb text without the program name, one word per item: [\"list\", \"--json\"] or [\"new\", \"change\", \"add-auth\"].
A verb that writes content (an artifact, a discussion round, a stamp's scope) reads it from `stdin`: when the verb has a --stdin flag, put --stdin in argv and the content in `stdin`.
Add --json to get a structured result instead of text.
Add --help to a verb to get its arguments; the help text comes back as the error message.
";

/// The `speclink` tool: argv and an optional stdin in, the verb list in the
/// description. Verbs that need a checkout are left out.
pub fn tool_definition() -> ToolDefinition {
    let mut tree = crate::parse::verb_tree();
    tree.build();
    let mut lines = Vec::new();
    for sub in tree.get_subcommands_mut().filter(|s| s.get_name() != "help") {
        list_leaves(sub, &mut vec![sub.get_name().to_string()], &mut lines);
    }
    ToolDefinition {
        name: "speclink",
        description: format!("{USAGE}\nVerbs:\n{}", lines.join("\n")),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "argv": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "The verb text without the program name, one word per item."
                },
                "stdin": {
                    "type": "string",
                    "description": "The content a verb reads with --stdin."
                }
            },
            "required": ["argv"],
            "additionalProperties": false
        }),
    }
}

/// One line per leaf verb under `cmd` that converts to an engine command:
/// its usage and its one-line help.
fn list_leaves(cmd: &mut clap::Command, words: &mut Vec<String>, lines: &mut Vec<String>) {
    let has_subs = cmd.get_subcommands().any(|s| s.get_name() != "help");
    if !has_subs {
        if minimal_argv_of(words, cmd).is_some() {
            let usage = cmd.render_usage().to_string();
            let usage = usage.trim().trim_start_matches("Usage:").trim().trim_start_matches("speclink ");
            let about = cmd.get_about().map(|a| a.to_string()).unwrap_or_default();
            lines.push(format!("- {usage}: {about}"));
        }
        return;
    }
    for sub in cmd.get_subcommands_mut().filter(|s| s.get_name() != "help") {
        words.push(sub.get_name().to_string());
        list_leaves(sub, words, lines);
        words.pop();
    }
}

/// The shortest argv that converts this leaf to an engine command, or `None`
/// when the leaf has none: the verb words plus a placeholder for each
/// required argument, then the same with `--stdin` when the verb has that
/// flag (some verbs convert only with it). The placeholder is a valid
/// kebab-case name, so the conversion's name checks (a tool name) accept it.
fn minimal_argv_of(words: &[String], leaf: &clap::Command) -> Option<Vec<String>> {
    let mut argv = words.to_vec();
    let push = |argv: &mut Vec<String>, arg: &clap::Arg| {
        let value = arg
            .get_possible_values()
            .first()
            .map(|v| v.get_name().to_string())
            .unwrap_or_else(|| "demo".to_string());
        match arg.get_long() {
            None => argv.push(value),
            Some(long) => {
                argv.push(format!("--{long}"));
                if arg.get_action().takes_values() {
                    argv.push(value);
                }
            }
        }
    };
    for arg in leaf.get_arguments().filter(|a| a.is_required_set()) {
        push(&mut argv, arg);
    }
    // A required group (`--before` or `--after`) asks for one of its
    // arguments: the first one stands for the group.
    for group in leaf.get_groups().filter(|g| g.is_required_set()) {
        let first = group.get_args().next();
        if let Some(arg) = first.and_then(|id| leaf.get_arguments().find(|a| a.get_id() == id)) {
            push(&mut argv, arg);
        }
    }
    if converts(&argv) {
        return Some(argv);
    }
    let has_stdin_flag = leaf.get_arguments().any(|a| a.get_long() == Some("stdin") && !a.is_hide_set());
    argv.push("--stdin".to_string());
    (has_stdin_flag && converts(&argv)).then_some(argv)
}

fn converts(argv: &[String]) -> bool {
    match crate::parse::parse(argv) {
        Ok(parsed) => matches!(parsed.to_command(VerbInputs::default()), Ok(CommandTarget::Command(_))),
        Err(_) => false,
    }
}

/// [`minimal_argv_of`] for the leaf named by `verb` (`discuss add-round`).
#[cfg(test)]
fn minimal_argv(verb: &str) -> Option<Vec<String>> {
    let words: Vec<String> = verb.split(' ').map(str::to_string).collect();
    let mut tree = crate::parse::verb_tree();
    tree.build();
    let mut leaf = &tree;
    for word in &words {
        leaf = leaf.find_subcommand(word)?;
    }
    minimal_argv_of(&words, leaf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::families::test_support::local_only_of;
    use std::collections::BTreeSet;

    /// The leaf verbs that need a checkout: the CLI runs them, a host cannot.
    const NEEDS_CHECKOUT: &[&str] = &["drift", "review prepare", "review scope", "verify scope"];

    /// Every leaf verb of the clap tree, as its words (`discuss add-round`).
    fn leaves() -> BTreeSet<String> {
        fn walk(cmd: &clap::Command, words: &mut Vec<String>, out: &mut BTreeSet<String>) {
            let subs: Vec<&clap::Command> = cmd.get_subcommands().filter(|s| s.get_name() != "help").collect();
            if subs.is_empty() {
                out.insert(words.join(" "));
            }
            for sub in subs {
                words.push(sub.get_name().to_string());
                walk(sub, words, out);
                words.pop();
            }
        }
        let mut out = BTreeSet::new();
        walk(&crate::parse::verb_tree(), &mut Vec::new(), &mut out);
        out
    }

    /// The verb words of each line of the description's verb list: the words
    /// before the first `[` or `<`.
    fn listed() -> BTreeSet<String> {
        let description = tool_definition().description;
        let (_, list) = description.split_once("\nVerbs:\n").expect("the description has a verb list");
        list.lines()
            .filter_map(|line| line.strip_prefix("- "))
            .map(|line| {
                let usage = line.split(':').next().unwrap();
                usage
                    .split_whitespace()
                    .take_while(|w| !w.starts_with('[') && !w.starts_with('<'))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }

    #[test]
    fn the_tool_is_named_speclink() {
        assert_eq!(tool_definition().name, "speclink");
    }

    #[test]
    fn the_input_is_argv_and_an_optional_stdin_only() {
        let schema = tool_definition().input_schema;
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], serde_json::json!(["argv"]));
        assert_eq!(schema["additionalProperties"], false);
        let properties = schema["properties"].as_object().unwrap();
        assert_eq!(properties.keys().collect::<Vec<_>>(), ["argv", "stdin"]);
        assert_eq!(properties["argv"]["type"], "array");
        assert_eq!(properties["argv"]["items"]["type"], "string");
        assert_eq!(properties["stdin"]["type"], "string");
    }

    #[test]
    fn the_description_tells_how_to_use_stdin_and_json() {
        let description = tool_definition().description;
        assert!(description.contains("--stdin"), "{description}");
        assert!(description.contains("--json"), "{description}");
        assert!(description.contains("without the program name"), "{description}");
    }

    #[test]
    fn the_listed_verbs_are_the_leaves_with_an_engine_command() {
        let (leaves, listed) = (leaves(), listed());
        let checkout: BTreeSet<String> = NEEDS_CHECKOUT.iter().map(|v| v.to_string()).collect();
        assert!(listed.is_disjoint(&checkout), "{:?}", listed.intersection(&checkout).collect::<Vec<_>>());
        let covered: BTreeSet<String> = listed.union(&checkout).cloned().collect();
        assert_eq!(covered, leaves, "every leaf verb is listed or needs a checkout");
    }

    #[test]
    fn every_listed_verb_converts_to_an_engine_command() {
        for verb in listed() {
            let argv = minimal_argv(&verb).unwrap_or_else(|| panic!("'{verb}' has no minimal argv"));
            let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
            assert_eq!(local_only_of(&argv), None, "{argv:?}");
        }
    }

    #[test]
    fn the_verbs_that_need_a_checkout_are_local_only() {
        for (verb, argv) in [
            ("drift", &["drift", "demo"][..]),
            ("review prepare", &["review", "prepare", "demo"][..]),
            ("review scope", &["review", "scope", "demo"][..]),
            ("verify scope", &["verify", "scope", "demo"][..]),
        ] {
            assert_eq!(local_only_of(argv), Some(verb), "{argv:?}");
        }
    }
}
