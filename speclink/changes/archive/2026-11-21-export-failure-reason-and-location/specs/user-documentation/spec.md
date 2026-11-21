## ADDED Requirements

### Requirement: SDK 文件說明匯出失敗的位置、鍵的契約、範圍摘要與匯出匯入的耗時

Node SDK 文件（中英兩版）的「匯出本機專案」一節 SHALL 列出 `engine.exportScope()` 帶 `reason` 的六種失敗，含 `change_schema_unusable`，並 SHALL 寫明 `reason` 是 `invalid_change_metadata` 或 `change_schema_unusable` 時 Error 另有 `change`（變更的資料夾名稱）與 `path`（metadata 檔相對於專案根目錄的路徑）兩個屬性，其他失敗沒有這兩個屬性。文件 SHALL NOT 再把「變更指定的產出流程找不到」舉為沒有 `reason` 的失敗。

Node SDK 文件說明儲存函式的一節 SHALL 寫明鍵的對照表是契約：既有的鍵不會改，宿主可以依它在自己的儲存裡統計；文件 SHALL NOT 讓讀者以為鍵的命名是引擎的內部細節。同一節之後 SHALL 有一小節「範圍的摘要」，寫明三樣資料的來源：各類文件的數量用鍵的前綴在宿主自己的儲存裡計算；最後寫入的人與時間取自 `commit` 函式收到的 `actor` 與 `at`；各階段的變更數用每個範圍一次 `host.dispatch` 的 `list --json`，這個呼叫會經過 `beforeCommand`。文件 SHALL 寫明 Speclink 不提供另外的摘要呼叫。

Node SDK 文件的「在伺服端匯入」一節 SHALL 寫明匯出與匯入的耗時量級與進度的位置：`engine.exportScope()` 與 `host.importScope()` 沒有進度回呼；一個約 1,600 份文件、約 19 MB 的規格資料夾，兩步各在數秒內完成；匯入只呼叫宿主的 `list` 一次與 `commit` 一次，寫入的時間在宿主自己的 `commit` 函式裡；下載規格資料夾是宿主自己的工作，進度由宿主提供。

中英兩版的這三處 SHALL 結構與事實對等。套件的型別說明（`index.d.ts`）裡 `exportScope` 的說明 SHALL 與文件一致：六個代碼與 `change`、`path` 兩個屬性。

#### Scenario: 整合者照文件處理產出流程造成的匯出失敗

- **WHEN** 整合者讀任一語言的 Node SDK 文件的「匯出本機專案」一節，要替 `engine.exportScope()` 的失敗顯示自己的說明
- **THEN** 文件的表列出六個 `reason`，含 `change_schema_unusable`；文件寫明哪兩個 `reason` 帶 `change` 與 `path`，整合者不需要比對訊息的文字就能指出是哪一個變更

#### Scenario: 整合者照文件計算範圍的摘要

- **WHEN** 整合者要在自己的管理頁顯示一個範圍有幾份正式規格、幾個進行中的變更、幾份討論，以及最後寫入的人與時間
- **THEN** 文件的「範圍的摘要」一節寫明每一樣資料的來源，並寫明鍵的對照表是契約；整合者不需要翻正式規格，也不會去找不存在的摘要呼叫

##### Example: 摘要的三樣資料與來源

| 資料 | 來源 |
| --- | --- |
| 正式規格的數量 | 自己的儲存裡以 `specs/` 開頭的鍵 |
| 進行中變更的數量 | 自己的儲存裡以 `changes/` 開頭的鍵，取第二段不重複的名稱 |
| 已封存變更的數量 | 以 `archive/changes/` 開頭的鍵，取第三段不重複的名稱 |
| 討論的數量 | 以 `discussions/` 與 `archive/discussions/` 開頭的鍵 |
| 最後寫入的人與時間 | `commit` 函式收到的 `actor` 與 `at` |
| 各階段的變更數 | 每個範圍一次 `host.dispatch` 的 `list --json` |

#### Scenario: 整合者從文件得知進度由誰提供

- **WHEN** 整合者要在匯入大的規格資料夾時顯示進度，讀 Node SDK 文件的「在伺服端匯入」一節
- **THEN** 文件寫明引擎的兩步沒有進度回呼、各在數秒內完成，並寫明進度在宿主自己的下載與 `commit` 函式裡

#### Scenario: 中英兩版對等

- **WHEN** 對照 `docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md` 的這三處
- **THEN** 兩版有相同的小節、相同的表列數與相同的事實；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 的文件對照測試通過
