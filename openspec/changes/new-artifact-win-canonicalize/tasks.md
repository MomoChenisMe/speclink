## 1. 測試 helper 跨平台修正

- [x] 1.1 crates/speclink-cli/tests/it/new_artifact.rs 的 `TempProject::new` 改為非 Windows 才 canonicalize(沿用 crates/speclink-cli/tests/it/trace.rs 第 42-43 行的既有模式與註解措辭),使 Windows 上期望路徑不帶 `\\?\` 前綴、與 CLI 輸出同底。驗證:grep 確認 new_artifact.rs 帶 `cfg!(windows)` 分支,且 macOS 本機 `cargo test -p speclink-cli --test it new_artifact` 全綠。 <!-- speclink-task:tsk_01M132C0H2ZNXQ09JJF614MS7M -->
- [x] 1.2 收尾盤點:git status 只含 crates/speclink-cli/tests/it/new_artifact.rs 與本 change 目錄自身,無夾帶。驗證:`git status --porcelain` 逐項核對。 <!-- speclink-task:tsk_01M132C0H2N7ZDBC9SFPD86HRD -->

## 2. CI 確認

- [ ] [M] 2.1 推送後在 GitHub Actions 確認 build-and-smoke (windows-latest) 的 Test (Rust workspace) 步驟綠、`new_artifact::a_canonical_capability_keeps_the_exact_success_output` 通過,且三平台 job 全綠。 <!-- speclink-task:tsk_01M132C0H2C53X86EH5Q4EX3TH -->
