## 1. 測試期望值改逐段 join

- [x] 1.1 apps/desktop/core/src/settings.rs 的 `schemas_snapshot_carries_the_disk_path_for_file_backed_layers` 兩處期望值改逐段 join(`.join("openspec").join("schemas").join("my-flow")` 與 `.join("schemas").join("their-flow")`),使期望值與 `schemas_snapshot` 的原生路徑輸出同底、兩平台可逐位元比對。驗證:grep 確認該測試內不再有嵌正斜線的多段 join,且 macOS 本機 `cargo test -p speclink-desktop-core settings::` 全綠。 <!-- speclink-task:tsk_01M134W35199VHFG2DZFJ8MXKC -->
- [x] 1.2 收尾盤點:git status 只含 apps/desktop/core/src/settings.rs 與本 change 目錄自身,無夾帶。驗證:`git status --porcelain` 逐項核對。 <!-- speclink-task:tsk_01M134W351VGWSSJ03N44V1VP9 -->

## 2. CI 確認

- [x] [M] 2.1 推送後在 GitHub Actions 確認 build-and-smoke (windows-latest) 的 Test (Rust workspace) 步驟走完後段全部 binary 且綠、三平台 job 全綠。 <!-- speclink-task:tsk_01M134W351Y8HDSTE0EK09BRKC -->
