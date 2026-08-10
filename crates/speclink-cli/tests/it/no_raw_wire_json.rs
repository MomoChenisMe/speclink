//! Residue inventory: the wire layer is typed end to end. Neither
//! speclink-remote nor the CLI's verb layer may handle a wire payload as
//! `serde_json::Value` — every request and response goes through a
//! speclink-protocol DTO. A hit here means a raw-JSON bypass crept back in.
//!
//! The CLI side is fail-closed: every source file of the verb layer
//! (`remote_base.rs` plus all of `src/verbs/`, fs arms included) is scanned
//! whole, and the only lines allowed to mention `serde_json::Value` are the
//! fs-side output-assembly lines enumerated below. Adding a new legitimate
//! use means adding its exact line here — a conscious, reviewable act; any
//! other appearance fails the test, wherever and however the code is shaped.

use std::path::{Path, PathBuf};

/// fs-side output assembly that legitimately builds `serde_json::Value`:
/// (path suffix, exact trimmed line). An entry that stops matching fails the
/// test too, so the list can never go stale silently.
const FS_SIDE_ALLOWED: &[(&str, &str)] = &[
    (
        "verbs/query.rs",
        "return print_json(&serde_json::Value::Object(payload));",
    ),
    (
        "verbs/query.rs",
        "fn render_specs_section(specs: &serde_json::Value, json: bool) -> Result<()> {",
    ),
    (
        "verbs/lifecycle.rs",
        "let discussions: Vec<serde_json::Value> = unlinked",
    ),
    (
        "verbs/station.rs",
        "fn ticket_json(change: &str, ticket: &core::station::Ticket) -> serde_json::Value {",
    ),
];

fn assert_no_value_type(path: PathBuf) {
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let hits: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("serde_json::Value"))
        .map(|(i, line)| (i + 1, line.trim()))
        .collect();
    assert!(
        hits.is_empty(),
        "{} handles a wire payload as serde_json::Value — use a speclink-protocol DTO instead:\n{}",
        path.display(),
        hits.iter()
            .map(|(n, l)| format!("  line {n}: {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn rs_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("list {}: {e}", d.display()))
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|ext| ext == "rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn remote_crate_has_no_raw_wire_json() {
    let remote_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("speclink-remote")
        .join("src");
    let entries = rs_files_under(&remote_src);
    assert!(
        !entries.is_empty(),
        "expected .rs sources under {}, found none",
        remote_src.display()
    );
    for path in entries {
        assert_no_value_type(path);
    }
}

#[test]
fn verb_layer_has_no_raw_wire_json_outside_the_allowlist() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![src.join("remote_base.rs")];
    let families = rs_files_under(&src.join("verbs"));
    assert!(
        !families.is_empty(),
        "expected .rs sources under {}, found none",
        src.join("verbs").display()
    );
    files.extend(families);

    let mut hits: Vec<String> = Vec::new();
    let mut matched = vec![false; FS_SIDE_ALLOWED.len()];
    for path in files {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let rel = path.to_string_lossy().replace('\\', "/");
        for (i, line) in source.lines().enumerate() {
            if !line.contains("serde_json::Value") {
                continue;
            }
            let allowed = FS_SIDE_ALLOWED.iter().position(|(suffix, exact)| {
                rel.ends_with(suffix) && line.trim() == *exact
            });
            match allowed {
                Some(idx) => matched[idx] = true,
                None => hits.push(format!("  {}:{}: {}", rel, i + 1, line.trim())),
            }
        }
    }
    assert!(
        hits.is_empty(),
        "the verb layer handles a wire payload as serde_json::Value — use a speclink-protocol \
         DTO instead, or add a genuinely fs-side output-assembly line to FS_SIDE_ALLOWED:\n{}",
        hits.join("\n")
    );
    let stale: Vec<String> = FS_SIDE_ALLOWED
        .iter()
        .zip(&matched)
        .filter(|(_, m)| !**m)
        .map(|((suffix, exact), _)| format!("{suffix}: {exact}"))
        .collect();
    assert!(
        stale.is_empty(),
        "stale FS_SIDE_ALLOWED entries no longer match any line — remove or update them:\n  {}",
        stale.join("\n  ")
    );
}
