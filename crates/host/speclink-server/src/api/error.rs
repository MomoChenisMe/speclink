//! The wire error of the routes the server keeps for itself (design D4 of
//! host-kit-client-door).
//!
//! The envelope is the host kit's [`HttpError`]: the door and the server's
//! own routes answer in one shape, built in one place. [`ApiError`] only
//! makes it an axum response, and maps the failures of the event stream's
//! store reads and of the identity store.

use crate::identity::IdentityError;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use speclink_host::http::{HttpError, HttpResponse};
use speclink_store_port::PortError;

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

/// A failure of the store, as the host kit's door answers it: a temporary
/// failure is 503, any other is 500, and neither carries the store's text.
impl From<PortError> for ApiError {
    fn from(e: PortError) -> ApiError {
        HttpError::from(e).into()
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
