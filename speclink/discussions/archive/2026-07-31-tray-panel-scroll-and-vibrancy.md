---
topic: macOS tray 面板：捲動範圍固定頁首頁尾＋毛玻璃深色背景偏暗
slug: tray-panel-scroll-and-vibrancy
status: promoted
promoted_to: tray-panel-scroll-and-vibrancy
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: macOS tray 面板：捲動範圍固定頁首頁尾＋毛玻璃深色背景偏暗

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者以截圖回報兩個 macOS tray 面板問題：(1) 面板內容超過上限高度後整頁捲動，期望固定頁首（專案 tab 條）與頁尾（動作區三項），僅中段內容分區捲動；(2) HudWindow 毛玻璃在深色背景（深色 IDE／終端機）下面板明顯偏暗，對照 macOS 原生選單（NSMenu，如輸入法選單）同背景下亮度幾乎不受影響。

模式：assumptions——codebase scout 找到 4 支直接相關原始碼（apps/desktop/panel.html、src/panel/main.tsx、src/panel/TrayPanel.tsx、src-tauri/src/panel.rs）。

相關 spec 與歷史裁決：tray-status-menu spec「面板樣式（macOS）」（spec.md:341 高度自適應與「達上限後面板內部捲動」，未指明捲動範圍）；tray-panel-card-design（2026-07-16 實測裁決 vibrancy 材質 Menu → HudWindow，理由：Menu 淺色模式近乎不透、違反「毛玻璃底可透出」）；tray-copy-and-panel-mode（面板架構 D5／高度自適應）；tray-macos-panel-only（拆除原生選單樣式偏好，macOS 僅面板）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: 面板超過上限高度後的捲動範圍——整頁捲 vs 固定頁首頁尾、僅中段捲
**Position**: 三段式版面定案（使用者確認全部 5 項假設）：
- 固定頁首＝專案 tab 條＋其下分割線；可捲中段＝討論／已轉出／生命週期分區卡；固定頁尾＝分割線＋動作區三項（TrayPanel.tsx:358-575 現為單一 flex column）
- 捲動面從 body（panel.html:12 overflow-y:auto）移入中段容器（flex-1 min-h-0 overflow-y-auto）；overscroll-behavior:none 隨遷防 rubber-band
- 高度自適應量測改「頁首高＋中段自然內容高（scrollHeight）＋頁尾高」、上限 640 不變（main.tsx:36-48）——迴避「視窗高決定 root 高、root 高又回設視窗」的循環依賴；內容未超限時貼合無捲動（spec.md:341 既有要求維持）
- recovery／stale 分支歸入可捲中段、頁首頁尾結構不分支（TrayPanel.tsx:438-471）
- 落地為 tray-status-menu spec delta（明確化「內部捲動限於內容分區、tab 條與動作區常駐」）＋實作；root 主色漸層 wash 隨之固定於視窗（順手修正）
**Ruled out**: 維持整頁捲（body scroll）——tab 條與動作區被捲走，不符期望；只修實作不動 spec——無回歸防護，未來可能無聲退回整頁捲
**Open**: 中段捲軸要否隱藏；毛玻璃在深色背景偏暗如何處理（使用者第二題，下輪）

### Round 2 — assumptions (2026-07-31)

