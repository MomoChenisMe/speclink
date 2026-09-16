## Why

前兩刀讓 local 模式有了執行順序：引擎算波次、CLI 查詢與寫入、桌面看板與系統匣照順序呈現。remote 模式（桌面連 server 的工作區、CLI 的 remote 模式）還缺整條路：server 沒有 plan 端點、`depends_on` 沒有寫入端點、CLI 的兩個新動詞在 remote 模式被拒、remote 看板仍只照 board resource 排。本變更是討論 change-execution-order 的第三刀（最後一刀）：讓 remote 與 local 讀同一份計算、寫同一個欄位，桌面 remote 分頁與 CLI remote 模式的行為和 local 一致。目標使用者是團隊模式下共用 server 的開發者與 PM，對應 apply 前的挑選與看板排隊。

## What Changes

- server 新增唯讀端點 `GET /plan`：經 Command gateway 執行引擎的 plan 計算，rank 來源為 scope 的 board resource（寬鬆解析，壞內容視為全員缺 rank），回應與 CLI `plan --json` 同形並附 scope ETag；reader 與 editor 皆可用、零寫入。
- server 新增寫入端點 `POST /changes/{name}/depends`（body `{ on: string[], remove: bool }`）：經 Command gateway 直通引擎的依賴寫入命令，editor 限定；change 不存在 404，守門失敗（自依賴、目標不存在或已封存、成環）409 附語義化 reason；實際改動時發布領域事件、scope revision 前進、SSE invalidate。
- protocol 新增 `PlanResponse`、`PlanChangeEntry`、`SetDependsRequest`、`SetDependsResponse` 四個 DTO；remote client 新增 `plan()` 與 `set_depends()`。
- CLI `plan` 與 `change depends` 自 FsOnly 升為 Dual：remote 臂呼叫上述端點，人眼與 `--json` 輸出與 fs 模式同形。
- 桌面 remote 分頁：變更清單改以 `GET /plan` 的配置順序排列，並疊 wave／blockedBy／dependsOn／overlaps 四欄位與 planError（server 無此端點或請求失敗時退回 board overlay 順序、四欄缺席、planError 為 null）；拖排寫回 board resource 前以引擎純函式檢查宣告依賴，違反即單行錯誤；排程分頁的前置編輯對 editor 開放（capability `setDepends` 隨 editor role），走新的 Tauri command 呼叫 `set_depends`。
- 相容性影響：`GET /board-order` 與 `PUT /board-order` 行為不變（server 對 PUT 內容仍不校驗語意，只有 plan 端點以寬鬆方式讀 rank）；既有 list 端點與 ChangeSummary 不變；桌面 remote 清單 payload 只加欄位；舊 server 對新 client 的 plan 請求回 404 時 client 退回無排程資訊的舊行為。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `change-plan`: 新增「plan 與 change depends 的 remote 臂」需求（remote 模式下兩動詞經 server 端點、輸出同形；remote 拖排以純函式檢查）。
- `server-verb-api`: 新增「plan 唯讀衍生查詢端點」與「變更依賴寫入端點」兩條需求。
- `client-protocol`: 新增「plan 回應 payload」「依賴寫入請求與回應」「remote 變更清單的排程欄位」三條需求。
- `verb-contract`: 「模式分岔的單點宣告」的 plan 與 change 自 FsOnly 移入 Dual。
- `remote-board-order`: 「board resource 為 scope 單文件且 server 不解析」放行 plan 端點的寬鬆讀取；「remote 排序 overlay 與本地語意同構」改為變更卡依 plan 配置順序、討論卡維持 rank overlay。

## Impact

- Affected specs: `change-plan`、`server-verb-api`、`client-protocol`、`verb-contract`、`remote-board-order`（修改）
- Affected code:
  - New: crates/host/speclink-server/tests/plan_api.rs、crates/adapters/speclink-cli/tests/it/remote_plan.rs
  - Modified: crates/protocol/speclink-protocol/src/query.rs（PlanResponse、PlanChangeEntry）、crates/protocol/speclink-protocol/src/command.rs（SetDependsRequest、SetDependsResponse）、crates/protocol/speclink-remote/src/client.rs（plan、set_depends）、crates/protocol/speclink-remote/src/convert.rs（PlanResponse ↔ 引擎 PlanReport）、crates/host/speclink-server/src/app.rs（兩條 route）、crates/host/speclink-server/src/api/routes.rs（兩個 handler）、crates/host/speclink-host/src/bridge.rs 與 commit.rs（ChangeDependsChanged 事件的 commit 與 invalidate）、crates/adapters/speclink-cli/src/main.rs（plan 與 change 改 dual）、crates/adapters/speclink-cli/src/verbs/plan.rs（remote 臂）、crates/adapters/speclink-cli/tests/it/mode_dispatch.rs、crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs、apps/desktop/src-tauri/src/remote.rs（list 合併 plan、reorder 前檢查、set_depends）、apps/desktop/src-tauri/src/lib.rs（remote_set_change_depends command）、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/session.ts（remote setDepends capability）、apps/desktop/src/__tests__/remoteDataSource.test.ts、apps/desktop/src/__tests__/remoteCapabilities.test.tsx、apps/desktop/src-tauri/tests/remote_data.rs
  - Removed: 無
- 前置相依：add-change-plan-engine（引擎 `compute_with_ranks`、`violations`、`set_depends`、`ChangeDependsChanged` 事件）與 add-change-plan-desktop（`ChangeItem` 四欄位、排程分頁、資料源 `setDepends` 介面）必須先落地。
- 不動的部分：local 模式的任何行為；board resource 的存放格式與 PUT 校驗；apps/server-web。
