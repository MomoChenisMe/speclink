# Speclink 可組合平台架構藍圖

> 本文件描述 Speclink 從本地 SDD 工具擴展為可組合平台的完整目標架構，並作為後續設計的整合基準。
> 既有架構、團隊模式與動詞契約文件反映先前設計，暫不限制本文；待本藍圖定案後再依此更新。
> 本文不表示所有目標元件皆已完成。

## 1. 核心結論

Speclink 應定義成「**同一套 Engine、兩條執行路徑、多種可替換入口與呈現**」。

- **本地 repo 路徑**：保留現在的 `openspec/`、Git、直接讀檔與離線體驗，不要求服務。
- **遠端 Store 路徑**：規格正典位於自訂 Store，經使用者自建服務或官方 `speclink-server` 組合 Engine 與 Store。
- **外部 Agent 入口**：CLI、MCP、Desktop 或自訂 UI 經 Speclink Protocol 呼叫遠端 Host。
- **同系統 Agent 入口**：Copilot SDK 等自訂 Agent 以 In-process Tool 直接呼叫同程序 Speclink Host，不繞 CLI、MCP 或 HTTP。
- **UI 可替換**：Speclink Desktop 是官方開箱 UI 與參考實作，不是唯一 UI；本地 repo 建議使用 Desktop/native UI，遠端 Store 可建立 Web、Desktop 或 Agent 內嵌 UI。
- **Store 可替換**：本地 FS、PostgreSQL、Server FS、NAS 或自訂媒介皆可實作 Store，但必須聲明可提供的並發與交易保證。

最終產品邊界如下：

```text
Speclink
├── Engine SDK                 SDD 流程與規格語意
├── Speclink Host              身分、政策、並發、交易、事件
├── Store Adapter Contract     可替換的規格持久化介面
├── Client Protocol            Command / Query / Context / Event
├── Invocation Adapters        Embedded / CLI / MCP / HTTP / Tool
├── DataSource / UI Contract   可替換的呈現介面
└── 官方選配產品
    ├── FsStore
    ├── PostgreSQL Store
    ├── speclink-server
    ├── Speclink CLI
    ├── MCP Adapter
    └── Speclink Desktop
```

## 2. 設計原則

### 2.1 一套流程語意

本地與遠端必須使用同一顆 Engine。`discuss`、`propose`、`apply`、`validate`、`analyze`、
`drift`、`verify`、`archive` 等流程不能因 Store、Client 或 UI 不同而分叉。

### 2.2 程式碼工作區與規格 Store 正交

PM/PO 與 RD 在三種產品情境中都可以取得程式碼 checkout。差異不在「有沒有程式碼」，而在：

- 規格正典位於 repo 的 `openspec/`，或位於遠端 Store。
- Agent 在外部工具、同系統應用，或本地 CLI 中執行。
- UI 使用官方 Desktop 或自訂呈現。

### 2.3 Store 不是遠端 Client API

`Store` 是 Engine/Host 內部的持久化介面。外部 CLI、Desktop、Web UI 與 Agent 不應直接連 PostgreSQL、
NAS 或 Store 實作，而應經 Speclink Host 的應用層介面。

```text
正確：Client / Tool -> Host -> Engine -> Store
錯誤：Client / Tool -----------------> Store
```

In-process Tool 可以略過 HTTP，但不能略過 Host 與 Engine，否則會繞過 lifecycle、權限、CAS、交易、
封存規則與事件發布。

### 2.4 UI 是 Presentation Adapter

UI 不屬於 Engine 或 Store。官方與自訂 UI 都只依賴穩定的 DataSource/Client contract，因此 Store 的實體媒介
與 UI 技術選型可以獨立替換。部署建議仍有清楚邊界：本地 repo 以能直接內嵌 Engine 並存取 workspace 的
Desktop/native UI 為主；Web UI 以具備 Host、Query Protocol 與 Event transport 的遠端 Store 模式為主。

### 2.5 遠端規格只有一個正典

遠端模式不得維護可雙向寫入的第二份 `openspec/`。Agent 需要檔案閱讀體驗時，使用可丟棄、唯讀、
帶 revision 的 Context Projection；所有遠端寫入仍經 Command API 或 In-process Tool。

## 3. 完整架構圖

```mermaid
flowchart TB
    USER["PM / PO / RD<br/>Codex、Claude、Copilot SDK、自訂 Agent"]
    MODE{"Workspace 儲存模式"}

    subgraph PRESENTATION["可替換 Presentation 層"]
        DESKTOP["Speclink Desktop<br/>官方開箱 UI"]
        CUSTOMDESKTOP["自訂 Desktop UI"]
        WEBUI["自訂 Web UI"]
        AGENTUI["同系統 Agent 內建 UI"]
        HEADLESS["無 UI<br/>CLI / MCP / Agent Only"]
    end

    USER --> MODE
    DESKTOP --> MODE
    CUSTOMDESKTOP --> MODE
    WEBUI --> MODE
    AGENTUI --> MODE
    HEADLESS --> MODE

    subgraph LOCAL["路徑 A：本地 Repo 模式"]
        LOCALDS["LocalDataSource"]
        LOCALHOST["Embedded Speclink Host"]
        LOCALENGINE["Engine SDK"]
        FSSTORE["FsStore"]
        OPENSPEC["repo/openspec/<br/>specs、changes、discussions"]
        GIT["Git<br/>branch / commit / push / pull / merge"]
        FSWATCH["Filesystem Watcher<br/>或 Local Domain Events"]

        LOCALDS --> LOCALHOST --> LOCALENGINE --> FSSTORE --> OPENSPEC --> GIT
        OPENSPEC --> FSWATCH --> LOCALDS
    end

    subgraph REMOTE["路徑 B：遠端 Store 模式"]
        subgraph EXTERNAL["外部 Client 入口"]
            EXTCLIENT["CLI / MCP / Desktop / 自訂 UI"]
            PROTOCOL["Command / Query / Context Protocol"]
            MATERIALIZER["Context Materializer"]
            CONTEXT[".speclink/context/<br/>唯讀、gitignored"]

            EXTCLIENT --> PROTOCOL
            PROTOCOL --> MATERIALIZER --> CONTEXT
        end

        subgraph INTERNAL["同系統 Agent 入口"]
            APP["自家應用"]
            AGENT["Copilot SDK / 自訂 Agent"]
            TOOL["In-process Tool Adapter"]
            SESSIONCTX["Tool-native Context<br/>或 Session FS Projection"]

            APP --> AGENT --> TOOL
            AGENT --> SESSIONCTX
        end

        SERVICE["自訂服務<br/>或 speclink-server"]
        REMOTEHOST["Speclink Host<br/>Actor / Auth / Repo / CAS / Transaction"]
        BINDING["Project / Repo Registry<br/>Binding + Capabilities"]
        REMOTEENGINE["Engine SDK"]
        ADAPTER["Store Adapter"]
        STORES["PostgreSQL / Server FS / NAS / 自訂儲存"]
        BUS["Domain Event Bus"]
        EVENTS["Transport Discovery<br/>SSE / WebSocket / Polling + ETag"]

        PROTOCOL --> SERVICE --> REMOTEHOST
        TOOL -->|"同程序呼叫，不走 HTTP"| REMOTEHOST
        REMOTEHOST --> BINDING
        REMOTEHOST --> REMOTEENGINE --> ADAPTER --> STORES
        REMOTEHOST --> SESSIONCTX
        REMOTEHOST --> BUS
        BUS -->|"Local event"| TOOL
        BUS --> EVENTS --> EXTCLIENT
    end

    MODE -->|"無 remote 設定"| LOCALDS
    MODE -->|".speclink.yaml 設定 remote"| EXTCLIENT
    MODE -->|"應用內嵌 Speclink Host"| TOOL

    DESKTOP -. "官方雙 DataSource" .-> LOCALDS
    DESKTOP -. "官方雙 DataSource" .-> EXTCLIENT
    CUSTOMDESKTOP -. "可直接內嵌 SDK" .-> LOCALHOST
    CUSTOMDESKTOP -. "或使用 Client SDK" .-> PROTOCOL
    WEBUI -. "建議用於遠端模式" .-> PROTOCOL
    AGENTUI -. "直接使用 Host Query/Command" .-> TOOL
    CONTEXT -. "Agent Read / Search / Grep" .-> USER
    OPENSPEC -. "Agent 直接讀寫檔案" .-> USER

    LOCALENGINE -. "共用相同流程語意" .-> REMOTEENGINE
```

