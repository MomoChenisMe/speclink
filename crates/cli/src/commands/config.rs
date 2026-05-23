//! `speclink config show` / `config set` / `config edit` subcommands.
//!
//! 對應 `config-rw` capability requirements 與 design contract 觀察行為。

#![allow(clippy::doc_markdown)]

use std::io::{self, Read, Write};
use std::path::Path;

use speclink_provider::{ConfigValue, Etag, JsonPath, JsonPathParseError, JsonPathSegment};
use speclink_runtime::{ConfigOperations, RealGitProbe, RuntimeError, RuntimeWarning};

use crate::output::Warning;

/// `config show [--key <jsonpath>]`。
///
/// # Errors
/// `RuntimeError::ConfigKeyNotFound` 當 `--key` JSONPath 不存在或包含不支援語法。
/// 其他錯誤從 `ConfigOperations::read_config` 傳遞。
pub fn run_show(
    working_dir: &Path,
    key: Option<&str>,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let ops = ConfigOperations::new(RealGitProbe);
    let (versioned, warnings) = ops.read_config(working_dir)?;
    let value_json: serde_json::Value = serde_json::to_value(&versioned.value)
        .map_err(|e| RuntimeError::Internal(format!("serialize Config: {e}")))?;
    let etag = versioned.etag.as_str().to_string();

    let data = if let Some(k) = key {
        let path = JsonPath::parse(k).map_err(|e| jsonpath_error_with_user_input(e, k))?;
        let leaf =
            walk_json(&value_json, &path).ok_or_else(|| RuntimeError::ConfigKeyNotFound {
                key: k.to_string(),
                hint: String::new(),
            })?;
        serde_json::json!({
            "key": k,
            "value": leaf,
            "etag": etag,
        })
    } else {
        serde_json::json!({
            "value": value_json,
            "etag": etag,
        })
    };
    Ok((data, warnings_to_envelope(warnings)))
}

/// `config set <key> <value> [--expected-etag <etag>]`。
///
/// # Errors
/// JSONPath parse 失敗 → `config.key_not_found`。其他錯誤從 store 抛上來。
pub fn run_set(
    working_dir: &Path,
    key: &str,
    value: &str,
    expected_etag: Option<&str>,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let path = JsonPath::parse(key).map_err(|e| jsonpath_error_with_user_input(e, key))?;
    let parsed = ConfigValue::parse(value);
    let expected = expected_etag.map(|s| Etag::from_literal(s.to_string()));
    let ops = ConfigOperations::new(RealGitProbe);
    let (versioned, warnings) = ops.set_config(working_dir, path, parsed, expected, None)?;
    let value_json: serde_json::Value = serde_json::to_value(&versioned.value)
        .map_err(|e| RuntimeError::Internal(format!("serialize Config: {e}")))?;
    let data = serde_json::json!({
        "value": value_json,
        "etag": versioned.etag.as_str(),
        "keys_changed": [key],
    });
    Ok((data, warnings_to_envelope(warnings)))
}

/// `config edit [--stdin] [--editor <cmd>] [--expected-etag <etag>]`。
///
/// 三種輸入分支：(1) `--stdin` 從 stdin 讀完整 content；(2) `--editor <cmd>` 或
/// `$EDITOR` env 啟動 child process 編輯臨時檔；(3) 都沒給 → `config.edit_mode_required`
/// (exit 2) 並提示三條可行路徑。
///
/// # Errors
/// 缺少輸入路徑 → `config.edit_mode_required`（message 含字面 `--stdin` 與 `$EDITOR`）。
/// 其他錯誤從 store 抛上來。
pub fn run_edit(
    working_dir: &Path,
    use_stdin: bool,
    editor: Option<&str>,
    expected_etag: Option<&str>,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    let content = if use_stdin {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| RuntimeError::Internal(format!("read stdin: {e}")))?;
        buf
    } else if let Some(cmd) = editor
        .map(str::to_string)
        .or_else(|| std::env::var("EDITOR").ok())
    {
        edit_via_external_command(working_dir, &cmd)?
    } else {
        return Err(RuntimeError::ConfigEditModeRequired);
    };

    let expected = expected_etag.map(|s| Etag::from_literal(s.to_string()));
    let ops = ConfigOperations::new(RealGitProbe);
    let (versioned, warnings) = ops.edit_config(working_dir, content, expected, None)?;
    let value_json: serde_json::Value = serde_json::to_value(&versioned.value)
        .map_err(|e| RuntimeError::Internal(format!("serialize Config: {e}")))?;
    let data = serde_json::json!({
        "value": value_json,
        "etag": versioned.etag.as_str(),
    });
    Ok((data, warnings_to_envelope(warnings)))
}

