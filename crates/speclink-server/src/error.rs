//! The wire error envelope and the single mapping point (design 決策六).
//!
//! Every failure leaves a handler as an [`ApiError`] — an HTTP status plus a
//! reason from the protocol's eight-value closed registry plus the message.
//! The store's six failure classes, the Engine command layer's five codes, and
//! binding refusals all map to this type here and nowhere else; the three
//! vocabularies never merge and the registry is never widened.

use crate::identity::IdentityError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use speclink_core::command::{CommandError, ErrorCode};
use speclink_host::bridge::BridgeError;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_store::StoreError;

/// A protocol error ready to become the `{ status, reason, message }` triple.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub reason: ErrorReason,
    pub message: String,
}

impl ApiError {
    fn new(status: StatusCode, reason: ErrorReason, message: impl Into<String>) -> ApiError {
        ApiError {
            status,
            reason,
            message: message.into(),
        }
    }

    /// 401 — the caller could not be authenticated for this operation.
    pub fn permission_denied(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            ErrorReason::PermissionDenied,
            message,
        )
    }

    /// 403 — the authenticated caller is not allowed to act in this scope
    /// (store-layer permission, distinct from the 401 auth failure).
    pub fn forbidden(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorReason::PermissionDenied,
            message,
        )
    }

    /// 404 — the addressed subject does not exist (or is not visible).
    pub fn not_found(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::NOT_FOUND, ErrorReason::NotFound, message)
    }

    /// 409 — a precondition refused the request (ambiguous binding,
    /// incompatible version, guard not met).
    pub fn refused(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::CONFLICT, ErrorReason::Refused, message)
    }

    /// 400 — the arguments are invalid or ambiguous.
    pub fn invalid_argument(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorReason::InvalidArgument,
            message,
        )
    }

    /// 422 — a config document exists but cannot be parsed.
    pub fn invalid_config(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorReason::InvalidConfig,
            message,
        )
    }

    /// 409 — a CAS precondition failed; the message carries expected/actual.
    pub fn revision_conflict(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::CONFLICT, ErrorReason::RevisionConflict, message)
    }

    /// 503 — the store backend is temporarily unable to serve.
    pub fn unavailable(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorReason::Unavailable,
            message,
        )
    }

    /// 500 — any other failure.
    pub fn internal(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorReason::Internal,
            message,
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            status: self.status.as_u16(),
            reason: self.reason,
            message: self.message,
        };
        (self.status, Json(body)).into_response()
    }
}

// --- the single mapping point (design 決策六) ---

/// The Engine command layer's five codes → wire reason + status. The message
/// is the Engine's frozen text, relayed verbatim so the typed client's message
/// mapping stays byte-identical to fs mode.
impl From<CommandError> for ApiError {
    fn from(e: CommandError) -> ApiError {
        let message = e.message;
        match e.code {
            ErrorCode::InvalidArgv => ApiError::invalid_argument(message),
            ErrorCode::NotFound => ApiError::not_found(message),
            ErrorCode::InvalidConfig => ApiError::invalid_config(message),
            ErrorCode::Refused => ApiError::refused(message),
            ErrorCode::Error => ApiError::internal(message),
        }
    }
}

/// The store's six failure classes → wire reason + status. A `revision_conflict`
/// carries its expected/actual detail in the message (the envelope has no detail
/// fields); a store-layer permission denial is 403, distinct from the 401 the
/// auth layer returns.
impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> ApiError {
        match e {
            StoreError::NotFound => {
                ApiError::not_found("the addressed scope or document does not exist")
            }
            StoreError::PermissionDenied => {
                ApiError::forbidden("not allowed to perform this operation in this scope")
            }
            StoreError::RevisionConflict {
                doc,
                expected,
                actual,
            } => ApiError::revision_conflict(format!(
                "{:?}: expected {:?}, actual {:?}",
                doc.doc, expected, actual
            )),
            StoreError::Unavailable => {
                ApiError::unavailable("the store backend is temporarily unavailable")
            }
            StoreError::Corrupt { reason } => ApiError::internal(format!("corrupt: {reason}")),
            StoreError::Backend { source } => ApiError::internal(format!("backend: {source}")),
        }
    }
}

