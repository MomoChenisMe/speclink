## ADDED Requirements

### Requirement: 桌面清單與單一變更資料帶階段、建立者識別、就緒日與修改時間

桌面資料源交給前端的資料 SHALL 增列下列選填欄位（皆 camelCase）：

- 變更清單的每一項：`stage`（`proposed`、`in-progress` 或 `ready`）與 `createdById`。本機專案 SHALL 由桌面 core 以引擎的階段函式與 metadata 填入；沒有識別時 `createdById` SHALL 為 null；metadata 損壞的項目 SHALL NOT 帶 `stage`。遠端專案 SHALL 取自 Server 變更清單回應的同名欄位，回應沒有時欄位缺席。
- 單一變更的資料：`readyAt` 與 `createdById`。本機專案的 `readyAt` SHALL 只在變更目前至少有一個任務並且全部勾完、而且 metadata 記有就緒時間時有值（日期 `YYYY-MM-DD`），判定 SHALL 用引擎的同一條顯示規則；遠端專案 SHALL 取自 Server 單一變更回應的同名欄位。
- 討論清單的每一項：`modifiedAt`（RFC 3339 的 UTC 時間）與 `createdById`，本機與遠端 SHALL 皆取自引擎的討論資訊或 Server 的討論清單回應，缺席即省略。

這些欄位屬桌面資料源的組裝，SHALL NOT 改動 CLI 的 `--json` 輸出、Server 的回應與協定型別。前端對缺席的欄位 SHALL 以未知容錯：`stage` 缺席時自行推導、`createdById` 缺席時依名稱、`readyAt` 與 `modifiedAt` 缺席時不顯示對應的段落，SHALL NOT 偽造預設值。既有欄位 SHALL 不變。

#### Scenario: 本機清單帶階段與識別

- **WHEN** 本機專案有變更 demo（任務 3/3、metadata 沒有 `created_by_id`），桌面 core 組出變更清單
- **THEN** demo 這一項的 `stage` 為 `ready`、`createdById` 為 null；其餘欄位與這條需求加入之前相同

#### Scenario: 本機單一變更資料的就緒日

- **WHEN** 本機專案的變更 demo 任務全部勾完、metadata 有 `ready_at: 2026-11-25T09:00:00+08:00`，桌面 core 回傳單一變更的資料；之後取消勾選一個任務再取一次
- **THEN** 第一次的 `readyAt` 為 `2026-11-25`；第二次沒有值

#### Scenario: 遠端清單沿用回應的欄位

- **WHEN** 桌面 app 連到變更清單回應帶 `stage` 與 `createdById`、討論清單回應帶 `modifiedAt` 的 Server
- **THEN** 前端收到的變更清單項與討論清單項帶同名欄位，值與回應相同；連到不提供這些欄位的舊版 Server 時欄位缺席，清單照常載入
