# speclink：Speclink 的 Herdr mod

[English](README.en.md)

在 [Herdr](https://herdr.dev) 裡開一個面板，看目前專案的變更與討論，按一下就把技能交給對的 agent、開工，或叫 Speclink 開詳情。全程不用手打變更名稱、不用找視窗。

## 面板有什麼

按 `⌘⇧S` 開關面板。面板是一個可以拖動的浮動視窗，開著也不搶鍵盤。專案跟著目前焦點所在的終端機走，切 space 就換專案。

- **變更**：提案中、進行中、已就緒三張卡常駐。每一列有波次、名稱、worktree 標記、任務進度。被擋住的列會變淡，並寫出在等誰。
- **討論**：討論卡與已轉出卡。已轉出但還有刀沒轉出的討論留在討論卡，因為下一步還要 propose。
- **做這個變更的 agent**：列上顯示 agent 的位置與狀態，點了切過去。比對順序是 `speclink.change` 標籤、worktree 路徑、Claude Code 外掛設的終端機標題。
- **技能鈕**：只列這個階段用得到的技能，建議的下一步加粗外框。
- **開工**：變更列上的鈕，為這個變更開一個專用的新 agent。見下面的「開工」。
- **＋ 新工作**：不帶變更名稱的技能。選 discuss 可以直接輸入主題。
- **叫 Speclink 開頁面**：點變更列或討論列開它的詳情頁，點列首的波次數字或頂端的「路線圖」開路線圖。

每張卡可以收合，超過 5 列會摺疊成「還有 N 個…」，每一列都有複製鈕。

### 各階段的技能

| 狀態 | 技能鈕（粗體是建議的下一步） |
| --- | --- |
| 提案中，沒被擋 | **apply**、analyze、ingest |
| 提案中，被擋 | apply 灰掉、analyze、ingest |
| 進行中 | **apply**、ingest、drift、verify |
| 已就緒，沒有 worktree | **quality**、review、verify、archive、commit |
| 已就緒，有 worktree | **quality**、review、verify、commit、worktree-merge |
| 討論重新定案，變更還沒更新 | 建議改成 **ingest** |
| 討論中 | **discuss** |
| 已定案，或還有刀沒轉出 | **propose --from-discussion**、discuss |

### 技能會送給誰

依序找，找到就停：

1. 正在做這個變更的 agent。
2. 目前分頁的 agent。
3. 都沒有就開新分頁，跑最常用且已安裝的 agent。

每一步都會檢查 agent 所在的資料夾。有 worktree 的變更要在 worktree 裡跑技能；archive、worktree-merge、apply-with-worktree 要在主資料夾跑。agent 在別的資料夾就換下一步。有 worktree 但裡面沒有 agent 時，apply 鈕會改送 apply-with-worktree，接續原本的 worktree。

agent 在做事、在等你回答或出錯時不送，按鈕變灰，旁邊寫出原因。送出後畫面切到那個 agent。

Claude Code 與 GitHub Copilot 送 `/speclink-<技能>`，Codex 送 `$speclink-<技能>`。analyze 只有 Claude Code 有，送給其他 agent 時不顯示。其他 agent 沒有 speclink 技能，不會當送達對象。

### 開工

開工和技能鈕不同：技能鈕優先交給現有的 agent，開工一定開新的 agent。

| 工作流設定的 `worktree` | 按下開工 |
| --- | --- |
| 開啟 | 開一個新 space，跑 `/speclink-apply-with-worktree <變更>`。space 的名稱維持 Herdr 原樣，分頁名稱會寫出變更 |
| 關閉 | 在主資料夾的 space 開新分頁，跑 `/speclink-apply <變更>` |

沒被擋、任務還沒做完、也沒有 agent 在做的變更才有開工鈕。討論重新定案的變更要先 ingest，所以也沒有。

**整波開工**：`worktree` 開啟，而且最前面那一波有兩個以上能開工的變更時，面板頂端會多一列，寫出會開哪些變更。按一次就全部開工，不再多問。波次取自 `speclink plan`。

### 叫 Speclink 開頁面

面板用 `open` 送出 `speclink://` 網址，桌面 app 會切到這個專案再開頁面。app 沒開時會自動啟動。網址只能開啟與檢視，不會寫入任何檔案。

## 需求

- Herdr 0.1.12 以上（以這一版測試）。
- `speclink` CLI 在 PATH 上，版本要有 `plan --json` 的 `unpromotedCuts`。面板只讀 `speclink list`、`speclink plan`、`speclink discuss list`、`speclink workflow-config show` 的 `--json` 輸出。
- 專案執行過 `speclink init`。
- 要叫 Speclink 開頁面時，桌面 app 要認得 `speclink://` 網址（v0.11.0 之後的版本）。

## 安裝

在 repo 根目錄執行：

```bash
cp -R integrations/herdr/mods/speclink ~/.config/herdr-app/mods/
```

Herdr 會自動載入。接著在 `~/.config/herdr-app/keys.json` 的 `global` 加上這一行（[keys.json](keys.json) 是同一行）：

```json
"cmd-shift-s": "mod.open:speclink"
```

更新時再執行一次 `cp`，Herdr 會立刻重新載入。移除時刪掉那個資料夾和那一行。

不要用符號連結安裝：Herdr 載入得到程式，但畫不出連結裡的 SVG 圖示，存檔也不會自動重新載入。

## 已知限制

- **不能停靠在視窗裡**：Herdr 內建的 layout 不讓 mod 停靠，要複製 layout 才行，而複製版不會跟著 Herdr 更新。所以面板用快捷鍵彈出。
- **不顯示 review、verify 的章**：CLI 的輸出沒有蓋章狀態。已就緒的變更蓋完章後，建議仍停在 quality，archive 要自己按。
- **「已轉出」是粗分**：看 `discuss list` 的 `status`，以及 `plan` 有沒有未轉出的刀。
- **Codex 只查證了寫法**：叫用寫法取自 speclink 的文件與技能輸出，還沒有實際送給 Codex 測過。
- **技能鈕不看專案裝了哪些技能**：例如專案沒有 Codex 的技能檔時，鈕還是會送 `$speclink-…`。
- **開頁面只有 macOS**：面板用 `open`，其他系統會跳通知說開不了。專案在別台機器的 space 不能開，所以那些列點了沒有反應，也沒有「路線圖」鈕。
- **不看 `SPECLINK_WORKTREE` 環境變數**：面板只讀工作流設定。環境變數把 worktree 關掉時，開工一樣開新 space，技能會自己停下來並說明。
- **開工 space 可能還在主資料夾**：agent 用 `cd` 進 worktree，終端機回報的資料夾不一定跟著變。這時 space 的名稱是專案名，也不會縮排在專案底下，要看分頁名稱才分得出變更。
- **第一次開的資料夾會先問信任**：Claude Code 問要不要信任資料夾時還不算 agent，面板看不到它，15 秒後開工鈕會再出現。信任過的資料夾不會問。
- **整波開工可能撞 git 鎖**：每個 agent 都可能在主資料夾 commit 規格或建 worktree。撞到時那個 agent 會停下來回報，再按一次技能鈕即可。
- **畫面只有繁體中文**。

## 開發

```bash
node --test scripts/herdr/panel-model.test.mjs
herdr-app check ~/.config/herdr-app/mods/speclink
```

`model.js` 是純邏輯：資料合併、技能表、送達決策，測試直接跑它。`main.js` 是畫面與 Herdr API。`scripts/herdr/panel-model.test.mjs` 也拿技能表對照 `.claude/skills/speclink-*`：speclink 新增或移除技能時，這個測試會提醒你同步。
