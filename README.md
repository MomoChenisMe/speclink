<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>一套 SDD Engine，支援 Local Repo 與 Remote Store</b>
</p>

<p align="center">
  <b>繁體中文</b> · <a href="README.en.md">English</a>
</p>

Speclink 是以 Rust 實作的 Spec-Driven Development（SDD）引擎與工具平台。PM、PO、RD 與 AI Agent 在這裡
使用同一套語意：change、artifact、task、verify、archive。它同時保留兩種部署路徑：

- **Local Repo**：規格位於 repo 的 `openspec/`，由 Git 協作，不需要 server。
- **Remote Store**：規格位於共享 Store，由 Speclink Host 統一處理認證、revision、交易、事件與流程裁決。Host 與 Protocol 都是公開契約，所以 server 端可以用官方那一份，也可以自己寫。

**Local 模式**的產物刻意貼合 OpenSpec 的目錄結構：`specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/`
與 `config.yaml`。內容全是純 Markdown 與 YAML，沒有資料庫，也沒有專屬格式。不裝 Speclink 一樣讀得懂、
改得動，每次規格變動都看得出 Git diff。Speclink 只在這個結構上多放兩樣東西：`discussions/`（討論記錄），
以及每個變更的 `.openspec.yaml`（生命週期 metadata）。

Local CLI 設計之初以 [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) 所附 CLI 為行為參考。
golden 與 CLI 整合測試守住人眼輸出、`--json` shape 與核心工作流。Speclink 在這個基礎上加入 discussion、
Desktop、Store abstraction、Node SDK 與 Remote Platform。

規格不是寫完就擱著的文件——桌面 app 把每個變更放上看板，你看得到它站在哪一站、任務推到哪裡、規格改了什麼：

![Speclink 桌面 app 的變更看板與變更詳情面板](docs/assets/screenshots/desktop-board.png)

## Current capabilities / 目前能力

- **可用**：Local Repo CLI、Local Desktop、生成的 Agent 技能（Claude 與 Codex 全站別）、Command Runtime／Host／Protocol、SQLite／Server FS／PostgreSQL TeamStore、單節點 Server 與 Admin／Auth、Remote CLI 與 Context Projection、Server 營運（部署、備份還原）、桌面與 CLI 的安裝通路。
- **部分可用**：Node SDK（綁定可用，但尚未發布至 npm）、Desktop Remote Workspace、遠端的 task evidence。
- **規劃中**：MCP／Copilot in-process tools、SSO、runtime plugins 與 cluster mode。
- **已棄用**：legacy remote REST v1 prototype；新工作使用目前 Client Protocol／Host 路徑。

逐項證據、限制與最後查核日期不在 README 重複維護，一律以[專案能力狀態](docs/product-status.zh-TW.md)為準；之後會往哪走見[專案路線圖](docs/roadmap.zh-TW.md)。

## SDD workflow / SDD 工作流

```text
onboard? → discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive
                                            ↑
                                    閒置後續作：先 drift

worktree：apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive

工具：validate / analyze / audit / commit / config
```

從哪一站進來，看你手上的情況：

- 需求已經明確 → 直接 `propose`
- 需求還要收斂 → `discuss`（你帶題目）或 `improve`（要模型幫你找題目）
- 實作途中需求改變 → `ingest`
- 變更閒置一陣子才續作 → 先跑 `drift`

封存前有兩道可選的品質關卡：`review` 看工藝，`verify` 看合規。依風險自由組合。低風險變更兩道都跳過也是正當選擇。

手上有多個互不衝突的變更要一起推時走 worktree 流程：每個變更在自己的 git worktree 裡實作、互不干擾，完成後 `worktree-merge` 併回主分支再封存。

每一站的用途、對應的 `/speclink-*` 技能、完成判準與下一站，以及討論結論的分流與恢復路徑，見[完整 SDD 工作流](docs/workflow.zh-TW.md)。

## Install / 安裝

