//! `LocalConfigStore` integration tests — read path + fallback semantics.
//!
//! 對應 spec requirements:
//! - 「`speclink config show` SHALL read config.yaml and return `Versioned<Config>`」
//! - 「Read path SHALL fall back to defaults when config is missing or malformed」
//! - 「The defaults SHALL set `rules.require_*_review=false` to preserve walking-skeleton 4-state behavior」
//!
//! Section 4（read path + walking-skeleton fallback）對應的 4 條紅燈：
//! (a) happy read on fresh init
//! (b) missing file → defaults + `config.malformed_using_defaults` warning + `v0.malformed-fallback` etag
//! (c) malformed YAML → 同 (b)、無 audit row
//! (d) `read_defaults()` 回 walking-skeleton defaults。

use rusqlite::Connection;
use speclink_provider::{
    ConfigStore, ConfigValue, JsonPath, JsonPathSegment, Versioned, WriteConfigRequest,
    codes as provider_codes,
};
use speclink_provider_local::{LocalConfigStore, StateDb};
use std::path::Path;
use tempfile::TempDir;

const SAMPLE_CONFIG: &str =
    "rules:\n  require_artifact_review: false\n  require_code_review: false\n";

/// fresh init：建立 working_dir / state_root、寫 config.yaml、`state.db` 升至 v5 + seed
/// `config_state` row（同 `speclink init` 終態）。
fn fresh_init(tmp: &TempDir) -> LocalConfigStore {
    let working = tmp.path().to_path_buf();
    let speclink = working.join(".speclink");
    std::fs::create_dir_all(&speclink).expect("create .speclink");
    let config_path = speclink.join("config.yaml");
    std::fs::write(&config_path, SAMPLE_CONFIG).expect("write config.yaml");

    let state_root = working.join(".git").join("speclink");
    std::fs::create_dir_all(&state_root).expect("create state root");
    let db_path = state_root.join("state.db");
    let db = StateDb::open(&db_path).expect("open state.db");
    db.migrate(5).expect("migrate v5");
    db.seed_config_state(&config_path)
        .expect("seed config_state row");

    LocalConfigStore::new(working, state_root)
}

fn config_change_count(state_root: &Path) -> u32 {
    let conn = Connection::open(state_root.join("state.db")).expect("open db");
    conn.query_row("SELECT COUNT(*) FROM config_change", [], |r| r.get(0))
        .expect("count")
}

// ----- task 4.1 (a) happy read on fresh init -----

#[test]
fn read_config_returns_versioned_config_with_v1_etag_on_fresh_init() {
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);

    let Versioned { value, etag } = store.read_config().expect("read_config Ok");

    // Rules defaults align with walking-skeleton policy.
    assert!(!value.rules.require_artifact_review);
    assert!(!value.rules.require_code_review);

    // Etag SHALL match `v<version>.<sha[:12]>` per design「Config etag 命名格式對齊
    // artifact etag」decision.
    let s = etag.as_str();
    assert!(
        s.starts_with("v1."),
        "fresh init etag SHALL start with v1., got `{s}`"
    );
    let (ver, sha) = s.split_once('.').expect("etag SHALL contain a `.`");
    assert!(
        ver.starts_with('v') && ver[1..].chars().all(|c| c.is_ascii_digit()),
        "etag version segment `{ver}` SHALL be v<digits>"
    );
    assert_eq!(sha.len(), 12, "etag sha segment SHALL be 12 hex chars");
    assert!(
        sha.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "etag sha segment SHALL be lowercase hex"
    );

    // No warnings on happy read.
    let warnings = store.take_warnings();
    assert!(
        warnings.is_empty(),
        "expected no warnings on happy read, got: {warnings:?}"
    );

    // No audit row on plain read (state aligned).
    assert_eq!(
        config_change_count(tmp.path().join(".git").join("speclink").as_path()),
        0,
        "config_change SHALL remain empty on aligned read"
    );
}

// ----- task 4.1 (b) missing file fallback -----

#[test]
fn read_config_returns_defaults_when_config_file_is_missing() {
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    // Remove config.yaml after init.
    std::fs::remove_file(tmp.path().join(".speclink").join("config.yaml"))
        .expect("remove config.yaml");

    let Versioned { value, etag } = store.read_config().expect("read_config Ok (no error)");

    assert!(!value.rules.require_artifact_review);
    assert!(!value.rules.require_code_review);
    assert_eq!(
        etag.as_str(),
        "v0.malformed-fallback",
        "missing-file etag SHALL be the literal v0.malformed-fallback"
    );

    let warnings = store.take_warnings();
    assert_eq!(warnings.len(), 1, "expected exactly one warning");
    assert_eq!(
        warnings[0].code, "config.malformed_using_defaults",
        "missing-file warning code SHALL be config.malformed_using_defaults"
    );

    // Read path SHALL NOT write any audit row on fallback.
    assert_eq!(
        config_change_count(tmp.path().join(".git").join("speclink").as_path()),
        0,
        "fallback read SHALL NOT write a config_change row"
    );
}

