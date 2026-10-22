//! One read path for local and scoped Remote completed checks.
use crate::bridge::{BridgeError, BridgeStore};
use speclink_core::{
    history::{self, CheckKind},
    store::Store,
};
use speclink_protocol::query as wire;
use speclink_store::{Scope, TeamStore};
pub fn query(
    store: &dyn Store,
    change: &str,
    kind: CheckKind,
    id: Option<&str>,
) -> anyhow::Result<wire::CheckHistoryResponse> {
    if let Some(id) = id {
        let record = history::record(store, change, id, kind)?;
        let m = record.metadata;
        Ok(wire::CheckHistoryResponse::Record(
            wire::CheckHistoryRecord {
                record_id: record.record_id,
                content: record.content,
                metadata: wire::CheckHistoryMetadata {
                    created_at: m.created_at,
                    actor: m.actor,
                    tool: m.tool,
                    outcome: m.outcome,
                    scope_digest: m.scope_digest,
                },
            },
        ))
    } else {
        let records = history::index(store, change)?
            .entries
            .into_iter()
            .filter(|entry| entry.kind == kind)
            .map(|entry| wire::CheckHistoryEntry {
                record_id: entry.record_id,
                kind: entry.kind.as_str().into(),
                created_at: entry.created_at,
                outcome: entry.outcome,
                relative_path: entry.relative_path,
                content_digest: entry.content_digest,
            })
            .collect();
        Ok(wire::CheckHistoryResponse::List(wire::CheckHistoryList {
            records,
            acceptance_receipt: if kind == CheckKind::Security {
                history::acceptance_receipt(store, change)?.map(|receipt| {
                    let content = receipt.markdown();
                    wire::AcceptanceReceiptRecord {
                        created_at: receipt.created_at,
                        content_digest: speclink_core::security::data::sha256(content.as_bytes()),
                        content,
                    }
                })
            } else {
                None
            },
        }))
    }
}
pub fn query_remote(
    store: &dyn TeamStore,
    scope: &Scope,
    change: &str,
    kind: CheckKind,
    id: Option<&str>,
) -> Result<wire::CheckHistoryResponse, BridgeError> {
    let view = BridgeStore::materialize(store, scope).map_err(BridgeError::Store)?;
    query(&view, change, kind, id).map_err(|error| {
        let code = if error
            .downcast_ref::<speclink_core::station::NotFound>()
            .is_some()
        {
            speclink_core::command::ErrorCode::NotFound
        } else {
            speclink_core::command::ErrorCode::Error
        };
        BridgeError::Command(speclink_core::command::CommandError::new(
            code,
            error.to_string(),
        ))
    })
}
