---
title: Claude Code：選擇技能
section: Claude Code
order: 370
keywords: ["Claude Code", "技能列", "規劃", "實作", "品質", "收尾", "Copilot"]
sources: ["claude-code-skill-bar", "archive-skill#封存完成後的收尾提交提醒"]
generated: 2026-10-10T08:25:58+08:00
---

# Claude Code：選擇技能

Claude Code 的 Speclink 技能列放在輸入框上方，讓你選擇已安裝的技能，再自行送出指令。這一頁說明該介面的操作；Codex 與 Copilot 的技能入口見 [建立工作區與指令檔](init-workspace.md)。規格沒有載明此介面的安裝步驟。

## 選一個技能

技能依序分成「規劃／實作／品質／收尾／其他」。只列已安裝的 Speclink 技能，沒有內容的分頁隱藏；不認識的技能歸「其他」。沒有可列技能，或輸入框正在顯示問卷時，原畫面維持，不加技能列。

點技能會把它的斜線指令放到輸入框最前面，取代原指令，保留後面的文字。例如原本是 `/speclink-review add-x`，點 verify 後變成 `/speclink-verify add-x`。它不會自動送出，也不會因此建立變更或修改任務。

只有 archive 與 commit 都已安裝時，才有 `archive+commit`；點它會填入 `/speclink-archive + /speclink-commit`。再改選 apply 時，兩個指令一起被取代，原參數保留。

> [!NOTE]
> 合併按鈕的文字與封存技能的收尾規定有差異：正式封存流程要求封存後以一般 Git commit 提交，變更範圍的 commit 技能不適用於已搬走的變更目錄。完成封存後請依 [封存](archive.md) 的提醒收尾，這個按鈕不代表可以略過提交範圍檢查。差異記錄於 [本手冊的來源](about.md)。

## 技能說明與窄視窗

寬視窗下，滑鼠移到技能才顯示說明；合併按鈕的說明為 `/speclink-archive → /speclink-commit`。視窗窄到分頁與面板按鈕放不下一列時，面板按鈕移到 speclink 標題列，技能說明隱藏。

對話開始、清空、接回與每輪完成後，技能列重新取得已安裝清單。工作中的步驟與變更名稱見 [Claude Code：目前工作與對話標題](claude-session-focus.md)，側邊面板見 [Claude Code：查看進度與工單](claude-workflow-panel.md)。

## 介面語言

此介面的語言設定有 auto、en、zh-TW，預設 auto。明選英文或繁中時優先採你的選擇；auto 依 Claude Code 語言文字判定：含「中文」、chinese 或 zh 時使用繁中，其餘（含日文或未設定）使用英文。這與專案的產物／規格語言設定各自獨立。

**出處**：`claude-code-skill-bar`、`archive-skill`
