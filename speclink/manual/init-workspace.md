---
title: 建立工作區與指令檔
section: 開始使用
order: 30
keywords: [init, update, 工作區, 技能檔, 指令檔, 過期, 降級, skills_dir, init --force, Copilot, Pi, Cursor, OpenSpec, 匯入, 遠端專案, Server 提供的技能, 技能版本提醒, symlink]
sources: [workspace-tools, skill-routing, "spec-dir-migration#init 匯入 OpenSpec 專案", "spec-dir-migration#update 自動遷移舊佈局", "user-documentation#Getting Started 僅使用已驗證入口", "user-documentation#遠端專案的登入與技能來源有說明"]
generated: 2026-11-19T16:40:41+08:00
---

# 建立工作區與指令檔

一個專案要用 Speclink，先在專案根目錄執行 `speclink init`。這個指令會建立規格資料夾 `speclink/`、寫下設定檔，並替你選的 AI 工具產生技能檔。之後引擎版本更新時，用 `speclink update` 讓技能檔跟上。

> [!NOTE]
> 在 OpenSpec 專案（有 `openspec/`、沒有 `.speclink.yaml`）執行 `speclink init` 會直接匯入：`openspec/` 搬到 `speclink/`，之後 OpenSpec CLI 讀不到這個專案。舊版 Speclink 建立的 `openspec/` 專案則由 `speclink update` 搬過來。兩者的條件與提醒見[規格資料夾與舊佈局遷移](spec-folder.md)。

## 用 speclink init 初始化

初始化前，你一定要選至少一個內建的 AI 工具。內建工具有五個：Claude、Codex、GitHub Copilot（指令裡寫 `copilot`）、Pi（`pi`）與 Cursor（`cursor`）。

```
speclink init --tools claude
speclink init --tools codex
speclink init --tools copilot
speclink init --tools pi
speclink init --tools cursor
speclink init --tools claude,codex,copilot,pi,cursor
```

`--tools` 可以只給一個名稱，也可以用逗號串起任意幾個。大小寫不拘，重複的名稱只算一次：

| `--tools` 的值 | 結果 |
| --- | --- |
| `copilot` | 接受，選 Copilot |
| `Copilot` | 接受，選 Copilot |
| `claude,copilot` | 接受，選 Claude 與 Copilot |
| `copilot,copilot` | 接受，選 Copilot（重複只算一次） |
| `github-copilot` | 拒絕：不認識的工具，錯誤訊息列出 claude、codex、copilot、pi、cursor |
| `pi` | 接受，選 Pi |
| `Cursor` | 接受，選 Cursor |
| `codex,pi` | 接受，選 Codex 與 Pi |
| `cursor-agent` | 拒絕：不認識的工具（不收別名） |
| `vscode` | 拒絕：不認識的工具 |

不帶 `--tools` 時的行為依你的終端而定：

- 互動終端：CLI 依序問你要不要 Claude、Codex、Copilot、Pi、Cursor，五題各自回答，選幾個都可以。五題都回答「否」時，init 不會開始：CLI 印一行提醒你至少選一個，再從 Claude 開始重問五題。
- 非互動（stdin 是管線或轉向）：init 直接失敗，不寫任何檔案，stdout 是空的。錯誤訊息只有一行，會提到 `--tools`、五個工具名與逗號分隔的組合寫法。

`--tools` 給空值、或給了不認識的名稱時，init 失敗，不寫任何檔案。

成功時，stdout 印出 Initialized 與 Generated files 摘要。產生的東西有：

- `.speclink.yaml`：專案根的設定檔，記錄你選的工具。
- `speclink/`：規格、變更與討論的資料夾。細節見[認識資料：變更、討論與規格](data-layout.md)。
- `.gitignore` 的 `.speclink/` 條目：`.speclink/` 是工作資料夾，不進版控。
- 技能檔：Claude 放在 `.claude/skills/`，Codex 放在 `.agents/skills/`，Copilot 放在 `.github/skills/`，Pi 放在 `.pi/skills/`，Cursor 放在 `.cursor/skills/`。每個技能一個資料夾，名稱以 `speclink-` 開頭。

