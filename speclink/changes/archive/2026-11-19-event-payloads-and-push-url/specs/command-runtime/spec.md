## ADDED Requirements

### Requirement: 蓋章事件帶 commit 與流程設定變動的事件

審查蓋章與驗證蓋章成功時回報的領域事件（`review-stamped`、`verify-stamped`）SHALL 在這次蓋章有記下 commit 時多帶一個 `commit` 欄位，值 SHALL 與寫進變更 metadata 的 commit 相同；這次蓋章沒有記下 commit 時 SHALL NOT 帶這個欄位，事件的內容 SHALL 與引入本需求之前相同（只有變更名稱）。事件的種類名、主體識別與時間戳 SHALL 不變。

流程設定的寫入命令（單項編輯與整份取代）真的寫入時，命令層 SHALL 回報一筆領域事件，種類名 `workflow-config-changed`，內容 SHALL 含 `keys`：這次寫入變動的鍵，字串陣列，順序與寫入命令結果裡「變動的鍵」相同。只有排版或註解改變而沒有任何鍵的值改變時，`keys` SHALL 是空陣列，事件仍 SHALL 回報。命令沒有寫入時——新內容與現有內容逐位元相同、帶 dry-run、新內容被拒絕、被執行前掛鉤拒絕——SHALL NOT 回報這筆事件。事件 SHALL NOT 含設定的值或全文。

這兩項事件內容 SHALL 經既有的路徑送達：與寫入在同一次提交交給儲存、交給提交後掛鉤。推送給用戶端的訊息 SHALL 維持只指出哪個資源變了：蓋章事件照舊歸到該變更；`workflow-config-changed` SHALL 以 unknown 類別發出，用戶端據此重新讀取全部。推送的訊息 SHALL NOT 含 `commit` 或 `keys`。動詞的 stdout、stderr、`--json` 輸出與 exit code SHALL 不因本需求改變。

#### Scenario: 帶 commit 的蓋章事件

- **WHEN** 以記下 commit `0123456789abcdef0123456789abcdef01234567` 的蓋章請求對變更 demo 完成審查蓋章
- **THEN** 回報一筆 `review-stamped` 事件，內容含變更名稱 demo 與 `commit: "0123456789abcdef0123456789abcdef01234567"`；變更 metadata 記下的 commit 是同一個值

#### Scenario: 沒有 commit 的蓋章事件維持原樣

- **WHEN** 蓋章的請求沒有帶 commit，對變更 demo 完成驗證蓋章
- **THEN** 回報一筆 `verify-stamped` 事件，內容只有變更名稱，沒有 `commit` 這個欄位

#### Scenario: 改一個政策鍵回報變動的鍵

- **WHEN** 流程設定的 tdd 目前是 false，執行把 tdd 設成 true 的寫入命令
- **THEN** 回報一筆 `workflow-config-changed` 事件，`keys` 是 `["tdd"]`；事件不含 true 或設定的其他內容

##### Example: 各種寫入對應的事件

| 寫入 | 是否真的寫入 | 事件 | `keys` |
| --- | --- | --- | --- |
| tdd 由 false 改成 true | 是 | 一筆 `workflow-config-changed` | `["tdd"]` |
| 整份取代，同時改了 locale 與 context | 是 | 一筆 | `["locale", "context"]` |
| 整份取代，只改了註解 | 是 | 一筆 | `[]` |
| tdd 設成它現在的值 | 否 | 沒有 | — |
| 帶 dry-run 的任何寫入 | 否 | 沒有 | — |
| locale 設成不合法的值 | 否（被拒絕） | 沒有 | — |

#### Scenario: 宿主在提交與提交後掛鉤收到事件

- **WHEN** 經宿主套件執行把 audit 設成 true 的流程設定寫入，宿主的儲存記下 commit 函式收到的事件，提交後掛鉤也記下它收到的事件
- **THEN** 兩邊都有一筆 `workflow-config-changed`，`keys` 是 `["audit"]`

#### Scenario: 推送的訊息不帶內容

- **WHEN** 訂閱者連上事件串流後，另一個請求完成一次帶 commit 的蓋章與一次流程設定的寫入
- **THEN** 訂閱者收到兩筆 invalidation：第一筆 scope 為 change、resourceId 為變更名稱；第二筆 scope 為 unknown；兩筆都沒有 `commit` 或 `keys`
