---
topic: Speclink Desktop 在系統選單列顯示討論與變更狀態
slug: system-tray-status
status: promoted
promoted_to: system-tray-status
created: 2026-07-13
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: Speclink Desktop 在系統選單列顯示討論與變更狀態

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者附截圖提出：希望 Speclink Desktop 進駐 macOS 選單列，點擊顯示討論與變更的狀態資訊，並詢問其他平台會是什麼形狀。模式：假設模式——scout 找到 `apps/desktop/src-tauri/src/lib.rs`（IPC 薄委派層）、`watch.rs`（openspec/ 檔案監看，已 emit workspace-changed）、`tauri.conf.json`、`apps/desktop/src/store.ts`（多專案分頁），脈絡充足。現況：Tauri 2 殼、無任何既有 tray 程式碼（Cargo.toml 未啟 tray-icon feature）；多專案分頁清單持久化在前端 localStorage（tabs.ts）。進行中變更 protocol-typed-client、context-materializer 與本題無關。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-13)

**Focus**: tray 的形式與跨平台約束
**Position**: 原生選單優先（menu-first）的 tray，選單內容由 Rust 側直接組出：
- 選單項目直接呼叫 speclink-desktop-core 查詢（list_changes_at 等）組出，不經前端——lib.rs 的 command 本就是對 core 的薄委派，同源直取
- 刷新掛既有 watch.rs 監看回呼（lib.rs:320 的 workspace-changed 發射點），切專案時 switch_root 已會重掛監看，tray 自然跟著換
- 跨平台形狀：macOS 選單列 NSStatusItem（template 單色圖示＋set_title 文字徽章顯示進行中數，僅 macOS 生效、零分支成本）；Windows 右下系統匣（常被收進溢出區）；Linux AppIndicator（GNOME 需擴充套件）
- Linux 硬約束：tray 點擊事件不觸發（Tauri 明文限制），唯一可靠互動是原生選單——這決定了 menu-first 是跨平台唯一穩的形狀
- 第一版不做「關窗縮到 tray 背景常駐」——牽涉 ActivationPolicy、Dock 圖示、結束語意，屬獨立的生命週期改造
**Ruled out**: 自訂 popover 面板視窗（Dropbox 式）——Linux 上點擊事件不觸發整個失效，且多出無邊框視窗、定位插件、失焦關閉的成本
**Open**: tray 顯示單一使用中專案還是彙總全部分頁；背景常駐要不要做

### Round 2 — assumptions (2026-07-13)

**Focus**: tray 是否承載專案切換（使用者對假設 3 的修正）
**Position**: 承載——tray 列出全部專案分頁、點選即切換，desktop 視窗同步跟切：
- 分頁清單權威在前端 localStorage（tabs.ts STORAGE_KEY），Rust 讀不到 → 前端在 persistTabs 這個唯一收斂點把清單推給 Rust（新增一個 sync 型 command），Rust 存進 state 並重建 tray 選單
- tray 點選專案 → Rust 直接走既有 switch_root（lib.rs:203）＋ emit 事件給前端 → 前端以既有 openProjectAt 路徑收斂（重複 probe 冪等、無迴圈：重建選單不再發事件）
- 專案目錄失效（被移走/刪除）時沿用既有 probe 失敗處理（store 的 tabErrors），tray 不需自建錯誤路徑
- MAX_TABS=10（tabs.ts:17）→ 選單長度天然受控；作用中專案以勾選標記
**Open**: tray 切換專案後要不要把視窗帶到前景；背景常駐（前輪遺留）

### Round 3 — assumptions (2026-07-13)

**Focus**: 遠端模式（platform-architecture.zh-TW §10）對 tray 資料路徑的約束；切換後焦點行為裁定
**Position**: tray 改為前端擁有、以 Tauri JS tray API 實作的純呈現面——第 1 輪「Rust 側直組選單」被推翻：
- 藍圖 §10.1 定位 Desktop 為雙模式 UI（LocalDataSource 內嵌 Engine／RemoteDataSource 走 Client SDK＋協商事件），§10.3 的 SpeclinkDataSource contract 讓 UI 不知道背後是 Tauri 還是遠端
- 若 tray 在 Rust 直呼 speclink-desktop-core，遠端模式下就得在 Rust 重複一整套 Client SDK＋事件訂閱——形成第二條資料路徑，違反單一 DataSource contract（§2.4 UI 是 presentation adapter）
- 反轉後：tray 掛在前端 store 上（Zustand subscribe → 去抖重建選單），資料與看板同源——本地走 watcher、遠端走 SSE/polling invalidation（§9），tray 免費繼承，且看板與 tray 永不分歧
- Tauri 2 的 JS tray API（@tauri-apps/api/tray）可全程在前端建圖示、組選單、掛點擊 handler——Rust 側零新模組，只需開 tray-icon feature＋capability 權限
- 專案切換點擊 handler 直接呼叫 store 既有 openProjectAt——第 2 輪的「Rust switch_root＋事件回推」管線也不需要了
- 焦點行為採 (b)：切換不搶焦點；開視窗走選單的「開啟 Speclink」項（show + setFocus）
- 代價：tray 只在 webview 存活期間更新——第一版「關窗即退出」下無此問題；未來背景常駐改「隱窗不關」即相容
**Ruled out**: Rust 側直呼 core 組選單（遠端模式需重複 Client SDK，雙資料路徑）；「前端推 tray model、Rust 渲染」的中間層（JS tray API 已覆蓋，中間層是零深度 pass-through）
**Open**: 背景常駐第一版不做——使用者尚未明確確認

## Conclusion

**Decision**: 系統匣狀態選單以「前端擁有」實作——新前端模組 tray.ts 用 Tauri 2 的 JS tray API（@tauri-apps/api/tray）建原生選單，內容訂閱 store（Zustand subscribe → 去抖重建）：專案分頁列（點選即切換、作用中打勾、不搶焦點）、進行中變更進度（名稱 n/m）、討論數、「開啟 Speclink」（show + setFocus）、「結束」；macOS 額外以 set_title 顯示進行中數文字徽章（其他平台自動忽略）。Rust 側零新模組，僅開 tray-icon feature＋capability 權限。第一版維持關窗即退出（使用者確認）。
**Rationale**: 兩個約束共同決定形狀——(1) Linux tray 點擊事件不觸發（Tauri 明文限制），唯一可靠互動是原生選單 → menu-first；(2) 平台藍圖 §10 定位 Desktop 為雙模式 UI，UI 只依賴 SpeclinkDataSource contract → tray 資料必須與看板同源（前端 store），本地走 watcher、遠端走 SSE/polling invalidation 免費繼承，看板與 tray 永不分歧。
**Rejected alternatives**: 自訂 popover 面板視窗（Linux 點擊失效整個功能不可用，且多出無邊框視窗/定位/失焦成本）；Rust 側直呼 speclink-desktop-core 組選單（遠端模式需在 Rust 重複 Client SDK＋事件訂閱，形成第二條資料路徑，違反 §2.4/§10.3）；「前端推 tray model、Rust 渲染」中間層（JS tray API 已覆蓋，屬零深度 pass-through）。
**Deferred**: 關窗背景常駐（隱窗不關、ActivationPolicy、Dock 圖示去留——獨立的生命週期改造，現有設計以「隱窗不銷毀」即可相容）；遠端分頁在選單上的視覺區分（等遠端模式落地）。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion system-tray-status