// ----- task 4.1 (c) malformed YAML fallback -----

#[test]
fn read_config_returns_defaults_when_config_file_has_broken_yaml() {
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    // Overwrite with syntactically invalid YAML.
    std::fs::write(
        tmp.path().join(".speclink").join("config.yaml"),
        "rules:\n  require_artifact_review: [unclosed",
    )
    .expect("write broken yaml");

    let Versioned { value, etag } = store.read_config().expect("read_config Ok (no error)");

    assert!(!value.rules.require_artifact_review);
    assert!(!value.rules.require_code_review);
    assert_eq!(etag.as_str(), "v0.malformed-fallback");

    let warnings = store.take_warnings();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code, "config.malformed_using_defaults");

    assert_eq!(
        config_change_count(tmp.path().join(".git").join("speclink").as_path()),
        0,
        "malformed read SHALL NOT write a config_change row"
    );
}

// ----- task 4.3 read_defaults() -----

#[test]
fn read_defaults_returns_walking_skeleton_review_flags() {
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let defaults = store.read_defaults();
    assert!(
        !defaults.rules.require_artifact_review,
        "walking-skeleton default require_artifact_review SHALL be false"
    );
    assert!(
        !defaults.rules.require_code_review,
        "walking-skeleton default require_code_review SHALL be false"
    );
}

// ----- task 5.1 external-edit detection (red) -----

#[test]
fn read_config_detects_external_edit() {
    // GIVEN fresh init at sha A、config_state.version=1
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let initial = store.read_config().expect("happy read");
    let etag_before = initial.etag.as_str().to_string();
    assert!(etag_before.starts_with("v1."));
    let _ = store.take_warnings(); // drain any seed-induced warnings (none expected)

    // WHEN user bypasses engine to overwrite config.yaml with new bytes (sha B)
    let config_path = tmp.path().join(".speclink").join("config.yaml");
    std::fs::write(
        &config_path,
        "rules:\n  require_artifact_review: true\n  require_code_review: false\n",
    )
    .expect("external overwrite");

    // THEN next read_config:
    //   - returns the new parsed value
    //   - bumps version → etag = v2.<new sha[:12]>
    //   - emits warning config.external_edit_detected
    //   - state.db config_change has exactly 1 row with mode=external_edit
    let after = store.read_config().expect("read after external edit");
    assert!(
        after.value.rules.require_artifact_review,
        "new parsed value SHALL include external edit"
    );
    let etag_after = after.etag.as_str().to_string();
    assert!(
        etag_after.starts_with("v2."),
        "etag SHALL bump to v2., got {etag_after}"
    );
    assert_ne!(etag_before, etag_after);

    let warnings = store.take_warnings();
    assert_eq!(warnings.len(), 1, "expected exactly one warning");
    assert_eq!(warnings[0].code, "config.external_edit_detected");

    // Audit row: mode=external_edit, keys_changed='["__external_edit__"]',
    // etag_before/etag_after both match.
    let conn = Connection::open(tmp.path().join(".git").join("speclink").join("state.db"))
        .expect("open db");
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, String, Option<String>, String, String)> = conn
        .prepare(
            "SELECT mode, keys_changed, etag_before, etag_after, reason \
             FROM config_change ORDER BY change_seq",
        )
        .expect("prepare")
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .expect("query")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(rows.len(), 1, "expected exactly one config_change row");
    assert_eq!(rows[0].0, "external_edit");
    assert_eq!(rows[0].1, "[\"__external_edit__\"]");
    assert_eq!(rows[0].2.as_deref(), Some(etag_before.as_str()));
    assert_eq!(rows[0].3, etag_after);
    assert_eq!(rows[0].4, "config_external_edit");

    // config_state row updated: sha = new sha (which == sha derived from the etag), version=2.
    let row_sha: String = conn
        .query_row(
            "SELECT content_sha256 FROM config_state WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .expect("query");
    let etag_sha = etag_after.split_once('.').expect("etag has `.`").1;
    assert!(
        row_sha.starts_with(etag_sha),
        "config_state.content_sha256 (`{row_sha}`) SHALL start with etag sha prefix (`{etag_sha}`)"
    );
    let row_ver: i64 = conn
        .query_row("SELECT version FROM config_state WHERE id = 1", [], |r| {
            r.get(0)
        })
        .expect("query version");
    assert_eq!(row_ver, 2);
}

