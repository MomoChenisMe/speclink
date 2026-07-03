# Speclink

以 Rust 重新實作的 Spec-Driven Development（SDD，規格驅動開發）CLI 引擎。以 [Spectra](https://github.com/kaochenlong/Spectra) 2.3.1 為藍本，與其達到 CLI 行為與輸出的位元級一致（byte-level parity），並在此基礎上做了數項刻意的功能延伸。

- **實作語言**：Rust（workspace：`speclink-core` 引擎 + `speclink-cli` 前端）
- **工作流**：`discuss? → propose → apply ⇄ ingest → verify? → archive`
- **相容對象**：Claude Code（`.claude/skills/`）與 Codex（`.agents/skills/` + `AGENTS.md`）

> Speclink 的機器可讀輸出（`--json`）、人眼互動輸出（含 ANSI 色彩）、技能內容與流程邏輯，除了下方「與 Spectra 的刻意差異」列出的項目外，與 Spectra 完全一致。

---

## 目錄

- [這是什麼](#這是什麼)
- [建置](#建置)
- [快速開始](#快速開始)
- [SDD 工作流](#sdd-工作流)
- [技能（skills）](#技能skills)
- [discuss：文件式討論](#discuss文件式討論)
- [CLI 指令參考](#cli-指令參考)
- [設定檔](#設定檔)
- [專案結構](#專案結構)
- [與 Spectra 的刻意差異](#與-spectra-的刻意差異)
- [開發與 parity 測試](#開發與-parity-測試)
- [設計緣起與 Roadmap](#設計緣起與-roadmap)

---

## 這是什麼

SDD 把「規格」當成開發的第一手真相：需求先寫成結構化的規格文件，程式碼再依規格實作。Speclink 管理兩類文件：

- **正典規格（canonical specs）** — `openspec/specs/<capability>/spec.md`，描述系統「現在」的行為，是唯一真相。
- **變更提案（change proposals）** — `openspec/changes/<name>/`，描述一次變更相對於正典的「差異（delta）」：新增（ADDED）、修改（MODIFIED）、移除（REMOVED）、更名（RENAMED）。實作完成後 `archive` 會把 delta 併入正典。

規格用固定的結構標記書寫，讓 CLI 能解析、驗證、注入追溯資訊：

```markdown
## ADDED Requirements

### Requirement: 使用者登入

系統 SHALL 允許使用者以電子郵件與密碼登入。密碼錯誤三次 MUST 鎖定帳號 15 分鐘。

#### Scenario: 成功登入

- **WHEN** 使用者輸入正確的電子郵件與密碼
- **THEN** 系統 SHALL 建立工作階段並導向首頁
```

`### Requirement:`、`#### Scenario:`、`- **WHEN**`/`- **THEN**` 與規範關鍵字 `SHALL`/`MUST` 是結構的一部分，永遠保持英文；散文內容的語言可由 `spec_locale` 設定（見[設定檔](#設定檔)）。

---

## 建置

需要 Rust 工具鏈（stable）。

```bash
git clone <this-repo>
cd speclink
cargo build --release
# 產物：target/release/speclink(.exe)
```

把 `target/release/` 加入 `PATH`，或直接以完整路徑呼叫。以下文件一律以 `speclink` 代稱。

---

## 快速開始

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

`init` 會建立：

- `openspec/`（`specs/`、`changes/archive/`、`config.yaml`）
- `.speclink.yaml`（應用層設定）
- 各工具的技能檔（`.claude/skills/speclink-*/SKILL.md` 或 `.agents/skills/`）與指令注入區塊（`CLAUDE.md` / `AGENTS.md`）
- `.gitignore` 區塊

在 AI 代理（Claude Code / Codex）中，直接用斜線指令 `/speclink-propose`、`/speclink-apply` 等驅動流程；技能內部會呼叫 `speclink` CLI 取得每一步的 instructions。

---

## SDD 工作流

```text
discuss?  →  propose  →  apply  ⇄  ingest  →  verify?  →  archive
```

| 階段 | 技能 | 什麼時候用 |
| --- | --- | --- |
| **discuss** | `/speclink-discuss` | 需求模糊、值得先辯清楚時（可選）。以蘇格拉底式問答進行，過程記錄成文件；結論可 `promote` 成一個 change。 |
| **propose** | `/speclink-propose` | 要規劃／設計一次變更。產出 proposal、delta specs、design、tasks 四類 artifact。`--from-discussion <slug>` 可從已結論的討論帶入。 |
| **apply** | `/speclink-apply` | 開始實作。逐條完成 tasks，`task done` 會記錄每個任務動到的檔案（touched）。 |
| **ingest** | `/speclink-ingest` | 實作到一半需求變了。更新 delta specs 與 tasks，不必打掉重來。 |
| **drift** | `/speclink-drift` | 變更擱置一陣子後恢復前先跑，偵測規格與程式碼是否已偏離。 |
| **verify** | `/speclink-verify` | 實作完成，確認程式碼真的符合規格（可選）。 |
| **archive** | `/speclink-archive` | 收尾。把 delta 併入正典規格、注入 `@trace` 追溯、快照以供還原、共同歸檔關聯的討論。 |

輔助技能：`/speclink-onboard`（在既有程式庫上導入 Speclink，從現況反推初始規格）、`/speclink-analyze`（檢查 artifact 一致性）、`/speclink-audit`（安全性審查）、`/speclink-commit`（只提交某個變更相關的檔案）。

---

## 技能（skills）

`init` 會為選定的工具產生 11 個技能。在 Claude Code / Codex 中以 `/speclink-<name>` 呼叫：

| 技能 | 說明 |
| --- | --- |
| `discuss` | 記錄並演進一場聚焦討論（Speclink 專屬的文件式討論） |
| `propose` | 建立含所有 artifact 的變更提案 |
| `onboard` | 在既有程式庫導入 Speclink，由現況產生初始規格 |
| `apply` | 實作或接續某個變更的 tasks |
| `ingest` | 依外部脈絡更新既有變更 |
| `drift` | 偵測變更與現況程式碼的偏離 |
| `verify` | 驗證實作是否符合 artifact |
| `analyze` | 分析 artifact 的一致性與缺口 |
| `audit` | 審查變更程式碼的安全性風險 |
| `archive` | 歸檔已完成的變更 |
| `commit` | 只提交某個變更相關的檔案 |

`.speclink.yaml` 的 `tools:` 清單決定產生哪幾套；用 `speclink update` 可依清單重新產生並清除已移除工具的殘留檔案。

---

## discuss：文件式討論

這是 Speclink 相對 Spectra 最主要的功能延伸。Spectra 的 discuss 是純技能、不留任何文件，討論到後面容易偏題。Speclink 把每一場討論落地成一份結構化文件（`openspec/discussions/<slug>.md`），流程邏輯與蘇格拉底式問答完全不變，但過程可被完整記錄、演進、最後轉成變更。

```bash
speclink discuss new hr-system                       # 建立討論文件（含固定骨架與撰寫規則）
speclink discuss context hr-system --stdin < ctx.md  # 設定 Context 段
speclink discuss add-round hr-system --mode socratic --stdin < round1.md   # 逐輪追加
speclink discuss add-round hr-system --mode socratic --stdin < round2.md
speclink discuss conclude hr-system --stdin < conclusion.md               # 結論（含決議/理由/排除方案/待議）
speclink discuss list                                # 列出討論（--archived 看已歸檔的）
speclink discuss show hr-system
speclink discuss promote hr-system                   # 轉成一個 change（proposal 由結論預填）
speclink discuss archive hr-system                   # 獨立歸檔（未 promote 的討論）
```

文件遵循四條規則（骨架中以註解說明）：每輪聚焦一個問題、只追加不改寫、明確記錄被排除的方案、結論必須解決或明確延後每一個未決問題。`promote` 後的討論會在該 change 歸檔時**自動一起歸檔**。

---

## CLI 指令參考

依用途分組。所有指令都支援 `--no-color`；多數支援 `--json` 供程式取用。

### 專案與設定

| 指令 | 用途 |
| --- | --- |
| `speclink init [PATH]` | 初始化專案。`--tools <claude,codex>` 明列工具（省略則自動偵測）；`--dir <DIR>` 自訂規格目錄（會寫入 `spec_dir`）；`--force` 覆寫 |
| `speclink update` | 依 `.speclink.yaml` 的 `tools:` 重新產生技能與注入區塊，並清除已移除工具的殘留 |
| `speclink config <get\|set\|unset\|list\|reset\|edit\|path>` | 全域設定（`%USERPROFILE%\AppData\Roaming\speclink\config.yaml`） |
| `speclink completion <generate\|install\|uninstall> <shell>` | shell 補全腳本 |

### 變更生命週期

| 指令 | 用途 |
| --- | --- |
| `speclink new change <name>` | 建立變更（kebab-case 名稱）。`--schema`、`--description` |
| `speclink new artifact <proposal\|design\|tasks\|spec> --change <name> [--stdin]` | 建立單一 artifact |
| `speclink list [--changes\|--specs] [--sort name\|modified\|created] [--json]` | 列出變更或正典規格 |
| `speclink show <item> [--item-type change\|spec] [--json]` | 檢視變更或規格內容 |
| `speclink status --change <name> [--json]` | 顯示 artifact 相依 DAG 的完成狀態 |
| `speclink instructions <artifact\|apply> --change <name> [--json]` | 取得某 artifact（或 apply 模式）的 instructions payload |
| `speclink task done <id> --change <name>` | 標記第 N 個任務完成並記錄 touched 檔案 |
| `speclink archive [name...] [-y] [--all]` | 歸檔。可多個名稱或 `--all` 批次；`--skip-specs`、`--no-validate`、`--mark-tasks-complete` |
| `speclink in-progress <...>` | 管理進行中標記 |

### 檢查與分析

| 指令 | 用途 |
| --- | --- |
| `speclink validate <name> [--all\|--specs\|--changes] [--strict] [--json]` | 結構驗證（重複 requirement 名、無操作 delta 等） |
| `speclink analyze <name> [--json]` | 四維度分析：Coverage / Consistency / Ambiguity / Gaps |
| `speclink drift <name> [--json]` | 偵測變更與現況程式碼的偏離（見[刻意差異](#與-spectra-的刻意差異)） |

### 其他

| 指令 | 用途 |
| --- | --- |
| `speclink schemas` / `speclink schema <show\|validate\|fork\|init>` | 工作流 schema 管理 |
| `speclink templates` | 顯示範本路徑 |
| `speclink demo` | 產生一個隨機主題的示範變更 |
| `speclink discuss <...>` | 文件式討論（見上節） |

任一指令加 `--help` 可看完整選項。

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

---

## 專案結構

```text
專案根/
├── .speclink.yaml                     # 應用層設定
├── CLAUDE.md / AGENTS.md              # 指令注入區塊（<!-- SPECLINK:START -->）
├── .claude/skills/speclink-*/         # Claude 技能（或 .agents/skills/）
├── openspec/
│   ├── config.yaml                   # 工作流設定
│   ├── specs/<capability>/spec.md    # 正典規格（唯一真相）
│   ├── changes/
│   │   ├── <name>/                   # 進行中的變更
│   │   │   ├── proposal.md
│   │   │   ├── design.md
│   │   │   ├── tasks.md
│   │   │   ├── specs/<cap>/spec.md   # delta 規格
│   │   │   └── .openspec.yaml        # 變更 metadata
│   │   └── archive/<date>-<name>/    # 已歸檔的變更
│   └── discussions/                  # 討論文件（Speclink 專屬）
│       └── archive/
└── .speclink/                        # CLI 工作狀態（touched、snapshots）
```

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

Speclink 以「對照 Spectra 2.3.1 二進位」的方式開發：任一差異都先在受控 fixture 上以雙二進位實測確認機制，再實作對齊。測試基建在 scratchpad：

- **parity_suite** — 31 項 CLI 輸出對照（brand 正規化後逐 byte 比對，drift 的刻意分歧經正規化層中性化）
- **color_suite** — 16 項 `CLICOLOR_FORCE=1` 下的 ANSI 色彩對照
- **twin harness** — 雙沙盒跑 8 個 drift 情境

完整的功能對照與每一項差異的機制說明見 [`docs/spectra-speclink-comparison.md`](docs/spectra-speclink-comparison.md)；SDD 全流程實測報告見 [`docs/sdd-final-report-hr.md`](docs/sdd-final-report-hr.md)。

---

## 設計緣起與 Roadmap

### 緣起

Speclink 源自對 Spectra 與 OpenSpec 的比較分析（見 [`Spectra-OpenSpec-SDD-完整功能邏輯分析.md`](Spectra-OpenSpec-SDD-完整功能邏輯分析.md)），目標是保留兩者的優點、以 Rust 重寫，並延伸更進階的設計。第一階段（已完成）是做出與 Spectra 行為一致的完整 CLI，再疊加上述刻意差異。

### 願景：規格驅動引擎

目前不論 OpenSpec 或 Spectra，規格文件都綁在 git 儲存庫上。OpenSpec 雖有 store 的概念，但比較像是把規格抽離出來。Speclink 想更進一步——提供一套**規格驅動引擎**的抽象：文件怎麼存放、管理，由使用者自己決定（寫成 Markdown、存進資料庫、存成 JSON/YAML、串接自家系統或 JIRA 皆可），引擎只負責 SDD 的流程邏輯。

理想的最終形態是把角色與儲存解耦：

- **PO／PM** 在客製化系統中執行 `discuss + propose + ingest + archive`（規劃與規格管理）
- **RD／QA** 在本地 git 儲存庫中執行 `apply + verify`（實作與驗證）

兩端共用同一套規格驅動引擎，但各自選擇最適合的儲存與介面。這是 Speclink 相對於「規格必須跟著 git」的既有工具最想突破的方向。
