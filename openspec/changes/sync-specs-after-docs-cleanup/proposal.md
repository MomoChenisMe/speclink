## Summary

讓正式規格、工作流設定與幾處文字對齊程式碼現況，並重拍文件截圖、把沒用到的截圖放進文件。

## Motivation

2026-10-07 的文件整理（commit 147076be）照程式碼重寫了 README 與 docs/，稽核時發現正式規格與周邊文字有幾處落後：

- `user-documentation` 規格仍要求已移除的平台架構與實作路線圖文件、仍以 H2 字串相同判斷中英對等、仍寫 GET `/account/tokens` 回 405、仍把 server 文件列為「只有中文」、仍寫 sdk-node「以首個帶 engine 的 release 為準」，並把 macOS 桌面 app 寫成「每次啟動都換掉 CLI」（程式碼只在未安裝或版本不同時換）。
- `verb-contract` 規格的 409 reason（`version_conflict`、`ownership_lost`、`change_busy`、`repo_mismatch`）在 Protocol 的 reason 註冊表裡不存在；實際是 `revision_conflict` 與 `refused`。
- `openspec/config.yaml` 的專案說明仍引用已移除的 docs/design/platform-architecture.zh-TW.md。
- `speclink init` 產生的 `.speclink.yaml` 註解、`speclink feedback` 的輸出，以及本 repo 根目錄的 `.speclink.yaml`，都寫著錯的網址 `github.com/speclink-app/speclink`。
- CI smoke test 的 `test -f CLAUDE.md && test -f AGENTS.md && ...` 沒有作用：`init` 已不產生這兩個檔，而 `&&` 串列前段失敗不會觸發 `set -e`。
- 桌面設定頁的 `settings.rulesHelp` 文案仍寫「儲存會重寫 config.yaml，檔內註解不會保留」，現在只改目標那幾行、註解會保留。
- 文件截圖拍的是真實的 speclink repo，不是截圖腳本的示範 workspace；規格頁那張還顯示舊詞「正典規格」；腳本列出的 9 張截圖有 4 張沒有文件使用。

使用者是讀文件與規格的開發者／PO／PM，以及照規格實作的 AI 代理；情境是查詢正式規格、照文件操作，以及 `/speclink-manual` 從規格產生手冊。本變更來自討論 `docs-cleanup-findings` 的最後一刀，排在 `server-reader-write-guard` 與 `ingest-accepts-change-name` 之後。

## Proposed Solution

1. **規格**（隨封存合併進正式規格）：
   - `user-documentation`：修改 9 條需求、移除「目標架構與目前狀態維持清楚邊界」（兩份架構文件已移除）。重點：漸進揭露改為 README → getting-started／workflow → product-status → roadmap；中英對等改為 H2 錨點序列相同、涵蓋全部成對文件；405 端點改為 `/api/speclink/v1/web/account/tokens`；sdk-node 寫明 0.2.0 起可安裝；macOS 換 CLI 的時機照程式碼；截圖腳本的每張截圖都要有文件引用。
   - `verb-contract`：409 reason 改成註冊表的 `revision_conflict` 與 `refused`，CLI 訊息照現行實作。
2. **工作流設定**：以 `speclink workflow-config context` 改掉 `openspec/config.yaml` 專案說明中引用 docs/design 的那一句，改指 docs/roadmap.zh-TW.md。
3. **文字修正**（speclink-core、speclink-cli、apps/desktop、CI）：
   - `.speclink.yaml` 範本、`speclink feedback` 輸出與本 repo 的 `.speclink.yaml` 的網址改成 `https://github.com/MomoChenisMe/speclink`。
   - CI smoke test 改成逐行檢查 `init` 實際產生的 `.claude/skills/speclink-propose/SKILL.md` 與 `.agents/skills/speclink-propose/SKILL.md`。
   - 桌面 `settings.rulesHelp` 中英文案改成「只改目標那幾行，檔內註解會保留」。
4. **截圖**：用 `scripts/docs/docs-screenshots.mjs` 的示範 workspace 重拍全部 9 張（手動）；新增守門測試「腳本列出的每張截圖都要被文件引用」；把規格頁、討論頁、已封存頁、設定頁四張截圖放進入門、workflow 與設定說明的中英兩版。

## Non-Goals

- 不改 `user-documentation` 中仍符合現況的需求（工作流正典、Remote 入門流程、開發者入口、本地與遠端對照等）。
- 不重新命名任何需求或 scenario；「使用者面路線圖與內部交付順序分列」的名稱保留。
- 不改 CLI 的 409 訊息文字與 Protocol 型別：本變更讓規格跟上程式碼，不反過來。
- 不新增截圖或改截圖腳本的拍攝流程；只重拍既有 9 張。
- 不改 `speclink init` 的其他範本內容。

## Alternatives Considered

- 把 macOS 桌面 app 改成每次啟動都換 CLI，讓程式碼符合規格——會無故蓋掉使用者同版本的 CLI，討論已否決。
- 刪掉 4 張沒用到的截圖——截圖有助白話理解，討論已否決。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `user-documentation`: 9 條需求改成文件整理後的現況，移除「目標架構與目前狀態維持清楚邊界」，新增「沒有孤兒截圖」的檢查。
- `verb-contract`: 「樂觀並行控制與 409 語意」的 reason 改成 Protocol 註冊表的值。

## Impact

- Affected specs: user-documentation, verb-contract
- 相容性影響：
  - `speclink feedback` 的人眼輸出網址改變（沒有 `--json`），CLI 測試沒有比對這一行；`speclink init` 新產生的 `.speclink.yaml` 註解改變，既有檔案不受影響。
  - 其餘 CLI 人眼輸出、`--json` 與 golden 不變。
- Affected code:
  - Modified: openspec/config.yaml
  - Modified: .speclink.yaml
  - Modified: crates/engine/speclink-core/src/workspace/init.rs
  - Modified: crates/engine/speclink-core/src/workspace/init/tests.rs
  - Modified: crates/adapters/speclink-cli/src/verbs/toolchain.rs
  - Modified: .github/workflows/ci.yml
  - Modified: apps/desktop/src/i18n/messages.ts
  - Modified: scripts/docs/docs-screenshots.test.mjs
  - Modified: docs/assets/screenshots/（重拍的 9 張 png）
  - Modified: docs/getting-started.zh-TW.md
  - Modified: docs/getting-started.md
  - Modified: docs/workflow.zh-TW.md
  - Modified: docs/workflow.md
  - Modified: docs/configuration.zh-TW.md
  - Modified: docs/configuration.md
