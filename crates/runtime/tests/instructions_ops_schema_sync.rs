//! Integration test：確認 `instructions_ops::run` 產出的 11-field envelope 與
//! catalogue entry 32（`instructions.get`）的 `outputs_schema` 函式宣告 shape
//! 一致。Spec scenario「Hardcoded dependency table matches schema.yaml DAG」+
//! design Acceptance criteria 第 8 條（envelope shape 不漂移）。
//!
//! 不引入 `jsonschema` crate；以 structural keys + null-ability 手做比對，避免
//! 多一條 dev-dep。

use serde_json::Value;
use speclink_provider::{
    Config, ConfigStore, ConfigWarning, ProviderError, Versioned, WriteConfigRequest,
};
use speclink_runtime::catalogue::Catalogue;
use speclink_runtime::instructions_ops::{Input, run};

// ----- Mock stores ---------------------------------------------------------

struct EmptyConfigStore;

impl ConfigStore for EmptyConfigStore {
    fn read_config(&self) -> Result<Versioned<Config>, ProviderError> {
        Ok(Versioned {
            value: Config::default(),
            etag: speclink_provider::Etag::from_literal("v1.zerozerozero".to_string()),
        })
    }
    fn write_config(
        &self,
        _request: WriteConfigRequest,
    ) -> Result<Versioned<Config>, ProviderError> {
        unreachable!()
    }
    fn read_defaults(&self) -> Config {
        Config::default()
    }
    fn take_warnings(&self) -> Vec<ConfigWarning> {
        vec![]
    }
}

struct EmptyChangeStore;

#[async_trait::async_trait]
impl speclink_provider::ChangeStore for EmptyChangeStore {
    async fn create_change(
        &self,
        _name: &str,
        _schema_id: &str,
    ) -> Result<speclink_provider::ChangeRow, ProviderError> {
        unreachable!()
    }
    async fn list_changes(&self) -> Result<Vec<speclink_provider::ChangeRow>, ProviderError> {
        unreachable!()
    }
    async fn get_change(&self, name: &str) -> Result<speclink_provider::ChangeRow, ProviderError> {
        Err(ProviderError::ChangeNotFound {
            name: name.to_string(),
        })
    }
    async fn delete_change(&self, _name: &str) -> Result<(), ProviderError> {
        unreachable!()
    }
}

// ----- The sync test ------------------------------------------------------

#[tokio::test]
async fn output_envelope_matches_operations_md_schema() {
    let catalogue_entry =
        Catalogue::get("instructions.get").expect("catalogue entry 32 instructions.get exists");
    let outputs_schema: Value = (catalogue_entry.outputs_schema)();
    let schema_props = outputs_schema["properties"]
        .as_object()
        .expect("outputs_schema has properties");

    let cfg_store = EmptyConfigStore;
    let change_store = EmptyChangeStore;

    for kind in [
        "proposal", "spec", "design", "tasks", "apply", "ingest", "archive", "commit",
    ] {
        let input = Input {
            kind: kind.to_string(),
            change_id: None,
            role: None,
            discussion_id: None,
        };
        let (output, _warnings) = run(input, &cfg_store, &change_store)
            .await
            .unwrap_or_else(|e| panic!("run failed for kind={kind}: {e:?}"));
        let envelope_value = serde_json::to_value(&output)
            .unwrap_or_else(|e| panic!("serialize Output for kind={kind}: {e}"));
        let envelope_obj = envelope_value
            .as_object()
            .unwrap_or_else(|| panic!("Output should serialize to object for kind={kind}"));

        // (1) envelope keys 與 schema properties keys 一致（11 個）
        let envelope_keys: std::collections::BTreeSet<&str> =
            envelope_obj.keys().map(String::as_str).collect();
        let schema_keys: std::collections::BTreeSet<&str> =
            schema_props.keys().map(String::as_str).collect();
        assert_eq!(
            envelope_keys, schema_keys,
            "envelope keys mismatch schema for kind={kind}: envelope={envelope_keys:?} schema={schema_keys:?}"
        );

        // (2) required fields 必須非 null
        let required = outputs_schema["required"]
            .as_array()
            .expect("required array");
        for r in required {
            let key = r.as_str().expect("required entry is string");
            assert!(
                !envelope_obj[key].is_null(),
                "required field `{key}` is null for kind={kind}"
            );
        }

        // (3) artifact kinds: template + output_path 為 non-null string
        // workflow phase kinds: template + output_path 為 null
        let is_artifact_kind = matches!(kind, "proposal" | "spec" | "design" | "tasks");
        if is_artifact_kind {
            assert!(
                envelope_obj["template"].is_string(),
                "artifact kind {kind} should have non-null template"
            );
            assert!(
                envelope_obj["output_path"].is_string(),
                "artifact kind {kind} should have non-null output_path"
            );
        } else {
            assert!(
                envelope_obj["template"].is_null(),
                "phase kind {kind} should have null template"
            );
            assert!(
                envelope_obj["output_path"].is_null(),
                "phase kind {kind} should have null output_path"
            );
        }

        // (4) available_roles / linked_changes_context 永遠 null（Phase 2 reserved）
        assert!(
            envelope_obj["available_roles"].is_null(),
            "available_roles SHALL be null in P1-3 for kind={kind}"
        );
        assert!(
            envelope_obj["linked_changes_context"].is_null(),
            "linked_changes_context SHALL be null in P1-3 for kind={kind}"
        );
    }
}

#[tokio::test]
async fn output_dependencies_items_match_schema_shape() {
    // 對齊 schema 的 dependencies.items 結構：kind/capability/path 三 field、
    // kind required、capability/path nullable。
    let cfg_store = EmptyConfigStore;
    let change_store = EmptyChangeStore;

    let input = Input {
        kind: "tasks".to_string(),
        change_id: None,
        role: None,
        discussion_id: None,
    };
    let (output, _) = run(input, &cfg_store, &change_store).await.unwrap();
    let envelope = serde_json::to_value(&output).unwrap();
    let deps = envelope["dependencies"].as_array().expect("deps array");
    assert_eq!(deps.len(), 3);
    for dep in deps {
        let obj = dep.as_object().expect("dep is object");
        assert!(obj.contains_key("kind"));
        assert!(obj.contains_key("capability"));
        assert!(obj.contains_key("path"));
        assert!(obj["kind"].is_string());
        assert!(obj["capability"].is_null(), "P1-3 capability always null");
        assert!(obj["path"].is_string());
    }
}
