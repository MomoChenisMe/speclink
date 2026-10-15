<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>一套 SDD Engine，支援 Local Repo 與 Remote Store</b>
</p>

<p align="center">
  <a href="https://github.com/MomoChenisMe/speclink/actions/workflows/ci.yml"><img src="https://github.com/MomoChenisMe/speclink/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/MomoChenisMe/speclink/releases/latest"><img src="https://img.shields.io/github/v/release/MomoChenisMe/speclink?label=release" alt="Release" /></a>
  <a href="https://www.npmjs.com/package/@speclink/cli"><img src="https://img.shields.io/npm/v/@speclink/cli?label=npm" alt="npm" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT" /></a>
</p>

<p align="center">
  <b>繁體中文</b> · <a href="README.en.md">English</a>
</p>

Speclink 是用 Rust 寫的 Spec-Driven Development（SDD，規格驅動開發）引擎與工具平台。PM、PO、RD 與 AI Agent 在這裡用同一套語意協作：change（變更）、artifact（產物）、task（任務）、verify（驗證）與 archive（封存）。

Speclink 有兩條部署路徑：

- **Local Repo**：規格放在 repo 的 `openspec/`，用 Git 協作，不需要 server。
- **Remote Store**：規格放在團隊共用的 Store，由 Host 統一處理認證、版本、交易、事件與流程判定。Host 與 Protocol 都是公開契約，server 可以用官方那一份，也可以自己寫。

Local CLI 設計之初以 [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) 附的 CLI 為行為參考。golden 與 CLI 整合測試守住人眼輸出、`--json` 格式與核心工作流。Speclink 在這個基礎上加入討論、桌面 app、Store 抽象層、Node SDK 與遠端平台。

![Speclink 桌面 app 的變更看板](docs/assets/screenshots/desktop-board.png)

## <a id="features"></a>特色

- **純文字，相容 OpenSpec**：Local 模式沿用 OpenSpec 的目錄結構：`specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`。內容全是 Markdown 與 YAML，不裝 Speclink 也讀得懂、改得動，每次變動都看得到 Git diff。Speclink 只多放兩樣東西：`discussions/`（討論記錄）與每個變更的 `.openspec.yaml`（生命週期資料）。這一點只適用 Local 模式；遠端模式的正式規格在 Store，本機只有唯讀投影。
- **給 AI Agent 的技能**：`speclink init` 為 Claude Code、Codex 與 GitHub Copilot 產生技能檔。從討論、提案、實作、品質關卡到封存，每一站都有對應的 `/speclink-*` 指令（Codex 寫成 `$speclink-*`）。
- **桌面看板**：每個變更是一張卡片。你看得到它在哪一站、任務做到哪裡、規格改了什麼。
- **品質關卡**：`review` 看程式碼寫得好不好，`verify` 看交付是否符合規格。兩道都是選用的。
- **執行順序**：`speclink plan` 依宣告的依賴把變更分成一波一波，並指出下一個可以開工的變更。
- **團隊共用**：接上 server 之後，CLI、桌面 app 與 Agent 讀寫同一份正式規格。

## <a id="status"></a>目前狀態

- **可用**：Local Repo CLI、Local 桌面 app、Agent 技能（Claude、Codex 與 GitHub Copilot）、品質關卡、執行順序、手冊與溯源、Node SDK（`@speclink/engine`）、三種 TeamStore（SQLite、Server FS、PostgreSQL）、單節點 server 與後台、Remote CLI、安裝通路、server 營運（部署、備份還原）。
- **部分可用**：桌面 app 的遠端 workspace、Claude Code 外掛。
- **規劃中**：Agent 工具套件與 MCP、SSO、runtime plugin、多節點部署。

每一項的證據、限制與查核日期見[專案能力狀態](docs/product-status.zh-TW.md)。還沒做的方向見[專案路線圖](docs/roadmap.zh-TW.md)。

## <a id="install"></a>安裝

桌面 app 與 CLI 是同一套引擎的兩種用法，擇一即可：

- 想看看板、規格與討論：裝桌面 app。安裝檔內含同版 CLI。
- 不需要圖形介面，或要用在腳本與 CI：只裝 CLI。功能一樣完整。

Server 只有在團隊要共用同一份正式規格時才需要。一個人在自己的 repo 裡用，不必裝 server。

### 桌面 app

