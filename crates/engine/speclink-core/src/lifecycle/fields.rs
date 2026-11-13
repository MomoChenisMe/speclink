//! Free fields (host-free-fields D1, D2): key/value pairs a host or a user
//! keeps on a change or a discussion.
//!
//! The engine stores them and reads them back. It decides nothing by them:
//! no verb reads a key or a value to choose what it does.
//!
//! They live in a `fields:` block — in the change's metadata, and inside the
//! frontmatter of a discussion record:
//!
//! ```yaml
//! fields:
//!   source: "portal"
//!   ticket: "PROJ-123"
//! ```
//!
//! One key per line, in byte order, the value always in double quotes
//! ([`crate::util::yaml_quoted`]). The block goes away with its last key.
//!
//! The rules ([`check`]) apply to what a write adds. A read is lenient: a
//! block edited by hand gives every scalar entry as a string, as written,
//! and skips the entries that are not scalars. A write replaces the whole
//! block, so it is done only on a block of one `key: value` line per entry
//! that the read gave whole: an edit never drops or changes what a hand edit
//! put there.

use crate::command::{InvalidContent, Refusal};
use crate::keylines::KeyLines;
use crate::store::Store;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};

/// A document's free fields. The map's order is the order of the block.
pub type Fields = BTreeMap<String, String>;

/// The block's key in the metadata and in the frontmatter.
pub const FIELDS_KEY: &str = "fields";
/// The most keys one document holds.
pub const MAX_KEYS: usize = 32;
/// The longest key, in characters (all of them ASCII).
pub const MAX_KEY_LEN: usize = 64;
/// The longest value, in bytes.
pub const MAX_VALUE_BYTES: usize = 1024;

/// The result of one edit: every field after it, and the keys it changed
/// (in byte order; empty when the edit changed nothing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edited {
    pub fields: Fields,
    pub changed: Vec<String>,
}

/// `^[a-z][a-z0-9_.-]{0,63}$`
fn key_fits(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && key.len() <= MAX_KEY_LEN
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-'))
}

/// One entry against the rules. The message names the key and never the
/// value: a value can be the host's own data.
fn check_entry(key: &str, value: &str) -> Result<()> {
    if !key_fits(key) {
        bail!(InvalidContent(format!(
            "invalid field key {key:?} — expected a lowercase letter, then up to 63 of a-z, 0-9, '_', '.' and '-'"
        )));
    }
    if value.len() > MAX_VALUE_BYTES || value.chars().any(char::is_control) {
        bail!(InvalidContent(format!(
            "invalid value for field '{key}' — expected one line of at most {MAX_VALUE_BYTES} bytes with no control characters"
        )));
    }
    Ok(())
}

fn too_many() -> anyhow::Error {
    InvalidContent(format!("too many fields — one document holds at most {MAX_KEYS}")).into()
}

/// The rules for a whole set that a new document starts with.
pub fn check(fields: &Fields) -> Result<()> {
    for (key, value) in fields {
        check_entry(key, value)?;
    }
    if fields.len() > MAX_KEYS {
        return Err(too_many());
    }
    Ok(())
}

/// `existing` after the edit: `unset` first, then `set`. Every check runs
/// here, before a write. A key that is not there is not an error to unset.
/// The count is the count after the edit: a block that a hand edit made too
/// large takes no write until one edit unsets enough keys.
pub fn apply(existing: &Fields, set: &Fields, unset: &[String]) -> Result<Edited> {
    if set.is_empty() && unset.is_empty() {
        bail!(InvalidContent("no field to set or unset".to_string()));
    }
    if let Some(key) = unset.iter().find(|key| set.contains_key(*key)) {
        bail!(InvalidContent(format!("field '{key}' is both set and unset")));
    }
    for (key, value) in set {
        check_entry(key, value)?;
    }
    let mut fields = existing.clone();
    let mut changed = BTreeSet::new();
    for key in unset {
        if fields.remove(key).is_some() {
            changed.insert(key.clone());
        }
    }
    for (key, value) in set {
        if fields.insert(key.clone(), value.clone()).as_ref() != Some(value) {
            changed.insert(key.clone());
        }
    }
    if !changed.is_empty() && fields.len() > MAX_KEYS {
        return Err(too_many());
    }
    Ok(Edited { fields, changed: changed.into_iter().collect() })
}

/// The block's lines below `fields:`. A key that a hand edit left outside
/// the rule is quoted as well, so the line stays one YAML entry.
fn block_body(fields: &Fields) -> Vec<String> {
    fields
        .iter()
        .map(|(key, value)| {
            // Without quotes YAML reads these three as a null or a boolean, not as a key.
            let plain = key_fits(key) && !matches!(key.as_str(), "null" | "true" | "false");
            let key = if plain { key.clone() } else { crate::util::yaml_quoted(key) };
            format!("  {key}: {}", crate::util::yaml_quoted(value))
        })
        .collect()
}

/// The whole block as text, for a document that is written in one piece at
/// its creation. Empty when there are no fields.
pub fn block_text(fields: &Fields) -> String {
    if fields.is_empty() {
        return String::new();
    }
    let mut text = format!("{FIELDS_KEY}:\n");
    for line in block_body(fields) {
        text.push_str(&line);
        text.push('\n');
    }
    text
}

/// The fields of a document, read leniently. A string is its value. A
/// number, a boolean or a null is the text as written on its key's line
/// (`1.10` stays `1.10`); where the block has no such line, or the value is
/// not on it, it is the value as YAML shows it (`1.1`).
/// An entry that is not a scalar is skipped. A `fields` key that is not a
/// mapping, or a block that YAML cannot read, gives no fields.
pub fn read(lines: &KeyLines) -> Fields {
    parse(lines).fields
}

