//! Guard: nothing in speclink-verbs prints. The crate turns results into text
//! and hands the text back, so a print macro or a direct write to a standard
//! stream anywhere under `src/` — test modules included — is a defect. The
//! scan is recursive: it does not depend on which directory level a file sits
//! at. It reads lines as text, so a comment that spells one of these out
//! counts too; that is on purpose (the rule is "the source does not contain
//! them").

use std::path::{Path, PathBuf};

const PRINT_MACROS: [&str; 5] = ["println", "print", "eprintln", "eprint", "dbg"];

/// A handle on a standard stream: `std::io::stdout()` and `std::io::stderr()`
/// write without a print macro.
const STREAM_HANDLES: [&str; 2] = ["stdout()", "stderr()"];

/// What the output functions must not read: an environment variable, the
/// terminal state, a process-level switch.
const PROCESS_STATE: [&str; 6] =
    ["std::env", "env::var", "IsTerminal", "is_terminal", "OnceLock", "static mut"];

/// Every Rust source line under `dir`, at any depth: `(file, line number, text)`.
fn source_lines(dir: &Path) -> Vec<(PathBuf, usize, String)> {
    let mut lines = Vec::new();
    let mut pending: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the source directory") {
            let path = entry.expect("read a directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).expect("read a source file");
                for (number, line) in text.lines().enumerate() {
                    lines.push((path.clone(), number + 1, line.to_string()));
                }
            }
        }
    }
    lines.sort();
    lines
}

/// Every `<file>:<line>: <what>` under `dir` where the line prints: a print
/// macro, or a handle on a standard stream.
fn violations(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for (path, number, line) in source_lines(dir) {
        for name in PRINT_MACROS {
            if calls_macro(&line, name) {
                found.push(format!("{}:{number}: {name}!", path.display()));
            }
        }
        for handle in STREAM_HANDLES {
            if has_whole_name(&line, handle) {
                found.push(format!("{}:{number}: {handle}", path.display()));
            }
        }
    }
    found
}

/// Whether `line` has `name!` as a whole macro name (`print!` inside
/// `eprint!` or in `my_print!` does not count as `print!`).
fn calls_macro(line: &str, name: &str) -> bool {
    has_whole_name(line, &format!("{name}!"))
}

/// Whether `line` has `token` with no identifier character right before it
/// (`use_stderr()` is not `stderr()`).
fn has_whole_name(line: &str, token: &str) -> bool {
    line.match_indices(token).any(|(at, _)| {
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
fn a_print_at_any_depth_is_found() {
    let base = std::env::temp_dir().join(format!("speclink-verbs-no-print-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let deep = base.join("render").join("inner");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(base.join("clean.rs"), "fn f() -> String { format!(\"x\") }\n").unwrap();
    std::fs::write(
        deep.join("leaky.rs"),
        "#[cfg(test)]\nmod tests {\n    fn f() { println!(\"x\"); eprint!(\"y\"); }\n}\nfn g() { let _ = std::io::stdout().write_all(b\"z\"); }\n",
    )
    .unwrap();
    let found = violations(&base);
    std::fs::remove_dir_all(&base).unwrap();
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].ends_with("leaky.rs:3: println!"), "{found:?}");
    assert!(found[1].ends_with("leaky.rs:3: eprint!"), "{found:?}");
    assert!(found[2].ends_with("leaky.rs:5: stdout()"), "{found:?}");
}

#[test]
fn each_print_macro_is_recognized_by_its_whole_name() {
    for name in PRINT_MACROS {
        assert!(calls_macro(&format!("    {name}!(\"x\");"), name), "{name}");
    }
    assert!(!calls_macro("eprint!(\"x\")", "print"));
    assert!(!calls_macro("outln!(t, \"x\")", "println"));
    assert!(!calls_macro("my_dbg!(x)", "dbg"));
    assert!(has_whole_name("let out = std::io::stderr();", "stderr()"));
    assert!(!has_whole_name("if e.use_stderr() {", "stderr()"));
}

/// The output functions take "color or not" and "JSON or not" as parameters.
/// They read nothing from the process.
#[test]
fn the_output_functions_read_no_process_state() {
    let render = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("render");
    let mut found = Vec::new();
    for (path, number, line) in source_lines(&render) {
        for token in PROCESS_STATE {
            if line.contains(token) {
                found.push(format!("{}:{number}: {token}", path.display()));
            }
        }
    }
    assert!(found.is_empty(), "render reads process state:\n{}", found.join("\n"));
}
