## Why

平台架構藍圖 §7 定義 Agent Context Projection：遠端正典、本地唯讀 snapshot——Agent 以 Read/Search/Grep 操作 `.speclink/context/` 下的投影，而不是逐文件 API round trip，也不是可雙向寫入的第二份正典（§2.5）。重構路線圖 §3.8 指出這條路尚未接線：remote instructions 告訴 Agent 不要讀本地規格檔，但 apply／verify skill 會讀 instructions 的 contextFiles，CLI 尚未 materialize 這些遠端文件——遠端模式下 Agent 的檔案閱讀體驗目前是斷的。藍圖 §7.2 的 Context 規則明定交付面：永遠 gitignored、可刪除重建、文件帶 snapshot ID 與 revision 與 digest、staging 產生完整 snapshot 後 atomic switch（不逐檔覆寫閱讀中的 context）、盡可能唯讀、每次 command 前驗證 manifest digest 偵測被修改的 projection 並 fail closed、事件只標 stale 不偷換文件。§7.3 並要求依流程（discuss／propose／apply／verify／archive）縮小預設 context。本刀與 protocol-typed-client 合計構成路線圖 §4.2 順位 7；protocol 刀已定 Context API 的 DTO 與 ContextManifest 形狀，本刀交付 Materializer 與 skill 接線。

目標使用者：遠端模式下執行 speclink 技能的 AI 代理與開發者——apply／verify／drift 的 context 閱讀路徑；以及 Phase 2 Server 實作者——Context API 端點以本刀的 materializer 為既成消費者。

## What Changes

- **Context Materializer 落 speclink-host**：消費 protocol 刀的 Context snapshot DTO，將投影寫入 workspace root 的 speclink 工作目錄下 context 子目錄——manifest.json（snapshot ID、policy revision、逐文件 digest 與 revision，對齊 protocol ContextSnapshot 既有欄位）、INDEX.md 與 openspec 鏡像佈局（config、LANGUAGE、specs、changes）。
- **staging 加 atomic switch**：先在 staging 目錄產生完整 snapshot 再原子切換，不逐檔覆寫 Agent 正在閱讀的 context；投影可隨時整目錄刪除重建。
- **唯讀與完整性防護**：materializer 盡可能設定檔案唯讀屬性；提供 command 前的 manifest digest 驗證——投影被修改時 fail closed（拒絕並要求 refresh），不把改動當遠端寫入。
- **stale 標記**：提供將現有投影標記 stale 的操作（寫 stale marker，不改文件內容）；refresh 建立新 snapshot。
- **gitignore 保證**：投影目錄確保被 gitignore 涵蓋（沿 init／update 對 speclink 工作目錄的既有管理）。
- **依流程縮小 context**：materialize 接受流程參數（discuss／propose／apply／verify／archive），依藍圖 §7.3 的預設集合挑選文件；預設全量亦可用。
- **snapshot 來源以介面注入**：本刀以測試替身（本地 Store 快照與 stub 回應）驗證 materializer；真實 HTTP Context API 來源由 Phase 2 Server 接線。
- **remote skill 與 instructions 接線**：remote 模式的 instructions 將 contextFiles 指向投影路徑；apply／verify 等 skill 文案明確要求讀投影、禁止把直接修改投影當成遠端寫入。技能內容變更遵守三處同步：crates/speclink-core/assets、repo 技能實例（.claude/skills 與 .agents/skills）、render golden 於乾淨樹再生。
- 本地 fs 模式不建立投影、行為與輸出零變更。

## Non-Goals

- 不實作 Server 端 Context API 端點與 HTTP 傳輸（Phase 2 reference-server）。
- 不做 gitdir 投影位置選項（藍圖 §7.2 的 projection.location: git-dir 屬部署選項，非 portable default）。
- 不做無 checkout 情境的 MCP resources、Tool-native Context 與 Desktop app data 投影（Phase 3／4）。
- 不做事件驅動的自動 stale（push event 接線屬 Phase 2）；本刀 stale 為顯式操作。
- 不改本地 fs 模式的 instructions 與 skill 行為；人眼與 --json 輸出凍結（remote instructions 的 contextFiles 路徑值是刻意變更）。
- 不做投影內容的增量更新——每次 refresh 全量重建（藍圖明定可丟棄語意）。

## Capabilities

### New Capabilities

- `context-projection`: Agent Context Projection 的契約——投影佈局與 manifest、staging 加 atomic switch、唯讀與 digest 完整性 fail closed、stale 標記與 refresh、gitignore 保證、依流程縮小 context、remote skill 讀投影且禁止寫回。

### Modified Capabilities

（無）——本地模式行為不變；remote instructions 的 contextFiles 指向調整屬新 capability 的需求範圍。

## Impact

- 影響的 crate 與資產：`speclink-host`（materializer、manifest 驗證、stale）；`speclink-core`（remote instructions 的 contextFiles 指向、skill 資產文案）；`speclink-cli`（remote 模式的 materialize 觸發點）；內嵌技能三處同步（crates/speclink-core/assets 下 skills、倉庫技能實例、render golden 測試快照）。
- 相容性影響：本地 fs 模式零變更；remote 模式 instructions 的 contextFiles 值改指投影路徑（刻意變更，twin 對照同步更新）；投影目錄為新檔面、gitignored、可刪除重建。parity／color 全綠；render golden 於乾淨樹再生並審視 diff。
- Affected specs: `context-projection`（新增）。
- Affected code:
  - New: crates/speclink-host/src/projection.rs
  - Modified: crates/speclink-host/src/lib.rs、crates/speclink-core/src/instructions.rs、crates/speclink-core/src/init.rs（ensure_gitignore 公開沿用）、crates/speclink-cli/src/remote_commands.rs、crates/speclink-cli/tests/remote_read_path.rs（接線測試）、crates/speclink-core/assets/skills 下 apply 與 verify 技能檔（技能文案全在 .md 資產，renderer skills.rs 無需修改）、.claude/skills 與 .agents/skills 對應技能實例、crates/speclink-core/tests/golden 對應快照
  - Removed: 無
