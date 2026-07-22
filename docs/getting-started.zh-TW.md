# Speclink Local Repo 入門

**繁體中文** · [English](getting-started.md)

這份教學使用目前已實作的 Local Repo 路徑，走完第一輪：**init → propose → apply → checks → archive**。規格位於 repo 的 `openspec/`，由 Git 協作，不需要 server。完整 Remote／Server 狀態見[產品能力狀態](product-status.zh-TW.md)。

## Before you start / 開始前

本範例假設需求已清楚：「新增 CSV export」。如果仍需比較方向、形成決策或保存取捨，先讀[完整 SDD 工作流](workflow.zh-TW.md)並使用 `discuss`；不要把每個問題都記成 discussion。

Speclink 有三個呼叫層級：

| Layer / 層級 | This guide uses / 本教學用法 | Responsibility / 責任 |
| --- | --- | --- |
| Claude skill | `/speclink-propose add-csv-export` | Agent 依 workflow 讀背景、產生 artifacts、驗證並在需要時詢問。 |
| Codex skill | `$speclink-propose add-csv-export` | 與 Claude 產生相同 Speclink artifacts，呼叫語法使用 Codex `$skill`。 |
| Direct CLI | `speclink status --change add-csv-export --json` | CLI／Host 是執行引擎；它管理 change、artifact DAG、tasks 與生命週期，不替使用者做需求判斷。 |

以下 Agent 指令請依你使用的 Host 二選一；CLI 區塊則可直接在 shell 執行。

## 1. Install / 安裝

在 Speclink 原始碼 repo，以 stable Rust toolchain 安裝 CLI：

```bash
cargo install --path crates/speclink-cli
speclink --version
```

`speclink --help` 應列出 `init`、`status`、`validate`、`analyze`、`drift`、`archive` 與 `discuss` 等目前命令。

## 2. Initialize / 初始化

切到要導入 Speclink 的 repo：

```bash
speclink init --tools claude,codex
```

這會建立 `openspec/`、`.speclink.yaml`、gitignored `.speclink/` 工作資料，並為選定 Host 產生 skills。查看目前狀態：

```bash
speclink list
speclink validate --specs --all --strict
```

若 repo 已有大量程式但沒有 canonical specs，先用 Claude `/speclink-onboard` 或 Codex `$speclink-onboard` 依目前行為建規格，再建立新 change。

## 3. Propose / 提案

在 Claude：

```text
/speclink-propose add-csv-export
```

在 Codex：

```text
$speclink-propose add-csv-export
```

Agent 會建立 change，逐一讀取 schema instructions，並完成 `applyRequires` 依賴鏈上的 artifacts。常見 spec-driven change 會有 proposal、delta specs、tasks，跨模組或有重要技術決策時才需要 design；**design 是條件式 artifact，不保證每個 change 固定產生四份檔案。**

隨時查看 DAG：

```bash
speclink status --change add-csv-export --json
```

進階使用者若要直接操作 CLI，底層流程是：

```bash
speclink new change add-csv-export
speclink instructions proposal --change add-csv-export --json
speclink new artifact proposal --change add-csv-export --stdin
```

最後一行會從 stdin 讀取符合 instructions template 的完整 Markdown。接著重新執行 `status`，只建立顯示為 ready 且 schema 實際需要的 artifacts；spec 使用 `speclink new artifact spec <capability> --change add-csv-export --stdin`。直接 CLI 適合已理解 artifact contract 的使用者，否則使用 Agent skill。

如果來源是 concluded discussion，Claude 使用 `/speclink-propose --from-discussion <slug>`，Codex 使用 `$speclink-propose --from-discussion <slug>`。其他轉為變更或併入既有 change 的路徑見 workflow，不在 happy path 展開。

## 4. Apply / 實作

artifacts 完成後，在 Claude：

```text
/speclink-apply add-csv-export
```

在 Codex：

```text
$speclink-apply add-csv-export
```

Agent 會讀 proposal／specs／design（若存在）／tasks，逐項實作與驗證。底層進度入口是：

```bash
speclink instructions apply --change add-csv-export --json
speclink task done --change add-csv-export 1
```

只有 task 的行為、實作契約與驗證目標都完成後才可 `task done`。若實作被回滾或勾錯，使用：

```bash
speclink task undone --change add-csv-export 1
```

當 apply instructions 回傳 `state: all_done`，才進入最終檢查。

## 5. Check / 檢查

檢查 artifact 一致性與結構：

```bash
speclink analyze add-csv-export --json
speclink validate add-csv-export
```

再執行專案自己的 tests、lint、build 或人工驗收。`validate`／`analyze` 只檢查 artifacts，不能取代 code correctness。

引擎內有 verify workflow asset，但此 repo 目前沒有生成可呼叫的 `/speclink-verify` 或 `$speclink-verify`。因此不要把它當成本教學指令；使用專案 tests、逐 Requirement／Scenario 對照與 `task done` evidence 完成實作驗證。完整限制見 product-status 的 Verify and task evidence 列。

## 6. Archive / 封存

所有 tasks 完成、artifacts valid、delta assumptions 未過時且實作檢查通過後，在 Claude：

```text
/speclink-archive add-csv-export
```

在 Codex：

```text
$speclink-archive add-csv-export
```

或直接執行：

```bash
speclink archive add-csv-export -y
```

封存會將 delta specs 合併至 canonical specs，並把 change 移至 `openspec/changes/archive/`。不要用 `--mark-tasks-complete` 或 `--no-validate` 跳過未完成工作。

## What was created / 產物位置

| Path / 路徑 | Meaning / 意義 |
| --- | --- |
| `openspec/specs/<capability>/spec.md` | canonical specs，目前行為真相 |
| `openspec/changes/<name>/` | active change 與 schema 所需 artifacts |
| `openspec/changes/archive/` | 已封存 changes 的稽核記錄 |
| `openspec/discussions/` | 需要決策時才建立的 discussion documents |
| `openspec/config.yaml` | workflow policy、context、rules、locale、TDD／audit |
| `.speclink.yaml` | workspace binding 與本機工具整合 |
| `.speclink/` | gitignored Context Projection、touched/evidence 等工作資料 |

## Leave the happy path / 離開主路徑

- 需求仍模糊：先 `discuss`。
- discussion 結論要快速建立 change 或併入既有 change：查[Discussion outcomes](workflow.zh-TW.md#discussion-outcomes--討論結論分流)。
- 實作中需求改變：`$speclink-ingest <change>`（Claude 對應 `/speclink-ingest`）。
- change 暫停後續作：先 `$speclink-drift <change>`（Claude 對應 `/speclink-drift`）。
- 要判斷 Server、Desktop Remote Workspace、Agent tools 是否可用：查[產品能力狀態](product-status.zh-TW.md)，不要從架構藍圖推論目前已交付。
