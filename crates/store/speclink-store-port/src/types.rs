use std::fmt;

use crate::error::InvalidValue;

/// The largest version and timestamp: 2^53 − 1, the largest integer a
/// JavaScript number holds exactly.
pub const MAX_VERSION: u64 = 9_007_199_254_740_991;

/// The longest document key and key prefix, in UTF-8 bytes.
pub const MAX_KEY_BYTES: usize = 512;

/// The longest scope id, in UTF-8 bytes.
pub const MAX_SCOPE_BYTES: usize = 256;

fn non_empty(value: &'static str, text: &str) -> Result<(), InvalidValue> {
    if text.is_empty() {
        return Err(InvalidValue::new(value, "must not be empty"));
    }
    Ok(())
}

fn at_most(value: &'static str, text: &str, max_bytes: usize) -> Result<(), InvalidValue> {
    if text.len() > max_bytes {
        return Err(InvalidValue::new(
            value,
            format!("must be at most {max_bytes} bytes"),
        ));
    }
    Ok(())
}

/// U+0000 to U+001F and U+007F. Every byte of a multi-byte UTF-8 sequence
/// is 0x80 or more, so a byte scan finds exactly these characters.
fn no_control(value: &'static str, text: &str) -> Result<(), InvalidValue> {
    if text.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err(InvalidValue::new(
            value,
            "must not contain control characters",
        ));
    }
    Ok(())
}

fn no_backslash(value: &'static str, text: &str) -> Result<(), InvalidValue> {
    if text.contains('\\') {
        return Err(InvalidValue::new(value, "must not contain a backslash"));
    }
    Ok(())
}

fn within_max(value: &'static str, n: u64) -> Result<(), InvalidValue> {
    if n > MAX_VERSION {
        return Err(InvalidValue::new(
            value,
            format!("must not exceed {MAX_VERSION}"),
        ));
    }
    Ok(())
}

/// An opaque scope id: 1 to 256 bytes, no control characters. The contract
/// never reads its content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(String);

