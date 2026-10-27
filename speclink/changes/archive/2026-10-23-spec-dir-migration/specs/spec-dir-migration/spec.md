## Purpose

規格資料夾的預設位置（`speclink/`）、舊佈局（`openspec/`）的相容與警告、`speclink update` 對既有專案的自動遷移、`speclink init` 對 OpenSpec 專案的匯入，以及技能檔依實際規格資料夾渲染；資料夾內的檔案形狀與變更資料檔名由 store-abstraction 承諾，本機轉遠端的遷移屬 workspace-migration。

## ADDED Requirements

### Requirement: 規格資料夾的預設位置與舊佈局相容

本機模式的規格資料夾 SHALL 依序決定：`.speclink.yaml` 明寫 `spec_dir` 時用它；否則專案根有作為規格資料夾的 `speclink/`（其中有 `changes/`、`specs/`、`discussions/` 或 `config.yaml`）時用 `speclink/`；否則有 `openspec/` 時用 `openspec/`（舊佈局）；都沒有時用 `speclink/`。專案根的判定 SHALL 認得含有 `.speclink.yaml`、作為規格資料夾的 `speclink/`、`openspec/` 或 `.speclink.remote.yaml` 的資料夾；只是名為 `speclink` 的其他資料夾（例如 speclink repo 的 clone）SHALL NOT 讓所在資料夾成為專案根。未明寫 `spec_dir` 時，除 `speclink init` 與 `speclink update`（它們自己處理佈局）之外，CLI SHALL 在下列情況於 stderr 每次執行印一行固定開頭 `speclink: warning:` 的警告，且 SHALL NOT 改變 stdout 與 exit code：有 `.speclink.yaml`、只有 `openspec/` → 提示執行 `speclink update` 把資料夾搬到 `speclink/`；沒有 `.speclink.yaml`、只有 `openspec/` → 提示執行 `speclink init` 匯入；`speclink/` 與 `openspec/` 並存 → 說明 `openspec/` 不會被讀取、可能是舊版 speclink 建的，請升級後把內容搬進 `speclink/`。明寫 `spec_dir` 與遠端模式 SHALL NOT 印這些警告。

#### Scenario: 新專案建立 speclink 資料夾

- **WHEN** 在沒有任何規格資料夾的 git repo 執行 `speclink init --tools claude`
- **THEN** exit code 為 0，專案根建立 `speclink/`（含 `specs/`、`changes/archive/` 與 `config.yaml`）與 `.speclink.yaml`，`.speclink.yaml` 沒有啟用的 `spec_dir` 行，沒有建立 `openspec/`

#### Scenario: 舊佈局照常可用並提示遷移

- **WHEN** 專案有 `.speclink.yaml`（沒有 `spec_dir`）與 `openspec/`、沒有 `speclink/`，執行 `speclink list --json`
- **THEN** exit code 為 0，stdout 與升級前的版本位元級一致（讀的是 `openspec/`），stderr 有一行 `speclink: warning:` 開頭、提示執行 `speclink update` 的警告

#### Scenario: OpenSpec 專案提示匯入

- **WHEN** 專案沒有 `.speclink.yaml`、只有 `openspec/`，執行 `speclink list`
- **THEN** exit code 為 0，列出 `openspec/` 裡的變更，stderr 有一行提示執行 `speclink init` 匯入的警告

#### Scenario: 新舊資料夾並存

- **WHEN** 專案有 `.speclink.yaml`（沒有 `spec_dir`）、`speclink/` 與 `openspec/`，`openspec/changes/` 下有變更 legacy-only，執行 `speclink list --json`
- **THEN** exit code 為 0，stdout 只列 `speclink/` 裡的變更、沒有 legacy-only，stderr 有一行說明 `openspec/` 不會被讀取的警告

#### Scenario: 上層的 speclink clone 不是專案根

- **WHEN** `~/code/` 底下有 speclink repo 的 clone `~/code/speclink/`（其中沒有 `changes/`、`specs/`、`discussions/` 或 `config.yaml`）與未初始化的專案 `~/code/other/`，在 `~/code/other/src/` 執行 `speclink list`
- **THEN** speclink 回報不在專案裡，沒有把 `~/code/` 當成專案根

#### Scenario: 明寫 spec_dir 不警告

- **WHEN** `.speclink.yaml` 含 `spec_dir: openspec`，專案只有 `openspec/`，執行 `speclink list`
- **THEN** exit code 為 0，讀取 `openspec/`，stderr 沒有任何遷移或匯入警告

### Requirement: update 自動遷移舊佈局

