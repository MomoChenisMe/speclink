//! The wire error envelope and the single mapping point (design 決策六).
//!
//! Every failure leaves a handler as an [`ApiError`] — an HTTP status plus a
//! reason from the protocol's eight-value closed registry plus the message.
//! The store's six failure classes, the store port's three (the verb path),
//! the Engine command layer's five codes, and binding refusals all map to this
//! type here and nowhere else; the vocabularies never merge and the registry is
//! never widened.

use crate::identity::IdentityError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use speclink_core::command::{CommandError, ErrorCode};
use speclink_host::bridge::BridgeError;
use speclink_host::documents::DocumentsError;
use speclink_protocol::command::RevertBlockedEvidence;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_store::StoreError;
use speclink_store_port::{Expected, PortError};

/// A protocol error ready to become the `{ status, reason, message }` triple
/// (plus flattened refusal evidence when the flow attached it).
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub reason: ErrorReason,
    pub message: String,
    pub evidence: Option<RevertBlockedEvidence>,
}

impl ApiError {
    fn new(status: StatusCode, reason: ErrorReason, message: impl Into<String>) -> ApiError {
        ApiError {
            status,
            reason,
            message: message.into(),
            evidence: None,
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

    /// 413 — the request body exceeds the route's declared size cap.
    pub fn payload_too_large(message: impl Into<String>) -> ApiError {
        ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            ErrorReason::Refused,
            message,
        )
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
            evidence: self.evidence,
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
        // in-progress 移除守門的結構化證據(D4):自 CommandError 的 source
        // 下鑽 RevertBlocked,轉為 wire 形狀隨 409 封套 flatten 輸出。
        let evidence = e.source.as_ref().and_then(|s| {
            s.downcast_ref::<speclink_core::inprogress::RevertBlocked>().map(|b| {
                RevertBlockedEvidence {
                    checked_tasks: b.checked_tasks,
                    touched_files: b.touched_files.clone(),
                }
            })
        });
        // plan 的依賴成環（add-change-plan-remote D1）：引擎歸為一般錯誤（local
        // CLI 行為不動），但它是 meta 的資料缺陷、不是 server 故障——依 source
        // 型別改判 409 refused，訊息照舊逐字。
        let plan_cycle = e.source.as_ref().is_some_and(|s| {
            s.downcast_ref::<speclink_core::plan::PlanError>().is_some()
        });
        let message = e.message;
        let mut api = match e.code {
            ErrorCode::InvalidArgv => ApiError::invalid_argument(message),
            ErrorCode::NotFound => ApiError::not_found(message),
            ErrorCode::InvalidConfig => ApiError::invalid_config(message),
            ErrorCode::Refused => ApiError::refused(message),
            ErrorCode::Error if plan_cycle => ApiError::refused(message),
            ErrorCode::Error => ApiError::internal(message),
        };
        api.evidence = evidence;
        api
    }
}

/// What a client is told when the store cannot serve right now, whichever
/// store vocabulary reported it.
const STORE_UNAVAILABLE: &str = "the store backend is temporarily unavailable";

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
            StoreError::Unavailable => ApiError::unavailable(STORE_UNAVAILABLE),
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

/// The store port's three failures → wire reason + status: what a verb run
/// through the bridge reports. A conflict names the document key and carries
/// its expected and actual versions in the message (the envelope has no detail
/// fields).
impl From<PortError> for ApiError {
    fn from(e: PortError) -> ApiError {
        match e {
            PortError::Conflict {
                key,
                expected,
                actual,
            } => {
                let expected = match expected {
                    Expected::Absent => "absent".to_string(),
                    Expected::At(version) => version.to_string(),
                };
                let actual = actual.map_or_else(|| "absent".to_string(), |v| v.to_string());
                ApiError::revision_conflict(format!(
                    "{}: expected {expected}, actual {actual}",
                    key.as_str()
                ))
            }
            // The store's own text is not sent out: it can name
            // infrastructure. Every client sees the one fixed message.
            PortError::Unavailable { .. } => ApiError::unavailable(STORE_UNAVAILABLE),
            backend @ PortError::Backend { .. } => ApiError::internal(backend.to_string()),
        }
    }
}

/// A document read or write failure → the status each route has always
/// answered with: not found 404, a broken workflow config 422, an oversized
/// board order 413, a moved scope 409 `revision_conflict`, a corrupt record
/// 500, and the store's own failures as above.
impl From<DocumentsError> for ApiError {
    fn from(e: DocumentsError) -> ApiError {
        match e {
            DocumentsError::NotFound(message) => ApiError::not_found(message),
            DocumentsError::InvalidConfig(message) => ApiError::invalid_config(message),
            too_large @ DocumentsError::TooLarge { .. } => {
                ApiError::payload_too_large(too_large.to_string())
            }
            moved @ DocumentsError::ScopeMoved { .. } => {
                ApiError::revision_conflict(moved.to_string())
            }
            DocumentsError::Corrupt(message) => ApiError::internal(message),
            DocumentsError::Store(e) => ApiError::from(e),
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
    fn a_plan_dependency_cycle_is_a_refused_409_carrying_the_engine_line() {
        // add-change-plan-remote D1：引擎把成環歸為一般錯誤；映射點依 source 的
        // 型別改判 409 refused、訊息逐字——不比對字串。
        let cycle = speclink_core::plan::PlanError::Cycle(vec!["a".into(), "b".into(), "a".into()]);
        let api = ApiError::from(CommandError {
            code: ErrorCode::Error,
            message: cycle.to_string(),
            source: Some(cycle.into()),
        });
        assert_eq!(api.status, StatusCode::CONFLICT);
        assert_eq!(api.reason, ErrorReason::Refused);
        assert_eq!(api.message, "dependency cycle: a -> b -> a");

        // 其他一般錯誤照舊是 500，訊息同一句也不會被誤判。
        let other = ApiError::from(CommandError {
            code: ErrorCode::Error,
            message: "dependency cycle: a -> b -> a".into(),
            source: Some(anyhow::anyhow!("dependency cycle: a -> b -> a")),
        });
        assert_eq!(other.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(other.reason, ErrorReason::Internal);
    }

    #[test]
    fn bridge_error_dispatches_to_the_producing_layer() {
        let command = ApiError::from(BridgeError::Command(CommandError::new(
            ErrorCode::NotFound,
            "no change",
        )));
        assert_eq!(command.reason, ErrorReason::NotFound);

        let store = ApiError::from(BridgeError::Store(PortError::Unavailable {
            message: "connection lost".into(),
        }));
        assert_eq!(store.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(store.reason, ErrorReason::Unavailable);
    }

    #[test]
    fn store_port_three_failures_map_to_reason_and_status() {
        use speclink_store_port::{DocKey, Version};
        let key = || DocKey::new("changes/demo/tasks.md").unwrap();
        let v = |n| Version::new(n).unwrap();

        let conflict = ApiError::from(PortError::Conflict {
            key: key(),
            expected: Expected::At(v(3)),
            actual: Some(v(4)),
        });
        assert_eq!(conflict.status, StatusCode::CONFLICT);
        assert_eq!(conflict.reason, ErrorReason::RevisionConflict);
        assert_eq!(conflict.message, "changes/demo/tasks.md: expected 3, actual 4");

        let created_twice = ApiError::from(PortError::Conflict {
            key: key(),
            expected: Expected::Absent,
            actual: Some(v(4)),
        });
        assert_eq!(created_twice.message, "changes/demo/tasks.md: expected absent, actual 4");
        let gone = ApiError::from(PortError::Conflict {
            key: key(),
            expected: Expected::At(v(3)),
            actual: None,
        });
        assert_eq!(gone.message, "changes/demo/tasks.md: expected 3, actual absent");

        let unavailable = ApiError::from(PortError::Unavailable { message: "timeout".into() });
        assert_eq!(unavailable.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(unavailable.reason, ErrorReason::Unavailable);
        assert_eq!(unavailable.message, STORE_UNAVAILABLE);
        assert_eq!(
            ApiError::from(StoreError::Unavailable).message,
            unavailable.message,
            "one message for both store vocabularies"
        );

        let backend = ApiError::from(PortError::Backend { message: "disk full".into() });
        assert_eq!(backend.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(backend.reason, ErrorReason::Internal);
        assert_eq!(backend.message, "store backend error: disk full");
    }
}
