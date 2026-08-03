//! Startup fail-closed at the binary boundary (reference-server spec「啟動組態
//! fail closed」). A missing, unparseable, or unknown-driver configuration must
//! exit non-zero with a reason naming the file — before any port is bound.

use std::io::Write;
use std::process::Command;

/// Run the server binary with `--config <path>` and return its output. Every
/// case here uses a configuration that fails before binding, so the process
/// exits on its own and the call never hangs.
fn run_with_config(path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_speclink-server"))
        .args(["--config", path])
        .output()
        .expect("spawn speclink-server")
}

fn run_with_contents(contents: &str) -> (std::process::Output, tempfile::NamedTempFile) {
    let mut file = tempfile::NamedTempFile::new().expect("temp config");
    file.write_all(contents.as_bytes()).expect("write config");
    let out = run_with_config(file.path().to_str().expect("utf-8 path"));
    (out, file)
}

#[test]
fn a_missing_config_exits_non_zero_naming_the_file() {
    let _gate = crate::common::acquire_process_gate();
    let path = "/no/such/speclink-server-config.yaml";
    let out = run_with_config(path);
    assert!(!out.status.success(), "a missing config must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(path), "stderr names the config file: {stderr}");
}

#[test]
fn an_unparseable_config_exits_non_zero_with_the_reason() {
    let _gate = crate::common::acquire_process_gate();
    let (out, _file) = run_with_contents("store: : : not yaml\n  - broken");
    assert!(!out.status.success(), "unparseable YAML must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cannot parse config file"),
        "stderr states the parse reason: {stderr}"
    );
}

#[test]
fn an_unknown_driver_exits_non_zero_listing_supported_drivers() {
    let _gate = crate::common::acquire_process_gate();
    let (out, _file) = run_with_contents("store:\n  driver: mysql\n");
    assert!(!out.status.success(), "an unsupported driver must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("mysql"), "stderr names the bad driver: {stderr}");
    assert!(
        stderr.contains("sqlite")
            && stderr.contains("serverfs")
            && stderr.contains("postgres")
            && stderr.contains("memory"),
        "stderr lists the supported drivers: {stderr}"
    );
}

#[test]
fn a_serverfs_store_pointed_at_a_foreign_directory_exits_non_zero_and_touches_nothing() {
    let _gate = crate::common::acquire_process_gate();
    // The operator mistypes a path and lands on a directory that is already
    // someone's data. Starting here would mean writing a store's marker and
    // scope tree into it; the server refuses instead, and — the part that
    // actually matters — leaves every byte where it was.
    let dir = tempfile::tempdir().expect("data dir");
    std::fs::create_dir(dir.path().join("photos")).expect("create");
    std::fs::write(dir.path().join("photos/beach.jpg"), b"jpeg").expect("write");
    std::fs::write(dir.path().join("notes.txt"), b"do not delete").expect("write");

    let identity = dir.path().join("identity.db");
    let (out, _file) = run_with_contents(&format!(
        "store:\n  driver: serverfs\n  path: {}\nidentity:\n  driver: sqlite\n  path: {}\n",
        dir.path().display(),
        identity.display()
    ));

    assert!(!out.status.success(), "a foreign data directory must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(&dir.path().display().to_string()),
        "stderr names the directory: {stderr}"
    );
    assert!(
        stderr.contains("not a speclink team store"),
        "stderr states the reason: {stderr}"
    );

    assert_eq!(
        std::fs::read(dir.path().join("notes.txt")).expect("read back"),
        b"do not delete",
        "a refused start must not touch the directory's contents"
    );
    let mut left: Vec<String> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(
        left,
        ["notes.txt", "photos"],
        "a refused start leaves no marker, lock or scope tree behind"
    );
}

#[test]
fn a_residual_tokens_section_exits_non_zero_naming_its_replacement() {
    let _gate = crate::common::acquire_process_gate();
    let (out, _file) = run_with_contents(
        "store:\n  driver: memory\nidentity:\n  driver: memory\ntokens:\n  - token: x\n    actor:\n      display: A\n",
    );
    assert!(!out.status.success(), "a retired tokens section must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("identity store"),
        "stderr says the tokens section is replaced by the identity store: {stderr}"
    );
}

#[test]
fn a_residual_projects_section_exits_non_zero_naming_its_replacement() {
    let _gate = crate::common::acquire_process_gate();
    let (out, _file) = run_with_contents(
        "store:\n  driver: memory\nidentity:\n  driver: memory\nprojects:\n  - key: demo\n    repos:\n      - backend\n",
    );
    assert!(!out.status.success(), "a retired projects section must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("registry"),
        "stderr says the projects section is replaced by the registry: {stderr}"
    );
}