/// A block as [`read`] gives it, and whether a write can replace it with
/// nothing lost. `intact` is true only for a block of one `key: value` line
/// per entry that the read gave whole. It is false when:
///
/// - the lenient read left something out (an entry that is not a scalar, two
///   entries with one key, a block it could not read at all);
/// - a line of the block, read alone, is not one entry of the block: a value
///   goes on over more lines, and one rule covers every such shape;
/// - a number, a boolean or a null was read as YAML shows it because the
///   block has no `key:` line with its word (a flow-style block, a quoted key);
/// - the lines an edit removes are not everything YAML reads as the block:
///   the edit would leave the rest behind as stray lines.
struct Block {
    fields: Fields,
    intact: bool,
}

/// The value YAML gives the `fields` key in `text` (a key line and the lines
/// after it). `None`: YAML cannot read the text.
fn value_in(text: &str) -> Option<serde_yaml::Value> {
    match serde_yaml::from_str(text) {
        Ok(serde_yaml::Value::Mapping(mut document)) => document.remove(FIELDS_KEY),
        _ => None,
    }
}

fn parse(lines: &KeyLines) -> Block {
    use serde_yaml::Value;
    let none = |intact| Block { fields: Fields::new(), intact };
    let Some(whole) = lines.yaml_extent(FIELDS_KEY) else {
        return none(true);
    };
    let Some(value) = value_in(&whole) else {
        return none(false);
    };
    // The lines an edit removes ([`KeyLines::raw`]) must hold the same value.
    let all_removed = lines.raw(FIELDS_KEY).and_then(|text| value_in(&text)).as_ref() == Some(&value);
    let entries = match &value {
        Value::Mapping(entries) => entries,
        // `fields:` with nothing below it holds nothing.
        Value::Null => return none(all_removed),
        _ => return none(false),
    };
    // The block's lines below its key line, without the blank lines and the
    // comments, and whether each one, read alone, is one entry of the block:
    // the same key with the same value. A line that is not is a part of a
    // value that goes on over more lines (the next line, a multi-line string,
    // a nested entry, a flow-style block); a rewrite keeps only what one
    // `key: value` line holds, so it is not done on such a block.
    let entry_lines: Vec<&str> =
        whole.lines().skip(1).filter(|line| !matches!(line.trim_start().chars().next(), None | Some('#'))).collect();
    let one_entry_each = entry_lines.iter().all(|line| {
        matches!(serde_yaml::from_str(line), Ok(Value::Mapping(alone))
            if alone.len() == 1 && alone.iter().all(|(key, value)| entries.get(key) == Some(value)))
    });
    // False once a number, a boolean or a null reads as YAML shows it because
    // its text was not found (a flow-style block, a quoted key): a rewrite
    // would put that reading in place of the text.
    let mut as_written_all = true;
    let mut shown = |text: String| {
        as_written_all = false;
        text
    };
    let fields: Fields = entries
        .iter()
        .filter_map(|(key, value)| {
            let key = match key {
                Value::String(key) => key.clone(),
                Value::Number(key) => key.to_string(),
                Value::Bool(key) => key.to_string(),
                _ => return None,
            };
            let as_yaml_shows = match value {
                Value::String(value) => return Some((key, value.clone())),
                Value::Number(number) => number.to_string(),
                Value::Bool(flag) => flag.to_string(),
                Value::Null => String::new(),
                _ => return None,
            };
            // A number, a boolean or a null is the word on its key's line, when
            // that word alone reads as the same value: `2.10}` in a flow-style
            // block does not, and no word at all is only a null.
            let word = word_on_key_line(&entry_lines, &key)
                .filter(|word| serde_yaml::from_str::<Value>(word).ok().as_ref() == Some(value));
            Some((key, word.unwrap_or_else(|| shown(as_yaml_shows))))
        })
        .collect();
    Block { intact: all_removed && one_entry_each && as_written_all && fields.len() == entries.len(), fields }
}

/// The word after `key:` on the key's own line: YAML gives `1.10` as the
/// number 1.1, and the field is the text. A number, a boolean or a null is one
/// word; what comes after it is a comment, and so is a word that starts with
/// `#` (the empty text). `entry_lines` are the block's lines as [`parse`] takes
/// them; the key's line is one at the indent of the first, so a nested entry
/// with the same key does not count. `None`: the block has no line for the key
/// (a flow-style block, a quoted key).
fn word_on_key_line(entry_lines: &[&str], key: &str) -> Option<String> {
    let base = entry_lines.first()?.len() - entry_lines.first()?.trim_start().len();
    entry_lines
        .iter()
        .filter(|line| line.len() - line.trim_start().len() == base)
        .find_map(|line| line.trim_start().strip_prefix(key)?.strip_prefix(':'))
        .map(|rest| rest.split_whitespace().next().filter(|word| !word.starts_with('#')).unwrap_or_default().to_string())
}