到 [Releases](https://github.com/MomoChenisMe/speclink/releases/latest) 下載你的平台的安裝檔：

| 平台 | 安裝檔 |
| --- | --- |
| macOS | `Speclink_<版本>_universal.dmg`（Apple Silicon 與 Intel 同一檔） |
| Windows | `Speclink_<版本>_x64-setup.exe` |
| Linux 桌面 | `Speclink_<版本>_amd64.AppImage`（x86_64）或 `Speclink_<版本>_aarch64.AppImage`（arm64），免安裝 |

- Windows 安裝檔沒有程式碼簽章。第一次執行時 SmartScreen 會跳出警告，點「其他資訊」→「仍要執行」。
- 0.5.0 起不再提供 `.deb`。用 `.deb` 裝過的人，先執行 `sudo apt remove speclink`，再改用 AppImage。
- 沒有圖形介面的 Linux（伺服器、WSL、CI）不裝桌面 app，改用下面的 CLI。

### CLI

三種方式裝到的是同一份執行檔（來源是 npm 上的 `@speclink/cli`），擇一即可：

```bash
# 有 Node.js（任何平台）
npm i -g @speclink/cli

# 沒有 Node.js 的 macOS／Linux（伺服器、WSL、CI）
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# Homebrew（macOS／Linux）
brew install MomoChenisMe/tap/speclink
```

Windows 沒有安裝腳本：有 Node.js 就用 npm，沒有就裝桌面 app（安裝器會一併設好 CLI 的 PATH）。

安裝腳本會偵測平台、核對 npm 套件的 sha512，再把 `speclink` 放進 `~/.local/bin`。可用三個環境變數調整：

| 變數 | 用途 |
| --- | --- |
| `SPECLINK_INSTALL_DIR` | 改安裝位置 |
| `SPECLINK_INSTALL_VERSION` | 釘選版本（`0.5.0` 或 `v0.5.0` 都可以；0.5.0 之前的版本不在 npm 上） |
| `SPECLINK_INSTALL_REGISTRY` | 換 npm registry |

<details>
<summary><b>先裝了 CLI 又要裝桌面 app？先看這裡</b></summary>

桌面 app 和安裝腳本都用 `~/.local/bin/speclink` 這個位置，所以桌面 app 可能換掉你的 CLI：

| 平台 | 桌面 app 對 `~/.local/bin/speclink` 做的事 |
| --- | --- |
| macOS | 每次啟動都檢查一次；沒裝或版本和 app 不同時，刪掉原檔，換成指向內建 CLI 的 symlink |
| Linux AppImage | 只在版本不符時覆蓋 |
| Windows | 不動這個位置；PATH 由安裝器管理 |

所以 `SPECLINK_INSTALL_VERSION` 釘選的版本也會跟著失效：只要版本和 app 不同，就會被換掉。要保留自己那份 CLI，安裝時用 `SPECLINK_INSTALL_DIR` 指到別的目錄，再把那個目錄排在 PATH 中 `~/.local/bin` 的前面。

</details>

### Server（選用）

`speclink-server` 是官方的**參考實作**，讓你開箱即用，或拿來試遠端功能。三種跑法擇一：

| 跑法 | 指令 |
| --- | --- |
| npx（有 Node.js 就能跑） | `npx @speclink/server` |
| Docker | `docker run -d -p 8080:8080 -v speclink-data:/data ghcr.io/momochenisme/speclink-server:latest` |
| Docker Compose | `cd deploy && docker compose up -d` |

第一次啟動時，server 會印出一次性的 `/setup` 連結；之後重啟不會再印。預設用 SQLite，資料放在 `./speclink-data`（容器內是 `/data`）。環境變數、PostgreSQL 與升級回退見 [Server 部署](docs/server-deployment.zh-TW.md)。

遠端模式不綁這一份 server。遠端模式由兩份公開契約定義：`openspec/specs/` 底下的 `host-runtime` 與 `client-protocol`。你可以用 Speclink 引擎自己寫 server，接上自家的認證、資料庫與權限模型，CLI 與桌面 app 一樣接得上。載入引擎的方式見 [Node SDK](docs/sdk-node.zh-TW.md)。

### Claude Code 外掛（選用）

`speclink-skills` 在 Claude Code 的輸入框上方加一排可以點的 speclink 技能按鈕，另有一個側邊面板列出變更、討論與未結的品質工單。在 Claude Code 裡輸入：

```text
/plugin install speclink-skills --marketplace MomoChenisMe/speclink
```

需求、設定與已知限制見 [speclink-skills](integrations/claude-code/speclink-skills/README.md)。

## <a id="quick-start"></a>快速開始

在要導入 Speclink 的 repo 裡執行：

```bash
speclink init --tools claude,codex
speclink list
```

`--tools` 列出你用的 Agent 工具：`claude`、`codex`、`copilot`，用逗號任意組合。

接著請 Agent 開一個變更：在 Claude Code 或 GitHub Copilot 輸入 `/speclink-propose <變更名稱>`，在 Codex 輸入 `$speclink-propose <變更名稱>`。Agent 會建立這個變更需要的文件。

完整的第一輪（提案、實作、檢查、封存）見 [Local Repo 入門](docs/getting-started.zh-TW.md)。要接遠端 server，見 [Remote 入門](docs/remote-getting-started.zh-TW.md)。

## <a id="workflow"></a>工作流程

```text
baseline? → discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive
                                            ↑
                                    閒置後續作：先 drift

worktree：apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive

工具：validate / analyze / audit / commit / config / manual / trace / plan
```

帶 `?` 的站是選用的。從哪一站開始，看你手上的情況：

| 情況 | 入口 |
| --- | --- |
| 需求已經清楚 | `propose` |
| 需求還要收斂 | `discuss`（你帶題目）或 `improve`（請模型幫你找題目） |
| 既有程式還沒有規格 | `baseline` |
| 實作途中需求改變 | `ingest` |
| 變更閒置一陣子才繼續 | 先 `drift` |
| 想同時推多個互不衝突的變更 | worktree 流程 |

封存前有兩道選用的品質關卡：`review` 看工藝，`verify` 看是否符合規格。依風險決定要跑哪幾道；低風險的變更兩道都跳過也可以。

每一站的用途、技能、完成判準與下一站見[完整 SDD 工作流](docs/workflow.zh-TW.md)。

## <a id="docs"></a>文件

**一個人用，先讀這三份**

| 文件 | 內容 |
| --- | --- |
| [Local Repo 入門](docs/getting-started.zh-TW.md) | 從安裝到第一次封存 |
| [完整 SDD 工作流](docs/workflow.zh-TW.md) | 每一站的用途、技能、完成判準與下一站 |
| [設定說明](docs/configuration.zh-TW.md) | `.speclink.yaml`、`openspec/config.yaml` 與遠端設定 |

**團隊共用一份正式規格**

| 文件 | 內容 |
| --- | --- |
| [Remote 入門](docs/remote-getting-started.zh-TW.md) | 從啟動 server 到 Desktop 與 CLI 連上 |
| [Server 部署](docs/server-deployment.zh-TW.md) | npx、Docker、Compose、升級與回退 |
| [Server 儲存後端](docs/server-store-drivers.zh-TW.md) | SQLite、Server FS、PostgreSQL 怎麼選 |
| [Server 備份與還原](docs/server-backup.zh-TW.md) | `backup`、`verify-backup`、`restore` |

**把 Speclink 接進自己的程式**

| 文件 | 內容 |
| --- | --- |
| [Node SDK](docs/sdk-node.zh-TW.md) | `@speclink/engine` 的載入、Store 介面與 `dispatch` |
| [動詞與旗標契約](docs/verb-contract.zh-TW.md) | 動詞在本地與遠端的歸屬、輸出格式與 HTTP 端點 |

**專案現況與方向**

| 文件 | 內容 |
| --- | --- |
| [專案能力狀態](docs/product-status.zh-TW.md) | 哪些能用、哪些部分可用，附證據與限制 |
| [專案路線圖](docs/roadmap.zh-TW.md) | 之後要做的方向 |
| [更新日誌](CHANGELOG.md) | 每個版本改了什麼 |
| [品牌資產](docs/assets/brand/README.md) | Logo、配色與使用方式 |

`openspec/` 是 Speclink 自己的規格。`openspec/changes/archive/` 與 `openspec/discussions/archive/` 是歷史記錄，不是操作文件。

## <a id="contributing"></a>參與開發

從原始碼建置 CLI（需要 stable Rust toolchain）：

```bash
cargo install --path crates/adapters/speclink-cli
speclink --version
```

一鍵開發環境（`npm run dev` 等五個入口）、完整的測試指令與 repo 結構，見[開發環境](docs/development.zh-TW.md)。

## <a id="license"></a>授權

[MIT](LICENSE)