init 不會產生 `CLAUDE.md`、`AGENTS.md` 或 `.github/copilot-instructions.md`，也不會建立 `.cursor/rules/` 或 `.cursor/agents/`。它也不會碰 AI 工具的使用者設定檔，例如 `.claude/settings.json`。

### 五個內建工具的技能檔差在哪

| 工具 | 技能資料夾 | 技能裡互相引用的寫法 | 有沒有 analyze 技能 |
| --- | --- | --- | --- |
| Claude | `.claude/skills/` | `/speclink-apply` | 有 |
| Codex | `.agents/skills/` | `$speclink-apply` | 沒有 |
| Copilot | `.github/skills/` | `/speclink-apply` | 沒有 |
| Pi | `.pi/skills/` | `/skill:speclink-apply` | 沒有 |
| Cursor | `.cursor/skills/` | `/speclink-apply` | 沒有 |

Copilot、Pi 與 Cursor 拿到的技能組合都與 Codex 相同：Claude 有的技能，除了 analyze 以外都有（兩個 worktree 技能一樣只在政策開啟時產生）。它們的技能內容除了叫用前綴與工具名以外，與 Codex 的逐字相同。技能在建立變更與蓋章時，記下的工具名分別是 copilot、pi、cursor。

同時選 Codex 與 Pi 時，兩組技能各自產生在 `.agents/skills/` 與 `.pi/skills/`，不合併、也不互相略過。

> [!NOTE]
> Pi 要先信任專案，才會讀專案裡的技能。同時選了 Codex 時，Pi 會對同名的技能顯示警告，實際使用的仍是 Pi 版。

在 `.github/` 底下，選 Copilot 只會動到 `.github/skills/`。Speclink 不建立、不改寫、也不剝除 `.github/copilot-instructions.md`，`.github/` 底下的其他檔案（例如 CI 設定）一律不碰。`.speclink.yaml` 沒有工具清單時，引擎會依資料夾裡的工具痕跡決定產生哪些技能，但不會因為專案裡有 `.github/` 或 `.github/skills/` 就推斷你用 Copilot。

Pi 與 Cursor 也一樣只動自己的技能資料夾。選 Pi 只寫 `.pi/skills/`，`.pi/` 底下的其他檔案（例如 `.pi/settings.json`）不碰。選 Cursor 只寫 `.cursor/skills/`，不產生、不改寫 `.cursor/rules/` 底下的檔案，也不產生 `.cursor/agents/` 的子代理檔。引擎同樣不會因為專案裡有 `.pi/`、`.pi/skills/`、`.cursor/` 或 `.cursor/skills/` 就推斷你用 Pi 或 Cursor。

遠端專案的 init（`speclink init --store remote`）同樣要選工具，規則與上面相同。產物是帶 remote 區段的 `.speclink.yaml` 與技能檔，不會建立 `speclink/`。結尾會多一行提示：登入後執行 `speclink update`，取得 Server 提供的技能，見本頁的「遠端專案的技能由 Server 提供」一節。連線與登入見 [CLI 連接 remote](cli-remote.md)。

## 指令檔裡沒有 Speclink 的受管區塊

Speclink 不在 `CLAUDE.md`、`AGENTS.md` 或任何指令檔裡維護流程總表。工作流的入口寫在每個技能的描述裡，出口寫在每個技能的結尾。舊版引擎曾在指令檔裡注入 SPECLINK 標記區塊；新版 `speclink update` 會把這些區塊剝除，保留你自己寫的內容。剝除後全空的檔案才會被刪掉。

技能資料夾裡也不會有 speclink-tdd、speclink-clarify、speclink-sync。這三個是內部技能，只給其他技能取用。

## 用 speclink update 更新技能檔

```
speclink update
```

update 會：

