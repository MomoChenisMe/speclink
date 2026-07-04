//! Integration tests for the deprecation warning on legacy policy keys in `.speclink.yaml`.
//!
//! Pinned behavior: when `.speclink.yaml` contains any of locale / spec_locale / tdd / audit,
//! every command emits EXACTLY ONE stderr line with a fixed prefix listing the detected keys
//! and pointing at `openspec/config.yaml`; stdout (including `--json`) is unaffected; without
//! legacy keys there is no warning at all.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Throwaway project root laid out like a real project, removed on drop.
struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str, app_yaml: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-deprecation-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            // Isolate from developer/CI machines that export SPECLINK_* overrides.
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .output()
            .expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stderr_text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn stdout_text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn legacy_policy_keys_warn_exactly_once_listing_all_keys() {
    let with_legacy = TempProject::new(
        "all-keys",
        "locale: tw\nspec_locale: tw\ntdd: true\naudit: true\ntools:\n  - claude\n",
    );
    let clean = TempProject::new("clean-baseline", "tools:\n  - claude\n");

    let warned = with_legacy.run(&["list", "--json"]);
    let silent = clean.run(&["list", "--json"]);

    assert!(warned.status.success(), "exit code must stay 0");
    let err = stderr_text(&warned);
    let lines: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "exactly one warning line, got: {err:?}");
    let line = lines[0];
    assert!(
        line.starts_with("speclink: warning:"),
        "fixed prefix for script-friendly filtering, got: {line:?}"
    );
    for key in ["locale", "spec_locale", "tdd", "audit"] {
        assert!(line.contains(key), "warning must name detected key {key}: {line:?}");
    }
    assert!(
        line.contains("openspec/config.yaml"),
        "warning must point at the canonical home: {line:?}"
    );

    // stdout (--json) is byte-identical to the no-legacy-keys run.
    assert_eq!(stdout_text(&warned), stdout_text(&silent));
}

#[test]
fn legacy_key_subset_lists_only_detected_keys() {
    // Spec scenario 含舊鍵時單行警告: tdd + audit only.
    let p = TempProject::new("subset", "tdd: true\naudit: true\ntools:\n  - claude\n");
    let out = p.run(&["list", "--json"]);

    assert!(out.status.success());
    let err = stderr_text(&out);
    let lines: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "exactly one warning line, got: {err:?}");
    assert!(lines[0].contains("tdd") && lines[0].contains("audit"));
    // Keys NOT present in the file must not be listed ("locale" appears nowhere
    // else in the message, so a plain substring check is sound).
    assert!(!lines[0].contains("locale"), "absent keys must not be listed: {:?}", lines[0]);
}

#[test]
fn no_legacy_keys_means_no_warning() {
    // Spec scenario 無舊鍵時無警告: only tools + spec_dir.
    let p = TempProject::new("no-keys", "spec_dir: openspec\ntools:\n  - claude\n");
    let out = p.run(&["list"]);

    assert!(out.status.success());
    assert_eq!(stderr_text(&out).trim(), "", "no deprecation warning expected");
}
