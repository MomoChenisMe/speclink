//! The JS Store bridge: adapts a host JavaScript Store object (methods may
//! return plain values or Promises) to the engine's synchronous `Store` trait.
//!
//! Mechanics: each trait call sends a `CallRequest` through one
//! ThreadsafeFunction to the JS thread, where `index.js`'s invoker executes
//! `store[method](...args)` and settles the outcome back through an mpsc
//! channel; the engine worker thread blocks on `recv()` until the JS Promise
//! resolves. Because `dispatch` never runs synchronously on the JS thread,
//! this wait cannot deadlock against the callback (see design decision 1).

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{
    ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi::{JsFunction, JsObject, JsUnknown, ValueType};
use speclink_core::model::{Change, ChangeMeta};
use speclink_core::store::{DiscussionDoc, Store};
use std::path::PathBuf;
use std::sync::mpsc;

/// The Store interface, one-to-one with `speclink_core::store::Store`
/// (camelCase, per the npm API naming convention). Validated at construction:
/// a missing method fails `createEngine` before an engine instance exists.
pub const REQUIRED_METHODS: &[&str] = &[
    // changes
    "listChanges",
    "findChange",
    "changeExists",
    "createChange",
    "updatedAtSecs",
    // artifacts
    "readArtifact",
    "writeArtifact",
    "artifactExists",
    // delta specs
    "deltaCapabilities",
    "hasCapabilityDirs",
    // canonical specs
    "listCanonicalCapabilities",
    "canonicalSpecExists",
    "readCanonicalSpec",
    "writeCanonicalSpec",
    "canonicalSpecPath",
    // archive
    "archivedChangeExists",
    "archiveChange",
    "readArchivedMeta",
    "writeArchivedMeta",
    // discussions
    "liveDiscussionExists",
    "archivedDiscussionExists",
    "liveDiscussionPath",
    "readLiveDiscussion",
    "writeLiveDiscussion",
    "deleteLiveDiscussion",
    "readDiscussion",
    "listLiveDiscussions",
    "listArchivedDiscussions",
    "archiveDiscussion",
    // workflow config / shared vocabulary
    "readWorkflowConfig",
    "readLanguage",
];

/// A store-method failure (throw/reject on the JS side, or a shape the bridge
/// cannot convert). `Display` yields the contract's method-prefixed message.
#[derive(Debug, Clone)]
pub struct BridgeFailure {
    pub method: String,
    pub message: String,
    pub code: Option<String>,
}

impl BridgeFailure {
    fn new(method: &str, message: impl Into<String>) -> BridgeFailure {
        BridgeFailure {
            method: method.to_string(),
            message: message.into(),
            code: None,
        }
    }
}

impl std::fmt::Display for BridgeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.method, self.message)
    }
}

impl std::error::Error for BridgeFailure {}

/// One bridged call, sent from an engine worker thread to the JS thread.
pub struct CallRequest {
    method: &'static str,
    /// JSON array of the method's arguments.
    args: serde_json::Value,
    tx: mpsc::Sender<std::result::Result<serde_json::Value, BridgeFailure>>,
}

/// The invoker's error argument (built by `index.js` from the JS throw/reject).
#[derive(serde::Deserialize)]
struct SettleError {
    message: String,
    code: Option<String>,
}