`speclink update` SHALL 在同步技能檔之前檢查遷移條件：專案根有 `.speclink.yaml` 且其中沒有啟用的 `spec_dir`、專案根有 `openspec/`、沒有作為規格資料夾的 `speclink/`，且不是遠端模式。沒有 `.speclink.yaml` 的 OpenSpec 專案 SHALL NOT 由 `update` 遷移：`update` 不移動任何檔案，stdout 提示改用 `speclink init` 匯入。條件成立時：

- 目前在 linked worktree 裡、`git worktree list` 有主目錄以外的 worktree、`openspec/` 有未提交的修改（含未追蹤檔案）、`openspec` 是符號連結，或 `speclink` 已被規格資料夾以外的東西（例如檔案）佔用時，SHALL 暫緩：不移動任何檔案，stdout 說明暫緩原因（列出其他 worktree 的路徑，或說明有未提交修改，或說明要回主目錄執行），技能檔照常以 `openspec/` 同步，exit code 為 0。不在 git repo 裡時不做這兩項檢查。
- 否則 SHALL 把 `openspec/` 改名為 `speclink/`，再把其中每個變更（進行中與已封存）的 `.openspec.yaml` 改名為 `.meta.yaml`，檔案內容逐位元組不變；SHALL NOT 寫入 `spec_dir`；SHALL NOT 執行 git commit。資料夾改名失敗時 SHALL 以非零 exit code 結束且沒有任何檔案被移動；資料夾改名成功後某個資料檔改名失敗時，SHALL 回報失敗的路徑並以非零 exit code 結束，已改名的部分保留（讀取順序仍讀得到舊檔名），再次執行 `speclink update` SHALL 繼續改名 `speclink/` 內殘留的 `.openspec.yaml`。
- 遷移完成後，技能檔 SHALL 以 `speclink/` 同步；stdout SHALL 依序提示：資料夾與資料檔的改名結果（含改名的資料檔數）、要 commit 的路徑（`speclink/` 新增、`openspec/` 刪除）、團隊成員要先升級 speclink 再 pull 這個 commit、自己的腳本、CI 設定與 `.gitignore` 裡寫到 `openspec/` 的地方要自己改。

規格資料夾為 `speclink/`（未明寫 `spec_dir`）且其中有殘留的 `.openspec.yaml` 時，`speclink update` SHALL 依同樣的暫緩條件把它們改名為 `.meta.yaml`；同一個變更資料夾已有 `.meta.yaml` 時，旁邊的 `.openspec.yaml` 是過時的舊檔，SHALL NOT 改名覆蓋。明寫 `spec_dir`、遠端模式與已沒有舊佈局的專案 SHALL 不做任何遷移動作。

#### Scenario: 乾淨的舊專案自動遷移

- **WHEN** git repo 已 commit、沒有其他 worktree，`.speclink.yaml` 沒有 `spec_dir`，`openspec/` 有 2 個進行中變更與 3 個已封存變更，執行 `speclink update`
- **THEN** exit code 為 0；`openspec/` 不存在、`speclink/` 存在；5 個變更資料夾都只有 `.meta.yaml`，內容與原 `.openspec.yaml` 逐位元組相同；`.speclink.yaml` 沒有新增 `spec_dir`；git 沒有新的 commit；stdout 含改名的資料檔數 5、要 commit 的路徑、團隊成員先升級的提醒與自訂腳本的提醒；技能檔裡的規格路徑為 `speclink/`

#### Scenario: 有其他 worktree 時暫緩

- **WHEN** 符合遷移條件，但 `git worktree list` 另有一個 worktree，執行 `speclink update`
- **THEN** exit code 為 0，`openspec/` 原封不動，stdout 說明因其他 worktree 暫緩並列出其路徑，技能檔照常同步且規格路徑為 `openspec/`

#### Scenario: 有未提交修改時暫緩

- **WHEN** 符合遷移條件，但 `openspec/changes/demo/tasks.md` 有未提交的修改，執行 `speclink update`
- **THEN** exit code 為 0，`openspec/` 原封不動，stdout 說明因 `openspec/` 有未提交修改而暫緩

#### Scenario: linked worktree 裡不遷移

- **WHEN** 在某個 linked worktree 裡執行 `speclink update`，該 worktree 只有 `openspec/`
- **THEN** exit code 為 0，沒有移動任何檔案，stdout 說明要回主目錄執行遷移

#### Scenario: 殘留的舊檔名被整理

- **WHEN** 專案已是 `speclink/`，`speclink/changes/late/` 只有 `.openspec.yaml`（例如從舊分支合併進來），沒有其他 worktree 與未提交修改，執行 `speclink update`
- **THEN** exit code 為 0，`speclink/changes/late/.openspec.yaml` 改名為 `.meta.yaml`、內容不變，stdout 說明改名了 1 個資料檔