**Focus**: 毛玻璃深色背景偏暗的根因與修法——原生選單為何不變暗
**Position**: 使用者更正了錯誤模型（原生≠不透明主題色底），修正後的根因與選項空間：
- 使用者在 macOS 26.5（Liquid Glass 世代）：原生選單是真半透明玻璃，靠亮度自適應（tone-mapping、深淺變體自動切換）在深色背景下維持亮度——不是靠不透明基底
- 面板用的 NSVisualEffectView（HudWindow，panel.rs:89-94）是 pre-Liquid-Glass 相容路徑：固定比例線性混合、無亮度自適應——背景多暗面板就多暗；換任何舊材質（Menu/Popover）都追不上原生
- 驗證：window-vibrancy 0.6.0（Cargo.lock 鎖定版）原始碼僅有 NSVisualEffectMaterial 列舉、無 NSGlassEffectView 支援；真玻璃公開 API＝AppKit NSGlassEffectView（macOS 26+），可於 panel.rs 以 objc2 runtime（NSClassFromString）動態插入、執行期偵測免升 SDK；專案未宣告 minimumSystemVersion（Tauri 預設 10.13）→ 發行版需 fallback
- 選項：A＝CSS 補光層（bg-background/60 級靜態近似，10 行但永非真自適應）；B＝NSGlassEffectView 真玻璃（需 spike 驗證 NSPanel＋透明 WKWebView 疊玻璃視圖）；C＝B＋執行期偵測 fallback（沿用專案「面板失敗退原生選單」的分層退路模式）
- 建議 C：訴求本質是「跟原生一樣」，A 永遠是模仿；spike 驗不過再退 A 不浪費
**Ruled out**: 「原生選單＝近乎不透明主題色底」的錯誤模型——macOS 26 原生是自適應真玻璃（使用者更正）；換回 Menu/Popover 舊材質——同屬無自適應的舊世代，追不上原生且退回 2026-07-16 已淘汰的近不透外觀
**Open**: 路線抉擇（建議 C，待使用者裁定）；fallback 側（macOS 25 以前）要否順帶做 A 補光層或接受現狀；第一題遺留：中段捲軸要否隱藏

### Round 3 — assumptions (2026-07-31)

**Focus**: 毛玻璃路線與中段捲軸樣式的最終裁定
**Position**: 使用者裁定兩案，討論收斂：
- 毛玻璃走 A：維持現有 HudWindow 材質，面板 root 加主題色半透明 CSS 補光層先看效果（bg-background/55–65 起跳、真實視窗實測調參；theme token 隨 prefers-color-scheme，深淺模式自動）——B/C 的 NSGlassEffectView 真玻璃暫緩
- A 是全平台單一路徑（CSS 層不分 macOS 版本），先前「fallback 側要否補光」的分岔隨之消解
- 中段捲軸：維持 WebKit 預設 overlay 捲軸（捲動時浮現、停止自動淡出、不佔寬度）——使用者確認要避免常駐捲軸
**Ruled out**: NSGlassEffectView 真玻璃（B/C 路線）——暫緩非否決：需 spike 驗證且僅 macOS 26+，待 CSS 補光實測不滿意再重啟；中段捲軸完全隱藏——失去「還有內容可捲」的唯一提示；常駐捲軸——佔寬且突兀
**Open**: 無——全部節點已解，進結論

### Round 4 — assumptions (2026-07-31)

**Focus**: tray 動作區缺「專案設定」入口——落點、模型層與語意（使用者追加的第三題）
**Position**: 使用者確認全部 5 項假設：
- 動作區順序改為「開啟 Speclink → 專案設定 → 設定 → 結束」——專案級在前、app 級在後，呼應主視窗側欄層次（App.tsx:558-567）
- 改在 buildTrayModel 模型層加新 kind——macOS 面板與非 macOS 原生選單一次到位（tray.ts:118-120、279-281）；動作語意與「設定」同構：openIn(setBoardView("project-settings"))（tray.ts:476-477、547）；TrayStoreApi.setBoardView 型別自 "settings" 放寬（tray.ts:312）
- 導向作用中專案的專案設定、recovery 狀態不特判——tray 只負責喚起主視窗＋切頁，異常 workspace 的呈現交主視窗既有行為；動作區於 recovery 排列下照常顯示（與現行「設定」一致）
- 文案沿用既有 app.navProjectSettings「專案設定」（messages.ts:14）、圖示沿用側欄 SlidersHorizontal（App.tsx:558-563）——同概念同詞，不造新鍵
- 主視窗落點已存在：BoardView 已含 "project-settings"（store.ts:93），無新後端工作
- spec delta 兩處同步：原生選單需求（spec.md:10）與面板區塊順序（spec.md:335）的動作區三項→四項，相關 scenario（:14、:346 等）跟改
**Ruled out**: 只加面板不動原生選單——非 macOS 平台入口缺失、兩形態不一致；tray 端 recovery 特判——複雜度上升且與「設定」現行為不一致
**Open**: 無——三題全收斂，重寫結論納入第三題