桌面 app 與 CLI 是**同一套引擎的兩種用法，擇一即可**。想看看板、規格與討論，就裝桌面 app。不需要圖形介面，或要把 Speclink 放進腳本與 CI，就只裝 CLI——功能不打折。

Server 是第三個東西，**只有團隊要共用同一份規格正典時才需要**。一個人在自己 repo 裡用，完全不必碰它。

**桌面 app**——到 [Releases](https://github.com/MomoChenisMe/speclink/releases/latest) 下載對應平台的安裝檔：

| 平台 | 安裝檔 |
| --- | --- |
| macOS | `Speclink_<版本>_aarch64.dmg`（Apple Silicon）、`Speclink_<版本>_x64.dmg`（Intel） |
| Windows | `Speclink_<版本>_x64-setup.exe` |
| Linux | `.AppImage`（免安裝）或 `.deb`，各有 x86_64 與 aarch64 |

桌面安裝檔內含同版 CLI，可於 app 設定中一鍵安裝到 PATH。

**已經裝過 CLI 的人請注意：先裝 CLI、後裝桌面 app，你的 `speclink` 會被換成桌面 app 那一版。**

原因是兩者都用 `~/.local/bin/speclink` 這個位置。逐平台的行為不同：

| 平台 | 桌面 app 對 `~/.local/bin/speclink` 做的事 |
| --- | --- |
| macOS | 每次啟動都刪掉原檔，換成指向內建 CLI 的 symlink |
| Linux AppImage | 只在版本不符時覆蓋 |
| Windows | 不動這個位置；PATH 由安裝器管理 |
| Linux `.deb` | 不動這個位置；套件管理器佈署到 `/usr/bin` |

`SPECLINK_INSTALL_VERSION` 釘選的版本也會一起失效。要保留自己那份 CLI，安裝時用 `SPECLINK_INSTALL_DIR`
指到別的目錄，再把該目錄排在 PATH 中 `~/.local/bin` 的前面。

**CLI**——擇一：

```bash
# 安裝腳本（macOS／Linux）
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# 安裝腳本（Windows PowerShell）
irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex

# Homebrew（macOS／Linux）
brew install MomoChenisMe/tap/speclink
```

安裝腳本會偵測平台、核對 SHA-256，再把 `speclink` 放進 `~/.local/bin`（Windows 為使用者層級目錄）。`SPECLINK_INSTALL_DIR` 可改安裝位置，`SPECLINK_INSTALL_VERSION` 可釘選版本。

Windows 的安裝檔目前未經程式碼簽章，首次執行時 SmartScreen 會出現警告——點「其他資訊」→「仍要執行」即可。

**Server**（只有要團隊共用時才需要）——`speclink-server` 是官方的**參考實作**，給你開箱即用、或拿來試遠端功能。三種形態擇一，都會在啟動後印出一次性的 `/setup` 連結：

| 形態 | 指令 |
| --- | --- |
| npx（有 Node 就能跑） | `npx @speclink/server` |
| Docker | `docker run -d -p 8080:8080 -v speclink-data:/data ghcr.io/momochenisme/speclink-server:latest` |
| Compose | `cd deploy && docker compose up -d` |

預設使用 SQLite、資料落在 `./speclink-data`（容器內為 `/data`）。環境變數、PostgreSQL profile 與升級回退見[Server 部署](docs/server-deployment.zh-TW.md)。

**遠端模式不綁這一份 server。** 規格正典放在哪、由誰守，是 Host 與 Protocol 兩份公開契約定義的；官方 server 只是照這兩份契約做出來的一個實作。要接自家的認證、資料庫或權限模型，就拿 Speclink 引擎自己寫一個 server 端，CLI 與桌面 app 照樣接得上。契約見 `openspec/specs/` 的 `client-protocol` 與 `host-runtime`，載入引擎的方式見 [Node SDK](docs/sdk-node.zh-TW.md)。

## Local Repo quick start / Local Repo 快速開始

在要導入 Speclink 的 repo：

```bash
speclink init --tools claude,codex
speclink list
```

接著在 Claude 呼叫 `/speclink-propose <change>`，或在 Codex 呼叫 `$speclink-propose <change>`；Agent 會依 schema DAG
建立必要 artifacts。可複製的第一輪與直接 CLI 對照見[Local Repo 入門教學](docs/getting-started.zh-TW.md)。

## Deployment paths / 部署路徑

- **Local Repo**：Embedded Rust Runtime → FsStore → `openspec/` → Git；適合單一 repo、本機與離線協作。
- **Remote Store**：CLI／Desktop／其他 Client → Speclink Host → 同一 Rust Runtime → TeamStore；適合共享規格正典、集中認證、revision、交易與事件。中間那個 Host 可以是官方的 `speclink-server`，也可以是你自己照 Protocol 做的實作。

Remote Store 不會同步成第二份可寫的本地真相。有 checkout 的 Agent 只讀 `.speclink/context/`，遠端寫入仍走
Host command。這些邊界的正典是 `openspec/specs/` 底下的規格，例如 `host-runtime`、`client-protocol`、
`teamstore-contract` 與 `context-projection`。從 setup 到登入的完整操作見
[Remote 入門教學](docs/remote-getting-started.zh-TW.md)。

## Documentation map / 文件地圖

**一個人用，先讀這三份就夠**

| 文件 | 用途 |
| --- | --- |
| [Local Repo 入門教學](docs/getting-started.zh-TW.md) | 目前可複製的第一輪 Local Repo 流程 |
| [完整 SDD 工作流](docs/workflow.zh-TW.md) | 每一站的用途、對應技能、完成判準與下一站 |
| [專案能力狀態](docs/product-status.zh-TW.md) | 可用／部分可用／規劃中／已棄用，附證據與限制 |

**要團隊共用一份規格正典才需要**

| 文件 | 用途 |
| --- | --- |
| [Remote Server、Desktop 與 CLI 入門教學](docs/remote-getting-started.zh-TW.md) | 從 `/setup`、membership、登入到 Desktop／CLI 與失聯恢復的完整流程 |
| [Server 部署](docs/server-deployment.zh-TW.md) | npx／Docker／Compose 與升級操作 |
| [Server Store drivers](docs/server-store-drivers.zh-TW.md) | SQLite／Server FS／PostgreSQL 選型與前提 |
| [Server 備份與還原](docs/server-backup.zh-TW.md) | backup／verify-backup／restore |

**遇到問題或想調整時再查**

| 文件 | 用途 |
| --- | --- |
| [設定說明](docs/configuration.zh-TW.md) | Local／Remote 設定歸屬與目前欄位 |
| [開發環境入口](docs/development.zh-TW.md) | 一鍵開發環境、checkout 內 CLI 與測試指令 |
| [專案路線圖](docs/roadmap.zh-TW.md) | 之後會往哪走：SDK、以引擎自建客戶端、遠端協作、Agent 工具整合、系統整合 |
| [品牌資產](docs/assets/brand/README.md) | Logo、配色與使用方式 |

**要把 Speclink 接進自己的程式，或自建客戶端才需要**——只用桌面 app 或 CLI 的話這兩份可以完全跳過。

| 文件 | 用途 |
| --- | --- |
| [Node SDK](docs/sdk-node.zh-TW.md) | `@speclink/engine` 的載入方式、Store bridge 與 dispatch surface |
| [動詞與旗標契約](docs/verb-contract.zh-TW.md) | 動詞的模式歸屬、兩模式輸出同形與端點的 payload／錯誤形狀 |

`openspec/changes/archive/` 與 `openspec/discussions/archive/` 是歷史稽核資料，不是目前操作手冊。

## Development / 開發

從原始碼建置 CLI（需要 stable Rust toolchain）：

```bash
cargo install --path crates/speclink-cli
speclink --version
```

[開發環境入口](docs/development.zh-TW.md)收了三類東西：一鍵開發環境的四個入口（整套 `npm run dev`、只跑 server、只跑 desktop、checkout 內 CLI）、完整測試指令，以及下載安裝檔的未簽章放行步驟。

## License / 授權

[MIT](LICENSE)
