//! Path helpers for the LocalProvider implementation.

#![allow(clippy::doc_markdown)]

use std::path::{Path, PathBuf};

/// `.speclink/` 目錄名稱（相對於 working tree root）。
pub const ARTIFACT_ROOT: &str = ".speclink";

/// `<working_dir>/.speclink/changes/`。
#[must_use]
pub fn changes_dir(working_dir: &Path) -> PathBuf {
    working_dir.join(ARTIFACT_ROOT).join("changes")
}

/// `<working_dir>/.speclink/changes/<name>/`。
#[must_use]
pub fn change_dir(working_dir: &Path, name: &str) -> PathBuf {
    changes_dir(working_dir).join(name)
}

/// `<change_dir>/specs/`。
#[must_use]
pub fn specs_dir(working_dir: &Path, change_name: &str) -> PathBuf {
    change_dir(working_dir, change_name).join("specs")
}
