---
title: 規格資料夾與舊佈局遷移
section: 開始使用
order: 35
keywords: [規格資料夾, speclink/, openspec/, 舊佈局, 遷移, 匯入, OpenSpec, spec_dir, .meta.yaml, speclink update, speclink init]
sources: [spec-dir-migration, "user-documentation#入門教學與設定說明載明規格資料夾的遷移", "user-documentation#使用者文件載明本地產物的純文字可攜性"]
generated: 2026-10-27T11:03:57+08:00
---

# 規格資料夾與舊佈局遷移

新專案的規格資料夾是 `speclink/`。舊版 Speclink 建立的專案，以及 OpenSpec 專案，用的是 `openspec/` 資料夾，每個變更的狀態檔叫 `.openspec.yaml`。這一頁說明 Speclink 怎麼決定讀哪個資料夾、遇到舊佈局時會提示什麼，以及怎麼把它搬成 `speclink/` 與 `.meta.yaml`。

## Speclink 讀哪個資料夾

本地模式依下面的順序決定，取第一條成立的：

1. `.speclink.yaml` 明寫了 `spec_dir`：用它。
2. 專案根有作為規格資料夾的 `speclink/`（裡面有 `changes/`、`specs/`、`discussions/` 或 `config.yaml`）：用 `speclink/`。
3. 有 `openspec/`：用 `openspec/`，這是舊佈局。
4. 都沒有：用 `speclink/`。

Speclink 從你所在的資料夾往上找專案根。含有 `.speclink.yaml`、作為規格資料夾的 `speclink/`、`openspec/` 或 `.speclink.remote.yaml` 的資料夾，才算專案根。只是名字叫 `speclink` 的其他資料夾不算。例如 `~/code/` 底下有一份 speclink repo 的 clone `~/code/speclink/`（裡面沒有 `changes/`、`specs/`、`discussions/` 或 `config.yaml`），你在還沒初始化的 `~/code/other/src/` 執行 `speclink list`，Speclink 會回報你不在專案裡，不會把 `~/code/` 當成專案根。

## 舊佈局的警告

沒有明寫 `spec_dir` 時，CLI 遇到下面三種情況，每次執行都在 stderr 印一行以 `speclink: warning:` 開頭的警告。stdout 與 exit code 不受影響，指令照常做完。`speclink init` 與 `speclink update` 自己處理佈局，不印這些警告。

| 情況 | 讀哪裡 | 警告說什麼 |
| --- | --- | --- |
| 有 `.speclink.yaml`，只有 `openspec/` | `openspec/`，輸出與升級前完全相同 | 提示執行 `speclink update`，把資料夾搬到 `speclink/` |
| 沒有 `.speclink.yaml`，只有 `openspec/` | `openspec/` | 提示執行 `speclink init` 匯入 |
| `speclink/` 與 `openspec/` 並存 | 只讀 `speclink/` | 說明 `openspec/` 不會被讀取、可能是舊版 Speclink 建的，請升級後把內容搬進 `speclink/` |

例：並存時，`openspec/changes/` 底下有一個變更 legacy-only，`speclink list --json` 只列 `speclink/` 裡的變更，不會出現 legacy-only。

明寫了 `spec_dir` 時，以及 remote 模式，都不印這些警告。

## 用 speclink update 搬過來

舊版 Speclink 建立的專案，升級後執行一次：

```
speclink update
```

`update` 在同步技能檔之前，先檢查四個條件：專案根有 `.speclink.yaml` 而且沒有啟用的 `spec_dir`、有 `openspec/`、沒有作為規格資料夾的 `speclink/`、不是 remote 模式。四個都成立才遷移。

> [!NOTE]
> 沒有 `.speclink.yaml` 的 OpenSpec 專案，`update` 不會搬：它不移動任何檔案，只在 stdout 提示你改用 `speclink init` 匯入（見下一節）。

### 什麼時候暫緩

下面任一種情況，`update` 暫緩遷移：不移動任何檔案，在 stdout 說明原因，技能檔照常以 `openspec/` 同步，exit code 是 0。

- 你在 linked worktree 裡執行：說明要回主目錄執行。
- `git worktree list` 有主目錄以外的 worktree：列出那些 worktree 的路徑。
- `openspec/` 有未提交的修改（含未追蹤的檔案）：說明有未提交修改。例如 `openspec/changes/demo/tasks.md` 改了還沒 commit。
- `openspec` 是符號連結。
- `speclink` 這個名字已被規格資料夾以外的東西佔用，例如一個檔案。

不在 git repo 裡時，不檢查 worktree 與未提交修改這兩項。

### 搬的時候做什麼

條件成立、也沒有暫緩時：

1. 把 `openspec/` 改名為 `speclink/`。
2. 把裡面每個變更（進行中與已封存）的 `.openspec.yaml` 改名為 `.meta.yaml`，檔案內容逐位元組不變。
3. 以 `speclink/` 同步技能檔。

它不會寫入 `spec_dir`，也不會幫你 git commit。

搬完之後，stdout 依序提醒你：

1. 資料夾與資料檔的改名結果，含改名的資料檔數。
2. 要 commit 的路徑：`speclink/` 新增、`openspec/` 刪除。
3. 團隊成員要先升級 Speclink，再 pull 這個 commit。
4. 你自己的腳本、CI 設定與 `.gitignore` 裡寫到 `openspec/` 的地方，要自己改。