/// Fail `createEngine` when the store object is missing interface methods,
/// listing every missing name (fail fast, before any engine exists).
pub fn validate_store_methods(store: &JsObject) -> Result<()> {
    let mut missing = Vec::new();
    for name in REQUIRED_METHODS {
        let is_fn = store
            .get_named_property_unchecked::<JsUnknown>(name)
            .and_then(|v| v.get_type())
            .map(|t| t == ValueType::Function)
            .unwrap_or(false);
        if !is_fn {
            missing.push(*name);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(Error::from_reason(format!(
        "createEngine: store is missing required Store methods: {}",
        missing.join(", ")
    )))
}

/// The bridged store: `Clone` hands each dispatch worker its own handle to the
/// single underlying ThreadsafeFunction.
#[derive(Clone)]
pub struct JsStoreBridge {
    tsfn: ThreadsafeFunction<CallRequest, ErrorStrategy::Fatal>,
}

/// Build the bridge over the JS invoker `(method, args, settle) => void`
/// (created by `index.js` with the store object bound in its closure).
pub fn create_bridge(env: Env, invoker: JsFunction) -> Result<JsStoreBridge> {
    let mut tsfn: ThreadsafeFunction<CallRequest, ErrorStrategy::Fatal> = invoker
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<CallRequest>| {
            let CallRequest { method, args, tx } = ctx.value;
            let env = ctx.env;
            let method_js = env.create_string(method)?;
            let args_js = env.to_js_value(&args)?;
            let settle = env.create_function_from_closure("settle", move |fctx| {
                let err = fctx.get::<JsUnknown>(0)?;
                let outcome = match err.get_type()? {
                    ValueType::Null | ValueType::Undefined => {
                        let value = fctx.get::<JsUnknown>(1)?;
                        match value.get_type()? {
                            ValueType::Null | ValueType::Undefined => Ok(serde_json::Value::Null),
                            _ => fctx
                                .env
                                .from_js_value::<serde_json::Value, JsUnknown>(value)
                                .map_err(|e| {
                                    BridgeFailure::new(
                                        method,
                                        format!("store method returned a non-JSON value: {e}"),
                                    )
                                }),
                        }
                    }
                    _ => {
                        let parsed = fctx
                            .env
                            .from_js_value::<SettleError, JsUnknown>(err)
                            .unwrap_or(SettleError {
                                message: "store method failed".to_string(),
                                code: None,
                            });
                        Err(BridgeFailure {
                            method: method.to_string(),
                            message: parsed.message,
                            code: parsed.code,
                        })
                    }
                };
                let _ = tx.send(outcome);
                fctx.env.get_undefined()
            })?;
            Ok(vec![
                method_js.into_unknown(),
                args_js,
                settle.into_unknown(),
            ])
        })?;
    // The bridge must not keep the event loop alive on its own — an in-flight
    // dispatch already does (its deferred), and an idle engine should let the
    // process exit.
    tsfn.unref(&env)?;
    Ok(JsStoreBridge { tsfn })
}

impl JsStoreBridge {
    /// Call a store method and wait (on the worker thread) for it to settle.
    fn call(
        &self,
        method: &'static str,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, BridgeFailure> {
        let (tx, rx) = mpsc::channel();
        let status = self.tsfn.call(
            CallRequest { method, args, tx },
            ThreadsafeFunctionCallMode::NonBlocking,
        );
        if status != Status::Ok {
            return Err(BridgeFailure::new(
                method,
                format!("store bridge unavailable ({status:?})"),
            ));
        }
        rx.recv().unwrap_or_else(|_| {
            Err(BridgeFailure::new(
                method,
                "store bridge channel closed before the store method settled",
            ))
        })
    }

    /// For trait methods without a `Result` return: a failure aborts the
    /// dispatch by unwinding to the dispatch boundary, where it is mapped to
    /// the contract's Error (method-prefixed message + code).
    fn call_ok(&self, method: &'static str, args: serde_json::Value) -> serde_json::Value {
        match self.call(method, args) {
            Ok(v) => v,
            Err(f) => std::panic::panic_any(f),
        }
    }

    fn call_result(
        &self,
        method: &'static str,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(method, args).map_err(anyhow::Error::new)
    }

    /// `claim` is not a core Store method — it is an OPTIONAL host extension
    /// (ownership is a team-system concept; the host adjudicates). Returns the
    /// host payload verbatim; a missing implementation carries the
    /// `__missing__` marker code for the dispatcher to translate.
    pub fn claim(&self, name: &str) -> std::result::Result<serde_json::Value, BridgeFailure> {
        self.call("claim", serde_json::json!([name]))
    }

    fn invalid_shape(&self, method: &'static str, expected: &str) -> ! {
        std::panic::panic_any(BridgeFailure::new(
            method,
            format!("store method returned an invalid shape (expected {expected})"),
        ))
    }
}

// --- JS value → domain conversions ---

fn change_from_value(v: &serde_json::Value) -> Option<Change> {
    let obj = v.as_object()?;
    let name = obj.get("name")?.as_str()?.to_string();
    let dir = obj
        .get("dir")
        .and_then(|d| d.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("changes/{name}")));
    let meta = obj.get("meta").and_then(|m| m.as_object());
    let get = |key: &str| -> Option<String> {
        meta.and_then(|m| m.get(key)).and_then(|x| x.as_str()).map(str::to_string)
    };
    Some(Change {
        name,
        dir,
        meta: ChangeMeta {
            schema: get("schema"),
            created: get("created"),
            created_by: get("createdBy"),
            created_with: get("createdWith"),
            from_discussion: get("fromDiscussion"),
            restale_from: get("restaleFrom"),
            started_at: get("startedAt"),
            started_by: get("startedBy"),
            started_with: get("startedWith"),
            board_rank: get("boardRank"),
        },
    })
}

fn discussion_from_value(v: &serde_json::Value) -> Option<DiscussionDoc> {
    let obj = v.as_object()?;
    Some(DiscussionDoc {
        slug: obj.get("slug")?.as_str()?.to_string(),
        text: obj.get("text")?.as_str()?.to_string(),
        path: PathBuf::from(obj.get("path")?.as_str()?),
        archived: obj.get("archived").and_then(|a| a.as_bool()).unwrap_or(false),
    })
}