fn edit_via_external_command(working_dir: &Path, cmd: &str) -> Result<String, RuntimeError> {
    let config_path = working_dir.join(".speclink").join("config.yaml");
    let current =
        std::fs::read_to_string(&config_path).map_err(|_| RuntimeError::ConfigNotFound {
            path: config_path.display().to_string(),
        })?;
    let mut tmp = tempfile::NamedTempFile::new_in(working_dir)
        .map_err(|e| RuntimeError::Internal(format!("create editor tempfile: {e}")))?;
    tmp.write_all(current.as_bytes())
        .map_err(|e| RuntimeError::Internal(format!("write editor tempfile: {e}")))?;
    tmp.flush()
        .map_err(|e| RuntimeError::Internal(format!("flush editor tempfile: {e}")))?;
    let tmp_path = tmp.path().to_path_buf();

    // `$EDITOR` / `--editor` 允許帶參數（如 "vim -p"）；split 後第一個 token 為 program、其餘為 args。
    let mut parts = cmd.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| RuntimeError::Internal("empty editor command".into()))?;
    let extra: Vec<&str> = parts.collect();
    let status = std::process::Command::new(program)
        .args(&extra)
        .arg(&tmp_path)
        .status()
        .map_err(|e| RuntimeError::Internal(format!("spawn editor `{cmd}`: {e}")))?;
    if !status.success() {
        return Err(RuntimeError::Internal(format!(
            "editor `{cmd}` exited with non-zero status {status}"
        )));
    }
    let content = std::fs::read_to_string(&tmp_path)
        .map_err(|e| RuntimeError::Internal(format!("read editor tempfile back: {e}")))?;
    drop(tmp); // 顯式釋放 tempfile（Drop 自動清理檔案）。
    Ok(content)
}

fn warnings_to_envelope(rws: Vec<RuntimeWarning>) -> Vec<Warning> {
    rws.into_iter()
        .map(|w| Warning {
            code: w.code,
            message: w.message,
            details: w.details,
        })
        .collect()
}

/// Polish-config-error-messages：JSONPath parse 失敗時 SHALL 把 user 原始 `--key` 字面
/// 字串保留在 `key` 欄位，診斷理由（wildcard / filter / bad-segment）走 `hint`。
fn jsonpath_error_with_user_input(e: JsonPathParseError, raw_key: &str) -> RuntimeError {
    let hint = match e {
        JsonPathParseError::UnsupportedWildcard => {
            ": wildcards not supported in JSONPath subset".to_string()
        }
        JsonPathParseError::UnsupportedSyntax => {
            ": filters / recursive-descent not supported in JSONPath subset".to_string()
        }
        JsonPathParseError::BadSegment { detail, .. }
        | JsonPathParseError::BadIndex { detail, .. } => format!(": {detail}"),
    };
    RuntimeError::ConfigKeyNotFound {
        key: raw_key.to_string(),
        hint,
    }
}

fn walk_json<'a>(value: &'a serde_json::Value, path: &JsonPath) -> Option<&'a serde_json::Value> {
    let mut cur = value;
    for seg in path.segments() {
        match seg {
            JsonPathSegment::Field(name) => {
                cur = cur.as_object()?.get(name)?;
            }
            JsonPathSegment::Index(i) => {
                cur = cur.as_array()?.get(*i)?;
            }
        }
    }
    Some(cur)
}
