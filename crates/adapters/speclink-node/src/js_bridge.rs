//! Calls from the native side into JavaScript: one function of `index.js`
//! (`makeInvoker`) runs a named function of the host — a store port's `list`,
//! `read` or `commit`, a hook, a conformance harness function — and settles
//! its outcome back.
//!
//! Mechanics: each call sends a `CallRequest` through one ThreadsafeFunction
//! to the JS thread, where the invoker runs `target[method](...args)` (a plain
//! value or a Promise) and settles the outcome back through an mpsc channel;
//! the worker thread blocks on `recv()` until it settles. Because a dispatch
//! never runs synchronously on the JS thread, this wait cannot deadlock
//! against the callback.

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{
    ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi::{JsFunction, JsUnknown, ValueType};
use std::sync::mpsc;

/// A failed call (throw/reject on the JS side, or a shape the bridge
/// cannot convert). `Display` yields the contract's method-prefixed message.
/// The thrown object's fields arrive as the JSON text index.js's
/// `thrownFields` builds; `method` is the bridge's own.
#[derive(Debug, serde::Deserialize)]
pub struct BridgeFailure {
    #[serde(skip)]
    pub method: String,
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
    /// The thrown object's `key` (a store port's conflict names it).
    #[serde(default)]
    pub key: Option<String>,
    /// The thrown object's `actual` as JSON: a number, null, or another value
    /// (`NaN` and `Infinity` as text), which a store port's conflict refuses.
    #[serde(default)]
    pub actual: serde_json::Value,
}

impl BridgeFailure {
    pub(crate) fn new(method: &str, message: impl Into<String>) -> BridgeFailure {
        BridgeFailure {
            method: method.to_string(),
            message: message.into(),
            code: None,
            key: None,
            actual: serde_json::Value::Null,
        }
    }
}

impl std::fmt::Display for BridgeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.method, self.message)
    }
}

/// One call, sent from a worker thread to the JS thread.
pub struct CallRequest {
    method: &'static str,
    /// JSON array of the method's arguments.
    args: serde_json::Value,
    tx: mpsc::Sender<std::result::Result<serde_json::Value, BridgeFailure>>,
}

/// The way to one JS invoker: `Clone` hands each worker its own handle to the
/// single underlying ThreadsafeFunction.
#[derive(Clone)]
pub struct JsBridge {
    tsfn: ThreadsafeFunction<CallRequest, ErrorStrategy::Fatal>,
}

/// Build the bridge over the JS invoker `(method, args, settle) => void`
/// (created by `index.js` with the target object bound in its closure).
pub fn create_bridge(env: Env, invoker: JsFunction) -> Result<JsBridge> {
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
                                        format!("the function returned a non-JSON value: {e}"),
                                    )
                                }),
                        }
                    }
                    _ => {
                        // index.js sends the thrown object's fields as JSON text.
                        let parsed = fctx
                            .env
                            .from_js_value::<String, JsUnknown>(err)
                            .ok()
                            .and_then(|text| serde_json::from_str::<BridgeFailure>(&text).ok());
                        Err(match parsed {
                            Some(failure) => BridgeFailure { method: method.to_string(), ..failure },
                            None => BridgeFailure::new(method, "the function failed"),
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
    Ok(JsBridge { tsfn })
}

impl JsBridge {
    /// Call a JS function and wait (on the worker thread) for it to settle.
    pub(crate) fn call(
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
                format!("JS bridge unavailable ({status:?})"),
            ));
        }
        rx.recv().unwrap_or_else(|_| {
            Err(BridgeFailure::new(
                method,
                "JS bridge channel closed before the function settled",
            ))
        })
    }

    /// Call a JS function and do not wait for it: nothing reads its result,
    /// and a failure to queue the call is dropped too.
    pub(crate) fn notify(&self, method: &'static str, args: serde_json::Value) {
        // The receiver is dropped at once, so the settle callback's send fails
        // quietly when the call completes.
        let (tx, _) = mpsc::channel();
        let _ = self.tsfn.call(CallRequest { method, args, tx }, ThreadsafeFunctionCallMode::NonBlocking);
    }
}
