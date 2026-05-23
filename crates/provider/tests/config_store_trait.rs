//! Compile-time surface 測試：證明 `ConfigStore` trait + `Provider::config_store()`
//! 入口存在、可從外部 crate 取用、`provider.config_store().read_config()` 可編譯
//! 並回傳 `Result<Versioned<Config>, ProviderError>`。
//!
//! 對應 `config-rw` capability requirement「`ConfigStore` trait SHALL be exposed
//! via `Provider::config_store()`」（spec scenario「Provider trait surface stable
//! across crates」）。任何違反此 surface 契約的修改會在 compile 階段被拒絕。

use speclink_provider::{
    Actor, Config, ConfigStore, ConfigValue, ConfigWarning, Etag, JsonPath, JsonPathSegment,
    Provider, ProviderError, Rules, Versioned, WriteConfigRequest,
};

/// Stub provider：實作 `Provider` trait 的最小可編譯型別；
/// 內部維護一個固定 `Config`、不接觸 filesystem，僅用於 surface check。
#[allow(dead_code)]
struct SurfaceProvider {
    inner: SurfaceConfigStore,
}

#[allow(dead_code)]
struct SurfaceConfigStore;

impl ConfigStore for SurfaceConfigStore {
    fn read_config(&self) -> Result<Versioned<Config>, ProviderError> {
        Ok(Versioned {
            value: Config::default(),
            etag: Etag::from_bytes(b""),
        })
    }
    fn write_config(
        &self,
        _request: WriteConfigRequest,
    ) -> Result<Versioned<Config>, ProviderError> {
        Err(ProviderError::Internal("stub".into()))
    }
    fn read_defaults(&self) -> Config {
        Config::default()
    }
    fn take_warnings(&self) -> Vec<ConfigWarning> {
        Vec::new()
    }
}

impl Provider for SurfaceProvider {
    fn config_store(&self) -> &dyn ConfigStore {
        &self.inner
    }
}

#[test]
fn config_store_trait_is_object_safe_and_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn ConfigStore>>();
}

#[test]
fn provider_trait_exposes_config_store_returning_read_config_result() {
    // Spec scenario「Provider trait surface stable across crates」核心斷言：
    // `provider.config_store().read_config()` SHALL compile 並回
    // `Result<Versioned<Config>, ProviderError>`.
    let provider = SurfaceProvider {
        inner: SurfaceConfigStore,
    };
    let result: Result<Versioned<Config>, ProviderError> = provider.config_store().read_config();
    let versioned = result.expect("stub returns Ok");
    assert!(!versioned.value.rules.require_artifact_review);
    assert!(!versioned.value.rules.require_code_review);
}

#[test]
fn write_config_request_set_variant_has_four_fields() {
    // 結構穩定性：Set / Edit 兩 variant 的欄位定義不變動。
    let _set = WriteConfigRequest::Set {
        key: JsonPath::from_segments(vec![
            JsonPathSegment::Field("rules".into()),
            JsonPathSegment::Field("require_code_review".into()),
        ]),
        value: ConfigValue::Bool(true),
        expected_etag: Some(Etag::from_bytes(b"")),
        actor: Some(Actor {
            agent_host: "cli".into(),
            os_user: "alice".into(),
            host_id: "macbook".into(),
        }),
    };
    let _set_no_etag = WriteConfigRequest::Set {
        key: JsonPath::from_segments(vec![JsonPathSegment::Field("rules".into())]),
        value: ConfigValue::Null,
        expected_etag: None,
        actor: None,
    };
}

#[test]
fn write_config_request_edit_variant_has_three_fields() {
    let _edit = WriteConfigRequest::Edit {
        content: "rules:\n  require_code_review: true\n".into(),
        expected_etag: None,
        actor: None,
    };
    let _edit_with_etag = WriteConfigRequest::Edit {
        content: "version: 1\n".into(),
        expected_etag: Some(Etag::from_bytes(b"x")),
        actor: Some(Actor {
            agent_host: "cli".into(),
            os_user: "bob".into(),
            host_id: "h".into(),
        }),
    };
}

#[test]
fn config_value_has_five_variants_for_set_value_precedence() {
    // 對應 spec scenario「Value parsing precedence」example table 的 5 種輸出型別。
    let _b = ConfigValue::Bool(false);
    let _n = ConfigValue::Null;
    let _i = ConfigValue::Int(42);
    let _f = ConfigValue::Float(1.5);
    let _s = ConfigValue::String("abc".into());
}

#[test]
fn config_default_matches_walking_skeleton_review_flags() {
    // 對應 spec requirement「The defaults SHALL set `rules.require_artifact_review=false`
    // and `rules.require_code_review=false` to preserve walking-skeleton 4-state behavior」。
    let c = Config::default();
    assert!(!c.rules.require_artifact_review);
    assert!(!c.rules.require_code_review);
}

#[test]
fn rules_default_is_both_false() {
    let r = Rules::default();
    assert!(!r.require_artifact_review);
    assert!(!r.require_code_review);
}

#[test]
fn jsonpath_from_segments_round_trips_via_segments_accessor() {
    let path = JsonPath::from_segments(vec![
        JsonPathSegment::Field("rules".into()),
        JsonPathSegment::Field("require_code_review".into()),
    ]);
    let segs = path.segments();
    assert_eq!(segs.len(), 2);
    assert!(matches!(&segs[0], JsonPathSegment::Field(s) if s == "rules"));
    assert!(matches!(&segs[1], JsonPathSegment::Field(s) if s == "require_code_review"));
}

#[test]
fn jsonpath_segment_supports_field_and_index_variants() {
    // Grammar「`segment ( '.' segment | '[' index ']' )*`」：Index variant 必須存在。
    let _field = JsonPathSegment::Field("rules".into());
    let _index = JsonPathSegment::Index(0);
}
