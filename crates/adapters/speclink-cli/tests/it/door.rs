//! A real Server for the remote tests: the host kit's HTTP door on the
//! in-memory store port, behind a small HTTP listener. What it answers is
//! what a Server of this engine answers. It has one project, `demo`, with one
//! repo, `backend`, on scope `s1`, writable.

use speclink_host::http::{HttpRequest, Method};
use speclink_host::kit::{BeforeCommand, Decision, Hooks, HostKit, HostKitOptions, Identity, ProjectEntry, Reader, RepoEntry};
use speclink_store_port::{Actor, Commit, DocKey, Expected, MemoryStorePort, Op, ScopeId, StorePort, Timestamp};
use std::sync::{Arc, Mutex};

pub(crate) struct Door {
    server: Arc<tiny_http::Server>,
    /// The project-scoped connection URL.
    pub(crate) base: String,
    store: Arc<MemoryStorePort>,
    /// `METHOD path?query` of every request, the project prefix removed.
    asked: Arc<Mutex<Vec<String>>>,
}

pub(crate) const PROJECT: &str = "/api/speclink/v1/projects/demo";

fn scope() -> ScopeId {
    ScopeId::new("s1").unwrap()
}

fn commit(store: &MemoryStorePort, ops: Vec<Op>, at_millis: u64) {
    let at = Timestamp::new(at_millis).unwrap();
    let seed = Commit::new(Actor::new("seed", "seed").unwrap(), "seed", at, ops, Vec::new()).unwrap();
    store.commit(&scope(), &seed).expect("seed commit");
}

/// A server whose store holds `docs` (store keys), committed at `at_millis`.
pub(crate) fn door(docs: &[(&str, &str)], at_millis: u64) -> Door {
    door_with_hooks(docs, at_millis, Hooks::default())
}

/// [`door`] with the host's `hooks`.
pub(crate) fn door_with_hooks(docs: &[(&str, &str)], at_millis: u64, hooks: Hooks) -> Door {
    let scope = scope();
    let store = Arc::new(MemoryStorePort::new());
    // A commit holds at least one operation: an empty store gets none.
    if !docs.is_empty() {
        let ops = docs
            .iter()
            .map(|(key, content)| Op::Put {
                key: DocKey::new(*key).unwrap(),
                content: content.to_string(),
                expected: Expected::Absent,
            })
            .collect();
        commit(&store, ops, at_millis);
    }

    let options = HostKitOptions {
        scopes: Some(Box::new(move |_identity: &Identity| {
            Ok(vec![ProjectEntry {
                key: "demo".to_string(),
                name: "Demo".to_string(),
                repos: vec![RepoEntry {
                    key: "backend".to_string(),
                    name: "Backend".to_string(),
                    scope: scope.clone(),
                    can_write: true,
                    can_write_policy: true,
                }],
            }])
        })),
        ..Default::default()
    };
    let kit = HostKit::with_options(store.clone(), hooks, options).unwrap();
    let identity = Identity::new("u_1", "Tester").unwrap();

    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}{PROJECT}", server.server_addr());
    let asked: Arc<Mutex<Vec<String>>> = Arc::default();
    let (looper, log) = (server.clone(), asked.clone());
    std::thread::spawn(move || {
        for mut req in looper.incoming_requests() {
            let mut body = Vec::new();
            let _ = req.as_reader().read_to_end(&mut body);
            let url = req.url().to_string();
            let (path, query) = match url.split_once('?') {
                Some((path, query)) => (path.to_string(), Some(query.to_string())),
                None => (url.clone(), None),
            };
            log.lock().unwrap().push(format!("{} {}", req.method(), url.trim_start_matches(PROJECT)));
            let request = HttpRequest {
                method: Method::from_name(&req.method().to_string()).expect("a method the door takes"),
                path,
                query,
                headers: req.headers().iter().map(|h| (h.field.to_string(), h.value.to_string())).collect(),
                body,
            };
            let response = match kit.handle(&request, Some(&identity)) {
                Some(response) => {
                    let mut out = tiny_http::Response::from_data(response.body).with_status_code(response.status);
                    for (name, value) in &response.headers {
                        out.add_header(tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap());
                    }
                    out
                }
                None => tiny_http::Response::from_data(Vec::new()).with_status_code(404),
            };
            let _ = req.respond(response);
        }
    });
    Door { server, base, store, asked }
}

impl Door {
    /// Write one document of the store (a store key), new or in place.
    pub(crate) fn put(&self, key: &str, content: &str, at_millis: u64) {
        let key = DocKey::new(key).unwrap();
        let current = self.store.read(&scope(), std::slice::from_ref(&key)).unwrap().remove(0);
        let expected = current.map_or(Expected::Absent, |doc| Expected::At(doc.version));
        commit(&self.store, vec![Op::Put { key, content: content.to_string(), expected }], at_millis);
    }

    /// Every request so far.
    pub(crate) fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }

    /// Every request but the handshake.
    pub(crate) fn verb_calls(&self) -> Vec<String> {
        self.asked().into_iter().filter(|call| call != "GET /binding").collect()
    }
}

impl Drop for Door {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

// --- 入口的讀取被宿主拒絕時，CLI 的行為和 GET /changes 被拒絕時相同
// (change door-reads-ask-before-command) ---

/// Hooks that refuse `command` with `reason` and allow every other one.
fn refusing(command: &'static str, reason: &'static str) -> Hooks {
    Hooks {
        before_command: Some(Box::new(move |info: &BeforeCommand, _: &Reader<'_>| match info.command == command {
            true => Decision::Deny { reason: reason.to_string() },
            false => Decision::Allow,
        })),
        after_commit: None,
    }
}

#[test]
fn door_refusing_workflow_config_show_fails_the_verb_like_a_refused_change_list() {
    use crate::remote_read_verbs::TempProject;
    const DOCS: [(&str, &str); 1] = [("config.yaml", "schema: spec-driven\n")];
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();

    let server = door_with_hooks(&DOCS, 1_794_452_400_000, refusing("workflow-config-show", "設定只給管理員看"));
    let p = TempProject::remote("door-config-refused", &server.base);
    let out = p.run(&["workflow-config", "show"]);
    assert!(!out.status.success(), "stdout: {}", text(&out.stdout));
    assert_eq!(text(&out.stdout), "", "nothing on stdout");
    assert!(server.verb_calls().contains(&"GET /config".to_string()), "{:?}", server.verb_calls());
    let config_refused = text(&out.stderr);
    assert_eq!(config_refused, "Error: 設定只給管理員看\n", "the host's reason as the error");

    // The same host reason on the change list: the same exit code and the
    // same stderr, with the reason in its place.
    let server = door_with_hooks(&DOCS, 1_794_452_400_000, refusing("list", "設定只給管理員看"));
    let p = TempProject::remote("door-list-refused", &server.base);
    let listed = p.run(&["list"]);
    assert_eq!(listed.status.code(), out.status.code());
    assert_eq!(text(&listed.stdout), "");
    assert_eq!(text(&listed.stderr), config_refused);
}
