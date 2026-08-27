# Speclink 開發環境入口

**繁體中文** · [English](development.md)

這份文件面向 clone 原始碼的開發者，以及想自架後端的使用者。它涵蓋 repo root 的五個一鍵入口：整套開發環境、只跑 server、只跑 desktop、重置本機狀態，以及以同一份原始碼執行 CLI。最後附上下載安裝檔時的未簽章放行步驟。

## Prerequisites / 前置需求

- stable Rust toolchain（`cargo` 可用）
- Node.js 與 npm；首次 clone 後先在 repo root 執行：

```bash
npm install
```

- `npm run dev` 與 `npm run dev:desktop` 會啟動 Tauri 桌面視窗，需要該平台的 Tauri 系統依賴（macOS 為 Xcode Command Line Tools）。只跑 `npm run dev:server` 或 `npm run cli` 則不需要。

## `npm run dev` — full dev environment / 整套開發環境

- **用途**：一次啟動完整開發環境。它先建置當前 checkout 的 CLI，然後同時啟動 `speclink-server` 與 desktop 的 tauri dev；desktop 前端由 tauri dev 啟動的 Vite dev server 供應，改前端不需要重建。
- **前置條件**：上述全部前置需求。設定來源是 repo root 的 `.env`，可對照 `.env.example`。沒有 `.env` 時全用預設值：sqlite、`.dev/store.db`、`127.0.0.1:8080`。
- **預期可觀察結果**：終端先出現 `speclink dev: 建置當前 checkout 的 speclink-cli…`。首次啟動時 server 印出一行含 `/setup?token=` 的連結，只顯示一次，開啟它完成初始設定。接著 desktop 視窗開啟。Ctrl+C 同時收束 server 與 desktop，無殘留 process。狀態持久化於 gitignored 的 `.dev/`。

## `npm run dev:server` — server only / 只跑後端

- **用途**：只檢查 dev 設定並啟動 `speclink-server`，適合在這個 checkout 內做純後端開發（checkout 之外要自架，最短路徑是 `npx @speclink/server`，見 [Remote 入門教學](remote-getting-started.zh-TW.md)）。它不建置 CLI、不建置前端，也不開 desktop 視窗。
- **前置條件**：Rust toolchain 與 `npm install`。零設定即可啟動，預設走 sqlite。設定不合法時它以非零 exit code 拒絕啟動，錯誤訊息與 `npm run dev` 相同——例如 `SPECLINK_STORE_DRIVER=postgres` 但缺 `SPECLINK_POSTGRES_URL`。
- **預期可觀察結果**：全新環境下，終端印出含 `/setup?token=` 的連結。過程中沒有任何建置步驟，也沒有 desktop 視窗。Ctrl+C 後無殘留 process，`.dev/` 的持久化行為與 `npm run dev` 一致。

## `npm run dev:desktop` — desktop only / 只跑桌面

- **用途**：直接啟動 tauri dev。適合桌面開發與本地模式試用。前端由 Vite dev server 供應，改原始碼即時反映。它不啟動 server，也不要求任何 remote 設定。
- **前置條件**：前置需求全項，含 Tauri 系統依賴。設定檢查與 `npm run dev` 共用。`.env` 不合法時，例如 postgres 缺 `SPECLINK_POSTGRES_URL`，它同樣以非零 exit code 拒絕啟動。
- **預期可觀察結果**：沒有前置建置步驟——tauri dev 啟動 Vite dev server 後開啟 desktop 視窗，畫面來自當前原始碼，改前端會就地重載，你不會看到過期畫面。前端 dev server 起不來時，指令以非零 exit code 結束且不開視窗。視窗以本地模式運作，可直接瀏覽本 repo 的 `openspec/` 看板，機器上沒有 server 也能用。

## `npm run dev:reset` — reset local dev state / 重置本機開發狀態

- **用途**：清空 `.dev/`，也就是 dev 用的 server 設定與資料庫，讓下次 `npm run dev` 回到全新 `/setup`。它不建置任何東西，也不碰 `.env` 與 `deploy/`。
- **前置條件**：無（對不存在的 `.dev/` 冪等成功）。
- **預期可觀察結果**：終端印出 `speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。` 之後立即結束。注意：postgres 的資料在外部資料庫，不在 reset 範圍內。

## `npm run cli -- <args>` — CLI from this checkout / checkout 內 CLI

- **用途**：固定執行同一 checkout 的 `target/debug/speclink`，讓「啟動環境」與「以同版 CLI 核對」落在同一份原始碼上。它絕不使用 PATH 中已安裝的 `speclink`。
- **前置條件**：Rust toolchain。binary 尚未建置時，wrapper 會先在 checkout root 執行 `cargo build -p speclink-cli`，再執行結果。建置失敗就以非零 exit code 結束，不執行任何 CLI。
- **預期可觀察結果**：

```bash
npm run cli -- --version
```

首次執行時，或刪掉 binary 之後，你會先在 stderr 看到 cargo 建置進度，接著才是版本。需要純 machine-readable stdout 時加 `--silent`：

```bash
npm run --silent cli -- list --json
```

stdout 只含 CLI 的 JSON payload，欄位為 camelCase，建置進度不會混進來。CLI 的 exit code、stdin、stdout、stderr 與工作目錄（`INIT_CWD`）都透明轉送。

## Tests / 測試

一次跑完全部測試面——Rust workspace、三個前端 package、scripts，以及 Node SDK 的建置與測試：

```bash
npm run test:all
```

只跑其中一塊時分別是：

```bash
cargo test --workspace                      # Rust（引擎、CLI、Host、Store drivers、Server）
npm test -w apps/desktop                    # 桌面前端
npm test -w packages/ui                     # 共用 UI
npm test -w apps/server-web                 # Server 後台前端
node --test "scripts/**/*.test.mjs"         # repo 腳本
npm --prefix crates/speclink-node test      # Node SDK（需先 npm --prefix crates/speclink-node run build）
```

Node SDK 不是 npm workspace 成員，所以要用 `--prefix` 指路徑；`--workspace @speclink/engine` 會找不到。桌面測試需要先備妥 sidecar 與 server 後台的 `dist/`，而 `npm run test:all` 已經含這兩步。

golden 與 CLI 整合測試守住 CLI 的人眼輸出與 `--json` shape。Server、Store 與 Desktop 各自的測試前提與目前限制，見[專案能力狀態](product-status.zh-TW.md)。

## Unsigned installer bypass / 下載安裝檔的未簽章放行

桌面安裝檔都在 [Releases](https://github.com/MomoChenisMe/speclink/releases/latest) 上：macOS dmg（aarch64 與 x86_64 各一）、Windows NSIS 安裝器（x86_64），以及 Linux 的 AppImage 與 deb（x86_64 與 aarch64）。`SHA256SUMS.txt` 收錄全部檔案。三平台的簽章狀態不同，只有 Windows 需要手動放行：

### macOS

已完成程式碼簽章與公證。直接開啟 dmg 拖進「應用程式」即可，不需要任何放行步驟。只有在你開自己本機建置的未簽章 bundle 時，才需要走「系統設定 > 隱私權與安全性 > 強制打開」。

### Windows

1. 執行 NSIS 安裝器時，SmartScreen 顯示「Windows 已保護您的電腦」。
2. 點「其他資訊」（More info），再點「仍要執行」（Run anyway）即可繼續安裝。

### Linux

AppImage 與 deb 沒有對應的簽章阻擋機制。AppImage 加上執行權限（`chmod +x`）就能執行，deb 依發行版慣例安裝。
