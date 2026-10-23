---
title: 建立工作區與指令檔
section: 開始使用
order: 30
keywords: [init, update, 工作區, 技能檔, 指令檔, 過期, 降級, skills_dir, init --force, Copilot, Antigravity]
sources: [workspace-tools, skill-routing]
generated: 2026-10-23T14:19:42+08:00
---

# 建立工作區與指令檔

一個專案要用 Speclink，先在專案根目錄執行 `speclink init`。這個指令會建立 openspec 資料夾、寫下設定檔，並替你選的 AI 工具產生技能檔。之後引擎版本更新時，用 `speclink update` 讓技能檔跟上。

## 用 speclink init 初始化

初始化前，你一定要選至少一個內建的 AI 工具。內建工具有四個：Claude、Codex、GitHub Copilot（指令裡寫 `copilot`）與 Antigravity（指令裡寫 `antigravity`）。

```
speclink init --tools claude
speclink init --tools codex
speclink init --tools copilot
speclink init --tools antigravity
speclink init --tools claude,codex,copilot,antigravity
```

`--tools` 可以只給一個名稱，也可以用逗號串起任意幾個。大小寫不拘，重複的名稱只算一次：

| `--tools` 的值 | 結果 |
| --- | --- |
| `copilot` | 接受，選 Copilot |
| `Copilot` | 接受，選 Copilot |
| `claude,copilot` | 接受，選 Claude 與 Copilot |
| `copilot,copilot` | 接受，選 Copilot（重複只算一次） |
| `antigravity` | 接受，選 Antigravity |
| `github-copilot` | 拒絕：不認識的工具，錯誤訊息列出 claude、codex、copilot、antigravity |
| `vscode` | 拒絕：不認識的工具 |

不帶 `--tools` 時的行為依你的終端而定：

- 互動終端：CLI 依序問你要不要 Claude、Codex、Copilot、Antigravity，各自回答，選幾個都可以。四題都回答「否」時，init 不會開始：CLI 印一行提醒你至少選一個，再從 Claude 開始重問。
- 非互動（stdin 是管線或轉向）：init 直接失敗，不寫任何檔案，stdout 是空的。錯誤訊息只有一行，會提到 `--tools`、四個工具名與逗號分隔的組合寫法。

`--tools` 給空值、或給了不認識的名稱時，init 失敗，不寫任何檔案。

成功時，stdout 印出 Initialized 與 Generated files 摘要。產生的東西有：

- `.speclink.yaml`：專案根的設定檔，記錄你選的工具。
- `openspec/`：規格、變更與討論的資料夾。細節見[認識資料：變更、討論與規格](data-layout.md)。
- `.gitignore` 的 `.speclink/` 條目：`.speclink/` 是工作資料夾，不進版控。
- 技能檔：Claude 放在 `.claude/skills/`，Codex 放在 `.agents/skills/`，Copilot 放在 `.github/skills/`，Antigravity 放在 `.agent/skills/`。每個技能一個資料夾，名稱以 `speclink-` 開頭。

init 不會產生 `CLAUDE.md`、`AGENTS.md` 或 `.github/copilot-instructions.md`。它也不會碰 AI 工具的使用者設定檔，例如 `.claude/settings.json`。

### 三個內建工具的技能檔差在哪

| 工具 | 技能資料夾 | 技能裡互相引用的寫法 | 有沒有 analyze 技能 |
| --- | --- | --- | --- |
| Claude | `.claude/skills/` | `/speclink-apply` | 有 |
| Codex | `.agents/skills/` | `$speclink-apply` | 沒有 |
| Copilot | `.github/skills/` | `/speclink-apply` | 沒有 |
| Antigravity | `.agent/skills/` | `/speclink-apply` | 沒有 |

Copilot 拿到的技能組合與 Codex 相同：Claude 有的技能，除了 analyze 以外都有（兩個 worktree 技能一樣只在政策開啟時產生）。兩者的技能內容除了叫用前綴與工具名以外逐字相同。Copilot 的技能在建立變更與蓋章時，記下的工具名是 copilot。

