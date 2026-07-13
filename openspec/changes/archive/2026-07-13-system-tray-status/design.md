## Context

Speclink Desktop 是 Tauri 2 app：Rust 殼（apps/desktop/src-tauri）內嵌 speclink-desktop-core，前端（apps/desktop/src）以 Zustand store 管理看板資料與多專案分頁（tabs.ts，localStorage 持久化）。外部寫者改動 openspec/ 時，殼層檔案監看發出 workspace-changed 事件、前端整批刷新。平台架構藍圖 §10 定位 Desktop 為雙模式 UI（本地 LocalDataSource／遠端 RemoteDataSource），UI 只依賴 SpeclinkDataSource contract。

本變更整合兩份討論：system-tray-status（原生選單）與 tray-rich-panel（曾探索 CodexBar 式 webview 面板）。tray-rich-panel 一度裁定改用 webview 面板，實作後發現：CodexBar/ChatGPT 的原生質感來自原生 Swift/AppKit（NSPopover＋SwiftUI 視圖），Tauri webview 本質追不到（透明/vibrancy/字距/自動貼合皆是 AppKit 專利，且 setSize 與 NSPanel 相衝）。使用者遂裁定放棄 webview 面板、回歸原生 NSMenu——並在原生選單能力內把內容做豐富（分區、進度條、子選單、討論列表）。webview 面板整套（面板視窗、NSPanel、後端分頁 sink、雙進入點）已移除。

## Goals / Non-Goals

**Goals**

- 系統匣圖示＋macOS 進行中數徽章：三平台共用
- 原生下拉選單（menu-first，跨平台一致）：專案切換、生命週期分區與變更進度、變更子選單動作、討論列表、開窗/結束
- 資料與看板同源：選單內容訂閱既有前端 store，本地/遠端同一條 DataSource

**Non-Goals**

- 關窗背景常駐（隱窗不關、ActivationPolicy、Dock 圖示）——獨立生命週期改造
- 自訂 webview 彈出面板——原生 AppKit 質感 Tauri webview 追不到，已探索後放棄（見 Context）
- Rust 側直呼 core 組選單（遠端模式形成第二條資料路徑）

## Decisions

**D1 — tray 由前端擁有，以 Tauri 2 JS tray API 實作**
新模組 apps/desktop/src/tray.ts 於前端建圖示、組選單、掛點擊 handler；Rust 側零新模組。理由：藍圖 §2.4/§10.3 要求 UI 只依賴單一 DataSource contract——tray 掛在 store 上，本地走既有監看刷新、未來遠端走 SSE/polling invalidation 免費繼承。替代方案：Rust 直呼 core（遠端需重複 Client SDK，否決）。

**D2 — 互動一律原生選單（menu-first），跨平台一致**
原生 NSMenu 是三平台唯一穩的形狀（Linux tray 點擊事件不觸發、只有原生選單可靠）；且原生選單才有真原生質感（vibrancy、⌘Q、字距）——這是放棄 webview 面板後的核心結論。

**D3 — 選單重建走 store 訂閱＋去抖**
tray.ts 以 store 訂閱監聽分頁/變更/討論變化，去抖後整份重建選單（含 macOS setTitle 徽章）。與看板「事件即 invalidation hint、收到即整批重讀」同一心智模型。

**D4 — 專案切換復用 store 既有動作，不搶焦點**
專案項點擊直接呼叫 store 的 openProjectAt；切換 SHALL NOT 呼叫視窗 show/focus。變更子選單「開啟此變更」與討論項則刻意開主視窗（openMainWindow）＋跳詳情——這兩類是「我要去看它」的意圖，開窗合理。

**D5 — 圖示 template 技法與 D5a — 使用者提供的 Speclink 標記**
單色 tray 圖示內嵌為 apps/desktop/src/trayIcon.ts 的 base64 常數，以 Image.fromBytes 解碼建圖示（需 image-png feature）。放前端而非以 Rust 資源載入：tray 為前端擁有（D1），Rust 資源載入需額外 capability。使用者已提供選單列尺寸的 Speclink 標記（src-tauri/icons/speclink-tray-18.png 18×18＋@2x 36×36，單色深藍剪影），以其 @2x base64 編碼填入 trayIcon.ts。macOS 以 iconAsTemplate 依 alpha 渲染為系統色（適應深淺選單列）。macOS 徽章以 setTitle 呈現進行中變更數（0 時清空）；僅 macOS 生效，wiring 以 navigator.userAgent 精確偵測（/Macintosh|Mac OS/，避免 node 平台字串 "darwin" 誤中）。app bundle 圖示（icon.icns 等）由使用者更新透明度，tauri.conf.json 既已引用、下次 build 自動採用。

