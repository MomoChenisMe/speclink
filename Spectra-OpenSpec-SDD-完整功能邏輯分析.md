# Spectra 與 OpenSpec：規格驅動開發（SDD）完整功能邏輯分析

> 本文件是針對使用者系統內安裝的 **Spectra CLI（Rust 編寫，版本 2.3.1）** 進行行為重現與反組譯分析，搭配隨附的 `spectra-*` 技能（skills），再對照其前身開源專案 **OpenSpec（Fission-AI）** 的功能，整理而成的一份完整技術說明。
>
> **研究方法**：直接執行 `spectra.exe` 的各種唯讀指令與 `--help`、在臨時專案中做受控實驗（init → demo → propose → apply → archive 全流程觀察）、萃取 8.3MB 二進位檔的字串（約 11 萬筆）做靜態分析、完整閱讀 12 個技能檔、抓取 OpenSpec 的 GitHub 原始碼與文件，最後以多代理對抗式交叉驗證所有結論。文末附「證據與方法」一節。
>
> **閱讀提示**：SDD 有很多專有名詞。第 1 章是「白話辭典」，看不懂任何術語都可以先回去查。每個術語第一次出現時也會就地解釋。程式碼識別字、指令、檔名一律保留原文。

---

## 目錄

1. [先看這個：SDD 白話辭典（專有名詞解釋）](#1-先看這個sdd-白話辭典專有名詞解釋)
2. [一句話總覽：Spectra 與 OpenSpec 是什麼、什麼關係](#2-一句話總覽spectra-與-openspec-是什麼什麼關係)
3. [Spectra 的三個組成元件](#3-spectra-的三個組成元件)
4. [核心資料模型：openspec/ 目錄、正典規格 vs. 變更](#4-核心資料模型openspec-目錄正典規格-vs-變更)
5. [產物（artifact）格式詳解](#5-產物artifact格式詳解)
6. [【重點】三層設定系統與 instruction 注入時機](#6-重點三層設定系統與-instruction-注入時機)
7. [Spectra CLI 完整命令參考](#7-spectra-cli-完整命令參考)
8. [工作流 schema 與產物相依圖（DAG）](#8-工作流-schema-與產物相依圖dag)
9. [12 個技能逐一詳解](#9-12-個技能逐一詳解)
10. [技能 × CLI 搭配總表](#10-技能--cli-搭配總表)
11. [進階引擎內部：analyze／drift／preflight／archive／向量搜尋／worktree／資料庫](#11-進階引擎內部)
12. [OpenSpec 功能與 CLI 完整參考](#12-openspec-功能與-cli-完整參考)
13. [Spectra vs. OpenSpec 對照表](#13-spectra-vs-openspec-對照表)
14. [端到端範例：一次完整的變更生命週期](#14-端到端範例一次完整的變更生命週期)
15. [已知細節、平台限制與隱藏功能](#15-已知細節平台限制與隱藏功能)
16. [證據與方法](#16-證據與方法)

---

## 1. 先看這個：SDD 白話辭典（專有名詞解釋）

| 術語 | 白話解釋 |
|---|---|
| **SDD（Spec-Driven Development，規格驅動開發）** | 一種開發方法：**先把「要做什麼」寫成一份規格文件，人和 AI 都同意了，才開始寫程式**。目的是避免「需求只存在於聊天記錄裡」造成 AI 亂寫、做錯方向。相對於傳統「邊聊邊寫」。 |
| **spec（規格）** | 描述「系統應該有什麼行為」的文件。注意：它寫的是**行為**（做什麼），不是**實作**（怎麼做）。 |
| **artifact（產物）** | SDD 流程中產生的各種文件，每一份都叫一個 artifact。Spectra 的四種產物是 `proposal`、`specs`、`design`、`tasks`（下面解釋）。 |
| **proposal（提案）** | 一份變更的「為什麼要做、要改什麼」文件。回答 Why。 |
| **design（設計）** | 「要怎麼實作」的技術設計文件。回答 How。可選，簡單變更可略過。 |
| **tasks（任務清單）** | 把實作工作拆成一條條可勾選的待辦清單（`- [ ]`）。是追蹤進度的唯一依據。 |
| **capability（能力）** | 系統的一塊功能領域，例如 `user-auth`（使用者驗證）、`snapshot-restore`（快照還原）。每個 capability 對應一個 spec 檔。名稱用 kebab-case（小寫加連字號）。 |
| **requirement（需求）** | spec 裡的一條規範，例如「系統 SHALL 在登入成功後發出 JWT token」。 |
| **scenario（情境）** | 需求底下的具體案例，用 `WHEN（當…）/ THEN（則…）` 描述，類似測試案例。 |
| **SHALL / MUST / SHOULD / MAY** | 來自網路標準 **RFC 2119** 的規範用語。`SHALL`/`MUST` = 絕對必須；`SHOULD` = 建議；`MAY` = 可選。SDD 要求需求一律用 `SHALL`/`MUST`（強制語氣），不准用模糊的 should/may，因為規格必須明確、可驗證。 |
| **WHEN / THEN（／GIVEN）** | 行為描述法（源自 BDD「行為驅動開發」的 Given-When-Then）。`GIVEN` 前提、`WHEN` 觸發條件、`THEN` 預期結果。 |
| **SBE（Specification by Example，以實例說明規格）** | 用「具體的數值例子」來說明一條規格，避免抽象。例如「輸入 A(0.9)、B(0.3)、C(0.7) → 輸出排序為 A、C、B」。在 Spectra 裡用 `##### Example:`（五個井字號）標示。 |
| **delta（差異／增量）** | 「這次變更**相對於現狀改了什麼**」。一個變更的 spec 只寫 delta（新增/修改/刪除/改名了哪些需求），不重抄整份規格。這是 OpenSpec/Spectra 的核心設計。 |
| **正典規格 / 真實來源（source of truth）** | 放在 `openspec/specs/` 的規格，代表「系統現在實際長怎樣」。變更被歸檔（archive）時，它的 delta 會被合併進正典規格。 |
| **schema（工作流綱要）** | 定義「一個變更需要哪些產物、產物之間的先後順序」的設定檔。預設 schema 叫 `spec-driven`。 |
| **DAG（Directed Acyclic Graph，有向無環圖）** | 「有方向、不會繞回來的相依關係圖」。Spectra 用它表示產物的相依：proposal 完成才能做 specs/design，specs 完成才能做 tasks。 |
| **archive（歸檔）** | 變更實作完成後的收尾動作：把 delta 規格合併進正典規格，並把整個變更資料夾移到 `archive/` 留存。 |
| **drift（漂移）** | 「變更計畫」和「目前程式碼」對不上的程度。例如變更建立後很久沒動、設計文件提到的函式已經不存在了。Spectra 有專門指令量化 drift。 |
| **anchor（錨點）** | 產物（尤其 design）裡提到的程式碼識別字（檔名、函式名、符號）。drift 檢查會去程式庫找這些 anchor 還在不在，找不到就叫「broken anchor（斷掉的錨點）」。 |
| **preflight（起飛前檢查）** | apply（開始實作）前的自動健檢：產物引用的檔案還在嗎？變更建立後檔案被改過嗎？變更放太久了嗎？ |
| **@trace（追溯註解）** | 歸檔時自動寫進正典規格的一段 HTML 註解，記錄「這條需求是哪個變更帶來的、哪天更新、改動了哪些程式檔」，方便日後追溯。 |
| **park / unpark（暫存／取回）** | Spectra 特有。把一個變更「暫時收起來」移出 `openspec/changes/`，不會出現在 `spectra list`，需要時再 `unpark` 取回。propose 完成後會自動 park，等你準備好 apply 時再自動 unpark。 |
| **worktree（工作樹）** | Git 功能：在同一個 repo 開出另一個獨立的工作目錄＋分支。Spectra 可選擇讓每個變更在自己的 worktree／分支裡實作，互不干擾。 |
| **向量語意搜尋（vector / embedding search）** | 用 AI 把文件轉成數字向量，依「語意相近」而非「關鍵字相同」來搜尋。能跨語言（中英日）。Spectra 的此功能**只在 Apple M 系列晶片的 Mac 上可用**，Windows 不支援。 |
| **instruction（指令／指示文）** | Spectra CLI 針對某個產物，吐給 AI 看的「該怎麼寫這份產物」的指導文＋空白模板＋專案情境。技能會呼叫 `spectra instructions <產物>` 取得它。這是本文第 6 章的重點。 |
| **skill（技能）** | 一份寫給 AI 助理照著做的工作流程說明書（Markdown）。使用者打 `/spectra-propose` 之類的斜線指令就會觸發對應技能。技能本身不是程式，是「給 AI 的劇本」。 |
| **fork context / Explore agent** | 技能的一種執行模式。「fork」= 開一個隔離的子代理去跑，不污染主對話；「Explore」= 唯讀探索型代理；搭配 `disallowedTools: [Edit, Write]` 表示這個技能**只能讀不能改檔案**。Spectra 把分析類技能（analyze、ask、audit、drift、verify、discuss）都設成這種唯讀 fork。 |
| **locale（語系）** | 設定 AI 生成產物要用哪種語言。`tw` = 繁體中文、`ja` = 日文。但**規格 spec 永遠用英文**（因為 SHALL/MUST/WHEN/THEN 是規範語言）。 |
| **TDD（Test-Driven Development，測試驅動開發）** | 「先寫一個會失敗的測試，再寫程式讓它通過」的開發紀律（紅→綠→重構）。Spectra 的 `.spectra.yaml` 有 `tdd: true` 開關。 |

---

## 2. 一句話總覽：Spectra 與 OpenSpec 是什麼、什麼關係

- **OpenSpec** 是 **Fission-AI** 開源的「規格驅動開發」工具，用 **TypeScript/Node.js** 寫成，透過 npm 套件 `@fission-ai/openspec` 安裝（需 Node.js ≥ 20.19）。GitHub 上約 5.7 萬顆星、MIT 授權、2025-08-05 建立。它的口號是「**Spec-driven development (SDD) for AI coding assistants**」——讓 AI 寫程式前先和你談好規格。

- **Spectra** 是把 OpenSpec 的概念**用 Rust 重新打造、再加料**的版本。它沿用了 OpenSpec 的整個資料模型（`openspec/` 目錄、`specs/` 正典、`changes/` 變更、delta 規格格式、proposal→apply→archive 主幹），但：
  - 改用單一原生執行檔 `spectra.exe`（不需要 Node.js）。
  - 多了一個**桌面 GUI App**（`app.exe`，Tauri 框架，27MB）。
  - 把工作流包裝成一整套 **`/spectra-*` 技能**（劇本），讓 AI 助理照著跑。
  - 新增了一堆 OpenSpec 沒有的東西：**park/unpark（暫存）**、**drift（漂移偵測）**、**analyze（4 維一致性分析）**、**audit（安全稽核）**、**debug（除錯紀律）**、**commit（單一變更提交）**、向量語意搜尋、git worktree、`.spectra.yaml` 的 tdd/audit/parallel_tasks 等開關、locale 多語系。

> **作者線索**：二進位檔內的 feedback 端點是 `https://github.com/kaochenlong/Spectra/issues`，建置路徑是 `/Users/kaochenlong/.cargo/...`，而 `.spectra.yaml` 標頭指向 `github.com/spectra-app/spectra`。可推斷 Spectra 由 **kaochenlong（高見龍）** 開發。

**血緣關係圖**：

```
OpenSpec (TypeScript, Fission-AI, 開源)
   │  概念、目錄結構、delta 格式、proposal/apply/archive 主幹
   ▼
Spectra (Rust, kaochenlong, 重新實作 + 大量加料)
   ├── spectra.exe   ← 命令列工具（本文反組譯的主角）
   ├── app.exe       ← 桌面 GUI（向量搜尋的真正引擎在這裡）
   └── /spectra-* 技能 ← 給 AI 照著跑的工作流劇本
```

---

## 3. Spectra 的三個組成元件

Spectra 安裝在 `C:\Users\momoc\AppData\Local\Spectra\`，由三個執行檔組成：

| 檔案 | 大小 | 角色 |
|---|---|---|
| `spectra.exe` | 8.35 MB | **命令列工具（CLI）**。Rust + clap 框架。是 AI 技能背後真正幹活的引擎：建立變更、產生 instruction、驗證、分析、歸檔等全靠它。 |
| `app.exe` | 27 MB | **桌面 GUI App**。Tauri 框架（Rust 後端 + 網頁前端）。向量搜尋的索引建立／模型下載在此進行。直接執行 `spectra`（不帶子命令）會嘗試啟動它。 |
| `uninstall.exe` | 87 KB | 解除安裝程式。 |

**三者如何協作**：你（人）在 Claude Code 裡打 `/spectra-propose 加上深色模式` → 觸發 **propose 技能**（劇本）→ 技能照著步驟去呼叫 **`spectra.exe`** 的各個子命令（建立變更、取得 instruction、寫產物、分析、驗證）→ AI 根據 CLI 吐回的 instruction 把產物內容寫出來。GUI App 則是給人用滑鼠瀏覽規格、管理變更、建向量索引的另一個入口。

**技能檔放在哪**：`spectra init` 會把技能檔寫到專案裡：

- `.claude/skills/spectra-*/SKILL.md` — 給 Claude Code 用，共 **12 個**技能，呼叫語法是 `/spectra-名稱`。
- `.agents/skills/spectra-*/SKILL.md` — 給其他 AI 工具用，共 **10 個**（少了 analyze 和 verify），呼叫語法是 `$spectra-名稱`。
- `.claude/commands/spectra/*.md` — 若開了 `claude_slash_commands: true`，額外生成 **10 個**斜線指令檔，呼叫語法是 `/spectra:名稱`（冒號）。

---

## 4. 核心資料模型：openspec/ 目錄、正典規格 vs. 變更

Spectra（與 OpenSpec）的全部運作都圍繞一個 `openspec/` 目錄。`spectra init` 後的結構：

```
專案根目錄/
├── .spectra.yaml              ← Spectra「應用設定」（語系、TDD、並行等開關）
├── CLAUDE.md / AGENTS.md      ← 給 AI 看的工作流說明（init 自動生成、夾在 SPECTRA:START/END 之間）
├── .gitignore                 ← 含一行 .spectra/
├── .claude/
│   ├── settings.json          ← {"includeGitInstructions": false}
│   ├── skills/spectra-*/SKILL.md      ← 12 個技能
│   └── commands/spectra/*.md          ← 10 個斜線指令（可選）
├── .spectra/                  ← Spectra 的「工作資料」(被 gitignore)
│   ├── spectra.db             ← SQLite 中繼資料庫
│   ├── snapshots/             ← 歸檔快照（供還原 unarchive）
│   ├── touched/               ← 每個變更「task 動過哪些程式檔」的記錄
│   └── worktrees/             ← git worktree（若啟用）
└── openspec/
    ├── config.yaml            ← 「工作流設定」（schema、專案 context、per-artifact rules）
    ├── specs/                 ← 【正典規格】＝系統現狀的真實來源
    │   └── <capability>/spec.md
    └── changes/               ← 【變更】＝提案中、進行中的修改
        ├── <change-name>/
        │   ├── .openspec.yaml ← 此變更的中繼資料（schema、建立者、日期）
        │   ├── proposal.md
        │   ├── design.md
        │   ├── tasks.md
        │   └── specs/<capability>/spec.md   ← delta 規格（只寫改了什麼）
        └── archive/
            └── YYYY-MM-DD-<change-name>/     ← 已歸檔的變更
```

**最關鍵的兩個觀念**：

1. **`openspec/specs/` 是「正典」**：描述系統**現在**長怎樣，是真實來源。
2. **`openspec/changes/<name>/specs/` 是「delta」**：只描述這個變更**要改什麼**（新增/修改/刪除/改名哪些需求）。

當變更被 `archive` 時，delta 會被**合併**進正典。例如實測中歸檔一個變更時 CLI 印出：

```
✓ Archived: spx-tall-gyarados → 2026-06-29-spx-tall-gyarados
Specs applied: snapshot-restore (added: 2, modified: 0, removed: 0, renamed: 0)
Snapshot created for unarchive support.
```

意思是：把該變更對 `snapshot-restore` 能力新增的 2 條需求併入 `openspec/specs/snapshot-restore/spec.md`，並建立快照以便還原。

**一個變更的生命週期狀態**（由 `.spectra/spectra.db` 與檔案系統共同記錄）：

- **active（活躍）**：在 `openspec/changes/<name>/`，會出現在 `spectra list`。
- **in-progress（進行中）**：在 `in_progress_change` 資料表標記了（`spectra in-progress add`）。
- **parked（暫存）**：被移出 `changes/`，記錄在 `parked_changes` 資料表，`spectra list` 看不到（要 `--parked`）。
- **archived（已歸檔）**：在 `openspec/changes/archive/YYYY-MM-DD-<name>/`，delta 已併入正典。

---

## 5. 產物（artifact）格式詳解

預設 `spec-driven` schema 有四種產物。以下格式同時來自 CLI 的嵌入式模板、`spectra instructions` 的指導文，以及 demo 變更的實際內容。

### 5.1 proposal.md（提案）

回答「為什麼」。章節：

```markdown
## Why                 ← 1-2 句講問題或機會
## What Changes        ← 條列要改什麼；破壞性變更標 **BREAKING**
## Non-Goals (optional)← 排除範圍／被否決的做法（若略過 design 則必填）
## Capabilities        ← 【最關鍵】列出要新增/修改哪些能力
### New Capabilities
- `capability-name`: 描述     ← 每個會變成 specs/<name>/spec.md
### Modified Capabilities
- `existing-name`: 改了什麼需求
## Impact              ← 影響到的程式碼、API、相依套件
```

> Capabilities 區塊是「proposal 和 specs 兩階段之間的契約」。這裡列的每個能力名稱，**就會變成 `specs/<名稱>/` 資料夾名**。分析器（analyzer）會把「列了能力卻沒有對應 spec 檔」標成 Critical（嚴重）問題。

依變更類型（propose 技能會先分類），proposal 有三種骨架：**Feature**（Why/What/Capabilities/Impact）、**Bug Fix**（Problem/Root Cause/Proposed Solution/Success Criteria/Impact）、**Refactor**（Summary/Motivation/Proposed Solution/Alternatives/Impact）。

CLI 的結構驗證規則：proposal 必須含 `## Why`、`## Problem` 或 `## Summary` 其中之一。

### 5.2 design.md（技術設計，可選）

回答「怎麼做」。只在符合條件時才建立（跨模組、新架構模式、新外部相依、安全/效能/遷移複雜度、需要先做技術決策的模糊處）。章節：

```markdown
## Context              ← 背景、現狀、限制
## Goals / Non-Goals    ← 目標與明確排除
## Decisions            ← 關鍵技術選擇＋理由（為何選 X 不選 Y）
## Implementation Contract  ← 【重要】交接契約：可觀察的行為、介面/資料形狀、
                              失敗模式、驗收標準、範圍邊界。不可只靠行號或檔案路徑。
## Risks / Trade-offs   ← 風險 → 緩解
## Migration Plan       ← 部署/回滾（若適用）
## Open Questions       ← 待解決
```

CLI 驗證：design 必須含 `## Context`。分析器會交叉比對：design 的每個 `###` 標題應該出現在某條 task 描述裡。

> **Implementation Contract（實作契約）** 是 Spectra 特別強調的一節：對任何「會產生或修改行為」的變更都必須寫。它是把設計「持久交接」給 apply 階段的依據——就算換另一個 AI 來實作，也能照契約做。apply 技能在實作每條 task 前都會被要求先讀對應的契約。

### 5.3 tasks.md（任務清單）

把實作拆成可勾選清單。格式被機器解析，**極度要求精確**：

```markdown
## 1. 任務群組名稱
- [ ] 1.1 任務描述（必須說明：交付什麼行為 + 如何驗證）
- [ ] 1.2 [P] 可並行的任務描述
## 2. 下一個群組
- [ ] 2.1 ...
```

規則：
- 每條 task **必須**是 `- [ ] X.Y 描述` 格式的勾選框，否則進度追蹤抓不到。
- `[P]` 標記 = 此任務可與相鄰任務並行（需 `parallel_tasks: true`）。
- 每條 task 必須同時講清楚「**交付什麼可觀察的行為**」和「**怎麼驗證完成**」（測試名、CLI 指令、分析器檢查、人工確認）。只寫「修改檔案 X」是無效任務。
- 交叉引用（分析器會檢查）：specs 的每個 `### Requirement:` 名稱必須以子字串出現在至少一條 task 描述中；design 的每個 `###` 標題也應被某條 task 引用。
- 人類可讀部分（群組名、描述）用 locale 語言；機器可讀符號（勾選框、編號、`[P]`、檔名、指令）一律保留原文不翻譯。

CLI 驗證：tasks 必須至少含一個 `- [ ]` 勾選框。

### 5.4 specs/<capability>/spec.md（delta 規格）

只寫「改了什麼」，用四種 `##` 區塊：

```markdown
## ADDED Requirements         ← 新增需求
### Requirement: 需求名稱
系統 SHALL ……（強制語氣）
#### Scenario: 情境名稱        ← 注意：剛好 4 個井字號！
- **WHEN** 條件
- **THEN** 預期結果
##### Example: 例子名稱（可選） ← 5 個井字號，SBE 具體數值
- **GIVEN** 真實測試資料
- **WHEN** 具體輸入
- **THEN** 具體輸出

## MODIFIED Requirements      ← 修改需求（必須貼「整條」更新後內容，不能只貼片段）
## REMOVED Requirements       ← 刪除需求（必須含 **Reason** 和 **Migration**）
## RENAMED Requirements       ← 改名（用 FROM:/TO: 格式）
```

**鐵則**（CLI 與分析器強制）：
- `### Requirement:`＝3 個井字號；`#### Scenario:`＝**剛好 4 個**；`##### Example:`＝5 個。**用錯井字號數量會「無聲失敗」**（解析器直接忽略，不報錯），所以模板裡再三警告。
- 每條需求至少要有一個 scenario。
- 禁用模糊詞（分析器會標）：`should, may, might, consider, possibly, TBD, TODO, ???, TKTK` → 一律換成 `SHALL / SHALL NOT / MUST / MUST NOT`。
- **delta 規格至少要含一個操作**（ADDED/MODIFIED/REMOVED/RENAMED），否則驗證失敗。
- **規格永遠用英文寫**，不管 locale 設什麼——因為用的是規範語言。

### 5.5 歸檔後的正典規格長相

歸檔時，delta 的 `## ADDED Requirements` 會被轉成正典格式並注入 `@trace` 追溯註解。實測歸檔後 `openspec/specs/snapshot-restore/spec.md`：

```markdown
# snapshot-restore Specification
## Purpose
TBD - created by archiving change 'spx-tall-gyarados'. Update Purpose after archive.
## Requirements
### Requirement: Automatic Snapshots
The system SHALL create automatic snapshots ...
#### Scenario: Periodic snapshot
- **WHEN** ...
- **THEN** ...

<!-- @trace
source: spx-tall-gyarados
updated: 2026-06-29
code:
  - .spectra.yaml
  - CLAUDE.md
-->
```

`@trace` 的 `code:` 清單來自 apply 階段 `spectra task done` 記錄的「該變更動過哪些檔案」。

### 5.6 .openspec.yaml（變更中繼資料）

每個變更資料夾根目錄有一個：

```yaml
schema: spec-driven
created: 2026-06-29
created_by: MomoChen <momochenisme@gmail.com>
```

二進位中的結構 `ChangeMetadata` 還有 `created_with`、`archived_by`、`archived_at` 欄位（歸檔時填入）。`created_by`/`created_with` 來自 git 的 `user.name`/`user.email`。

---

## 6. 【重點】三層設定系統與 instruction 注入時機

> 這一章直接回答你的核心問題：「`.spectra.yaml` 在什麼階段被 CLI 讀取來注入 instruction？`config.yaml` 在什麼階段讀取？」

### 6.1 三個設定檔，各管什麼

Spectra 有**三層設定**，很容易搞混，先講清楚：

| # | 檔案 | 範圍 | 由誰讀 | 內容 |
|---|---|---|---|---|
| ① | `<專案>/.spectra.yaml` | 專案層「**應用設定**」 | CLI 在多個指令中讀；**技能也會直接讀** | `spec_dir, locale, tdd, audit, parallel_tasks, claude_slash_commands, worktree, worktrees_dir, claude_effort, tools, dir` |
| ② | `<專案>/openspec/config.yaml` | 專案層「**工作流設定**」 | CLI 在 `instructions` 等指令中讀 | `schema, context, rules`（per-artifact） |
| ③ | `AppData\Roaming\openspec\config.yaml` | 全域「**使用者設定**」 | `spectra config` 系列指令讀寫 | 使用者層級的鍵值（預設空白） |

對應的 Rust 結構（從二進位確認）：
- ① `.spectra.yaml` → `struct SpectraConfig`（`crates/spectra-core/src/spectra_config.rs`），欄位順序：`dir, spec_dir, locale, tdd, audit, parallel_tasks, claude_slash_commands, worktree, worktrees_dir, claude_effort, tools`。
- ② `openspec/config.yaml` → 含 `schema / context / rules` 三鍵。
- 注意 `spectra config path` 指向的是 **③ 全域設定**（`AppData\Roaming\openspec\config.yaml`），**不是**專案的那兩個。這是一個常見誤解點。

`.spectra.yaml` 各鍵的作用：

| 鍵 | 作用 |
|---|---|
| `spec_dir` | 把 `openspec/` 改到別的路徑（例如 `docs/specs`）。**改了要重建向量索引**。 |
| `locale` | AI 生成產物的語言。`tw`→繁中、`ja`→日文（二進位裡只有這兩種非英語對應）。 |
| `tdd` | `true` 時 apply/debug 技能採用測試先行紀律。 |
| `audit` | `true` 時 apply 技能對程式碼套用「安全銳角」稽核紀律。 |
| `parallel_tasks` | `true` 時 propose/ingest 會在 tasks 加 `[P]` 標記、apply 會並行派工。 |
| `claude_slash_commands` | `true` 時 init/update 額外生成 `/spectra:X` 斜線指令檔。 |
| `worktree` / `worktrees_dir` | 啟用 git worktree 隔離；預設目錄 `.spectra/worktrees`。 |
| `claude_effort` | 各技能的「思考力道」等級（low/medium/high/xhigh/max），可逐技能設定，例如 `apply: high`。 |
| `tools` | 要生成哪些 AI 工具的指令檔（claude、cursor、codex…）。 |

### 6.2 instruction 注入的「時機」與「機制」

**核心命令是 `spectra instructions <產物|apply>`**。技能（如 propose、apply、ingest）在工作流的**每一個產生產物的步驟**，都會即時呼叫它。CLI 在被呼叫的當下，做以下事：

1. 從二進位**內嵌的** schema 取出該產物的「指導文（instruction）＋空白模板（template）」。
2. **即時讀取 ① `.spectra.yaml`**，把 `locale` 轉成人類可讀語言名，放進輸出 JSON 的 `locale` 欄。
3. **即時讀取 ② `openspec/config.yaml`**，把 `context`（原文照搬）和 `rules.<該產物>`（**只取該產物的規則**）放進 JSON 的 `context`、`rules` 欄。
4. 加上動態資訊（相依產物、目前進度、preflight、worktreePath 等），整包以 JSON 吐回給技能/AI。

換句話說——**設定不是在 init 或某個固定時間被「燒進」instruction，而是在每次技能呼叫 `spectra instructions` 的那一刻，CLI 才現場去讀 `.spectra.yaml` 和 `openspec/config.yaml`，當場合併產生**。

### 6.3 用實驗證明（對照實測）

我在臨時專案做了對照實驗。**先把預設（locale 註解掉、config 無 context/rules）跑一次**，`spectra instructions proposal --json` 的 `locale` 是 `"English"`、沒有 `context`/`rules` 欄。

**接著改設定**：`.spectra.yaml` 設 `locale: tw`；`openspec/config.yaml` 設：

```yaml
context: |
  Tech stack: Rust, Tauri, SvelteKit
  We use conventional commits
  Domain: developer tooling
rules:
  proposal:
    - Keep proposals under 500 words
    - Always include a "Non-goals" section
  tasks:
    - Break tasks into chunks of max 2 hours
```

**再跑 `spectra instructions proposal --json`**，輸出多了：

```json
{
  "context": "Tech stack: Rust, Tauri, SvelteKit\nWe use conventional commits\nDomain: developer tooling",
  "rules": [
    "Keep proposals under 500 words",
    "Always include a \"Non-goals\" section"
  ],
  "locale": "Traditional Chinese (繁體中文)",
  ...
}
```

**而 `spectra instructions tasks --json` 的 `rules` 卻是**：

```json
"rules": ["Break tasks into chunks of max 2 hours"]
```

證明三件事：
1. `.spectra.yaml` 的 `locale: tw` → 在 instructions **被呼叫時**現場讀取，轉成 `"Traditional Chinese (繁體中文)"`。
2. `openspec/config.yaml` 的 `context` → 原文注入。
3. `rules` → **依產物過濾**：proposal 只拿到 proposal 的規則，tasks 只拿到 tasks 的規則。

### 6.4 instructions JSON 的完整欄位

從二進位確認 `instructions` 輸出的完整欄位集合：

`changeName, changeDir, schemaName, contextFiles, progress, tasks, state, missingArtifacts, locale, instruction, worktreePath, preflight, artifactId, outputPath, description, context, rules, template, dependencies, unlocks, total, complete, remaining, id, path`

- 一般產物（proposal/specs/design/tasks）：`artifactId, outputPath, description, instruction, template, context, rules, locale, dependencies, unlocks`。
- `instructions apply`（實作模式）：`contextFiles{proposal,specs,design,tasks 路徑}, progress{total,complete,remaining}, tasks[{id,description,done,parallel}], state, locale, instruction, worktreePath, preflight{status, missingFiles, driftedFiles, staleness{daysOld, isStale}}`。

### 6.5 設定開關「落在哪個階段、由誰讀」總整理

很重要的一點：**有些設定是 CLI 注入的，有些是技能自己直接讀 `.spectra.yaml`**：

| 設定 | 誰讀、何時讀 | 效果 |
|---|---|---|
| `locale` | **CLI 注入**：propose/ingest 呼叫 `instructions` 時 → JSON `locale` 欄 | AI 用該語言寫產物（spec 除外永遠英文） |
| `context` / `rules`（openspec/config.yaml） | **CLI 注入**：呼叫 `instructions` 時，rules 依產物過濾 | 當作 AI 寫產物的約束（但不可抄進產物） |
| `tdd` | **技能直接讀** `.spectra.yaml`：apply 步驟 5、debug 階段 4 | 取 `spectra instructions --skill tdd`，走紅-綠-重構 |
| `audit` | **技能直接讀**：apply 步驟 5 | 取 `spectra instructions --skill audit`，套銳角稽核紀律 |
| `parallel_tasks` | **技能直接讀**：propose/ingest 建 tasks 時加 `[P]`；apply 步驟 5 並行派工 | 並行任務分派 |
| `worktree` / `claude_effort` | CLI／harness 側處理，技能不直接分支 | 隔離分支／思考力道 |

> 也就是說：`locale`、`context`、`rules` 走「**CLI → instructions JSON → 技能**」這條注入路線；而 `tdd`、`audit`、`parallel_tasks` 是技能**自己打開 `.spectra.yaml` 檔案讀**的。兩條路線並存。

---

## 7. Spectra CLI 完整命令參考

`spectra.exe` v2.3.1，全域旗標 `--no-color`、`-h/--help`、`-V/--version`。以下為全部 23 個頂層命令（以實際 `--help` 為準）。

### 7.1 初始化與更新

| 命令 | 作用 |
|---|---|
| `init [PATH] [--tools claude,cursor] [--force] [--dir openspec]` | 在專案建立 Spectra 結構：`openspec/`（config.yaml、changes/、specs/）、`.spectra.yaml`（全註解模板）、`.gitignore`、各 AI 工具的指令檔與技能。`--dir` 自訂 openspec 目錄名。 |
| `update [PATH] [--force]` | 重新生成指令檔與技能（刷新 `SPECTRA:START..END` 區塊與 skills），用於 CLI 升級後。 |

`init`/`update` 寫技能檔時會：加上 YAML frontmatter（name、description、license: MIT、metadata）、把作者體裡的 `/spectra:名稱` 換成 `/spectra-名稱`（Claude）或 `$spectra-名稱`（其他 agent）、替換佔位符 `{{SPEC_DIR}}`、`{{PLAN_DIR}}`、`{{TOOL}}`。

### 7.2 瀏覽與檢視

| 命令 | 作用 |
|---|---|
| `list [--specs] [--changes] [--parked] [--sort name\|modified\|created] [--json]` | 列出變更或規格。`--parked` 顯示暫存的變更。 |
| `show [ITEM] [--json] [--item-type change\|spec] [--deltas-only] [-r/--requirements]` | 顯示某個變更或規格的內容。 |
| `status [--change X] [--schema S] [--json]` | 顯示產物相依圖（DAG）狀態。輸出含 `applyRequires`、各產物 `done/ready/blocked`、`isComplete`。 |
| `search <QUERY> [--limit 10] [--json]` | 向量語意搜尋文件。**Windows 不支援**（需 Apple M 系列）。 |

### 7.3 產物與變更的建立

| 命令 | 作用 |
|---|---|
| `new change <NAME> [--description] [--schema] [--agent claude\|codex\|gemini]` | 建立變更目錄（只生成 `.openspec.yaml`，無產物）。 |
| `new artifact <proposal\|design\|tasks\|spec> [capability] [--change X] [--stdin] [--force]` | 建立產物檔。`--stdin` 從標準輸入讀內容（技能用 heredoc 傳入），否則用空白模板。 |
| `demo` | 生成一個示範變更 `spx-<字>-<寶可夢>`（含完整範例產物），供體驗。 |

### 7.4 品質檢查

| 命令 | 作用 |
|---|---|
| `validate [ITEM] [--all] [--changes] [--specs] [--strict] [--json]` | 結構驗證。輸出 `{change, errors[], valid, warnings[]}`。 |
| `analyze [CHANGE] [--json]` | 4 維一致性分析（Coverage/Consistency/Ambiguity/Gaps）。詳見第 11 章。 |
| `drift [CHANGE] [--json]` | 偵測變更與程式碼的漂移（Time/Structure/Tasks/Environment）。詳見第 11 章。 |

### 7.5 instruction 與 schema

| 命令 | 作用 |
|---|---|
| `instructions [ARTIFACT\|apply] [--change X] [--schema S] [--json] [--skill NAME]` | 取得某產物的指導文＋模板＋設定注入（見第 6 章）。`--skill NAME` 直接吐出某個嵌入式技能本體。 |
| `schemas [--json]` | 列出可用工作流 schema。 |
| `templates [--schema S] [--json]` | 顯示某 schema 各產物的模板路徑。 |
| `schema which [NAME] [--all]` | 顯示 schema 從何處解析（project → openspec/schemas → user → 內嵌 → 內建）。 |
| `schema validate [NAME] [--verbose]` | 驗證 schema。 |
| `schema fork <SOURCE> [NAME] [--force]` | 複製一個 schema 來改。 |
| `schema init <NAME> [--artifacts a,b,c] [--default] [--description]` | 建立自訂 schema。 |

### 7.6 變更生命週期

| 命令 | 作用 |
|---|---|
| `archive <CHANGE> [-y] [--skip-specs] [--no-validate] [--mark-tasks-complete]` | 歸檔：移到 `archive/`、把 delta 併入正典、注入 @trace、建快照。`--skip-specs` 跳過併規格（純工具/文件變更用）。 |
| `park <NAME>` | 暫存：移出 `changes/`，記入 `parked_changes` 表。 |
| `unpark <NAME>` | 取回暫存的變更。 |
| `in-progress add <NAME>` | 把變更標記為進行中（寫入 `in_progress_change` 表）。 |
| `task done <TASK_ID> [--change X] [--json]` | 把第 N 條 task 的勾選框 `- [ ]`→`- [x]`，並記錄這條 task 動過哪些檔案（供 @trace）。 |

### 7.7 設定與雜項

| 命令 | 作用 |
|---|---|
| `config path \| list \| get <K> \| set <K> <V> [--string] [--allow-unknown] \| unset <K> \| reset \| edit` | 管理**全域**設定（`AppData\Roaming\openspec\config.yaml`）。 |
| `completion generate \| install \| uninstall [SHELL]` | Shell 自動補完（支援 Bash、Elvish、Fish、PowerShell、Zsh）。 |
| `feedback <MESSAGE> [--body B]` | 送出意見（導向 `github.com/kaochenlong/Spectra/issues`）。 |
| `help` | 顯示說明。 |

> **隱藏／特殊行為**：直接執行 `spectra`（不帶子命令）會嘗試**啟動桌面 GUI**（找不到打包版時提示 `GUI launch requires a packaged build`）。二進位中還有一個 `self-update` 字串出現在某命令清單，但**不在** clap 的子命令定義裡，推測為隱藏／功能旗標控制，本 Windows 版未必可呼叫。

---

## 8. 工作流 schema 與產物相依圖（DAG）

`schemas --json` 顯示唯一內建 schema：

```json
{
  "name": "spec-driven",
  "artifacts": ["proposal", "specs", "design", "tasks"],
  "description": "Default OpenSpec workflow - proposal → specs → design → tasks",
  "source": "package"
}
```

**產物相依圖（DAG）**——`status` 會依此判斷每個產物可不可以做：

```
proposal ──┬──► specs ──► tasks
           └──► design
                          (apply 需要 tasks 完成)
```

- `proposal` 一開始就 **ready（可做）**。
- `design`、`specs` 被 `proposal` **blocked（卡住）**，等 proposal 完成才解鎖。
- `tasks` 被 `specs` 卡住。
- `design` 是**可選**的：二進位內嵌 schema 的描述其實寫「proposal → specs → tasks (design optional)」，apply 只要求 `tasks` 完成（`applyRequires: ["tasks"]`）。

狀態符號（`spectra status`）：`✓` done（完成）、`○` ready（可做）、`✗` blocked（被卡，會顯示 `blocked by: <相依>`）。

schema 的內部結構（二進位 `struct Schema`）：`name, version, description, artifacts[], apply`。每個 artifact 有 `id, generates, description, template, instruction, requires`；`apply` 階段有 `requires`（清單）與 `tracks`（預設 `.md`）。CLI 會做**循環相依偵測**（`Circular dependency detected`）。

你可以用 `schema fork` / `schema init` 自訂 schema，定義自己的產物與相依鏈。解析順序：`project → openspec/schemas → user → (內嵌於二進位) → 內建`。

---

## 9. 12 個技能逐一詳解

技能是「給 AI 照著做的劇本」。Spectra 內嵌了 **14 個**技能本體（可用 `spectra instructions --skill <名稱>` 取出，名稱不帶 `spectra-` 前綴），其中：

- **12 個會生成 SKILL.md 檔**（Claude 版）：`analyze, apply, archive, ask, audit, commit, debug, discuss, drift, ingest, propose, verify`。
- **10 個會生成斜線指令檔**（少了 analyze、verify）。
- **另 3 個是內部用、不生成獨立檔**：`sync`（agent 驅動的 delta→正典合併）、`clarify`（被動式釐清提問）、`tdd`（測試紀律）。`tdd`/`audit` 由其他技能用 `instructions --skill` 取用。

技能分兩大類：
- **唯讀分析型**（`context: fork` + `agent: Explore` + 禁用 Edit/Write）：`analyze, ask, audit, discuss, drift, verify`。它們在隔離子代理裡跑，**只能讀不能改**。
- **主流程型**（在主對話執行、可改檔）：`propose, apply, ingest, archive, commit, debug`。

整體工作流（CLAUDE.md 定義）：

```
discuss?（可選）→ propose → apply ⇄ ingest → archive
                                  ↘ verify / analyze / drift / audit / commit / debug（隨時可插入）
```

---

### 9.1 `/spectra-discuss`（討論，可選的起點）

**何時用**：寫程式前需要先把想法談清楚、收斂出結論。**只思考、不實作**（禁 Edit/Write；要動手請先離開 discuss 改用 propose）。

**步驟邏輯**：
1. **Step 0 載入共用詞彙**：先讀 `openspec/LANGUAGE.md`（專案的標準術語表，含 `definition/avoid/why`）。有就優先用標準詞、發現用了 `avoid` 同義詞就在結論裡標「詞彙漂移」；沒有就靜默略過（不報錯）。
2. 從題目抽 2-5 個關鍵字 → 用 Grep/Glob **掃描原始碼**（非文件/測試），讀最多 5 個相關檔。
3. **選模式**：找到 ≥3 個相關原始檔 → **Assumptions 模式**（直接列 3-5 條假設，每條含「做法＋證據檔案＋若錯的後果」，問「哪些錯了？」）；<3 個 → **Interview 模式**（一次問一個問題）。會宣告選了哪個模式及理由，使用者可隨時切換。
4. **介面深度檢查（條件式）**：只在引入新架構接縫時觸發（新模組／新 IPC 指令／新跨層流程／新儲存抽象），問四題：接縫位置、轉接層數量、深度、刪除測試（刪了會壞什麼？沒壞代表是多餘的 pass-through）。純 UI 文案/樣式/文件用字則跳過。
5. **收斂**：縮小選項 → 點出關鍵取捨 → 給建議 → 明確結論（四選一：設計決策／方向共識／下一步建議／明確延後）。會用「實例引導」確認理解（並順手產生 `##### Example:` 內容）。
6. **節奏規則**：使用者想加快時，最多只「提醒一次」未解問題，再被催就直接收斂。
7. **捕捉結論**：收斂時主動提出 `## Conclusion`（Decision／Rationale／Capture to），依路由表把結論寫進對應產物（新需求→spec.md；設計決策→design.md；範圍變動→proposal.md；新工作→tasks.md；詞彙漂移→LANGUAGE.md）。預設會捕捉，使用者可拒絕。最後可接 `/spectra-propose`。

**呼叫 CLI**：`spectra list --json`（看現有變更）。**讀設定**：`openspec/LANGUAGE.md`（不讀 .spectra.yaml 開關）。

---

### 9.2 `/spectra-propose`（建立完整變更提案）

**何時用**：要規劃／設計一個變更。SDD 工作流的入口。**完成後會把變更 park 起來、不寫任何程式**。

**步驟邏輯**（11 步）：
1. **判定需求來源**：argument > plan 檔（`~/.claude/plans/<name>.md`，用 AskUserQuestion 選用 plan 檔或對話脈絡）> 對話脈絡。推導 kebab-case 變更名（去掉 `YYYY-MM-DD-` 日期前綴）。
2. **分類變更類型**：Feature／Bug Fix／Refactor（決定 proposal 模板）。
3. **掃描既有規格**：`Glob openspec/specs/*/spec.md`，找最多 5 個相關（讀前 10 行取 Purpose），只當資訊顯示、不擋流程。
4. `spectra new change "<name>" --agent claude`（建目錄）。
5. **寫 proposal**：`spectra instructions proposal --change X --json` 取指導 → 依類型生成內容 → `spectra new artifact proposal --change X --stdin`（heredoc 傳入，CLI 驗格式，錯了就改）。（Impact 路徑須以專案根為基準、要「錨定」，不可用 `parser/mod.rs` 這種片段，也不要用反引號包路徑——preflight 會誤判。）
6. `spectra status --change X --json` 取 `applyRequires` 與產物清單（建構順序）。
7. **依相依順序建其餘產物**（specs/design/tasks）：對每個 ready 的產物——先判斷是否可選（不在 applyRequires 相依鏈上就是可選，讀 instruction 的條件判定，不符就 `⊘ Skipped`）→ 取 instructions JSON（含 `context/rules/template/instruction/locale/dependencies`）→ 讀相依檔 → 用 template 當結構生成（把 context/rules 當約束但**不抄進檔案**）→ `spectra new artifact <id> --change X --stdin`（specs 每個能力一條指令）→ 重跑 status 直到 applyRequires 都 done。
8. **內聯自審（送 CLI 分析器前）**：5 項檢查——無佔位符／內部一致性／範圍（>15 任務或 >3 子系統考慮拆分）／模糊度／持久交接審查（拒絕「只給檔名的任務」「綁行號的指令」「模糊驗收標準」「缺範圍邊界」）。
9. **analyze-fix 迴圈（最多 2 輪）**：`spectra analyze X --json`，只處理 Critical/Warning（忽略 Suggestion），修完再跑，2 輪後仍有就摘要列出、不擋。
10. `spectra validate X`（驗證，錯了改）。
11. **無條件 `spectra park "<name>"`**：把變更暫存，告知使用者之後 `/spectra-apply <name>` 會自動取回並開始實作。**流程到此結束，絕不自動呼叫 apply**。

**讀設定**：`parallel_tasks` 直接讀 `.spectra.yaml`（建 tasks 時加 `[P]`）；`locale/context/rules` 透過 instructions JSON 注入。

---

### 9.3 `/spectra-apply`（實作／續作任務）

**何時用**：變更的 tasks 備妥，要開始寫程式。是工作流的 apply 節點，可隨時插入（甚至產物未全備、只要有 tasks）。

**步驟邏輯**（9 步＋3 道前置檢查閘）：
1. **選變更**：給名稱就用；否則推斷／單一自動選；模糊時 `spectra list --json` ＋ `spectra list --parked --json`（暫存的標 `(parked)`）用 AskUserQuestion 選。宣告「Using change: <name>」。
2. **查狀態／處理暫存**：`spectra status --change X --json`（失敗就停）。再 `spectra list --parked --json` 查是否 parked——是的話問使用者，同意則 `spectra unpark` + `spectra in-progress add`（靜默）後重跑 status；不是 parked 就直接 `spectra in-progress add`。
3. **取 apply 指令**：`spectra instructions apply --change X --json`（含 contextFiles、progress、tasks、state、preflight）。`state: blocked` → 建議先 propose；`all_done` → 建議歸檔。
   - **3b Preflight 閘**：依 `preflight.status`——`clean` 靜默過；`warnings`（漂移檔／陳舊天數）顯示摘要後自動續；`critical`（缺檔）列出後用 AskUserQuestion 問「續/停」。
   - **3c 產物品質閘**：`spectra analyze X --json`——0 發現靜默過；只有 warning/suggestion 顯示一行續；有 Critical 顯示後問「修正並續／直接續／停」。
   - **3d 漂移休眠閘（被動觸發）**：當變更**閒置 >5 天 且 變更目錄近 3 天 0 commit** → 跑 `spectra drift X` 顯示報告，問「續作／先 refresh（跑 ingest）／停」。此閘只是建議、**不硬擋**。
4. **讀 contextFiles**（proposal/specs/design/tasks）。
5. **讀專案偏好** `.spectra.yaml`：`tdd: true` → 測試先行＋取 `instructions --skill tdd` 走紅-綠-重構；`audit: true` → 取 `instructions --skill audit` 套銳角稽核；`parallel_tasks: true` → 相鄰 `[P]` 任務並行派工。
6. 顯示進度。
7. **實作迴圈**（直到完成或受阻）：對每條 task——重讀相關 design/spec 段落（別信壓縮過的記憶）→ 讀該 task 範圍的 Implementation Contract → 偵測「只給檔名／模糊／與契約衝突」的任務就暫停→實作前檢查（重用/品質/效率/無佔位符/把 `##### Example:` 當測試）→ 最小改動 → **完成前驗證**（重讀任務描述＋契約，確認驗證目標真的通過）→ `spectra task done --change X <id>`（翻勾選框＋記錄動過的檔）。受阻就暫停回報。
8. **最終檢查**：重跑 `instructions apply --json` 確認 `state: all_done`。
9. 顯示狀態，全完成則建議 `/spectra-archive`。

> 核心紀律：**進度只認 tasks 檔的勾選框**，禁用任何外部待辦工具。

---

### 9.4 `/spectra-ingest`（從外部脈絡更新既有變更）

**何時用**：已有進行中的變更，需求中途改變（plan 模式產生新 plan 檔、或對話演進）要把新脈絡吸收進產物。**只更新、不建立**新變更。

**步驟邏輯**：定位來源（plan 檔／對話，用 AskUserQuestion）→ 解析 plan 結構 → `spectra list --json` ＋ `--parked` 確認有現存變更（沒有就叫你先 propose）→ 選變更、處理 parked（unpark）→ **逐產物更新**（`spectra instructions <id> --change X --json` 取模板/context/rules/locale，**合併而非取代**，**保留已完成 `[x]` 任務與 `[P]` 標記**）→ 6 項內聯自審（多一項「保存檢查」確認 `[x]` 沒被動）→ analyze-fix 迴圈（最多 2 輪）→ `spectra validate` → 摘要（用 AskUserQuestion 給「Done／Apply」兩選項強制收尾）。

**讀設定**：`parallel_tasks` 直接讀；`locale/context/rules` 經 instructions JSON。**鐵則**：絕不改原始 plan 檔、絕不寫程式、絕不建新變更。

---

### 9.5 `/spectra-archive`（歸檔完成的變更）

**何時用**：實作完成，要收尾：把變更移出 `changes/`、delta 併入正典。工作流終點。

**步驟邏輯**：選變更（`spectra list --json`，**絕不自動選**）→ `spectra status --change X --json` 查產物是否全 done（否則警告＋確認）→ 讀 tasks 數勾選框（有未完成就警告＋確認）→ **評估 delta 同步狀態**（比對 `changes/<name>/specs/` 與 `openspec/specs/<cap>/spec.md`，算出會 add/modify/remove/rename 什麼，顯示摘要後問「現在同步／不同步直接歸檔」）→ 清掉 `.spectra/touched/<name>.json` → **`spectra archive <name>`**（內部做：快照、套 delta、注入 @trace、記錄身分、向量索引）→ 顯示摘要。

三道閘都是「警告後確認、不硬擋」。歸檔失敗（同名已存在）會提示改名而非覆蓋。

> **同步的執行細節**：archive 技能若選擇同步，是透過 **Task 工具呼叫 `spectra-sync-specs` 技能**（agent 驅動，讀 delta 直接改正典），**不是**呼叫 `spectra sync` CLI 指令（見第 15 章已知問題）。

---

### 9.6 `/spectra-analyze`（產物一致性分析，唯讀）

**何時用**：實作前想確認變更的產物彼此一致。唯讀 fork。也會在「所有產物完成時」被動觸發、在建議 apply 前自動跑。

**步驟邏輯**：`spectra analyze <name> --json` → 取四維（Coverage/Consistency/Ambiguity/Gaps）的 `dimensions` 與 `findings`（id/dimension/severity/location/summary/recommendation）→ 整理成 Markdown 表（依 Critical > Warning > Suggestion 分組）→ **可選的 AI 語意補強**（讀產物找程式化分析抓不到的深層矛盾：design 與 spec 牴觸、task 超出 proposal 範圍、design 風險無 spec 覆蓋）→ 建議下一步。

fork 內若變更選擇模糊，**不互動發問**，而是把候選清單丟回主執行緒請其重跑。

---

### 9.7 `/spectra-verify`（實作 vs. 產物的驗證，唯讀）

**何時用**：實作完成後、歸檔前的品質閘。檢查「程式碼是否真的符合 spec/tasks/design」。Claude 專屬（`.agents` 版沒有）。

**步驟邏輯**：`spectra list/status` 選變更 → `spectra instructions apply --change X --json` 取 contextFiles 讀全部產物 → 三維驗證：
- **Completeness（完整性）**：tasks 勾選框數完成度（每條未完成 = CRITICAL）；delta spec 的每條 `### Requirement:` 去程式庫找實作證據（找不到 = CRITICAL）。
- **Correctness（正確性）**：需求對應實作（有偏離 = WARNING）；`#### Scenario:` 是否被程式/測試覆蓋；`##### Example:` 的 GIVEN/WHEN/THEN 值是否有對應測試（表格每列對應一個參數化測試）。
- **Coherence（連貫性）**：實作是否遵守 design 決策；程式風格是否與專案一致（偏離 = SUGGESTION）。

→ 產出計分卡 + 依 CRITICAL/WARNING/SUGGESTION 分組的報告 + 最終「可否歸檔」裁決。是 apply 與 archive 之間的閘。

---

### 9.8 `/spectra-drift`（漂移偵測，唯讀）

**何時用**：續作一個變更前，確認它是否已和現在的程式碼脫節。唯讀 fork。也會被 apply 的休眠閘被動觸發。

**步驟邏輯**：`spectra drift <name> --json` → 取 `severity`（light/medium/heavy）、`total_score`、`dimensions`（Time/Structure/Tasks/Environment）、`broken_anchors`、`tasks_blocked_external`、`tasks_maybe_resolved`、`primary_recommendation` → **結論先行**地呈現（第一段先講「該怎麼辦」，技術細節放後面，空的區段省略）→ 用 AskUserQuestion 依嚴重度給選項：
- Light（0-3 分）：直接開工 `/spectra-apply` ／ 暫緩。
- Medium（4-8）：先 refresh `/spectra-ingest`（帶斷錨與任務衝突當脈絡）／直接開工／暫緩。
- Heavy（>8 或錨點衰減 >30%）：歸檔重來（通常是 `spectra archive <name> --skip-specs`）／先 refresh／暫緩。

**絕不自動執行**任何後續指令，一律等使用者選。

---

### 9.9 `/spectra-audit`（安全銳角稽核，唯讀）

**何時用**：要稽核變動的程式碼有沒有「安全銳角」——危險預設值、型別混淆、無聲失敗。報告型 fork，**絕不改檔**。

**步驟邏輯**：`git diff HEAD` 取變動（空的就回報無問題並停）→ 用**三種對手視角**檢視：Scoundrel（惡意：能否關掉安全防護？降級演算法？）、Lazy Developer（懶惰：第一個範例安全嗎？預設安全嗎？）、Confused Developer（困惑：參數會不會被默默搞錯？型別分得清嗎？）→ 套**六大陷阱類別**：演算法選擇陷阱、危險預設、原始型別 vs 語意型別、設定懸崖、無聲失敗、字串化安全 → 分級（Critical/High/Medium/Low）→ 反合理化檢查（駁回「文件有寫」「進階使用者需要彈性」等藉口，建議讓安全成為唯一/預設路徑）→ 回報給主執行緒決定是否修。

> 這是 `.spectra.yaml` `audit: true` 時 apply 會「以紀律形式」套用的同一套框架（apply 取 `instructions --skill audit` 用的是檢查清單版，不是這個獨立 3-agent 流程）。

---

### 9.10 `/spectra-debug`（系統化除錯紀律）

**何時用**：回報 bug、要系統化診斷修復。獨立方法論，不屬變更工作流。

**步驟邏輯**：**四階段＋三次嘗試規則**：
- **全域規則：每個假設最多 3 次修復嘗試**（只算階段 4；1-3 階段不算）。第 3 次失敗就停手、記錄、質疑假設、換完全不同角度。「不要一直試同一招的變體，那叫迴圈不叫除錯。」
- **階段 1 Reproduce（重現）**：先能穩定重現（不能重現就不能除錯）。
- **階段 2 Isolate（隔離）**：二分定位、檢查邊界輸入輸出、只在決策點加日誌、回歸 bug 用 `git bisect`。
- **階段 3 Root Cause（根因）**：問為什麼而非哪裡；假設要可預測、能解釋全部症狀、能用測試證明。
- **階段 4 Fix（只在 1-3 完成後）**：先寫會失敗的測試（`tdd: true` 則取 `instructions --skill tdd`）→ 最小改動修根因 → 測試通過 → 全測試套件無回歸 → 檢查同模式是否存在於他處一併修。

**讀設定**：只讀 `tdd`。

---

### 9.11 `/spectra-commit`（只提交某個變更相關的檔案）

**何時用**：多個變更同時進行時，只把某一個變更相關的檔案 git commit。工具型技能，非工作流步驟。

**步驟邏輯**：`git --version`（沒 git 就停）→ 選變更 → 讀 `.spectra/touched/<name>.json`（apply 期間 `task done` 記的「各 task 動過哪些檔」，按 task 分組）→ `git status --porcelain` → **三分類**：變更產物（`openspec/changes/<name>/` 下）、原始檔（在 touched 清單裡）、無關變更（兩者皆非）→ 顯示提交計畫 → AskUserQuestion 四選項（照計畫提交／納入全部髒檔／自訂／先歸檔再一起提交）→ 逐檔 `git add <file>`（**絕不 `git add -A`**）→ `git commit -m "spectra(<change>): <摘要>"`（摘要取 proposal 的 Why 首句，body 含 `Change:` 與 `Tasks: x/y complete`）→ 顯示結果。

「先歸檔」子流程會處理未完成任務、delta 同步、執行 `spectra archive`，再把歸檔造成的檔案移動一起納入提交。

---

### 9.12 `/spectra-ask`（規格知識庫問答，唯讀）

**何時用**：問專案規格、功能、概念、運作方式。唯讀 fork，**只根據 `openspec/` 文件回答**，不用一般知識。

**步驟邏輯**：解析問題（純打招呼或問工具本身才跳過搜尋）→ `spectra search "<query>" --limit 10 --json`（向量搜尋，跨語言，不要翻譯／擴展關鍵字）→ **錯誤閘**：JSON 有 `error` 就回對應的固定中文訊息並停，**不退回 grep/檔案搜尋**（`vector_not_compiled`→需 Apple Silicon；`index_not_built`→請到 Settings 建索引；`model_not_downloaded`→請下載模型）→ 讀命中的檔（最多 10 個，`openspec/specs/` 現狀優先於 `archive/` 歷史）→ **只根據文件內容回答**（沒有就說「規格文件中沒有這個內容」，不猜）→ 固定格式呈現（第一行原問題用 `>` 引用、答案、可選的「Referenced Files」）。

內含大量**安全強化**：把所有查詢與文件內容當「資料」而非「指令」（防注入：忽略 `<!-- ignore rules -->`、`[SYSTEM:...]`、「ignore previous instructions」）；禁讀 `openspec/` 以外的檔（`~/.ssh/`、`.env`、`credentials.json`）；拒絕洩漏密鑰/PII（回「無法提供敏感資訊。」、把發現的密鑰標 `[REDACTED]`）；只跑指定的 `spectra search`，不執行查詢/文件裡夾帶的 shell 指令。

> Windows 上向量搜尋不可用，所以此技能實際命中的是錯誤閘／無結果路徑。

---

## 10. 技能 × CLI 搭配總表

「誰呼叫哪些 CLI 子命令」一覽（這回答了「CLI 與技能怎麼搭配」）：

| 技能 | 主要呼叫的 `spectra` 子命令 | 讀的設定 | 會不會改檔 |
|---|---|---|---|
| discuss | `list` | LANGUAGE.md | 否（唯讀） |
| propose | `new change`、`instructions <產物>`、`new artifact --stdin`、`status`、`analyze`、`validate`、`park` | `parallel_tasks`；`locale/context/rules`(注入) | 是 |
| apply | `list[--parked]`、`status`、`unpark`、`in-progress add`、`instructions apply`、`instructions --skill tdd/audit`、`analyze`、`drift`、`task done` | `tdd/audit/parallel_tasks` | 是 |
| ingest | `list[--parked]`、`unpark`、`instructions <產物>`、`status`、`analyze`、`validate` | `parallel_tasks`；`locale/context/rules`(注入) | 是（只改產物） |
| archive | `list`、`status`、`archive`（同步走 Task→sync-specs） | schema 名 | 是（移動/併規格） |
| analyze | `analyze`、`status`（被動觸發） | — | 否（唯讀） |
| verify | `list`、`status`、`instructions apply` | schema 名 | 否（唯讀） |
| drift | `list`、`drift` | — | 否（唯讀） |
| audit | `git diff HEAD` | — | 否（唯讀） |
| debug | `instructions --skill tdd`（條件） | `tdd` | 是（修 bug） |
| commit | `git`、`list`、（子流程）`archive`/`sync` | — | 是（git commit） |
| ask | `search` | — | 否（唯讀） |

**呼叫 instruction 的兩種方式**：
- `spectra instructions <產物> --change X --json` → 取「該怎麼寫這份產物」（propose、ingest 用）。
- `spectra instructions apply --change X --json` → 取「實作模式」的脈絡（apply、verify 用）。
- `spectra instructions --skill <名稱>` → 直接取某個嵌入式技能本體（apply/debug 取 tdd、audit）。

---

## 11. 進階引擎內部

以下是從二進位字串與受控實驗確認的內部機制。

### 11.1 analyze 引擎（4 維一致性分析）

來源 `crates/spectra-core/src/analyzer.rs`。四個維度，每個發現有 id 前綴、嚴重度（Critical/Warning/Suggestion）、i18n 訊息鍵：

| 維度 | id | 代表性檢查（訊息鍵 → 意義） |
|---|---|---|
| **Gaps（缺口）** | GAP- | `gapNoProposal`（有 spec 卻無 proposal）、`gapNoMainSpec`（MODIFIED 指向的能力找不到正典 spec）、`gapModifiedNotFound`（要改的需求在正典找不到） |
| **Coverage（覆蓋）** | COV- | `covMissingSpec`（能力沒對應 spec 檔）、`covMissingTask`（需求沒對應 task）、`covDeltaValidation`（delta 結構錯） |
| **Ambiguity（模糊）** | AMB- | `ambWeakLanguage`（用了模糊詞，建議換 SHALL）、`ambAbstractScenario`（情境無具體 Example）、`ambNoScenario`（需求無情境） |
| **Consistency（一致）** | CON- | `conDesignNotInTasks`（design 主題沒被 tasks 引用） |

實測 demo 變更得到 4 個 `AMB`（Suggestion 級，情境缺 Example）。

### 11.2 drift 引擎（4 維漂移偵測）

來源 `crates/spectra-core/src/drift.rs`。從 design.md 抽出「程式碼錨點」，去 repo 找還在不在：
- 符號錨點用正則 `\b[A-Z][a-zA-Z0-9]+\b` 抽取，再用一份內建停用字表（`Context, State, Result, Error, Option, Vec, …, Rust, JSON, CLI, API` 等常見字）過濾掉誤判。
- 錨點類別：`FilePath, Symbol, Function, CliFlag`；斷錨原因：`function not found in repo`、`symbol not found in repo`、`file does not exist`、`cli flags unchecked`。
- 四維：`Time`（時間/陳舊）、`Structure`（斷錨）、`Tasks`（任務與外部 commit 衝突）、`Environment`（僅顯示、不計入總分）。
- 錨點檢查上限 `ANCHOR_CAP = 50`。
- 嚴重度 light/medium/heavy，`primary_recommendation` 例如 `spectra archive <name> --skip-specs`。

> 實測時因臨時專案剛 init、符號都不在 repo，drift 報告出現「20/20 anchors broken」、severity heavy——這是預期的（demo 內容是虛構的，程式庫裡當然沒有那些符號）。

### 11.3 preflight 引擎（起飛前檢查）

來源 `crates/spectra-core/src/preflight.rs`。用兩條正則檢查產物引用的檔案路徑是否為「真實、錨定」的路徑（限定副檔名 `rs|ts|tsx|jsx|svelte|md|json|yaml|toml|css|html|js`），相對片段會被當「非錨定」拒絕。這就是 propose 技能要求 Impact 路徑必須以專案根為基準的原因。輸出 `staleness{daysOld, isStale}`、`missingFiles`、`driftedFiles`。

### 11.4 archive 提升與還原

來源 `crates/spectra-core/src/archive.rs`。歸檔時：建立快照（`.spectra/snapshots/<archived>/created_specs.json` 記錄建了哪些正典 spec）→ 套用 delta（ADDED 附加、MODIFIED 覆寫、REMOVED 刪除、RENAMED 改名）→ 注入 `@trace`（`trace_parser.rs`，記 source/updated/code/test）→ 記錄身分 → 向量索引。「Snapshot created for unarchive support」表示可還原。

### 11.5 向量語意搜尋（平台限定）

- **只在 Apple M 系列晶片編譯／可用**。Windows 版根本沒編進向量引擎，`spectra search` 直接回 `Vector search is not available on this platform (requires Apple M-series)`，內部錯誤碼 `vector_not_compiled`。
- 三種錯誤狀態：`vector_not_compiled`（平台不支援）、`index_not_built`（索引未建）、`model_not_downloaded`（模型未下載），各有對應的繁中提示，導向桌面 App 的 Settings → Vector Search。
- 真正的嵌入模型／ML 執行階段在 **`app.exe`**（桌面 App），CLI 二進位裡沒有模型名。
- 「跨語言查詢（中英日）原生支援」。改 `spec_dir` 需重建索引。
- **注意**：二進位裡那些 BM25／tantivy／inverted-index 的字串**是 `demo` 指令的虛構範例內容**，不是 Spectra 真正的搜尋實作——別被誤導。

### 11.6 git worktree（隔離分支）

- `.spectra.yaml` 的 `worktree: true` 啟用；`worktrees_dir` 預設 `.spectra/worktrees`。
- 用 `git2` 函式庫（0.20.4）。分支前綴 `spx/`（Spectra）與 `opsx/`（OpenSpec 舊）。
- 若同一變更同時存在於主 repo 與 worktree，會報衝突要你刪掉其一。
- `instructions apply --json` 會多一個 `worktreePath` 欄告訴 AI 該在哪個 worktree 工作。
- 資料表 `worktree_artifact_ownership` 追蹤所有權。

### 11.7 SQLite 中繼資料庫

- 位置 `.spectra/spectra.db`（舊名 `.vector-search.db`），驅動 `rusqlite` 0.38。有完整的舊資料庫遷移／合併機制（檔案鎖、完整性檢查、copy-to-tmp-then-rename）。
- 資料表：`parked_changes`、`archived_cache`、`shared_changes`、`worktree_artifact_ownership`、`documents`、`change_sort_order`、`in_progress_change`、`agent_input_history`。
- `in_progress_change(change_id TEXT PRIMARY KEY)`——`in-progress add` 寫一列。
- `parked_changes(change_id, original_modified, tasks_total, tasks_done, has_proposal, has_tasks, created_by, created_with)`——park 把變更目錄實際移走、用這列記錄好還原（含原始修改時間）。

### 11.8 touched 追蹤（task→檔案）

`task done` 把該 task 動過的程式檔記下來，供歸檔時注入 `@trace` 的 `code:` 清單、以及 commit 技能分類用。實測 `task done` 後產生 `.spectra/touched/<change>.json`，結構：

```json
{ "change": "...", "touched": [ { "task_id": "1", "task_desc": "...", "files": [ ... ] } ] }
```

---

## 12. OpenSpec 功能與 CLI 完整參考

> 資料來源：OpenSpec 的 GitHub README、`docs/cli.md`、`docs/concepts.md`、`schemas/spec-driven/`、`src/` 原始碼、GitHub API（皆為一手來源）。

### 12.1 基本資訊

- npm 套件 `@fission-ai/openspec`，需 Node.js ≥ 20.19.0；安裝 `npm install -g @fission-ai/openspec@latest`，再 `openspec init`。CLI 二進位名 `openspec`。
- 描述「Spec-driven development (SDD) for AI coding assistants」；MIT；TypeScript（約 99%）；建立於 2025-08-05；約 5.7 萬星、4 千 fork。
- 哲學：**流動而非僵硬**（無階段閘門）、**迭代而非瀑布**、**棕地優先**（適用既有程式庫）。
- 支援 31 種 AI 工具（Claude Code、Cursor、Copilot、Codex、Gemini CLI、Windsurf…）。

### 12.2 OpenSpec 的兩個世代（重要）

OpenSpec 的 repo 同時有「兩代」，**Spectra 對應的是「經典版」的格式與主幹**：
- **經典版**（tag v0.9.0，保留為 `README_OLD.md`）：用 `openspec/project.md` ＋ `openspec/AGENTS.md`，`openspec` CLI（init/list/show/validate/archive/update），三階段 create → apply → archive。
- **現行「流動／opsx」版**（main 分支）：用 `schemas/spec-driven/`（schema.yaml ＋ 模板）、**`/opsx:*` 斜線指令**（explore、propose/new、ff/continue、apply、verify、sync、archive…）、`config.yaml`/`.openspec.yaml`。

### 12.3 OpenSpec 的 AI 工作流（現行 opsx 版）

斜線命名空間是 **`/opsx:`**（不是 `/openspec:`）：
- `/opsx:explore`——AI 當思考夥伴先審視程式碼、權衡選項。
- `/opsx:propose "<要做什麼>"`——建立變更資料夾，草擬 proposal、specs、design、tasks。
- `/opsx:apply`——實作任務。
- `/opsx:archive`（＋自動的 `/opsx:sync`）——歸檔並把 delta 併入正典。
- 擴充 profile 還有 `/opsx:new`、`/opsx:continue`、`/opsx:ff`、`/opsx:verify`、`/opsx:bulk-archive`、`/opsx:onboard`。

### 12.4 OpenSpec CLI 命令（現行 main，Commander 框架）

設定/瀏覽：`init`、`update`、`list`、`view`（互動式儀表板）、`show`、`validate`、`archive`、`status`。
工作流引擎：`new change`、`instructions`、`templates`、`schemas`、`schema (init/fork/validate/which)`。
設定：`config (path/list/get/set/unset/reset/edit/profile)`。
**Stores（測試版，OpenSpec 獨有）**：`store (setup/register/unregister/remove/list/doctor)`、`doctor`、`context`——可註冊/分享獨立的規格庫。
**Worksets（OpenSpec 獨有）**：`workset (create/list/open/remove)`——本機具名工作視圖。
其他：`feedback`、`completion (generate/install/uninstall)`。
已棄用：`change` 命令群（改用頂層 `list/show/validate`）。
**沒有 `diff` 命令**（已從現行原始碼確認不存在）。

### 12.5 OpenSpec 的格式（Spectra 沿用）

- 正典 spec：`# [Domain] Specification` / `## Purpose` / `## Requirements` / `### Requirement:` / `#### Scenario:`（WHEN/THEN）。
- delta：四種 `## ADDED/MODIFIED/REMOVED/RENAMED Requirements`。
- **同樣的鐵則**：`#### Scenario:` 剛好 4 個井字號、用錯會「無聲失敗」；MODIFIED 要貼整條；REMOVED 要含 Reason+Migration；RENAMED 用 FROM:/TO:；每條需求至少一個 scenario；用 SHALL/MUST 避免 should/may。
- proposal：`## Why / ## What Changes / ## Capabilities / ## Impact`。design（可選）：`## Context / Goals-Non-Goals / Decisions / Risks`。tasks：`## 1. 群組` + `- [ ] 1.1`。
- 歸檔合併語意：ADDED 附加、MODIFIED 覆寫、REMOVED 刪除、RENAMED 改名，資料夾移到 `changes/archive/YYYY-MM-DD-<name>/`。

---

## 13. Spectra vs. OpenSpec 對照表

| 面向 | OpenSpec | Spectra |
|---|---|---|
| 實作語言 | TypeScript / Node.js | **Rust**（原生執行檔，免 Node） |
| 散布方式 | npm `@fission-ai/openspec` | 安裝程式（`spectra.exe` + 桌面 `app.exe`） |
| 桌面 GUI | 無（有 `view` 終端儀表板） | **有**（Tauri App） |
| AI 介面 | `/opsx:*` 斜線指令 | **`/spectra-*` 技能**（＋可選 `/spectra:*` 斜線指令、`$spectra-*` 其他 agent） |
| 工作流 | explore → propose → apply → archive | discuss? → propose → apply ⇄ ingest → archive |
| 探索/討論 | `/opsx:explore` | `/spectra-discuss`（對應概念） |
| 中途改需求 | （無明確獨立階段） | **`/spectra-ingest`**（新增） |
| 目錄結構 | `openspec/specs` + `openspec/changes` | **相同** |
| 專案情境檔 | 經典版 `project.md`；現行版 `config.yaml` | `openspec/config.yaml` 的 `context`；**無 `project.md`** |
| delta 格式 | ADDED/MODIFIED/REMOVED/RENAMED | **相同**（再加 `##### Example:` SBE、禁用詞分析器、`@trace` 錨點） |
| 規格語言 | — | **locale 多語系**（但 spec 永遠英文） |
| 暫存變更 | 無 | **park / unpark（暫存）**（新增） |
| 漂移偵測 | 無 | **`spectra drift`**（新增） |
| 一致性分析 | `validate`（結構） | `validate` ＋ **`analyze`（4 維）**（新增） |
| 安全稽核 | 無 | **`/spectra-audit`**（新增） |
| 除錯紀律 | 無 | **`/spectra-debug`**（新增） |
| 單一變更提交 | 無 | **`/spectra-commit`**（新增） |
| 驗證實作 | `/opsx:verify` | `/spectra-verify`（兩者皆有） |
| 向量語意搜尋 | 無 | **`spectra search`**（限 Apple M 系列） |
| git worktree | 無 | **可選**（隔離分支） |
| 共享規格庫／工作視圖 | **Stores、Worksets** | 無對應 |
| TDD/audit/並行開關 | 無 | **`.spectra.yaml` tdd/audit/parallel_tasks** |

**Spectra「保留」自 OpenSpec**：`openspec/` 目錄、specs 正典 vs changes delta 的模型、四種 delta 操作格式、proposal/design/tasks 產物、propose→apply→archive 主幹、SHALL/Scenario/4-井字號等鐵則。

**Spectra「新增」**：技能系統、park/unpark、drift、analyze、audit、debug、commit、向量搜尋、worktree、locale、`.spectra.yaml` 開關、桌面 App。

**OpenSpec 有而 Spectra 無**：Stores（共享規格庫）、Worksets（工作視圖）、`view` 互動儀表板。

> 校正：`verify` **不是** Spectra 獨有（OpenSpec 也有 `/opsx:verify`）。真正 Spectra 獨有的診斷類技能是 audit、debug、drift、analyze、commit。

---

## 14. 端到端範例：一次完整的變更生命週期

把上面所有東西串起來，一個典型流程（以「加上深色模式」為例）：

1. **（可選）討論** `/spectra-discuss 深色模式要全域切換還是逐頁？`
   → 技能掃描原始碼、列假設、收斂出「全域切換」決策，寫進 design.md 或直接接 propose。

2. **提案** `/spectra-propose 加上深色模式`
   → 技能分類為 Feature → `spectra new change add-dark-mode --agent claude` → 逐產物呼叫 `spectra instructions <產物> --json`（CLI 現場注入 locale=繁中、context、rules）→ AI 依模板寫 proposal/specs/design/tasks → `spectra new artifact ... --stdin` 寫入 → `spectra analyze`／`validate` 把關 → **`spectra park add-dark-mode`**（自動暫存）。

3. **實作** `/spectra-apply add-dark-mode`
   → `spectra unpark` 自動取回 → `spectra in-progress add` → `spectra instructions apply --json`（含 preflight）→ 三道前置閘 → 讀 `.spectra.yaml`（tdd? audit? parallel?）→ 逐 task 實作、`spectra task done 1`、`spectra task done 2`…（翻勾選框＋記錄動過的檔）。

4. **（中途需求變了）** Plan 模式產新計畫 → `/spectra-ingest add-dark-mode`
   → 把新脈絡合併進產物、保留已完成的 `[x]` → 回到 apply 續作。

5. **（可選）驗證／稽核** `/spectra-verify add-dark-mode`、`/spectra-audit`
   → 確認實作符合 spec、檢查安全銳角。

6. **歸檔** `/spectra-archive add-dark-mode`
   → 檢查產物/任務完成度 → 評估 delta 同步 → **`spectra archive add-dark-mode`**：把 delta 併入 `openspec/specs/<能力>/spec.md`（注入 @trace）、移到 `changes/archive/2026-06-29-add-dark-mode/`、建還原快照。

7. **（可選）提交** `/spectra-commit add-dark-mode`
   → 只把此變更相關的產物＋原始檔 `git add` 後 commit（`spectra(add-dark-mode): ...`）。

---

## 15. 已知細節、平台限制與隱藏功能

從反組譯與對抗式驗證發現的細節（對使用者很實用）：

1. **向量搜尋在 Windows 完全不可用**——需 Apple M 系列。所以 `/spectra-ask` 在你的系統上實際上只會回「不支援」訊息。索引建立與模型下載要在桌面 App 的 Settings 裡做。

2. **`spectra sync` 不是真的 CLI 子命令**——commit 技能本體裡寫了 `spectra sync <name>`，但 clap 子命令表裡沒有這個指令，跑了會報「子命令未識別」。真正的同步是**agent 驅動的 `spectra-sync-specs` 技能**（透過 `/spectra:sync` 或 Task 工具觸發，由 AI 讀 delta 直接改正典）。archive 技能做對了（走 Task→sync-specs），commit 技能體裡的這段是個 bug。

3. **apply 休眠檢查的路徑有潛在 bug**——apply 技能 Step 3d 偵測休眠時硬編了 `git log -1 --format=%at -- docs/specs/changes/<name>/`，但你的專案用預設的 `openspec/`（不是 `docs/specs/`）。這個路徑不會解析到，所以「近 3 天 0 commit」那半個條件會誤判。（這是 v2.3.1 內嵌技能體的潛在瑕疵，只在休眠閘被觸發時才有影響。）

4. **14 個嵌入技能，但只生成 12 個檔**——`sync`、`clarify`、`tdd` 是內部用、不生成獨立 SKILL.md。`tdd`/`audit` 由 apply/debug 用 `instructions --skill` 取用；`clarify` 是「被動式釐清提問」技能。

5. **analyze 與 verify 只有技能、沒有斜線指令**——`claude_slash_commands: true` 只生成 10 個 `/spectra:X` 指令（少了 analyze、verify）。`.agents` 版技能也只有 10 個（少了這兩個）。

6. **三種呼叫前綴**：技能 `/spectra-名稱`、斜線指令 `/spectra:名稱`（冒號）、其他 agent `$spectra-名稱`。同一份本體用 `{{...}}` 佔位符在生成時替換。

7. **locale 只有兩種非英語**：`tw`（繁中）、`ja`（日文）。其他值會落回英文。

8. **`spectra config` 管的是全域設定**，不是專案的 `.spectra.yaml` 或 `openspec/config.yaml`——別搞混。

9. **直接跑 `spectra` 會試圖開桌面 App**；`self-update` 字串存在但似乎未在此版啟用。

10. **`update --force` 不會自動補上新工具的目錄**——即使 `.spectra.yaml` 的 `tools:` 列了多個工具，實測 `update` 只刷新已存在的（claude）指令檔。要新增工具產物可能需 `init --tools`。

---

## 16. 證據與方法

本分析的可信度建立在多重一手證據上：

- **CLI 直接探測**：對 `spectra.exe` 執行全部 23 個命令及其子命令的 `--help`、`--version`，以及 `schemas/templates/config/instructions/status/analyze/drift/validate` 等唯讀指令的 `--json` 輸出。
- **受控實驗**：在隔離的臨時專案做完整 `init → demo → new change → new artifact → instructions（含改 locale/context/rules 對照）→ task done → analyze/validate/drift → archive` 流程，直接觀察檔案系統變化與 CLI 行為（本文第 6.3 節的 instruction 注入證明即出於此）。
- **二進位靜態分析**：以 Python 萃取 8.35MB 二進位的約 11 萬筆 ASCII／UTF-8 字串，比對出 Rust 結構（`SpectraConfig`、`Schema`、`ChangeMetadata`…）、serde 欄位、SQLite 建表語句、正則、i18n 訊息鍵、錯誤字串、原始檔路徑等。
- **技能全文閱讀**：完整閱讀 12 個 `.claude/skills/spectra-*/SKILL.md`。
- **OpenSpec 一手來源**：抓取 GitHub 的 README、`docs/cli.md`、`docs/concepts.md`、`schemas/spec-driven/schema.yaml` 與模板、`src/cli/index.ts` 等原始碼，及 GitHub API 中繼資料。
- **對抗式交叉驗證**：以多個獨立代理反向查核每一條關鍵結論，已在本文中標出的校正包括：`spectra sync` 非真指令、apply 休眠路徑 bug、`verify` 非 Spectra 獨有、現行 OpenSpec 已無 `project.md`、二進位中 BM25/tantivy 字串為 demo 虛構內容等。

> 少數無法在 Windows 環境實證的部分（向量搜尋實際行為、桌面 App 內部、`self-update` 可否呼叫）已在文中明確標示為「推測／未證實」。

---

*本文件由反組譯與行為重現整理，僅供理解與互通用途。Spectra © kaochenlong；OpenSpec © Fission-AI（MIT）。*