## Conclusion

**Decision**: macOS tray 面板三項體驗修正，併同一個 change 落地：
1. 捲動範圍三段式——固定頁首（專案 tab 條＋分割線）、可捲中段（討論／已轉出／生命週期分區卡，含 recovery／stale 分支）、固定頁尾（分割線＋動作區）。捲動面從 body（panel.html）移入中段容器（flex-1 min-h-0 overflow-y-auto、overscroll-behavior:none 隨遷）；高度自適應量測改「頁首高＋中段自然內容高（scrollHeight）＋頁尾高」、上限 640 不變（main.tsx）；中段捲軸維持 WebKit 預設 overlay（捲動時浮現、自動淡出，不隱藏不常駐）；root 主色漸層 wash 固定於視窗。
2. 毛玻璃補光——維持 HudWindow 材質，面板 root 加主題色半透明 CSS 補光層（bg-background/55–65 起跳，真實視窗於深、淺兩種背景實測調參；判準：深色背景下亮度明顯上提、淺色背景下毛玻璃仍可辨）；theme token 隨 prefers-color-scheme 深淺自動；全平台單一路徑，不分 macOS 版本。
3. 動作區加「專案設定」——順序改為「開啟 Speclink → 專案設定 → 設定 → 結束」（專案級前、app 級後，呼應側欄層次）。改在 buildTrayModel 模型層加新 kind，macOS 面板與非 macOS 原生選單一次到位；動作語意與「設定」同構：openIn(setBoardView("project-settings"))，導向作用中專案、recovery 不特判；文案沿用 app.navProjectSettings、圖示沿用側欄 SlidersHorizontal；TrayStoreApi.setBoardView 型別放寬。主視窗落點（BoardView "project-settings"）已存在，無新後端工作。
**Rationale**: 捲動——tab 條與動作區是常駐導航／動作面，不應被內容捲走；量測基準改寫是避免「視窗高⇄root 高」循環依賴的必要配套。毛玻璃——偏暗根因是 NSVisualEffectView 舊世代材質無亮度自適應（macOS 26 原生 Liquid Glass 靠 tone-mapping 維持亮度），真自適應需 NSGlassEffectView spike 且僅 26+；先以 10 行級 CSS 補光驗證效果、保留 HudWindow 透感裁決（2026-07-16）。專案設定——模型層單點修改讓兩形態（面板／原生選單）同步取得入口，沿用既有詞彙與圖示避免漂移。
**Rejected alternatives**: 維持整頁捲（tab 條與動作區被捲走）；只修實作不動 spec（無回歸防護）；換回 Menu／Popover 舊材質（同無自適應、退回 2026-07-16 已淘汰的近不透外觀）；中段捲軸完全隱藏（失去可捲提示）或常駐（佔寬突兀）；專案設定只加面板不動原生選單（非 macOS 平台入口缺失）；tray 端 recovery 特判守門（複雜度上升且與「設定」現行為不一致）。
**Deferred**: NSGlassEffectView 真 Liquid Glass 路線（objc2 runtime 動態插入＋執行期偵測 fallback）——CSS 補光實測不滿意時重啟。
**Capture to**: proposal（spec delta → tray-status-menu：面板內部捲動限於內容分區、tab 條與動作區常駐；面板底色錨定主題背景、深色背景下不明顯偏暗；動作區「開啟 Speclink」「專案設定」「設定」「結束」四項——原生選單需求與面板區塊順序兩處同步）
**Next**: /speclink-propose --from-discussion tray-panel-scroll-and-vibrancy
