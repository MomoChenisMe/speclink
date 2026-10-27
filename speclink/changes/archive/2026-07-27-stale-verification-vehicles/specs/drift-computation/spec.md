## MODIFIED Requirements

### Requirement: 本地 drift 路徑輸出凍結

本地 cmd_drift 改為「蒐集 WorkspaceFacts → 兩段運算 → merger」三段串接後，speclink drift 的人眼輸出、--json 輸出與 exit code SHALL 與拆分前逐位元一致，含 git 可用、git 不可用、無 design 與 broken anchors 等既有情境。

#### Scenario: 重構前後輸出逐位元一致

- **WHEN** 對同一樣本 workspace（涵蓋 git 可用、git 不可用、無 design、broken anchors 情境）於拆分前後執行 speclink drift 與 speclink drift --json
- **THEN** stdout、stderr 與 exit code 逐位元一致；`crates/speclink-cli/tests/` 的整合測試（含 `--no-color` 人眼輸出斷言與 fs／remote 對照）與 `crates/speclink-core/tests/render_golden.rs` 全綠
