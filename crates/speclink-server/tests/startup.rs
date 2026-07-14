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
    let path = "/no/such/speclink-server-config.yaml";
    let out = run_with_config(path);
    assert!(!out.status.success(), "a missing config must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(path), "stderr names the config file: {stderr}");
}

#[test]
fn an_unparseable_config_exits_non_zero_with_the_reason() {
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
    let (out, _file) = run_with_contents("store:\n  driver: postgres\n");
    assert!(!out.status.success(), "an unsupported driver must fail startup");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("postgres"), "stderr names the bad driver: {stderr}");
    assert!(
        stderr.contains("sqlite") && stderr.contains("memory"),
        "stderr lists the supported drivers: {stderr}"
    );
}
