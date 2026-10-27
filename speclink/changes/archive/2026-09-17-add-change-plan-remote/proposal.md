## Why

前兩刀讓 local 模式有了執行順序：引擎算波次、CLI 查詢與寫入、桌面看板與系統匣照順序呈現。remote 模式（桌面連 server 的工作區、CLI 的 remote 模式）還缺整條路：server 沒有 plan 端點、`depends_on` 沒有寫入端點、CLI 的兩個新動詞在 remote 模式被拒、remote 看板仍只照 board resource 排。本變更是討論 change-execution-order 的第三刀（最後一刀）：讓 remote 與 local 讀同一份計算、寫同一個欄位，桌面 remote 分頁與 CLI remote 模式的行為和 local 一致。目標使用者是團隊模式下共用 server 的開發者與 PM，對應 apply 前的挑選與看板排隊。

第二刀封存時接受了一條限制、留給本刀收尾：local 主 checkout 開 worktree 映射時，排程分頁的新增候選來自看板清單（主名冊＋副本），但前置寫入作用在該變更的 worktree 副本，分支後才在主 checkout 建立的變更選了會被引擎拒絕。本變更一併把候選改為該變更所在的名冊。

## What Changes

- server 新增唯讀端點 `GET /plan`：經 Command gateway 執行引擎的 plan 計算，rank 來源為 scope 的 board resource（寬鬆解析，壞內容視為全員缺 rank），回應與 CLI `plan --json` 同形並附 scope ETag；reader 與 editor 皆可用、零寫入；依賴成環回 409。
- server 新增寫入端點 `POST /changes/{name}/depends`（body `{ on: string[], remove: bool }`）：經 Command gateway 直通引擎的依賴寫入命令，editor 限定；change 不存在 404，守門失敗（自依賴、目標不存在或已封存、成環）409；實際改動時發布領域事件、scope revision 前進、SSE invalidate。
- 兩個端點的 409 一律沿用封閉 error reason registry 的 `refused`，message 為引擎原文（如 `dependency cycle: a -> b -> a`）——不新增 reason 值；typed client 原樣轉發 message，CLI 印出的句子與 fs 模式相同。
- protocol 新增 `PlanResponse`、`PlanWave`、`PlanChangeEntry`、`PlanOverlap`、`PlanSkipped`、`SetDependsRequest`、`SetDependsResponse`；board resource 內容的寬鬆解析（`BoardOrderDoc`）自桌面移入 protocol，server 的 plan 端點與桌面 overlay 共用同一份；remote client 新增 `plan()` 與 `set_depends()`。
- CLI `plan` 與 `change depends` 自 FsOnly 升為 Dual：remote 臂呼叫上述端點，人眼與 `--json` 輸出與 fs 模式同形。
- 桌面 remote 分頁：變更清單改以 `GET /plan` 的配置順序排列，並疊 wave／blockedBy／dependsOn／overlaps 四欄位與 planError（plan 回 409 成環時退回 board overlay 順序、四欄缺席、planError 為訊息；404 舊 server 或其他錯誤時同樣退回、planError 為 null）；拖排寫回 board resource 前以引擎純函式檢查宣告依賴（plan 略過的壞 meta 卡不列入檢查，與 local 同構），違反即單行錯誤；排程分頁的前置編輯對 editor 開放（capability `setDepends` 隨 editor role），走新的 Tauri command 呼叫 `set_depends`。
- 桌面 local 排程分頁：新增前置的候選改為該變更所在名冊的作用中變更（有 worktree 映射時為其副本、否則為主 checkout），經新的 Tauri command `depends_candidates` 與資料源選配方法 `dependsCandidates` 取得；未提供此方法的資料源（remote——scope 只有一份名冊）照舊自清單派生。
- 相容性影響：`GET /board-order` 與 `PUT /board-order` 行為不變（server 對 PUT 內容仍不校驗語意，只有 plan 端點以寬鬆方式讀 rank）；既有 list 端點與 ChangeSummary 不變；error reason registry 不變；桌面 remote 清單 payload 只加欄位；舊 server 對新 client 的 plan 請求回 404 時 client 退回無排程資訊的舊行為；local 排程分頁在無 worktree 映射時候選集合與本變更前相同（順序改依名稱排序）；桌面前端資料源介面的 `listChanges` 改為回傳 `{ changes, planError }` 整包（併入第二刀的選配 `listChangesWithPlan`，屬 apps/desktop 與 packages/ui 的內部介面，無外部使用者）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `change-plan`: 新增「plan 與 change depends 的 remote 臂」需求（remote 模式下兩動詞經 server 端點、輸出同形；remote 拖排以純函式檢查）。
- `server-verb-api`: 新增「plan 唯讀衍生查詢端點」與「變更依賴寫入端點」兩條需求。
- `client-protocol`: 新增「plan 回應 payload」「依賴寫入請求與回應」「remote 變更清單的排程欄位」三條需求。
- `verb-contract`: 「模式分岔的單點宣告」的 plan 與 change 自 FsOnly 移入 Dual。
- `remote-board-order`: 「board resource 為 scope 單文件且 server 不解析」放行 plan 端點的寬鬆讀取；「remote 排序 overlay 與本地語意同構」改為變更卡依 plan 配置順序、討論卡維持 rank overlay；「拖排寫回以全文 CAS 與一次重試收斂」改為讀取時一併取 plan、依顯示序推導欄成員、rank 序與顯示序不一致時整欄重派、違反宣告依賴（不計 plan 略過的卡）不發 PUT。
- `desktop-app`: 「看板卡片的波次與阻擋標示」的缺 wave 情形改為 remote 未取得 plan（舊 server、請求失敗、成環）或 local 成環；「依賴成環時看板提示」的退回序分寫 local 基底序與 remote board overlay 序；「詳情抽屜的排程分頁」的新增候選改為該變更所在名冊（worktree 映射時為其副本），並明定名冊查詢未完成、首次失敗與重新查詢失敗時的呈現。

