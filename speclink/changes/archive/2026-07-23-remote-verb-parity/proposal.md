## Why

remote-data-source 刀依「server 缺什麼就停用什麼」凍結原則，如實停用了 server 沒有端點的動詞與寫入面：validate/analyze 動詞、刪除變更、任務拖排。Phase 3 已關門，但這三組操作讓 remote workspace 相對本地仍是次等公民——desktop remote 分頁按鈕停用、CLI 在 remote checkout 下 validate/analyze 無遠端分流（開本地專案失敗）、discard 明確拒絕，而 RD 在 checkout 內用技能跑 speclink validate 是日常路徑。引擎面其實已就緒：Command enum 已有 Validate/Analyze/Discard 變體與 typed outcome，core discard 是 store-trait 編排，host BridgeStore 已實作 delete_change——缺的只是端點、client 方法與兩端解鎖。

## What Changes

- server 三組動詞端點（全部經既有 Command gateway，不另寫流程）：
  - GET /changes/{name}/validate 與 GET /changes/{name}/analyze——唯讀衍生查詢，沿 GET /changes/{name}/drift 的形狀，回 ValidationResult／AnalyzeReport 的 typed DTO。
  - DELETE /changes/{name}（force 參數）——走 Command::Discard 全語意（fail-closed meta 檢查、started-work guard、來源討論 unlink、UoW 原子刪除、outbox 事件→SSE invalidate）；editor role 限定。
  - POST /changes/{name}/tasks/move——新 Command::TaskMove；搬移＋重編號語意自桌面 core 遷入 speclink-core，index 定址（1-based checkbox ordinal＋before 側別）沿 UI moveTask 形狀；editor role 限定。
- speclink-protocol 新增對應 DTOs；speclink-remote client 新增四個方法。
- CLI remote 解鎖：validate／analyze 指令加 remote 分流打新端點（渲染沿本地路徑）、discard 由「remote 不可用」拒絕改為實作（--force 直通端點 force 參數）。
- Desktop 解鎖：remote.rs 新增四個指令、remoteDataSource.ts 四方法由 unsupported 拒絕改 invoke 直達、RemoteCapabilities 的 deleteChange／moveTask／validate／analyze 翻真——UI 停用 affordance（按鈕 disabled、拖排把手不渲染）由既有 capability 管線自動解除。
- 桌面本地路徑去重複：move_task_at 的行編輯邏輯遷入 core 後，桌面本地實作改薄呼叫 core 同一函式（可觀察行為不變）。

## Non-Goals

- 看板拖排（reorderCard）不在本刀——它踩到架構「不把 board/card 呈現狀態混入共享規格 revision」的明文，需要獨立 CAS board resource 設計，由後續 remote-board-order 刀處理；本刀後 capability 停用清單僅剩它。
- 桌面本地刪除不改走 discard 語意（本地 delete 現況為無 guard 直刪目錄、不 unlink 討論——既有行為，動它是另一個範圍）；本刀只在 remote 路徑採 discard 全語意。
- 任務 move 不引入 stable ID 定址 wire——UI 層未暴露任務 stable ID，index 定址與既有 ordinal 勾選路徑同等的競態邊界（server 端越界拒絕、SSE invalidate 即時矯正），stable ID 定址留待 UI 暴露任務識別後再議。
- 不做 changeCapabilities／changeMeta 的 remote 補洞（停用原因是 server payload 缺口，屬讀取面另案）。

## Capabilities

### New Capabilities

- `server-verb-api`: server 動詞端點的行為保證——validate/analyze 唯讀衍生查詢與本地同 outcome、discard 的 guard/force/unlink/原子性/role、task move 的定址語意與重編號效果、寫入動詞的事件發布。

### Modified Capabilities

- `remote-workspace-data`: capability 停用清單縮減——validate/analyze、刪除變更、任務拖排改直達；看板拖排維持停用（待 board resource 刀）。
- `verb-contract`: Command 契約新增 TaskMove 變體與其 typed outcome。
- `command-runtime`: 變更型動詞事件對應表新增 task move → task-moved 一列。

## Impact

- Affected specs: `server-verb-api`（新增）、`remote-workspace-data`（修改）、`verb-contract`（修改）、`command-runtime`（修改：事件對應表加 task-moved 列）
- Affected code:
  - New: `crates/speclink-server/tests/verb_api.rs`（新端點整合測試）
  - Modified: `crates/speclink-core/src/tasks.rs`（move＋重編號邏輯遷入）、`crates/speclink-core/src/command/mod.rs`（TaskMove 變體與 outcome）、`crates/speclink-server/src/app.rs`（路由）、`crates/speclink-server/src/routes.rs`（四 handlers）、`crates/speclink-protocol/src/command.rs` 與 `crates/speclink-protocol/src/query.rs`（DTOs）、`crates/speclink-remote/src/client.rs`（四方法）、`crates/speclink-cli/src/commands.rs`（validate/analyze remote 分流）、`crates/speclink-cli/src/remote_commands.rs`（remote_validate/remote_analyze/remote_discard）、`apps/desktop/core/src/manage.rs`（move_task_at 改呼叫 core）、`apps/desktop/src-tauri/src/remote.rs`（四指令＋RemoteCapabilities 翻真）、`apps/desktop/src-tauri/src/lib.rs`（指令註冊）、`apps/desktop/src/adapter/remoteDataSource.ts`（四方法直達）、`apps/desktop/src/__tests__/remoteDataSource.test.ts`（既有測試更新）
  - Removed: （無）
