---
title: 建立工作區與指令檔
section: 開始使用
order: 30
keywords: [init, update, 工作區, 技能檔, 指令檔, 過期, 降級]
sources: [workspace-tools, skill-routing]
generated: 2026-09-02
---

# 建立工作區與指令檔

一個專案要用 Speclink，先在專案根目錄執行 `speclink init`。這個指令會建立 openspec 資料夾、寫下設定檔，並替你選的 AI 工具產生技能檔。之後引擎版本更新時，用 `speclink update` 讓技能檔跟上。

## 用 speclink init 初始化

初始化前，你一定要選至少一個內建的 AI 工具。內建工具只有兩個：Claude 與 Codex。

```
speclink init --tools claude
speclink init --tools codex
speclink init --tools claude,codex
```

不帶 `--tools` 時的行為依你的終端而定：

- 互動終端：CLI 會問你要不要 Claude、要不要 Codex。你可以選一個或兩個。兩個都回答「否」時，init 不會開始，並再次要求你至少選一個。
- 非互動（stdin 是管線或轉向）：init 直接失敗，不寫任何檔案。錯誤訊息會提到 `--tools` 與三種有效寫法。

`--tools` 給空值、或給了不認識的名稱（例如 vscode）時，init 失敗，不寫任何檔案。

成功時，stdout 印出 Initialized 與 Generated files 摘要。產生的東西有：

- `.speclink.yaml`：專案根的設定檔，記錄你選的工具。
- `openspec/`：規格、變更與討論的資料夾。細節見[認識資料：變更、討論與規格](data-layout.md)。
- `.gitignore` 的 `.speclink/` 條目：`.speclink/` 是工作資料夾，不進版控。
- 技能檔：Claude 放在 `.claude/skills/`，Codex 放在 `.agents/skills/`。每個技能一個資料夾，名稱以 `speclink-` 開頭。

init 不會產生 `CLAUDE.md` 或 `AGENTS.md`。它也不會碰 AI 工具的使用者設定檔，例如 `.claude/settings.json`。

Remote 模式的 init 同樣要選工具。產物是帶 remote 區段的 `.speclink.yaml` 與技能檔，不會建立 `openspec/`。見 [CLI 連接 remote](cli-remote.md)。

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
- 清掉已經從清單移除的工具所產生的技能檔。
- 清掉技能資料夾下名稱以 `speclink-` 開頭、但這次不該產生的資料夾。例如舊版的 speclink-onboard 會被清掉，換成 speclink-baseline。你自己建的技能資料夾（名稱不以 `speclink-` 開頭）不會被動到。
- 補齊缺少的技能檔。

`.speclink.yaml` 無法解析時，update 以單行解析錯誤失敗，任何檔案都不會改。

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

多個工具狀態不同時，整體以「較新 > 缺失 > 過期 > 現版」的順序回報。探測會一併列出「更新將新建或改寫」的受管檔清單。只有換行形式不同的檔案不列入。桌面 app 用這個探測提示你更新，見[自動更新、安裝 CLI 與指令檔過期](desktop-update.md)。

## 降級守門：工作區比引擎新時拒絕更新

工作區的技能檔版號新於引擎現版時，`speclink update` 會拒絕執行：印出一行英文說明（含兩個版號）、以錯誤結束、不寫任何檔案。這是為了避免用舊版引擎把新版技能檔覆蓋掉。

要強制用引擎現版覆蓋，唯一的方法是：

```
speclink update --allow-downgrade
```

`speclink init --force` 不能越過這道守門。`speclink workflow-config set` 寫入政策後的技能檔同步，也受同一道守門：設定值會寫入，但技能檔同步被拒絕。

## 自訂 AI 工具

除了 Claude 與 Codex，你可以在 `.speclink.yaml` 的 tools 清單裡加自訂描述子，讓其他 AI 工具也拿到技能檔。描述子的欄位：

- name（必填）：kebab-case，2 到 50 字，不能與 claude、codex 同名。
- skills_dir（必填）：專案根相對路徑，不能逸出專案根。
- invocation（選填）：cli 或 tool-call，預設 cli。決定技能檔裡怎麼稱呼 speclink 動詞。
- instructions_file（選填）：已棄用，不再產生任何東西。仍留著這個欄位時，update 印一行棄用提示，不影響結果。

自訂工具的技能檔用中性寫法：不含 `/speclink-` 前綴，也不提 plan mode。描述子驗證失敗時，指令以單行錯誤結束並指出錯誤欄位。

## 已有 openspec 但沒有 .speclink.yaml 的專案

引擎提供「工作區補齊」入口：對已有 `openspec/` 但沒有 `.speclink.yaml` 的資料夾，補齊 openspec 骨架缺件（specs 與 changes/archive 目錄；config.yaml 只在不存在時寫範本）、寫 `.speclink.yaml`、產生技能檔、確保 `.gitignore` 涵蓋 `.speclink/`。既有的規格、變更、討論與 config.yaml 完全不動。重複執行得到相同結果。規格未載對應的 CLI 指令名。

下一步：[工作流總覽：站別與交棒](workflow-overview.md)。

**出處**：`workspace-tools`、`skill-routing`
