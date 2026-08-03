//! Residue inventory: the wire layer is typed end to end. Neither
//! speclink-remote nor the CLI's remote intercept layer may handle a wire
//! payload as `serde_json::Value` — every request and response goes through
//! a speclink-protocol DTO. A hit here means a raw-JSON bypass crept back in.

use std::path::PathBuf;

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

#[test]
fn remote_crate_has_no_raw_wire_json() {
    let remote_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("speclink-remote")
        .join("src");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&remote_src)
        .expect("list speclink-remote/src")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "speclink-remote sources found");
    for path in entries {
        assert_no_value_type(path);
    }
}

#[test]
fn remote_intercept_layer_has_no_raw_wire_json() {
    assert_no_value_type(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("remote_commands.rs"),
    );
}
