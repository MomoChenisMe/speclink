<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>一套 SDD Engine，支援 Local Repo 與 Remote Store</b>
</p>

<p align="center">
  <b>繁體中文</b> · <a href="README.en.md">English</a>
</p>

Speclink 是以 Rust 實作的 Spec-Driven Development（SDD）引擎與工具平台。它讓 PM、PO、RD 與 AI Agent
使用同一套 change、artifact、task、verify 與 archive 語意，同時保留兩種部署路徑：

- **Local Repo**：規格位於 repo 的 `openspec/`，由 Git 協作，不需要 server。
- **Remote Store**：規格位於共享 Store，由 Speclink Host 統一處理認證、revision、交易、事件與流程裁決。

Local CLI 的設計以 [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) 所附 CLI 為行為參考與相容基準，並以
parity/golden tests 保護人眼輸出、`--json` shape 與核心工作流；Speclink 在此基礎上加入 discussion、Desktop、
Store abstraction、Node SDK 與後續 Remote Platform 等延伸。

> **目前狀態：**Local Repo、CLI、Local Desktop 與 Node N-API SDK 已有可運作實作。新的 TeamStore contract、
> `speclink-server`、Server Admin UI 與 Desktop Remote Workspace 仍依[平台架構藍圖](docs/platform-architecture.zh-TW.md)
> 分階段建置。舊 remote REST v1 不再是目標架構。

目前程式碼與目標架構的差距、重構優先順序及各 Phase 驗收 gate，見
[現況對齊與重構路線圖](docs/implementation-refactor-roadmap.zh-TW.md)。

## 核心架構

Speclink 只有一份 Rust 流程語意實作。CLI、Desktop、Server、Node SDK、MCP 與 Copilot Tools 都必須呼叫同一個
Command Runtime，不得各自重寫 lifecycle 或 archive 規則。

```text
Local Repo
  Agent / CLI / Desktop
    -> Embedded Rust Runtime
    -> FsStore
    -> repo/openspec/
    -> Git

Remote Store
  Desktop / CLI / MCP / Web UI / Copilot Tool
    -> Speclink Host
    -> Same Rust Runtime
    -> TeamStore（SQLite / Server FS / PostgreSQL / Custom）
```

遠端正典不會雙向同步成另一份可寫本地真相。需要讀檔、搜尋與 grep 的 Agent 透過唯讀
`.speclink/context/` snapshot 取得規格；所有遠端寫入仍走 Host command。

## 實作狀態

| 元件 | 現況 | 目標 |
|---|---|---|
| `speclink-core` | Rust SDD engine 與既有流程模組 | Typed Command Runtime、domain events、fail-closed policy |
| `speclink-fs` | 本地 `openspec/` Store | Local mode 保持無 server；Server FS 另加 journal/recovery |
| `speclink-cli` | Local Repo CLI 與現有 remote client 基礎 | 共用新的 Command/Query/Context Protocol |
| `speclink-desktop` | Local Repo 看板、規格、封存與設定 UI | Local/Remote `WorkspaceSession`、離線與 CAS conflict UX |
| `@speclink/engine` | Rust 經 N-API 的 Node SDK 與 Store bridge | 完整 typed commands、Host/Tool 整合與跨平台預編譯 binary |
| `speclink-server` | 尚未交付 | Rust single-node server、SQLite 預設、Admin UI、SSE、backup/restore |

## SDD 工作流

```text
discuss? -> propose -> apply <-> ingest -> verify? -> archive
```

PM、PO、RD 是流程中的 Human roles；Claude Code、Codex App/CLI、GitHub Copilot、Cursor 等是執行模型與 tools 的
Agent Host/Application。Agent Host 載入 Speclink Skill，再透過 CLI、MCP 或 In-process Tool 呼叫 Speclink Host。
Skill 是 workflow knowledge，不是 Embedded Speclink Host；本地/遠端都使用相同分層。

