//! How each kind of speclink document maps to a key. Pure functions, no
//! I/O.
//!
//! Keys match the relative paths under a local project's `speclink/`
//! folder, with two exceptions: the board order has no local file, and the
//! archive lives under a top-level `archive/` instead of
//! `changes/archive/` and `discussions/archive/`. A list with the prefix
//! [`CHANGES`] or [`DISCUSSIONS`] thus never returns archived documents.

use crate::{DocKey, InvalidValue};

/// Every document of an active change.
pub const CHANGES: &str = "changes/";
/// Every document of an archived change.
pub const ARCHIVED_CHANGES: &str = "archive/changes/";
/// Every canonical spec.
pub const SPECS: &str = "specs/";
/// Every active discussion.
pub const DISCUSSIONS: &str = "discussions/";
/// Every archived discussion.
pub const ARCHIVED_DISCUSSIONS: &str = "archive/discussions/";

/// A name that becomes exactly one key segment.
fn segment<'a>(kind: &'static str, name: &'a str) -> Result<&'a str, InvalidValue> {
    if name.is_empty() {
        return Err(InvalidValue::new(kind, "must not be empty"));
    }
    if name.contains('/') {
        return Err(InvalidValue::new(kind, "must not contain '/'"));
    }
    Ok(name)
}

/// `YYYY-MM-DD`, digits only. The day and month ranges are not checked.
fn is_date(text: &[u8]) -> bool {
    text.len() == 10
        && text.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        })
}

/// A change name is one segment, and not `archive`: a local project keeps
/// its archived changes in `changes/archive/`.
fn change_name(name: &str) -> Result<&str, InvalidValue> {
    const KIND: &str = "change name";
    let name = segment(KIND, name)?;
    if name == "archive" {
        return Err(InvalidValue::new(
            KIND,
            "must not be 'archive', the local folder of archived changes",
        ));
    }
    Ok(name)
}

/// `changes/<name>/.meta.yaml`
pub fn change_meta(name: &str) -> Result<DocKey, InvalidValue> {
    DocKey::new(format!("{CHANGES}{}/.meta.yaml", change_name(name)?))
}

/// `changes/<name>/<artifact>`. The artifact path can have several
/// segments, for example `specs/auth/spec.md`.
pub fn change_artifact(name: &str, artifact: &str) -> Result<DocKey, InvalidValue> {
    DocKey::new(format!("{CHANGES}{}/{artifact}", change_name(name)?))
}

/// `changes/<name>/.evidence.json`
pub fn change_evidence(name: &str) -> Result<DocKey, InvalidValue> {
    DocKey::new(format!("{CHANGES}{}/.evidence.json", change_name(name)?))
}

/// `archive/changes/<dated-name>/<doc>`, where `dated_name` is the archive
/// date, a `-` and the change name (`2026-11-10-add-x`). The document path
/// can have several segments.
pub fn archived_change_doc(dated_name: &str, doc: &str) -> Result<DocKey, InvalidValue> {
    const KIND: &str = "archived change name";
    let dated_name = segment(KIND, dated_name)?;
    let bytes = dated_name.as_bytes();
    if bytes.len() < 12 || !is_date(&bytes[..10]) || bytes[10] != b'-' {
        return Err(InvalidValue::new(
            KIND,
            "must be a date in the form YYYY-MM-DD, a '-' and the change name",
        ));
    }
    DocKey::new(format!("{ARCHIVED_CHANGES}{dated_name}/{doc}"))
}

/// `specs/<capability>/spec.md`
pub fn spec(capability: &str) -> Result<DocKey, InvalidValue> {
    let capability = segment("capability name", capability)?;
    DocKey::new(format!("{SPECS}{capability}/spec.md"))
}

/// `discussions/<slug>.md`
pub fn discussion(slug: &str) -> Result<DocKey, InvalidValue> {
    let slug = segment("discussion slug", slug)?;
    DocKey::new(format!("{DISCUSSIONS}{slug}.md"))
}

