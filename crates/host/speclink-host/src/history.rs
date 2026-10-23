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
        let record = if kind == CheckKind::Security {
            history::record(store, change, id, kind)?
        } else {
            history::browse_record(store, change, id, kind)?
        };
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
        let records = if kind == CheckKind::Security {
            history::index(store, change)?
                .entries
                .into_iter()
                .filter(|entry| entry.kind == kind)
                .map(|entry| wire_entry(entry, None))
                .collect()
        } else {
            history::browse_index(store, change, kind)?
                .entries
                .into_iter()
                .map(|row| wire_entry(row.entry, row.read_error))
                .collect()
        };
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

fn wire_entry(
    entry: speclink_core::history::CheckEntry,
    read_error: Option<history::CheckReadError>,
) -> wire::CheckHistoryEntry {
    wire::CheckHistoryEntry {
        record_id: entry.record_id,
        kind: entry.kind.as_str().into(),
        created_at: entry.created_at,
        outcome: entry.outcome,
        relative_path: entry.relative_path,
        content_digest: entry.content_digest,
        read_error: read_error.map(|error| match error {
            history::CheckReadError::MissingCheckRecord => wire::CheckHistoryReadError::MissingCheckRecord,
            history::CheckReadError::IntegrityError => wire::CheckHistoryReadError::IntegrityError,
            history::CheckReadError::CheckRecordReadError => wire::CheckHistoryReadError::CheckRecordReadError,
        }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::{history::{CheckEntry, CheckIndex, CheckKind, CheckMetadata}, security::data::sha256};
    use speclink_store::{memory::MemoryStore, CommandContext, DocumentId};

    #[test]
    fn remote_station_history_isolates_other_kind_and_keeps_revision_read_only() {
        let store = MemoryStore::new();
        let binding = crate::binding::local_default_binding();
        let scope = Scope::new(binding.project, binding.repo);
        let id = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
        let review_path = CheckKind::Review.path_with_started_at(id, "2026-10-22T15:37:33Z").unwrap();
        let metadata = CheckMetadata {
            created_at: "2026-10-22T15:37:33Z".into(),
            actor: None,
            tool: None,
            outcome: "clean".into(),
            scope_digest: "a".repeat(64),
        };
        let review_content = format!("<!-- speclink-check: {} -->\n# review\n", serde_json::to_string(&metadata).unwrap());
        let review = CheckEntry {
            record_id: id.into(),
            kind: CheckKind::Review,
            created_at: metadata.created_at.clone(),
            outcome: metadata.outcome.clone(),
            relative_path: review_path.clone(),
            content_digest: sha256(review_content.as_bytes()),
        };
        let missing_id = "01ARZ3NDEKTSV4RRFFQ69G5FAY";
        let missing_path = CheckKind::Verify.path(missing_id).unwrap();
        let index = CheckIndex { format_version: 1, entries: vec![
            review,
            CheckEntry { record_id: missing_id.into(), kind: CheckKind::Verify, created_at: metadata.created_at, outcome: "clean".into(), relative_path: missing_path, content_digest: "b".repeat(64) },
        ]};
        let mut uow = store.begin_unit_of_work(&scope, CommandContext { command: "seed".into(), actor: "test".into() }).unwrap();
        uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
        uow.create(DocumentId::ChangeArtifact { change: "demo".into(), artifact: "checks/index.json".into() }, serde_json::to_string(&index).unwrap());
        uow.create(DocumentId::ChangeArtifact { change: "demo".into(), artifact: review_path }, review_content);
        store.commit(uow, vec![]).unwrap();

        let before = store.snapshot(&scope).unwrap().revision();
        let response = query_remote(&store, &scope, "demo", CheckKind::Review, None).unwrap();
        let wire::CheckHistoryResponse::List(list) = response else { panic!("expected a history list") };
        assert_eq!(list.records.len(), 1);
        assert_eq!(list.records[0].outcome, "clean");
        assert_eq!(list.records[0].read_error, None);
        assert_eq!(store.snapshot(&scope).unwrap().revision(), before);
    }
}
