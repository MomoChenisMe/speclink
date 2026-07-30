## Why

release 管線目前只發 CLI 與 server 二進位（見 .github/workflows/release.yml 明文註記「桌面 app 不是 release 產物」），想使用桌面看板的人只能 clone 原始碼自建。為了讓 GitHub 訪客下載即得 desktop＋CLI、並在後續版本自動更新，desktop 必須升格為 release 產物並內建更新機制。

- 目標使用者：想直接安裝使用 Speclink 的開發者（不 clone 原始碼、不碰建置工具鏈）。
- 使用情境：初次安裝（下載一個安裝檔同時取得 desktop 與 CLI）與後續版本更新（desktop 自動更新、CLI 搭便車），不對應特定 speclink 技能。
- 源自已結論討論 one-click-install-and-run；同時落實 keychain-always-allow-reprompt 討論 Deferred 的「正式發行簽章策略」裁定（暫不投資簽章、預留開關）。

## What Changes

- release workflow 新增三平台 desktop 安裝檔（macOS dmg、Windows NSIS、Linux AppImage 與 deb）與自動更新描述檔（latest.json），隨 GitHub Release 一併發布；release 產物「全有全無」的承諾由四形態擴為五形態。
- 安裝檔以未簽章起步；OS 程式碼簽章（Apple Developer ID／Windows 憑證）做成 env-gated 開關——對應 secrets 存在才簽、不存在照常產出未簽章版，未來升級是填鑰匙而非改管線。
- 自動更新的完整性採 Tauri updater 簽名（自產金鑰對）：私鑰存 GitHub secrets 於 CI 簽更新包、公鑰嵌入 app；desktop 啟動後檢查 GitHub Releases 上的更新描述檔，經使用者同意後下載並套用更新。無自架伺服器。
- desktop 新增「安裝 CLI 指令」動作：macOS 與 Linux 以 symlink 把 app bundle 內的 speclink CLI 加入 PATH（Windows 由安裝器佈署並寫入 PATH），CLI 隨 desktop 更新自動同版。
- 相容性影響：既有 CLI／server 壓縮檔命名、SHA256SUMS、Docker 映像的形態與名稱不變，新增產物不影響既有下載腳本；desktop 的人眼介面新增更新與安裝 CLI 的入口，不改動既有 CLI 指令的人眼輸出與 --json shape。

## Non-Goals

- 不購買 Apple Developer ID 與 Windows 簽章憑證、不做 notarization——本變更僅預留簽章開關；投資觸發時機（非開發者用戶出現、或 remote 模式成主打）依討論 Deferred 另行裁定。
- 不自架 update server——分階段灰度、更新統計、私有 repo 皆不在情境內。
- CLI 不做獨立 self-update 子指令——symlink 模式下與 desktop 更新重複。
- 不含 clone 開發者的一鍵執行 scripts 與開發文件——屬同討論扇出的第二個變更（dev-quickstart-and-docs）。

## Capabilities

### New Capabilities

- `desktop-release`: desktop 發行產物與更新通道——release 管線產三平台安裝檔、更新描述檔與 updater 簽名，簽章為 env-gated 開關，GitHub Releases 即更新端點。

### Modified Capabilities

- `desktop-app`: 新增兩項行為要求——自動更新（檢查、徵得同意、套用、失敗不影響既有安裝）與「安裝 CLI 指令到 PATH」動作（含已安裝狀態的呈現）。

## Impact

- Affected specs: `desktop-release`（新增）、`desktop-app`（修改）
- Affected code:
  - New: 無新增頂層模組；更新檢查與安裝 CLI 的介面元件落點由 design 定於 apps/desktop/src 之下
  - Modified: .github/workflows/release.yml、apps/desktop/src-tauri/tauri.conf.json、apps/desktop/src-tauri/Cargo.toml、apps/desktop/src-tauri/src/lib.rs、apps/desktop/package.json、apps/desktop/src/store.ts
  - Removed: 無
- 相依性：新增 tauri-plugin-updater（Rust 與前端綁定各一）；CI 新增 GitHub secrets 兩枚（updater 私鑰與其密語）
- 驗收註記（2026-07-30）：可自動化驗證全數通過（vitest 392、node --test 62、cargo check、actionlint 零新增、本機 tauri build 產出簽章更新包與同版 sidecar CLI）；tasks 5.2／5.3／6.1 的 tag 級驗收（預演 tag 實跑 release 管線、Release assets 對照、端到端更新迴圈、Windows 跨機器 PATH）依使用者裁定延後至首次發行 tag 時執行，屆時逐項對照 desktop-release 與 desktop-app spec 場景。release.yml 已加 tag＝tauri.conf.json 版本的 fail-closed 閘門，防止未 bump 版本即發 tag 造成的無盡更新迴圈。
