//! Guard: nothing in speclink-verbs prints. The crate turns results into text
//! and hands the text back, so a print macro anywhere under `src/` — test
//! modules included — is a defect. The scan is recursive: it does not depend
//! on which directory level a file sits at.

use std::path::{Path, PathBuf};

const PRINT_MACROS: [&str; 5] = ["println", "print", "eprintln", "eprint", "dbg"];

/// Every `<file>:<line>: <macro>!` under `dir`, at any depth.
fn violations(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the source directory") {
            let path = entry.expect("read a directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).expect("read a source file");
                for (number, line) in text.lines().enumerate() {
                    for name in PRINT_MACROS {
                        if calls_macro(line, name) {
                            found.push(format!("{}:{}: {name}!", path.display(), number + 1));
                        }
                    }
                }
            }
        }
    }
    found.sort();
    found
}

/// Whether `line` has `name!` as a whole macro name (`print!` inside
/// `eprint!` or in `my_print!` does not count as `print!`).
fn calls_macro(line: &str, name: &str) -> bool {
    let call = format!("{name}!");
    line.match_indices(&call).any(|(at, _)| {
        !line[..at].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

#[test]
fn the_source_has_no_direct_print() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let found = violations(&src);
    assert!(found.is_empty(), "speclink-verbs must not print:\n{}", found.join("\n"));
}

#[test]
fn a_print_macro_at_any_depth_is_found() {
    let base = std::env::temp_dir().join(format!("speclink-verbs-no-print-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let deep = base.join("render").join("inner");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(base.join("clean.rs"), "fn f() -> String { format!(\"x\") }\n").unwrap();
    std::fs::write(
        deep.join("leaky.rs"),
        "#[cfg(test)]\nmod tests {\n    fn f() { println!(\"x\"); eprint!(\"y\"); }\n}\n",
    )
    .unwrap();
    let found = violations(&base);
    std::fs::remove_dir_all(&base).unwrap();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].ends_with("leaky.rs:3: eprint!"), "{found:?}");
    assert!(found[1].ends_with("leaky.rs:3: println!"), "{found:?}");
}

#[test]
fn each_print_macro_is_recognized_by_its_whole_name() {
    for name in PRINT_MACROS {
        assert!(calls_macro(&format!("    {name}!(\"x\");"), name), "{name}");
    }
    assert!(!calls_macro("eprint!(\"x\")", "print"));
    assert!(!calls_macro("outln!(t, \"x\")", "println"));
    assert!(!calls_macro("my_dbg!(x)", "dbg"));
}

/// The output functions take "color or not" and "JSON or not" as parameters.
/// They read nothing from the process: no environment variable, no terminal
/// state, no process-level switch.
#[test]
fn the_output_functions_read_no_process_state() {
    const PROCESS_STATE: [&str; 6] =
        ["std::env", "env::var", "IsTerminal", "is_terminal", "OnceLock", "static mut"];
    let render = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("render");
    let mut found = Vec::new();
    let mut pending = vec![render];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the render directory") {
            let path = entry.expect("read a directory entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read a source file");
            for (number, line) in text.lines().enumerate() {
                for token in PROCESS_STATE {
                    if line.contains(token) {
                        found.push(format!("{}:{}: {token}", path.display(), number + 1));
                    }
                }
            }
        }
    }
    assert!(found.is_empty(), "render reads process state:\n{}", found.join("\n"));
}
