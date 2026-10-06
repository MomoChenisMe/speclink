# speclink-skills：Speclink 的 Claude Code mod

[English](README.en.md)

在 Claude Code 裡加上兩樣東西：

- **技能列**：輸入框上方一排 speclink 技能按鈕，依工作流程分成「規劃／實作／品質／收尾／其他」五頁。點一下就把 `/speclink-…` 指令填進輸入框，已經打的字留在後面當參數。「收尾」頁的 `archive+commit` 一次填入 `/speclink-archive + /speclink-commit`。
- **側邊面板**：點技能列的「面板」或輸入 `/speclink-panel` 開關。面板列出下一步、討論，以及提案中／進行中／已就緒的變更（波次、worktree、任務進度、被誰擋住）。點名稱會把它填進輸入框，成為最前面那個指令的參數。

## 需求

- Claude Code 2.1.290 以上（以這一版測試）。mod 的 API 目前是 early access，Claude Code 改版時，這個 mod 可能要跟著更新。
- `speclink` CLI 在 PATH 上。面板讀的是 `speclink list`、`speclink plan`、`speclink discuss list` 的 `--json` 輸出。
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

- **面板不顯示 review、verify 的章**：CLI 的輸出沒有蓋章狀態。`list --json` 刻意不放這些欄位；蓋章時工單會被刪掉，所以蓋過章的 `review show` 也會失敗。蓋章紀錄只在變更的 `.openspec.yaml` 裡，要看章請用桌面 app。
- **`archive+commit`**：Claude Code 只把第一個斜線指令當成指令。實際執行的是 `/speclink-archive`，後面的 `+ /speclink-commit …` 成為它的參數，由模型在封存後接著提交。
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