## 4. 核心元件與契約

### 4.1 Engine SDK

Engine 負責 SDD 領域語意：

- change、artifact、discussion 與 canonical spec 的生命週期。
- schema、instructions、task、validate、analyze、drift 與 archive。
- 流程守衛、artifact 依賴與狀態推導。
- 成功異動後產生 domain events，例如 `ArtifactUpdated`、`ChangeReady`、`TaskCompleted`、`Archived`。

SDK 應逐步提供 typed command 介面；既有 `dispatch(argv, stdin)` 保留為 CLI、skills 與通用 Tool 的相容入口。

Rust Engine 是唯一流程語意實作。除 Rust crate 外，官方還必須以 N-API 發布 Node.js native addon，讓
Copilot SDK、自訂 Node.js Agent 與 `speclink-server` 直接內嵌同一份 Engine，而不是在 TypeScript 重寫流程規則。
Node binding 應包裝 typed commands、queries、context 與 domain events；`dispatch(argv, stdin)` 只是相容層。

```ts
await host.execute({
  type: "discussion.addRound",
  actor,
  project,
  repo,
  expectedRevision,
  payload: { slug, mode, content },
});
```

### 4.2 Speclink Host

Host 是 Engine 對外的唯一應用層邊界，負責：

- actor、project、repo context 與認證授權。
- lifecycle 與 ownership 的服務端裁決。
- optimistic concurrency、CAS、idempotency 與 Unit of Work。
- 組合 Engine 與 Store。
- commit 成功後發布 domain events。
- 提供 Query 與 Context Service。

HTTP server 只是 Host 的一個 adapter。使用者在自家系統內可直接把 Host 當 library 使用，不必先自我呼叫 HTTP。

### 4.3 Store Adapter Contract

現有 `speclink_core::store::Store` 與 Node SDK Store 以領域詞彙存取 change、artifact、spec、discussion、config
與 archive。實體佈局由 adapter 決定。

為避免「任何媒介都能接」退化成「每個媒介的正確性不同且不可知」，Store/Host 應聲明能力等級：

| 能力等級 | 適用媒介 | 最低保證 |
|---|---|---|
| Single-writer | 本地 FS、單一 Host 掛載 NAS | Host 內序列化、原子檔案替換或 recovery journal |
| Team | PostgreSQL 等 | revision、CAS、transaction、多使用者安全寫入 |
| Cluster | PostgreSQL + 分散式 Event Bus | 多 Host instance、可靠 invalidation 與跨 instance 協調 |

NAS 或網路磁碟應由單一 Host 存取，不應讓每台 Desktop 直接共同寫入同一掛載點。若要多 Host，adapter 必須另有可靠的分散式鎖與交易機制。

### 4.4 Invocation Adapters

所有入口都應轉為同一個 Host command/query，而不是各自實作流程：

| Adapter | 使用位置 | 是否經網路 |
|---|---|---|
| Embedded Adapter | 本地 CLI、Desktop、自訂 native app | 否 |
| CLI Adapter | Claude Code、Codex CLI、Codex App | 遠端模式需要 |
| MCP Adapter | Claude Desktop、通用 Agent Host | 通常需要 |
| HTTP Protocol Adapter | Desktop、Web UI、第三方 Client | 是 |
| In-process Tool Adapter | Copilot SDK、自家 Agent 平台 | 否 |

### 4.5 Client Protocol

自訂服務若希望被 Speclink CLI、Desktop 或第三方 UI 使用，應實作並通過同一套 protocol conformance tests：

- **Command API**：寫入與流程動詞。
- **Query API**：change、spec、discussion、artifact 與 derived status。
- **Context API**：為 Agent 建立一致 snapshot。
- **Event discovery**：Query + ETag 為必要恢復地基，另宣告 SSE/WebSocket push transports 與 resume 能力。
- API version、標準 error reason、ETag/If-Match 與 actor/repo scope。

Protocol 應發布 OpenAPI/JSON Schema、Client SDK 與 conformance suite，避免每個自訂服務對相同動詞產生不同語意。

### 4.6 Project / Repo Binding Contract

遠端 Store 先以 Project 隔離產品或 SDD 工作空間，再於 Project 內註冊一至多個 Repo：

```text
Speclink Service
└── Project: ERP（prj_01H...）
    ├── Repo: backend（repo_01H...）
    ├── Repo: frontend（repo_02H...）
    └── Repo: mobile（repo_03H...）
```