- 依 `.speclink.yaml` 的工具清單，重新產生每個工具的技能檔。遠端專案會先向 Server 要技能，見本頁的「遠端專案的技能由 Server 提供」一節。
- 剝除指令檔裡遺留的 SPECLINK 標記區塊，並在 stdout 列出被剝除的檔案。
- 清掉已經從清單移除的工具所產生的技能檔。拿掉 Copilot、Pi 或 Cursor 時，只移除它的技能資料夾（`.github/skills/`、`.pi/skills/`、`.cursor/skills/`）底下名稱以 `speclink-` 開頭的資料夾；技能資料夾因此變空時，連同因此變空的 `.github/`、`.pi/`、`.cursor/` 一併移除，原本就是空的資料夾會保留；這三個資料夾底下的其他內容不動，例如你自己放在 `.cursor/skills/` 的技能與 `.cursor/rules/` 的檔案。被拿掉的工具，它的技能資料夾路徑上任何一層是 symlink 時（技能資料夾本身，或它的上層資料夾，例如 `.cursor` 連到 `.claude`），Speclink 不會穿過 symlink 刪任何東西，symlink 本身也保留。
- 清掉技能資料夾下名稱以 `speclink-` 開頭、但這次不該產生的資料夾。例如舊版的 speclink-onboard 會被清掉，換成 speclink-baseline。你自己建的技能資料夾（名稱不以 `speclink-` 開頭）不會被動到。
- 補齊缺少的技能檔。
- 專案還是舊佈局 `openspec/` 時，先把它搬到 `speclink/`，再同步技能檔。有其他 worktree 或未提交修改時暫緩，見[規格資料夾與舊佈局遷移](spec-folder.md)。

`.speclink.yaml` 無法解析時，update 以單行解析錯誤失敗，任何檔案都不會改。

孤兒資料夾的清理不只 update 會做。`speclink init`、`speclink init --force`、設定頁改工具或改政策後的技能同步、桌面的更新動作，每個再生入口完成技能生成後都清同一批。任一個資料夾刪不掉時，該入口以錯誤結束，已產生的檔案保留；重跑會收斂到同一個結果。

### 技能資料夾經 symlink 共用時拒絕同步

同步在寫任何檔案之前（包含改寫 `.speclink.yaml`），會先檢查各工具的技能資料夾有沒有經 symlink 指到同一個地方。下面兩種情況，同步以錯誤結束，印一行錯誤指出是哪兩個資料夾、各屬於哪個工具，而且不寫任何檔案：

- 兩個已選的工具（內建工具與自訂描述子都算），技能資料夾順著 symlink 解析後是同一個資料夾。例：選了 Claude 與 Cursor，`.cursor/skills` 是指向 `../.claude/skills` 的 symlink。
- 已選工具的技能資料夾，解析後等於一個沒選的內建工具的技能資料夾（那個資料夾路徑上沒有 symlink，同步本來會清理它）。例：只選 Cursor，`.cursor/skills` 指向 `../.claude/skills`，而 `.claude/skills/` 裡有 Claude 的技能。Speclink 不會先寫入 Cursor 版，再把它當成沒選的 Claude 清掉。

`speclink update --allow-downgrade` 也不能越過這項檢查。symlink 指到別的位置時不受影響，照常同步：例如 `.claude` 是指向專案外共用資料夾的 symlink，Claude 的技能就寫進那個共用資料夾的 `skills/`。

### init --force 等於重設工具選集

`speclink init --force --tools <清單>` 把你這次選的內建工具當成完整的期望狀態：

- 沒選的工具，它的 speclink- 技能資料夾整組移除。例如原本是 Claude、這次只選 Codex，`.claude/skills/` 下的 speclink- 資料夾全部移除；因此變空的 `.claude/skills/` 與 `.claude/` 一併移除。
- 自訂描述子的足跡記錄隨設定檔重寫歸零，描述子 skills 目錄下的 speclink- 資料夾一併移除。
- `CLAUDE.md` 與 `AGENTS.md` 裡遺留的 SPECLINK 區塊，不論有沒有選那個工具都剝除，你自己的內容保留。不存在的指令檔不會被建立。
- stdout 仍然只有 Initialized 與 Generated files 兩行，不列清理明細。

不帶 `--force` 的 init 在一個沒有 `.speclink.yaml`、沒有 `speclink/`、但殘留舊技能檔的資料夾執行時，也會把舊技能檔改寫成現版、清掉不該有的 speclink- 資料夾、補齊其餘技能檔。

### worktree 技能只在政策開啟時產生

speclink-apply-with-worktree 與 speclink-worktree-merge 這兩個技能，只在 `speclink/config.yaml` 的 worktree 設為 true 時才會產生。政策關閉或未設時不產生。從開改關後再跑 update，這兩個技能資料夾會被移除。環境變數不影響技能檔的產生。見[平行實作與合回：worktree](worktree.md)。