- `discuss` 是需求需要釐清時的選用步驟。
- `propose` 建立 proposal、design、tasks 與 delta specs。
- `apply` 依 tasks 實作；需求改變時以 `ingest` 更新 change。
- `verify` 在有 code checkout 的 RD/Agent 環境執行。
- `archive` 將 delta 合併至 canonical specs 並封存 change。

## Local Repo 快速開始

需要 stable Rust toolchain：

```bash
git clone <this-repo>
cd speclink
cargo build --release
```

在要導入 Speclink 的 repo：

```bash
speclink init
speclink demo
speclink list
speclink status --change <change-name>
speclink validate <change-name> --strict
```

`speclink init` 會建立：

```text
repo/
├── openspec/
│   ├── config.yaml
│   ├── LANGUAGE.md
│   ├── specs/
│   ├── changes/
│   └── discussions/
├── .speclink.yaml
└── .speclink/          # 本機工作資料，gitignored
```

它也會依工具設定產生 Claude Code 或 Codex skills。完整本地流程見[入門教學](docs/getting-started.zh-TW.md)。

## 主要 CLI

| 指令 | 用途 |
|---|---|
| `speclink init` / `update` | 初始化或同步工具 skills |
| `speclink new change` / `new artifact` | 建立 change 與 artifacts |
| `speclink list` / `show` / `status` | 查詢 changes、specs 與 artifact DAG |
| `speclink instructions` | 取得 artifact/apply instructions |
| `speclink task done` | 完成 task 並記錄 touched files |
| `speclink validate` / `analyze` / `drift` | 結構、品質與 drift 分析 |
| `speclink discuss ...` | 建立、收斂、連結與封存討論 |
| `speclink archive` / `discard` | 封存或廢棄 change |

使用 `speclink <command> --help` 查看完整參數。多數查詢支援 `--json`，既有 CLI parity tests 保護輸出相容性。

## Speclink Desktop

目前 Desktop 是 Local Repo 的官方開箱 UI，提供 change 看板、canonical specs、封存瀏覽、詳情抽屜、tasks 與設定頁。

Desktop 的部分本地功能與 UI/UX 參考 [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app)，包含以視覺介面
追蹤 changes/specs/tasks、專案切換、任務進度與 archive 瀏覽等產品方向。Speclink Desktop 不是 Spectra App fork；
目前程式以 Tauri/React 獨立實作，並延伸自己的 discussion lifecycle、DataSource contract 與後續 Remote Workspace。

```bash
npm install
npm --workspace @speclink/desktop run tauri -- dev
```

目標 Desktop 將同時支援：

- 本地資料夾 workspace。
- 沒有 code checkout 的 PM/PO remote spec-only workspace。
- 綁定本機 checkout 的 RD remote workspace。
- Server 登入、Project/Repo 選擇、OS Keychain、SSE/ETag、離線唯讀與 revision conflict。

Desktop 是官方預設 Presentation UI，但不是唯一 UI；第三方 Desktop、Web UI 與同系統 Agent UI 可實作相同
DataSource/Workspace contract。

## Node SDK 與 Agent Tools

`@speclink/engine` 以 N-API 載入同一份 Rust Engine，供 Node server 或 Agent host 內嵌。現有安裝、Store bridge 與
`dispatch` surface 見 [Node SDK 文件](docs/sdk-node.zh-TW.md)。

目標同系統 Agent 路徑：

```text
Copilot SDK Agent
  -> @speclink/copilot-tools
  -> @speclink/host
  -> @speclink/engine（N-API / Rust）
  -> Node TeamStore Adapter
```

Tool 直接呼叫同 process Host，不需要繞 CLI、MCP 或 HTTP，也不能繞過 Host 直接寫 Store。

官方 `@speclink/store-sqlite`、`@speclink/store-fs`、`@speclink/store-postgres` 若發布，都是相同 Rust driver
crate 的可選 N-API facade，不是 TypeScript 重寫。官方 `speclink-server` 直接連結 Rust crates；Node 套件只供
Node Host/in-process Agent 使用。自訂 JavaScript Store 仍可走 async bridge，但必須通過 TeamStore conformance suite。

## Remote Platform 目標

官方 `speclink-server` 將提供：

