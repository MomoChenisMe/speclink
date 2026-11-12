//! The wire error of the routes the server keeps for itself (design D4 of
//! host-kit-client-door).
//!
//! The envelope is the host kit's [`HttpError`]: the door and the server's
//! own routes answer in one shape, built in one place. [`ApiError`] only
//! makes it an axum response, and maps the two failure vocabularies that are
//! the server's alone: the TeamStore's six classes (the import and the event
//! stream still use the TeamStore) and the identity store's failures.

use crate::identity::IdentityError;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use speclink_host::http::{HttpError, HttpResponse, STORE_UNAVAILABLE};
use speclink_store::StoreError;

/// An [`HttpError`] as the error of an axum handler.
#[derive(Debug, Clone)]
pub struct ApiError(pub HttpError);

impl From<HttpError> for ApiError {
    fn from(e: HttpError) -> ApiError {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        into_axum(self.0.into_response())
    }
}

/// A response of the host kit's door as an axum response.
pub(crate) fn into_axum(response: HttpResponse) -> Response {
    let mut builder = Response::builder().status(response.status);
    for (name, value) in response.headers {
        builder = builder.header(name, value);
    }
    builder.body(Body::from(response.body)).expect("the door's status and headers are valid")
}

/// The store's six failure classes → wire reason + status. A `revision_conflict`
/// carries its expected/actual detail in the message (the envelope has no detail
/// fields); a store-layer permission denial is 403, distinct from the 401 the
/// auth layer returns.
impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> ApiError {
        match e {
            StoreError::NotFound => HttpError::not_found("the addressed scope or document does not exist"),
            StoreError::PermissionDenied => {
                HttpError::forbidden("not allowed to perform this operation in this scope")
            }
            StoreError::RevisionConflict {
                doc,
                expected,
                actual,
            } => HttpError::revision_conflict(format!(
                "{:?}: expected {:?}, actual {:?}",
                doc.doc, expected, actual
            )),
            StoreError::Unavailable => HttpError::unavailable(STORE_UNAVAILABLE),
            StoreError::Corrupt { reason } => HttpError::internal(format!("corrupt: {reason}")),
            StoreError::Backend { source } => HttpError::internal(format!("backend: {source}")),
        }
        .into()
    }
}

/// An identity-store failure → wire reason + status, for the admin API. A guard
/// rejection (duplicate key, refused last-admin) is 409; an unknown subject is
/// 404; an open/backend failure is 500. Auth-layer identity errors keep their
/// own inline mapping (a uniform 401/403), so this covers the admin actions.
impl From<IdentityError> for ApiError {
    fn from(e: IdentityError) -> ApiError {
        match e {
            IdentityError::Duplicate(m) => HttpError::refused(m),
            IdentityError::Refused(m) => HttpError::refused(m),
            IdentityError::NotFound(m) => HttpError::not_found(m),
            IdentityError::InvalidInvitation => HttpError::refused("the invitation is invalid"),
            IdentityError::Open(m) | IdentityError::Backend(m) => {
                HttpError::internal(format!("identity store error: {m}"))
            }
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_protocol::error::ErrorReason;
    use speclink_store::{DocRef, DocumentId, ExpectedRevision, ProjectId, RepoId, Revision};

    #[test]
    fn store_six_classes_map_to_reason_and_status() {
        let doc = DocRef {
            project: ProjectId::new("demo"),
            repo: RepoId::new("backend"),
            doc: DocumentId::WorkflowConfig,
        };
        let cases = [
            (StoreError::NotFound, 404, ErrorReason::NotFound),
            (StoreError::PermissionDenied, 403, ErrorReason::PermissionDenied),
            (
                StoreError::RevisionConflict {
                    doc: doc.clone(),
                    expected: ExpectedRevision::At(Revision(2)),
                    actual: Some(Revision(5)),
                },
                409,
                ErrorReason::RevisionConflict,
            ),
            (StoreError::Unavailable, 503, ErrorReason::Unavailable),
            (StoreError::Corrupt { reason: "bad".into() }, 500, ErrorReason::Internal),
            (StoreError::Backend { source: "io".into() }, 500, ErrorReason::Internal),
        ];
        for (err, status, reason) in cases {
            let is_conflict = matches!(err, StoreError::RevisionConflict { .. });
            let api = ApiError::from(err).0;
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
}
