---
topic: quality skill 每輪檢查後暫停＋桌面 UI 修整批次（tooltip 延遲一致、詳情章列重做、tray hover、截斷統一、sticky 橫幅）
slug: quality-skill-pause-and-ui-polish
status: promoted
promoted_to: quality-skill-round-pause, desktop-ui-stamp-and-overflow-polish
created: 2026-08-07
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: quality skill 每輪檢查後暫停＋桌面 UI 修整批次（tooltip 延遲一致、詳情章列重做、tray hover、截斷統一、sticky 橫幅）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 quality-skill-canonicalization 封存（2026-08-07）後提出一批回饋：一項 skill 編排行為（兩站檢查完不應自顧自開修，要停下來問使用者）＋六項桌面 UI 問題（卡片 tooltip 消失感、詳情抽屜章列位置怪且溢出裁切、tray hover 未處理章色、截斷作法淡出／點點點並存、指令檔過期橫幅不 sticky）。

模式：assumptions——codebase scout 找到大量相關檔案（packages/ui 的 ChangeCard.tsx、RichDetailDrawer.tsx、CardNameRow.tsx；apps/desktop 的 TrayPanel.tsx、InstructionUpdatePrompt.tsx、App.tsx；引擎正典 asset crates/speclink-core/assets/skills/quality.md）。

相關變更：quality-skill-canonicalization（已封存，skill 行為改動須開新變更）；2026-08-06-verify-station-parity 與 2026-08-05-semantic-color-system 被點名為 tooltip 移除嫌疑，scout 證實兩者皆未刪 tooltip（一個只加、一個只改氣泡配色）。

相關 specs：quality-skill、desktop-app（看板卡片統一解剖學、品質站蓋章配色、語意色分層）、tray-status-menu（面板變更列的品質站章）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-07)

**Focus**: 七項回饋的病根定位、修法假設與變更路由
**Position**: 以 codebase 證據逐項提出假設，使用者確認多數、修正兩項：
- 路由拆兩變更：skill 暫停（引擎 asset＋三處同步＋golden＋quality-skill spec）與桌面 UI 修整批次（純前端）——未異議
- skill 暫停（修正）：不是只在開修前停一次，而是「每輪兩站檢查完都停」，彙整兩站 findings 問使用者下一步；證據 .claude/skills/speclink-quality/SKILL.md:38-40 步驟 3 現為檢查完自動全修
- tooltip（修正＋擴充）：程式碼未刪 tooltip（ChangeCard.tsx:45-151 皆在；兩個被點名的封存變更只加不刪），感知主因為 Radix 預設 700ms hover 延遲（RichDetailDrawer.tsx:383 設 0、ChangeCard 未設，兩處不一致）；使用者裁定全系統 shadcn tooltip 延遲一致（desktop＋server-web，皆消費 packages/ui）
- 詳情抽屜章列重做（確認）：RichDetailDrawer.tsx:332-379 把進度條＋兩站章＋日期＋完整作者身分塞單行、shrink-0 不可壓縮，被 sheet 的 overflow-x-hidden 裁切；改為章籤＋日期作者收 tooltip，與下方 provenance row（:383-402）同構
- tray hover 章變白（確認）：TrayPanel.tsx:759,768 的 StationBadges 補 group-hover:text-primary-foreground 覆寫，與同列其他元素一致，審查／驗證靠 icon 形狀區分
- 截斷統一（裁定）：選 A 全面 ellipsis——拔 CardNameRow.tsx:8 的淡出遮罩與 ResizeObserver 量測邏輯（fade 全系統僅此一處，其餘 20+ 處皆 truncate）
- sticky 橫幅（確認）：InstructionUpdatePrompt.tsx:33-37 加 sticky top-0＋背景改不透明（現為 bg-muted/40 半透明）
**Ruled out**: ingest 回 quality-skill-canonicalization（已封存）；暫停僅開修前一次（使用者要每輪都停）；截斷選項 B 分類規則（tray 的 slug 需反向改淡出、工多）與 C 維持現狀（淡出被讀成破圖，Image #3）
**Open**: 乾淨輪（兩站零 findings）仍停下來問、還是自動進蓋章；tooltip 統一延遲值（0ms 立即 vs 300ms 短延遲）

