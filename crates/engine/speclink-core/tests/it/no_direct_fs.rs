//! Architectural check: the engine must not touch spec-directory content with
//! direct `std::fs` calls — all spec-document access goes through the `Store`
//! interface. This scans the crate's source files for the `std::fs` token.
//!
//! A source scan cannot see a call's *target*, so the assertion is scoped by
//! an explicit allowlist of files whose `std::fs` usage is host-side (not spec
//! documents), each entry justified by a design decision of the
//! store-trait-and-fs-adapter change:
//!
//! - `util.rs` — the generic file helpers; named as an exception by the
//!   check's own definition. (`workspace.rs`, the other named exception, turns
//!   out to need no `std::fs` at all — host paths use `Path` probes only — so
//!   it is deliberately NOT allowlisted: the check is stricter than required.)
//! - `init.rs` — project bootstrap: scaffolds the initial (empty) storage
//!   skeleton and host tool files (CLAUDE.md, skills, .gitignore) before a
//!   store exists.
//! - `schema.rs` — workflow schema definitions (project/user `schemas/`
//!   directories); outside the Store method inventory by design decision 1.
//! - `testkit.rs` — doc-hidden test-support helpers for the workspace's test
//!   suites (they rewrite throwaway temp workspaces' skill files); never part
//!   of an engine flow, but compiled into the lib so sibling crates' tests can
//!   share it.
//!
//! Every other source file must be free of `std::fs`. The allowlist is exact:
//! an allowlisted file that no longer uses `std::fs` fails the test too, so
//! the list cannot silently rot.

use std::path::Path;

const ALLOWLIST: &[&str] = &[
    "util.rs",
    "init.rs",
    "schema.rs",
    "testkit.rs",
];

#[test]
fn engine_sources_do_not_touch_spec_storage_directly() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    let mut stale_allowlist: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&src).expect("read crate src dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("utf-8 file name")
            .to_string();
        let text = std::fs::read_to_string(&path).expect("read source file");
        // Unit-test modules are host-side by nature (they scaffold throwaway
        // temp workspaces) and never ship in the lib; the engine-flow rule
        // covers production code only, so the scan stops at the conventional
        // trailing `#[cfg(test)]` module.
        let prod = text.split("#[cfg(test)]").next().unwrap_or(&text);
        let uses_fs = prod.contains("std::fs");
        if ALLOWLIST.contains(&name.as_str()) {
            if !uses_fs {
                stale_allowlist.push(name);
            }
        } else if uses_fs {
            offenders.push(name);
        }
    }

    assert!(
        offenders.is_empty(),
        "direct std::fs usage in engine flow modules (spec documents must go \
through the Store interface): {offenders:?}"
    );
    assert!(
        stale_allowlist.is_empty(),
        "allowlisted files no longer use std::fs — remove them from the \
allowlist to keep it exact: {stale_allowlist:?}"
    );
}