上面說的是本機專案。遠端專案從 Server 取得技能時，這兩個技能有沒有，由 Server 依這個專案的流程設定決定：Server 上的 worktree 是 true，執行 `speclink update` 後 checkout 出現這兩個技能資料夾；之後 Server 的設定改成 false 再執行一次，這兩個資料夾被移除。遠端專案退回本機範本時（Server 沒有提供技能或連不上），用的是沒有流程設定時的預設值，也就是不產生。

## 技能檔的版本與過期

每個技能檔的 frontmatter 都帶同一個版號。這個版號只在引擎內嵌的技能內容變動時才遞增，不隨 app 或 CLI 發版變動。`speclink --version` 會一起印出它：

```
<套件版號> (<架構>, engine <產物層版號>)
```

引擎提供唯讀的技能檔過期探測，結果是五種狀態之一：

| 狀態 | 意思 |
| --- | --- |
| 缺失 | 某個工具的技能資料夾裡沒有任何 speclink- 技能檔 |
| 過期 | 某個工具的技能版號舊於引擎現版 |
| 較新 | 某個工具的技能版號新於引擎現版，也就是工作區檔案領先引擎 |
| 現版 | 全部技能檔都是現版 |
| 無法判定 | 設定檔解析失敗，或技能檔存在但讀取錯誤 |

多個工具狀態不同時，整體以「較新 > 缺失 > 過期 > 現版」的順序回報。探測會一併列出「更新將新建或改寫」的受管檔清單。只有換行形式不同的檔案不列入。桌面 app 用這個探測提示你更新，見[自動更新、CLI、技能檔過期與關於頁](desktop-update.md)。

## 降級守門：工作區比引擎新時拒絕更新

工作區的技能檔版號新於引擎現版時，`speclink update` 會拒絕執行：印出一行英文說明（含兩個版號）、以錯誤結束、不寫任何檔案。這是為了避免用舊版引擎把新版技能檔覆蓋掉。

要強制用引擎現版覆蓋，唯一的方法是：

```
speclink update --allow-downgrade
```

`speclink init --force` 不能越過這道守門。`speclink workflow-config set` 寫入政策後的技能檔同步，也受同一道守門：設定值會寫入，但技能檔同步被拒絕。

遠端專案從 Server 取得技能時不套用這道守門，因為技能版本跟著 Server。Server 提供的版本比 checkout 既有的舊時，照常寫入，並在輸出說明技能已換成 Server 提供的版本。例：checkout 的技能檔是 v1.60.0、Server 提供 v1.58.0，不帶 `--allow-downgrade` 執行 `speclink update`，exit code 是 0，技能檔變成 v1.58.0。遠端專案退回本機範本時，守門照常有效。

## 遠端專案的技能由 Server 提供

`.speclink.yaml` 帶 remote 區段的專案是遠端專案。它的技能不一定來自你本機的 CLI：執行 `speclink update` 時，CLI 先連上 Server，看 Server 有沒有提供技能。

| 情況 | 結果 |
| --- | --- |
| Server 有提供技能 | CLI 依 `.speclink.yaml` 的工具清單向 Server 要技能全文，寫成各工具技能資料夾下的技能檔。檔案有哪些、內容是什麼，都以 Server 的回應為準；checkout 多出來的 speclink- 技能資料夾會被清掉。例如 Server 的回應沒有 improve，`speclink-improve` 資料夾就被移除。自訂工具的技能也一樣從 Server 取得 |
| Server 對某個工具沒有回任何技能 | 那個工具的技能資料夾完全不動，不寫也不刪。stdout 多一行英文列出這些工具，說明它們的技能檔維持原狀。其他工具照常安裝，exit code 是 0 |
| Server 沒有提供技能、沒有可用的憑證、連不上，或連線的第一步沒有成功 | 退回本機的範本產生技能檔，stderr 印一行英文說明原因，exit code 是 0 |
| Server 說有技能，但要技能時失敗（例如回應 500） | 以錯誤結束，stderr 說明取得技能失敗，不寫任何檔案，也不會退回本機的範本 |
| Server 的回應對不上：技能名稱不是小寫英數與連字號組成（最多 64 個字元）、同一個工具的技能名稱重複、缺少全文，或回應的工具與要求的不同 | 以錯誤結束，不寫任何檔案。技能名稱會變成資料夾名稱，所以像 `../../escape` 這種名稱一律拒絕 |