## Impact

- Affected specs: `change-plan`、`server-verb-api`、`client-protocol`、`verb-contract`、`remote-board-order`、`desktop-app`（修改）
- Affected code:
  - New: crates/host/speclink-server/tests/it/api/plan_api.rs、crates/adapters/speclink-cli/tests/it/remote_plan.rs、apps/desktop/src/__tests__/helpers/changeList.ts（listChanges 替身的 payload 組裝）；docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 的 `GET /plan`、`POST /changes/{name}/depends` 兩段（兩份文件的模式表同步把 plan、change 移到 Dual）
  - Modified: crates/engine/speclink-core/src/command/mod.rs 與 command/tests.rs（`Command::Plan` 加 ranks 欄位）、crates/engine/speclink-core/src/lifecycle/plan.rs（抽出本機與 remote 共用的 `blocked_move`）、crates/engine/speclink-core/src/lifecycle/model.rs（`Stage::parse`，`as_str` 的反查）、crates/protocol/speclink-protocol/Cargo.toml（serde_json 升為一般相依）、crates/protocol/speclink-protocol/src/query.rs（plan DTO、BoardOrderDoc）、crates/protocol/speclink-protocol/src/command.rs（SetDependsRequest、SetDependsResponse）、crates/protocol/speclink-remote/src/client.rs（plan、set_depends）、crates/protocol/speclink-remote/src/convert.rs（PlanResponse → 引擎 PlanReport，違約回錯）、crates/protocol/speclink-remote/tests/it/typed_client.rs（plan、set_depends 與 409 refused 原文轉發測試）、crates/host/speclink-server/src/app.rs（兩條 route）、crates/host/speclink-server/src/api/routes.rs（兩個 handler）、crates/host/speclink-server/src/api/error.rs（plan 成環 → 409 refused）、crates/host/speclink-server/src/api/events.rs（change-depends-changed 歸 change 類 invalidate）、crates/host/speclink-server/tests/it/api/mod.rs、crates/host/speclink-host/src/bridge.rs（depends 寫入的 command 標籤）、crates/adapters/speclink-cli/src/main.rs（plan 與 change 改 dual）、crates/adapters/speclink-cli/src/verbs/plan.rs（remote 臂）、crates/adapters/speclink-cli/tests/it/main.rs、crates/adapters/speclink-cli/tests/it/mode_dispatch.rs、apps/desktop/src-tauri/src/remote.rs（list 合併 plan、reorder 前檢查、set_depends、BoardOrderDoc 改用 protocol）、apps/desktop/src-tauri/src/lib.rs（remote_set_change_depends 與 depends_candidates command）、apps/desktop/src-tauri/tests/it/remote_data.rs、apps/desktop/src-tauri/tests/it/phase3_chain.rs（清單項欄位存取）、apps/desktop/core/src/manage.rs（depends_candidates_at；本地拖排改用共用的重派判定）、apps/desktop/core/src/lib.rs（`home_store_for`，前置寫入與本地拖排共用的所在 store 解析）、apps/desktop/core/src/query.rs（`plan_placed`，本地 board_order 與 remote 共用的顯示序排列）、apps/desktop/core/src/rank.rs（`ranked_in_order`，本地與 remote 拖排共用的整欄重派判定）、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/session.ts、apps/desktop/src/store.ts（refresh 直接讀 listChanges 整包、移除退路）、apps/desktop/src/App.tsx、apps/desktop/src/__tests__/remoteDataSource.test.ts、apps/desktop/src/__tests__/remoteCapabilities.test.tsx、apps/desktop/src/__tests__/planDataSource.test.ts、apps/desktop/src/__tests__/tauriDataSource.test.ts、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/workspace.test.ts、apps/desktop/src/__tests__/remoteResilience.test.tsx、apps/desktop/src/__tests__/helpers/remoteFixtures.ts（listChanges 替身回傳整包）、packages/ui/src/adapter.ts（dependsCandidates 選配方法；listChanges 回傳 `{ changes, planError }`、併入 listChangesWithPlan）、packages/ui/src/components/RichDetailDrawer.tsx（候選載入）、packages/ui/src/__tests__/planTab.test.tsx
  - Removed: 無
- 前置相依：add-change-plan-engine（引擎 `compute_with_ranks`、`violations`、`set_depends`、`ChangeDependsChanged` 事件）與 add-change-plan-desktop（`ChangeItem` 四欄位、排程分頁、資料源 `setDepends` 與 `listChangesWithPlan` 介面）必須先落地。
- 不動的部分：local 模式的 plan 計算、清單排序與拖排的行為（實作改呼叫與 remote 共用的排列與重派判定函式）；board resource 的存放格式與 PUT 校驗；error reason registry；apps/server-web。