/// Writes the block, or removes it when no field is left, and makes sure the
/// document reads back as `fields`. The edit only sees a key written as
/// `fields:` in the first column; a hand-written `fields : …` or `"fields":`
/// would get a second block beside it, so such a document is refused. So is
/// a block the lenient read does not give whole: the write would drop the rest.
fn write(lines: &mut KeyLines, fields: &Fields, document: &str) -> Result<()> {
    let cannot_rewrite = || {
        Refusal(format!(
            "cannot write the fields of {document}: it has a `{FIELDS_KEY}` entry this command cannot rewrite — \
             make it a plain `{FIELDS_KEY}:` block first"
        ))
    };
    // Before the block is replaced: nothing a hand edit put there is lost,
    // and no second `fields` key is removed with the first.
    if lines.has_unseen(FIELDS_KEY) || lines.count(FIELDS_KEY) > 1 || !parse(lines).intact {
        bail!(cannot_rewrite());
    }
    if fields.is_empty() {
        lines.remove(FIELDS_KEY);
    } else {
        // The one failure left is a frontmatter with no closing fence: the
        // block has no place to go. It is the document's state, not a defect.
        lines
            .set_block(FIELDS_KEY, &block_body(fields))
            .map_err(|e| Refusal(format!("cannot write the fields of {document}: {e}")))?;
    }
    if read(lines) != *fields {
        bail!(cannot_rewrite());
    }
    Ok(())
}

/// Edits the fields of the active change `name`: one edit of its metadata,
/// or no write at all when a check fails or nothing changes. `Ok(None)`: the
/// change has no metadata document.
pub fn edit_change(store: &dyn Store, name: &str, set: &Fields, unset: &[String]) -> Result<Option<Edited>> {
    crate::model::edit_meta(store, name, |lines| {
        let edited = apply(&read(lines), set, unset)?;
        if !edited.changed.is_empty() {
            write(lines, &edited.fields, &store.change_meta_display(name))?;
            // The whole document must still be a metadata document.
            crate::model::check_meta_text(store, name, Some(&lines.text()))?;
        }
        Ok(edited)
    })
}

/// Edits the fields of the live discussion `slug`, inside its frontmatter;
/// the rest of the record stays byte for byte. `Ok(None)`: no live record
/// has that slug.
pub fn edit_discussion(store: &dyn Store, slug: &str, set: &Fields, unset: &[String]) -> Result<Option<Edited>> {
    let Some(text) = store.read_live_discussion(slug) else {
        return Ok(None);
    };
    let mut lines = KeyLines::frontmatter(&text);
    let edited = apply(&lines.as_ref().map(read).unwrap_or_default(), set, unset)?;
    if !edited.changed.is_empty() {
        let Some(lines) = lines.as_mut() else {
            bail!(Refusal(format!("discussion '{slug}' has no frontmatter — it cannot hold fields")));
        };
        write(lines, &edited.fields, &format!("discussion '{slug}'"))?;
        store.write_live_discussion(slug, &lines.text())?;
    }
    Ok(Some(edited))
}

#[cfg(test)]
mod tests {
    use super::{edit_change, edit_discussion, Fields};
    use crate::discuss::DiscussionHead;
    use crate::model::ChangeMeta;
    use crate::store::Store;
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";

