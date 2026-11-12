//! Shared application state handed to every route handler.

use crate::config::ServerConfig;
use crate::events::EventHub;
use crate::identity::{IdentityStore, MembershipRole};
use speclink_host::kit::{AfterCommit, Hooks, HostKit, HostKitOptions, Identity, ProjectEntry, RepoEntry};
use speclink_host::teamstore_port::{scope_id, teamstore_scope, TeamStorePort};
use speclink_protocol::auth::AuthMethodsResponse;
use speclink_protocol::events::{EventTransport, TransportKind};
use speclink_store::{ProjectId, RepoId, Scope, TeamStore};
use std::collections::HashSet;
use std::sync::Arc;

/// The store backend, shared across async handlers. `Send + Sync` so a handler
/// can move a clone into `spawn_blocking` to run the synchronous Host path.
pub type SharedStore = Arc<dyn TeamStore + Send + Sync>;

/// The store backend as a store port: the one adapter every request shares,
/// so a run of requests with no commit in between loads a scope once. The
/// project routes go through the host kit on it, each request with its
/// thread pinned (`forward`), so a request reads from one load.
pub type SharedPort = Arc<TeamStorePort<SharedStore>>;

/// The identity store, shared across handlers. The API auth precondition and the
/// web entry both resolve identity through it.
pub type SharedIdentity = Arc<dyn IdentityStore>;

/// State every handler receives: the store backend and its store-port
/// adapter, the host kit on that adapter, the identity store, the validated
/// configuration (Project/Repo registry and public origin), and the
/// per-scope event broadcaster fed from the store's outbox.
///
/// The project routes and the scope list go through `kit` (design D8).
/// `store` is left for what the store port has no operation for: the
/// import, the event stream's outbox, the admin surfaces and the backups.
#[derive(Clone)]
pub struct AppState {
    pub store: SharedStore,
    pub port: SharedPort,
    pub kit: Arc<HostKit>,
    pub identity: SharedIdentity,
    pub config: Arc<ServerConfig>,
    pub events: Arc<EventHub>,
}

impl AppState {
    /// The state over `store`, with the one store-port adapter and the host
    /// kit built here.
    pub fn new(
        store: SharedStore,
        identity: SharedIdentity,
        config: Arc<ServerConfig>,
        events: Arc<EventHub>,
    ) -> AppState {
        let port = Arc::new(TeamStorePort::new(store.clone()));
        let kit = Arc::new(host_kit(port.clone(), identity.clone(), events.clone()));
        AppState {
            store,
            port,
            kit,
            identity,
            config,
            events,
        }
    }
}

/// The demo server as a host of the kit (design D8): the scopes are the
/// caller's memberships, both switches are "is an editor", the login methods
/// are device login and access keys, the push stream is the server's own
/// SSE route, and every commit wakes that stream.
fn host_kit(port: SharedPort, identity: SharedIdentity, events: Arc<EventHub>) -> HostKit {
    let hooks = Hooks {
        before_command: None,
        after_commit: Some(Box::new(move |info: &AfterCommit| {
            if let Ok(scope) = teamstore_scope(&info.caller.scope) {
                events.notify(&scope);
            }
        })),
    };
    let options = HostKitOptions {
        scopes: Some(Box::new(move |who: &Identity| member_scopes(&*identity, who.id()))),
        auth_methods: Some(AuthMethodsResponse {
            methods: vec!["device".to_string(), "token".to_string()],
            browser: None,
        }),
        push_transports: vec![EventTransport {
            kind: TransportKind::Sse,
            url: "/events".to_string(),
            resume: true,
        }],
        // Every skill of the Engine.
        skills: None,
    };
    HostKit::with_options(port, hooks, options).expect("no skill list, so no unknown skill name")
}

/// The projects `user_id` is a member of, in registry order, each with its
/// registered repos.
fn member_scopes(identity: &dyn IdentityStore, user_id: &str) -> Result<Vec<ProjectEntry>, String> {
    let unavailable = |_| "identity store unavailable".to_string();
    let members: HashSet<String> = identity
        .list_memberships(user_id)
        .map_err(unavailable)?
        .into_iter()
        .collect();
    let mut projects = Vec::new();
    for project in identity.list_projects().map_err(unavailable)? {
        if !members.contains(&project.key) {
            continue;
        }
        let editor = identity
            .membership_role(user_id, &project.key)
            .map_err(unavailable)?
            == Some(MembershipRole::Editor);
        let mut repos = Vec::new();
        for repo in identity.list_repos(&project.key).map_err(unavailable)? {
            let scope = Scope::new(ProjectId::new(project.key.clone()), RepoId::new(repo.key.clone()));
            repos.push(RepoEntry {
                scope: scope_id(&scope).map_err(|e| e.to_string())?,
                key: repo.key,
                name: repo.name,
                can_write: editor,
                can_write_policy: editor,
            });
        }
        projects.push(ProjectEntry {
            key: project.key,
            name: project.name,
            repos,
        });
    }
    Ok(projects)
}
