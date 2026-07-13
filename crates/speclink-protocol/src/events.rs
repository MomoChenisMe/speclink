//! Event discovery declaration types (blueprint §9.2): transports, polling,
//! and resume capability. Declaration only — no transport is implemented or
//! connected in this knife; Query + ETag remains the recovery bedrock.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The `capabilities.events` declaration a server hands out at handshake.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventsDeclaration {
    #[serde(default)]
    pub transports: Vec<EventTransport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polling: Option<PollingDeclaration>,
}

/// One declared push transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventTransport {
    #[serde(rename = "type")]
    pub kind: TransportKind,
    pub url: String,
    #[serde(default)]
    pub resume: bool,
}

/// The declared transport kind. Unknown kinds deserialize as
/// [`TransportKind::Unknown`] so a newer server never breaks an older client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Sse,
    Websocket,
    #[serde(untagged)]
    Unknown(String),
}

/// The polling fallback declaration — the mandatory recovery bedrock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PollingDeclaration {
    pub url: String,
    #[serde(default)]
    pub etag: bool,
}
