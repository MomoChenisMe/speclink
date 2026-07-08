<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>規格驅動開發（SDD）平台</b> — 一顆 Rust 引擎，多種前端。
</p>

<p align="center">
  <b>繁體中文</b> · <a href="README.en.md">English</a>
</p>

---

Speclink 把「規格」當成開發的第一手真相：需求先寫成結構化規格，程式碼再依規格實作。核心是一顆以 Rust 重寫的 SDD 引擎——以 [Spectra](https://github.com/kaochenlong/Spectra) 2.3.1 為藍本，與其達到 CLI 行為與輸出的**位元級一致（byte-level parity）**，並在此基礎上做了數項刻意的功能延伸。同一顆引擎被 CLI、桌面 app、Node SDK 各自內嵌，並可將規格真相接到本地檔案或團隊系統。

- **實作語言**：Rust（引擎）+ TypeScript／React（桌面前端）
- **相容對象**：Claude Code（`.claude/skills/`）與 Codex（`.agents/skills/` + `AGENTS.md`）
- **工作流**：`discuss? → propose → apply ⇄ ingest → verify? → archive`
- **授權**：MIT

> Speclink 的機器可讀輸出（`--json`）、人眼互動輸出（含 ANSI 色彩）、技能內容與流程邏輯，除了[「與 Spectra 的刻意差異」](#與-spectra-的刻意差異)列出的項目外，與 Spectra 完全一致。

---

## 目錄

- [平台總覽](#平台總覽)
- [引擎與 SDD 核心](#引擎與-sdd-核心)
- [SDD 工作流](#sdd-工作流)
- [CLI](#cli)
- [桌面 app](#桌面-app)
- [Node SDK（@speclink/engine）](#node-sdkspeclinkengine)
- [團隊模式（遠端 store）](#團隊模式遠端-store)
- [設定檔](#設定檔)
- [與 Spectra 的刻意差異](#與-spectra-的刻意差異)
- [開發與 parity 測試](#開發與-parity-測試)
- [文件](#文件)
- [願景與 Roadmap](#願景與-roadmap)

---

## 平台總覽

一顆引擎、一套 Store 縫線、多個前端：

```text
   前端 / 宿主   ┌───────────┬──────────────┬────────────┐
                │    CLI     │   桌面 app    │  Node SDK   │
                └───────────┴──────┬───────┴────────────┘
                                   │  內嵌同一顆引擎
   引擎          ┌────────────────▼─────────────────┐
                │            speclink-core           │
                │       SDD 流程邏輯 · 呈現           │
                └────────────────┬─────────────────┘
                                   │  Store 縫線（seam）
   儲存          ┌────────────────▼─────────────────┐
                │   speclink-fs        speclink-remote │
                │   本地 markdown      團隊系統 REST    │
                └──────────────────────────────────┘
```

| 元件 | crate／套件 | 一句話 |
| --- | --- | --- |
| **引擎** | `speclink-core` | SDD 流程邏輯與呈現的唯一真相，以 Rust 實作 |
| **本地儲存** | `speclink-fs` | 預設 Store——`openspec/` 下的 markdown 即真相 |
| **遠端儲存** | `speclink-remote` | 團隊模式 Store——[verb contract](docs/verb-contract.md) REST 薄 client |
| **CLI** | `speclink-cli`（`speclink`） | 命令列前端，人眼互動輸出 + `--json` 機器輸出 |
| **桌面 app** | `@speclink/desktop`（Tauri） | 內嵌 core 的生命週期看板 GUI |
| **Node SDK** | `@speclink/engine`（`speclink-node`） | 把引擎嵌進 Node 行程（[napi-rs](https://napi.rs) 綁定） |
| **共用 UI** | `@speclink/ui` | 桌面前端的 React 元件庫 |

三者共用引擎的意義：不論你用 CLI、桌面 app 還是自家 Node 服務驅動 SDD，動詞行為、`--json` 形狀與產生的技能／指令內容都由同一份 Rust 程式碼決定，一致由構造保證。

---

## 引擎與 SDD 核心

Speclink 管理兩類文件：

- **正典規格（canonical specs）** — `openspec/specs/<capability>/spec.md`，描述系統「現在」的行為，是唯一真相。
- **變更提案（change proposals）** — `openspec/changes/<name>/`，描述一次變更相對於正典的「差異（delta）」：新增（ADDED）、修改（MODIFIED）、移除（REMOVED）、更名（RENAMED）。實作完成後 `archive` 會把 delta 併入正典。

規格用固定的結構標記書寫，讓引擎能解析、驗證、注入追溯資訊：

```markdown
## ADDED Requirements

### Requirement: 使用者登入

系統 SHALL 允許使用者以電子郵件與密碼登入。密碼錯誤三次 MUST 鎖定帳號 15 分鐘。

#### Scenario: 成功登入

- **WHEN** 使用者輸入正確的電子郵件與密碼
- **THEN** 系統 SHALL 建立工作階段並導向首頁
```

`### Requirement:`、`#### Scenario:`、`- **WHEN**`/`- **THEN**` 與規範關鍵字 `SHALL`/`MUST` 是結構的一部分，永遠保持英文；散文內容的語言可由 `spec_locale` 設定（見[設定檔](#設定檔)）。

引擎本身分三層——**引擎 → Store → 呈現**。真相的存放由 Store 縫線抽象：`speclink-fs` 把 markdown 檔案當真相（預設），`speclink-remote` 則把真相移到團隊系統、以 REST 契約存取。前端（CLI／桌面／SDK）只跟引擎對話，不直接碰儲存。三層與縫線的細節見 [docs/architecture.md](docs/architecture.md)。

---

## SDD 工作流

```text
discuss?  →  propose  →  apply  ⇄  ingest  →  verify?  →  archive
```

不論在 CLI 或桌面 app，都由這套流程驅動；在 AI 代理中以 `/speclink-<name>` 斜線指令呼叫，技能內部會透過引擎取得每一步的 instructions。

| 階段 | 技能 | 什麼時候用 |
| --- | --- | --- |
| **discuss** | `/speclink-discuss` | 需求模糊、值得先辯清楚時（可選）。以蘇格拉底式問答進行，過程記錄成文件；結論可 `promote` 成一個 change。 |
| **propose** | `/speclink-propose` | 要規劃／設計一次變更。產出 proposal、delta specs、design、tasks 四類 artifact。`--from-discussion <slug>` 可從已結論的討論帶入。 |
| **apply** | `/speclink-apply` | 開始實作。逐條完成 tasks，`task done` 會記錄每個任務動到的檔案（touched）。 |
| **ingest** | `/speclink-ingest` | 實作到一半需求變了。更新 delta specs 與 tasks，不必打掉重來。 |
| **drift** | `/speclink-drift` | 變更擱置一陣子後恢復前先跑，偵測規格與程式碼是否已偏離。 |
| **verify** | `/speclink-verify` | 實作完成，確認程式碼真的符合規格（可選）。 |
| **archive** | `/speclink-archive` | 收尾。把 delta 併入正典規格、注入 `@trace` 追溯、快照以供還原、共同歸檔關聯的討論。 |

輔助技能：`/speclink-onboard`（在既有程式庫上導入 Speclink，從現況反推初始規格）、`/speclink-analyze`（檢查 artifact 一致性）、`/speclink-audit`（安全性審查）、`/speclink-commit`（只提交某個變更相關的檔案）。`init` 會為選定的工具產生 11 個技能。

### discuss：文件式討論

這是 Speclink 相對 Spectra 最主要的功能延伸。Spectra 的 discuss 是純技能、不留任何文件，討論到後面容易偏題。Speclink 把每一場討論落地成一份結構化文件（`openspec/discussions/<slug>.md`），流程邏輯與蘇格拉底式問答完全不變，但過程可被完整記錄、演進、最後轉成變更。

文件遵循四條規則：每輪聚焦一個問題、只追加不改寫、明確記錄被排除的方案、結論必須解決或明確延後每一個未決問題。`promote` 後的討論會在該 change 歸檔時**自動一起歸檔**（一場討論可扇出多個 change）。討論文件**不會在開場就建立**：直到第一個實質回合才落地，誤觸或一句話答完的話題不留檔案；聊到一半發現不需要的用 `discard` 清掉——「決定不做」也是值得 `conclude` + `archive` 保留的結論。

---

## CLI

命令列前端，也是導入 Speclink 的入口。需要 Rust 工具鏈（stable）。

```bash
git clone <this-repo>
cd speclink
cargo build --release
# 產物：target/release/speclink(.exe)
```

把 `target/release/` 加入 `PATH`，或以完整路徑呼叫。以下一律以 `speclink` 代稱。

### 快速開始

```bash
# 1. 在專案根目錄初始化（自動偵測 .claude/ 或 .agents/AGENTS.md 決定產生哪套技能）
speclink init

# 2. 想直接看一個範例變更長什麼樣子：
speclink demo            # 產生一個隨機主題的示範 change
speclink list            # 列出目前的變更
speclink show <name>     # 檢視某個變更的內容

# 3. 檢查與歸檔
speclink validate <name> --strict
speclink analyze <name>
speclink archive <name> -y
```

`init` 會建立 `openspec/`（`specs/`、`changes/archive/`、`config.yaml`）、`.speclink.yaml`（應用層設定）、各工具的技能檔與指令注入區塊（`CLAUDE.md`／`AGENTS.md`）、以及 `.gitignore` 區塊。

### 指令參考

依用途分組。所有指令都支援 `--no-color`；多數支援 `--json` 供程式取用。任一指令加 `--help` 可看完整選項。

**專案與設定**

| 指令 | 用途 |
| --- | --- |
| `speclink init [PATH]` | 初始化專案。`--tools <claude,codex>` 明列工具（省略則自動偵測）；`--dir <DIR>` 自訂規格目錄；`--store remote` 直接以團隊模式初始化；`--force` 覆寫 |
| `speclink link` / `speclink unlink` | 綁定／解除既有 repo 到團隊系統（見[團隊模式](#團隊模式遠端-store)） |
| `speclink auth <login\|status\|logout>` | 團隊模式認證 |
| `speclink update` | 依 `.speclink.yaml` 的 `tools:` 重新產生技能與注入區塊，並清除已移除工具的殘留 |
| `speclink config <get\|set\|unset\|list\|reset\|edit\|path>` | 全域設定 |
| `speclink completion <generate\|install\|uninstall> <shell>` | shell 補全腳本 |

**變更生命週期**

| 指令 | 用途 |
| --- | --- |
| `speclink new change <name>` | 建立變更（kebab-case）。`--schema`、`--description` |
| `speclink new artifact <proposal\|design\|tasks\|spec> --change <name> [--stdin]` | 建立單一 artifact |
| `speclink list [--changes\|--specs] [--sort name\|modified\|created] [--json]` | 列出變更或正典規格 |
| `speclink show <item> [--item-type change\|spec] [--json]` | 檢視變更或規格內容 |
| `speclink status --change <name> [--json]` | 顯示 artifact 相依 DAG 的完成狀態 |
| `speclink instructions <artifact\|apply> --change <name> [--json]` | 取得某 artifact（或 apply 模式）的 instructions payload |
| `speclink task done <id> --change <name>` | 標記第 N 個任務完成並記錄 touched 檔案 |
| `speclink archive [name...] [-y] [--all]` | 歸檔。可多個名稱或 `--all` 批次；`--skip-specs`、`--no-validate`、`--mark-tasks-complete` |
| `speclink in-progress <...>` | 管理進行中標記 |
| `speclink discuss <...>` | 文件式討論（`new`／`context`／`add-round`／`conclude`／`promote`／`archive`／`discard` 等，見上節） |

**檢查與分析**

| 指令 | 用途 |
| --- | --- |
| `speclink validate <name> [--all\|--specs\|--changes] [--strict] [--json]` | 結構驗證（重複 requirement 名、無操作 delta 等） |
| `speclink analyze <name> [--json]` | 四維度分析：Coverage／Consistency／Ambiguity／Gaps |
| `speclink drift <name> [--json]` | 偵測變更與現況程式碼的偏離（見[刻意差異](#與-spectra-的刻意差異)） |

**其他**：`speclink schemas` / `schema <show\|validate\|fork\|init>`（schema 管理）、`speclink templates`（範本路徑）、`speclink demo`（示範變更）。

---

## 桌面 app

`@speclink/desktop` 是一個 [Tauri](https://tauri.app) 應用，**直接內嵌 `speclink-core`**（非 spawn CLI 子進程），在本地 `openspec/` 專案上運作。它以生命週期看板呈現變更（討論／提案中／進行中／已就緒），支援多專案分頁、詳情抽屜與互動任務，並監看檔案系統——外部的 CLI、agent、編輯器改動 `openspec/` 後看板即時更新。markdown 檔案始終是真相，app 不把任何文件真相移出檔案系統。

```bash
npm install                      # 於 repo 根安裝 workspace 相依
npm run tauri dev -w apps/desktop     # 開發模式（熱更新）
npm run tauri build -w apps/desktop   # 打包桌面安裝檔
```

僅重建前端可用 `npm run build -w apps/desktop`（vite → `dist`）；僅重建原生殼可用 `cargo build --release -p speclink-desktop`。前端測試：`npm test -w apps/desktop`、`npm test -w packages/ui`。詳細行為規格見 `openspec/specs/desktop-app/`。

---

## Node SDK（@speclink/engine）

`@speclink/engine` 把 Speclink 引擎嵌進 Node.js 行程：你的伺服器（或 AI 代理宿主）在行程內 dispatch speclink 動詞、透過自訂 `Store` 把規格文件存進自家資料庫，並為所在的 harness 渲染工作流知識（技能、指令區塊）。它是 CLI 內建的同一顆 Rust 引擎——以 napi-rs 綁定、非重新實作——動詞行為與 `--json` 形狀由構造保證一致。

```bash
npm install @speclink/engine
```

原生模組，預建二進位以 `optionalDependencies` 隨附，`npm install` 在五個支援平台（Windows x64、macOS x64／arm64、Linux x64／arm64 glibc）即裝即用、免工具鏈。

```js
const { createEngine } = require('@speclink/engine')

// 形式一：內建 fs store，指向本地專案根
const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })

// 形式二：自訂 Store（例如接 Postgres），引擎透過它讀寫文件
const engine = createEngine({ store: myStore })
```

完整的 Store 介面、dispatch 契約與渲染 API 見 [docs/sdk-node.md](docs/sdk-node.md)。

---

## 團隊模式（遠端 store）

在團隊模式下，規格文件與變更狀態存在**團隊系統**（一台內嵌 Speclink 引擎的伺服器），你的程式碼與 git 留在本地。`speclink` CLI 變成 [verb contract](docs/verb-contract.md) 的薄 client：你已經在用的每個動詞（`list`、`status`、`instructions`、`task done`、`discuss …`）輸出形狀不變，只有背後的儲存搬家。

模式由 `.speclink.yaml` 的一個區段決定：

```yaml
# .speclink.yaml —— 會提交，如同 .lfsconfig：每次 clone 都拿到相同綁定
tools:
  - claude
remote:
  url: https://team.example.com/api/speclink/v1/projects/erp   # 專案作用域
  repo: backend    # 單一 repo 專案可省略——此 repo 註冊名
```

無 `remote:` 區段＝fs 模式（不變）；有＝remote 模式。憑證絕不寫進此檔。

```bash
# 全新 repo 直接以團隊模式初始化（不建 openspec/，文件在伺服器）
speclink init --store remote --url <project-url> --repo backend

# 既有 repo 綁定／解除
speclink link <project-url> --repo backend
speclink unlink

# 認證
speclink auth login
speclink auth status
```

`SPECLINK_STORE_URL` 可為單一 shell 或 CI job 覆寫 url。client 側連接、認證、repo 識別與錯誤對照見 [docs/team-mode.md](docs/team-mode.md)。

這正是 Speclink 想突破的方向——把**角色與儲存解耦**：PO／PM 在客製化系統中執行 `discuss + propose + ingest + archive`，RD／QA 在本地 git 儲存庫中執行 `apply + verify`，兩端共用同一顆引擎、各選最適合的儲存與介面。

---

## 設定檔

### `.speclink.yaml`（應用層）

`init` 產生，含說明註解。主要欄位：

```yaml
# 產出物語言（AI 產生的 proposal/design/tasks 等），預設英文
# locale: tw

# 規格檔語言（specs/*/spec.md 的散文），預設英文；"auto" 跟隨 locale
# 結構標記與 SHALL/MUST 一律維持英文
# spec_locale: tw

# 工作流紀律開關，預設關閉
# tdd: true      # apply 時採測試先行紀律
# audit: true    # apply 時內嵌安全審查紀律

# init 產生技能的工具清單（驅動 update 的同步與清理）
tools:
  - claude
  - codex

# remote: 區段存在時切換為團隊模式（見上節）
```

- `locale` / `spec_locale` 支援 `tw`（繁體中文）、`ja`（日本語）、`en`／未設定（英文）等。設 `tw`/`zh*` 時，specs 的 instructions 會額外注入中文弱詞警示，且 analyzer 會偵測「應該、也許、考慮、待定、可能……」等中文弱語言（比照英文的 should/may/TBD）。
- `tdd` / `audit` 是 apply 技能內嵌的紀律，不是獨立技能。

### `openspec/config.yaml`（工作流層）

```yaml
schema: spec-driven

# 專案脈絡（建立 artifact 時提供給 AI）
context: |
  技術棧、慣例、領域知識……

# 各 artifact 的自訂規則
rules:
  proposal:
    - 一定要有「Non-goals」段落
```

四層解析、工具描述子與遷移指引見 [docs/configuration.md](docs/configuration.md)。

---

## 與 Spectra 的刻意差異

Speclink 對 Spectra 的偏離全部是刻意設計，其餘一律保持位元級一致。四項結構性分歧：

1. **持久化 discuss** — 討論落地成可演進的文件並可 `promote` 成變更（見上節）。
2. **drift 增強** — Spectra 的 drift 有數個失真點，Speclink 修正並強化：
   - `--since` 錨定到當日午夜（Spectra 用裸日期，git approxidate 會讓當日變更恆算 0 次提交）
   - anchor 擷取只收 code-like token（camelCase／snake_case／多段 PascalCase），散文大寫詞不再誤報；反引號路徑改為存在性檢查（File anchor）
   - anchor 搜尋語料排除變更自身目錄（Spectra 的語料含自己，導致已提交的 design 永遠自我滿足）
   - 新增 **Specs 維度**與 `spec_assumptions`：偵測 delta 的正典目標已被改寫（MODIFIED/REMOVED/RENAMED 目標不存在、ADDED 目標已存在），這類「歸檔會靜默略過」的情形一律導向 `ingest`
   - Tasks 維度給出真實信號（依 task 的檔案引用 × 提交窗口判斷「可能已完成／被外部變更阻擋」）
3. **audit 雙模式** — 重寫為 standalone（三代理平行分析）與 apply 內嵌 discipline 兩種模式，因此不是 fork 技能。
4. **RENAMED 實際執行** — Spectra 記載 `## RENAMED Requirements` 但任何語法都不執行、`renamed:` 恆 0；Speclink 於歸檔時真正改寫正典 requirement 標頭並計數，rename-only 的變更也能通過驗證與歸檔。

其他延伸：`spec_locale` 規格語言設定與中文弱語言偵測、`onboard` 導入技能、`archive` 批次歸檔（多名稱／`--all`，含乾淨工作樹強制、逐項略過附原因、fail-fast）、`init` 工具自動偵測、`update` 的同步與無足跡清理、MODIFIED 的 `<!-- BEFORE: -->` 前值註記（歸檔時剝除）。

移除的 Spectra 功能：`ask`、`debug`、向量搜尋（`search`）、`worktree`、`park`/`unpark`、`parallel_tasks`、`claude_effort`。工具範圍限定 claude + codex。

---

## 開發與 parity 測試

Speclink 以「對照 Spectra 2.3.1 二進位」的方式開發：任一差異都先在受控 fixture 上以雙二進位實測確認機制，再實作對齊。

- **parity_suite** — 31 項 CLI 輸出對照（brand 正規化後逐 byte 比對，drift 的刻意分歧經正規化層中性化）
- **color_suite** — 16 項 `CLICOLOR_FORCE=1` 下的 ANSI 色彩對照
- **twin harness** — 雙沙盒跑 8 個 drift 情境

完整的功能對照與每一項差異的機制說明見 [docs/spectra-speclink-comparison.md](docs/spectra-speclink-comparison.md)；SDD 全流程實測報告見 [docs/sdd-final-report-hr.md](docs/sdd-final-report-hr.md)。

---

## 文件

| 主題 | English | 繁體中文 |
|---|---|---|
| 架構說明（引擎—Store—呈現三層、儲存縫線） | [docs/architecture.md](docs/architecture.md) | [docs/architecture.zh-TW.md](docs/architecture.zh-TW.md) |
| 入門教學（純本地走完一輪 SDD） | [docs/getting-started.md](docs/getting-started.md) | [docs/getting-started.zh-TW.md](docs/getting-started.zh-TW.md) |
| 設定說明（兩檔一目錄體系、四層解析、工具描述子、遷移指引） | [docs/configuration.md](docs/configuration.md) | [docs/configuration.zh-TW.md](docs/configuration.zh-TW.md) |
| 團隊模式（連接檔、init/link/auth、repo 識別、錯誤對照、升級指引） | [docs/team-mode.md](docs/team-mode.md) | [docs/team-mode.zh-TW.md](docs/team-mode.zh-TW.md) |
| 動詞契約（remote store 的 REST 契約正典：端點、payload、409 語意） | [docs/verb-contract.md](docs/verb-contract.md) | [docs/verb-contract.zh-TW.md](docs/verb-contract.zh-TW.md) |
| Node SDK（@speclink/engine：createEngine 兩形式、Store 橋接、dispatch 契約、渲染 API） | [docs/sdk-node.md](docs/sdk-node.md) | [docs/sdk-node.zh-TW.md](docs/sdk-node.zh-TW.md) |
| 品牌資產（Logo、配色、使用指引） | [docs/assets/brand/README.md](docs/assets/brand/README.md) | — |

---

## 願景與 Roadmap

### 緣起

Speclink 源自對 Spectra 與 OpenSpec 的比較分析（見 [`Spectra-OpenSpec-SDD-完整功能邏輯分析.md`](Spectra-OpenSpec-SDD-完整功能邏輯分析.md)），目標是保留兩者的優點、以 Rust 重寫，並延伸更進階的設計。第一階段（已完成）是做出與 Spectra 行為一致的完整 CLI，再疊加上述刻意差異。

### 規格驅動引擎

目前不論 OpenSpec 或 Spectra，規格文件都綁在 git 儲存庫上。Speclink 想更進一步——提供一套**規格驅動引擎**的抽象：文件怎麼存放、管理由使用者自己決定（寫成 Markdown、存進資料庫、串接自家系統或 JIRA 皆可），引擎只負責 SDD 的流程邏輯。

這個方向已在落地中——`speclink-core` 的 Store 縫線、`speclink-remote` 的[團隊模式](#團隊模式遠端-store)、與 [`@speclink/engine`](#node-sdkspeclinkengine) Node SDK，正是「規格不必跟著 git」的第一批成果。桌面 app 則是這顆引擎的第一個 GUI 前端；更多前端（含 web）在規劃中。
