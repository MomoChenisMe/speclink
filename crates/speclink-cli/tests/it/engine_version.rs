//! `speclink --version` 的引擎版號查詢面（spec「引擎版號查詢面」）。
//!
//! 「這顆 binary 的引擎是哪版」以前無從問起——過期的 app 就這樣裝了進來。
//! 版號輸出把它變成一條指令可斷言，本機安裝腳本的兩道斷言也建立在這上面。

use speclink_core::init::ASSET_VERSION;
use std::process::Command;

/// Scenario「--version 含引擎版號」＋ Example「版號輸出格式」。
#[test]
fn version_prints_the_package_arch_and_engine_version_on_one_line() {
    let out = Command::new(env!("CARGO_BIN_EXE_speclink"))
        .arg("--version")
        .output()
        .expect("run speclink binary");

    assert!(out.status.success(), "--version must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        stdout.trim_end().lines().count(),
        1,
        "stdout must be a single line: {stdout}"
    );
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "must carry the package version: {stdout}"
    );
    assert!(
        stdout.contains(&format!("engine {ASSET_VERSION}")),
        "must carry the artifact-layer version: {stdout}"
    );

    // Example「版號輸出格式」：`<套件版號> (<架構>, engine <產物層版號>)`。
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let arch = if cfg!(target_arch = "x86_64") { "x64" } else { "arm64" };
        assert_eq!(
            stdout.trim_end(),
            format!(
                "speclink {} ({arch}, engine {ASSET_VERSION})",
                env!("CARGO_PKG_VERSION")
            ),
            "版號格式是對外契約"
        );
    }
}
