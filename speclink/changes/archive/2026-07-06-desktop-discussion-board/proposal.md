## Why

discuss 是 Speclink 相對 Spectra 的自有增強：討論記錄（openspec/discussions/）是 propose 的前站、可扇出多個 change，但桌面 app 完全看不到它——生命週期的第 0 站在看板上缺席，開著的討論、已結論待促轉的討論、促轉後子 change 的同源關係，全都只能回 CLI 查。目標使用者：透過桌面 GUI 追蹤 SDD 全流程的開發者與 PO/PM——尤其 PO/PM 正是討論的主要參與者。本案來源：討論「桌面即時刷新與封存瀏覽」（2026-07-06 結論，第二刀），依賴第一刀 desktop-board-parity 的檔案監看（已涵蓋 discussions/）、封存頁展開結構與看板 stage 派生。

## What Changes

- **A 看板擴為四欄，「討論」為第 0 欄，兩級呈現**：open（討論中）與 concluded（已結論）用全尺寸卡——討論中卡片唯讀（回合推進在 CLI／agent），已結論卡片帶促轉與歸檔兩個動詞；promoted（已促轉）降級為欄底收合細列，列出各子 change 的名稱 chip 與階段點（提案中／進行中／已就緒／已封存），全數歸檔時討論隨引擎既有的自動歸檔離板。
- **B 討論抽屜**：脈絡／回合／結論／促轉四分頁（前三者以記錄文件的區段切分渲染）；促轉分頁列出各子 change 現況與跳轉，並提供「再促轉」（一份討論扇出多刀）。
- **C change 側同源連結**：來自討論的 change 卡帶討論徽章；詳情抽屜標頭顯示「來自討論」並列出同源 change（兄弟刀）互跳。
- **D 已封存頁擴為雙節**：既有「變更」節之外新增「討論」節，封存討論唯讀展開（含「不做」收尾與隨最後子 change 自動歸檔者）。
- **E 引擎配套（行為不變的重構與補洞）**：促轉的流程邏輯（衍生 change 名、建 change 帶 from_discussion、以結論預填提案、標記 promoted）自 CLI 層下沉至 speclink-core 供桌面與 CLI 共用——CLI 的 discuss promote 輸出與行為不變；core 新增 promoted_to 讀取（DiscussionInfo 序列化不變，CLI discuss list --json 輸出不變）。
- **F 子 change 被刪除（非歸檔）的呈現定案**：chip 標示「已刪除」、討論維持 promoted 不回退（歷史事實不回滾）、再促轉恆可用——純 GUI 派生（以 change 於 active 與 archive 清單的存在性判定），引擎零變更。

## Non-Goals

- GUI 執行 conclude、add-round、set-context、new、discard——討論的推進與結論撰寫屬 agent／CLI，GUI 只做檢視與兩個推進動詞（促轉、歸檔）。
- 討論記錄的 GUI 編輯；討論內容的搜尋。
- per-discussion 顏色編碼在看板分組同源 change（討論同時在飛者個位數，徽章＋同源清單已足）。
- 任何 web／remote 端實作（討論瀏覽騎在 Store 上，remote 屆時實作既有 discussions 方法即得——僅落現況註記）。
- CLI 討論指令的輸出或行為變更。

## Capabilities

### New Capabilities

（無——引擎的討論生命週期行為（promote 累積、自動歸檔）不變，E 為等價重構；新行為全數落在桌面呈現。）

### Modified Capabilities

- `desktop-app`: 新增三項需求——討論於看板第 0 欄兩級呈現、討論抽屜檢視與 GUI 促轉、已封存頁含討論節。既有需求不變。

## Impact

- Affected specs: 修改 `desktop-app`。
- Affected crates:
  - `speclink-core`：promote 流程自 CLI 下沉（新 pub 函式）、promoted_to 查詢函式。
  - `speclink-cli`：discuss promote 呼叫點改用下沉後的 core 函式，輸出零變更。
  - `speclink-desktop-core`（apps/desktop/core）：discussions 查詢橋接（清單含 promoted 子 change 現況、記錄全文）與促轉／歸檔命令橋接。
  - `speclink-desktop`（apps/desktop/src-tauri）：對應 Tauri command。
- Affected code:
  - Modified: crates/speclink-core/src/discuss.rs、crates/speclink-cli/src/commands.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src/App.tsx、apps/desktop/src/adapter/tauriDataSource.ts、packages/ui/src/adapter.ts、packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx
  - New: apps/desktop/core/src/discussions.rs、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/DiscussionDrawer.tsx
  - Removed: （無）
- 相容性影響：CLI 的 discuss 全部子指令（list、show、promote、archive 等）stdout、--json 欄位與 exit code 位元級不變——promote 下沉為等價重構、promoted_to 查詢不進 DiscussionInfo 序列化；parity／color 對照不受影響（discuss 為 Speclink 自有指令，以自我基線護欄）。
- 設定欄位：無。技能與注入區塊：無。
- 依賴：第一刀 desktop-board-parity 須先完成（監看事件、封存頁展開結構、stage 派生為本刀前置）。
