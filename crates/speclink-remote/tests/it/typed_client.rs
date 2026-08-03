//! The typed-client contract: every verb's request path and body match the
//! protocol DTOs, responses come back as protocol types (no raw JSON
//! bypass), If-Match travels typed and a stale write surfaces as
//! `revision_conflict` with the existing conflict wording, and every registry
//! reason maps to a CLI message byte-identical to the current remote error
//! translation (design decision three).

use speclink_protocol::command::{
    AddDiscussionRoundRequest, BindDiscussionRequest, CreateChangeRequest,
    CreateDiscussionRequest, PromoteDiscussionRequest, PutArtifactRequest, TaskDoneRequest,
};
use speclink_protocol::context::ContextSnapshotRequest;
use speclink_remote::client::{Client, ContextSnapshotOutcome};
use std::sync::{Arc, Mutex};

// --- capturing mock server (serves every request with one status/body) ---

#[derive(Clone, Debug)]
struct Captured {
    method: String,
    path: String,
    headers: Vec<(String, String)>, // lowercased names
    body: String,
}

impl Captured {
    fn header(&self, name: &str) -> Option<&str> {
        let want = name.to_ascii_lowercase();
        self.headers.iter().find(|(k, _)| *k == want).map(|(_, v)| v.as_str())
    }
}

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
    captured: Arc<Mutex<Vec<Captured>>>,
}

fn serve(status: u16, body: &'static str) -> MockServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip addr").port();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let looper = Arc::clone(&server);
    let sink = Arc::clone(&captured);
    std::thread::spawn(move || {
        for mut req in looper.incoming_requests() {
            let mut body_text = String::new();
            let _ = req.as_reader().read_to_string(&mut body_text);
            sink.lock().unwrap().push(Captured {
                method: req.method().to_string(),
                path: req.url().to_string(),
                headers: req
                    .headers()
                    .iter()
                    .map(|h| (h.field.to_string().to_ascii_lowercase(), h.value.to_string()))
                    .collect(),
                body: body_text,
            });
            let response = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                );
            let _ = req.respond(response);
        }
    });
    MockServer {
        server,
        base: format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo"),
        captured,
    }
}

