//! The free-field verbs (host-free-fields D4): `change field set` / `unset`
//! and `discuss field set` / `unset`. Both families hold the same two
//! subcommands; [`FieldKind`] says which family a parsed verb belongs to.
//! The `--field` flag of `new change` and `discuss new` reads its pairs with
//! the same [`parse_pairs`].

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError, ErrorCode};
use speclink_core::fields::Fields;

#[derive(Args, Debug)]
pub struct FieldArgs {
    #[command(subcommand)]
    pub command: FieldCommands,
}
#[derive(Subcommand, Debug)]
pub enum FieldCommands {
    /// Set free fields: your own key=value pairs, stored and never read by speclink (do not put a secret in one)
    Set {
        /// Change name, or discussion slug
        name: String,
        /// Fields to set; the value is everything after the first `=`
        #[arg(required = true, value_name = "KEY=VALUE")]
        fields: Vec<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove free fields (a key that is not there is not an error)
    Unset {
        /// Change name, or discussion slug
        name: String,
        /// Keys to remove
        #[arg(required = true, value_name = "KEY")]
        keys: Vec<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// One field verb's request: the change name or discussion slug, the fields
/// to set and the keys to remove. An entry point that does not run the engine
/// command itself (the CLI's remote mode) sends these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldEdit {
    /// The change name or discussion slug.
    pub subject: String,
    pub set: Fields,
    pub unset: Vec<String>,
}

/// The kind of document a field verb edits: the `change` family's verbs edit
/// a change, the `discuss` family's a discussion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Change,
    Discussion,
}

impl FieldArgs {
    /// The verb's stable name, as the other families give theirs.
    pub fn name(&self, kind: FieldKind) -> &'static str {
        match (kind, &self.command) {
            (FieldKind::Change, FieldCommands::Set { .. }) => "change field set",
            (FieldKind::Change, FieldCommands::Unset { .. }) => "change field unset",
            (FieldKind::Discussion, FieldCommands::Set { .. }) => "discuss field set",
            (FieldKind::Discussion, FieldCommands::Unset { .. }) => "discuss field unset",
        }
    }

    pub fn json(&self) -> bool {
        match self.command {
            FieldCommands::Set { json, .. } | FieldCommands::Unset { json, .. } => json,
        }
    }

    /// The change name or discussion slug the verb names.
    pub fn subject(&self) -> &str {
        match &self.command {
            FieldCommands::Set { name, .. } | FieldCommands::Unset { name, .. } => name,
        }
    }

    /// What the verb asks for. A verb sets or unsets, never both: the other
    /// list is empty.
    pub fn edit(&self) -> Result<FieldEdit, CommandError> {
        Ok(match &self.command {
            FieldCommands::Set { name, fields, .. } => {
                FieldEdit { subject: name.clone(), set: parse_pairs(fields)?, unset: Vec::new() }
            }
            FieldCommands::Unset { name, keys, .. } => {
                if let Some(key) = repeated(keys.iter().map(String::as_str)) {
                    return Err(given_twice(key));
                }
                FieldEdit { subject: name.clone(), set: Fields::new(), unset: keys.clone() }
            }
        })
    }

    /// The edit command for a document of `kind`.
    pub fn command(&self, kind: FieldKind) -> Result<Command, CommandError> {
        let FieldEdit { subject, set, unset } = self.edit()?;
        Ok(match kind {
            FieldKind::Change => Command::ChangeFieldsEdit { name: subject, set, unset },
            FieldKind::Discussion => Command::DiscussFieldsEdit { slug: subject, set, unset },
        })
    }
}

fn repeated<'a>(keys: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut seen = std::collections::BTreeSet::new();
    keys.into_iter().find(|key| !seen.insert(*key))
}

fn given_twice(key: &str) -> CommandError {
    CommandError::new(ErrorCode::InvalidArgv, format!("field '{key}' is given twice"))
}