- **Project ID** 與 **Repo ID** 是 server 產生、不可變的內部身分。
- `key` 是穩定、可讀的專案內名稱，例如 `erp`、`backend`；顯示名稱可修改。
- `gitUrl`、default branch 與 monorepo path 是驗證或呈現 metadata，不是身分來源。fork、mirror 與搬移 repo 不得改變 Repo ID。
- canonical specs 與 changes 預設為 repo-scoped，避免不同 repo 使用相同 capability 名時碰撞。
- discussion 可為 project-scoped；轉為 change 時必須指定 Repo。一個跨 repo discussion 可轉出多個 changes，每個 change 恰屬一個 Repo。

官方 server 應提供 admin UI/API 建立 Project 與註冊 Repo；自訂服務可映射既有產品、tenant 與 repository model，
但對 Speclink Host 必須提供相同的 `projectId`/`repoId` binding。

本地 workspace 使用 project-scoped URL 加 repo key 綁定：

```yaml
remote:
  url: https://team.example.com/api/speclink/v1/projects/prj_01H
  repo: backend
```

連接流程：

1. 管理員在 server 建立 Project 並註冊 repos。
2. 使用者在 code workspace 執行 `speclink link <project-url> --repo <repo-key>`，或由 Desktop 設定相同 binding。
3. Client 以使用者憑證向 Host 驗證 Project 存取權與 Repo 是否存在。
4. 驗證成功後才保存 binding；每個遠端 command/query 自動攜帶 repo 身分。
5. Git remote 僅作不一致警告，不自動覆寫 binding。

對 monorepo，Repo registration 可另外帶 `workspacePath`；同一 Git remote 下不同子工作區仍可擁有不同 Repo ID。

### 4.7 Agent Execution Binding

Agent 不應自行推測或在 tool arguments 中選擇 Project/Repo。每個 Agent session 在開始前必須取得不可含糊的
`SpeclinkExecutionContext`：

```ts
interface SpeclinkExecutionContext {
  actor: Actor;
  project: { id: string; key: string; name: string };
  repo: { id: string; key: string; name: string };
  workspaceRoot?: string;
  mode: "fs" | "shared-store";
  apiVersion: string;
  engineVersion: string;
  capabilities: SpeclinkCapabilities;
}
```

| 入口 | Binding 來源 |
|---|---|
| CLI、Codex、Claude Code | 從 cwd 向上尋找 `.speclink.yaml` |
| Workspace-scoped MCP | MCP 啟動時綁定 workspace root |
| Global MCP | 使用者先於 host UI 選 workspace，MCP 只取得 opaque workspace ID |
| Copilot SDK / In-process Tool | Tool closure 綁定 session 的 actor/project/repo |
| Speclink Desktop | 開啟的 workspace 與 `.speclink.yaml` |
| 遠端 Web UI | 登入後由使用者選 Project/Repo，保存於 server session |
| 同系統 Agent UI | 應用自己的登入、tenant 與 workspace context |

遠端 Host 應提供 binding handshake，例如 `GET /binding`：

```json
{
  "actor": { "id": "u_42", "name": "王小明" },
  "project": { "id": "prj_01H", "key": "erp", "name": "ERP" },
  "repo": { "id": "repo_01H", "key": "backend", "name": "Backend" },
  "apiVersion": "1",
  "engineVersion": "1.4.0",
  "capabilities": {
    "contextSnapshots": true,
    "events": ["sse", "websocket", "polling"]
  }
}
```

每個 skill/tool workflow 開始時先確認 binding，並把 `Project / Repo` 顯示給使用者與 Agent。binding 缺失、無權限
或有多個候選時必須停止，不得自動選第一個。長時間 Agent session 的 binding 應固定；切換 Project/Repo 必須建立
新 session 或執行明確 rebind，避免前一個 Project 的 context 洩漏到下一個。

### 4.8 Workflow Policy 的歸屬

本地 repo 模式以 `openspec/config.yaml` 為 workflow policy 文件；遠端模式則以 Remote Store 內 repo-scoped、
可版本化的 `config.yaml` 為唯一 authoritative policy。`schema`、`context`、`rules`、`spec_locale`、`tdd` 與
`audit` 均由 Host 在固定 Store revision 讀取，再交給同一份 Rust Engine fail-closed 解析成
`EffectiveWorkflowPolicy`；`policyRevision` 與 digest 必須進入 instructions、Context Snapshot、apply 與 verify bundle。

遠端模式的 `.speclink.yaml` 只負責 endpoint、Project/Repo binding 與本機 client preferences，不鏡射 workflow
policy，也不得用本機環境變數或 deprecated policy keys 靜默覆寫團隊規則。Context Projection 可以包含遠端
`config.yaml` 的唯讀副本供 Agent 閱讀，但該副本不是第二個正典，修改它不會更新 policy。

## 5. 本地 Repo 模式

### 5.1 架構與特性

```text
Agent / UI
  -> Embedded Speclink Host
  -> Engine SDK
  -> FsStore
  -> repo/openspec/
  -> Git
```

本地模式完整保留目前優點：

- Agent 可直接找到並搜尋 `openspec/`。
- 規格與程式碼共享 Git history。
- PM 到 RD 的交接透過 commit、push、pull 或 PR。
- 完整離線使用。
- 不需要認證、server、SSE 或 Context cache。
- Desktop 以 LocalDataSource 讀本地檔案，使用 filesystem watcher 或 local event 更新。
- 使用者可不用 Desktop，自行建立 native UI 並內嵌 SDK/Host。

### 5.2 完整流程

```mermaid
sequenceDiagram
    participant PM as PM/PO Agent
    participant H as Embedded Host
    participant E as Engine SDK
    participant FS as openspec/
    participant Git as Git Remote
    participant RD as RD Agent
    participant UI as Desktop / 自訂 UI

    PM->>H: discuss + propose
    H->>E: 執行流程規則
    E->>FS: 建立 discussion 與 change artifacts
    PM->>Git: commit + push 規格
    Git-->>RD: pull / checkout 規格變更
    RD->>FS: 讀 proposal、design、tasks、specs
    RD->>H: apply / task done / verify
    H->>E: 執行流程與更新任務
    E->>FS: 更新 tasks 與 change artifacts
    RD->>Git: commit + push 程式碼與規格
    FS-->>UI: Filesystem watcher / local event
    RD->>H: archive
    H->>E: 檢查任務、合併 delta specs
    E->>FS: 更新 canonical specs 並封存 change
```

### 5.3 本地 UI 建議