impl MockServer {
    fn last(&self) -> Captured {
        self.captured.lock().unwrap().last().expect("a captured request").clone()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

fn client(mock: &MockServer) -> Client {
    Client::new(&mock.base, "tok", Some("backend"))
}

fn assert_call(cap: &Captured, method: &str, path_suffix: &str) {
    assert_eq!(cap.method, method);
    assert!(
        cap.path.ends_with(path_suffix),
        "path was {}, wanted suffix {path_suffix}",
        cap.path
    );
}

// --- read verbs: typed responses off the wire samples ---

#[test]
fn list_changes_returns_typed_summaries() {
    let mock = serve(
        200,
        r#"{"changes":[{"name":"demo","summary":"Demo change summary","status":"done","completedTasks":2,"totalTasks":2,"repo":"backend","lifecycle":"applying","claimedBy":"me"}]}"#,
    );
    let resp = client(&mock).list_changes().expect("list ok");
    assert_eq!(resp.changes.len(), 1);
    assert_eq!(resp.changes[0].name, "demo");
    assert_eq!(resp.changes[0].completed_tasks, 2);
    assert_eq!(resp.changes[0].repo.as_deref(), Some("backend"));
    assert_call(&mock.last(), "GET", "/changes");
}

#[test]
fn get_change_returns_the_typed_status() {
    let mock = serve(
        200,
        r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":true,"applyRequires":["tasks"],"artifacts":[{"id":"proposal","outputPath":"proposal.md","status":"done","version":3}],"repo":"backend","lifecycle":"applying","statusVersion":4,"claimedBy":"me"}"#,
    );
    let status = client(&mock).get_change("demo").expect("status ok");
    assert_eq!(status.change_name, "demo");
    assert_eq!(status.artifacts[0].version, Some(3));
    assert_call(&mock.last(), "GET", "/changes/demo");
}

#[test]
fn instructions_split_into_apply_and_artifact_typed_calls() {
    let mock = serve(
        200,
        r#"{"changeName":"demo","changeDir":"changes/demo","schemaName":"spec-driven","contextFiles":{"tasks":"tasks.md"},"progress":{"total":2,"complete":2,"remaining":0},"tasks":[{"id":"1","description":"1.1 First","done":true,"parallel":false}],"state":"all_done","locale":"English","instruction":"Work through the tasks.\n"}"#,
    );
    let apply = client(&mock).apply_instructions("demo").expect("apply ok");
    assert_eq!(apply.state, "all_done");
    assert_eq!(apply.progress.remaining, 0);
    assert_call(&mock.last(), "GET", "/changes/demo/instructions/apply");

    let mock2 = serve(
        200,
        r###"{"changeName":"demo","artifactId":"proposal","schemaName":"spec-driven","changeDir":"changes/demo","outputPath":"proposal.md","description":"Initial proposal document outlining the change","instruction":"Create the proposal.\n","locale":"English","template":"## Why\n","dependencies":[],"unlocks":["design"]}"###,
    );
    let artifact = client(&mock2)
        .artifact_instructions("demo", "proposal")
        .expect("artifact ok");
    assert_eq!(artifact.artifact_id, "proposal");
    assert_eq!(artifact.unlocks, ["design"]);
    assert_call(&mock2.last(), "GET", "/changes/demo/instructions/proposal");
}

#[test]
fn get_artifact_returns_typed_content_and_version() {
    let mock = serve(
        200,
        r###"{"artifact":"design","content":"## Context\n","version":8}"###,
    );
    let got = client(&mock).get_artifact("demo", "design").expect("artifact ok");
    assert_eq!(got.content, "## Context\n");
    assert_eq!(got.version, 8);
    assert_call(&mock.last(), "GET", "/changes/demo/artifacts/design");
}

/// The drift response arrives as protocol types and maps back to the Engine's
/// spec-side report through the Host's single mapping — the merger downstream
/// only ever sees Engine types.
#[test]
fn spec_drift_returns_typed_report_and_basis() {
    let mock = serve(
        200,
        r###"{"specDrift":{"dimension":{"kind":"Specs","status":"1 stale assumptions","score":4,"contributesToTotal":true},"specAssumptions":[{"capability":"auth","operation":"MODIFIED","requirement":"Rotate tokens","reason":"target requirement no longer exists in the canonical spec"}]},"basis":{"spec":"sha256:aaa","tasks":"sha256:bbb","policy":"sha256:ccc"},"change":{"created":"2026-07-13","design":"## Context\n","tasks":"- [ ] 1.1 First\n"}}"###,
    );
    let response = client(&mock).spec_drift("demo").expect("drift ok");
    assert_call(&mock.last(), "GET", "/changes/demo/drift");

    assert_eq!(response.spec_drift.dimension.kind, "Specs");
    assert_eq!(response.spec_drift.dimension.score, 4);
    assert_eq!(response.spec_drift.spec_assumptions[0].capability, "auth");
    assert_eq!(response.basis.spec, "sha256:aaa");
    assert_eq!(response.change.created.as_deref(), Some("2026-07-13"));
    assert_eq!(response.change.design.as_deref(), Some("## Context\n"));

    // Back to Engine types via the Host's mapping — what the merger consumes.
    let report = speclink_host::drift::spec_drift_from_wire(&response.spec_drift);
    assert_eq!(report.dimension.kind, "Specs");
    assert!(report.dimension.contributes_to_total);
    assert_eq!(report.spec_assumptions.len(), 1);
    assert_eq!(report.spec_assumptions[0].requirement, "Rotate tokens");

    let basis = speclink_host::drift::basis_from_wire(&response.basis);
    assert_eq!(basis.tasks, "sha256:bbb");
}

#[test]
fn list_specs_language_config_and_whoami_are_typed() {
    let mock = serve(
        200,
        r#"{"specs":[{"id":"user-auth","path":"specs/user-auth/spec.md"}]}"#,
    );
    let specs = client(&mock).list_specs().expect("specs ok");
    assert_eq!(specs.specs[0].id, "user-auth");
    assert_call(&mock.last(), "GET", "/specs");

    let mock2 = serve(200, r###"{"content":"# Language\n"}"###);
    let language = client(&mock2).language().expect("language ok");
    assert_eq!(language.content, "# Language\n");
    assert_call(&mock2.last(), "GET", "/language");

    let mock3 = serve(200, r#"{"schema":"spec-driven"}"#);
    let config = client(&mock3).config().expect("config ok");
    assert_eq!(config.schema, "spec-driven");
    assert_call(&mock3.last(), "GET", "/config");

    let mock4 = serve(
        200,
        r#"{"user":{"name":"王小明","handle":"ming"},"repos":[{"name":"backend","gitUrl":"https://git.example.com/erp.git"}]}"#,
    );
    let whoami = client(&mock4).whoami().expect("whoami ok");
    assert_eq!(whoami.user.handle, "ming");
    assert_eq!(whoami.repos[0].name, "backend");
    assert_call(&mock4.last(), "GET", "/whoami");
}

#[test]
fn discussion_reads_are_typed() {
    let mock = serve(
        200,
        r#"{"discussions":[{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","path":"discussions/demo-topic.md","archived":false}]}"#,
    );
    let list = client(&mock).list_discussions(false).expect("list ok");
    assert_eq!(list.discussions[0].slug, "demo-topic");
    assert_call(&mock.last(), "GET", "/discussions");

    let archived = client(&mock).list_discussions(true).expect("archived ok");
    assert_eq!(archived.discussions.len(), 1);
    assert!(
        mock.last().path.ends_with("/discussions?archived=true"),
        "archived filter travels as the query string: {}",
        mock.last().path
    );

    let mock2 = serve(
        200,
        r###"{"info":{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","path":"discussions/demo-topic.md","archived":false},"content":"# Discussion: Demo topic\n"}"###,
    );
    let show = client(&mock2).show_discussion("demo-topic").expect("show ok");
    assert_eq!(show.info.topic, "Demo topic");
    assert_eq!(show.content, "# Discussion: Demo topic\n");
    assert_call(&mock2.last(), "GET", "/discussions/demo-topic");
}

// --- write verbs: bodies are exactly the protocol DTO serializations ---

#[test]
fn create_change_posts_the_typed_request_body() {
    let mock = serve(
        201,
        r#"{"name":"demo","schema":"spec-driven","repo":"backend","lifecycle":"drafting"}"#,
    );
    let req = CreateChangeRequest {
        name: "demo".into(),
        schema: Some("spec-driven".into()),
        description: None,
        agent: Some("claude".into()),
        from_discussion: None,
    };
    let resp = client(&mock).create_change(req.clone()).expect("create ok");
    assert_eq!(resp.name, "demo");
    assert_eq!(resp.schema.as_deref(), Some("spec-driven"));
    let cap = mock.last();
    assert_call(&cap, "POST", "/changes");
    assert_eq!(
        cap.body,
        serde_json::to_string(&req).unwrap(),
        "the wire body is the DTO serialization, nothing re-assembled"
    );
}

#[test]
fn put_artifact_carries_if_match_and_the_content_envelope() {
    let mock = serve(200, r#"{"artifact":"design","version":8}"#);
    let resp = client(&mock)
        .put_artifact("demo", "design", "## Context\n", 7)
        .expect("put ok");
    assert_eq!(resp.version, 8);
    let cap = mock.last();
    assert_call(&cap, "PUT", "/changes/demo/artifacts/design");
    assert_eq!(cap.header("if-match"), Some("7"));
    assert_eq!(
        cap.body,
        serde_json::to_string(&PutArtifactRequest { content: "## Context\n".into() }).unwrap()
    );
}

#[test]
fn stale_write_surfaces_revision_conflict_with_the_existing_wording() {
    let mock = serve(
        409,
        r#"{"status":409,"reason":"revision_conflict","message":"expected 3, at 7"}"#,
    );
    let err = client(&mock)
        .put_artifact("demo", "design", "content", 3)
        .unwrap_err();
    assert_eq!(mock.last().header("if-match"), Some("3"));
    assert_eq!(err.reason.as_deref(), Some("revision_conflict"));
    assert_eq!(
        err.message, "content changed since you read it — re-read it and re-apply your edit",
        "byte-identical to the current 409 conflict wording"
    );
}

#[test]
fn task_verbs_post_typed_bodies() {
    let mock = serve(200, r#"{"taskDesc":"1.1 First","alreadyDone":false}"#);
    let done = client(&mock)
        .task_done("demo", "3", &["src/lib.rs".to_string()])
        .expect("done ok");
    assert_eq!(done.task_desc, "1.1 First");
    assert!(!done.already_done);
    let cap = mock.last();
    assert_call(&cap, "POST", "/changes/demo/tasks/3/done");
    assert_eq!(
        cap.body,
        serde_json::to_string(&TaskDoneRequest { touched_files: vec!["src/lib.rs".into()] })
            .unwrap()
    );

    let empty = client(&mock).task_done("demo", "4", &[]).expect("done ok");
    assert!(!empty.already_done);
    assert_eq!(mock.last().body, "{}", "empty attribution stays the bare object");

    let mock2 = serve(200, r#"{"taskDesc":"1.1 First","alreadyUndone":true}"#);
    let undone = client(&mock2).task_undone("demo", "3").expect("undone ok");
    assert!(undone.already_undone);
    let cap2 = mock2.last();
    assert_call(&cap2, "POST", "/changes/demo/tasks/3/undone");
    assert_eq!(cap2.body, "{}", "unchecking never records touched files");
}

#[test]
fn claim_and_archive_return_typed_responses() {
    let mock = serve(200, r#"{"lifecycle":"applying","claimedBy":"me"}"#);
    let claim = client(&mock).claim("demo").expect("claim ok");
    assert_eq!(claim.claimed_by.as_deref(), Some("me"));
    let cap = mock.last();
    assert_call(&cap, "POST", "/changes/demo/claim");
    assert_eq!(cap.body, "{}");

    let mock2 = serve(200, r#"{"specs":[{"capability":"user-auth"}]}"#);
    let archive = client(&mock2).archive("demo", false).expect("archive ok");
    assert_eq!(archive.specs[0].capability, "user-auth");
    assert_call(&mock2.last(), "POST", "/changes/demo/archive?carryReview=false");

    // 帶未結審查工單封存（D5 第三處置）：旗標須真的上 wire，否則 remote 只剩兩條出路。
    let mock3 = serve(200, r#"{"specs":[]}"#);
    client(&mock3).archive("demo", true).expect("carry-review archive ok");
    assert_call(&mock3.last(), "POST", "/changes/demo/archive?carryReview=true");
}

#[test]
fn discussion_writes_post_typed_bodies() {
    let mock = serve(
        201,
        r#"{"slug":"auth-scope","topic":"Auth scope","path":"discussions/auth-scope.md"}"#,
    );
    let created = client(&mock).new_discussion("Auth scope", None).expect("new ok");
    assert_eq!(created.slug, "auth-scope");
    let cap = mock.last();
    assert_call(&cap, "POST", "/discussions");
    assert_eq!(
        cap.body,
        serde_json::to_string(&CreateDiscussionRequest {
            topic: "Auth scope".into(),
            slug: None,
        })
        .unwrap(),
        "no override keeps the body byte-identical to the pre-slug client's"
    );

    let mock2 = serve(200, r#"{"round":3}"#);
    let round = client(&mock2)
        .discussion_add_round("auth-scope", "assumptions", "…")
        .expect("round ok");
    assert_eq!(round.round, 3);
    let cap2 = mock2.last();
    assert_call(&cap2, "POST", "/discussions/auth-scope/rounds");
    assert_eq!(
        cap2.body,
        serde_json::to_string(&AddDiscussionRoundRequest {
            mode: "assumptions".into(),
            content: "…".into(),
        })
        .unwrap()
    );

    let mock3 = serve(200, "{}");
    client(&mock3)
        .discussion_context("auth-scope", "context body")
        .expect("context ok");
    assert_call(&mock3.last(), "PUT", "/discussions/auth-scope/context");
    client(&mock3)
        .discussion_conclude("auth-scope", "the conclusion")
        .expect("conclude ok");
    assert_call(&mock3.last(), "POST", "/discussions/auth-scope/conclude");

    let mock4 = serve(200, r#"{"archivedTo":"discussions/archive/auth-scope.md"}"#);
    let archived = client(&mock4).discussion_archive("auth-scope").expect("archive ok");
    assert_eq!(archived.archived_to, "discussions/archive/auth-scope.md");

    let mock5 = serve(200, r#"{"change":"add-auth"}"#);
    let promoted = client(&mock5)
        .discussion_promote("auth-scope", Some("add-auth"))
        .expect("promote ok");
    assert_eq!(promoted.change, "add-auth");
    assert_eq!(
        mock5.last().body,
        serde_json::to_string(&PromoteDiscussionRequest { name: Some("add-auth".into()) })
            .unwrap()
    );
    let bare = client(&mock5).discussion_promote("auth-scope", None).expect("promote ok");
    assert_eq!(bare.change, "add-auth");
    assert_eq!(mock5.last().body, "{}", "no explicit name posts the bare object");
}

#[test]
fn discussion_parity_verbs_post_typed_bodies() {
    // new_discussion 攜帶 slug 覆寫（verb-contract：remote 建立討論帶 slug）。
    let mock = serve(
        201,
        r#"{"slug":"board-search-bar","topic":"看板搜尋列","path":"discussions/board-search-bar.md"}"#,
    );
    let created = client(&mock)
        .new_discussion("看板搜尋列", Some("board-search-bar"))
        .expect("new with slug ok");
    assert_eq!(created.slug, "board-search-bar");
    let cap = mock.last();
    assert_call(&cap, "POST", "/discussions");
    assert_eq!(
        cap.body,
        serde_json::to_string(&CreateDiscussionRequest {
            topic: "看板搜尋列".into(),
            slug: Some("board-search-bar".into()),
        })
        .unwrap()
    );

    // discard：force 走 query 參數（鏡射 change 側 DELETE）。
    let mock2 = serve(200, r#"{"slug":"board-search-bar"}"#);
    let discarded = client(&mock2)
        .discard_discussion("board-search-bar", false)
        .expect("discard ok");
    assert_eq!(discarded.slug, "board-search-bar");
    let cap2 = mock2.last();
    assert_call(&cap2, "DELETE", "/discussions/board-search-bar?force=false");
    assert_eq!(cap2.body, "", "force travels as the query parameter, not a body");
    client(&mock2)
        .discard_discussion("board-search-bar", true)
        .expect("force discard ok");
    assert_call(&mock2.last(), "DELETE", "/discussions/board-search-bar?force=true");

    // link 與 seal：body 帶 change 名稱。
    let mock3 = serve(200, r#"{"slug":"auth-scope","change":"add-auth"}"#);
    let linked = client(&mock3).link_discussion("auth-scope", "add-auth").expect("link ok");
    assert_eq!((linked.slug.as_str(), linked.change.as_str()), ("auth-scope", "add-auth"));
    let cap3 = mock3.last();
    assert_call(&cap3, "POST", "/discussions/auth-scope/link");
    assert_eq!(
        cap3.body,
        serde_json::to_string(&BindDiscussionRequest { change: "add-auth".into() }).unwrap()
    );
    let sealed = client(&mock3).seal_discussion("auth-scope", "add-auth").expect("seal ok");
    assert_eq!(sealed.change, "add-auth");
    assert_call(&mock3.last(), "POST", "/discussions/auth-scope/seal");

    // in-progress：空 body POST，回應內容無人消費。
    let mock4 = serve(200, "{}");
    client(&mock4).in_progress_add("demo").expect("in-progress ok");
    let cap4 = mock4.last();
    assert_call(&cap4, "POST", "/changes/demo/in-progress");
    assert_eq!(cap4.body, "{}");
}

#[test]
fn in_progress_remove_deletes_the_marker_resource() {
    // 與 add 同資源、反向方法(D4):無 body 的 DELETE,200 Ack 無人消費。
    let mock = serve(200, "{}");
    client(&mock).in_progress_remove("demo").expect("in-progress remove ok");
    assert_call(&mock.last(), "DELETE", "/changes/demo/in-progress");
}

#[test]
fn in_progress_remove_409_evidence_deserializes_structurally() {
    // 守門 409:flatten 進錯誤封套的 camelCase 證據欄位反序列化為
    // RemoteError::evidence,message 逐字轉發(fs parity)。
    let mock = serve(
        409,
        r#"{"status":409,"reason":"refused","message":"cannot remove the in-progress marker for 'demo': work traces exist","checkedTasks":2,"touchedFiles":["src/a.rs","src/b.ts"]}"#,
    );
    let err = client(&mock).in_progress_remove("demo").unwrap_err();
    assert_eq!(err.reason.as_deref(), Some("refused"));
    assert_eq!(
        err.message,
        "cannot remove the in-progress marker for 'demo': work traces exist",
        "engine-class message relayed verbatim"
    );
    let evidence = err.evidence.expect("the 409 carries structured evidence");
    assert_eq!(evidence.checked_tasks, 2);
    assert_eq!(evidence.touched_files, vec!["src/a.rs", "src/b.ts"]);
}

#[test]
fn discussion_parity_verbs_map_a_404_to_the_typed_error() {
    // 未升級的舊 server 對新動詞回 404 → 語義化 RemoteError，不 panic。
    let mock = serve(
        404,
        r#"{"status":404,"reason":"not_found","message":"discussion 'no-such' not found"}"#,
    );
    let err = client(&mock).discard_discussion("no-such", false).unwrap_err();
    assert_eq!(err.reason.as_deref(), Some("not_found"));
    assert_eq!(
        err.message, "discussion 'no-such' not found",
        "engine-class message relayed verbatim"
    );
    let err = client(&mock).link_discussion("no-such", "demo").unwrap_err();
    assert_eq!(err.reason.as_deref(), Some("not_found"));
    let err = client(&mock).seal_discussion("no-such", "demo").unwrap_err();
    assert_eq!(err.reason.as_deref(), Some("not_found"));
    let err = client(&mock).in_progress_add("demo").unwrap_err();
    assert_eq!(err.reason.as_deref(), Some("not_found"));
}

// --- the registry mapping table, byte for byte (design decision three) ---

#[test]
fn every_registry_reason_maps_to_the_frozen_cli_message() {
    // (status, wire body, expected message, expected reason)
    let cases: Vec<(u16, &str, String, Option<&str>)> = vec![
        // Connection-layer wording is client-owned, byte-identical to the
        // current translation table.
        (
            409,
            r#"{"status":409,"reason":"revision_conflict","message":"stale"}"#,
            "content changed since you read it — re-read it and re-apply your edit".into(),
            Some("revision_conflict"),
        ),
        (
            401,
            r#"{"status":401,"reason":"permission_denied","message":"bad token"}"#,
            "authentication failed — run `speclink auth login`".into(),
            Some("permission_denied"),
        ),
        (
            403,
            r#"{"status":403,"reason":"permission_denied","message":"no access"}"#,
            "access denied — your account has no access to this project; ask a project admin"
                .into(),
            Some("permission_denied"),
        ),
        (
            400,
            r#"{"status":400,"reason":"internal","message":"if-match required"}"#,
            "internal speclink error — update speclink or report a bug".into(),
            Some("internal"),
        ),
        // Engine-class reasons relay the server's message verbatim, the way
        // fs mode prints engine messages.
        (
            404,
            r#"{"status":404,"reason":"not_found","message":"Change 'ghost' not found."}"#,
            "Change 'ghost' not found.".into(),
            Some("not_found"),
        ),
        (
            400,
            r#"{"status":400,"reason":"invalid_argument","message":"Unknown artifact type 'blueprint'."}"#,
            "Unknown artifact type 'blueprint'.".into(),
            Some("invalid_argument"),
        ),
        (
            400,
            r#"{"status":400,"reason":"invalid_config","message":"this project has multiple repos — set `remote.repo` in .speclink.yaml (candidates: backend, frontend)"}"#,
            "this project has multiple repos — set `remote.repo` in .speclink.yaml (candidates: backend, frontend)".into(),
            Some("invalid_config"),
        ),
        (
            409,
            r#"{"status":409,"reason":"refused","message":"change is held by chiang — coordinate, or re-claim if it was released"}"#,
            "change is held by chiang — coordinate, or re-claim if it was released".into(),
            Some("refused"),
        ),
        // Unknown reasons stay a generic error with the status for bug
        // reports only — never a panic.
        (
            418,
            r#"{"status":418,"reason":"im_a_teapot","message":"short and stout"}"#,
            "unexpected server response — update speclink or report a bug (HTTP 418)".into(),
            Some("im_a_teapot"),
        ),
    ];
    for (status, body, want_message, want_reason) in cases {
        let mock = serve(status, Box::leak(body.to_string().into_boxed_str()));
        let err = client(&mock).list_changes().unwrap_err();
        assert_eq!(err.message, want_message, "message frozen for {body}");
        assert_eq!(err.reason.as_deref(), want_reason, "reason kept for {body}");
    }
}

#[test]
fn unavailable_covers_every_5xx_before_the_reason_table() {
    let mock = serve(503, r#"{"status":503,"reason":"unavailable","message":"maintenance"}"#);
    let err = client(&mock).list_changes().unwrap_err();
    assert_eq!(
        err.message,
        "server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)"
    );
}

// --- context snapshot: the two-valued conditional read ---

#[test]
fn context_snapshot_fresh_posts_the_request_and_returns_the_snapshot() {
    let mock = serve(
        200,
        r#"{"snapshotId":"snap-5","policyRevision":1,"digest":"sha256:aa","documents":[{"path":"openspec/config.yaml","content":"schema: spec-driven\n","revision":1,"digest":"sha256:bb"}]}"#,
    );
    let request = ContextSnapshotRequest { change: Some("demo".into()), flow: Some("apply".into()) };
    let outcome = client(&mock).context_snapshot(&request, None).expect("snapshot ok");
    match outcome {
        ContextSnapshotOutcome::Fresh(snap) => {
            assert_eq!(snap.snapshot_id, "snap-5");
            assert_eq!(snap.documents.len(), 1);
            assert_eq!(snap.documents[0].path, "openspec/config.yaml");
        }
        ContextSnapshotOutcome::Unchanged => panic!("expected a fresh snapshot, got Unchanged"),
    }
    let cap = mock.last();
    assert_call(&cap, "POST", "/context");
    assert_eq!(
        cap.body,
        serde_json::to_string(&request).unwrap(),
        "the wire body is the request DTO serialization, nothing re-assembled"
    );
    assert_eq!(cap.header("if-none-match"), None, "no known id → no If-None-Match header");
}

#[test]
fn context_snapshot_sends_if_none_match_and_304_is_unchanged() {
    let mock = serve(304, "");
    let request = ContextSnapshotRequest { change: Some("demo".into()), flow: Some("apply".into()) };
    let outcome = client(&mock)
        .context_snapshot(&request, Some("snap-5"))
        .expect("a 304 is a normal outcome, not an error");
    assert!(matches!(outcome, ContextSnapshotOutcome::Unchanged), "a 304 is the Unchanged value");
    let cap = mock.last();
    assert_call(&cap, "POST", "/context");
    assert_eq!(
        cap.header("if-none-match"),
        Some("snap-5"),
        "the caller's known snapshot id travels as If-None-Match"
    );
}

#[test]
fn context_snapshot_translates_a_503_like_any_verb() {
    let mock = serve(503, r#"{"status":503,"reason":"unavailable","message":"maintenance"}"#);
    let request = ContextSnapshotRequest { change: None, flow: None };
    let err = client(&mock).context_snapshot(&request, None).unwrap_err();
    assert_eq!(
        err.message,
        "server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)",
        "a 5xx collapses to the frozen unavailable message"
    );
}
