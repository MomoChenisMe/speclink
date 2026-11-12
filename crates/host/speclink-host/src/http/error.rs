//! The wire error envelope and its one mapping point (design D4, moved here
//! from the demo server).
//!
//! Every failure leaves a handler as an [`HttpError`]: a status, a reason
//! from the protocol's closed registry, and the message. The Engine command
//! layer's codes, the store port's failures and the document reads' failures
//! map to it here and nowhere else. A host that keeps routes of its own next
//! to the door (the demo server's login, import and event stream) builds the
//! same envelope with it.

use super::HttpResponse;
use crate::documents::DocumentsError;
use crate::port_exec::ExecError;
use speclink_core::command::{CommandError, ErrorCode};
use speclink_protocol::command::RevertBlockedEvidence;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_store_port::{Expected, PortError};

/// A protocol error ready to become the `{ status, reason, message }`
/// envelope (with the refusal evidence flattened in when a flow gave one).
#[derive(Debug, Clone)]
pub struct HttpError {
    pub status: u16,
    pub reason: ErrorReason,
    pub message: String,
    pub evidence: Option<RevertBlockedEvidence>,
}

impl HttpError {
    fn new(status: u16, reason: ErrorReason, message: impl Into<String>) -> HttpError {
        HttpError { status, reason, message: message.into(), evidence: None }
    }

    /// 401: the request carries no identity.
    pub fn unauthenticated(message: impl Into<String>) -> HttpError {
        HttpError::new(401, ErrorReason::PermissionDenied, message)
    }

    /// 403: the caller may not do this in this scope.
    pub fn forbidden(message: impl Into<String>) -> HttpError {
        HttpError::new(403, ErrorReason::PermissionDenied, message)
    }

    /// 404: the addressed subject does not exist, or the caller cannot see it.
    pub fn not_found(message: impl Into<String>) -> HttpError {
        HttpError::new(404, ErrorReason::NotFound, message)
    }

    /// 409: a precondition refused the request.
    pub fn refused(message: impl Into<String>) -> HttpError {
        HttpError::new(409, ErrorReason::Refused, message)
    }

    /// 413: the content is larger than the route allows.
    pub fn payload_too_large(message: impl Into<String>) -> HttpError {
        HttpError::new(413, ErrorReason::Refused, message)
    }

    /// 400: the arguments are invalid.
    pub fn invalid_argument(message: impl Into<String>) -> HttpError {
        HttpError::new(400, ErrorReason::InvalidArgument, message)
    }

    /// 422: a config document cannot be parsed.
    pub fn invalid_config(message: impl Into<String>) -> HttpError {
        HttpError::new(422, ErrorReason::InvalidConfig, message)
    }

    /// 409: a version precondition failed; the message names expected and
    /// actual.
    pub fn revision_conflict(message: impl Into<String>) -> HttpError {
        HttpError::new(409, ErrorReason::RevisionConflict, message)
    }

    /// 503: the store, or the host, cannot serve right now.
    pub fn unavailable(message: impl Into<String>) -> HttpError {
        HttpError::new(503, ErrorReason::Unavailable, message)
    }

    /// 500: any other failure.
    pub fn internal(message: impl Into<String>) -> HttpError {
        HttpError::new(500, ErrorReason::Internal, message)
    }

    /// The envelope as the door's response.
    pub fn into_response(self) -> HttpResponse {
        let body = ErrorResponse {
            status: self.status,
            reason: self.reason,
            message: self.message,
            evidence: self.evidence,
        };
        HttpResponse::json(self.status, &body)
    }
}

/// The Engine command layer's five codes. The message is the Engine's frozen
/// text, relayed as it is.
impl From<CommandError> for HttpError {
    fn from(e: CommandError) -> HttpError {
        // The in-progress removal guard's evidence travels with the 409.
        let evidence = e.source.as_ref().and_then(|s| {
            s.downcast_ref::<speclink_core::inprogress::RevertBlocked>().map(|b| RevertBlockedEvidence {
                checked_tasks: b.checked_tasks,
                touched_files: b.touched_files.clone(),
            })
        });
        // A plan's dependency cycle is a data defect, not a server fault:
        // refused, by the type of its source, with the Engine's text.
        let plan_cycle =
            e.source.as_ref().is_some_and(|s| s.downcast_ref::<speclink_core::plan::PlanError>().is_some());
        // A caller who may not change workflow policy tried to: 403, like
        // any other write the caller's access does not allow.
        let policy_refused =
            e.source.as_ref().is_some_and(|s| s.downcast_ref::<crate::kit::PolicyChangeRefused>().is_some());
        let message = e.message;
        let mut error = match e.code {
            _ if policy_refused => HttpError::forbidden(message),
            ErrorCode::InvalidArgv => HttpError::invalid_argument(message),
            ErrorCode::NotFound => HttpError::not_found(message),
            ErrorCode::InvalidConfig => HttpError::invalid_config(message),
            ErrorCode::Refused => HttpError::refused(message),
            ErrorCode::Error if plan_cycle => HttpError::refused(message),
            ErrorCode::Error => HttpError::internal(message),
        };
        error.evidence = evidence;
        error
    }
}

/// What a client is told when the store cannot serve right now.
pub const STORE_UNAVAILABLE: &str = "the store backend is temporarily unavailable";

/// What a client is told when the store failed otherwise.
const STORE_FAILED: &str = "the store failed; the host's store has the cause";

