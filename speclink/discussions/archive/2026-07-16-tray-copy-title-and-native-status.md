---
topic: 強化 desktop tray：選單複製討論/變更標題＋原生 statusbar 設計研究
slug: tray-copy-title-and-native-status
status: promoted
promoted_to: tray-copy-and-panel-mode
created: 2026-07-16
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 強化 desktop tray：選單複製討論/變更標題＋原生 statusbar 設計研究

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者想強化 desktop 系統匣（tray）：(1) hover 變更/討論時選單內可複製標題（比照詳情抽屜的複製鈕）；(2) 研究 Rust tray 能否有原生的 statusbar 設計。模式選 assumptions——掃到 apps/desktop/src/tray.ts（選單模型＋接線層）、trayIcon.ts、__tests__/tray.test.ts、RichDetailDrawer.tsx / DiscussionDrawer.tsx（既有複製鈕先例），程式碼脈絡充分。相關規格：openspec/specs/tray-status-menu（tray 已規格化，後續變更以 delta 修改）。進行中變更（server-release-packaging、phase2-e2e-chain）與本題無關。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-16)

**Focus**: 五項假設校準——複製功能的落點與內容、剪貼簿通道、以及「statusbar 設計」的解讀與可行路徑
**Position**: 複製功能落在原生選單子選單；statusbar 走「webview 面板」試驗路線，與原生 NSMenu 並存，由使用者實測後裁決：
- 變更子選單擴充「複製名稱」動作，複製 change.name（與 RichDetailDrawer.tsx:162 一致），不複製含進度條字元的標籤全文
- 討論項改造為子選單（開啟此討論＋複製）——原生選單帶子選單的父項不觸發點擊動作，無 hover 浮出按鈕可用
- 討論複製 slug 而非 topic（LANGUAGE.md 明文例外：討論識別錨點與複製鈕以 slug 直出；DiscussionDrawer.tsx:263 先例）
- 剪貼簿走 tauri-plugin-clipboard-manager（Rust 端）——tray 點擊時主視窗可能隱藏/無焦點，navigator.clipboard 會拒寫
- 「statusbar 設計」研究結論分三層：(a) NSStatusItem title 純文字＝已在用（badge）零成本；(b) 原生選單項嵌自訂 view＝muda 未暴露 NSMenuItem.view，走不通；(c) tray 彈出貼齊面板視窗＝tauri-plugin-positioner（TrayCenter）＋ tauri-nspanel/nspopover（不搶焦點）＋ window-vibrancy（毛玻璃），外觀近原生
- 使用者裁定：之前試過 webview 感覺不原生（很可能缺上述三件套），願意再試路徑 (c)，但原生 NSMenu 程式先保留；任務規劃須讓使用者實測，最後一個 task 由使用者決定保留 webview 面板或原生 NSMenu
**Ruled out**: 路徑 (b) 原生選單內嵌自訂 view——muda 不支援，除非 fork；複製標籤全文——會夾帶進度條雜訊；navigator.clipboard——無焦點時不可靠
**Open**: 試用期間 NSMenu 與面板如何並存（左右鍵分流 vs 設定切換）；複製功能與面板試驗是否拆成兩個變更；面板的失焦收合、深淺色等原生質感細節

### Round 2 — assumptions (2026-07-16)

**Focus**: 試用期間兩種 tray 呈現的並存機制，以及複製功能與面板試驗的變更拆分
**Position**: 設定頁切換＋單一變更，最後一個 task 由使用者實測裁決：
- desktop 設定頁新增「系統匣樣式」切換（原生選單 / 面板），兩模式並存供 A/B 對照
- 複製功能與面板試驗合為一個變更，任務序大致為：NSMenu 子選單複製 → 設定切換 → webview 面板（positioner＋nspanel＋vibrancy）→ 使用者實測裁決（最後 task）
- 面板模式下複製可直接做成 hover 複製鈕——即使用者最初引用的詳情抽屜複製鈕形式，NSMenu 做不到、webview 反而自然
- 裁決若拆除一邊（含設定項去留），規格面走 ingest 收斂後再 verify/archive
**Ruled out**: 左右鍵分流與 dev flag 並存機制——使用者裁定設定頁切換；拆兩個變更——使用者裁定合一，接受封存時點繫於實測完成
**Open**: 無——面板版型與資訊密度、badge 文字擴充皆明列為 deferred

### Round 3 — assumptions (2026-07-16)

**Focus**: 追補——tray 討論項目前顯示 topic 而非 slug，違反識別錨點慣例
**Position**: tray 討論項改以 slug 為標籤，納入本變更範圍：
- 現況 tray.ts:109 以 d.topic 為選單標籤；看板討論卡則以 slug 為題、topic 為副標
- LANGUAGE.md 識別錨點例外（討論以 slug 直出）原枚舉面不含 tray，屬範圍擴充，須記入 LANGUAGE.md
- 討論項改子選單後，topic 的呈現位置（子選單內灰字副標或捨棄）進 design 再定
**Open**: 無

## Conclusion

**Decision**: 以單一變更強化 desktop tray：(1) 原生 NSMenu 的變更/討論項改為子選單並加「複製」——變更複製 name、討論複製 slug，剪貼簿經 tauri-plugin-clipboard-manager（Rust 端，不受視窗焦點限制）；(2) tray 討論項標籤改以 slug 直出（現為 topic，違反識別錨點慣例；LANGUAGE.md 例外範圍擴充須記入）；(3) 新增 webview 面板模式——tauri-plugin-positioner（TrayCenter 定位）＋ tauri-nspanel（不搶焦點）＋ window-vibrancy（毛玻璃），面板內複製做成 hover 複製鈕；(4) desktop 設定頁新增「系統匣樣式」切換讓兩模式並存；(5) 原生 NSMenu 程式全程保留，最後一個 task 由使用者實測後裁決保留 webview 面板或原生 NSMenu，裁決改變規格時走 ingest。
**Rationale**: 豐富的原生 statusbar 呈現在選單內走不通（Tauri 選單層 muda 未暴露 NSMenuItem.view），只剩面板路線；但面板的原生質感未經本專案驗證（使用者先前 webview 經驗不佳，疑因缺 nspanel/vibrancy/定位三件套），故不押邊——兩模式並存、設定切換、實測裁決。
**Rejected alternatives**: fork muda 補自訂 view（工程量不成比例）；左右鍵分流或 dev flag 並存（使用者裁定設定頁切換）；拆成兩個變更（使用者裁定合一）；navigator.clipboard（tray 點擊時視窗可能無焦點會拒寫）；複製標籤全文（夾帶進度條字元雜訊）；討論複製 topic（LANGUAGE.md 明文例外：討論識別錨點與複製鈕以 slug 直出）。
**Deferred**: 面板 UI 版型與資訊密度（進 design/tasks 再定）；討論子選單內 topic 的呈現位置（灰字副標或捨棄，進 design）；裁決後「系統匣樣式」設定項去留（若裁決兩者皆留則設定留下）；badge 文字擴充狀態資訊（路徑(a)零成本可行，本次不動）。
**Capture to**: proposal（spec delta 動 tray-status-menu 與 desktop-config；識別錨點範圍擴充記入 openspec/LANGUAGE.md）
**Next**: /speclink-propose --from-discussion tray-copy-title-and-native-status