/// `archive/discussions/<created>-<slug>.md`, where `created` is the
/// discussion's creation date (`2026-11-10`).
pub fn archived_discussion(created: &str, slug: &str) -> Result<DocKey, InvalidValue> {
    const KIND: &str = "creation date";
    let created = segment(KIND, created)?;
    if !is_date(created.as_bytes()) {
        return Err(InvalidValue::new(
            KIND,
            "must be a date in the form YYYY-MM-DD",
        ));
    }
    let slug = segment("discussion slug", slug)?;
    DocKey::new(format!("{ARCHIVED_DISCUSSIONS}{created}-{slug}.md"))
}

fn fixed(key: &'static str) -> DocKey {
    DocKey::new(key).expect("a fixed key follows the key format")
}

/// `config.yaml`
pub fn config() -> DocKey {
    fixed("config.yaml")
}

/// `LANGUAGE.md`
pub fn language() -> DocKey {
    fixed("LANGUAGE.md")
}

/// `board/order`
pub fn board_order() -> DocKey {
    fixed("board/order")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyPrefix;

    fn rule(err: InvalidValue) -> (&'static str, String) {
        (err.kind(), err.rule().to_string())
    }

    #[test]
    fn every_document_kind_gets_the_key_from_the_spec() {
        // 「鍵的命名規則」的對照表，一列一個斷言。
        let rows: [(DocKey, &str); 10] = [
            (change_meta("add-x").unwrap(), "changes/add-x/.meta.yaml"),
            (
                change_artifact("add-x", "specs/auth/spec.md").unwrap(),
                "changes/add-x/specs/auth/spec.md",
            ),
            (
                change_evidence("add-x").unwrap(),
                "changes/add-x/.evidence.json",
            ),
            (
                archived_change_doc("2026-11-10-add-x", "proposal.md").unwrap(),
                "archive/changes/2026-11-10-add-x/proposal.md",
            ),
            (spec("auth").unwrap(), "specs/auth/spec.md"),
            (
                discussion("board-search").unwrap(),
                "discussions/board-search.md",
            ),
            (
                archived_discussion("2026-11-10", "board-search").unwrap(),
                "archive/discussions/2026-11-10-board-search.md",
            ),
            (config(), "config.yaml"),
            (language(), "LANGUAGE.md"),
            (board_order(), "board/order"),
        ];
        for (key, expected) in rows {
            assert_eq!(key.as_str(), expected);
        }
    }

    #[test]
    fn names_must_be_one_non_empty_segment() {
        assert_eq!(
            rule(change_meta("").unwrap_err()),
            ("change name", "must not be empty".to_string())
        );
        assert_eq!(
            rule(change_artifact("a/b", "proposal.md").unwrap_err()),
            ("change name", "must not contain '/'".to_string())
        );
        assert_eq!(
            rule(change_evidence("a/b").unwrap_err()),
            ("change name", "must not contain '/'".to_string())
        );
        assert_eq!(
            rule(spec("").unwrap_err()),
            ("capability name", "must not be empty".to_string())
        );
        assert_eq!(
            rule(spec("auth/x").unwrap_err()),
            ("capability name", "must not contain '/'".to_string())
        );
        assert_eq!(
            rule(discussion("").unwrap_err()),
            ("discussion slug", "must not be empty".to_string())
        );
        assert_eq!(
            rule(discussion("a/b").unwrap_err()),
            ("discussion slug", "must not contain '/'".to_string())
        );
        assert_eq!(
            rule(archived_discussion("2026-11-10", "a/b").unwrap_err()),
            ("discussion slug", "must not contain '/'".to_string())
        );
        assert_eq!(
            rule(archived_discussion("", "board-search").unwrap_err()),
            ("creation date", "must not be empty".to_string())
        );
        assert_eq!(
            rule(archived_change_doc("", "proposal.md").unwrap_err()),
            ("archived change name", "must not be empty".to_string())
        );
    }

    #[test]
    fn a_change_cannot_be_named_after_the_archive_folder() {
        // 本機的 changes/archive/ 是封存資料夾，變更叫 archive 會撞名。
        for err in [
            change_meta("archive").unwrap_err(),
            change_artifact("archive", "proposal.md").unwrap_err(),
            change_evidence("archive").unwrap_err(),
        ] {
            assert_eq!(
                rule(err),
                (
                    "change name",
                    "must not be 'archive', the local folder of archived changes".to_string()
                )
            );
        }
        assert!(change_meta("archive-old").is_ok());
    }

    #[test]
    fn paths_may_have_segments_but_still_follow_the_key_format() {
        assert_eq!(
            archived_change_doc("2026-11-10-add-x", "specs/auth/spec.md")
                .unwrap()
                .as_str(),
            "archive/changes/2026-11-10-add-x/specs/auth/spec.md"
        );
        assert_eq!(
            rule(change_artifact("add-x", "../config.yaml").unwrap_err()),
            (
                "document key",
                "must not contain a '.' or '..' segment".to_string()
            )
        );
        assert_eq!(
            rule(change_artifact("..", "proposal.md").unwrap_err()),
            (
                "document key",
                "must not contain a '.' or '..' segment".to_string()
            )
        );
    }

    #[test]
    fn swapped_archive_arguments_are_rejected() {
        // 兩個參數都是字串，對調時編譯器不會擋，所以由格式檢查擋下。
        assert_eq!(
            rule(archived_discussion("board-search", "2026-11-10").unwrap_err()),
            (
                "creation date",
                "must be a date in the form YYYY-MM-DD".to_string()
            )
        );
        assert_eq!(
            rule(archived_change_doc("proposal.md", "2026-11-10-add-x").unwrap_err()),
            (
                "archived change name",
                "must be a date in the form YYYY-MM-DD, a '-' and the change name".to_string()
            )
        );
        assert_eq!(
            rule(archived_change_doc("2026-11-10-", "proposal.md").unwrap_err()),
            (
                "archived change name",
                "must be a date in the form YYYY-MM-DD, a '-' and the change name".to_string()
            )
        );
        assert_eq!(
            rule(archived_change_doc("2026-11-1x-add-x", "proposal.md").unwrap_err()),
            (
                "archived change name",
                "must be a date in the form YYYY-MM-DD, a '-' and the change name".to_string()
            )
        );
    }

    #[test]
    fn listing_active_documents_never_reaches_the_archive() {
        let archived = [
            archived_change_doc("2026-11-10-add-x", "proposal.md").unwrap(),
            archived_discussion("2026-11-10", "board-search").unwrap(),
        ];
        for prefix in [CHANGES, DISCUSSIONS] {
            let prefix = KeyPrefix::new(prefix).unwrap();
            for key in &archived {
                assert!(
                    !prefix.matches(key),
                    "{} matches {}",
                    prefix.as_str(),
                    key.as_str()
                );
            }
        }
        let active = [
            (CHANGES, change_meta("add-x").unwrap()),
            (ARCHIVED_CHANGES, archived[0].clone()),
            (SPECS, spec("auth").unwrap()),
            (DISCUSSIONS, discussion("board-search").unwrap()),
            (ARCHIVED_DISCUSSIONS, archived[1].clone()),
        ];
        for (prefix, key) in active {
            assert!(KeyPrefix::new(prefix).unwrap().matches(&key));
        }
    }

    #[test]
    fn a_store_lists_active_changes_without_the_archived_ones() {
        use crate::{Actor, Commit, Expected, MemoryStorePort, Op, ScopeId, StorePort, Timestamp};

        let store = MemoryStorePort::new();
        let scope = ScopeId::new("s").unwrap();
        let active = change_meta("add-x").unwrap();
        let archived = archived_change_doc("2026-11-10-add-y", "proposal.md").unwrap();
        let ops = [&active, &archived]
            .into_iter()
            .map(|key| Op::Put {
                key: key.clone(),
                content: "x".into(),
                expected: Expected::Absent,
            })
            .collect();
        let commit = Commit::new(
            Actor::new("u-1", "Momo").unwrap(),
            "archive",
            Timestamp::new(1).unwrap(),
            ops,
            Vec::new(),
        )
        .unwrap();
        store.commit(&scope, &commit).unwrap();

        let listing = store
            .list(&scope, &KeyPrefix::new(CHANGES).unwrap())
            .unwrap();
        let keys: Vec<&str> = listing
            .entries
            .iter()
            .map(|entry| entry.key.as_str())
            .collect();
        assert_eq!(keys, [active.as_str()]);
    }
}
