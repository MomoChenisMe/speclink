//! The HTTP door of the host kit (design D1–D3): a request in a form no web
//! framework owns, and the identity the host already verified, go in; a
//! response comes out, or "not the door's path".
//!
//! The door serves the original CLI and Speclink Desktop: every path is under
//! `/api/speclink/v1/`. It never reads the `Authorization` header; the
//! identity comes from the host alone. A path the door does not know is
//! handed back, so the host keeps its own routes under the same prefix.
//!
//! A host writes one route: verify the token, then
//!
//! ```text
//! match kit.handle(&request, identity.as_ref()) {
//!     Some(response) => send(response.status, response.headers, response.body),
//!     None => the host's own routes, or its 404,
//! }
//! ```

mod coverage;
mod error;
mod router;
mod routes;
mod wire;

pub use coverage::{wire_coverage, WireCoverage};
pub use error::{HttpError, STORE_UNAVAILABLE};
pub use router::{match_route, routes, RouteMatch};

use crate::binding::{resolve_binding, BindingCandidate, BindingError};
use crate::kit::{Caller, HostKit, Identity, ProjectEntry};
use router::{Route, READ_ONLY_POSTS};
use serde::Serialize;
use speclink_protocol::binding::ScopeRef;
use speclink_protocol::query::{AuthWhoamiResponse, ProjectScope, ScopesResponse, WhoamiUser};
use speclink_protocol::API_VERSION;
use speclink_store::{ProjectId, RepoId};

/// The five methods the door takes. A host answers any other with 405.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
}

impl Method {
    /// The method of an HTTP method name (upper case, as on the wire).
    pub fn from_name(name: &str) -> Option<Method> {
        match name {
            "GET" => Some(Method::Get),
            "HEAD" => Some(Method::Head),
            "POST" => Some(Method::Post),
            "PUT" => Some(Method::Put),
            "DELETE" => Some(Method::Delete),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }

    fn reads(self) -> bool {
        matches!(self, Method::Get | Method::Head)
    }
}

/// A request as the host received it. `path` is the path as sent, still
/// percent-encoded; `query` is the text after `?`, if any. Header names
/// match in any case.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// The value of the first header named `name`, in any case.
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

/// The door's answer. The headers are `content-type` and, where the route
/// has one, `etag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// `body` as JSON with `status`.
    pub(crate) fn json<T: Serialize>(status: u16, body: &T) -> HttpResponse {
        HttpResponse {
            status,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: serde_json::to_vec(body).expect("a wire type serializes to JSON"),
        }
    }
}

const API_ROOT: &str = "/api/speclink/v1";

/// What the door answers when the host gave no identity.
const NO_IDENTITY: &str = "this request needs an authenticated identity";

impl HostKit {
    /// Answer `request` for `identity` (none when the host could not verify
    /// one), or `None` when the path is not the door's.
    ///
    /// For a project path the door checks, in this order: the path is a
    /// route (else `None`); there is an identity (401); the API version
    /// header is this one (409); the caller's scope function lists the
    /// project (404) and the repo — the `X-Speclink-Repo` header's, or the
    /// only one (404, or 409 with the candidates when there are several);
    /// a request that writes needs `can_write` (403). Then the route runs.
    pub fn handle(&self, request: &HttpRequest, identity: Option<&Identity>) -> Option<HttpResponse> {
        let reads = request.method.reads();
        let result = match request.path.strip_prefix(API_ROOT) {
            Some("/auth-methods") if reads => self.auth_methods(),
            Some("/whoami") if reads => self.root_whoami(identity),
            Some("/scopes") if reads => self.scope_list(identity),
            _ => {
                let (route, params) = router::find(request.method, &request.path)?;
                self.project_request(request, identity, route, params)
            }
        };
        let mut response = result.unwrap_or_else(HttpError::into_response);
        if request.method == Method::Head {
            response.body.clear();
        }
        Some(response)
    }

    /// The caller's projects, as the host's scope function gives them.
    fn projects_of(&self, identity: &Identity, project_key: Option<&str>) -> Result<Vec<ProjectEntry>, HttpError> {
        let Some(scopes) = &self.options.scopes else {
            return Err(match project_key {
                Some(key) => project_not_found(key),
                None => HttpError::not_found("this host lists no scopes"),
            });
        };
        // The host's text can name its infrastructure: it is not sent out.
        scopes(identity).map_err(|_| HttpError::unavailable("the host could not list the caller's projects"))
    }

    /// `GET /api/speclink/v1/auth-methods`: the host's declaration, as it
    /// gave it. No identity and no version header: a client asks it before
    /// it has a credential.
    fn auth_methods(&self) -> Result<HttpResponse, HttpError> {
        let declared = self.options.auth_methods.as_ref();
        let declared = declared.ok_or_else(|| HttpError::not_found("this host declares no login methods"))?;
        Ok(HttpResponse::json(200, declared))
    }

    /// `GET /api/speclink/v1/whoami`: the identity, with no project and no
    /// version header.
    fn root_whoami(&self, identity: Option<&Identity>) -> Result<HttpResponse, HttpError> {
        let identity = identity.ok_or_else(|| HttpError::unauthenticated(NO_IDENTITY))?;
        let user = WhoamiUser { name: identity.shown_name().to_string(), handle: identity.id().to_string() };
        Ok(HttpResponse::json(200, &AuthWhoamiResponse { user }))
    }

