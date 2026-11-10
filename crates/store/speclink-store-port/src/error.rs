use std::fmt;

use crate::types::{DocKey, Expected, Version};

/// A value that breaks one of the contract's format rules. Constructors
/// return it, so a store implementation never sees a malformed value.
///
/// The error names the kind of value and the rule. It never carries the
/// rejected input itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidValue {
    value: &'static str,
    rule: String,
}

impl InvalidValue {
    pub(crate) fn new(value: &'static str, rule: impl Into<String>) -> Self {
        Self {
            value,
            rule: rule.into(),
        }
    }

    /// Which kind of value was rejected, for example `document key`.
    pub fn value(&self) -> &'static str {
        self.value
    }

    /// The rule the value breaks, for example `must not start with '/'`.
    pub fn rule(&self) -> &str {
        &self.rule
    }
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {}: {}", self.value, self.rule)
    }
}

impl std::error::Error for InvalidValue {}

/// A failure a store reports. There are exactly three kinds. A document
/// that does not exist is not a failure: a read returns `None` for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    /// An operation's expectation does not match the document. `actual` is
    /// the document's current version, or `None` when it does not exist.
    /// When several operations do not match, any one of them is named.
    Conflict {
        key: DocKey,
        expected: Expected,
        actual: Option<Version>,
    },
    /// A temporary failure, such as a lost connection or a timeout. The
    /// caller can retry.
    Unavailable { message: String },
    /// Any other failure of the store.
    Backend { message: String },
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortError::Conflict {
                key,
                expected,
                actual,
            } => {
                write!(f, "conflict on {}: expected ", key.as_str())?;
                match expected {
                    Expected::Absent => write!(f, "no document")?,
                    Expected::At(version) => write!(f, "version {version}")?,
                }
                match actual {
                    Some(version) => write!(f, ", found version {version}"),
                    None => write!(f, ", found no document"),
                }
            }
            PortError::Unavailable { message } => write!(f, "store unavailable: {message}"),
            PortError::Backend { message } => write!(f, "store backend error: {message}"),
        }
    }
}

impl std::error::Error for PortError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> DocKey {
        DocKey::new("config.yaml").unwrap()
    }

    fn v(n: u64) -> Version {
        Version::new(n).unwrap()
    }

    #[test]
    fn conflict_carries_key_expected_and_actual() {
        let err = PortError::Conflict {
            key: key(),
            expected: Expected::At(v(5)),
            actual: Some(v(7)),
        };
        match &err {
            PortError::Conflict {
                key: k,
                expected,
                actual,
            } => {
                assert_eq!(k, &key());
                assert_eq!(expected, &Expected::At(v(5)));
                assert_eq!(actual, &Some(v(7)));
            }
            other => panic!("not a conflict: {other:?}"),
        }
        assert_eq!(
            err.to_string(),
            "conflict on config.yaml: expected version 5, found version 7"
        );
    }

    #[test]
    fn conflict_text_names_absent_documents() {
        let exists = PortError::Conflict {
            key: key(),
            expected: Expected::Absent,
            actual: Some(v(7)),
        };
        assert_eq!(
            exists.to_string(),
            "conflict on config.yaml: expected no document, found version 7"
        );
        let missing = PortError::Conflict {
            key: key(),
            expected: Expected::At(v(5)),
            actual: None,
        };
        assert_eq!(
            missing.to_string(),
            "conflict on config.yaml: expected version 5, found no document"
        );
    }

    #[test]
    fn unavailable_and_backend_carry_a_message() {
        let unavailable = PortError::Unavailable {
            message: "connection refused".into(),
        };
        assert_eq!(
            unavailable.to_string(),
            "store unavailable: connection refused"
        );
        let backend = PortError::Backend {
            message: "disk full".into(),
        };
        assert_eq!(backend.to_string(), "store backend error: disk full");
    }

    #[test]
    fn a_rejected_value_is_not_repeated_in_the_error() {
        let secret = "token-1234567890";
        let errors = [
            crate::DocKey::new(format!("{secret}\\x")).unwrap_err(),
            crate::DocKey::new(format!("{secret}\n")).unwrap_err(),
            crate::ScopeId::new(format!("{secret}\u{7f}")).unwrap_err(),
            crate::KeyPrefix::new(format!("{secret}\\")).unwrap_err(),
            crate::Actor::new("", secret).unwrap_err(),
        ];
        for err in errors {
            assert!(!err.to_string().contains(secret), "{err}");
            assert!(!format!("{err:?}").contains(secret), "{err:?}");
        }
    }
}
