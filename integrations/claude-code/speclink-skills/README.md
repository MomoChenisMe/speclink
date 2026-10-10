# speclink-skills：Speclink 的 Claude Code mod

[English](README.en.md)

在 Claude Code 裡加上四樣東西：

- **技能列**：輸入框上方一排 speclink 技能按鈕，依工作流程分成「規劃／實作／品質／收尾／其他」五頁。點一下就把 `/speclink-…` 指令填進輸入框，已經打的字留在後面當參數。「收尾」頁的 `archive+commit` 一次填入 `/speclink-archive + /speclink-commit`。滑鼠移到技能上，技能列右邊會顯示它的說明（技能本身的 description）。技能列分三列：「speclink」（右邊是目前在處理的步驟與 change）、分頁與面板鈕、技能。終端機太窄、分頁和面板鈕放不下一列時（中文標籤不到 55 欄、英文不到 60 欄），面板鈕移到第一列最右邊，分頁不會從中間拆開；這時右邊沒有空間，不顯示技能說明。
- **側邊面板**：點技能列的「面板」或輸入 `/speclink-panel` 開關。上方三個分頁：
  - **看板**：下一步，以及提案中／進行中／已就緒的變更（波次、worktree、任務進度、被誰擋住）。按變更前的 ▸ 展開，顯示提案／設計／規格／任務四份文件是否完成，以及依 `##` 分組的任務清單（已勾的畫暗，`[M]` 標成「手動」）。
  - **討論**：討論中與已結論分開列，按 ▸ 展開讀討論內文。
  - **品質**：每個變更未結的審查、驗證工單：第幾輪、各嚴重度幾條；按 ▸ 展開讀每條發現。
  - 每個區塊的標題前都有 ▾，按一下收起。點名稱會把它填進輸入框，成為最前面那個指令的參數。面板只顯示，不會勾任務或改任何東西。
- **輸入框的 `#` 選單**：在輸入框打 `#`，上方會出現下拉選單，列出提案中的變更；`#` 後面再打字可以篩選名稱（不分大小寫）。按 Tab 選第一個，或按 ↓ 選一個再按 Tab／Enter，`#` 那個字就換成變更名稱，例如打 `/speclink-apply #` 再選一個變更。沒有選就按 Enter，會照你打的字送出。句中也可以用，`#` 前面要有空白。
- **目前在處理什麼**：技能列的「speclink」右邊標出這個 session 正在跑的步驟與 change，例如 `▸ apply · add-auth`，步驟用它所在分頁的顏色。session 標題也換成 `apply · add-auth`，Warp 等會顯示終端機標題的分頁列，從分頁名稱就分得出每個 session 在做哪個 change。步驟取自你送出的 `/speclink-…` 指令，以及模型自己呼叫的 speclink 技能（例如 quality 裡接著跑的 review）；change 取自指令的參數，以及技能執行 speclink CLI 時帶的 change 名稱，所以沒打名稱也認得出來。在某個 change 的 worktree 裡開的 session，一開始就標出那個 change。封存完成（change 從清單消失）後標上 `✓`，例如 `archive · add-auth ✓`。送出 `/speclink-…` 指令後，模型還沒開始跑任何工具就按 Esc 中斷，會回到送出前的步驟與 change；已經開始跑工具才中斷，就留在新的步驟。`compact` 不影響；`/clear` 會清掉，標題在你下一次送出訊息時改回 `Claude Code`；`/resume` 接回的對話，從它的標題或紀錄找回步驟與 change，被 compact 過的也找得回來。還沒用過 speclink 的 session 不顯示，也不改標題。

## 需求

- Claude Code 2.1.290 以上（以這一版測試）。mod 的 API 目前是 early access，Claude Code 改版時，這個 mod 可能要跟著更新。
- `speclink` CLI 在 PATH 上。`#` 選單讀的是 `speclink plan` 的 `--json` 輸出（5 秒內重用同一份）。面板讀的是 `speclink list`、`speclink plan`、`speclink discuss list` 的 `--json` 輸出；展開變更時讀 `show`、`status`，展開討論時讀 `discuss show`，切到品質分頁時讀每個變更的 `review show`、`verify show`。worktree 裡的變更在它的 worktree 資料夾讀。
- 專案執行過 `speclink init`。技能列只顯示專案裡實際裝好的 speclink 技能；沒有技能的專案不顯示技能列。
- 用滑鼠點按鈕需要 Claude Code 的全螢幕版面（settings.json 的 `"tui": "fullscreen"`）。

## 安裝

在 Claude Code 的輸入框輸入：

```
/plugin install speclink-skills --marketplace MomoChenisMe/speclink
```

第一次會問要不要加入 `github:MomoChenisMe/speclink` 這個 marketplace，回答 `y`；範圍選 user，每個專案都會載入。

更新：執行 `claude plugin update speclink-skills`，再重開 Claude Code。

## 設定

`/config` 裡的 **speclink-skills → Language**：

| 值 | 效果 |
| --- | --- |
| `auto`（預設） | 跟著 Claude Code 的 `language` 設定：寫的是中文就用繁體中文，其餘用英文 |
| `en` | 英文 |
| `zh-TW` | 繁體中文 |

## 已知限制

- **面板不顯示 review、verify 的章**（品質分頁只列未結的工單）：CLI 的輸出沒有蓋章狀態。`list --json` 刻意不放這些欄位；蓋章時工單會被刪掉，所以蓋過章的 `review show` 也會失敗。蓋章紀錄只在變更的 `.openspec.yaml` 裡，要看章請用桌面 app。
- **`archive+commit`**：Claude Code 只把第一個斜線指令當成指令。實際執行的是 `/speclink-archive`，後面的 `+ /speclink-commit …` 成為它的參數，由模型在封存後接著提交。
- **session 標題要等下一次送出才更新**：Claude Code 只在送出訊息與開 session 時讓 mod 改標題，所以技能中途才認出（或換了）的 change、封存的 `✓`、`/clear` 後改回 `Claude Code`、按 Esc 後回到原本的步驟，技能列會立刻更新，標題要等你下一次送出訊息。設定了焦點之後，你用 `/rename` 取的名字也會在下一次送出時被換掉；`/clear` 時只改回這個 mod 設的標題，你自己取的不動。
- **Windows**：面板經 `cmd.exe /d /c speclink …` 執行 CLI。npm 裝的 `speclink` 是 `.cmd` 殼，而 mod 執行指令時不經過 shell，直接執行會失敗。這條路徑還沒有在實機 Windows 上驗證。

## 開發

```bash
claude plugin validate integrations/claude-code/speclink-skills
claude plugin test integrations/claude-code/speclink-skills
```

本機開發時，把這個 repo 加成本地 marketplace 再安裝。Claude Code 會直接讀工作目錄，改完執行 `/reload-plugins` 就生效：

```bash
claude plugin marketplace add <speclink repo 的路徑>
claude plugin install speclink-skills@speclink
```

Claude Code 載入 mod 後，會在 `.claude-plugin/types/` 產生型別檔（已列入 `.gitignore`）。之後在 repo 根目錄執行 `npx tsc -p integrations/claude-code/speclink-skills` 就能做型別檢查。

技能分頁表在 `hooks/groups.ts`。`scripts/claude-code/skill-groups.test.mjs` 會拿它對照 repo 裡的 `.claude/skills/speclink-*`：speclink 新增或移除技能時，這個測試會提醒你同步分頁表。
