## Why

從其他規格體系（如上游 OpenSpec）遷移、或隊友未提交 `.speclink.yaml` 的專案，資料夾裡已有 `openspec/` 目錄——desktop 開啟時探測只要看到 `openspec/` 就判為既有專案直接進看板，speclink 技能與 CLAUDE.md/AGENTS.md 受管區塊從未安裝也毫無提示，AI 代理在這種專案裡拿不到任何 speclink 工作流；初始化確認框只在「向上探索完全未命中」時出現，這類「有規格資料但未接上 speclink」的資料夾被靜默吞掉。討論 desktop-workspace-auto-init 定案：以「專案根 `.speclink.yaml` 是否存在」為「已啟用 speclink」的判準，desktop 對未啟用的資料夾提示啟用、確認後補齊工作區檔。

## What Changes

- **desktop 探測第四態「未啟用」**：開啟本機資料夾的探測在「向上探索命中 workspace 且 store mode 為本地檔案、但 workspace root 無 `.speclink.yaml`」時，回報未啟用（帶命中的 root）而非既有專案。`.speclink.yaml` 存在 → 照舊判既有專案；完全未命中 → 照舊判未初始化；remote marker 的分流（含舊版 remote 標記檔的遷移警告路徑）不動。core 引擎的向上探索（`Workspace::discover`）行為不動——第四態只加在 desktop 探測層，CLI 對 bare openspec/ 專案照舊運作。
- **啟用確認框**：前端對未啟用態顯示「啟用 speclink」確認對話框，與初始化確認框同型（AI 工具多選 claude／codex、預設勾選 claude；取消零寫入、維持原專案；失敗單行錯誤、不切換）。文案為啟用語意（資料夾已有規格資料、尚未啟用 speclink），遵循 openspec/LANGUAGE.md。
- **引擎 adopt 入口**：speclink-core 新增「補齊工作區」入口——冪等補 openspec/ 骨架缺件（specs/、changes/archive/ 目錄；config.yaml 僅在不存在時寫入範本，既有檔零觸碰）、寫 `.speclink.yaml` 記錄所選 tools、為每個所選工具生成技能檔與指令檔受管區塊、確保 `.gitignore` 涵蓋 `.speclink/` 工作資料夾；既有 openspec/ 內容（specs、changes、discussions 等文件）SHALL 零觸碰。組合素材已存在（store_init 的 write_if 冪等、reconcile_builtin_tools 的 tools 寫入＋整套再生、ensure_gitignore 的追加式補件），入口僅繞過 init 的「Already initialized」擋板。
- 確認後 desktop 以該 root 切入專案（與初始化確認後的切入語意一致）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-config`: 新增需求——開啟「有 openspec/ 但無 .speclink.yaml」的資料夾時判未啟用並顯示啟用確認框，確認後經引擎 adopt 入口補齊工作區檔並切入專案；取消零寫入。
- `workspace-tools`: 新增需求——引擎提供冪等的工作區補齊（adopt）入口：補骨架缺件、寫 tools、生成受管檔，既有規格內容零觸碰。

## Impact

- Affected specs: `desktop-config`（修改）、`workspace-tools`（修改）
- Affected code:
  - Modified:
    - crates/speclink-core/src/init.rs
    - apps/desktop/core/src/project.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src/adapter/workspace.ts
    - apps/desktop/src/store.ts
    - apps/desktop/src/App.tsx
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/workspace.test.ts
    - apps/desktop/src/__tests__/App.test.tsx
  - New: (none)
  - Removed: (none)
- 影響的 crate／app：speclink-core（adopt 入口）、apps/desktop/core 與 src-tauri（第四態與 IPC）、desktop 前端（分流、確認框、文案）。CLI 指令面不動；`Workspace::discover` 不動。

## Non-Goals

（範圍排除與否決方案詳見 design.md 的 Goals / Non-Goals。）
