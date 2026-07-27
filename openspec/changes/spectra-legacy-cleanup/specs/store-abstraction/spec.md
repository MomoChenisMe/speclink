## MODIFIED Requirements

### Requirement: 儲存重構後既有指令行為保持不變

引擎改經儲存介面存取規格文件後，CLI 的所有既有指令 SHALL 維持與重構前完全一致的可觀察行為：人眼輸出（含色彩與 --no-color）、`--json` payload 的欄位（camelCase）與值、exit code、以及對檔案系統的效果。本需求為輸出凍結敏感：重構前的既有輸出基線 SHALL 維持位元級不變（驗證載體為 crates/speclink-cli/tests/ 的整合測試與 speclink-core 的 render_golden 測試）。

#### Scenario: 既有專案的清單查詢輸出一致

- **WHEN** 於既有 fs 專案根目錄執行 speclink list --json
- **THEN** 輸出 JSON 的欄位與值與重構前基線一致，exit code 為 0

#### Scenario: 無專案目錄時的錯誤輸出一致

- **WHEN** 於不含任何 speclink 專案標記的目錄執行 speclink list
- **THEN** stderr 的錯誤訊息文字與 exit code 與重構前基線一致

#### Scenario: 人眼輸出與 --no-color 一致

- **WHEN** 於既有專案分別執行 speclink status --change 某 change 與加上 --no-color 的同一指令
- **THEN** 兩種輸出均與重構前基線一致（含 ANSI 色彩序列與去色版本）