- 單一 Rust binary 與 Docker image。
- SQLite 預設 Store，並內建 Server FS、PostgreSQL driver。
- First-run `/setup` 與 `/admin` Server Admin UI。
- 一般使用者 `/account`、自助 PAT、invite 與 Desktop browser/device authorization。
- PAT/角色、Project/Repo registry、binding handshake。
- TeamStore transaction、CAS、immutable revisions、outbox 與 backup/restore。
- Query + ETag 正確性基礎與 SSE 預設 push；WebSocket 選配。
- Context Snapshot、verify evidence、remote drift 分解與 lifecycle gates。

Server Admin UI 只管理 installation、Store、帳號、Project/Repo、migration 與備份；日常規格操作仍由 Desktop 或其他
Presentation UI 負責。

有 checkout 時 Agent-visible Context Projection 預設位於 workspace root 的 `.speclink/context/`，不是 `.git/`；
這可避開 linked worktree 的 `.git` file、搜尋工具排除與 cross-worktree cache 混用。無 checkout 時使用 Desktop app
data、MCP resources/search 或 host-managed Session FS。

## 設定歸屬

| 位置 | 責任 |
|---|---|
| `openspec/config.yaml` 或 Remote Store config | Workflow policy：schema、context、rules、locale、TDD、audit |
| `.speclink.yaml` | Repo/workspace binding 與本機工具整合；不得存 credential |
| `.speclink/` | Context snapshot、touched/evidence 暫存與其他本機工作資料 |
| OS Keychain | Remote credential |

本地現行解析與遷移方式見[設定說明](docs/configuration.zh-TW.md)。遠端 policy 以 Store revision 為正典，不允許本機
override 靜默改寫團隊規則。

## Repository Layout

```text
crates/
├── speclink-core       Rust SDD engine
├── speclink-fs         Local filesystem Store
├── speclink-cli        CLI frontend
├── speclink-remote     Existing remote client foundation
└── speclink-node       N-API Node bindings

apps/desktop/           Tauri + React Desktop
packages/ui/            Shared UI and DataSource contract
openspec/               This repository's specs and change history
docs/                   Current documentation and platform blueprint
```

## 開發與驗證

```bash
cargo test --workspace
npm --workspace @speclink/desktop test
npm --workspace @speclink/desktop run build
```

Engine/Store 重構必須維持 CLI parity、render golden、Store abstraction 與跨平台測試。Remote Phase 還必須加入
TeamStore conformance、故障注入、transaction recovery、Protocol 與 Desktop end-to-end 測試。

## 文件

| 文件 | 用途 |
|---|---|
| [平台架構藍圖](docs/platform-architecture.zh-TW.md) | 唯一目標架構、Local/Remote、Server、Desktop、Store 與 Roadmap 基準 |
| [本地入門教學](docs/getting-started.zh-TW.md) | 目前 Local Repo 完整 SDD 流程 |
| [設定說明](docs/configuration.zh-TW.md) | 目前本地設定、歸屬與遷移 |
| [Node SDK](docs/sdk-node.zh-TW.md) | 目前 N-API SDK、Store bridge 與 dispatch surface |
| [品牌資產](docs/assets/brand/README.md) | Logo、配色與使用方式 |

`openspec/changes/archive/` 與 `openspec/discussions/archive/` 是歷史稽核資料，不是現行架構入口。

## Roadmap

1. **Phase 1：Engine 與正確性基礎**：Typed Runtime、TeamStore contract、UoW、events、binding 與 Protocol。
2. **Phase 2：Remote Server**：SQLite/FS/PostgreSQL、Auth、Admin UI、SSE、migration 與 backup/restore。
3. **Phase 3：Desktop Remote Workspace**：WorkspaceSession、登入、Project/Repo、checkout、offline/conflict UX。
4. **Phase 4：Agent 與生態**：Copilot Tools、MCP、自訂 UI、SSO、runtime plugins 與 Cluster mode。

詳細依賴、P0/P1 正確性條件與非目標見[平台架構藍圖](docs/platform-architecture.zh-TW.md)。

## License

[MIT](LICENSE)