本地 repo 模式建議使用 Speclink Desktop 或自訂 Desktop/native UI，讓 UI 直接內嵌 Host/Engine、讀取
workspace 的 `openspec/`，並以 filesystem watcher 或 local domain event 更新。此路徑維持零服務、完整離線與
作業系統檔案權限的一致體驗。

不建議為本地模式另建 Web UI。一般瀏覽器無法任意讀取 repo 檔案，也無法直接載入 native Engine；為此加入
localhost bridge 會額外引入程序生命週期、port、認證、CORS 與檔案權限問題，卻沒有遠端協作帶來的收益。
需要 Web UI 時，應採遠端 Store 模式，讓瀏覽器透過正式 Query/Command/Event Protocol 存取 Host。

## 6. 遠端 Store 模式

### 6.1 外部 Client 路徑

```text
Codex / Claude / CLI / MCP / Desktop / Web UI
  -> Speclink Protocol
  -> 自訂服務或 speclink-server
  -> Speclink Host
  -> Engine SDK
  -> Store Adapter
  -> Remote Store
```

外部 Agent 仍在本地 code checkout 工作；規格位於遠端。Agent 透過 Context Projection 取得容易搜尋的唯讀規格，寫入一律走 Command API。

### 6.2 同系統 Agent 路徑

```text
Copilot SDK / 自訂 Agent
  -> @speclink/copilot-tools
  -> @speclink/host
  -> @speclink/engine（Rust 經 N-API）
  -> Node Store Adapter
  -> 同系統 Store
```

此路徑不需要 CLI、MCP 或外部 HTTP。Tool handler 與 Host/Store 位於同一系統時，直接做 library call 即可，但仍須通過 Host 與 Engine。

### 6.3 完整流程

```mermaid
sequenceDiagram
    participant A as PM/RD Agent
    participant C as CLI / MCP
    participant T as In-process Tool
    participant H as Speclink Host
    participant E as Engine SDK
    participant S as Remote Store
    participant UI as Desktop / Web / Embedded UI

    alt 外部 Client binding
        A->>C: 從 code workspace 啟動
        C->>H: GET /binding（workspace 設定）
        H-->>C: actor + project + repo + capabilities
        C-->>A: 顯示目前 Project / Repo
    else 同系統 Agent binding
        A->>T: 建立 Agent session
        T->>H: 綁定 session actor/project/repo
        H-->>T: execution context + capabilities
    end

    alt 外部 Agent
        A->>C: 啟動 discuss / propose / apply / verify
        C->>H: Protocol prepare context
        H->>S: 建立一致 snapshot
        S-->>H: documents + revisions
        H-->>C: Context Manifest
        C-->>A: 產生 .speclink/context/
    else 同系統 Agent
        A->>T: speclink tool call
        T->>H: 直接 prepare context
        H->>S: 建立一致 snapshot
        S-->>H: documents + revisions
        H-->>T: Tool-native context
        T-->>A: 結果或 Session FS projection
    end

    Note over A: Agent 讀取程式碼<br/>並搜尋取得的規格 context

    alt 外部寫入
        A->>C: Speclink 寫入動詞
        C->>H: Command + expectedRevision
    else 同系統寫入
        A->>T: In-process Tool call
        T->>H: Command + bound actor/project/repo
    end

    H->>E: 執行流程規則
    E->>S: CAS / transaction
    S-->>H: 新 revision
    H-->>T: Local domain event
    H-->>UI: 選定 transport 的 invalidate
    UI->>H: 重新查詢
    H-->>C: Invalidate / conflict on next command
    C-->>A: Context 標記 stale 或 refresh
```

### 6.4 規格運算與程式碼驗證的邊界

- `validate`、`analyze` 與規格面的 `drift` 在遠端模式由 server 端固定 Engine 版本運算，確保全隊一致。
- `apply` 與實作面的 `verify` 必須在有 code checkout 的 RD/Agent 環境執行，因為只有該環境能檢查程式碼並執行測試。
- Remote Store 保存 tasks 與 task completion 時回報的 touched-file evidence；verify 先取得固定版本 bundle，再於 RD 本機比對該 evidence、實際程式碼與測試結果。
- verify 結果寫回遠端成為 evidence，但 server 不應假裝能驗證它拿不到的本地程式碼狀態。

```mermaid
sequenceDiagram
    participant R as RD Agent
    participant C as CLI / In-process Tool
    participant H as Speclink Host
    participant S as Remote Store
    participant W as Local Code Checkout

    R->>C: prepare verify(change)
    C->>H: Query VerifyBundle
    H->>S: 讀取一致的 specs、tasks、policy 與 task evidence
    S-->>H: snapshot + revisions + touched files
    H-->>C: VerifyBundle
    C-->>R: 固定 spec/tasks/policy revision
    R->>W: 比對 touched paths、commit、程式碼並執行 test/audit
    W-->>R: Local verification results
    R->>C: submit VerifyEvidence
    C->>H: result + commit SHA + expected revisions
    H->>S: 驗證 revision 並保存 evidence
    alt 任一 basis revision 已改變
        H-->>C: stale_evidence
    else evidence 接受
        H-->>C: verified 或 gate status
    end
```

`VerifyBundle` 至少包含：

```ts
interface VerifyBundle {
  changeId: string;
  repoId: string;
  specRevision: string;
  tasksRevision: string;
  policyRevision: string;
  tasks: Task[];
  taskEvidence: Array<{
    taskId: string;
    actorId: string;
    touchedFiles: string[];
    baseCommit?: string;
    headCommit?: string;
  }>;
  requiredDisciplines: Array<"test" | "tdd" | "audit">;
}
```

Touched files 是檢查範圍與追蹤線索，不是實作正確性的證明。它們必須按 stable task ID、actor、repo 與
base/head commit 保存，不能只保留跨任務合併後的路徑清單。完成本機驗證後，`VerifyEvidence` 應帶回
spec/tasks/policy revisions、commit SHA、逐 task 結果、test/audit 摘要、Agent/CLI/Engine 版本與 trust level。
若任一 basis revision 已變動，Host 回 `stale_evidence` 並要求重新取得 bundle。

### 6.5 遠端 Drift 的分解與彙整

`drift` 在產品語意上仍是一個 read-only 動詞，但遠端執行必須拆成 Store 規格面與本機實作面，不能把
現有依賴 Git/worktree 的五個維度整體搬到 server：