#### Scenario: OpenSpec 專案執行 update 不遷移

- **WHEN** 已 commit 的 OpenSpec 專案沒有 `.speclink.yaml`、只有 `openspec/`，執行 `speclink update`
- **THEN** exit code 為 0，`openspec/` 原封不動，沒有建立 `speclink/` 或 `.speclink.yaml`，stdout 提示執行 `speclink init` 匯入

#### Scenario: 明寫 spec_dir 不遷移

- **WHEN** `.speclink.yaml` 含 `spec_dir: openspec`，執行 `speclink update`
- **THEN** exit code 為 0，沒有移動任何檔案，stdout 沒有遷移相關的提示

### Requirement: init 匯入 OpenSpec 專案

在有 `openspec/`、沒有 `.speclink.yaml` 的資料夾執行 `speclink init` 時，系統 SHALL 視為匯入 OpenSpec 專案並自動完成，不另外詢問：以與 `speclink update` 遷移相同的程式把 `openspec/` 改名為 `speclink/`、把每個 `.openspec.yaml` 改名為 `.meta.yaml`；SHALL 保留既有的 `config.yaml`、`schemas/`、正式規格與變更，SHALL NOT 以範本覆蓋任何既有檔案；再照一般 init 補上 `.speclink.yaml`、`.gitignore` 設定與所選工具的技能檔。在 git repo 裡、有主目錄以外的 worktree 或 `openspec/` 有未提交修改時，init SHALL 以非零 exit code 拒絕、stderr 說明原因、零寫入。匯入成功時 stdout SHALL 提示要 commit 的路徑、OpenSpec CLI 從此讀不到這個專案、OpenSpec 自己安裝的技能與指令檔不會被移動或刪除。已有 `.speclink.yaml` 的資料夾 SHALL 維持現行的 Already initialized 行為。

#### Scenario: 匯入 OpenSpec 專案

- **WHEN** 已 commit 的 OpenSpec 專案有 `openspec/config.yaml`（含自訂的 context 與 rules）、`openspec/schemas/my-flow/`、兩份正式規格與一個含 `.openspec.yaml` 的變更，執行 `speclink init --tools claude`
- **THEN** exit code 為 0；`speclink/config.yaml` 與原檔逐位元組相同，`speclink/schemas/my-flow/`、正式規格與變更都在，變更資料檔為 `.meta.yaml`；`.speclink.yaml` 與 claude 技能檔建立；`openspec/` 不存在；stdout 含要 commit 的路徑、OpenSpec CLI 讀不到的提醒與 OpenSpec 技能檔需自行刪除的提醒

#### Scenario: 有未提交修改時拒絕匯入

- **WHEN** OpenSpec 專案的 `openspec/changes/demo/proposal.md` 有未提交修改，執行 `speclink init --tools claude`
- **THEN** exit code 非零，stderr 說明 `openspec/` 有未提交修改，沒有建立 `.speclink.yaml`、`speclink/` 或任何技能檔

#### Scenario: 已初始化的專案維持現行行為

- **WHEN** 已有 `.speclink.yaml` 的專案執行 `speclink init --tools claude`（不帶 `--force`）
- **THEN** exit code 非零，stderr 為現行的 Already initialized 訊息，沒有任何檔案被改動

### Requirement: 技能檔依實際規格資料夾渲染

`speclink init`、`speclink update` 與其他同步技能檔的入口 SHALL 以專案實際使用的規格資料夾（依「規格資料夾的預設位置與舊佈局相容」決定）渲染技能原稿裡的 `{{SPEC_DIR}}`。技能原稿 SHALL NOT 寫死 `openspec/`，變更資料檔一律寫作 `.meta.yaml`。

#### Scenario: 遷移後的技能路徑

- **WHEN** 專案以 `speclink/` 為規格資料夾，執行 `speclink update`
- **THEN** 生成的 speclink-commit 技能檔中的範例路徑以 `speclink/changes/<name>/` 開頭，沒有 `openspec/`

#### Scenario: 暫緩中的專案仍渲染舊路徑

- **WHEN** 專案因有其他 worktree 而暫緩遷移，仍只有 `openspec/`，執行 `speclink update`
- **THEN** 生成的技能檔中的規格路徑為 `openspec/`

#### Scenario: 自訂 spec_dir

- **WHEN** `.speclink.yaml` 含 `spec_dir: docs/specs`，執行 `speclink update`
- **THEN** 生成的技能檔中的規格路徑為 `docs/specs/`