    fn map(pairs: &[(&str, &str)]) -> Fields {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn set(store: &TestStore, pairs: &[(&str, &str)]) -> anyhow::Result<()> {
        edit_change(store, "demo", &map(pairs), &[]).map(|edited| {
            edited.expect("the change has a metadata document");
        })
    }

    fn stored(store: &TestStore) -> ChangeMeta {
        ChangeMeta::from_text(Some(&store.meta("demo"))).expect("the metadata still parses")
    }

    // --- 規格「值讀回逐字相同」 ---

    #[test]
    fn values_read_back_verbatim_and_the_metadata_still_parses() {
        for value in [
            "123",
            "true",
            "null",
            "a: b # c",
            "say \"hi\" to C:\\dir\\",
            "一段中文",
            "",
            "~",
            "1.10",
            " padded ",
            "- item",
            "&anchor *alias !tag",
            "line\u{2028}separator",
        ] {
            let store = TestStore::with_meta("demo", META);
            set(&store, &[("v", value)]).unwrap_or_else(|e| panic!("{value:?}: {e}"));
            let meta = stored(&store);
            assert_eq!(meta.fields, map(&[("v", value)]), "round trip for {value:?}");
            // 其餘欄位不變。
            assert_eq!(meta.schema.as_deref(), Some("spec-driven"));
            assert_eq!(meta.created.as_deref(), Some("2026-11-01"));
            assert!(store.meta("demo").starts_with(META), "the other lines stay byte for byte");
        }
    }

    // --- 規格「鍵不合規則」 ---

    #[test]
    fn keys_outside_the_rule_are_refused_and_the_document_stays_byte_for_byte() {
        let long = "k".repeat(65);
        for key in ["Ticket", "1st", "my key", long.as_str(), "", "a:b", "a#b", "-a", "鍵"] {
            let store = TestStore::with_meta("demo", META);
            assert!(set(&store, &[("ok", "1"), (key, "x")]).is_err(), "{key:?} must be refused");
            assert_eq!(store.meta("demo"), META, "{key:?}: nothing written");
        }
        // 64 個字元剛好合規則。
        let store = TestStore::with_meta("demo", META);
        let edge = format!("a{}", "9_.-".repeat(15) + "zzz");
        assert_eq!(edge.len(), 64);
        set(&store, &[(edge.as_str(), "x")]).unwrap();
        assert_eq!(stored(&store).fields, map(&[(edge.as_str(), "x")]));
    }

    // --- 規格「值含換行」與值的其他規則 ---

    #[test]
    fn values_with_a_line_break_a_control_character_or_over_1024_bytes_are_refused() {
        let too_long = "x".repeat(1025);
        // 1023 個 ASCII 加一個三位元組的字 = 1026 位元組：以位元組計，不是字元數。
        let too_long_utf8 = format!("{}中", "x".repeat(1023));
        for value in ["a\nb", "a\r", "a\tb", "a\u{0}b", "a\u{7f}", "a\u{85}b", too_long.as_str(), too_long_utf8.as_str()] {
            let store = TestStore::with_meta("demo", META);
            assert!(set(&store, &[("v", value)]).is_err(), "{:?} must be refused", value.escape_debug().to_string());
            assert_eq!(store.meta("demo"), META);
        }
        let store = TestStore::with_meta("demo", META);
        let edge = "x".repeat(1024);
        set(&store, &[("v", edge.as_str())]).unwrap();
        assert_eq!(stored(&store).fields, map(&[("v", edge.as_str())]));
    }

    #[test]
    fn a_refusal_names_the_key_and_never_the_value() {
        let store = TestStore::with_meta("demo", META);
        let err = set(&store, &[("token", "s3cret\nvalue")]).unwrap_err().to_string();
        assert!(err.contains("token"), "{err}");
        assert!(!err.contains("s3cret"), "{err}");
    }

    // --- 規格「超過 32 個鍵」 ---

    #[test]
    fn the_33rd_key_is_refused_and_the_32_stay() {
        let store = TestStore::with_meta("demo", META);
        let keys: Vec<String> = (0..32).map(|i| format!("k{i:02}")).collect();
        let full: Vec<(&str, &str)> = keys.iter().map(|k| (k.as_str(), "v")).collect();
        set(&store, &full).unwrap();
        let before = store.meta("demo");
        assert_eq!(stored(&store).fields.len(), 32);

        assert!(set(&store, &[("one-more", "v")]).is_err());
        assert_eq!(store.meta("demo"), before);
        // 改既有的鍵不增加數量，照常可寫。
        set(&store, &[("k00", "changed")]).unwrap();
        assert_eq!(stored(&store).fields["k00"], "changed");
    }

    // --- D2：區塊的樣子 ---

    #[test]
    fn the_block_lists_keys_in_byte_order_and_always_quotes_the_value() {
        let store = TestStore::with_meta("demo", META);
        set(&store, &[("ticket", "PROJ-123"), ("source", "portal"), ("a_b", "true"), ("a1", "1"), ("a.b", ""), ("a-b", "x")])
            .unwrap();
        assert_eq!(
            store.meta("demo"),
            format!(
                "{META}fields:\n  a-b: \"x\"\n  a.b: \"\"\n  a1: \"1\"\n  a_b: \"true\"\n  source: \"portal\"\n  ticket: \"PROJ-123\"\n"
            )
        );
    }

    #[test]
    fn a_second_edit_rewrites_the_one_block() {
        let store = TestStore::with_meta("demo", META);
        set(&store, &[("ticket", "PROJ-1")]).unwrap();
        set(&store, &[("source", "portal")]).unwrap();
        assert_eq!(store.meta("demo"), format!("{META}fields:\n  source: \"portal\"\n  ticket: \"PROJ-1\"\n"));
    }

    // --- 規格「移除最後一個鍵」 ---

    #[test]
    fn removing_the_last_key_removes_the_block() {
        let store = TestStore::with_meta("demo", META);
        set(&store, &[("ticket", "PROJ-1")]).unwrap();
        let edited = edit_change(&store, "demo", &Fields::new(), &["ticket".to_string()]).unwrap().unwrap();
        assert!(edited.fields.is_empty());
        assert_eq!(edited.changed, vec!["ticket"]);
        assert_eq!(store.meta("demo"), META);
    }

    // --- 規格「手改過的區塊仍可讀」 ---

    #[test]
    fn a_hand_edited_block_reads_leniently_and_the_change_is_still_listed() {
        let meta = format!(
            "{META}fields:\n  count: 123\n  ratio: 1.10\n  flag: True\n  nested:\n    a: 1\n  list:\n    - x\n  flow: [1, 2]\n  ok: \"yes\"  # 註解\n  Odd Key: plain text\n"
        );
        let store = TestStore::with_meta("demo", &meta);
        let listed = store.list_changes();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].meta_error, None, "the document still reads");
        assert_eq!(
            listed[0].meta.fields,
            map(&[("count", "123"), ("ratio", "1.10"), ("flag", "True"), ("ok", "yes"), ("Odd Key", "plain text")])
        );
    }

    #[test]
    fn a_fields_key_that_is_not_a_mapping_reads_as_no_fields() {
        for meta in [format!("{META}fields: 3\n"), format!("{META}fields:\n  - a\n"), format!("{META}fields:\n")] {
            let parsed = ChangeMeta::from_text(Some(&meta)).expect("the document still reads");
            assert!(parsed.fields.is_empty(), "{meta:?}");
        }
    }

    // --- 跨平台：CRLF 的 metadata ---

    #[test]
    fn a_crlf_document_keeps_crlf_on_every_line_after_a_write() {
        let store = TestStore::with_meta("demo", "schema: spec-driven\r\ncreated: 2026-11-01\r\n");
        set(&store, &[("ticket", "PROJ-1"), ("source", "portal")]).unwrap();
        let text = store.meta("demo");
        assert_eq!(
            text,
            "schema: spec-driven\r\ncreated: 2026-11-01\r\nfields:\r\n  source: \"portal\"\r\n  ticket: \"PROJ-1\"\r\n"
        );
        assert_eq!(stored(&store).fields, map(&[("source", "portal"), ("ticket", "PROJ-1")]));
    }

    // --- 討論記錄開頭區塊的 fields ---

    const RECORD: &str = "---\ntopic: Board search\nslug: board-search\nstatus: open\ncreated: 2026-11-01\n---\n\n\
                          # Discussion: Board search\n\n## Rounds\n\n### Round 1 — explore (2026-11-01)\n\n\
                          fields:\n  owner: \"not a field\"\n\n## Conclusion\n";

    #[test]
    fn discussion_fields_round_trip_inside_the_frontmatter_and_the_rounds_stay() {
        let store = TestStore::with_live_discussion("board-search", RECORD);
        let (head, body) = RECORD.split_once("---\n\n").unwrap();
        for value in ["alice", "123", "true", "a: b # c", "say \"hi\" \\", "---", "中文"] {
            let edited = edit_discussion(&store, "board-search", &map(&[("owner", value)]), &[]).unwrap().unwrap();
            assert_eq!(edited.fields, map(&[("owner", value)]));
            let text = store.discussion("board-search");
            assert_eq!(DiscussionHead::parse(&text).fields, map(&[("owner", value)]), "round trip for {value:?}");
            assert!(text.starts_with(head), "the other frontmatter lines stay");
            assert!(text.ends_with(&format!("---\n\n{body}")), "the body stays byte for byte: {text}");
        }
        // 其餘 frontmatter 欄位照常讀得到。
        let parsed = DiscussionHead::parse(&store.discussion("board-search"));
        assert_eq!(parsed.slug.as_deref(), Some("board-search"));
        assert_eq!(parsed.topic.as_deref(), Some("Board search"));

        edit_discussion(&store, "board-search", &Fields::new(), &["owner".to_string()]).unwrap();
        assert_eq!(store.discussion("board-search"), RECORD, "removing the last key restores the record");
    }

    #[test]
    fn a_discussion_refusal_leaves_the_record_byte_for_byte() {
        let store = TestStore::with_live_discussion("board-search", RECORD);
        assert!(edit_discussion(&store, "board-search", &map(&[("Owner", "alice")]), &[]).is_err());
        assert!(edit_discussion(&store, "board-search", &map(&[("owner", "a\nb")]), &[]).is_err());
        assert_eq!(store.discussion("board-search"), RECORD);
    }

    #[test]
    fn a_record_that_cannot_take_the_block_is_refused_and_stays() {
        // 沒有 frontmatter，或 frontmatter 沒有收尾：沒有地方放區塊。這是記錄的狀態擋下來的，
        // 不是程式壞了，所以是 Refusal（命令層回 refused）。
        for record in ["# Discussion: no frontmatter\n\n## Rounds\n", "---\ntopic: T\nslug: t\n\n## Rounds\n"] {
            let store = TestStore::with_live_discussion("t", record);
            let err = edit_discussion(&store, "t", &map(&[("owner", "alice")]), &[]).unwrap_err();
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{err}");
            assert_eq!(store.discussion("t"), record);
            // 沒有欄位可移除：不是錯誤，也不寫入。
            let edited = edit_discussion(&store, "t", &Fields::new(), &["owner".to_string()]).unwrap().unwrap();
            assert!(edited.changed.is_empty());
        }
    }

    // --- 稽核（sharp edges）：值與鍵都碰不到 `fields` 區塊以外的地方 ---

    /// 會和 YAML 或 frontmatter 語法相撞的值。
    const HOSTILE: &[&str] = &[
        "x\nschema: evil",
        "x\r\nstatus: promoted",
        "---",
        "--- ",
        "...",
        "\"",
        "\\",
        "ends with a backslash\\",
        "\\n is two characters",
        "\"\nschema: evil\n\"",
        ": ",
        "a: b",
        " #",
        "# c",
        "&anchor",
        "*alias",
        "!tag",
        "!!binary x",
        "|",
        ">",
        "%YAML 1.2",
        "@x",
        "`x`",
        "'",
        "{a: 1}",
        "[1, 2]",
        "- x",
        "? x",
        "fields:",
        "  indented: 1",
        "schema: evil",
        "\u{FEFF}bom",
        "\u{2028}line separator",
        "\u{2029}paragraph separator",
        "\u{85}next line",
        "\ttab",
        "\u{0}nul",
        "\u{FFFE}",
    ];

    #[test]
    fn no_value_changes_anything_outside_the_block_of_a_change() {
        const BASE: &str = "schema: spec-driven\ncreated: 2026-11-01\nreviewed_scope:\n  - path: src/a.rs\n    hash: aa\nboard_rank: n\n";
        let before = ChangeMeta::from_text(Some(BASE)).unwrap();
        for value in HOSTILE {
            let store = TestStore::with_meta("demo", BASE);
            if set(&store, &[("v", value)]).is_err() {
                assert_eq!(store.meta("demo"), BASE, "{value:?}: a refusal writes nothing");
                continue;
            }
            let text = store.meta("demo");
            let block = text.strip_prefix(BASE).unwrap_or_else(|| panic!("{value:?}: the lines before the block stay: {text}"));
            let lines: Vec<&str> = block.split('\n').collect();
            assert_eq!(lines.len(), 3, "{value:?}: the block is its key line and one entry line: {block:?}");
            assert_eq!((lines[0], lines[2]), ("fields:", ""), "{value:?}");
            assert!(lines[1].starts_with("  v: \"") && lines[1].ends_with('"'), "{value:?}: {block:?}");
            let after = ChangeMeta::from_text(Some(&text)).unwrap_or_else(|e| panic!("{value:?}: {e}"));
            assert_eq!(after.fields, map(&[("v", value)]), "{value:?}");
            assert_eq!((&after.schema, &after.created, &after.board_rank), (&before.schema, &before.created, &before.board_rank));
            assert_eq!(after.reviewed_scope, before.reviewed_scope, "{value:?}");
        }
    }

    #[test]
    fn no_value_ends_the_frontmatter_or_reaches_the_body_of_a_discussion() {
        let (head, body) = RECORD.split_once("---\n\n").unwrap();
        for value in HOSTILE {
            let store = TestStore::with_live_discussion("board-search", RECORD);
            if edit_discussion(&store, "board-search", &map(&[("v", value)]), &[]).is_err() {
                assert_eq!(store.discussion("board-search"), RECORD, "{value:?}: a refusal writes nothing");
                continue;
            }
            let text = store.discussion("board-search");
            let block = text
                .strip_prefix(head)
                .and_then(|rest| rest.strip_suffix(&format!("---\n\n{body}")))
                .unwrap_or_else(|| panic!("{value:?}: the frontmatter lines and the body stay: {text}"));
            assert_eq!(block.matches('\n').count(), 2, "{value:?}: two lines: {block:?}");
            let parsed = DiscussionHead::parse(&text);
            assert_eq!(parsed.fields, map(&[("v", value)]), "{value:?}");
            assert_eq!(parsed.slug.as_deref(), Some("board-search"));
            assert_eq!(parsed.status.as_str(), "open", "{value:?}");
        }
    }

    #[test]
    fn a_key_that_yaml_reads_as_another_type_is_still_a_key() {
        // 鍵的規則收這些字；YAML 會把沒加引號的 null、true、false 讀成別的型別。
        for key in ["null", "true", "false", "yes", "no", "on", "off", "y", "n", "nan", "inf", "e1", "o17", "x1f"] {
            let store = TestStore::with_meta("demo", META);
            set(&store, &[(key, "v"), ("other", "w")]).unwrap_or_else(|e| panic!("{key}: {e}"));
            assert_eq!(stored(&store).fields, map(&[(key, "v"), ("other", "w")]), "{key}");
            let store = TestStore::with_live_discussion("board-search", RECORD);
            edit_discussion(&store, "board-search", &map(&[(key, "v")]), &[]).unwrap_or_else(|e| panic!("{key}: {e}"));
            assert_eq!(DiscussionHead::parse(&store.discussion("board-search")).fields, map(&[(key, "v")]), "{key}");
        }
    }

    #[test]
    fn a_key_cannot_carry_a_character_yaml_gives_a_meaning() {
        for c in [':', '#', '"', '\'', '\n', '\r', '\t', ' ', '[', ']', '{', '}', '&', '*', '!', '|', '>', '%', '@', '`', '?', ',', '\\', '/', '=', '~', 'A', 'é', '中'] {
            for key in [format!("a{c}b"), format!("a{c}"), format!("{c}a")] {
                let store = TestStore::with_meta("demo", META);
                assert!(set(&store, &[(key.as_str(), "v")]).is_err(), "{key:?} must be refused");
                assert_eq!(store.meta("demo"), META);
            }
        }
        // 開頭只能是小寫字母：數字與三個符號開頭的都不收。
        for key in ["1a", "-a", ".a", "_a"] {
            let store = TestStore::with_meta("demo", META);
            assert!(set(&store, &[(key, "v")]).is_err(), "{key:?} must be refused");
        }
    }

    #[test]
    fn a_hand_written_fields_line_the_edit_cannot_see_is_refused_and_stays() {
        // 文字手術只認第 0 欄的 `fields:`；手寫成別的樣子時，多寫一個區塊會讓文件壞掉。
        for meta in [format!("{META}fields : {{a: \"1\"}}\n"), format!("{META}\"fields\":\n  a: \"1\"\n")] {
            let store = TestStore::with_meta("demo", &meta);
            let err = set(&store, &[("b", "2")]).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{err}");
            assert_eq!(store.meta("demo"), meta);
        }
    }

    #[test]
    fn an_edit_never_drops_what_a_hand_edit_put_in_the_block() {
        // 讀取寬鬆地略過這些內容；寫入會整個重寫區塊，所以有東西會被弄丟時拒絕，文件不變。
        for block in [
            "fields:\n  ok: \"1\"\n  nested:\n    a: 1\n",
            "fields:\n  list:\n    - x\n",
            "fields:\n  a: \"1\"\n  a: \"2\"\n",
            "fields: 3\n",
            "fields: [a, b]\n",
            "fields:\n  broken: \"unterminated\n",
        ] {
            let meta = format!("{META}{block}");
            let store = TestStore::with_meta("demo", &meta);
            for (set_pairs, unset) in [(&[("b", "2")][..], &[][..]), (&[][..], &["ok".to_string(), "a".to_string()][..])] {
                match edit_change(&store, "demo", &map(set_pairs), unset) {
                    Err(err) => assert!(
                        err.downcast_ref::<crate::command::Refusal>().is_some()
                            || err.downcast_ref::<crate::model::MetaError>().is_some(),
                        "{block:?}: {err}"
                    ),
                    // 沒有變動的編輯不寫入，也就沒有東西會丟。
                    Ok(edited) => assert!(edited.unwrap().changed.is_empty(), "{block:?}"),
                }
                assert_eq!(store.meta("demo"), meta, "{block:?}: the document stays");
            }
        }
    }

    #[test]
    fn an_edit_keeps_the_scalars_a_hand_edit_wrote_without_quotes() {
        let meta = format!("{META}fields:\n  count: 123\n  ratio: 1.10\n  flag: True\n  empty:\n");
        let store = TestStore::with_meta("demo", &meta);
        set(&store, &[("ticket", "PROJ-1")]).unwrap();
        assert_eq!(
            store.meta("demo"),
            format!("{META}fields:\n  count: \"123\"\n  empty: \"\"\n  flag: \"True\"\n  ratio: \"1.10\"\n  ticket: \"PROJ-1\"\n")
        );
    }

    // --- 品質關卡第 1 輪的發現 ---

    #[test]
    fn a_block_with_comments_reads_whole_and_an_edit_that_would_leave_lines_behind_is_refused() {
        // `fields:` 那一行後面有註解或錨點、區塊中間有第 0 欄的註解：YAML 看得到整個區塊，
        // 文字手術卻只會刪掉一部分，剩下的行變成孤兒。讀要讀到全部，寫要拒絕。
        for block in [
            "fields: # 手寫的註解\n  a: \"1\"\n  b: \"2\"\n",
            "fields:\n  a: \"1\"\n# 中間的註解\n  b: \"2\"\n",
            "fields: &anchor\n  a: \"1\"\n  b: \"2\"\n",
        ] {
            let both = map(&[("a", "1"), ("b", "2")]);
            let meta = format!("{META}{block}board_rank: n\n");
            let store = TestStore::with_meta("demo", &meta);
            assert_eq!(stored(&store).fields, both, "{block:?}");
            let err = set(&store, &[("c", "3")]).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{block:?}: {err}");
            assert_eq!(store.meta("demo"), meta, "{block:?}");

            let record = RECORD.replacen("created: 2026-11-01\n", &format!("created: 2026-11-01\n{block}"), 1);
            let store = TestStore::with_live_discussion("board-search", &record);
            assert_eq!(DiscussionHead::parse(&record).fields, both, "{block:?}");
            let err = edit_discussion(&store, "board-search", &map(&[("c", "3")]), &[]).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{block:?}: {err}");
            assert_eq!(store.discussion("board-search"), record, "{block:?}");
        }
    }

    #[test]
    fn a_comment_after_the_block_does_not_stop_an_edit_and_stays() {
        let meta = format!("{META}fields:\n  a: \"1\"\n# about the rank\nboard_rank: n\n");
        let store = TestStore::with_meta("demo", &meta);
        set(&store, &[("b", "2")]).unwrap();
        assert_eq!(
            store.meta("demo"),
            format!("{META}# about the rank\nboard_rank: n\nfields:\n  a: \"1\"\n  b: \"2\"\n")
        );
    }

    #[test]
    fn a_scalar_whose_text_is_not_on_its_key_line_reads_but_takes_no_rewrite() {
        // 數字、布林與 null 要照原文讀。原文不在「鍵: 值」那一行時（值在下一行、flow 寫法、
        // 鍵加了引號）拿不到原文：讀取給 YAML 看到的值，寫入則拒絕，原文不會被悄悄改掉。
        for (block, read) in [
            ("fields:\n  a:\n    1.10\n", ("a", "1.1")),
            ("fields: {a: 1.10}\n", ("a", "1.1")),
            ("fields:\n  \"a\": 1.10\n", ("a", "1.1")),
            ("fields:\n  \"a\": True\n", ("a", "true")),
            ("fields:\n  \"a\": ~\n", ("a", "")),
            ("fields:\n  a:\n    0x1F\n", ("a", "31")),
            ("fields:\n  a:\n    ~\n", ("a", "")),
            ("fields:\n  a:\n    null\n", ("a", "")),
            ("fields:\n  a:\n    false\n", ("a", "false")),
            // 鍵和值之間夾一行註解，縮排深淺都一樣。
            ("fields:\n  a:\n  # note\n    1.10\n", ("a", "1.1")),
            ("fields:\n  a:\n  # note\n    ~\n", ("a", "")),
            ("fields:\n  a:\n      # note\n    True\n", ("a", "true")),
        ] {
            let meta = format!("{META}{block}");
            let store = TestStore::with_meta("demo", &meta);
            assert_eq!(stored(&store).fields, map(&[read]), "{block:?}");
            let err = set(&store, &[("c", "3")]).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{block:?}: {err}");
            assert_eq!(store.meta("demo"), meta, "{block:?}");
        }
    }

    #[test]
    fn a_key_with_nothing_after_it_is_the_empty_text_and_takes_an_edit() {
        // `a:` 底下沒有更深的行：它就是空的值，不是「值在下一行」。
        let store = TestStore::with_meta("demo", &format!("{META}fields:\n  a:\n  b: \"x\"\n  z:\n"));
        assert_eq!(stored(&store).fields, map(&[("a", ""), ("b", "x"), ("z", "")]));
        set(&store, &[("c", "3")]).unwrap();
        assert_eq!(store.meta("demo"), format!("{META}fields:\n  a: \"\"\n  b: \"x\"\n  c: \"3\"\n  z: \"\"\n"));
    }

    #[test]
    fn a_block_with_any_line_deeper_than_its_entries_takes_no_rewrite() {
        // 改寫只認「一行一個 鍵: 值」的區塊。有任何一行縮排比項目深（多行的字串、區塊純量、
        // 值在下一行），不論型別都拒絕：不用去猜那一行屬於誰。讀取照常。
        for (block, read) in [
            ("fields:\n  a: \"long\n    continued\"\n", ("a", "long continued")),
            ("fields:\n  a: |\n    text\n", ("a", "text\n")),
            ("fields:\n  a:\n    plain text below\n", ("a", "plain text below")),
            ("fields:\n  a: one\n   two\n", ("a", "one two")),
        ] {
            let meta = format!("{META}{block}");
            let store = TestStore::with_meta("demo", &meta);
            assert_eq!(stored(&store).fields, map(&[read]), "{block:?}");
            let err = set(&store, &[("c", "3")]).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{block:?}: {err}");
            assert_eq!(store.meta("demo"), meta, "{block:?}");
        }
    }

    #[test]
    fn a_line_that_alone_is_not_one_entry_of_the_block_takes_no_rewrite() {
        // 縮排一樣深也不夠：每一行單獨讀，都要是區塊的一個項目（同一個鍵、同一個值）。
        // 跨行的 flow 寫法、以 `#` 開頭的字串續行都不是。讀取給 YAML 看到的值，不帶括號或逗號。
        for (block, read) in [
            ("fields:\n  {a: \"x\",\n  b: 2.10}\n", &[("a", "x"), ("b", "2.1")][..]),
            ("fields:\n  {b: \"x\",\n  a: 1.10,\n  c: \"y\"}\n", &[("a", "1.1"), ("b", "x"), ("c", "y")][..]),
            ("fields:\n  {a: \"x\",\n  b: \"y\"}\n", &[("a", "x"), ("b", "y")][..]),
            ("fields:\n  a: \"one\n    # two\"\n", &[("a", "one # two")][..]),
            ("fields:\n  a: |\n    # only\n", &[("a", "# only\n")][..]),
        ] {
            let meta = format!("{META}{block}");
            let store = TestStore::with_meta("demo", &meta);
            assert_eq!(stored(&store).fields, map(read), "{block:?}");
            let err = set(&store, &[("z", "9")]).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{block:?}: {err}");
            assert_eq!(store.meta("demo"), meta, "{block:?}");
        }
    }

    #[test]
    fn a_comment_after_a_tab_is_not_part_of_the_value() {
        // 註解前面可以是空白或 tab；數字、布林與 null 的原文只有一個字。
        let store =
            TestStore::with_meta("demo", &format!("{META}fields:\n  a: 1.10\t# c\n  b: True \t# d\n  n: ~\t# e\n"));
        assert_eq!(stored(&store).fields, map(&[("a", "1.10"), ("b", "True"), ("n", "~")]));
        set(&store, &[("c", "3")]).unwrap();
        assert_eq!(store.meta("demo"), format!("{META}fields:\n  a: \"1.10\"\n  b: \"True\"\n  c: \"3\"\n  n: \"~\"\n"));
    }

    #[test]
    fn comment_lines_do_not_count_as_deeper_lines() {
        // 註解不是內容：只有註解比項目深的區塊照常可改（註解在重寫後不保留）。
        let store = TestStore::with_meta("demo", &format!("{META}fields:\n  # 上面的註解\n  a:\n      # 更深的註解\n  b: 1.10\n"));
        assert_eq!(stored(&store).fields, map(&[("a", ""), ("b", "1.10")]));
        set(&store, &[("c", "3")]).unwrap();
        assert_eq!(store.meta("demo"), format!("{META}fields:\n  a: \"\"\n  b: \"1.10\"\n  c: \"3\"\n"));
    }

    #[test]
    fn a_flow_style_block_of_strings_takes_an_edit() {
        // 字串的值由 YAML 給出，和原文相同，所以這種寫法照常可改。
        let store = TestStore::with_meta("demo", &format!("{META}fields: {{a: \"1.10\", b: x}}\n"));
        set(&store, &[("c", "3")]).unwrap();
        assert_eq!(stored(&store).fields, map(&[("a", "1.10"), ("b", "x"), ("c", "3")]));
        assert_eq!(store.meta("demo"), format!("{META}fields:\n  a: \"1.10\"\n  b: \"x\"\n  c: \"3\"\n"));
    }

    #[test]
    fn a_document_with_two_fields_blocks_is_refused_and_stays() {
        // 讀只認第一個，刪卻會兩個都刪：第二個區塊的內容會無聲消失。
        let meta = format!("{META}fields:\n  a: \"1\"\nboard_rank: n\nfields:\n  b: \"2\"\n");
        let store = TestStore::with_meta("demo", &meta);
        for (pairs, unset) in [(&[("c", "3")][..], &[][..]), (&[][..], &["a".to_string()][..])] {
            let err = edit_change(&store, "demo", &map(pairs), unset).expect_err("refused");
            assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{err}");
            assert_eq!(store.meta("demo"), meta);
        }
        let record = RECORD.replacen("status: open\n", "fields:\n  a: \"1\"\nstatus: open\nfields:\n  b: \"2\"\n", 1);
        let store = TestStore::with_live_discussion("board-search", &record);
        let err = edit_discussion(&store, "board-search", &map(&[("c", "3")]), &[]).expect_err("refused");
        assert!(err.downcast_ref::<crate::command::Refusal>().is_some(), "{err}");
        assert_eq!(store.discussion("board-search"), record);
    }

    #[test]
    fn a_block_over_the_limit_takes_no_write_until_one_edit_brings_it_back_within_it() {
        // 規格：一份文件的鍵數不超過 32，不符就拒絕整次寫入。手改成 40 個鍵的區塊也一樣。
        let block: String = (0..40).map(|i| format!("  k{i:02}: \"v\"\n")).collect();
        let meta = format!("{META}fields:\n{block}");
        let store = TestStore::with_meta("demo", &meta);
        assert_eq!(stored(&store).fields.len(), 40, "the read is lenient");
        let unset = |keys: std::ops::Range<usize>| -> Vec<String> { keys.map(|i| format!("k{i:02}")).collect() };
        for (pairs, gone) in [(&[("k00", "changed")][..], unset(0..0)), (&[][..], unset(0..1)), (&[("new", "v")][..], unset(0..1))] {
            let err = edit_change(&store, "demo", &map(pairs), &gone).expect_err("still over the limit");
            assert!(err.downcast_ref::<crate::command::InvalidContent>().is_some(), "{err}");
            assert_eq!(store.meta("demo"), meta);
        }
        edit_change(&store, "demo", &Fields::new(), &unset(0..8)).unwrap();
        assert_eq!(stored(&store).fields.len(), 32);
    }

    #[test]
    fn a_record_without_fields_reads_as_no_fields() {
        // 內文裡長得像區塊的文字不是欄位。
        assert!(DiscussionHead::parse(RECORD).fields.is_empty());
    }
}
