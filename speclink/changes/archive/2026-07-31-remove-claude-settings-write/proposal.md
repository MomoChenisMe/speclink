## Problem

speclink 的工具檔生成把 `.claude/settings.json` 當成受管產生檔：init 時寫入固定內容（`includeGitInstructions: false`），而 update／reconcile 路徑以 force 再生受管檔時將它整檔覆寫。使用者放在該檔的自有設定（如 enabledPlugins 的外掛啟用清單）在任何一次工具同步（speclink update、desktop 設定頁改工具選集、desktop 啟用未啟用專案）後被靜默清空——2026-07-31 於使用者專案 wadpilot 實證：三個外掛設定被洗成僅剩 `includeGitInstructions` 一行。

## Root Cause

兩層問題疊加：

1. **機制**：`.claude/settings.json` 的內容是引擎內寫死的常數，生成走與技能檔相同的「受管檔以 force 覆寫」路徑；但它實際上是 AI 工具的使用者設定檔，語意上屬使用者所有（對照：CLAUDE.md 走 marker 區塊合併保留使用者內容、技能檔是純受管檔可覆寫——settings.json 被錯放進後者）。
2. **值本身**：寫入的 `includeGitInstructions: false` 是替使用者關閉 Claude Code 內建 git 指令的行為偏好，無任何規格、討論或文件背書（openspec/ 全樹與 docs/ 均零記載），且 speclink 支援多種 AI 工具，不應假設使用者使用 Claude Code 並代其調整偏好。

## Proposed Solution

工作區檔生成完全停止寫入 `.claude/settings.json`（使用者裁定，2026-07-31）：移除引擎內的固定內容常數與生成呼叫，init／update／reconcile／adopt 所有路徑一致不再產生或觸碰該檔。既有專案已存在的 settings.json 一律視為使用者檔案：不覆寫、不清理（prune 現行即不碰它，行為不變）。以規格明文釘死「工具檔生成 SHALL NOT 寫入 AI 工具的使用者設定檔」，防止未來回歸。

## Non-Goals

- 不對既有專案做遷移或清理——已被寫入的 settings.json 留在原地（內容可能已被使用者編輯，屬使用者資料）。
- 不提供「代使用者設定 AI 工具偏好」的替代機制（如文件建議、選配旗標）——需要時另開變更。
- 不改動 CLAUDE.md／AGENTS.md marker 區塊與技能檔的既有生成與清理行為。
- 不改動 desktop 的工具同步流程——它經由引擎入口，引擎修正後自然收斂。

## Success Criteria

- 對全新目錄執行 init（任一工具選集）後，`.claude/settings.json` 不存在；`.claude/skills/` 技能檔照常生成。
- 對「已有自訂 `.claude/settings.json`」的專案執行 update／reconcile／adopt 後，該檔位元級不變。
- 規格新增需求「工具檔生成不寫入 AI 工具的使用者設定檔」並有對應測試釘死；cargo test 全綠（含改寫後的 CLI 整合測試，不再斷言 settings.json 存在）。

## Impact

- Affected specs: `workspace-tools`（修改——新增不寫入使用者設定檔的需求）
- Affected code:
  - Modified:
    - crates/speclink-core/src/init.rs
    - crates/speclink-cli/tests/remote_section.rs
  - New: (none)
  - Removed: (none)