#[test]
fn read_config_idempotent_when_sha_unchanged() {
    // Read 同一份 config 兩次：version 不應 bump、不應產生 audit row、不應發出 warning。
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let first = store.read_config().expect("first");
    let second = store.read_config().expect("second");
    assert_eq!(first.etag.as_str(), second.etag.as_str());
    let warnings = store.take_warnings();
    assert!(
        warnings.is_empty(),
        "no warnings expected, got {warnings:?}"
    );
    assert_eq!(
        config_change_count(tmp.path().join(".git").join("speclink").as_path()),
        0,
    );
}

// ----- task 6.1 write_config Set / Edit (red) -----

fn jp(parts: &[&str]) -> JsonPath {
    JsonPath::from_segments(
        parts
            .iter()
            .map(|s| JsonPathSegment::Field((*s).to_string()))
            .collect(),
    )
}

fn read_config_yaml(working: &Path) -> String {
    std::fs::read_to_string(working.join(".speclink").join("config.yaml"))
        .expect("read config.yaml")
}

fn change_rows_summary(state_root: &Path) -> Vec<(String, String, Option<String>, String, String)> {
    let conn = Connection::open(state_root.join("state.db")).expect("open db");
    conn.prepare(
        "SELECT mode, keys_changed, etag_before, etag_after, reason \
         FROM config_change ORDER BY change_seq",
    )
    .expect("prepare")
    .query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })
    .expect("query")
    .filter_map(Result::ok)
    .collect()
}

#[test]
fn write_config_set_happy_without_expected_etag_patches_file_and_writes_audit_row() {
    // (a) write set happy path
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let initial = store.read_config().expect("initial read");
    assert!(initial.etag.as_str().starts_with("v1."));

    let result = store
        .write_config(WriteConfigRequest::Set {
            key: jp(&["rules", "require_code_review"]),
            value: ConfigValue::Bool(true),
            expected_etag: None,
            actor: None,
        })
        .expect("write set Ok");

    assert!(
        result.value.rules.require_code_review,
        "Config struct SHALL reflect the patch"
    );
    assert!(
        result.etag.as_str().starts_with("v2."),
        "etag SHALL bump to v2., got {}",
        result.etag.as_str()
    );

    // File SHALL be rewritten so `require_code_review: true` appears literally.
    let yaml = read_config_yaml(tmp.path());
    assert!(yaml.contains("require_code_review: true"), "got:\n{yaml}");

    let rows = change_rows_summary(tmp.path().join(".git").join("speclink").as_path());
    assert_eq!(rows.len(), 1, "exactly one config_change row");
    assert_eq!(rows[0].0, "set");
    assert_eq!(rows[0].1, "[\"rules.require_code_review\"]");
    assert_eq!(rows[0].4, "config_write");
}

#[test]
fn write_config_set_with_correct_expected_etag_succeeds() {
    // (b) CAS pass
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let initial = store.read_config().expect("initial read");
    let result = store
        .write_config(WriteConfigRequest::Set {
            key: jp(&["rules", "require_artifact_review"]),
            value: ConfigValue::Bool(true),
            expected_etag: Some(initial.etag.clone()),
            actor: None,
        })
        .expect("write set with matching etag");
    assert!(result.value.rules.require_artifact_review);
}

#[test]
fn write_config_set_with_wrong_expected_etag_returns_state_etag_mismatch() {
    // (c) CAS fail
    use speclink_provider::Etag;
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let _ = store.read_config().expect("read");
    let yaml_before = read_config_yaml(tmp.path());

    let err = store
        .write_config(WriteConfigRequest::Set {
            key: jp(&["rules", "require_code_review"]),
            value: ConfigValue::Bool(true),
            expected_etag: Some(Etag::from_literal("v1.WRONGSHA1234".to_string())),
            actor: None,
        })
        .expect_err("CAS mismatch SHALL be rejected");
    assert_eq!(err.code(), provider_codes::STATE_ETAG_MISMATCH);

    // File SHALL NOT be modified.
    assert_eq!(read_config_yaml(tmp.path()), yaml_before);
    // No audit row.
    assert_eq!(
        change_rows_summary(tmp.path().join(".git").join("speclink").as_path()).len(),
        0,
    );
}

#[test]
fn write_config_set_unknown_key_returns_config_key_not_found() {
    // (d) unknown key
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let err = store
        .write_config(WriteConfigRequest::Set {
            key: jp(&["rules", "unknown_flag"]),
            value: ConfigValue::Bool(true),
            expected_etag: None,
            actor: None,
        })
        .expect_err("unknown key SHALL be rejected");
    assert_eq!(err.code(), provider_codes::CONFIG_KEY_NOT_FOUND);
    assert_eq!(
        change_rows_summary(tmp.path().join(".git").join("speclink").as_path()).len(),
        0,
    );
}

