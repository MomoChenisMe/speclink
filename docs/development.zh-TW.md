# Speclink 開發環境入口

**繁體中文** · [English](development.md)

這份文件面向 clone 原始碼的開發者與想自架後端的使用者，涵蓋 repo root 的五個一鍵入口：整套開發環境、只跑 server、只跑 desktop、重置本機狀態，以及以同一份原始碼執行 CLI。最後附上下載安裝檔時的未簽章放行步驟。

## Prerequisites / 前置需求

- stable Rust toolchain（`cargo` 可用）
- Node.js 與 npm；首次 clone 後先在 repo root 執行：

```bash
npm install
```

- `npm run dev` 與 `npm run dev:desktop` 會啟動 Tauri 桌面視窗，需要該平台的 Tauri 系統依賴（macOS 為 Xcode Command Line Tools）。只跑 `npm run dev:server` 或 `npm run cli` 則不需要。

## `npm run dev` — full dev environment / 整套開發環境

- **用途**：一次啟動完整開發環境——依序建置當前 checkout 的 CLI、建置 desktop 前端資產，然後同時啟動 `speclink-server` 與 desktop 的 tauri dev。
- **前置條件**：上述全部前置需求。設定來源為 repo root 的 `.env`（對照 `.env.example`；沒有 `.env` 時全用預設值：sqlite、`.dev/store.db`、`127.0.0.1:8080`）。
- **預期可觀察結果**：終端先出現 `speclink dev: 建置當前 checkout 的 speclink-cli…` 與 `speclink dev: 建置當前前端資產…`；首次啟動時 server 印出一行含 `/setup?token=` 的連結（僅顯示一次，開啟它完成初始設定）；desktop 視窗開啟。Ctrl+C 同時收束 server 與 desktop，無殘留 process。狀態持久化於 gitignored 的 `.dev/`。

## `npm run dev:server` — server only / 只跑後端

- **用途**：只驗證 dev 設定並啟動 `speclink-server`，適合自架試用與純後端開發。不建置 CLI、不建置前端、不開 desktop 視窗。
- **前置條件**：Rust toolchain 與 `npm install`。零設定即可啟動（預設 sqlite）；設定不合法時（例如 `SPECLINK_STORE_DRIVER=postgres` 但缺 `SPECLINK_POSTGRES_URL`）以非零 exit code 拒絕啟動，錯誤訊息與 `npm run dev` 相同。
- **預期可觀察結果**：全新環境下終端印出含 `/setup?token=` 的連結；過程中沒有任何建置步驟、沒有 desktop 視窗。Ctrl+C 後無殘留 process；`.dev/` 持久化行為與 `npm run dev` 一致。

## `npm run dev:desktop` — desktop only / 只跑桌面

- **用途**：先建置 desktop 前端（vite 產出 `dist/`）再啟動 tauri dev，適合桌面開發與本地模式試用。不啟動 server、不要求任何 remote 設定。
- **前置條件**：前置需求全項（含 Tauri 系統依賴）。設定驗證與 `npm run dev` 共用——`.env` 不合法時（例如 postgres 缺 `SPECLINK_POSTGRES_URL`）同樣以非零 exit code 拒絕啟動。
- **預期可觀察結果**：終端先出現 `speclink dev: 建置當前前端資產…` 與 vite 建置輸出，完成後才開啟 desktop 視窗；視窗呈現的是本次原始碼建置出的畫面（tauri dev 載入靜態 `dist/`，因此每次啟動都重建，避免過期畫面）。前端建置失敗時以非零 exit code 結束、不開視窗。視窗以本地模式運作，可直接瀏覽本 repo 的 `openspec/` 看板，機器上沒有 server 也能用。

## `npm run dev:reset` — reset local dev state / 重置本機開發狀態

- **用途**：清空 `.dev/`（dev 用的 server 設定與資料庫），下次 `npm run dev` 回到全新 `/setup`。不建置任何東西、不碰 `.env` 與 `deploy/`。
- **前置條件**：無（對不存在的 `.dev/` 冪等成功）。
- **預期可觀察結果**：終端印出 `speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。` 後立即結束。注意：postgres 的資料在外部資料庫，不在 reset 範圍。

## `npm run cli -- <args>` — CLI from this checkout / checkout 內 CLI

- **用途**：固定執行同一 checkout 的 `target/debug/speclink`，讓「啟動環境」與「以同版 CLI 驗證」落在同一份原始碼上；絕不使用 PATH 中已安裝的 `speclink`。
- **前置條件**：Rust toolchain。binary 尚未建置時會先自動於 checkout root 執行 `cargo build -p speclink-cli` 再執行（建置失敗以非零 exit code 結束，不執行任何 CLI）。
- **預期可觀察結果**：

```bash
npm run cli -- --version
```

首次執行（或刪除 binary 後）會先在 stderr 看到 cargo 建置進度，接著輸出版本。需要純 machine-readable stdout 時加 `--silent`：

```bash
npm run --silent cli -- list --json
```

stdout 僅含 CLI 的 JSON payload（camelCase 欄位），建置進度不會混入。CLI 的 exit code、stdin/stdout/stderr 與工作目錄（`INIT_CWD`）皆透明轉送。

## Unsigned installer bypass / 下載安裝檔的未簽章放行

桌面安裝檔由 desktop-installer-and-updater 變更交付，**尚未出現在現有 GitHub Release**。交付後的產物形態依 desktop-release 規格為：macOS dmg（aarch64 與 x86_64 各一）、Windows NSIS 安裝器（x86_64）、Linux AppImage 與 deb（x86_64 與 aarch64），全部收錄於 `SHA256SUMS.txt`。在專案設定 OS 程式碼簽章金鑰之前，這些安裝檔為未簽章產物，首次開啟需手動放行：

### macOS

1. 開啟 dmg，將 **Speclink** 拖進「應用程式」。
2. 首次開啟會被阻擋（「無法打開，因為 Apple 無法檢查…」）。
3. 前往「系統設定 > 隱私權與安全性」，在「安全性」段落找到 Speclink 被封鎖的訊息，點「強制打開」（Open Anyway），再於確認框選「打開」。
4. 放行只需做這一次，之後照常開啟。

### Windows

1. 執行 NSIS 安裝器時，SmartScreen 顯示「Windows 已保護您的電腦」。
2. 點「其他資訊」（More info），再點「仍要執行」（Run anyway）即可繼續安裝。

### Linux

AppImage 與 deb 沒有對應的簽章阻擋機制：AppImage 加上執行權限（`chmod +x`）即可執行，deb 依發行版慣例安裝。
