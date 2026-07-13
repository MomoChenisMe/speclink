//! speclink-protocol: the single Rust definition of the Speclink Client
//! Protocol wire contract (see `docs/platform-architecture.zh-TW.md` §4.5).
//!
//! Rust types are the canon; JSON Schema is an export (design decision two).
//! This crate holds types, constants, and the error reason registry only —
//! it must never depend on speclink-core, speclink-host, or speclink-store:
//! the wire contract is the shared downstream of both client and server, and
//! neither side's implementation details may leak in.

pub mod binding;
pub mod command;
pub mod context;
pub mod error;
pub mod events;
pub mod query;

/// The contract major version this protocol speaks — sent as the
/// `X-Speclink-Api-Version` request header and declared back by the server
/// in the handshake's `apiVersion` field.
pub const API_VERSION: &str = "1";
