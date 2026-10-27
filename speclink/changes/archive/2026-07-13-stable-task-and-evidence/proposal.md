## Why

任務目前以 ordinal 定址：CLI 的 task done／undone 吃「第 N 個 checkbox」、桌面 TaskList 以序數樂觀翻轉、領域事件 task-completed 載荷也是序數。多人或多 session 環境下 tasks.md 被重排或增刪後，「第 3 項」可能已是另一個任務——這是平台架構藍圖 §15.1 P0「task ordinal 不是穩定身分」列與 P0 驗收條件 4（同一任務經重排後仍由相同 stable ID 定址）指名的缺口，也是重構路線圖 §3.5 的主題。同時任務完成與驗證缺少可稽核的證據：task done 只記 touched 檔案清單，沒有 actor、repo、commit 與規格基準；verify 沒有固定 basis 的 VerifyBundle，規格中途變動時舊結果無從判 stale（P0 驗收條件 5：apply 開始記錄 spec revision，中途變動明確失敗而非靜默混用；藍圖 §15.4：任一 basis 改變即以 stale evidence 拒絕）。路線圖 §3.3 的責任表要求 evidence 由 client 產生、Host 驗證與保存、archive trace 由已接受且未 stale 的 evidence 建立——本刀在本地模式落地這些形狀，為 Phase 2 的 Server 端儲存鋪路。

目標使用者：透過 AI 代理跑 SDD 的開發者與 PM——apply 階段的任務勾選、verify 階段的證據、archive 的 trace 全部經過這裡；桌面使用者的任務清單改以穩定身分呈現與操作。

## What Changes

- **task stable ID**（路線圖 §3.5 的五條規則）：tasks.md 的任務行內嵌不可變 ID 註解（形如 speclink-task 標記加 tsk_ 前綴的 ULID）。Engine 產出 tasks artifact 時全檔指派 ID；task done 遇目標行無 ID 時於同一次寫入補該行 ID；重複 ID 使 task 動詞拒絕。reorder 或編輯只改順序與顯示編號，ID 不變。ordinal 只作顯示與舊 CLI 相容，不作永久身分；不以內容 hash 代替 ID。
- **定址值域擴充（BREAKING 邊界縮小）**：task done 與 task undone 的 task-id 參數接受既有數字 ordinal（行為與輸出逐位元不變）與新的 tsk_ 前綴 stable ID；「非數字即錯」的既有錯誤敘述改為「非數字且非 tsk_ 前綴即錯」。領域事件 task-completed／task-uncompleted 載荷改攜 stable ID（事件契約為 experimental，允許不相容調整）。
- **task-done evidence**：task done 時寫入的追蹤記錄由檔案清單演進為逐任務證據——task stable ID、actor（來自 ExecutionContext）、repo（來自 binding）、head commit、touched files 與規格基準 digest（spec／tasks／policy）；舊格式記錄可讀（向後相容）。task undone 維持純狀態翻轉、不寫任何記錄。
- **VerifyBundle 與 stale 判定**：speclink-host 提供固定 basis 的 VerifyBundle 產生（change、任務 ID 清單、spec／tasks／policy digest）與 evidence 的 stale 判定 API（任一 basis digest 不符即判 stale 拒絕）；本刀不新增 CLI 動詞、不強制本地流程（接線屬順位 7 與 Phase 2）。
- **archive trace 來源演進**：archive 注入的 trace 改由 evidence 記錄建立（輸出格式與現行一致），archive gate 的 evidence 檢查函式在 host 提供、本地模式不強制。
- **桌面與 UI stable ID 化**：任務解析剝離 ID 註解後顯示（使用者不見標記）、清單以 stable ID 作 key、勾選經 stable ID 定址；ordinal 僅餘顯示編號。

## Non-Goals

- 不拆分 spec drift 與 code/git drift（順位 6 drift-client-server-split）。
- 不定案 Protocol／Client SDK、不做 evidence 上行的網路傳輸（順位 7 與 Phase 2）；Server 端 evidence 儲存與驗證服務屬 Phase 2。
- 不做 approval 綁 revision 的完整 review gate（drafting→review→ready 的 approval 語意屬後續刀）；lifecycle gate 仍維持順位 4 的不強制狀態。
- 不新增 task reorder 動詞或 UI 拖排功能——只保證「重排編輯不改 ID」的不變式。
- 不改 task done／undone 對數字 ordinal 的既有輸出與錯誤（逐位元凍結）；tasks.md 檔面的 ID 註解是唯一刻意的檔面變更。
- 不動 Node dispatch 的動詞覆蓋（維持 list／status／new／claim，不含 task 動詞）。

## Capabilities

### New Capabilities

- `task-identity`: 任務穩定身分——ID 格式與內嵌標記、Engine 指派與單行補章、重複 ID 拒絕、tsk_ 定址與 ordinal 相容、reorder 不變式、UI 剝離顯示與 stable key。
- `verify-evidence`: 任務完成與驗證證據——task-done evidence 的欄位契約與向後相容、VerifyBundle 的固定 basis、stale 判定、archive trace 由 evidence 建立。

### Modified Capabilities

- `verb-contract`: 任務取消勾選動詞的 task-id 值域由「僅數字」擴充為「數字或 tsk_ 前綴 stable ID」，非法值錯誤敘述同步修訂；純狀態翻轉契約不變。

## Impact

- 影響的 crate 與套件：`speclink-core`（任務解析／蓋章／定址／事件載荷、archive trace 來源、touched 記錄演進）、`speclink-host`（VerifyBundle、stale 判定、archive gate 檢查函式）、`speclink-cli`（task-id 值域與錯誤訊息）、`@speclink/ui` 與 desktop core（剝離顯示、stable key、勾選定址）；新增 ULID 產生的輕量依賴。
- 相容性影響：數字 ordinal 的 task 動詞輸出逐位元不變，parity／color／twin 全綠；tasks.md 檔面新增 ID 註解是刻意變更（產出時全檔、task done 時單行補章），既有無 ID 的 tasks.md 繼續可用 ordinal 操作；touched 記錄新格式向下可讀舊檔；桌面顯示剝離註解，使用者可見文字不變。
- Affected specs: `task-identity`（新增）、`verify-evidence`（新增）、`verb-contract`（修改）。
- Affected code:
  - New: crates/speclink-host/src/evidence.rs、crates/speclink-cli/tests/task_stable_id.rs
  - Modified: crates/speclink-core/src/tasks.rs、crates/speclink-core/src/command/mod.rs（事件型別與雙值域 dispatch）、crates/speclink-core/src/archive.rs、crates/speclink-core/src/newcmd.rs（tasks artifact 產出全檔蓋章）、crates/speclink-cli/src/commands.rs、crates/speclink-node/src/lib.rs（ExecutionContext 組裝點）、crates/speclink-host/src/lib.rs、apps/desktop/core/src/manage.rs（desktop 任務動詞雙值域）、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src/App.tsx、apps/desktop/src/adapter/tauriDataSource.ts、packages/ui/src/tasks.ts、packages/ui/src/adapter.ts、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/RichDetailDrawer.tsx、Cargo.toml、crates/speclink-core/Cargo.toml、crates/speclink-host/Cargo.toml、Cargo.lock
  - Removed: 無
