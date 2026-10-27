---
topic: 系統匣改用 CodexBar 式豐富彈出面板（Linux 退原生選單）
slug: tray-rich-panel
status: promoted
promoted_to: system-tray-status
created: 2026-07-13
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 系統匣改用 CodexBar 式豐富彈出面板（Linux 退原生選單）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者看到 system-tray-status 剛做好的原生選單覺得「陽春」，提出想做成 CodexBar（https://github.com/steipete/CodexBar，純 macOS 選單列 app）那種豐富彈出面板。關鍵事實：原生 Menu（NSMenu）只能放文字/勾號/分隔線，做不到進度條/彩色帶/多欄/玻璃擬態——CodexBar 是自訂 webview popover 視窗。這正是 system-tray-status 討論明文否決的「自訂 popover 面板」（因 Linux tray click 事件不觸發）。使用者已裁定：(1) 平台範圍＝macOS/Windows 面板＋Linux 退回既有原生選單；(2) 先開短討論再推進。模式：假設模式——脈絡充足（本 session 剛實作 apps/desktop/src/tray.ts 204 行的原生選單、熟悉 store.ts/tabs.ts/App.tsx 與 Tauri tray/menu/window API）。探到：WebviewWindow 支援 transparent/decorations/alwaysOnTop/skipTaskbar/windowEffects(vibrancy)/shadow；tray click 事件帶 rect{position,size} 可定位；無專用 window-effects plugin（內建 WindowOptions）。相關 change：system-tray-status（status promoted，9/10，原生選單版）。參考圖 scratchpad/codexbar.png。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-13)

**Focus**: 面板的呈現機制與資料架構（CodexBar 式豐富 UI 怎麼在 Tauri 落地）
**Position**: 面板＝獨立 WebviewWindow＋輕量 store，走同一條 DataSource：
- 硬事實：第二個 popover 是獨立 WebviewWindow＝獨立 JS context，無法共用主視窗的記憶體 Zustand store 物件；「同源」指同一條 DataSource（Tauri 查詢指令）＋同一批 workspace-changed 事件，非同一 store 實例
- 面板視窗：label 如 tray-panel，decorations:false、transparent:true、skipTaskbar:true、alwaysOnTop:true、windowEffects 給 vibrancy；預設隱藏，tray 點擊以 TrayIconEvent.rect 定位到圖示下方後 show()＋失焦 hide()
- 面板掛精簡 store（開啟查一次＋訂閱 workspace-changed 增量），只保留呈現要的資料（專案、changes、討論數），不搬主視窗專屬的抽屜/搜尋/確認框狀態
- CodexBar 視覺→Speclink 內容：頂部分頁＝專案（點選切換、不奪焦）；主體按生命週期分區（提案中/進行中/已就緒/討論）、每張 change 一條任務進度條（n/m，色帶用 stage.ts 的 teal STAGE_BADGE）；底部動作列（開啟 Speclink、設定、結束）。不照搬 usage/token 語意——那是 CodexBar 領域，我們的是 SDD 生命週期
- 符合平台架構藍圖 §10.3：面板與看板都是同一 DataSource 的 presentation adapter，不新增第二條資料路徑
**Ruled out**: 面板嵌主視窗當 overlay（選單列彈出須在主視窗關閉時也能出現，必為獨立視窗）；面板複製整個 AppState（兩 context 各跑全量狀態、徒增同步面）
**Open**: macOS 面板要不要 NSPanel non-activating；面板的 tabs 來源（localStorage 跨窗 vs 下沉後端）

### Round 2 — assumptions (2026-07-13)

**Focus**: 平台分流、macOS 面板品質、跨窗 tabs 一致性、與 system-tray-status 的關係
**Position**: Linux 復用既有選單、macOS 用 NSPanel crate、tabs 下沉後端、以 ingest 併入 system-tray-status：
- 平台分流：initTray 依平台走向——macOS/Windows 面板（showMenuOnLeftClick:false＋click handler 開面板）、Linux 保留現有 buildTrayModel＋原生 Menu（system-tray-status 那 204 行選單碼原封當 Linux 分支）
- macOS 面板用社群 crate tauri-nspanel 做 non-activating panel（不搶焦點、不進 Mission Control，貼近 CodexBar 原生體驗）——確切 crate 版本由 design 釘死；代價是多一個 Rust 依賴
- tabs 下沉後端：目前 tabs 存主視窗 localStorage，另一 webview 讀不到；改由後端持有分頁清單讓主視窗與面板共讀，避免兩窗專案分頁不一致（面板切換走同一後端狀態）
- 承載範圍：以 ingest 併入既有 system-tray-status（非新開 change）——面板與選單是同一 tray 功能的兩半（Mac/Win 面板、Linux 選單 fallback），共用同一 tray 圖示與初始化入口；使用者裁定
- 後果（已知並接受）：system-tray-status 由 9/10 退回進行中，多出面板視窗/NSPanel/後端 tabs/面板 UI 與測試任務，須等面板做完才能封存；既有原生選單 9 項（含已驗證的模型/接線）不作廢、轉為 Linux 分支
**Ruled out**: 另開新 change tray-rich-panel（使用者選擇 ingest 併入——同一功能兩半不宜拆兩張卡）；tabs 靠跨窗共享 localStorage（同 app 資料目錄行為需驗證、且面板應讀後端權威狀態）
**Open**: 無——分歧已裁定；面板細部佈局與 NSPanel crate 版本屬 propose/design 細節

## Conclusion

**Decision**: 系統匣在 macOS/Windows 改用自訂 WebviewWindow popover 面板（CodexBar 式豐富 UI），Linux 退回既有原生選單。面板要點：獨立 WebviewWindow（decorations:false、transparent:true、skipTaskbar:true、alwaysOnTop:true、windowEffects vibrancy），預設隱藏、tray 點擊以 TrayIconEvent.rect 定位到圖示下方 show()、失焦 hide()；macOS 以社群 crate tauri-nspanel 做 non-activating panel；面板掛精簡 store，經同一條 DataSource＋workspace-changed 事件與看板同源；內容按 SDD 生命週期呈現（頂部專案分頁、每張 change 任務進度條 n/m 用 teal STAGE_BADGE、提案中/進行中/已就緒/討論分區、底部動作列）。tabs 清單下沉後端讓主視窗與面板共讀。initTray 依平台分流：Mac/Win 面板、Linux 現有 buildTrayModel＋原生 Menu。
**Rationale**: 原生 Menu 有硬天花板（只能文字/勾號/分隔線），CodexBar 式豐富 UI 必為自訂 webview 面板——這推翻 system-tray-status 原討論「否決 popover」的結論，但該結論的理由（Linux click 不觸發）以「Linux 退原生選單」化解，Mac/Win 得到豐富面板、Linux 仍可用。
**Rejected alternatives**: 面板嵌主視窗 overlay（選單列彈出須在主視窗關閉時也能出現，必為獨立視窗）；面板複製整個 AppState（兩 webview context 各跑全量狀態、徒增同步）；tabs 靠跨窗共享 localStorage（面板應讀後端權威狀態，且同資料目錄跨窗行為需驗證）；另開新 change（使用者選擇 ingest——面板與選單是同一 tray 功能兩半）。
**Deferred**: 面板細部佈局、tauri-nspanel 確切 crate 版本、面板動作列項目集——屬 propose/design 細節。
**Capture to**: 既有 change system-tray-status（經 /speclink-ingest 更新 proposal/design/specs/tasks——非新開 change）
**Next**: speclink discuss link tray-rich-panel system-tray-status → /speclink-ingest system-tray-status