stdout 的摘要會指出技能的來源（server 或 local）與版本。從 Server 寫下來的技能檔，路徑與格式和本機產生的一樣，過期探測與清理照常處理它們。

本機專案不會向任何 Server 要技能：`speclink update` 不發出任何網路請求，行為與輸出照舊。

> [!WARNING]
> 遠端專案的技能由 Server 提供，版本跟著 Server。連到一台 Server，等於信任它提供的技能。

### 遠端專案的技能內容有什麼不同

遠端專案的規格由 Server 持有，checkout（你 clone 下來的那份程式碼資料夾）裡沒有規格檔。所以本機範本替遠端專案產生的技能，不含規格資料夾的路徑，也不含對規格檔的 git 步驟。例如 commit 技能會多一句，說明這次 commit 只含程式碼的檔案。本機專案的技能則保留這些內容，例如開啟 worktree 政策時，apply-with-worktree 技能有「變更的產物還沒 commit 時先 commit」的步驟。

用哪一種寫法，只看 `.speclink.yaml` 有沒有 remote 區段，不看磁碟上有沒有規格資料夾：

- `speclink init`（本機專案）產生的是本機專案的寫法；`speclink init --store remote` 產生的是遠端專案的寫法。
- `speclink update` 與其他同步技能檔的入口，依 `.speclink.yaml` 決定。遠端專案的 checkout 裡就算留著一個舊的 `speclink/` 資料夾，技能仍然是遠端專案的寫法。
- 過期探測用同一個判斷：遠端專案剛執行完 `speclink update`，馬上探測，結果是現版，沒有檔案被列為不同。

### CLI 的技能版本提醒

在遠端專案執行指令時，CLI 連上 Server 後會比對兩邊的技能版號。Server 的技能版號比你本機 CLI 的新時，stderr 印一行英文提醒，內容有兩邊的版號與更新 Speclink 的方式。

- 同一台 Server、同一個日曆日最多提醒一次，隔天再執行會再提醒。當天改連另一台同樣較新的 Server，各算各的。
- 提醒不改變 stdout、`--json` 的內容與 exit code，也不擋指令執行。例：本機是 v1.57.0、Server 是 v1.58.0，當天第一次執行 `speclink list --json`，stdout 照常是清單，exit code 是 0，stderr 多一行含 v1.57.0、v1.58.0 與更新方式。
- `speclink update` 自己不印這一行，也不會用掉當天的那一次，因為它的摘要已經寫出技能的來源與版本。當天先執行 `speclink update` 再執行 `speclink list`，提醒出現在後者。
- 本機的版號與 Server 相同或更新、Server 沒有提供技能，或任一邊的版號無法解讀成數字時，不提醒。
- 提醒的記錄存在使用者層級的設定目錄，不寫進專案裡的任何檔案。記錄讀寫不了時，不提醒也不報錯。
- CLI 不會自己下載或安裝更新。

## 自訂 AI 工具

除了五個內建工具，你可以在 `.speclink.yaml` 的 tools 清單裡加自訂描述子，讓其他 AI 工具也拿到技能檔。描述子的欄位：

- name（必填）：kebab-case，2 到 50 字，不能與內建工具同名：claude、codex、copilot、pi、cursor，以及 codex 的別名 agents 都不行。
- skills_dir（必填）：專案根相對路徑，不能逸出專案根。結尾的 `/` 會被削掉，之後所有地方（技能生成、足跡記錄、過期探測回報的路徑）都用削掉後的形式。
- invocation（選填）：cli 或 tool-call，預設 cli。決定技能檔裡怎麼稱呼 speclink 動詞。
- instructions_file（選填）：已棄用，不再產生任何東西。仍留著這個欄位時，update 印一行棄用提示，不影響結果。

skills_dir 還有兩條拒絕規則，比對的是把 `.` 段丟掉、`..` 回退一段之後的路徑，所以等價拼法一樣被拒：不能等同專案根本身，也不能等同內建工具的 skills 目錄（`.claude/skills`、`.agents/skills`、`.github/skills`、`.pi/skills`、`.cursor/skills`）。和內建目錄的比對不分大小寫，因為在 macOS 與 Windows 的檔案系統上，`.GitHub/skills` 就是 `.github/skills`。

