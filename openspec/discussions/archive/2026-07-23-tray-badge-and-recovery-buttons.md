---
topic: tray 徽章數字與連線失敗按鈕排版
slug: tray-badge-and-recovery-buttons
status: promoted
promoted_to: tray-badge-and-recovery-buttons
created: 2026-07-23
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: tray 徽章數字與連線失敗按鈕排版

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者以兩張系統匣截圖提出兩個問題：(1) macOS 系統匣圖示旁的數字「1」是什麼；(2) 連線失敗恢復卡的次要按鈕「在 Speclink 中查看問題」在 320px 面板的半欄寬下換行。模式選擇：假設模式——tray 相關原始碼充足（apps/desktop/src/tray.ts、src/panel/TrayPanel.tsx、src-tauri/src/panel.rs、src/i18n/messages.ts）。相關規格：openspec/specs/tray-status-menu/spec.md（「macOS 進行中數文字徽章」需求明定徽章語意與更新行為）。無進行中的變更或討論。問題 1 在對話中追查後從理解性問題升級為 bug：徽章不隨資料與 workspace 切換更新。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-23)

**Focus**: 連線失敗恢復卡的次要按鈕換行要怎麼修
**Position**: 縮短文案（方案 A），排版結構不動：
- 根因是文案長度 × 固定半欄寬：面板固定 320px（panel.rs:28），按鈕區 grid-cols-2（TrayPanel.tsx:278），每欄約 130px 塞不下「在 Speclink 中查看問題」
- 文案改「查看問題」，英文 "View issue in Speclink" → "View issue"（messages.ts:152/377）；跳轉語意由既有 ArrowUpRight 圖示承載，符合 LANGUAGE.md「動詞直說結果」
- 主鈕（重新連線／重新登入）col-span-2 全寬＋兩顆次鈕各半欄的階層是對的；needsReauth 分支共用同一 grid，一併修好
- 使用者裁定：選 A
**Ruled out**: 方案 B 次鈕改直排——卡片變高、面板更長；方案 C 縮字級或收 padding——可讀性差且治標
**Open**: 徽章數字「1」的語意與行為（使用者回報：進行中為 0 仍顯示 1、切換 workspace 不更新）

### Round 2 — assumptions (2026-07-23)

**Focus**: 徽章數字「1」是什麼、為何不隨資料更新
**Position**: 徽章語意規格已明定，觀察到的行為是違規 bug，不是設計題：
- 語意：macOS tray title 顯示作用中專案的進行中變更數（tray.ts:286-289；tray-status-menu spec「macOS 進行中數文字徽章」需求），0 時隱藏；作用中 workspace 為 error/restoring 時 hideWorkspaceData 亦應歸零隱藏（tray.ts:156-157）
- 觀察與規格矛盾：截圖 1 作用中為連線失敗的 workspace 應隱藏卻顯示 1；截圖 2 切至進行中 0 的 workspace 仍顯示 1——徽章是殘留值，後續更新未落地
- 頭號嫌疑：面板樣式的去抖更新路徑在 setTitle 前失敗——tray.ts:567-570 先 setMenu(null)＋setShowMenuOnLeftClick(false) 再 setTitle，若前者於 macOS 擲錯會被 void async 吞掉；面板快照推送（pushSnapshot）在擲錯點之前，故面板資料新、徽章舊，與觀察吻合
- 確診需實機 runtime probe（console 觀測去抖分支），屬實作階段工作；修復以既有規格為行為基準，不改語意
**Open**: 無——tray tooltip 是否補徽章說明文字列為 Deferred

## Conclusion

**Decision**: 一個變更修兩件事：(1) 恢復卡次要按鈕文案「在 Speclink 中查看問題」→「查看問題」、"View issue in Speclink" → "View issue"，排版結構不動；(2) 修復 macOS 徽章不更新 bug——行為以既有 tray-status-menu 規格為準（作用中專案進行中數、隨資料變動即時更新、0 或 error/restoring 態隱藏），根因於實作階段以實機 runtime probe 確診，優先檢查面板樣式去抖更新路徑（tray.ts:567-570 的 setMenu(null) 擲錯被吞、setTitle 未執行）。
**Rationale**: 按鈕換行根因是文案長度 × 320px 面板半欄寬，縮文案是最小改動且跳轉語意由 ArrowUpRight 圖示承載；徽章行為規格已明定且使用者期待與規格一致，無語意可辯，純屬修復。
**Rejected alternatives**: 次鈕改直排（卡片變高、面板更長）；縮字級或收 padding（可讀性差、治標）；重新定義徽章語意如跨 workspace 加總（規格與使用者期待一致，無需求）。
**Deferred**: tray tooltip（現固定 "Speclink"）是否補上徽章說明文字（如「N 個進行中變更」）——本次不做。
**Capture to**: proposal（轉為變更後由提案承載；徽章行為不改規格，文案變更屬 UI 細節）
**Next**: speclink discuss promote tray-badge-and-recovery-buttons
