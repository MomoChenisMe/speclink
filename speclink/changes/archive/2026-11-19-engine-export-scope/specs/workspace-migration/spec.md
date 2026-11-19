## ADDED Requirements

### Requirement: 本機專案的匯出與遷移產生相同的文件集合

對同一個本機專案，Node SDK 的 engine.exportScope() 回傳的包與 Speclink Desktop「遷移到 Server」送出的匯入包 SHALL 含相同的文件集合：把 Desktop 的包轉成以鍵標示的形式之後，兩者的鍵、內容與摘要 SHALL 逐項相同。兩者組裝包時的中止條件 SHALL 相同：會讓 Desktop 的遷移中止的來源（壞掉的變更 metadata、引擎不支援的文件形式、規格資料夾裡的符號連結、不在專案根目錄裡面的規格資料夾）SHALL 同樣讓 exportScope 失敗，並指出同一個檔案。Desktop 的遷移另外只接受預設的規格資料夾（`speclink/` 或 `openspec/`）：這是遷移流程在組裝之前的既有檢查，不屬於組裝的中止條件；exportScope 接受專案根目錄裡面的自訂規格資料夾（例如 `docs/specs`）。

兩個入口 SHALL NOT 經符號連結讀檔：通往規格資料夾路上的資料夾、規格資料夾本身或它底下任何一個檔案或資料夾是符號連結時（舊位置的 evidence 資料夾 `.speclink/touched/` 亦同），兩者 SHALL 在讀取任何文件之前失敗，訊息 SHALL 以相對專案根目錄、使用正斜線的路徑點名第一個找到的項目（同一層依名稱排序），SHALL NOT 含連結指向的位置或任何檔案內容。Desktop 的遷移 SHALL 在送出任何請求之前中止，本機 SHALL NOT 做備份或轉換；exportScope SHALL NOT 回傳包。這些位置有列不出內容的資料夾時，兩者 SHALL 同樣在讀取任何文件之前失敗並點名該資料夾：無法確認裡面沒有連結。規格資料夾不存在時不算這種失敗。

之後 Bundle 組裝的涵蓋範圍有任何增減（見「Bundle 組裝涵蓋本地 workspace 全集」），兩個入口 SHALL 同時跟著改變，SHALL NOT 出現一個入口有、另一個入口沒有的文件種類。Speclink Desktop 的遷移流程、確認畫面、送出的內容與成功後對本機的處理 SHALL 維持既有行為，唯一的例外是規格資料夾含有符號連結的專案：遷移改為在送出前中止（刻意變更：以前會跟著連結讀檔並上傳連結指向的內容）。

#### Scenario: 兩個入口的文件逐項相同

- **WHEN** 對一個含進行中變更（有 evidence）、已封存變更（有 evidence）、正式規格、進行中與已封存的討論、流程設定、詞彙表與看板順序的本機專案，分別取得 Speclink Desktop 的遷移匯入包與 engine.exportScope() 的包
- **THEN** 把前者轉成以鍵標示的形式後，兩者的文件數量相同，每一項的鍵、內容與摘要都相同

#### Scenario: 中止條件相同

- **WHEN** 本機專案有一個已封存變更的 metadata 無法解析，分別執行 Desktop 的遷移匯入包組裝與 engine.exportScope()
- **THEN** 兩者都失敗，訊息指出同一個檔案；兩者都沒有產生包

#### Scenario: 規格資料夾裡的符號連結讓兩個入口都中止

- **WHEN** 本機專案的 `speclink/specs/leak/spec.md` 是一個連到專案外檔案（內容是 `TOP SECRET`）的符號連結，分別執行 Desktop 的遷移匯入包組裝與 engine.exportScope()
- **THEN** 兩者都失敗，訊息都含 `speclink/specs/leak/spec.md`、都不含 `TOP SECRET`；Desktop 沒有產生包、沒有送出請求；exportScope 沒有回傳包

##### Example: 會讓匯出中止的符號連結

| 符號連結的位置 | 訊息點名的路徑 |
| --- | --- |
| 規格資料夾底下的檔案 `specs/leak/spec.md` | `speclink/specs/leak/spec.md` |
| 規格資料夾底下的資料夾 `changes/linked` | `speclink/changes/linked` |
| 規格資料夾本身 | `speclink` |

#### Scenario: Desktop 的遷移行為不變

- **WHEN** 使用者在 Speclink Desktop 對一個規格資料夾裡沒有符號連結的本機專案執行「遷移到 Server」到一個空的遠端專案
- **THEN** 遷移成功，遠端專案的內容與引入本需求之前的遷移結果相同；本機的備份與轉換照既有流程進行
