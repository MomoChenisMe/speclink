//! Architectural check: the engine's production code must not read process
//! environment variables or perform git identity lookups — the policy env
//! layer, store-mode env, machine-level config location, and actor identity
//! are all resolved at the Host boundary (speclink-host) and injected.
//!
//! This scans every production source file of the crate (the part before the
//! conventional trailing `#[cfg(test)]` module) for the forbidden tokens.
//! Zero hits, no allowlist: an exception would be the seam through which a
//! server-mode host silently reads its own environment instead of the
//! caller's context (the §3.3/§3.4 gap this change closes).

use std::path::{Path, PathBuf};

/// Env reads and git-identity lookups that must not appear in engine
/// production code. `std::env::current_dir` (a cwd read, not an env read)
/// stays allowed — project discovery from the CLI's cwd is host input.
const FORBIDDEN: &[&str] = &[
    "std::env::var",
    "env::var(",
    "std::env::vars",
    "git_identity",
    "\"user.name\"",
    "\"user.email\"",
];

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn engine_production_code_reads_no_process_env_and_no_git_identity() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src, &mut files);
    files.sort();
    assert!(!files.is_empty(), "source scan found no files");

    let mut offenders: Vec<String> = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("read source file");
        // Unit-test modules scaffold sandboxes and may touch the process env;
        // the boundary rule covers production code only, so the scan stops at
        // the conventional trailing `#[cfg(test)]` module.
        let prod = text.split("#[cfg(test)]").next().unwrap_or(&text);
        for token in FORBIDDEN {
            if prod.contains(token) {
                offenders.push(format!("{}: {token}", path.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "engine production code must not read process env or git identity — \
resolve at the Host boundary and inject instead:\n{}",
        offenders.join("\n")
    );
}
