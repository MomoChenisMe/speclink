## Why

透過 AI Agent 執行 SDD 的開發者、PO 與 PM，目前在 CLI 初始化或 Desktop 連接 Remote checkout 時，可能只得到 remote marker，卻沒有可用的 Claude／Codex Skills 與專案指令區塊；無 footprint 時 CLI 還會默認 Claude，使實際安裝結果不透明。工具選擇應成為所有初始化入口一致、可見且可重試的本機 Workspace 契約。

## What Changes

- 所有 filesystem 與 Remote Store 的 speclink init 在互動式終端且未提供 --tools 時，顯示 Claude／Codex 複選；顯式提供 --tools 時跳過詢問。
- **BREAKING**：非互動終端執行 speclink init 且未提供 --tools 時，於任何檔案寫入前以非零 exit code 失敗，stderr 指引顯式指定 claude、codex 或兩者；init 不新增 JSON 輸出，既有 --json 契約不變。
- Desktop 的 Workspace chooser 在連接本機 checkout 時提供相同的 Claude／Codex 複選，並在開啟 Workspace 前完成工具同步；既有相符 remote marker 的 checkout 也會補齊或更新受管產物。
- 將 `.speclink.yaml` 的 `tools` 清單視為權威期望狀態：勾選 Claude 會同步 Claude Skills 與 `CLAUDE.md` 的 Speclink 區塊，勾選 Codex 會同步 Codex Skills 與 `AGENTS.md` 的 Speclink 區塊；取消選取會移除對應的 Speclink 受管產物，但保留指令區塊外的使用者內容。
- filesystem init、Remote Store init、Desktop checkout 綁定與既有 checkout 修復共用 `speclink-core` 的生成與清理語意；Remote checkout 保留 remote section 且不建立本機 `openspec/`。
- Desktop 同步任一步驟失敗時不開啟 Workspace，顯示可操作錯誤並允許修正後安全重試。
- 相容性影響：互動式人眼流程新增工具選擇；非互動 init 從自動偵測改為要求 --tools。既有工具名稱、`.speclink.yaml` 欄位、Skills 內容、Remote marker、其他 CLI 人眼輸出與所有 JSON shape 不變。既有自動化須在 init 呼叫中加入 --tools。

## Non-Goals

- 不新增 Claude／Codex 以外的互動選項，也不在 picker 編輯自訂工具描述子。
- 不改變 speclink link、speclink update、Remote handshake、Remote capabilities、規格上傳或本機／Remote 內容遷移語意。
- Desktop 不啟動外部 speclink CLI process，也不要求使用者電腦另有匹配版本的 CLI binary。
- 不覆寫 `AGENTS.md`／`CLAUDE.md` 的使用者內容，不移除非 Speclink 產生的 Skills。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `workspace-tools`: 明定所有 init 的工具選擇、非互動失敗邊界，以及 `tools` 權威狀態對 Skills／指令區塊的同步與清理契約。
- `workspace-chooser`: Desktop 連接新舊 Remote checkout 時選擇工具、完成同步後才開啟 Workspace，並定義失敗與重試行為。
- `remote-connection`: Remote init 與 Desktop checkout 共用 Remote Workspace 初始化結果，保留 remote section 且不建立本機規格樹。

## Impact

- Affected specs: `workspace-tools`、`workspace-chooser`、`remote-connection`
- Affected code:
  - Modified:
    - `crates/speclink-core/src/init.rs`
    - `crates/speclink-cli/src/main.rs`
    - `crates/speclink-cli/src/commands.rs`
    - `crates/speclink-cli/src/remote_commands.rs`
    - `crates/speclink-cli/tests/remote_connect.rs`
    - `crates/speclink-cli/tests/remote_section.rs`
    - `apps/desktop/core/src/project.rs`
    - `apps/desktop/core/src/settings.rs`
    - `apps/desktop/src-tauri/src/connections.rs`
    - `apps/desktop/src-tauri/src/lib.rs`
    - `apps/desktop/src/adapter/connections.ts`
    - `apps/desktop/src/components/WorkspaceChooser.tsx`
    - `apps/desktop/src/i18n/messages.ts`
    - `apps/desktop/src/App.tsx`
    - `apps/desktop/src/main.tsx`
    - `apps/desktop/src/store.ts`
    - `apps/desktop/src/__tests__/App.test.tsx`
    - `apps/desktop/src/__tests__/remoteOpen.test.ts`
    - `apps/desktop/src/__tests__/workspaceChooser.test.tsx`
    - `openspec/specs/workspace-tools/spec.md`
    - `openspec/specs/workspace-chooser/spec.md`
    - `openspec/specs/remote-connection/spec.md`
  - New:
    - `crates/speclink-cli/tests/init_tools.rs`
  - Removed: none
- Affected systems: `speclink-core` 的 Workspace 工具同步、`speclink-cli` 的 init 輸入／exit code、Desktop React chooser 與 Tauri checkout 綁定。
- Configuration: 不新增欄位；`.speclink.yaml` 的 `tools` 保持既有 claude／codex 值域，但在 init 與 Desktop 綁定後代表完整權威狀態。
- Dependencies: 不新增外部 crate；CLI 互動使用標準函式庫的 terminal 判斷與行輸入。