impl ScopeId {
    pub fn new(id: impl Into<String>) -> Result<Self, InvalidValue> {
        const VALUE: &str = "scope id";
        let id = id.into();
        non_empty(VALUE, &id)?;
        at_most(VALUE, &id, MAX_SCOPE_BYTES)?;
        no_control(VALUE, &id)?;
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A document key: 1 to 512 bytes of UTF-8 in `/`-separated segments. No
/// control characters, no backslash, no leading or trailing `/`, no empty
/// segment, and no segment that is exactly `.` or `..`.
///
/// Keys order by their bytes.
///
/// A key comes only from [`DocKey::new`]:
///
/// ```
/// let key = speclink_store_port::DocKey::new("config.yaml").unwrap();
/// assert_eq!(key.as_str(), "config.yaml");
/// ```
///
/// ```compile_fail
/// let key = speclink_store_port::DocKey(String::from("../config.yaml"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocKey(String);

impl DocKey {
    pub fn new(key: impl Into<String>) -> Result<Self, InvalidValue> {
        const VALUE: &str = "document key";
        let key = key.into();
        non_empty(VALUE, &key)?;
        at_most(VALUE, &key, MAX_KEY_BYTES)?;
        no_control(VALUE, &key)?;
        no_backslash(VALUE, &key)?;
        if key.starts_with('/') {
            return Err(InvalidValue::new(VALUE, "must not start with '/'"));
        }
        if key.ends_with('/') {
            return Err(InvalidValue::new(VALUE, "must not end with '/'"));
        }
        for segment in key.split('/') {
            if segment.is_empty() {
                return Err(InvalidValue::new(
                    VALUE,
                    "must not contain an empty segment",
                ));
            }
            if segment == "." || segment == ".." {
                return Err(InvalidValue::new(
                    VALUE,
                    "must not contain a '.' or '..' segment",
                ));
            }
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A key prefix: 0 to 512 bytes, no control characters, no backslash.
/// Matching is a byte-level "starts with" and knows nothing of segments:
/// `changes/add` matches both `changes/add-x/…` and `changes/addendum/…`.
/// The empty prefix matches every key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyPrefix(String);

impl KeyPrefix {
    pub fn new(prefix: impl Into<String>) -> Result<Self, InvalidValue> {
        const VALUE: &str = "key prefix";
        let prefix = prefix.into();
        at_most(VALUE, &prefix, MAX_KEY_BYTES)?;
        no_control(VALUE, &prefix)?;
        no_backslash(VALUE, &prefix)?;
        Ok(Self(prefix))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, key: &DocKey) -> bool {
        key.as_str().starts_with(&self.0)
    }
}

/// A scope version or a document version: an integer from 0 to
/// [`MAX_VERSION`]. It has no `Default` and no public field:
///
/// ```compile_fail
/// let version = speclink_store_port::Version(9_007_199_254_740_992);
/// ```
///
/// ```compile_fail
/// let version: speclink_store_port::Version = Default::default();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(u64);

impl Version {
    /// The version of a scope that was never written.
    pub const ZERO: Version = Version(0);

    pub fn new(version: u64) -> Result<Self, InvalidValue> {
        within_max("version", version)?;
        Ok(Self(version))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Milliseconds since the Unix epoch, from 0 to [`MAX_VERSION`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn new(millis: u64) -> Result<Self, InvalidValue> {
        within_max("timestamp", millis)?;
        Ok(Self(millis))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

/// Who makes a commit: a stable id and a display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    id: String,
    display_name: String,
}

impl Actor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<Self, InvalidValue> {
        let id = id.into();
        non_empty("actor id", &id)?;
        Ok(Self {
            id,
            display_name: display_name.into(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

/// What a write expects of the document before the commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
    /// The document must not exist.
    Absent,
    /// The document must exist at exactly this version.
    At(Version),
}

/// One change in a commit. Every operation states what it expects; a
/// delete always expects a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Put {
        key: DocKey,
        content: String,
        expected: Expected,
    },
    Delete {
        key: DocKey,
        expected: Version,
    },
}

impl Op {
    pub fn key(&self) -> &DocKey {
        match self {
            Op::Put { key, .. } | Op::Delete { key, .. } => key,
        }
    }
}

/// An event handed to the store in a commit. The store decides whether to
/// keep it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    name: String,
    payload: serde_json::Value,
}

impl Event {
    pub fn new(name: impl Into<String>, payload: serde_json::Value) -> Result<Self, InvalidValue> {
        let name = name.into();
        non_empty("event name", &name)?;
        Ok(Self { name, payload })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn payload(&self) -> &serde_json::Value {
        &self.payload
    }
}

/// Everything one commit changes. The constructor rejects a key that
/// appears in two operations and a commit with no operation and no event,
/// so a store never receives either. A commit with only events is valid.
///
/// A commit cannot change after it is built:
///
/// ```compile_fail
/// # use speclink_store_port::{Actor, Commit, Event, Timestamp};
/// let event = Event::new("board.viewed", serde_json::Value::Null).unwrap();
/// let actor = Actor::new("u-1", "Momo").unwrap();
/// let at = Timestamp::new(0).unwrap();
/// let mut commit = Commit::new(actor, "board", at, Vec::new(), vec![event]).unwrap();
/// commit.events.clear();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    actor: Actor,
    command: String,
    at: Timestamp,
    ops: Vec<Op>,
    events: Vec<Event>,
}

impl Commit {
    pub fn new(
        actor: Actor,
        command: impl Into<String>,
        at: Timestamp,
        ops: Vec<Op>,
        events: Vec<Event>,
    ) -> Result<Self, InvalidValue> {
        let command = command.into();
        non_empty("command", &command)?;
        if ops.is_empty() && events.is_empty() {
            return Err(InvalidValue::new(
                "commit",
                "must contain at least one operation or event",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for op in &ops {
            if !seen.insert(op.key()) {
                return Err(InvalidValue::new(
                    "commit",
                    format!("key {} appears in two operations", op.key().as_str()),
                ));
            }
        }
        Ok(Self {
            actor,
            command,
            at,
            ops,
            events,
        })
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    /// The name of the engine command that made this commit.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The commit time, given by the caller. Every written document takes
    /// it as its last-modified time.
    pub fn at(&self) -> Timestamp {
        self.at
    }

    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

/// One stored document as a read returns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub content: String,
    /// The scope version of the commit that last wrote this document.
    pub version: Version,
    pub modified_at: Timestamp,
}

/// One document in a listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListEntry {
    pub key: DocKey,
    pub version: Version,
    pub modified_at: Timestamp,
}

/// The result of a list: the scope's current version and the matching
/// documents in byte order of their keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub version: Version,
    pub entries: Vec<ListEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected_key(input: &str) -> (&'static str, String) {
        let err = DocKey::new(input).expect_err("the key must be rejected");
        (err.value(), err.rule().to_string())
    }

    #[test]
    fn doc_key_boundaries_from_the_spec() {
        // 「文件鍵的格式」的對照表，一列一個斷言。
        assert_eq!(
            DocKey::new("changes/add-x/proposal.md").unwrap().as_str(),
            "changes/add-x/proposal.md"
        );
        assert_eq!(
            DocKey::new("changes/add-x/.meta.yaml").unwrap().as_str(),
            "changes/add-x/.meta.yaml"
        );
        assert_eq!(
            rejected_key(""),
            ("document key", "must not be empty".to_string())
        );
        assert_eq!(
            rejected_key(&"a".repeat(513)),
            ("document key", "must be at most 512 bytes".to_string())
        );
        assert_eq!(
            rejected_key("/config.yaml"),
            ("document key", "must not start with '/'".to_string())
        );
        assert_eq!(
            rejected_key("changes/"),
            ("document key", "must not end with '/'".to_string())
        );
        assert_eq!(
            rejected_key("changes//proposal.md"),
            (
                "document key",
                "must not contain an empty segment".to_string()
            )
        );
        assert_eq!(
            rejected_key("changes/../config.yaml"),
            (
                "document key",
                "must not contain a '.' or '..' segment".to_string()
            )
        );
        assert_eq!(
            rejected_key("changes\\add-x"),
            ("document key", "must not contain a backslash".to_string())
        );
    }

    #[test]
    fn doc_key_rejects_control_characters_and_dot_segments() {
        for input in ["config\u{0}.yaml", "config\u{1f}.yaml", "config\u{7f}.yaml"] {
            assert_eq!(
                rejected_key(input),
                (
                    "document key",
                    "must not contain control characters".to_string()
                )
            );
        }
        assert_eq!(
            rejected_key("changes/./config.yaml"),
            (
                "document key",
                "must not contain a '.' or '..' segment".to_string()
            )
        );
        assert_eq!(
            rejected_key(".."),
            (
                "document key",
                "must not contain a '.' or '..' segment".to_string()
            )
        );
    }

    #[test]
    fn doc_key_length_counts_bytes() {
        assert!(DocKey::new("a".repeat(512)).is_ok());
        // 171 個三位元組的字是 513 個位元組，字數只有 171。
        assert_eq!(
            rejected_key(&"規".repeat(171)),
            ("document key", "must be at most 512 bytes".to_string())
        );
    }

    #[test]
    fn scope_id_is_one_to_256_bytes_without_control_characters() {
        assert_eq!(ScopeId::new("s").unwrap().as_str(), "s");
        assert!(ScopeId::new("s".repeat(256)).is_ok());
        // 範圍是不透明的字串：斜線與反斜線都允許。
        assert!(ScopeId::new("org/repo\\x").is_ok());

        let err = ScopeId::new("").unwrap_err();
        assert_eq!((err.value(), err.rule()), ("scope id", "must not be empty"));
        let err = ScopeId::new("s".repeat(257)).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("scope id", "must be at most 256 bytes")
        );
        let err = ScopeId::new("a\nb").unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("scope id", "must not contain control characters")
        );
    }

    #[test]
    fn key_prefix_allows_empty_and_matches_bytes_not_segments() {
        let all = KeyPrefix::new("").unwrap();
        assert_eq!(all.as_str(), "");
        assert!(all.matches(&DocKey::new("config.yaml").unwrap()));

        let add = KeyPrefix::new("changes/add").unwrap();
        assert!(add.matches(&DocKey::new("changes/add-x/proposal.md").unwrap()));
        assert!(add.matches(&DocKey::new("changes/addendum/tasks.md").unwrap()));
        assert!(!add.matches(&DocKey::new("specs/auth/spec.md").unwrap()));

        assert!(KeyPrefix::new("a".repeat(512)).is_ok());
        let err = KeyPrefix::new("a".repeat(513)).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("key prefix", "must be at most 512 bytes")
        );
        let err = KeyPrefix::new("changes\\").unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("key prefix", "must not contain a backslash")
        );
        let err = KeyPrefix::new("changes\t").unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("key prefix", "must not contain control characters")
        );
    }

    #[test]
    fn version_stops_at_the_largest_exact_javascript_integer() {
        assert_eq!(MAX_VERSION, 9_007_199_254_740_991);
        assert_eq!(MAX_KEY_BYTES, 512);
        assert_eq!(MAX_SCOPE_BYTES, 256);

        assert_eq!(Version::new(0).unwrap(), Version::ZERO);
        assert_eq!(Version::new(MAX_VERSION).unwrap().get(), MAX_VERSION);
        assert!(Version::new(1).unwrap() > Version::ZERO);

        // 「超出範圍的版本號被拒絕」：9007199254740992。
        let err = Version::new(9_007_199_254_740_992).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("version", "must not exceed 9007199254740991")
        );
    }

    #[test]
    fn timestamp_has_the_same_upper_bound() {
        assert_eq!(Timestamp::new(0).unwrap().get(), 0);
        assert_eq!(Timestamp::new(MAX_VERSION).unwrap().get(), MAX_VERSION);
        let err = Timestamp::new(MAX_VERSION + 1).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("timestamp", "must not exceed 9007199254740991")
        );
    }

    #[test]
    fn actor_needs_an_id() {
        let actor = Actor::new("u-1", "Momo").unwrap();
        assert_eq!((actor.id(), actor.display_name()), ("u-1", "Momo"));
        let err = Actor::new("", "Momo").unwrap_err();
        assert_eq!((err.value(), err.rule()), ("actor id", "must not be empty"));
    }

    fn key(text: &str) -> DocKey {
        DocKey::new(text).unwrap()
    }

    fn actor() -> Actor {
        Actor::new("u-1", "Momo").unwrap()
    }

    fn at() -> Timestamp {
        Timestamp::new(1_700_000_000_000).unwrap()
    }

    fn put(text: &str) -> Op {
        Op::Put {
            key: key(text),
            content: "a".into(),
            expected: Expected::Absent,
        }
    }

    #[test]
    fn commit_rejects_the_same_key_twice() {
        let ops = vec![
            put("config.yaml"),
            Op::Delete {
                key: key("config.yaml"),
                expected: Version::new(3).unwrap(),
            },
        ];
        let err = Commit::new(actor(), "archive", at(), ops, Vec::new()).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("commit", "key config.yaml appears in two operations")
        );
    }

    #[test]
    fn commit_rejects_no_operations_and_no_events() {
        let err = Commit::new(actor(), "archive", at(), Vec::new(), Vec::new()).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("commit", "must contain at least one operation or event")
        );
    }

    #[test]
    fn commit_rejects_an_empty_command() {
        let err = Commit::new(actor(), "", at(), vec![put("config.yaml")], Vec::new()).unwrap_err();
        assert_eq!((err.value(), err.rule()), ("command", "must not be empty"));
    }

    #[test]
    fn commit_with_only_events_is_valid() {
        let event = Event::new("change.archived", serde_json::json!({"name": "add-x"})).unwrap();
        let commit =
            Commit::new(actor(), "archive", at(), Vec::new(), vec![event.clone()]).unwrap();
        assert!(commit.ops().is_empty());
        assert_eq!(commit.events(), &[event]);
        assert_eq!(commit.command(), "archive");
        assert_eq!(commit.actor(), &actor());
        assert_eq!(commit.at(), at());
    }

    #[test]
    fn commit_keeps_its_operations_in_order() {
        let ops = vec![put("specs/auth/spec.md"), put("config.yaml")];
        let commit = Commit::new(actor(), "propose", at(), ops.clone(), Vec::new()).unwrap();
        assert_eq!(commit.ops(), ops.as_slice());
        assert_eq!(commit.ops()[0].key(), &key("specs/auth/spec.md"));
    }

    #[test]
    fn event_needs_a_name() {
        let event = Event::new("task.done", serde_json::json!(1)).unwrap();
        assert_eq!(
            (event.name(), event.payload()),
            ("task.done", &serde_json::json!(1))
        );
        let err = Event::new("", serde_json::Value::Null).unwrap_err();
        assert_eq!(
            (err.value(), err.rule()),
            ("event name", "must not be empty")
        );
    }

    #[test]
    fn invalid_value_names_the_value_and_the_rule() {
        let err = Version::new(MAX_VERSION + 1).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid version: must not exceed 9007199254740991"
        );
    }
}