/// The store port's three failures. A conflict names the document key with
/// its expected and actual versions. The store's own text for an outage or
/// another failure is not sent out: a host's driver can name addresses and
/// paths of its infrastructure.
impl From<PortError> for HttpError {
    fn from(e: PortError) -> HttpError {
        match e {
            PortError::Conflict { key, expected, actual } => {
                let expected = match expected {
                    Expected::Absent => "absent".to_string(),
                    Expected::At(version) => version.to_string(),
                };
                let actual = actual.map_or_else(|| "absent".to_string(), |v| v.to_string());
                HttpError::revision_conflict(format!("{}: expected {expected}, actual {actual}", key.as_str()))
            }
            PortError::Unavailable { .. } => HttpError::unavailable(STORE_UNAVAILABLE),
            PortError::Backend { .. } => HttpError::internal(STORE_FAILED),
        }
    }
}

/// A document read or write failure: not found 404, a broken workflow
/// config 422, an oversized board order 413, a moved scope 409, a corrupt
/// record 500, and the store's own failures as above.
impl From<DocumentsError> for HttpError {
    fn from(e: DocumentsError) -> HttpError {
        match e {
            DocumentsError::NotFound(message) => HttpError::not_found(message),
            DocumentsError::InvalidConfig(message) => HttpError::invalid_config(message),
            too_large @ DocumentsError::TooLarge { .. } => HttpError::payload_too_large(too_large.to_string()),
            moved @ DocumentsError::ScopeMoved { .. } => HttpError::revision_conflict(moved.to_string()),
            DocumentsError::Store(e) => HttpError::from(e),
        }
    }
}

/// A command run's failure, from whichever vocabulary produced it.
impl From<ExecError> for HttpError {
    fn from(e: ExecError) -> HttpError {
        match e {
            ExecError::Command(c) => HttpError::from(c),
            ExecError::Store(s) => HttpError::from(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store_port::{DocKey, Version};

    #[test]
    fn engine_five_codes_map_to_reason_and_status() {
        let cases = [
            (ErrorCode::InvalidArgv, 400, ErrorReason::InvalidArgument),
            (ErrorCode::NotFound, 404, ErrorReason::NotFound),
            (ErrorCode::InvalidConfig, 422, ErrorReason::InvalidConfig),
            (ErrorCode::Refused, 409, ErrorReason::Refused),
            (ErrorCode::Error, 500, ErrorReason::Internal),
        ];
        for (code, status, reason) in cases {
            let error = HttpError::from(CommandError::new(code, "engine text"));
            assert_eq!(error.status, status, "status for {code:?}");
            assert_eq!(error.reason, reason, "reason for {code:?}");
            assert_eq!(error.message, "engine text", "message relayed verbatim for {code:?}");
        }
    }

    #[test]
    fn a_plan_dependency_cycle_is_a_refused_409_carrying_the_engine_line() {
        let cycle = speclink_core::plan::PlanError::Cycle(vec!["a".into(), "b".into(), "a".into()]);
        let error = HttpError::from(CommandError {
            code: ErrorCode::Error,
            message: cycle.to_string(),
            source: Some(cycle.into()),
        });
        assert_eq!((error.status, error.reason), (409, ErrorReason::Refused));
        assert_eq!(error.message, "dependency cycle: a -> b -> a");

        // The same text from another source stays a 500.
        let other = HttpError::from(CommandError {
            code: ErrorCode::Error,
            message: "dependency cycle: a -> b -> a".into(),
            source: Some(anyhow::anyhow!("dependency cycle: a -> b -> a")),
        });
        assert_eq!((other.status, other.reason), (500, ErrorReason::Internal));
    }

    #[test]
    fn an_exec_error_dispatches_to_the_producing_layer() {
        let command = HttpError::from(ExecError::Command(CommandError::new(ErrorCode::NotFound, "no change")));
        assert_eq!(command.reason, ErrorReason::NotFound);

        let store = HttpError::from(ExecError::Store(PortError::Unavailable { message: "connection lost".into() }));
        assert_eq!((store.status, store.reason), (503, ErrorReason::Unavailable));
    }

    #[test]
    fn store_port_three_failures_map_to_reason_and_status() {
        let key = || DocKey::new("changes/demo/tasks.md").unwrap();
        let v = |n| Version::new(n).unwrap();

        let conflict = HttpError::from(PortError::Conflict { key: key(), expected: Expected::At(v(3)), actual: Some(v(4)) });
        assert_eq!((conflict.status, conflict.reason), (409, ErrorReason::RevisionConflict));
        assert_eq!(conflict.message, "changes/demo/tasks.md: expected 3, actual 4");

        let created_twice =
            HttpError::from(PortError::Conflict { key: key(), expected: Expected::Absent, actual: Some(v(4)) });
        assert_eq!(created_twice.message, "changes/demo/tasks.md: expected absent, actual 4");
        let gone = HttpError::from(PortError::Conflict { key: key(), expected: Expected::At(v(3)), actual: None });
        assert_eq!(gone.message, "changes/demo/tasks.md: expected 3, actual absent");

        let unavailable = HttpError::from(PortError::Unavailable { message: "timeout at 10.0.0.7".into() });
        assert_eq!((unavailable.status, unavailable.reason), (503, ErrorReason::Unavailable));
        assert_eq!(unavailable.message, STORE_UNAVAILABLE);

        let backend = HttpError::from(PortError::Backend { message: "disk full at /var/lib/db".into() });
        assert_eq!((backend.status, backend.reason), (500, ErrorReason::Internal));
        assert_eq!(backend.message, STORE_FAILED);
    }

    #[test]
    fn the_envelope_flattens_the_evidence() {
        let mut error = HttpError::refused("work recorded");
        error.evidence = Some(RevertBlockedEvidence { checked_tasks: 2, touched_files: vec!["a.rs".into()] });
        let body: serde_json::Value = serde_json::from_slice(&error.into_response().body).unwrap();
        assert_eq!(body["status"], 409);
        assert_eq!(body["reason"], "refused");
        assert_eq!(body["checkedTasks"], 2);
    }
}