| skills_dir | 結果 |
| --- | --- |
| `.xyz/skills` | 接受，原樣使用 |
| `.xyz/skills/` | 接受，削為 `.xyz/skills` |
| `../outside/skills` | 拒絕：逸出專案根 |
| `/`、`./`、`.`、`.xyz/..` | 拒絕：等同專案根 |
| `.claude/skills`、`./.claude/skills`、`.claude//skills`、`.xyz/../.claude/skills` | 拒絕：等同內建工具的 skills 目錄 |
| `.claude/skills-extra` | 接受：與內建目錄不同 |
| `.github/skills`、`./.github/skills` | 拒絕：等同內建 Copilot 的 skills 目錄 |
| `.GitHub/skills` | 拒絕：不分大小寫，等同內建 Copilot 的 skills 目錄 |
| `.github/skills-extra` | 接受：與內建目錄不同 |
| `.pi/skills`、`.xyz/../.pi/skills` | 拒絕：等同內建 Pi 的 skills 目錄 |
| `.pi/agent` | 接受：與內建目錄不同 |
| `.cursor/skills`、`.cursor//skills` | 拒絕：等同內建 Cursor 的 skills 目錄 |
| `.Cursor/skills` | 拒絕：不分大小寫，等同內建 Cursor 的 skills 目錄 |
| `.cursor/rules` | 接受：與內建目錄不同 |

> [!WARNING]
> 升級前就寫好、名叫 copilot、pi 或 cursor 的描述子，或 skills_dir 指向 `.github/skills`、`.pi/skills`、`.cursor/skills` 的描述子，升級後會被拒絕：`speclink update` 以單行錯誤結束，分別指出名稱與內建工具衝突、或 skills_dir 屬於內建工具的目錄。所有技能檔與這些目錄底下既有的檔案都不會被改動。描述子要改名或改路徑，才能通過驗證。

自訂工具的技能檔用中性寫法：不含 `/speclink-` 前綴，也不提 plan mode。技能裡提到其他技能時，寫成 `speclink-<名稱>`（技能的名稱），不會寫成長得像指令的 `speclink <名稱>`；每個技能檔開頭的說明也會寫明 `speclink-<名稱>` 是另一支技能、不是指令，不能當成指令或工具的參數送出。真正要執行的指令（例如 `speclink archive <name>`）維持指令的寫法。描述子驗證失敗時，指令以單行錯誤結束並指出錯誤欄位。通過驗證的描述子也參與技能檔過期探測，逐工具結果用描述子的 name 表示；沒通過驗證的描述子不參與探測、不讓結果變成無法判定，錯誤由 update 報出。

## 已有規格資料夾但沒有 .speclink.yaml 的專案

引擎提供「工作區補齊」入口，給已有規格資料夾、但沒有 `.speclink.yaml` 的資料夾用。它做這幾件事：

1. 規格資料夾是舊佈局的 `openspec/`（而且沒有 `speclink/`）時，先匯入：用與 `speclink init` 匯入相同的做法，把 `openspec/` 改名為 `speclink/`、每個 `.openspec.yaml` 改名為 `.meta.yaml`，檔案內容逐位元組不變。在 git repo 裡、有主目錄以外的 worktree，或 `openspec/` 有未提交修改時，回一行錯誤說明原因，什麼都不寫。
2. 補齊 `speclink/` 骨架缺件：specs 與 changes/archive 目錄；config.yaml 只在不存在時寫範本。
3. 在專案根寫 `.speclink.yaml`，記錄你選的工具，不寫規格資料夾的路徑。
4. 產生技能檔，並確保 `.gitignore` 涵蓋 `.speclink/`。

既有的規格、變更、討論、自訂產出流程與 config.yaml，內容都不會被改寫。重複執行得到相同結果。規格未載對應的 CLI 指令名；桌面 app 的「啟用」就是走這個入口，見[專案、新增專案與設定頁](desktop-projects.md)。

下一步：[工作流總覽：站別與交棒](workflow-overview.md)。

**出處**：`workspace-tools`、`skill-routing`、`spec-dir-migration`、`user-documentation`
