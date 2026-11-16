//! The scope of a project and a repo as a store-port scope id.
//!
//! A host that names its scopes by project and repo, as the reference server
//! does, turns the pair into the id with [`scope_id`] on every route: the id
//! is what the store port, the host kit and the event stream take.

use speclink_store_port::{PortError, ScopeId};

pub use speclink_store::{ProjectId, RepoId, Scope};

/// The store-port scope id of a project and a repo: `<project>/<repo>`.
///
/// A project or repo name can hold a `/`, so the project is written with its
/// `%` and `/` escaped (`%25`, `%2F`): the first `/` of an id always ends the
/// project, and no two scopes share an id. Fails when the names cannot make
/// a scope id (one longer than a scope id may be).
pub fn scope_id(scope: &Scope) -> Result<ScopeId, PortError> {
    let project = scope.project.as_str().replace('%', "%25").replace('/', "%2F");
    ScopeId::new(format!("{project}/{}", scope.repo.as_str())).map_err(|e| PortError::Backend { message: e.to_string() })
}