/// `<key>=<value>` arguments as fields: each one splits at its first `=`, so a
/// value can hold `=`. An argument with no `=` and a key given twice are
/// refused. The message of the first gives the argument's position and not
/// its text: a word with no `=` can be the value the caller meant to store.
pub fn parse_pairs(pairs: &[String]) -> Result<Fields, CommandError> {
    let mut fields = Fields::new();
    for (index, pair) in pairs.iter().enumerate() {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(CommandError::new(
                ErrorCode::InvalidArgv,
                format!("field argument {} has no '=' — expected <key>=<value>", index + 1),
            ));
        };
        if fields.insert(key.to_string(), value.to_string()).is_some() {
            return Err(given_twice(key));
        }
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of, target_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, ErrorCode};
    use speclink_core::fields::Fields;

    fn map(pairs: &[(&str, &str)]) -> Fields {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_pair_splits_at_its_first_equals_sign() {
        // 規格「值含等號」：query=a=b。
        assert_eq!(
            command_of(
                &["change", "field", "set", "demo", "ticket=PROJ-1", "query=a=b", "empty=", "--json"],
                VerbInputs::default()
            ),
            Command::ChangeFieldsEdit {
                name: "demo".into(),
                set: map(&[("ticket", "PROJ-1"), ("query", "a=b"), ("empty", "")]),
                unset: vec![],
            }
        );
    }

    #[test]
    fn the_four_verbs_convert_to_the_two_edit_commands() {
        let rows: Vec<(&[&str], Command)> = vec![
            (
                &["change", "field", "set", "demo", "a=1"],
                Command::ChangeFieldsEdit { name: "demo".into(), set: map(&[("a", "1")]), unset: vec![] },
            ),
            (
                &["change", "field", "unset", "demo", "ticket", "ghost", "--no-color"],
                Command::ChangeFieldsEdit { name: "demo".into(), set: Fields::new(), unset: strings(&["ticket", "ghost"]) },
            ),
            (
                &["discuss", "field", "set", "board-search", "owner=alice", "--json"],
                Command::DiscussFieldsEdit { slug: "board-search".into(), set: map(&[("owner", "alice")]), unset: vec![] },
            ),
            (
                &["discuss", "field", "unset", "board-search", "owner"],
                Command::DiscussFieldsEdit { slug: "board-search".into(), set: Fields::new(), unset: strings(&["owner"]) },
            ),
        ];
        for (argv, want) in rows {
            assert_eq!(command_of(argv, VerbInputs::default()), want, "{argv:?}");
            // 四個動詞都不讀 stdin：給了也不進命令。
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
            let piped = VerbInputs { stdin: Some("x=y".into()), ..Default::default() };
            assert_eq!(command_of(argv, piped), want, "{argv:?}");
        }
    }

    #[test]
    fn names_are_the_subcommand_words() {
        for (argv, name) in [
            (&["change", "field", "set", "x", "a=1"][..], "change field set"),
            (&["change", "field", "unset", "x", "a"], "change field unset"),
            (&["discuss", "field", "set", "x", "a=1"], "discuss field set"),
            (&["discuss", "field", "unset", "x", "a"], "discuss field unset"),
        ] {
            assert_eq!(name_of(argv), name);
        }
    }

    #[test]
    fn an_argument_without_an_equals_sign_or_a_repeated_key_is_invalid_argv() {
        let rows: &[&[&str]] = &[
            &["change", "field", "set", "demo", "ticket"],
            &["change", "field", "set", "demo", "a=1", "s3cret"],
            &["change", "field", "set", "demo", "a=1", "a=2"],
            &["change", "field", "unset", "demo", "a", "a"],
            &["discuss", "field", "set", "t", "owner"],
            &["discuss", "field", "set", "t", "a=1", "b=2", "a=3"],
            &["discuss", "field", "unset", "t", "a", "b", "a"],
            &["new", "change", "demo", "--field", "ticket"],
            &["new", "change", "demo", "--field", "a=1", "--field", "a=2"],
            &["discuss", "new", "Topic", "--field", "owner"],
            &["discuss", "new", "Topic", "--field", "a=1", "--field", "a=1"],
        ];
        for argv in rows {
            let err = target_of(argv, VerbInputs::default()).expect_err("refused");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{argv:?}: {}", err.message);
            // 沒有 `=` 的那個字可能就是使用者想放的值，訊息不照抄。
            assert!(!err.message.contains("s3cret") && !err.message.contains("=2"), "{}", err.message);
        }
    }

    #[test]
    fn a_missing_name_or_a_missing_field_is_a_usage_error() {
        for argv in [
            &["change", "field", "set", "demo"][..],
            &["change", "field", "unset", "demo"],
            &["change", "field", "set"],
            &["discuss", "field", "unset"],
            &["change", "field"],
        ] {
            let err = crate::parse(argv.iter().copied()).err().expect("usage error");
            assert_eq!(err.kind, crate::ParseErrorKind::Usage, "{argv:?}");
        }
    }

    #[test]
    fn the_field_flag_repeats_on_the_two_create_verbs() {
        // 規格「建立時附帶」：new change demo --field ticket=PROJ-1 --field source=portal。
        match command_of(
            &["new", "change", "demo", "--field", "ticket=PROJ-1", "--field", "source=portal", "--field=query=a=b"],
            VerbInputs::default(),
        ) {
            Command::NewChange { name, fields, .. } => {
                assert_eq!(name, "demo");
                assert_eq!(fields, map(&[("ticket", "PROJ-1"), ("source", "portal"), ("query", "a=b")]));
            }
            other => panic!("expected new change, got {other:?}"),
        }
        match command_of(&["discuss", "new", "Board search", "--field", "owner=alice", "--json"], VerbInputs::default()) {
            Command::DiscussNew { topic, fields, .. } => {
                assert_eq!(topic, "Board search");
                assert_eq!(fields, map(&[("owner", "alice")]));
            }
            other => panic!("expected discuss new, got {other:?}"),
        }
        // 沒有旗標就是沒有欄位。
        assert!(matches!(
            command_of(&["new", "change", "demo"], VerbInputs::default()),
            Command::NewChange { fields, .. } if fields.is_empty()
        ));
        assert!(matches!(
            command_of(&["discuss", "new", "T"], VerbInputs::default()),
            Command::DiscussNew { fields, .. } if fields.is_empty()
        ));
    }

    #[test]
    fn the_json_flag_is_read_from_argv() {
        use crate::families::fields::FieldCommands;
        use crate::families::plan::ChangeCommands;
        use crate::SpecVerb;
        for (argv, want) in [
            (&["change", "field", "set", "demo", "a=1"][..], false),
            (&["change", "field", "set", "demo", "a=1", "--json"], true),
            (&["change", "field", "unset", "demo", "a", "--json"], true),
        ] {
            let parsed = crate::parse(argv.iter().copied()).expect("parses");
            let SpecVerb::Change(change) = parsed.spec() else { panic!("expected the change family") };
            let ChangeCommands::Field(field) = &change.command else { panic!("expected the field verbs") };
            let json = match &field.command {
                FieldCommands::Set { json, .. } | FieldCommands::Unset { json, .. } => *json,
            };
            assert_eq!(json, want, "{argv:?}");
            assert_eq!(field.json(), want, "{argv:?}");
        }
    }
}