例：已 commit、沒有其他 worktree、`openspec/` 有 2 個進行中變更與 3 個已封存變更的專案，執行 `speclink update` 後，`openspec/` 不見了、`speclink/` 出現，5 個變更資料夾都只有 `.meta.yaml`，stdout 寫明改名了 5 個資料檔。

> [!WARNING]
> 搬完要自己 commit。請團隊成員先升級 Speclink，再 pull 這個 commit。

### 搬到一半失敗

- 資料夾改名失敗：以非零 exit code 結束，任何檔案都沒有被移動。
- 資料夾改名成功、某個資料檔改名失敗：回報失敗的路徑，以非零 exit code 結束。已改名的部分保留，Speclink 仍讀得到舊檔名。再執行一次 `speclink update`，會繼續改名 `speclink/` 裡殘留的 `.openspec.yaml`。

### 殘留的舊檔名

專案已經是 `speclink/`，但裡面還有 `.openspec.yaml` 時（例如從舊分支合併進來），`update` 依同樣的暫緩條件把它們改名為 `.meta.yaml`，並說明改名了幾個資料檔。同一個變更資料夾已經有 `.meta.yaml` 時，旁邊的 `.openspec.yaml` 是已不再使用的舊檔，不會被改名去覆蓋它。

明寫了 `spec_dir`、remote 模式，以及已經沒有舊佈局的專案，`update` 不做任何遷移動作，也不印遷移相關的提示。

## 在 OpenSpec 專案執行 speclink init：匯入

在有 `openspec/`、沒有 `.speclink.yaml` 的資料夾執行 `speclink init`，Speclink 會直接匯入，不另外問你：

1. 用與 `update` 相同的做法，把 `openspec/` 改名為 `speclink/`、每個 `.openspec.yaml` 改名為 `.meta.yaml`。
2. 既有的 `config.yaml`（含你自訂的專案說明與產出規則）、`schemas/`、正式規格與變更都保留，不會被範本覆蓋。
3. 照一般的 init 補上 `.speclink.yaml`、`.gitignore` 設定與你選的工具的技能檔。

在 git repo 裡、有主目錄以外的 worktree，或 `openspec/` 有未提交修改時，init 以非零 exit code 拒絕，stderr 說明原因，什麼都不寫。例如 `openspec/changes/demo/proposal.md` 有未提交修改，就不會建立 `.speclink.yaml`、`speclink/` 或任何技能檔。

匯入成功時，stdout 提醒你三件事：要 commit 的路徑；OpenSpec CLI 從此讀不到這個專案；OpenSpec 自己安裝的技能與指令檔不會被移動或刪除，要自己刪。

> [!CAUTION]
> 匯入是單向的。匯入後，OpenSpec CLI 讀不到這個專案。

已經有 `.speclink.yaml` 的資料夾執行 `speclink init`（不帶 `--force`），照舊回報 Already initialized、以非零 exit code 結束，任何檔案都不改。

## 技能檔裡的路徑

`speclink init`、`speclink update` 與其他同步技能檔的入口，都依專案實際的規格資料夾產生技能檔裡的路徑：

| 專案 | 技能檔裡的規格路徑 |
| --- | --- |
| 規格資料夾是 `speclink/` | `speclink/`，例如 speclink-commit 技能的範例路徑以 `speclink/changes/<name>/` 開頭 |
| 因有其他 worktree 而暫緩遷移，仍是 `openspec/` | `openspec/` |
| `.speclink.yaml` 寫了 `spec_dir: docs/specs` | `docs/specs/` |

技能檔裡的變更資料檔一律寫成 `.meta.yaml`。

## 切換 worktree 政策不會搬資料夾

切換 worktree 政策之後的技能同步，只同步技能檔，不搬規格資料夾、不改名資料檔。這包含 CLI 的 `speclink workflow-config set worktree true`（或 `false`），以及桌面 app 產出流程頁的 worktree 開關存檔。

例：已 commit、沒有其他 worktree、沒有 `spec_dir` 的舊佈局專案執行 `speclink workflow-config set worktree true`，exit code 是 0，`openspec/` 原封不動、沒有建立 `speclink/`，兩顆 worktree 技能照常產生，裡面的規格路徑是 `openspec/`，stdout 沒有遷移相關的提示。

搬資料夾只有兩個入口：`speclink update`，以及桌面 app 技能檔提示的更新鈕。兩者都會說明改名內容並提醒你 commit。桌面那一條見[自動更新、CLI、技能檔過期與關於頁](desktop-update.md)。

## spec_dir 設定

`.speclink.yaml` 的 `spec_dir` 預設是 `speclink`。遷移與匯入都不會寫入這個欄位。你明寫了 `spec_dir` 時，Speclink 不會自動遷移。

例：`.speclink.yaml` 寫了 `spec_dir: openspec`，專案只有 `openspec/`。`speclink list` 讀 `openspec/`、不印警告；`speclink update` 不移動任何檔案，也沒有遷移提示。

## 相關頁面

- 資料夾裡放什麼：[認識資料：變更、討論與規格](data-layout.md)。
- 初始化與更新技能檔：[建立工作區與指令檔](init-workspace.md)。
- 桌面 app 啟用 OpenSpec 專案：[專案、新增專案與設定頁](desktop-projects.md)。
- 把本地專案搬到 server（`speclink/` 與 `openspec/` 都可以）：[本地工作區遷移到 remote](migrate.md)。

**出處**：`spec-dir-migration`、`user-documentation`
