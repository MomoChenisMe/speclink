//! A fake speclink server for the auth tests: it answers the routes it is
//! given, 404s everything else, and records every path it was asked for.
//!
//! One copy serves two test crates: the binary tests here (`tests/it`) and the
//! function-level tests in `src/verbs/connection.rs`, which include this file
//! by path.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) struct Fake {
    server: Arc<tiny_http::Server>,
    /// `http://127.0.0.1:<port>`
    pub(crate) origin: String,
    routes: Arc<Mutex<HashMap<String, (u16, String)>>>,
    paths: Arc<Mutex<Vec<String>>>,
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

impl Fake {
    pub(crate) fn new() -> Fake {
        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
        let origin = format!("http://{}", server.server_addr());
        let routes: Arc<Mutex<HashMap<String, (u16, String)>>> = Arc::default();
        let paths: Arc<Mutex<Vec<String>>> = Arc::default();
        let (looper, table, log) = (server.clone(), routes.clone(), paths.clone());
        std::thread::spawn(move || {
            for req in looper.incoming_requests() {
                let path = req.url().split('?').next().unwrap_or_default().to_string();
                let (status, body) = table.lock().unwrap().get(&path).cloned().unwrap_or((
                    404,
                    r#"{"status":404,"reason":"not_found","message":"no route"}"#.to_string(),
                ));
                log.lock().unwrap().push(path);
                let resp = tiny_http::Response::from_string(body).with_status_code(status).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                );
                let _ = req.respond(resp);
            }
        });
        Fake { server, origin, routes, paths }
    }

    /// Answer `path` with `status` and `body` from now on.
    pub(crate) fn route(&self, path: &str, status: u16, body: &str) -> &Fake {
        self.routes.lock().unwrap().insert(path.to_string(), (status, body.to_string()));
        self
    }

    /// The paths asked for so far, in order, without their query.
    pub(crate) fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }

    /// A project-scoped connection URL on this server.
    pub(crate) fn project_url(&self) -> String {
        format!("{}/api/speclink/v1/projects/demo", self.origin)
    }
}