#[test]
fn write_config_edit_via_stdin_replaces_file_and_writes_audit_row() {
    // (e) write edit happy
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let new_yaml =
        "rules:\n  require_artifact_review: true\n  require_code_review: true\n".to_string();
    let result = store
        .write_config(WriteConfigRequest::Edit {
            content: new_yaml.clone(),
            expected_etag: None,
            actor: None,
        })
        .expect("edit Ok");
    assert!(result.value.rules.require_artifact_review);
    assert!(result.value.rules.require_code_review);
    assert!(result.etag.as_str().starts_with("v2."));

    let on_disk = read_config_yaml(tmp.path());
    assert!(on_disk.contains("require_artifact_review: true"));
    assert!(on_disk.contains("require_code_review: true"));

    let rows = change_rows_summary(tmp.path().join(".git").join("speclink").as_path());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "edit");
    assert_eq!(rows[0].1, "[\"__edit__\"]");
    assert_eq!(rows[0].4, "config_write");
}

#[test]
fn write_config_edit_with_malformed_yaml_returns_config_malformed() {
    // (f) edit malformed
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let yaml_before = read_config_yaml(tmp.path());
    let err = store
        .write_config(WriteConfigRequest::Edit {
            content: "rules:\n  require_code_review: [unclosed".into(),
            expected_etag: None,
            actor: None,
        })
        .expect_err("malformed edit SHALL be rejected");
    assert_eq!(err.code(), provider_codes::CONFIG_MALFORMED);
    assert_eq!(read_config_yaml(tmp.path()), yaml_before);
    assert_eq!(
        change_rows_summary(tmp.path().join(".git").join("speclink").as_path()).len(),
        0,
    );
}

// ----- task 5.3 stale write after external edit fails CAS -----

#[test]
fn read_config_external_edit_then_stale_write_fails() {
    let tmp = TempDir::new().expect("tempdir");
    let store = fresh_init(&tmp);
    let initial = store.read_config().expect("initial read"); // v1.<sha-A>
    let stale_etag = initial.etag.clone();

    // External edit drifts file to sha-B.
    std::fs::write(
        tmp.path().join(".speclink").join("config.yaml"),
        "rules:\n  require_artifact_review: true\n  require_code_review: false\n",
    )
    .expect("external overwrite");
    let yaml_before_write = read_config_yaml(tmp.path());

    // Stale process holds expected_etag=v1.<sha-A> → SHALL fail CAS.
    let err = store
        .write_config(WriteConfigRequest::Set {
            key: jp(&["rules", "require_code_review"]),
            value: ConfigValue::Bool(true),
            expected_etag: Some(stale_etag),
            actor: None,
        })
        .expect_err("stale write SHALL be rejected");
    assert_eq!(err.code(), provider_codes::STATE_ETAG_MISMATCH);

    // File untouched (still external_edit bytes); no config_change row from write path.
    assert_eq!(read_config_yaml(tmp.path()), yaml_before_write);
}

// ----- task 6.3 value parsing precedence -----

#[test]
fn config_value_parse_precedence_matches_spec_example_table() {
    // Spec scenario「Value parsing precedence」example table。
    let pairs: &[(&str, ConfigValue)] = &[
        ("true", ConfigValue::Bool(true)),
        ("false", ConfigValue::Bool(false)),
        ("null", ConfigValue::Null),
        ("42", ConfigValue::Int(42)),
        ("1.5", ConfigValue::Float(1.5)),
        ("abc", ConfigValue::String("abc".to_string())),
        // 引號被 shell 保留，原值 `"1.5"` (含引號) → 不解析為 float、回 string raw。
        ("\"1.5\"", ConfigValue::String("\"1.5\"".to_string())),
        // JSON literal 不解析、回 string raw。
        ("[1,2,3]", ConfigValue::String("[1,2,3]".to_string())),
    ];
    for (input, expected) in pairs {
        let got = ConfigValue::parse(input);
        assert_eq!(&got, expected, "parse({input:?})");
    }
}

#[test]
fn read_defaults_is_pure_and_does_not_touch_disk() {
    // Even when no .speclink/ exists, read_defaults() SHALL return defaults
    // without erroring or accessing the filesystem.
    let tmp = TempDir::new().expect("tempdir");
    let store = LocalConfigStore::new(
        tmp.path().to_path_buf(),
        tmp.path().join("state-root-does-not-exist"),
    );
    let defaults = store.read_defaults();
    assert!(!defaults.rules.require_artifact_review);
    assert!(!defaults.rules.require_code_review);
}