/// An identity-store failure → wire reason + status, for the admin API. A guard
/// rejection (duplicate key, refused last-admin) is 409; an unknown subject is
/// 404; an open/backend failure is 500. Auth-layer identity errors keep their
/// own inline mapping (a uniform 401/403), so this covers the admin actions.
impl From<IdentityError> for ApiError {
    fn from(e: IdentityError) -> ApiError {
        match e {
            IdentityError::Duplicate(m) => ApiError::refused(m),
            IdentityError::Refused(m) => ApiError::refused(m),
            IdentityError::NotFound(m) => ApiError::not_found(m),
            IdentityError::InvalidInvitation => ApiError::refused("the invitation is invalid"),
            IdentityError::Open(m) | IdentityError::Backend(m) => {
                ApiError::internal(format!("identity store error: {m}"))
            }
        }
    }
}

/// A bridge failure dispatches to whichever vocabulary produced it — the two
/// layers never merge.
impl From<BridgeError> for ApiError {
    fn from(e: BridgeError) -> ApiError {
        match e {
            BridgeError::Command(c) => ApiError::from(c),
            BridgeError::Store(s) => ApiError::from(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store::{DocRef, DocumentId, ExpectedRevision, ProjectId, RepoId, Revision};

    #[test]
    fn engine_five_codes_map_to_reason_and_status() {
        let cases = [
            (
                ErrorCode::InvalidArgv,
                StatusCode::BAD_REQUEST,
                ErrorReason::InvalidArgument,
            ),
            (
                ErrorCode::NotFound,
                StatusCode::NOT_FOUND,
                ErrorReason::NotFound,
            ),
            (
                ErrorCode::InvalidConfig,
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorReason::InvalidConfig,
            ),
            (
                ErrorCode::Refused,
                StatusCode::CONFLICT,
                ErrorReason::Refused,
            ),
            (
                ErrorCode::Error,
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorReason::Internal,
            ),
        ];
        for (code, status, reason) in cases {
            let api = ApiError::from(CommandError::new(code, "engine text"));
            assert_eq!(api.status, status, "status for {code:?}");
            assert_eq!(api.reason, reason, "reason for {code:?}");
            assert_eq!(
                api.message, "engine text",
                "message relayed verbatim for {code:?}"
            );
        }
    }

    #[test]
    fn store_six_classes_map_to_reason_and_status() {
        let doc = DocRef {
            project: ProjectId::new("demo"),
            repo: RepoId::new("backend"),
            doc: DocumentId::WorkflowConfig,
        };
        let cases = [
            (
                StoreError::NotFound,
                StatusCode::NOT_FOUND,
                ErrorReason::NotFound,
            ),
            (
                StoreError::PermissionDenied,
                StatusCode::FORBIDDEN,
                ErrorReason::PermissionDenied,
            ),
            (
                StoreError::RevisionConflict {
                    doc: doc.clone(),
                    expected: ExpectedRevision::At(Revision(2)),
                    actual: Some(Revision(5)),
                },
                StatusCode::CONFLICT,
                ErrorReason::RevisionConflict,
            ),
            (
                StoreError::Unavailable,
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorReason::Unavailable,
            ),
            (
                StoreError::Corrupt {
                    reason: "bad".into(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorReason::Internal,
            ),
            (
                StoreError::Backend {
                    source: "io".into(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorReason::Internal,
            ),
        ];
        for (err, status, reason) in cases {
            let is_conflict = matches!(err, StoreError::RevisionConflict { .. });
            let api = ApiError::from(err);
            assert_eq!(api.status, status, "status");
            assert_eq!(api.reason, reason, "reason");
            if is_conflict {
                assert!(
                    api.message.contains("At(Revision(2))") && api.message.contains("Revision(5)"),
                    "revision_conflict carries expected/actual: {}",
                    api.message
                );
            }
        }
    }

    #[test]
    fn bridge_error_dispatches_to_the_producing_layer() {
        let command = ApiError::from(BridgeError::Command(CommandError::new(
            ErrorCode::NotFound,
            "no change",
        )));
        assert_eq!(command.reason, ErrorReason::NotFound);

        let store = ApiError::from(BridgeError::Store(StoreError::Unavailable));
        assert_eq!(store.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(store.reason, ErrorReason::Unavailable);
    }
}