    /// `GET /api/speclink/v1/scopes`: the caller's projects and repos, without
    /// the store scopes or the switches. No version header, as before.
    fn scope_list(&self, identity: Option<&Identity>) -> Result<HttpResponse, HttpError> {
        let identity = identity.ok_or_else(|| HttpError::unauthenticated(NO_IDENTITY))?;
        let projects = self
            .projects_of(identity, None)?
            .into_iter()
            .map(|project| ProjectScope {
                id: format!("prj_{}", project.key),
                repos: project
                    .repos
                    .into_iter()
                    .map(|repo| ScopeRef { id: format!("repo_{}", repo.key), key: repo.key, name: repo.name })
                    .collect(),
                key: project.key,
                name: project.name,
            })
            .collect();
        Ok(HttpResponse::json(200, &ScopesResponse { projects }))
    }

    fn project_request(
        &self,
        request: &HttpRequest,
        identity: Option<&Identity>,
        route: &'static Route,
        raw_params: Vec<String>,
    ) -> Result<HttpResponse, HttpError> {
        let identity = identity.ok_or_else(|| HttpError::unauthenticated(NO_IDENTITY))?;
        check_api_version(request.header("x-speclink-api-version"))?;
        let mut params = raw_params.iter().map(|raw| decode(raw)).collect::<Result<Vec<_>, _>>()?;
        let key = params.remove(0);

        let projects = self.projects_of(identity, Some(&key))?;
        let project = projects.iter().find(|p| p.key == key).ok_or_else(|| project_not_found(&key))?;
        let repo = choose_repo(&project.key, &project.repos, |r| &r.key, request.header("x-speclink-repo"))?;

        let writes = !request.method.reads() && !READ_ONLY_POSTS.contains(&route.pattern);
        if writes && !repo.can_write {
            return Err(write_refused(&project.key));
        }

        let caller = Caller { identity: identity.clone(), scope: repo.scope.clone() };
        let bound = routes::Request { kit: self, http: request, caller, project, repo, params };
        (route.handler)(&bound)
    }
}

fn project_not_found(key: &str) -> HttpError {
    HttpError::not_found(format!("project '{key}' not found"))
}

/// The version header must name this API version. `sent` is the value of
/// `x-speclink-api-version`, if any.
pub fn check_api_version(sent: Option<&str>) -> Result<(), HttpError> {
    if sent == Some(API_VERSION) {
        return Ok(());
    }
    let sent = sent.map(|v| format!(", client sent '{v}'")).unwrap_or_default();
    Err(HttpError::refused(format!("incompatible api version — this server speaks version '{API_VERSION}'{sent}")))
}

/// The repo a request is for: the one `named` (the `x-speclink-repo` header),
/// which must be one of `repos`, or the project's only repo. Several repos
/// and no name is refused with the candidates: the Host's binding rule
/// ([`resolve_binding`]) decides, and it never picks one.
pub fn choose_repo<'r, R>(
    project_key: &str,
    repos: &'r [R],
    key_of: impl Fn(&R) -> &str,
    named: Option<&str>,
) -> Result<&'r R, HttpError> {
    if let Some(name) = named {
        return repos.iter().find(|repo| key_of(repo) == name).ok_or_else(|| {
            HttpError::not_found(format!("repo '{name}' is not registered in project '{project_key}'"))
        });
    }
    let candidates = repos
        .iter()
        .map(|repo| BindingCandidate {
            project: ProjectId::new(project_key.to_string()),
            repo: RepoId::new(key_of(repo).to_string()),
            project_key: project_key.to_string(),
            repo_key: key_of(repo).to_string(),
            permitted: true,
        })
        .collect();
    match resolve_binding(candidates) {
        // Resolved with every candidate permitted: there is exactly one.
        Ok(_) => Ok(&repos[0]),
        Err(BindingError::Missing) => {
            Err(HttpError::not_found(format!("project '{project_key}' has no registered repo")))
        }
        Err(BindingError::Ambiguous { candidates }) => Err(HttpError::refused(format!(
            "project '{project_key}' registers multiple repos — specify one with X-Speclink-Repo (candidates: {})",
            candidates.join(", ")
        ))),
        Err(BindingError::PermissionDenied { .. }) => {
            Err(HttpError::forbidden(format!("no permitted repo in project '{project_key}'")))
        }
    }
}

/// The refusal of a request that writes from a caller who cannot write in
/// the project. The text is the one server-verb-api fixes for a reader.
pub fn write_refused(project_key: &str) -> HttpError {
    HttpError::forbidden(format!(
        "your role in project '{project_key}' is reader; this action needs the editor role"
    ))
}

/// A path param, percent-decoded.
fn decode(raw: &str) -> Result<String, HttpError> {
    percent_encoding::percent_decode_str(raw)
        .decode_utf8()
        .map(|text| text.into_owned())
        .map_err(|_| HttpError::invalid_argument(format!("the path segment '{raw}' is not UTF-8 once decoded")))
}
