//! The single request layer plus the per-verb path mapping.
//!
//! Every request carries the contract's three headers (`Authorization`,
//! `X-Speclink-Api-Version`, `X-Speclink-Repo`); every non-2xx response goes
//! through the crate's central error-translation table. Verb methods are
//! thin path mappings over the request layer — no verb re-implements
//! transport or error handling.

use crate::{translate_status, translate_transport, RemoteError};

/// The contract major version this client speaks (`X-Speclink-Api-Version`).
pub const API_VERSION: &str = "1";

/// A client bound to one project-scoped base URL, one token, and (optionally)
/// one repo identity.
pub struct Client {
    base: String,
    token: String,
    repo: Option<String>,
    agent: ureq::Agent,
}

impl Client {
    /// `base_url` is the project-scoped connection URL (a trailing slash is
    /// tolerated); `repo` is the registered repo name from the connection
    /// file, when declared.
    pub fn new(base_url: &str, token: &str, repo: Option<&str>) -> Client {
        Client {
            base: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            repo: repo.map(str::to_string),
            agent: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(30))
                .build(),
        }
    }

    /// One request through the contract's header and error rules. Every verb
    /// funnels through here — headers and translation exist exactly once.
    fn call(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
        extra_headers: &[(&str, &str)],
    ) -> Result<serde_json::Value, RemoteError> {
        let mut req = self
            .agent
            .request(method, &format!("{}{}", self.base, path))
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("X-Speclink-Api-Version", API_VERSION);
        if let Some(repo) = &self.repo {
            req = req.set("X-Speclink-Repo", repo);
        }
        for (k, v) in extra_headers {
            req = req.set(k, v);
        }
        let result = match body {
            Some(json) => req.send_json(json.clone()),
            None => req.call(),
        };
        match result {
            Ok(resp) => resp.into_json().map_err(|_| RemoteError {
                message: "unexpected server response — the server did not return valid JSON"
                    .into(),
                reason: None,
            }),
            Err(ureq::Error::Status(status, resp)) => {
                let body: Option<serde_json::Value> = resp.into_json().ok();
                Err(translate_status(status, body.as_ref()))
            }
            Err(ureq::Error::Transport(_)) => Err(translate_transport()),
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, RemoteError> {
        self.call("GET", path, None, &[])
    }

    fn post(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, RemoteError> {
        self.call("POST", path, Some(&body), &[])
    }

    // --- read path ---

    /// `GET /changes`
    pub fn list_changes(&self) -> Result<serde_json::Value, RemoteError> {
        self.get("/changes")
    }

    /// `GET /changes/{name}`
    pub fn get_change(&self, name: &str) -> Result<serde_json::Value, RemoteError> {
        self.get(&format!("/changes/{name}"))
    }

    /// `GET /changes/{name}/instructions/{artifact}` (artifact may be "apply")
    pub fn instructions(
        &self,
        name: &str,
        artifact: &str,
    ) -> Result<serde_json::Value, RemoteError> {
        self.get(&format!("/changes/{name}/instructions/{artifact}"))
    }

    /// `GET /changes/{name}/artifacts/{artifact}`
    pub fn get_artifact(
        &self,
        name: &str,
        artifact: &str,
    ) -> Result<serde_json::Value, RemoteError> {
        self.get(&format!("/changes/{name}/artifacts/{artifact}"))
    }

    /// `GET /specs`
    pub fn list_specs(&self) -> Result<serde_json::Value, RemoteError> {
        self.get("/specs")
    }

    /// `GET /specs/{capability}`
    pub fn get_spec(&self, capability: &str) -> Result<serde_json::Value, RemoteError> {
        self.get(&format!("/specs/{capability}"))
    }

    /// `GET /language`
    pub fn language(&self) -> Result<serde_json::Value, RemoteError> {
        self.get("/language")
    }

    /// `GET /config`
    pub fn config(&self) -> Result<serde_json::Value, RemoteError> {
        self.get("/config")
    }

    /// `GET /whoami`
    pub fn whoami(&self) -> Result<serde_json::Value, RemoteError> {
        self.get("/whoami")
    }

    // --- write path ---

    /// `POST /changes`
    pub fn create_change(
        &self,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, RemoteError> {
        self.post("/changes", body)
    }

    /// `PUT /changes/{name}/artifacts/{artifact}` with `If-Match: <version>`
    /// (`0` = create-only). The version must always be the one obtained when
    /// the content was read — there is no force-write.
    pub fn put_artifact(
        &self,
        name: &str,
        artifact: &str,
        content: &str,
        if_match: u64,
    ) -> Result<serde_json::Value, RemoteError> {
        self.call(
            "PUT",
            &format!("/changes/{name}/artifacts/{artifact}"),
            Some(&serde_json::json!({ "content": content })),
            &[("If-Match", &if_match.to_string())],
        )
    }

    /// `POST /changes/{name}/tasks/{taskId}/done`
    pub fn task_done(
        &self,
        name: &str,
        task_id: &str,
        touched_files: &[String],
    ) -> Result<serde_json::Value, RemoteError> {
        let body = if touched_files.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({ "touchedFiles": touched_files })
        };
        self.post(&format!("/changes/{name}/tasks/{task_id}/done"), body)
    }

    /// `POST /changes/{name}/claim`
    pub fn claim(&self, name: &str) -> Result<serde_json::Value, RemoteError> {
        self.post(&format!("/changes/{name}/claim"), serde_json::json!({}))
    }

    /// `POST /changes/{name}/archive`
    pub fn archive(&self, name: &str) -> Result<serde_json::Value, RemoteError> {
        self.post(&format!("/changes/{name}/archive"), serde_json::json!({}))
    }

    // --- discussions ---

    /// `GET /discussions?archived=`
    pub fn list_discussions(&self, archived: bool) -> Result<serde_json::Value, RemoteError> {
        if archived {
            self.get("/discussions?archived=true")
        } else {
            self.get("/discussions")
        }
    }

    /// `POST /discussions`
    pub fn new_discussion(&self, topic: &str) -> Result<serde_json::Value, RemoteError> {
        self.post("/discussions", serde_json::json!({ "topic": topic }))
    }

    /// `GET /discussions/{slug}`
    pub fn show_discussion(&self, slug: &str) -> Result<serde_json::Value, RemoteError> {
        self.get(&format!("/discussions/{slug}"))
    }

    /// `PUT /discussions/{slug}/context`
    pub fn discussion_context(
        &self,
        slug: &str,
        content: &str,
    ) -> Result<serde_json::Value, RemoteError> {
        self.call(
            "PUT",
            &format!("/discussions/{slug}/context"),
            Some(&serde_json::json!({ "content": content })),
            &[],
        )
    }

    /// `POST /discussions/{slug}/rounds`
    pub fn discussion_add_round(
        &self,
        slug: &str,
        mode: &str,
        content: &str,
    ) -> Result<serde_json::Value, RemoteError> {
        self.post(
            &format!("/discussions/{slug}/rounds"),
            serde_json::json!({ "mode": mode, "content": content }),
        )
    }

    /// `POST /discussions/{slug}/conclude`
    pub fn discussion_conclude(
        &self,
        slug: &str,
        content: &str,
    ) -> Result<serde_json::Value, RemoteError> {
        self.post(
            &format!("/discussions/{slug}/conclude"),
            serde_json::json!({ "content": content }),
        )
    }

    /// `POST /discussions/{slug}/archive`
    pub fn discussion_archive(&self, slug: &str) -> Result<serde_json::Value, RemoteError> {
        self.post(
            &format!("/discussions/{slug}/archive"),
            serde_json::json!({}),
        )
    }

    /// `POST /discussions/{slug}/promote`
    pub fn discussion_promote(
        &self,
        slug: &str,
        name: Option<&str>,
    ) -> Result<serde_json::Value, RemoteError> {
        let body = match name {
            Some(n) => serde_json::json!({ "name": n }),
            None => serde_json::json!({}),
        };
        self.post(&format!("/discussions/{slug}/promote"), body)
    }
}