在 `.github/` 底下，選 Copilot 只會動到 `.github/skills/`。Speclink 不建立、不改寫、也不剝除 `.github/copilot-instructions.md`，`.github/` 底下的其他檔案（例如 CI 設定）一律不碰。`.speclink.yaml` 沒有工具清單時，引擎會依資料夾裡的工具痕跡決定產生哪些技能，但不會因為專案裡有 `.github/` 或 `.github/skills/` 就推斷你用 Copilot。

Remote 模式的 init 同樣要選工具，規則與上面相同。產物是帶 remote 區段的 `.speclink.yaml` 與技能檔，不會建立 `openspec/`。見 [CLI 連接 remote](cli-remote.md)。

## 指令檔裡沒有 Speclink 的受管區塊

Speclink 不在 `CLAUDE.md`、`AGENTS.md` 或任何指令檔裡維護流程總表。工作流的入口寫在每個技能的描述裡，出口寫在每個技能的結尾。舊版引擎曾在指令檔裡注入 SPECLINK 標記區塊；新版 `speclink update` 會把這些區塊剝除，保留你自己寫的內容。剝除後全空的檔案才會被刪掉。

技能資料夾裡也不會有 speclink-tdd、speclink-clarify、speclink-sync。這三個是內部技能，只給其他技能取用。

## 用 speclink update 更新技能檔

```
speclink update
```

update 會：

- 依 `.speclink.yaml` 的工具清單，重新產生每個工具的技能檔。
- 剝除指令檔裡遺留的 SPECLINK 標記區塊，並在 stdout 列出被剝除的檔案。
- 清掉已經從清單移除的工具所產生的技能檔。拿掉 Copilot 時，只移除 `.github/skills/` 底下名稱以 `speclink-` 開頭的資料夾；`.github/skills/` 因此變空才一併移除，原本就是空的會保留；`.github/` 的其他內容不動。被拿掉的工具，如果它的技能資料夾本身是 symlink，Speclink 不會穿過 symlink 刪任何東西，symlink 本身也保留。
- 清掉技能資料夾下名稱以 `speclink-` 開頭、但這次不該產生的資料夾。例如舊版的 speclink-onboard 會被清掉，換成 speclink-baseline。你自己建的技能資料夾（名稱不以 `speclink-` 開頭）不會被動到。
- 補齊缺少的技能檔。

`.speclink.yaml` 無法解析時，update 以單行解析錯誤失敗，任何檔案都不會改。

孤兒資料夾的清理不只 update 會做。`speclink init`、`speclink init --force`、設定頁改工具或改政策後的技能同步、桌面的更新動作，每個再生入口完成技能生成後都清同一批。任一個資料夾刪不掉時，該入口以錯誤結束，已產生的檔案保留；重跑會收斂到同一個結果。

### init --force 等於重設工具選集

`speclink init --force --tools <清單>` 把你這次選的內建工具當成完整的期望狀態：

- 沒選的工具，它的 speclink- 技能資料夾整組移除。例如原本是 Claude、這次只選 Codex，`.claude/skills/` 下的 speclink- 資料夾全部移除；因此變空的 `.claude/skills/` 與 `.claude/` 一併移除。
- 自訂描述子的足跡記錄隨設定檔重寫歸零，描述子 skills 目錄下的 speclink- 資料夾一併移除。
- `CLAUDE.md` 與 `AGENTS.md` 裡遺留的 SPECLINK 區塊，不論有沒有選那個工具都剝除，你自己的內容保留。不存在的指令檔不會被建立。
- stdout 仍然只有 Initialized 與 Generated files 兩行，不列清理明細。

不帶 `--force` 的 init 在一個沒有 `.speclink.yaml`、沒有 `openspec/`、但殘留舊技能檔的資料夾執行時，也會把舊技能檔改寫成現版、清掉不該有的 speclink- 資料夾、補齊其餘技能檔。

### worktree 技能只在政策開啟時產生