| 維度 | 執行位置 | 主要輸入 |
|---|---|---|
| Specs | Server | change delta、canonical specs、base/current revisions |
| Time | Client | change created metadata、本機 Git commit window |
| Structure | Client | design anchors、Context Projection、本機 tracked files/symbols |
| Tasks | Client | tasks、task file references、本機 commit 與 worktree 狀態 |
| Environment | Client | 本機 Git log、HEAD、dirty state、task evidence/touched files |

完整遠端流程：

```mermaid
sequenceDiagram
    participant R as RD Agent
    participant C as Remote CLI / Tool
    participant H as Speclink Host
    participant E as Rust Engine
    participant S as Remote Store
    participant W as Local Code Checkout

    R->>C: drift(change)
    C->>H: prepareDrift(change, binding)
    H->>S: 建立一致 spec/config/change snapshot
    S-->>H: documents + revisions
    H->>E: computeSpecDrift(snapshot)
    E-->>H: SpecDriftReport + DriftBundle
    H-->>C: bundle + Context Manifest
    C->>W: materialize context 並讀 git/worktree
    C->>E: computeWorkspaceDrift(bundle, workspace facts)
    E-->>C: Time + Structure + Tasks + Environment
    C->>E: mergeDriftReports(server, client)
    E-->>C: CombinedDriftReport
    C-->>R: 顯示單一報告與 basis revisions
```

`DriftBundle` 應固定 `projectId`、`repoId`、change/spec/config revisions、Context Snapshot ID、created metadata、
design、tasks 與 task evidence。Remote CLI 偵測到 binding 後，不再進入目前的 fs-only `cmd_drift`，而是呼叫
`prepareDrift`、materialize context、執行本機 workspace analyzer，再使用 Rust Engine 的共用 merger 輸出既有
human/JSON shape。如此 CLI、Node SDK 與 Desktop 不會各自重寫 scoring 與合併規則。

沒有 code checkout 時，Time、Structure、Tasks、Environment 必須標示 `unavailable`，不能視為 clean 或零分。
Remote CLI 預設回 `workspace_required`；只有顯式 `--spec-only` 才輸出 `coverage: "spec-only"` 的部分報告。
合併結果帶 basis revisions；若報告產生期間遠端 revision 已改變，標示 stale 並要求重跑。Drift 是診斷結果，
不直接寫回正典；若要保存為 handoff/evidence，另走帶 revision 的明確 command。

## 7. Agent Context Projection

### 7.1 為什麼需要

Coding Agent 擅長對檔案做 Read、Search 與 Grep。若遠端模式只提供逐文件 API，Agent 會增加大量 round trip，難以做跨文件搜尋；若完整雙向同步遠端 Store，又會產生第二個可寫真相。

因此採用「遠端正典、本地唯讀 snapshot」：

```text
Remote Store（唯一真相）
  -> Context API / Snapshot
  -> .speclink/context/（可丟棄、唯讀、gitignored）
  -> Agent Read / Search / Grep
```

### 7.2 建議佈局

```text
.speclink/context/
├── manifest.json
├── INDEX.md
└── openspec/
    ├── config.yaml
    ├── LANGUAGE.md
    ├── specs/
    │   └── payment/spec.md
    └── changes/
        └── add-payment/
            ├── proposal.md
            ├── design.md
            ├── tasks.md
            └── specs/
```

Context 規則：

- 永遠 gitignored，且可隨時刪除重建。
- 文件帶 snapshot ID、revision 與 digest。
- Agent 可讀但不能把直接檔案修改視為遠端寫入。
- 所有寫入帶 `expectedRevision` 經 Host 執行。
- push/local event 只將 context 標記 stale，不在 Agent 閱讀途中偷偷覆換文件。
- `refresh` 建立新 snapshot；衝突時要求重新讀取或先執行 drift/ingest。

### 7.3 依流程縮小 context

| 流程 | 預設 context |
|---|---|
| discuss | config、LANGUAGE、canonical specs 索引 |
| propose | discussion、相關 canonical specs、schema/template |
| apply | proposal、design、tasks、delta specs、base specs |
| verify | apply context、最新 tasks、驗證規則 |
| archive | delta specs、canonical base、tasks、revision |

同系統 Agent 可用 Tool-native Context 或 Session FS Projection，通常不需要先寫入實體 `.speclink/context/`；兩者仍消費同一個 Context Service 與 snapshot 語意。

## 8. In-process Tool

Copilot SDK 等 Agent runtime 可註冊自訂 Tool。Speclink 應提供一個 `InProcessToolAdapter`，把 tool call 轉成 Host command/query。

Copilot SDK 的 Node.js/TypeScript runtime 不能直接載入 Rust crate，因此官方 Node 路徑必須明確交付為：

```text
@speclink/engine
  Rust speclink-runtime -> napi-rs / N-API -> 預編譯 .node binary

@speclink/host
  Node application boundary -> auth / binding / execution context / Engine + Store composition

@speclink/copilot-tools
  Copilot SDK tool schemas + handlers -> 同 process 呼叫 @speclink/host
```

完整的同系統執行路徑為：

```text
Copilot SDK Agent
  -> @speclink/copilot-tools
  -> @speclink/host
  -> @speclink/engine（N-API / Rust）
  -> Node Store Adapter
  -> 既有 PostgreSQL repository 或其他同系統儲存層
```

此路徑沒有 CLI、MCP 或 HTTP round trip。Tool handler 不直接讀寫 Store，而是直接呼叫同 process Host；
Node Store Adapter 才連到同系統的 repository。如此可省去網路繞行，同時保留 lifecycle、authorization、
revision、CAS、policy、stable task ID 與 domain event 規則。

Node 發布契約至少包含 Windows、macOS、Linux 的 x64/arm64 預編譯 binary、Node ABI/N-API 相容範圍、
Engine/native binary 版本檢查與載入錯誤。Native addon 不可用時必須 fail closed，不得靜默切換到另一套
JavaScript 流程實作。若 Store 由 Node.js 實作，Node binding 需提供受測試的 async Store bridge，並維持
與 Rust Store contract 相同的 transaction、CAS 與錯誤語意。

初期可維持既有技能的動詞詞彙：

```ts
const speclinkTool = defineTool("speclink", {
  description: "Execute a Speclink workflow verb",
  parameters: z.object({
    argv: z.array(z.string()),
    stdin: z.string().optional(),
  }),
  handler: ({ argv, stdin }) =>
    speclinkHost.dispatch({
      actor: sessionUser,
      project: sessionProject,
      repo: sessionRepo,
      argv,
      stdin,
    }),
});
```

