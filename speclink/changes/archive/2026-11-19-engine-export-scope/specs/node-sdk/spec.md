## ADDED Requirements

### Requirement: 本機引擎的 exportScope 方法

createEngine 回傳的引擎 SHALL 有方法 exportScope()：不接受參數，回傳 Promise。成功時的值 SHALL 是第 2 版的匯入包——`formatVersion` 是 2、`documents` 是陣列，每一項有 `key`（文件在儲存裡的鍵）、`content`（全文）與 `digest`（`sha256:` 加上 content 的 SHA-256 十六進位）；`documents` SHALL 依鍵的位元組順序排列，鍵 SHALL NOT 重複。值 SHALL NOT 含 `scopeVersion`：本機專案沒有範圍版本號。這個值 SHALL 能原樣交給 host.importScope 寫進一個空的範圍。

包的內容 SHALL 是這個本機專案的全部規格文件（見 workspace-migration「Bundle 組裝涵蓋本地 workspace 全集」）：進行中變更的 metadata、文件與 evidence，正式規格，進行中與已封存的討論，已封存變更的全部文件，流程設定，詞彙表，以及看板順序（一個順序鍵都沒有時不含）。把這個包匯入宿主之後，經 host.dispatch 讀到的變更、正式規格與討論 SHALL 與在原本的本機專案經 engine.dispatch 讀到的相同。

exportScope SHALL 只讀取、不寫入：本機專案的檔案 SHALL 逐位元不變。它 SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。專案沒有任何規格文件時 SHALL 成功並回傳 `documents` 為空陣列的包。

失敗時 SHALL 以 Error 拒絕 Promise，SHALL NOT 回傳不完整的包：任何一個變更（進行中或已封存）的 metadata 無法解析時，message SHALL 指出是哪個檔案；變更含有引擎的文件命名規則對不上的文件時，message SHALL 指出是哪份文件；通往規格資料夾路上的資料夾、規格資料夾本身或它底下任何一個檔案或資料夾是符號連結時（舊位置的 evidence 資料夾 `.speclink/touched/` 亦同），exportScope SHALL NOT 經連結讀檔，message SHALL 以相對專案根目錄、使用正斜線的路徑指出那個項目，連結指向的內容 SHALL NOT 出現在 message 裡（見 workspace-migration「本機專案的匯出與遷移產生相同的文件集合」）。

exportScope SHALL 只讀 createEngine 的 root 底下的規格資料夾：規格資料夾名稱（來自本機專案 `.speclink.yaml` 的 `spec_dir`，或 createEngine 的 `specDir`）是絕對路徑、含 `..` 或 `.` 路徑段、或是空的時，exportScope SHALL 在讀取任何檔案之前以 Error 拒絕，`code` 是 `error`，message SHALL 以雙引號原樣指出那個名稱，SHALL NOT 含任何檔案內容。專案根目錄裡面的多層名稱（例如 `docs/specs`）SHALL 照常匯出。engine.dispatch 讀規格資料夾的方式不受這條影響。Error 的形狀（message 加 code 欄位）SHALL 與 engine.dispatch 的失敗相同。createHost 回傳的宿主 SHALL 維持既有的 exportScope（接受身分與範圍），兩者互不影響。

#### Scenario: 匯出本機專案並匯入宿主

- **WHEN** 以 createEngine 指向一個含一個進行中變更 demo（有提案與任務）、一份正式規格 auth、一份進行中的討論 topic 的本機專案，呼叫 `await engine.exportScope()`，再把結果交給以 JS 記憶體儲存建立的宿主的 `host.importScope({ identity, scope: 's1', bundle })`
- **THEN** 匯入成功；之後 `host.dispatch` 在範圍 s1 執行 `['list', '--json']`、`['show', 'auth', '--item-type', 'spec']`、`['discuss', 'list', '--json']`，結果與在本機專案以 engine.dispatch 執行同樣的 argv 相同（清單裡只有本機才有的欄位除外）

##### Example: 匯出的包的形狀

本機專案只有一個變更 demo，它只有提案與 metadata。

| 鍵（依順序） | 內容 |
| --- | --- |
| `changes/demo/.meta.yaml` | 變更的 metadata 原文 |
| `changes/demo/proposal.md` | 提案原文 |

回傳值是 `{ "formatVersion": 2, "documents": [ … 上面兩項，各有 key、content、digest … ] }`，沒有 `scopeVersion`。

#### Scenario: 匯出不改動本機專案

- **WHEN** 記下本機專案規格資料夾每個檔案的內容與修改時間，呼叫 `await engine.exportScope()`
- **THEN** 每個檔案的內容與修改時間都沒有改變，沒有新增或刪除的檔案

#### Scenario: 壞掉的 metadata 讓整個匯出失敗

- **WHEN** 本機專案有兩個變更，其中 broken 的 metadata 無法解析，呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，message 指出 broken 的 metadata 檔；沒有回傳任何包

#### Scenario: 規格資料夾裡的符號連結讓匯出失敗

- **WHEN** 本機專案的 `speclink/specs/leak/spec.md` 是一個連到專案外檔案的符號連結，呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，`code` 是 `error`，message 含 `speclink/specs/leak/spec.md`、不含那個外部檔案的內容；沒有回傳任何包

#### Scenario: 規格資料夾指到專案外讓匯出失敗

- **WHEN** 本機專案的 `.speclink.yaml` 寫 `spec_dir: ../other/speclink`，那個資料夾裡有一份正式規格，呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，`code` 是 `error`，message 含 `"../other/speclink"`、不含那份正式規格的內容；沒有回傳任何包

##### Example: 規格資料夾名稱

| 規格資料夾名稱 | 結果 |
| --- | --- |
| `../other/speclink` | 失敗，message 含 `"../other/speclink"` |
| 專案外的一個絕對路徑 | 失敗，message 含那個路徑 |
| `docs/specs` | 成功，包含 `docs/specs` 裡的文件 |

#### Scenario: 空的專案

- **WHEN** 以 createEngine 指向一個剛初始化、沒有任何變更、規格與討論的本機專案，呼叫 `await engine.exportScope()`
- **THEN** 回傳的包 `formatVersion` 是 2；`documents` 只含初始化時就存在的文件（例如流程設定），沒有的話是空陣列

#### Scenario: 已封存的變更與討論也在包裡

- **WHEN** 本機專案有一個已封存的變更與一份已封存的討論，呼叫 `await engine.exportScope()` 並匯入宿主
- **THEN** 宿主的已封存變更清單與 `discuss list --archived` 都列出它們，內容與本機相同
