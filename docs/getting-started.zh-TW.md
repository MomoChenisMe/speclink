# Speclink Local Repo 入門

**繁體中文** · [English](getting-started.md)

照這份文件走完 Local Repo 的第一輪：**init → propose → apply → checks → archive**。規格放在 repo 的 `openspec/`，用 Git 協作，不需要 server。

每一步都附上預期輸出。你看到的應該和這裡寫的一致；不一致就代表有東西沒對上，先停下來看差在哪。本文只走主路徑，選用分支一律連到[完整 SDD 工作流](workflow.zh-TW.md)。

## Before you start / 開始前

本範例假設需求已經清楚：「新增 CSV export」。如果還在比較方向、需要形成決策，先走 `discuss`——別把每個問題都記成討論。

Agent 指令有兩種呼叫字面：Claude 用 `/speclink-*`，Codex 用 `$speclink-*`。Codex 的 `$` 是技能的明確呼叫寫法；打 `/skills` 也能從清單裡挑到同一個技能，兩種都走得通。

下面兩種字面都會列出，擇一即可。標成 shell 的區塊則是直接執行 CLI。技能、CLI 與 Host 三層各自負責什麼，見工作流文件的[呼叫層級](workflow.zh-TW.md#call-layers--呼叫層級)。

## 1. Install / 安裝

安裝 CLI，擇一：

```bash
# 安裝腳本（macOS／Linux）
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# 安裝腳本（Windows PowerShell）
irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex

# Homebrew（macOS／Linux）
brew install MomoChenisMe/tap/speclink
```

想要圖形介面的話，[Releases](https://github.com/MomoChenisMe/speclink/releases/latest) 有三平台的桌面安裝檔，內含同版 CLI。

已經用上面的腳本裝過 CLI 的人請注意：再裝桌面 app 會蓋掉那份執行檔。先看 [README 的安裝章節](../README.md#install--安裝)，再決定裝哪一個。裝完確認：

```bash
speclink --version
```

**預期輸出**：一行版本字串，形如 `speclink 0.1.0 (arm64, engine v1.x.y)`。`speclink --help` 會列出 `init`、`status`、`validate`、`analyze`、`drift`、`archive`、`discuss`、`review`、`verify` 等目前命令。

## 2. Initialize / 初始化

切到要導入 Speclink 的 repo：

```bash
speclink init --tools claude,codex
```

**預期輸出**：

```text
✓ Initialized at /path/to/your-repo/openspec
Generated files for: claude, codex
```

這會建立 `openspec/` 與 `.speclink.yaml`，為選定的 Host 產生技能檔（`.claude/skills/`、`.agents/skills/`），並把 `.speclink/` 加進 `.gitignore`。不會寫任何指令檔——`CLAUDE.md`、`AGENTS.md` 是你自己的檔案，流程路由由技能自身的 description 承載。`.speclink/` 本身不在這一步建立，之後有本機工作資料要落時才會出現。

**這是 Local 模式的產物。** `openspec/` 的結構刻意貼合 OpenSpec 慣例，方便你直接讀、直接改，也方便從 OpenSpec 搬過來：

```text
openspec/
├── config.yaml              工作流政策（locale、tdd、audit、worktree）
├── specs/<capability>/spec.md   正典規格，一個 capability 一份
├── changes/<名稱>/           進行中的變更（proposal、design、tasks、specs delta）
├── changes/archive/          已封存的變更
└── discussions/              討論記錄（Speclink 新增）
```

全部是純 Markdown 與 YAML，沒有資料庫，也沒有專屬格式。不裝 Speclink 也讀得懂、改得動，每次規格變動 Git diff 都看得出來。Speclink 只多放兩樣東西：`discussions/`，以及每個變更目錄裡的 `.openspec.yaml`——後者記生命週期 metadata，例如開工時間與來源討論。

這份結構相容性只適用 Local 模式。接上遠端之後規格的正典在 Store 裡，本機只留一份唯讀投影（`.speclink/context/`），不是可寫的檔案樹。

確認起點乾淨：

```bash
speclink list
speclink validate --specs --all --strict
```

**預期輸出**：`list` 印出 `No active changes.`；`validate` 在還沒有任何正典規格時**不印任何東西**且以 0 結束——沒有輸出就是通過。

如果 repo 已經有大量程式但沒有正典規格，先用 `/speclink-baseline`（Codex 為 `$speclink-baseline`）依目前行為建規格，再開新變更。

## 3. Propose / 提案

在 Claude：

```text
/speclink-propose add-csv-export
```

在 Codex：

```text
$speclink-propose add-csv-export
```

Agent 會建立變更、逐一讀取 schema instructions，並把 `applyRequires` 依賴鏈上的 artifacts 補完。隨時查 DAG：

```bash
speclink status --change add-csv-export --json
```

**預期輸出**：剛建立時只有 proposal 可寫，其餘被擋著——

```text
proposal → ready
design   → blocked
specs    → blocked
tasks    → blocked
```

補完之後會變成：

```text
proposal → done
design   → ready
specs    → done
tasks    → done
```

注意 `design` 停在 `ready` 而不是 `done`，`isComplete` 也仍是 `false`。**這是正常的。** design 是條件式 artifact，跨模組或有重要技術決策時才需要，而 `applyRequires` 只要求 `tasks`。換句話說，不是每個變更都固定產生四份檔案。

想直接用 CLI 操作的話，底層流程是：

```bash
speclink new change add-csv-export
speclink instructions proposal --change add-csv-export --json
speclink new artifact proposal --change add-csv-export --stdin
```

**預期輸出**（第一行）：

```text
✓ Created change: add-csv-export
  Path: /path/to/your-repo/openspec/changes/add-csv-export
  Schema: spec-driven
```

最後一行會從 stdin 讀入符合 instructions template 的完整 Markdown。規格則用 `speclink new artifact spec <capability> --change add-csv-export --stdin`。直接用 CLI 適合已經理解 artifact 契約的人；還不熟就用技能。

來源是已結論的討論時，改用 `/speclink-propose --from-discussion <slug>`。其他轉為變更或併入既有變更的路徑見[討論結論分流](workflow.zh-TW.md#discussion-outcomes--討論結論分流)。

## 4. Apply / 實作

artifacts 完成後，在 Claude：

```text
/speclink-apply add-csv-export
```

在 Codex：

```text
$speclink-apply add-csv-export
```

Agent 會讀 proposal、specs、design（若有）與 tasks，逐項實作並自行檢查。底層進度入口是：

```bash
speclink instructions apply --change add-csv-export --json
speclink task done --change add-csv-export 1
```

**預期輸出**：`task done` 會回報勾掉的是哪一項——

```text
✓ Task 1 marked as done: 1.1 Serialize report rows to CSV
```

只有這一項的行為、實作契約與該過的檢查都通過了，才可以勾。勾錯或實作被回滾時用 `speclink task undone --change add-csv-export 1`。不要直接改 `tasks.md`。

全部勾完後：

```bash
speclink instructions apply --change add-csv-export --json
speclink list
```

**預期輸出**：instructions 的 `state` 變成 `all_done`；`list` 顯示進度已滿——

```text
Changes:
  • add-csv-export [2/2] — Reports can only be read insid…
```

## 5. Check / 檢查

**多數時候你不必自己跑這一步。** `propose`、`apply` 與 `ingest` 三個技能都會自動跑 `analyze`：

- `propose` 與 `ingest`：收尾前跑，修到沒有 Critical 才寫入，然後再跑一次 `validate`
- `apply`：開始實作前跑一次，遇到 Critical 會停下來問你

自己下這兩行的時機有三個：想在流程之外臨時看一眼、Agent 沒跑起來要手動確認、或想在 CI 裡把關。

```bash
speclink analyze add-csv-export --json
speclink validate add-csv-export
```

**預期輸出**：`analyze` 逐維度回報，`validate` 印一行結果——

```text
Coverage    → 1 issue(s) found
Consistency → Skipped (insufficient artifacts)
Ambiguity   → 1 issue(s) found
Gaps        → Clean

✓ add-csv-export — valid
```

第一輪這兩條幾乎都會出現，各自的意思是：

- Coverage 的 `Requirement 'X' has no matching task`（Warning）——規格寫了一條要求，但任務清單沒有對應項目。
- Ambiguity 的 `Scenario 'X' has no concrete examples`（Suggestion）——場景只有敘述，沒有具體的 GIVEN／WHEN／THEN 值。

Consistency 顯示 `Skipped` 是因為沒寫 design。這個維度要跨 artifact 比對才有意義，少一份就不判。

Warning 與 Suggestion 不擋你往下走，但都該看過再決定。Critical 一定要先修 artifact 再實作。

接著跑專案自己的測試、lint、build 或人工驗收。`validate` 與 `analyze` 只看 artifacts，**不能取代 code correctness**。

實作面的把關有兩道可選的品質關卡：`/speclink-review` 看程式碼工藝，`/speclink-verify` 看是否符合規格。兩道都要跑就用 `/speclink-quality`。第一輪的低風險變更跳過它們是正當選擇。兩道關卡的判準、蓋章時序與必修集合規則見[完整 SDD 工作流](workflow.zh-TW.md)。

## 6. Archive / 封存

任務全部完成、artifacts valid、delta 假設沒過期，且你選擇要跑的品質關卡都結案之後，在 Claude：

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

**預期輸出**：

```text
✓ Archived: add-csv-export → <日期>-add-csv-export
Specs applied: csv-export (added: 1, modified: 0, removed: 0, renamed: 0)
Snapshot created for unarchive support.
```

封存把 delta specs 合併進正典規格，並把變更移到 `openspec/changes/archive/`。之後 `speclink list` 回到 `No active changes.`，同時 `openspec/specs/csv-export/` 出現。那就是這一輪的成果落點。

不要用 `--mark-tasks-complete` 或 `--no-validate` 跳過沒做完的工作。

## What was created / 產物位置

| Path / 路徑 | Meaning / 意義 |
| --- | --- |
| `openspec/specs/<capability>/spec.md` | 正典規格，目前行為的真相 |
| `openspec/changes/<name>/` | 進行中的變更與 schema 所需 artifacts |
| `openspec/changes/archive/` | 已封存變更的稽核記錄 |
| `openspec/discussions/` | 需要決策時才建立的討論記錄 |
| `openspec/config.yaml` | 工作流政策、脈絡、規則、語言、TDD／audit |
| `.speclink.yaml` | workspace 綁定與本機工具整合 |
| `.speclink/` | gitignored 的 Context Projection 與工作資料 |

## Leave the happy path / 離開主路徑

- 需求仍模糊：先走 `discuss`；講不出要改哪裡則走 `improve`。
- 討論結論要快速建立變更或併入既有變更：見[討論結論分流](workflow.zh-TW.md#discussion-outcomes--討論結論分流)。
- 實作中需求改變：`/speclink-ingest <change>`。
- 變更暫停後續作：先跑 `/speclink-drift <change>`。
- 要平行推多個變更：`/speclink-apply-with-worktree`，收尾 `/speclink-worktree-merge`。
- 要用共享的 Remote Store 而不是本地 repo：見[Remote 入門教學](remote-getting-started.zh-TW.md)。
- 要判斷某項能力今天能不能用：查[專案能力狀態](product-status.zh-TW.md)，不要從架構藍圖推論已交付。