speclink-apply-with-worktree 與 speclink-worktree-merge 這兩個技能，只在 `openspec/config.yaml` 的 worktree 設為 true 時才會產生。政策關閉或未設時不產生。從開改關後再跑 update，這兩個技能資料夾會被移除。環境變數不影響技能檔的產生。見[平行實作與合回：worktree](worktree.md)。

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

## 自訂 AI 工具

除了三個內建工具，你可以在 `.speclink.yaml` 的 tools 清單裡加自訂描述子，讓其他 AI 工具也拿到技能檔。描述子的欄位：

- name（必填）：kebab-case，2 到 50 字，不能與內建工具同名：claude、codex、copilot，以及 codex 的別名 agents 都不行。
- skills_dir（必填）：專案根相對路徑，不能逸出專案根。結尾的 `/` 會被削掉，之後所有地方（技能生成、足跡記錄、過期探測回報的路徑）都用削掉後的形式。
- invocation（選填）：cli 或 tool-call，預設 cli。決定技能檔裡怎麼稱呼 speclink 動詞。
- instructions_file（選填）：已棄用，不再產生任何東西。仍留著這個欄位時，update 印一行棄用提示，不影響結果。

skills_dir 還有兩條拒絕規則，比對的是把 `.` 段丟掉、`..` 回退一段之後的路徑，所以等價拼法一樣被拒：不能等同專案根本身，也不能等同內建工具的 skills 目錄（`.claude/skills`、`.agents/skills`、`.github/skills`）。和內建目錄的比對不分大小寫，因為在 macOS 與 Windows 的檔案系統上，`.GitHub/skills` 就是 `.github/skills`。

| skills_dir | 結果 |
| --- | --- |
| `.wad/skills` | 接受，原樣使用 |
| `.wad/skills/` | 接受，削為 `.wad/skills` |
| `../outside/skills` | 拒絕：逸出專案根 |
| `/`、`./`、`.`、`.wad/..` | 拒絕：等同專案根 |
| `.claude/skills`、`./.claude/skills`、`.claude//skills`、`.wad/../.claude/skills` | 拒絕：等同內建工具的 skills 目錄 |
| `.claude/skills-extra` | 接受：與內建目錄不同 |
| `.github/skills`、`./.github/skills` | 拒絕：等同內建 Copilot 的 skills 目錄 |
| `.GitHub/skills` | 拒絕：不分大小寫，等同內建 Copilot 的 skills 目錄 |
| `.github/skills-extra` | 接受：與內建目錄不同 |

> [!WARNING]
> 升級前就寫好、名叫 copilot 的描述子，或 skills_dir 指向 `.github/skills` 的描述子，升級後會被拒絕：`speclink update` 以單行錯誤結束，分別指出名稱與內建工具衝突、或 skills_dir 屬於內建 Copilot 目錄。所有技能檔與 `.github/skills/` 底下既有的檔案都不會被改動。描述子要改名或改路徑，才能通過驗證。

自訂工具的技能檔用中性寫法：不含 `/speclink-` 前綴，也不提 plan mode。描述子驗證失敗時，指令以單行錯誤結束並指出錯誤欄位。通過驗證的描述子也參與技能檔過期探測，逐工具結果用描述子的 name 表示；沒通過驗證的描述子不參與探測、不讓結果變成無法判定，錯誤由 update 報出。

## 已有 openspec 但沒有 .speclink.yaml 的專案

引擎提供「工作區補齊」入口：對已有 `openspec/` 但沒有 `.speclink.yaml` 的資料夾，補齊 openspec 骨架缺件（specs 與 changes/archive 目錄；config.yaml 只在不存在時寫範本）、寫 `.speclink.yaml`、產生技能檔、確保 `.gitignore` 涵蓋 `.speclink/`。既有的規格、變更、討論與 config.yaml 完全不動。重複執行得到相同結果。規格未載對應的 CLI 指令名。

下一步：[工作流總覽：站別與交棒](workflow-overview.md)。

**出處**：`workspace-tools`、`skill-routing`
