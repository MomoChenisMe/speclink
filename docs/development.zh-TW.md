# 開發環境

**繁體中文** · [English](development.md)

這份文件給 clone 原始碼的開發者。它說明 repo 根目錄的五個一鍵入口、怎麼跑測試、repo 的結構，最後附上下載安裝檔時的未簽章放行步驟。

只想在 checkout 之外自架 server，不需要這份文件：用 `npx @speclink/server`，見 [Remote 入門](remote-getting-started.zh-TW.md)。

## <a id="prerequisites"></a>前置需求

- stable Rust 工具鏈（`cargo` 可用）。
- Node.js 21 以上與 npm。CI 用 Node 20 分開跑各項測試；`npm run test:all` 的寫法要 Node 21 以上（見[測試](#tests)）。
- 第一次 clone 後，在 repo 根目錄安裝前端依賴：

  ```bash
  npm install
  ```

- 會啟動 `speclink-server` 的入口（`npm run dev`、`npm run dev:server`）與 Rust 測試，需要先建一次 server 網頁後台：

  ```bash
  npm run build -w apps/server-web
  ```

  `speclink-server` 編譯時要求 `apps/server-web/dist` 資料夾存在，開發模式下也直接讀這個資料夾。改了 `apps/server-web` 的程式碼要重建，否則後台顯示的是舊畫面。

- 會開桌面視窗的入口（`npm run dev`、`npm run dev:desktop`）需要 Tauri 的系統依賴：macOS 是 Xcode Command Line Tools；Linux 要裝 GTK 與 WebKit 套件（清單見 `.github/workflows/ci.yml` 的 Install Tauri system dependencies 步驟）；其他情況見 [Tauri 的前置需求說明](https://v2.tauri.app/start/prerequisites/)。

## <a id="dev"></a>`npm run dev`：整套開發環境

- **用途**：一次啟動完整的開發環境。它先建置當前 checkout 的 `speclink-cli`，成功後同時啟動 `speclink-server` 與桌面 app（tauri dev）。桌面前端由 Vite dev server 提供，改前端會即時重載。
- **前置條件**：上面全部的前置需求。設定來自 repo 根目錄的 `.env`（可以複製 `.env.example` 修改）。沒有 `.env` 時全用預設值：sqlite、資料放在 `.dev/`、server 在 `127.0.0.1:8080`。`.env` 不合法時（例如 `SPECLINK_STORE_DRIVER=postgres` 卻沒設 `SPECLINK_POSTGRES_URL`），指令以非零結束，什麼都不啟動。
- **預期可觀察結果**：
  1. 終端先印出 `speclink dev: 建置當前 checkout 的 speclink-cli…`。建置失敗就以非零結束，不留下任何背景程序。
  2. 第一次啟動時，server 印出一行 `Speclink 首次啟動：開啟 http://localhost:8080/setup?token=… 完成初始設定（此連結 24 小時內有效，且僅顯示這一次）。`。打開這個連結完成初始設定。
  3. 桌面那一側先放好 sidecar（桌面 app 內附的 CLI，見 [`npm run dev:desktop`](#dev-desktop)），再開啟桌面 app 視窗。
  4. 按 Ctrl+C 會同時停止 server 與桌面 app，不留背景程序。任一邊先結束時，另一邊也會跟著停止。資料存在 `.dev/`（不進版本控制）。

先建 CLI 是刻意的：你接著用 [`npm run cli`](#cli) 核對時，CLI 與 server 來自同一份原始碼。

## <a id="dev-server"></a>`npm run dev:server`：只跑 server

- **用途**：只啟動 `speclink-server`，用來在這個 checkout 做後端開發。不建置 CLI，也不開桌面視窗。
- **前置條件**：Rust、`npm install`，以及建好的 `apps/server-web/dist`。`.env` 的檢查與 `npm run dev` 相同，不合法就以非零結束。
- **預期可觀察結果**：第一次執行時 cargo 會先編譯 server，要等一陣子。`.dev/` 是全新的時候，終端印出含 `/setup?token=` 的連結。沒有桌面視窗。按 Ctrl+C 後不留背景程序；`.dev/` 的資料與 `npm run dev` 共用。

## <a id="dev-desktop"></a>`npm run dev:desktop`：只跑桌面 app

- **用途**：只啟動桌面 app（tauri dev），用於桌面開發與試用本機模式。不啟動 server，也不需要任何遠端設定。
- **前置條件**：Rust、`npm install`、Tauri 系統依賴。`.env` 的檢查與 `npm run dev` 相同。
- **預期可觀察結果**：
  1. 先建置 `speclink-cli`，再把它放到桌面 app 需要的位置。這個內附在桌面 app 裡的 CLI 叫做 sidecar。終端會印出 `sidecar 佈署完成（debug）：…`，或內容沒變時印出 `sidecar 內容未變，跳過複製：…`。這一步失敗就以非零結束，不開視窗。
  2. Vite dev server 啟動後開啟桌面視窗。畫面來自目前的原始碼，改前端會就地重載。
  3. 視窗以本機模式運作，可以直接開這個 repo 的 `openspec/` 看板。機器上沒有 server 也能用。

## <a id="dev-reset"></a>`npm run dev:reset`：清掉本機開發狀態

- **用途**：刪掉 `.dev/`，也就是開發用的 server 設定與資料庫，讓下次 `npm run dev` 回到全新的 `/setup`。不建置任何東西，不碰 `.env` 與 `deploy/`。
- **前置條件**：無。`.dev/` 不存在也會成功。
- **預期可觀察結果**：終端印出 `speclink dev: .dev/ 已清空，下次 npm run dev 回到全新 /setup。` 後立即結束。postgres 的資料在外部資料庫，不會被清掉；要重來請自己刪掉並重建資料庫。

## <a id="cli"></a>`npm run cli -- <args>`：用這個 checkout 的 CLI

- **用途**：執行這個 checkout 的 `target/debug/speclink`，絕不使用 PATH 上的 `speclink`。這樣「啟動環境」與「用同版 CLI 核對」用的是同一份原始碼。
- **前置條件**：Rust 工具鏈。binary 還不存在時，wrapper 先在 checkout 根目錄執行 `cargo build -p speclink-cli` 再執行；建置失敗就以非零結束，不執行任何 CLI。binary 已經存在時**不會**重建，所以改了 Rust 程式碼後，先自己跑一次 `cargo build -p speclink-cli`，否則執行的是舊版。
- **預期可觀察結果**：

  ```bash
  npm run cli -- --version
  ```

  ```text
  > speclink-workspace@0.1.3 cli
  > node scripts/dev/cli.mjs --version

  speclink 0.8.0 (arm64, engine v1.41.0)
  ```

  版號與架構依你的 checkout 與機器而定。第一次執行（或刪掉 binary 之後）會先在 stderr 看到 cargo 的建置過程。

需要乾淨的 stdout（例如讓程式讀 JSON）時加 `--silent`，npm 自己的訊息就不會混進來：

```bash
npm run --silent cli -- list --json
```

在別的資料夾（例如測試用的 repo）使用這個 checkout 的 CLI：

```bash
npm --prefix <speclink-checkout> run cli -- list
```

CLI 會在你下指令的資料夾執行。exit code、stdin、stdout、stderr 都原樣轉送。

### 核對遠端環境

在 checkout 裡核對遠端環境時（例如 `auth status`、`list --json`），一律用 `npm run cli -- <args>`，不要用 PATH 上的 `speclink`。PATH 上那顆可能來自另一個 checkout，或是沒跟上的舊安裝版。用這兩行比對引擎版號：

```bash
speclink --version
npm run --silent cli -- --version
```

兩行的 `engine v…` 不一樣，就表示 PATH 上的 `speclink` 不是這份原始碼建出來的。

## <a id="tests"></a>測試

全新 clone 要先備好三樣建置產物，Rust 測試才編譯得過。順序和 CI 相同：

```bash
npm install
npm run build -w apps/desktop                              # 桌面 crate 編譯時要 apps/desktop/dist
npm run build -w apps/server-web                           # speclink-server 編譯時要 apps/server-web/dist
node scripts/desktop/desktop-sidecar.mjs --profile debug   # 桌面 crate 編譯時要 sidecar
npm run test:all
```

`npm run test:all` 依序跑 repo 腳本、三個前端 package、Rust workspace 與 Node SDK（安裝、建置、測試）。它自己會重建 `apps/server-web/dist`，但不建 `apps/desktop/dist`，也不放 sidecar，所以全新 clone 要先跑上面三行。之後只要沒刪掉這些產物，就不必重跑。

只跑其中一塊：

```bash
node --test "scripts/**/*.test.mjs"                     # repo 腳本（Node 21 以上）
node --test scripts/*.test.mjs scripts/*/*.test.mjs     # 同上，Node 20 用這行
npm test -w packages/ui                                 # 共用 UI
npm test -w apps/desktop                                # 桌面前端
npm test -w apps/server-web                             # server 網頁後台
cargo test --workspace                                  # Rust：引擎、CLI、Host、儲存後端、server、桌面 app
npm --prefix crates/adapters/speclink-node ci           # Node SDK：先安裝依賴
npm --prefix crates/adapters/speclink-node run build    #           再建置 .node
npm --prefix crates/adapters/speclink-node test         #           再跑測試
```

- `cargo test --workspace` 很久。只改了某個 crate 時，用 `cargo test -p <crate>`。
- PostgreSQL 儲存後端的測試在沒設 `SPECLINK_TEST_POSTGRES_URL` 時會自動跳過。要跑就把它指向一個可用的資料庫（CI 用 PostgreSQL 15）。
- Node SDK 不是 npm workspace 的成員，所以要用 `npm --prefix` 指定路徑；用 `-w` 找不到它。

## <a id="layout"></a>repo 結構

| 路徑 | 內容 |
| --- | --- |
| `apps/desktop/` | 桌面 app（Tauri）：`src/` 是前端，`core/` 與 `src-tauri/` 是兩個 Rust crate |
| `apps/server-web/` | server 的網頁後台（React），建置後內嵌進 `speclink-server` |
| `crates/engine/` | 引擎核心：`speclink-core`（動詞、工作流、技能產生）與 `speclink-fs`（讀寫本機 `openspec/`） |
| `crates/protocol/` | 遠端協定：`speclink-protocol`（client 與 server 共用的資料型別）與 `speclink-remote`（CLI 與桌面 app 用的遠端 client 與登入） |
| `crates/store/` | server 的儲存後端（Store driver）：共用介面 `speclink-store`，以及 sqlite、檔案（serverfs）、postgres 三種實作 |
| `crates/host/` | `speclink-host`（引擎與 CLI、桌面 app、server 之間的共用層）與 `speclink-server`（官方參考 server） |
| `crates/adapters/` | 對外入口：`speclink-cli`（`speclink` 指令）與 `speclink-node`（Node SDK `@speclink/engine`） |
| `packages/ui/` | 桌面 app 與網頁後台共用的 UI 元件 |
| `packages/cli-npm/`、`packages/server-npm/` | npm 套件 `@speclink/cli` 與 `@speclink/server` 的外殼；各平台的 binary 另外發布 |
| `integrations/claude-code/` | Claude Code 的 mod（`speclink-skills`：技能列與側邊面板） |
| `scripts/` | 開發入口、文件守門、桌面建置、npm 打包與發版用的腳本 |
| `deploy/` | Docker Compose 部署檔（sqlite 與 postgres 各一份） |
| `docs/` | 使用者文件 |
| `openspec/` | Speclink 自己的規格、變更與討論 |

## <a id="unsigned"></a>下載安裝檔的未簽章放行

桌面安裝檔都在 [Releases](https://github.com/MomoChenisMe/speclink/releases/latest)：

| 平台 | 檔名 | 要不要放行 |
| --- | --- | --- |
| macOS（Apple Silicon 與 Intel 同一檔） | `Speclink_<版本>_universal.dmg` | 不用：已簽章並經 Apple 公證 |
| Windows（x64） | `Speclink_<版本>_x64-setup.exe` | 要：安裝器沒有程式碼簽章 |
| Linux（x86_64） | `Speclink_<版本>_amd64.AppImage` | 不用：加上執行權限即可 |
| Linux（aarch64） | `Speclink_<版本>_aarch64.AppImage` | 不用：加上執行權限即可 |

### macOS

打開 dmg，把 Speclink 拖進「應用程式」就好，不需要放行。只有你自己在本機建置、沒有簽章的 app 才會被擋：

1. 先打開一次 app，看到無法打開的提示後關掉。
2. 到「系統設定 > 隱私權與安全性」，在安全性區塊找到 Speclink，按「強制打開」。

### Windows

1. 執行 `Speclink_<版本>_x64-setup.exe`，SmartScreen 顯示「Windows 已保護您的電腦」。
2. 按「其他資訊」，再按「仍要執行」，繼續安裝。

### Linux

```bash
chmod +x Speclink_<版本>_amd64.AppImage
./Speclink_<版本>_amd64.AppImage
```

aarch64 機器換成 `Speclink_<版本>_aarch64.AppImage`。AppImage 執行時需要 FUSE；出現 FUSE 相關錯誤時，先安裝 `libfuse2`（Ubuntu 24.04 起叫 `libfuse2t64`）。