安全規則：

- `actor`、`project`、`repo` 由應用 session 綁定，不能讓模型自行傳入。
- Generic `speclink` Tool 不應整體 `skipPermission`，因為同時含讀寫動詞。
- Host 仍執行授權、revision、ownership 與 lifecycle 檢查。
- 多租戶 runtime 應顯式註冊 Tool、隔離 session 與 workspace，避免環境權限外溢。

未來若需更細權限，可拆成：

```text
speclink_query      唯讀查詢
speclink_command    寫入與流程命令
speclink_context    Context snapshot/search
```

參考：[GitHub Copilot SDK 自訂 Tool 範例](https://github.com/github/copilot-sdk/blob/main/nodejs/examples/basic-example.ts)、
[多租戶部署指南](https://github.com/github/copilot-sdk/blob/main/docs/setup/multi-tenancy.md)。

## 9. Event 與即時更新

### 9.1 事件原則

Engine/Host commit 成功後發布 domain event；事件是 invalidation hint，不承載完整規格內容。Client 收到事件後經 Query API 重讀，因此漏掉事件仍能以 polling 恢復。

```json
{
  "type": "invalidate",
  "scope": "change",
  "id": "add-payment",
  "revision": 42
}
```

### 9.2 Transport 決策

遠端更新應拆成「正確性地基」與「通知 transport」兩層：

```text
Query + ETag（所有遠端服務必備）
├── 純 Polling
├── SSE invalidation
└── WebSocket invalidation / bidirectional channel
```

三種遠端選擇都可支援，但不是三個彼此獨立的真相來源：

- **Polling + ETag** 是必要 fallback，也是 Client 初次載入與漏事件後恢復的地基。
- **SSE** 適合 Speclink 的單向 invalidation，實作與 proxy 維運成本較低，建議作官方預設 push transport。
- **WebSocket** 適合已有 WS 基礎設施，或同時需要 Agent chat、presence、共同編輯等雙向通道的服務。
- 一個 Client subscription 一次只選一個 push transport，不同時開 SSE 與 WebSocket，避免重複事件與排序歧義。
- 事件只攜 `eventId`、scope、resource ID 與 revision；Client 收到後仍以 Query + ETag 重讀正典。

Server 經 binding/capability discovery 宣告：

```json
{
  "events": {
    "transports": [
      { "type": "sse", "url": "/events", "resume": true },
      { "type": "websocket", "url": "/ws", "resume": true }
    ],
    "polling": { "url": "/sync-state", "etag": true }
  }
}
```

Client 選擇規則：

1. In-process Agent/UI 使用 Local Event Bus。
2. 遠端規格 invalidation 優先選 SSE；服務只提供 WebSocket 時選 WS。
3. 自訂應用若同時承載 Agent 雙向串流，可明確偏好 WebSocket。
4. push 連線失敗或 resume cursor 過期時，立即以 Polling + ETag 恢復，不猜測遺失內容。
5. 一般短命 CLI 不維持事件連線；只有 `watch` 或長時間 Agent workflow 訂閱。

Browser 認證需額外約束：原生 `EventSource` 不適合攜帶自訂 Bearer header。Web UI 應使用 same-origin HttpOnly
cookie 或 fetch streaming；不得把 PAT 放進 SSE/WS URL query string。WebSocket 應在 handshake 或連線後第一個
受保護訊息完成認證，並沿用相同 Project/Repo binding。

| 環境 | 更新方式 |
|---|---|
| 本地 repo | Filesystem watcher 或 local domain event |
| 同系統 Agent/UI | Local Event Subscriber |
| 遠端 Desktop/Web | SSE 或 WebSocket；Polling + ETag fallback |
| CLI | 一般命令不訂閱；watch/長任務才選 transport |

## 10. UI 與 Presentation Contract

### 10.1 Speclink Desktop 定位

Speclink Desktop 是：

- 沒有自訂 UI 時的官方開箱方案。
- 本地 repo 與遠端 Store 的雙模式 UI。
- UI/DataSource contract 的參考實作。
- 一個可替換的 presentation consumer，而非 Store 管理器或唯一介面。

```text
Speclink Desktop
├── LocalDataSource
│   └── Embedded Host -> Engine -> FsStore -> openspec/
└── RemoteDataSource
    └── Client SDK -> Protocol + negotiated events -> Remote Host
```

### 10.2 自訂 UI 選擇

| UI 形式 | 本地 Repo | 遠端 Store |
|---|---|---|
| Speclink Desktop | Embedded Host + FsStore | Client SDK + Protocol |
| 自訂 Desktop/native UI | 直接內嵌 SDK/Host | Client SDK 或 Protocol |
| 自訂 Web UI | 不建議；改用 Desktop/native UI | Web backend 或 Protocol（建議） |
| 同系統 Agent UI | 內嵌 Host | In-process Host/Tool |
| 無 UI | CLI/Agent | CLI、MCP、Tool |

### 10.3 UI 整合契約

現有 `SpeclinkDataSource` 已讓 UI 不知道背後是 Tauri 或其他來源；目標介面應收斂 Query、Command 與 Subscribe：

```ts
interface SpeclinkDataSource {
  listChanges(): Promise<ChangeSummary[]>;
  getChange(name: string): Promise<ChangeDetail>;
  getArtifact(change: string, artifact: string): Promise<Document>;
  listSpecs(): Promise<SpecSummary[]>;
  getSpec(capability: string): Promise<Document>;
  listDiscussions(): Promise<DiscussionSummary[]>;
  execute(command: SpeclinkCommand): Promise<CommandResult>;
  subscribe(listener: InvalidationListener): Unsubscribe;
}
```

可提供三個官方 adapter：

```text
SpeclinkDataSource
├── LocalDataSource       Embedded Host / Tauri / FsStore
├── RemoteDataSource      Client SDK / Protocol / negotiated event transport
└── InProcessDataSource   同系統 Host / local event
```

## 11. 三種產品情境

| 情境 | 使用路徑 | PM/PO | RD | UI 選擇 |
|---|---|---|---|---|
| 1. 外部 AI Agent | 遠端外部 Client | 本地 code checkout + Codex/Claude + remote spec | 本地 checkout + CLI/MCP + remote spec | Desktop 或自訂 UI |
| 2. 規格應用內建 Agent | 遠端 In-process Tool | 應用 Agent + code access + Tool-native context | 本地外部 Agent或同系統 Agent | 應用內建 UI、Desktop 或自訂 UI |
| 3. 目前本地 Repo | 本地 Embedded | checkout + `openspec/` + Git | checkout + `openspec/` + Git | Desktop、自訂 Desktop/native UI 或無 UI |

跨 repo 需求在 v1 維持一個 change 歸屬一個 repo；共同 discussion 可轉出多個 repo-specific changes，避免單一 change 同時承擔多個 checkout 的 apply/verify 狀態。

## 12. 兩種模式的行為差異

| 項目 | 本地 Repo | 遠端外部 Client | 遠端 In-process Tool |
|---|---|---|---|
| 規格正典 | `openspec/` | Remote Store | Remote Store |
| Agent 閱讀 | 直接讀檔 | 唯讀 Context Projection | Tool Context 或 Session FS |
| Agent 寫入 | 檔案／CLI 動詞 | Command Protocol | Host library call |
| 網路繞行 | 無 | 有 | 無 |
| 協作同步 | Git | Service + revision | Service + revision |
| 衝突控制 | Git merge | CAS / transaction | CAS / transaction |
| UI 資料來源 | LocalDataSource | RemoteDataSource | InProcessDataSource |
| 即時更新 | FS watcher/local event | SSE/WS + Polling/ETag | Local event |
| 離線 | 完整支援 | 僅既有 snapshot 可讀 | 依宿主與 Store 而定 |
| 服務需求 | 無 | 自訂服務或 `speclink-server` | 應用內嵌 Host |

## 13. 自訂服務與官方 speclink-server

使用者可以：

1. 直接內嵌 Engine/Host，實作 Store 與自己的 UI/Agent。
2. 在既有服務掛載 Speclink Protocol adapter，沿用自家認證、租戶與部署。
3. 不自行實作服務，直接部署官方 `speclink-server`。

建議官方交付拆成：

```text
@speclink/engine          Rust Engine 的 Node N-API SDK 與各平台預編譯 binary
@speclink/host            Host / command / query / context composition
@speclink/copilot-tools   Copilot SDK Tool schemas 與 in-process handlers
@speclink/protocol        schema、errors、client types、conformance fixtures
@speclink/client          遠端 Client SDK
@speclink/store-fs        FS adapter
@speclink/store-postgres  PostgreSQL adapter
@speclink/ui              DataSource contract 與共用 UI primitives
speclink-server           開箱即用 standalone server
speclink-desktop          官方雙模式 UI
```

若 `speclink-server` 宣稱可直接供團隊使用，就不能只是丟棄式 demo；最低應包含 PAT/身分、Project/Repo registry、
binding handshake、CAS、交易、migration、備份指引、Polling/ETag、SSE 與 docker-compose。WebSocket 可作選配。
若只提供流程範例，則應明確命名為 example/dev server。

## 14. 建議交付順序

1. 穩定 Engine typed commands、typed outcomes 與 domain events，保留 `dispatch` 相容層。
2. 定義 Store revision/CAS/Unit of Work 能力與 conformance suite。
3. 建立 Speclink Host，集中 actor、repo、policy、交易與事件。
4. 固定 Command、Query、Context、Event Protocol 與 Client SDK。
5. 提供 Context Materializer 與 remote skills 的 snapshot/read/write 規則。
6. 提供官方 `speclink-server`，先支援 FS 與 PostgreSQL。
7. Desktop 接入 Local/Remote/InProcess DataSource，加入 event capability negotiation、Polling/ETag 與 SSE；WebSocket 作選配。
8. 發布 Node N-API Engine binary、In-process/Copilot Tool Adapter、MCP Adapter 與多租戶整合指南。
9. 穩定 UI integration contract，讓自訂 Desktop、Web 與 Agent UI 共用。

## 15. 主要漏洞與必要補強

這些項目不是一般功能清單，而是架構成立前必須關閉的正確性、安全性與操作缺口。

### 15.1 P0：不處理就不能安全宣稱支援遠端團隊模式

| 漏洞 | 目前邏輯問題 | 必要補強 |
|---|---|---|
| Store 能力與 Host 承諾不一致 | 基礎 Store 可讀寫文件，不代表能提供 CAS、snapshot 或 transaction | 分離基礎 DocumentStore 與 TeamStore/UnitOfWork capabilities；Host 啟動時驗證，能力不足即拒絕 Team/Cluster mode |
| Context snapshot 可能混版 | 逐份讀 artifact 時其他使用者可能正在寫，manifest 內文件不是同一時點 | Store 提供 consistent snapshot，或 Host 在 project/repo read barrier 下產生 snapshot；manifest 固定 project revision |
| commit 與 event 發布不原子 | Store 已 commit 但事件發布失敗，UI/Agent 永遠收不到 invalidate | PostgreSQL 採 transactional outbox；FS 採 journal + local event；事件帶可重播 cursor，Polling/ETag 永遠可恢復 |
| task ordinal 不是穩定身分 | 任務重排或多人編輯後，第 3 項可能已不是 Agent 原先完成的任務 | 每個 task 使用 stable task ID；UI 順序是屬性，不以 ordinal 作 command identity |
| PM 到 RD handoff 缺少正式 gate | proposal 寫完不等於可 apply；RD 可能讀到仍在修改的 artifacts | 定義 `drafting -> review -> ready -> applying -> verified -> archived` 或等價 gate；內容修改使舊 approval/verify evidence 失效 |
| 遠端規格離開 Git 後沒有完整歷史 | 只存最新 Markdown 無法稽核誰改了什麼、回到哪一版 | 保存 immutable document revisions、actor、timestamp、digest、command 與 tombstone；rollback 以新 revision 表達 |
| Project/Repo scope 不完整 | capability 同名、跨 repo discussion、monorepo path 可能互相碰撞 | canonical specs/change 採 repo scope；discussion 可 project scope；Repo ID 與 workspacePath 唯一且由 binding 驗證 |
| Tool 可繞過租戶邊界 | 模型若能傳 actor/project/repo，可能存取別的 tenant | execution context 綁定 session closure；Host 重新授權每個 command；禁止模型覆寫 identity |
| 壞設定會靜默切換語意或模式 | 現有 parse error 會退回 defaults；損壞的 `.speclink.yaml` 可能被當成無 remote，損壞的 `config.yaml` 可能關掉 policy | 缺少文件時才允許明定預設；文件存在但 parse/validation 失敗必須回 typed error 並停止，不得退回 fs mode 或產生 instructions |

P0 驗收條件：

1. Team mode 不接受只宣稱支援、卻無法完成 CAS/snapshot/Unit of Work 的 Store。
2. 每個成功 command 回傳新的 resource/project revision 與 event cursor。
3. 中斷 event transport 後，Client 能單靠 Query + ETag 收斂到正確狀態。
4. 同一任務經重排後仍由相同 stable ID 定址。
5. apply 開始時記錄 spec revision；規格中途變動時後續寫入或 verify 明確失敗，而非靜默混用。
6. 管理員可查詢任一 artifact 的完整 revision、actor 與來源 command。
7. `.speclink.yaml` 或 remote `config.yaml` 存在但損壞時，所有入口一致 fail closed，且不改變目前 binding/mode。

### 15.2 P1：進入正式整合與大規模使用前補齊

| 漏洞 | 風險 | 建議 |
|---|---|---|
| 實作與規格版本缺少追蹤 | 無法回答某 commit 依哪版規格實作 | 記錄 `appliedAgainstRevision`、repo、commit/PR refs 與 verify evidence |
| Context cache 含敏感規格 | 共享電腦、備份或誤提交造成外洩 | user-only ACL、gitignore、TTL、logout/project switch 清除；敏感環境可禁用實體 projection |
| 大型專案 Context 過大 | 全量 materialize 造成延遲與模型 context 浪費 | repo-scoped index、search API、按需抓取、目的導向 context manifest |
| UI 不知道服務能力 | UI 顯示 Store/Host 不支援的 archive、events 或 approvals | binding handshake 回傳 capabilities；DataSource 依能力停用並說明，不猜測 |
| 缺少匯入、匯出與災難復原 | 無法從本地遷移、換 Store 或復原 | 定義 versioned export bundle、import validation、backup/restore 與 `store push/pull` |
| Browser transport 認證不清楚 | PAT 進 URL 或跨站 token 外洩 | same-origin HttpOnly cookie 或 fetch streaming；WS/SSE URL 禁止 token query |
| Protocol/Engine/skill 版本漂移 | Client 理解的流程與 server 不同 | handshake 協商 API、Engine、schema、skill contract；不相容就 fail closed |
| Markdown/規格內容成為攻擊面 | Web UI XSS，或規格內 prompt injection 誘導 Agent | 統一 sanitize HTML；標示不可信外部內容；Tool policy 不因文件指示提升權限 |
| 自訂 Store 品質不可知 | 名義上通過型別，實際排序、archive、衝突語意不同 | 提供 conformance suite、故障注入、transaction/event recovery 測試與 capability certification |
| Reference server 定位不清 | demo 被當 production 使用，缺備份與維運保證 | 分開 example/dev server 與 production-lite `speclink-server` 支援範圍 |
| 多 Agent 長 session 漂移 | session 切 repo、binding 過期或憑證撤銷後仍操作 | binding 固定且有 expiry；每次寫入重驗授權；切換 workspace 建新 session/rebind |
| 遠端離線語意模糊 | Client 可能把 stale snapshot 當可寫真相或建立難以合併的 outbox | v1 明定 snapshot 離線唯讀、無寫入 outbox；恢復連線後 refresh 再寫 |

### 15.3 Store 與事件的失敗模型

每個 adapter 必須文件化並接受測試的失敗情境：

- 寫入一半程序崩潰後，是否有 partial artifact/archive。
- DB commit 成功但 process 在 publish event 前崩潰。
- 兩個 actor 同時更新 tasks、proposal 或 canonical spec。
- SSE/WS 斷線、事件重複、事件亂序與 resume cursor 過期。
- NAS 暫時失聯、檔案鎖失效或 mtime 精度不足。
- Client 使用舊 Engine/schema/skill contract。

Event 消費者必須以 `eventId` 去重、以 revision 忽略舊事件，且永遠能透過 Query + ETag 重建狀態。事件是加速，不是正確性的唯一來源。

### 15.4 Lifecycle、approval 與 verify evidence

遠端多人流程至少需要以下可驗證事實：

```text
drafting
  -> review
  -> ready（approval 綁定 artifact revisions）
  -> applying（claim + appliedAgainstRevision）
  -> verified（verify evidence 綁定 code commit + spec revision）
  -> archived
```

- approval 後任何受保護 artifact 改動，都使 approval 失效並回到 review。
- apply 只能從 ready 開始，claim 必須原子化。
- verify 先從 Host 取得固定 spec/tasks/policy revisions 的 `VerifyBundle`，再於有 code checkout 的 RD/Agent 執行。
- Remote task evidence 以 stable task ID 保存 touched files、actor、repo 與 base/head commit；路徑清單只是驗證線索。
- verify 結果寫回 Host，附逐 task 結果、測試/audit 摘要、commit SHA、basis revisions、工具版本與 trust level。
- 任一 basis revision 已改變時 evidence 以 `stale_evidence` 拒絕，不得推進 verified gate。
- archive gate 檢查 tasks、有效 verify evidence、ownership、canonical base revisions，並以單一 Unit of Work 套用。

## 16. 明確非目標

- 不讓 Desktop、Agent 或第三方 UI 直接連 PostgreSQL/NAS 後寫規格。
- 不以可雙向寫入的本地鏡像取代遠端正典。
- 不要求本地 repo 模式部署 server、認證或事件通道。
- 不把 localhost bridge 或本地 Web UI 當成本地 repo 模式的建議方案；本地優先 Desktop/native UI。
- 不把 Speclink Desktop 定義成唯一 UI。
- 不把 SSE 綁進 Engine 或 Store；它只是 Event Bus 的遠端 transport adapter。
- 不為 CLI、MCP、Tool 各維護一套流程語意或 skills。
- 不把 SSE 或 WebSocket 當作正典或必要正確性地基；Query + ETag 才是遠端恢復路徑。

## 17. 文件狀態與後續同步

本文是新的整合架構基準。下列文件目前可作既有實作與歷史決策的參考，但不反向限制本文；待本藍圖定案後，
再依新的 Project/Repo binding、Context、transport、Host 與 UI contract 統一更新或取代：

- [架構說明](architecture.zh-TW.md)：目前 Engine、Store、呈現三層與既有行為保證。
- [Node SDK](sdk-node.zh-TW.md)：`@speclink/engine`、自訂 Store 與 `dispatch`。
- [團隊模式](team-mode.zh-TW.md)：`.speclink.yaml` remote mode、認證與 repo 綁定。
- [動詞契約](verb-contract.zh-TW.md)：目前遠端 REST 契約、payload 與錯誤正典。
