> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ① 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: 無（本刀打底）。**下游**: ② desktop-acp-agent 疊在本刀之上；③ web-server-postgres 復用本刀的 React 共用元件庫。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。

## Why

情境 4（完全本地）目前只有 CLI（`speclink.exe`）可用，缺一個「像 spectra.exe 一樣雙擊即跑」的桌面工具讓使用者看得到 change 看板、文件樹與 spec，並直接執行動詞。本刀交付情境 4 的桌面儀表板，同時打底一套跨桌面/web 共用的 React 元件庫——後續情境 3 的 web GUI（③）直接復用，避免兩邊各刻一份。

參照架構（反組譯 spectra 2.3.1 取得）：Tauri 殼直嵌引擎、SQLite 當檔案系統之上的快取/索引、markdown 檔仍為真相（git 跟隨）。本刀對齊此架構但前端改用可跨桌面/web 共用的框架。

## What Changes

- 新增 Tauri 桌面 app 殼，直嵌 `speclink-core`（同步 Rust lib，非 sidecar），以 Tauri command 暴露引擎動詞給前端。
- 新增 React 共用元件庫（TailwindCSS + shadcn/ui 設計系統，狀態管理 Zustand 於 app 層）：change 看板、文件樹、spec 瀏覽、change/spec 內容檢視——設計系統與領域元件放 packages/ui，可被情境 3 的 web GUI 復用。
- 唯讀瀏覽 ＋ 動詞操作面：list / show / status / validate / analyze / archive。
- SQLite 索引快取層：加速歸檔（archived）change 清單呈現，供 GUI 快速查詢；本地 `openspec/` markdown 為真相來源，快取為衍生、可重建、帶 schema 版本。

<!-- 細節（React 選型、Tauri 版本、SQLite schema、Tauri command 面）待 /speclink-propose 於 design 階段定案 -->

## Capabilities

### New Capabilities

- `desktop-app`: 情境 4 的本地桌面 GUI——Tauri 殼直嵌引擎、React 元件庫、SQLite 索引快取、動詞操作面。

## Impact

- Affected crates: 新增桌面 app（Tauri，直嵌 speclink-core）；speclink-core 可能需補少量 query API 供 GUI。
- 共用資產: 新增 React 元件庫，情境 3（③）復用。
- 不影響 CLI 與現行 fs 模式行為（回歸對照不受影響）。
- 儲存: 新增本地 SQLite 索引快取（衍生、可重建）；真相仍為 `openspec/` markdown。