### Round 2 — assumptions (2026-08-07)

**Focus**: 收攏最後兩個開放問題——乾淨輪行為與 tooltip 統一延遲值
**Position**: 兩題皆由使用者裁定，方向如下：
- 乾淨輪也停：skill 全程不自行跨階段——每輪兩站檢查完必停（含零 findings 的輪），乾淨輪停下來報告兩站皆綠，由使用者決定「蓋章＋封存」與否；蓋章不再自動發生
- tooltip 統一延遲 300ms：共用預設下沉到 packages/ui/src/components/ui/tooltip.tsx，移除各處 local delayDuration 覆寫（含 RichDetailDrawer.tsx:383 現行的 0ms）；desktop 與 server-web 皆消費 packages/ui、同吃一個預設；tray 刻意用原生 title（TrayPanel.tsx:739-747 doc comment）不在統一範圍
**Ruled out**: 乾淨輪自動蓋章——少一次互動，但破壞「skill 不自作主張」的單一心智模型；0ms 立即彈——看板 icon 密集區滑鼠掃過會頻繁閃現提示
**Open**: 無——實作分工細節交給 propose

## Conclusion

**Decision**: 拆兩個新變更落地這批回饋：
1. quality skill 每輪暫停——兩站（審查＋驗證）每一輪檢查完成後，彙整兩站 findings 停下來問使用者下一步（全修／挑著修／不修就停）；乾淨輪（零 findings）也停，報告兩站皆綠，由使用者決定蓋章＋封存；skill 全程不自行跨階段。動引擎正典 asset（crates/speclink-core/assets/skills/quality.md）＋MARKER_VERSION／golden／assets.lock 三連動＋三處技能同步＋quality-skill spec。
2. 桌面 UI 修整批次（純前端，packages/ui＋apps/desktop）：
   a. shadcn tooltip 延遲統一 300ms 共用預設（下沉 packages/ui/src/components/ui/tooltip.tsx、移除各處覆寫；tray 原生 title 刻意除外）＋實機驗證卡片 tooltip 確實會彈——tooltip 從未被刪，感知消失主因是 Radix 預設 700ms 延遲
   b. 詳情抽屜章列重做：章籤化（icon＋已審查／已驗證），日期與作者收進 tooltip，與下方 provenance row 同構——同時解「位置怪」與「溢出被裁切」（RichDetailDrawer.tsx:332-379）
   c. tray StationBadges 補 group-hover 變白（TrayPanel.tsx:759,768），與同列元素一致，兩站靠 icon 形狀區分
   d. 截斷作法全面統一 ellipsis：拔 CardNameRow.tsx 淡出遮罩與 ResizeObserver 量測（fade 全系統僅此一處）
   e. 指令檔過期橫幅 sticky top-0＋背景改不透明（InstructionUpdatePrompt.tsx:33-37）
**Rationale**: 引擎資產改動（版號波及、golden 再生、三處同步）與純前端改動分開，兩個變更的審查驗證面各自最小；skill 側「每輪都停、乾淨輪也停」讓使用者永遠握住階段轉換，心智模型單一；UI 側每項皆有病根定位而非表象猜測。
**Rejected alternatives**: ingest 回 quality-skill-canonicalization（2026-08-07 已封存）；暫停僅開修前一次（使用者裁定每輪都停）；乾淨輪自動蓋章（破壞一致心智模型）；截斷 B 分類規則（tray 的 slug 需反向改淡出、工多）與 C 維持現狀（淡出被讀成破圖）；tooltip 0ms（密集區閃現）；tooltip 修法「先重現 runtime 壞掉」（scout 證實程式碼未刪、延遲不一致即充分病因假說，實機驗證留在變更內做）
**Deferred**: none
**Capture to**: proposal（兩個新變更）
**Next**: /speclink-propose --from-discussion quality-skill-pause-and-ui-polish（跑兩次，或用 speclink discuss promote quality-skill-pause-and-ui-polish --name <name> 快速轉出）