fn opt_string(v: serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s),
        _ => None,
    }
}

impl Store for JsStoreBridge {
    // --- changes ---

    fn list_changes(&self) -> Vec<Change> {
        let m = "listChanges";
        match self.call_ok(m, serde_json::json!([])) {
            serde_json::Value::Array(items) => items
                .iter()
                .map(|v| change_from_value(v).unwrap_or_else(|| self.invalid_shape(m, "a change object with a string 'name'")))
                .collect(),
            serde_json::Value::Null => Vec::new(),
            _ => self.invalid_shape(m, "an array of change objects"),
        }
    }

    fn find_change(&self, name: &str) -> Option<Change> {
        let v = self.call_ok("findChange", serde_json::json!([name]));
        if v.is_null() {
            return None;
        }
        Some(
            change_from_value(&v)
                .unwrap_or_else(|| self.invalid_shape("findChange", "a change object or null")),
        )
    }

    fn change_exists(&self, name: &str) -> bool {
        self.call_ok("changeExists", serde_json::json!([name]))
            .as_bool()
            .unwrap_or(false)
    }

    fn create_change(&self, name: &str, meta_text: &str) -> anyhow::Result<PathBuf> {
        let v = self.call_result("createChange", serde_json::json!([name, meta_text]))?;
        Ok(opt_string(v).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(format!("changes/{name}"))))
    }

    fn updated_at_secs(&self, name: &str) -> u64 {
        self.call_ok("updatedAtSecs", serde_json::json!([name]))
            .as_u64()
            .unwrap_or(0)
    }

    // The active-meta raw pair is an OPTIONAL host method (not in
    // REQUIRED_METHODS — existing JS stores keep validating): a store without
    // `readChangeMeta` reads as "no metadata"; a real JS failure still aborts
    // the dispatch like every other store method.
    fn read_change_meta(&self, name: &str) -> Option<String> {
        match self.call("readChangeMeta", serde_json::json!([name])) {
            Ok(v) => opt_string(v),
            Err(f) if f.code.as_deref() == Some("__missing__") => None,
            Err(f) => std::panic::panic_any(f),
        }
    }

    fn write_change_meta(&self, name: &str, content: &str) -> anyhow::Result<()> {
        self.call_result("writeChangeMeta", serde_json::json!([name, content]))?;
        Ok(())
    }

    // Like the change-meta raw pair, `deleteChange` is an OPTIONAL host method
    // (not in REQUIRED_METHODS — existing JS stores keep validating): a store
    // without it fails the call only when `discard` actually reaches it.
    fn delete_change(&self, name: &str) -> anyhow::Result<()> {
        self.call_result("deleteChange", serde_json::json!([name]))?;
        Ok(())
    }

    // --- artifacts ---

    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        opt_string(self.call_ok("readArtifact", serde_json::json!([change, artifact])))
    }

    fn write_artifact(&self, change: &str, artifact: &str, content: &str) -> anyhow::Result<PathBuf> {
        let v = self.call_result("writeArtifact", serde_json::json!([change, artifact, content]))?;
        Ok(opt_string(v)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("changes/{change}/{artifact}"))))
    }

    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.call_ok("artifactExists", serde_json::json!([change, artifact]))
            .as_bool()
            .unwrap_or(false)
    }

    // --- delta specs ---

    fn delta_capabilities(&self, change: &str) -> Vec<String> {
        let m = "deltaCapabilities";
        match self.call_ok(m, serde_json::json!([change])) {
            serde_json::Value::Array(items) => items
                .into_iter()
                .map(|v| opt_string(v).unwrap_or_else(|| self.invalid_shape(m, "an array of capability names")))
                .collect(),
            serde_json::Value::Null => Vec::new(),
            _ => self.invalid_shape(m, "an array of capability names"),
        }
    }

    fn has_capability_dirs(&self, change: &str) -> bool {
        self.call_ok("hasCapabilityDirs", serde_json::json!([change]))
            .as_bool()
            .unwrap_or(false)
    }

    // --- canonical specs ---

    fn list_canonical_capabilities(&self) -> Vec<String> {
        let m = "listCanonicalCapabilities";
        match self.call_ok(m, serde_json::json!([])) {
            serde_json::Value::Array(items) => items
                .into_iter()
                .map(|v| opt_string(v).unwrap_or_else(|| self.invalid_shape(m, "an array of capability names")))
                .collect(),
            serde_json::Value::Null => Vec::new(),
            _ => self.invalid_shape(m, "an array of capability names"),
        }
    }

    fn canonical_spec_exists(&self, cap: &str) -> bool {
        self.call_ok("canonicalSpecExists", serde_json::json!([cap]))
            .as_bool()
            .unwrap_or(false)
    }

    fn read_canonical_spec(&self, cap: &str) -> Option<String> {
        opt_string(self.call_ok("readCanonicalSpec", serde_json::json!([cap])))
    }

    fn write_canonical_spec(&self, cap: &str, content: &str) -> anyhow::Result<()> {
        self.call_result("writeCanonicalSpec", serde_json::json!([cap, content]))?;
        Ok(())
    }

    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        opt_string(self.call_ok("canonicalSpecPath", serde_json::json!([cap])))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("specs/{cap}/spec.md")))
    }

    // --- archive ---

    fn archived_change_exists(&self, dated_name: &str) -> bool {
        self.call_ok("archivedChangeExists", serde_json::json!([dated_name]))
            .as_bool()
            .unwrap_or(false)
    }

    fn archive_change(&self, name: &str, dated_name: &str) -> anyhow::Result<()> {
        self.call_result("archiveChange", serde_json::json!([name, dated_name]))?;
        Ok(())
    }

    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        opt_string(self.call_ok("readArchivedMeta", serde_json::json!([dated_name])))
    }

    fn write_archived_meta(&self, dated_name: &str, content: &str) -> anyhow::Result<()> {
        self.call_result("writeArchivedMeta", serde_json::json!([dated_name, content]))?;
        Ok(())
    }

    // --- discussions ---

    fn live_discussion_exists(&self, slug: &str) -> bool {
        self.call_ok("liveDiscussionExists", serde_json::json!([slug]))
            .as_bool()
            .unwrap_or(false)
    }

    fn archived_discussion_exists(&self, slug: &str) -> bool {
        self.call_ok("archivedDiscussionExists", serde_json::json!([slug]))
            .as_bool()
            .unwrap_or(false)
    }

    fn live_discussion_path(&self, slug: &str) -> PathBuf {
        opt_string(self.call_ok("liveDiscussionPath", serde_json::json!([slug])))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("discussions/{slug}.md")))
    }

    fn read_live_discussion(&self, slug: &str) -> Option<String> {
        opt_string(self.call_ok("readLiveDiscussion", serde_json::json!([slug])))
    }

    fn write_live_discussion(&self, slug: &str, content: &str) -> anyhow::Result<PathBuf> {
        let v = self.call_result("writeLiveDiscussion", serde_json::json!([slug, content]))?;
        Ok(opt_string(v)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("discussions/{slug}.md"))))
    }

    fn delete_live_discussion(&self, slug: &str) -> anyhow::Result<()> {
        self.call_result("deleteLiveDiscussion", serde_json::json!([slug]))?;
        Ok(())
    }

    fn read_discussion(&self, slug: &str) -> Option<DiscussionDoc> {
        let v = self.call_ok("readDiscussion", serde_json::json!([slug]));
        if v.is_null() {
            return None;
        }
        Some(discussion_from_value(&v).unwrap_or_else(|| {
            self.invalid_shape("readDiscussion", "{slug, text, path, archived} or null")
        }))
    }

    fn list_live_discussions(&self) -> Vec<DiscussionDoc> {
        let m = "listLiveDiscussions";
        match self.call_ok(m, serde_json::json!([])) {
            serde_json::Value::Array(items) => items
                .iter()
                .map(|v| {
                    discussion_from_value(v)
                        .unwrap_or_else(|| self.invalid_shape(m, "an array of {slug, text, path, archived}"))
                })
                .collect(),
            serde_json::Value::Null => Vec::new(),
            _ => self.invalid_shape(m, "an array of discussion objects"),
        }
    }

    fn list_archived_discussions(&self) -> Vec<DiscussionDoc> {
        let m = "listArchivedDiscussions";
        match self.call_ok(m, serde_json::json!([])) {
            serde_json::Value::Array(items) => items
                .iter()
                .map(|v| {
                    discussion_from_value(v)
                        .unwrap_or_else(|| self.invalid_shape(m, "an array of {slug, text, path, archived}"))
                })
                .collect(),
            serde_json::Value::Null => Vec::new(),
            _ => self.invalid_shape(m, "an array of discussion objects"),
        }
    }

    fn archive_discussion(&self, slug: &str, created: &str) -> anyhow::Result<Option<String>> {
        let v = self.call_result("archiveDiscussion", serde_json::json!([slug, created]))?;
        Ok(opt_string(v))
    }

    // --- workflow config / shared vocabulary ---

    fn read_workflow_config(&self) -> Option<String> {
        opt_string(self.call_ok("readWorkflowConfig", serde_json::json!([])))
    }

    fn read_language(&self) -> Option<String> {
        opt_string(self.call_ok("readLanguage", serde_json::json!([])))
    }
}