**D6 — Rust 殼僅開能力，不加邏輯**
Cargo.toml 啟用 tauri 的 tray-icon 與 image-png features；capabilities/default.json 補齊 tray、menu、視窗 show/set-focus/unminimize 權限。缺權限的表徵是前端呼叫被拒，驗收以「選單各動作實際可用」為準。

**D7 — 選單內容模型（豐富但在原生能力內）**
buildTrayModel（純函式）自 store 快照導出選單模型，區段順序：專案區（作用中打勾）→ 分隔 → 生命週期分區（提案中/進行中/已就緒，每個非空階段一個 header 分區標題＋各變更；全無變更則空狀態）→ 分隔 → 討論區（header＋各 active 討論項，無則「討論 0」）→ 分隔 → 動作區（開啟 Speclink、結束）。變更列標籤帶 unicode 文字進度條（▓░，total 為 0 不畫）與「名稱 n/m」；每張變更是子選單，含「開啟此變更」動作。header 以 disabled MenuItem 呈現（原生無 section header type 時的等效）。徽章＝進行中變更數。理由：分區/進度條/子選單/討論列表都在原生 NSMenu 能力內（文字項、disabled 標題、Submenu、進度以 unicode 文字），既豐富又保住真原生質感。

**D8 — 測試策略**
tray.ts 拆純函式核心與 Tauri API 接線兩層：buildTrayModel（分頁＋變更＋討論 → 分區/進度條/子選單/討論 的選單模型結構）純函式 vitest 直測（含 progressBar）；Tauri tray/menu API 以 vi.mock 樁替，驗證「store 變化 → 去抖 → 以新模型重建」與「點擊 handler → 呼叫對應 store 動作/視窗 API」（專案切換、變更子選單開詳情、討論項開討論、開窗、結束）。真實選單顯示屬 GUI 驗證（jsdom 測不出），以真實視窗驗證。

## Implementation Contract

**觀察得到的行為**

- 三平台：app 啟動後系統匣出現 Speclink 單色圖示；macOS 圖示旁徽章顯示進行中變更數、0 時清空
- 點擊圖示展開原生選單，依序：專案區（作用中打勾）→ 分隔 → 各非空階段（提案中/進行中/已就緒）的分區標題與其變更（進行中變更帶文字進度條＋n/m）→ 分隔 → 討論區（各 active 討論）→ 分隔 → 開啟 Speclink、結束
- 點非作用中專案 → 看板與選單同步切至該專案，主視窗焦點/前景不變
- 變更子選單「開啟此變更」→ 主視窗顯示並聚焦、開啟該變更詳情抽屜
- 討論項 → 主視窗顯示並聚焦、開啟該討論抽屜
- 「開啟 Speclink」→ 主視窗顯示並聚焦；「結束」→ app 結束（原生 predefined Quit，macOS 顯 ⌘Q）
- 外部寫者改動 openspec/ 後，下次展開選單顯示新狀態；徽章同步更新

**介面/資料形狀**：選單模型為純資料（項目種類：project/header/change(含 actions)/discussion/empty/separator/open/quit），由 buildTrayModel 自快照導出；tray.ts 對外暴露初始化入口（App.tsx 啟動呼叫）與其回傳的銷毀函式。

**驗收**：npm test -w apps/desktop 全綠（含選單模型、progressBar、接線測試）；cargo build --release -p speclink-desktop 成功；macOS 真實視窗驗證上述行為。

**範圍**：in——tray.ts、trayIcon.ts、App.tsx 初始化、i18n 文案、Cargo features、capabilities、tray 圖示資產、單元測試；out——speclink-core/speclink-cli、CLI 輸出、視窗生命週期（關窗語意不變）。

## Risks / Trade-offs

- [Linux 環境缺 AppIndicator 支援（如 GNOME 未裝擴充）圖示不可見] → 功能屬增益面，不影響主視窗；不承諾 Linux 全桌面環境可見
- [選單重建瞬間使用者正展開選單可能閃爍] → 去抖合併連續變動；重建僅在資料實際變化時發生
- [tray 只在 webview 存活期間更新] → 第一版關窗即退出，無隱窗狀態
- [原生選單無法放進度條/兩行項等原生視圖] → 以 unicode 文字進度條與子選單在原生能力內近似；真 AppKit 視圖（NSPopover＋SwiftUI）Tauri 追不到，屬明列 Non-Goal

## Migration Plan

無遷移需求：純新增呈現面，不動既有資料、設定與 CLI 行為。回滾即移除 tray 初始化與資產，主視窗功能不受影響。

## Open Questions

（無——面板方向已探索並放棄，回歸原生選單；分區/進度條/子選單/討論列表已落實）
