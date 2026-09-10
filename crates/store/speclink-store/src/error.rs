//! The closed store error set. Store-layer vocabulary only — command-layer
//! error codes (invalid_argv, …) live in the engine; the Host maps between
//! the two layers.
//!
//! Reads return `Result`: "the document does not exist" is the normal case
//! and travels as `Ok(None)`; every variant here is a failure. Implementations
//! must never swallow a failure into an empty collection or default value.

use crate::types::{DocRef, ExpectedRevision, Revision};
use std::fmt;

/// The closed set of store failures. Each variant has a stable code string
/// (see [`StoreError::code`]) that conformance and hosts assert on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The addressed scope or document tree is not visible to the caller.
    /// Also the isolation answer for cross-tenant access when the driver
    /// does not distinguish "absent" from "not yours".
    NotFound,
    /// The caller is not allowed to perform the operation in this scope.
    PermissionDenied,
    /// A CAS precondition failed: the document is not at the revision the
    /// writer expected. `actual` is `None` when the document does not exist.
    RevisionConflict {
        doc: DocRef,
        expected: ExpectedRevision,
        actual: Option<Revision>,
    },
    /// The backend is temporarily unable to serve the request.
    Unavailable,
    /// Persisted content exists but cannot be read back intact.
    Corrupt { reason: String },
    /// Any other backend failure, with a source description.
    Backend { source: String },
}

impl StoreError {
    /// Stable error code string for this variant.
    pub fn code(&self) -> &'static str {
        match self {
            StoreError::NotFound => "not_found",
            StoreError::PermissionDenied => "permission_denied",
            StoreError::RevisionConflict { .. } => "revision_conflict",
            StoreError::Unavailable => "unavailable",
            StoreError::Corrupt { .. } => "corrupt",
            StoreError::Backend { .. } => "backend",
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::RevisionConflict {
                doc,
                expected,
                actual,
            } => write!(
                f,
                "{}: {:?} expected {:?}, actual {:?}",
                self.code(),
                doc.doc,
                expected,
                actual
            ),
            StoreError::Corrupt { reason } => write!(f, "{}: {reason}", self.code()),
            StoreError::Backend { source } => write!(f, "{}: {source}", self.code()),
            _ => f.write_str(self.code()),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use crate::error::StoreError;
    use crate::types::{DocRef, DocumentId, ExpectedRevision, ProjectId, RepoId, Revision};

    fn doc_ref() -> DocRef {
        DocRef {
            project: ProjectId::new("default"),
            repo: RepoId::new("main"),
            doc: DocumentId::WorkflowConfig,
        }
    }

    #[test]
    fn six_variants_carry_stable_codes() {
        let cases: Vec<(StoreError, &str)> = vec![
            (StoreError::NotFound, "not_found"),
            (StoreError::PermissionDenied, "permission_denied"),
            (
                StoreError::RevisionConflict {
                    doc: doc_ref(),
                    expected: ExpectedRevision::Absent,
                    actual: Some(Revision(3)),
                },
                "revision_conflict",
            ),
            (StoreError::Unavailable, "unavailable"),
            (
                StoreError::Corrupt {
                    reason: "truncated frontmatter".into(),
                },
                "corrupt",
            ),
            (
                StoreError::Backend {
                    source: "io: disk full".into(),
                },
                "backend",
            ),
        ];
        for (err, code) in cases {
            assert_eq!(err.code(), code, "stable code for {err:?}");
        }
    }

    #[test]
    fn error_set_is_closed() {
        // Exhaustive match without a wildcard arm: adding a variant breaks
        // this test at compile time, which is the point of a closed set.
        let err = StoreError::NotFound;
        match err {
            StoreError::NotFound => {}
            StoreError::PermissionDenied => {}
            StoreError::RevisionConflict { .. } => {}
            StoreError::Unavailable => {}
            StoreError::Corrupt { .. } => {}
            StoreError::Backend { .. } => {}
        }
    }

    #[test]
    fn revision_conflict_carries_expected_and_actual() {
        let err = StoreError::RevisionConflict {
            doc: doc_ref(),
            expected: ExpectedRevision::At(Revision(2)),
            actual: Some(Revision(5)),
        };
        match err {
            StoreError::RevisionConflict {
                doc,
                expected,
                actual,
            } => {
                assert_eq!(doc, doc_ref());
                assert_eq!(expected, ExpectedRevision::At(Revision(2)));
                assert_eq!(actual, Some(Revision(5)));
            }
            other => panic!("expected RevisionConflict, got {other:?}"),
        }
    }

    #[test]
    fn implements_std_error_and_display() {
        let err: Box<dyn std::error::Error> = Box::new(StoreError::Corrupt {
            reason: "bad digest".into(),
        });
        let shown = err.to_string();
        assert!(shown.contains("corrupt"), "display carries the code: {shown}");
        assert!(shown.contains("bad digest"), "display carries the reason: {shown}");
    }
}
