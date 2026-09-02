---
title: 平行實作與合回：worktree
section: SDD 工作流
order: 190
keywords: [worktree, 平行實作, apply-with-worktree, worktree-merge, 合併]
sources: [worktree-apply-skill, worktree-merge-skill, worktree-overlay, workflow-config]
generated: 2026-09-02
---
# 平行實作與合回：worktree

當你同時要實作兩個以上互相獨立的變更，可以讓每個變更待在自己的 git worktree 裡。主資料夾全程不被動到。這條流程由兩個技能組成：`/speclink-apply-with-worktree` 在 worktree 裡實作，`/speclink-worktree-merge` 把結果合回主分支。

## 先開啟 worktree 政策

worktree 流程預設關閉。開啟方式：

```
speclink workflow-config set worktree true
```

開啟後，各工具的技能目錄會多出兩顆 worktree 技能。設回 false 時，兩顆技能會被移除。

> [!WARNING]
> 還有 worktree 掛著時，不能關閉政策。指令會拒絕寫入，逐列列出每個活躍 worktree 的變更名、分支與路徑，並提示你先收尾。

政策的其他細節見 [工作流政策與設定](policy-config.md)。

## 在 worktree 裡實作：/speclink-apply-with-worktree

技能先做幾項前置檢查，再進入與一般實作完全相同的流程。

1. **一次只收一個變更。** 你給了兩個以上的變更名時，技能停下來請你擇一，並印出「其餘變更各開一個新 session 執行本技能」的配方。它不會默默依序做完。
2. **檢查政策。** 有效的 worktree 政策不是 true 時，技能拒絕執行，說明「本專案未啟用 worktree 流程」，並告訴你用 `workflow-config set worktree true` 開啟。它不會改在主資料夾執行實作。
3. **確認變更存在且未封存。**
4. **確認變更的產物已提交進 HEAD。** worktree 由 HEAD 建出來。產物不在 HEAD，worktree 裡就沒有這個變更。未提交時，技能只提交該變更目錄本身，不夾帶其他髒檔。
5. **檢查進度與程式碼有沒有分家。** 技能讀變更目錄裡的證據檔（.evidence.json）記錄的觸及檔案，對主資料夾查 git 狀態。任一檔案在主資料夾是髒的，技能停下列出髒檔，依推薦順序給你三個選項：「先執行 speclink-commit 將本 change 的程式碼提交進 HEAD 再回來」、「照樣繼續（明知 worktree 缺這些實作）」、「停止」。沒有證據檔或清單為空時，靜默續行。
6. **建立 worktree。** 分支名為 `speclink/<變更名>`，位置在主資料夾旁邊的 `<repo 資料夾名>.worktrees/<變更名>/`。分支或 worktree 已存在時，沿用既有的續作，不重複建立。
7. **印出建置成本提示。** worktree 是完整的原始碼副本，你要自行安裝依賴與建置產物。
8. **在 worktree 資料夾內執行實作流程。** 之後的步驟與 [實作：完成任務](apply.md) 相同。

### 實作完成後

技能在 worktree 內完成這個變更的提交，沿用 [提交單一變更的檔案](commit.md) 的歸屬慣例。它不會合併回主分支，也不會移除 worktree。

接著技能建議你先在 worktree 內跑品質關卡：`/speclink-review`（工藝品質）、`/speclink-verify`（規格符合度），或 `/speclink-quality`（兩站合跑）。要不要跑由你判斷。蓋章會寫入變更的中介資料，技能會提示你補提交。最後交棒給 `/speclink-worktree-merge`。

## 合回主分支：/speclink-worktree-merge

合併是你明確觸發的一步。技能依序做：

1. **前置檢查。** 主資料夾當前分支不能是 `speclink/*`，也不能是 detached。技能先向你宣告合併目標分支，不代你切換分支。主資料夾的工作樹必須乾淨，該變更的 worktree 分支必須全數提交。任一條件不成立，技能停下說明缺什麼。它不代你 stash，也不代你提交主資料夾的變更。
2. **先 rebase，再 fast-forward。** 技能先在 worktree 內把 `speclink/<變更名>` rebase 到合併目標分支。成功後，在主資料夾以 fast-forward 限定方式合併，不產生合併節點。
3. **rebase 衝突時退回一般 merge。** 技能中止 rebase，讓分支完整復原，改在主資料夾做一般 merge。fast-forward 被拒時走同一條出口，並告訴你本次會留下合併節點。fast-forward 被拒的常見原因：另一個 worktree 先合回，目標分支前進了。
4. **一般 merge 衝突時立即停止。** 技能回報衝突檔案清單，中止合併。它不代你編輯衝突內容，也不留下未完成的合併狀態。
5. **合併成功後清理。** 技能移除該 worktree 並刪除分支。
6. **確認收尾。** 成功輸出標示本次以 fast-forward 或合併節點落地，並提示下一步：回主 checkout [封存](archive.md)。

> [!NOTE]
> 品質關卡建議在 worktree 內完成。你沒在 worktree 內跑，仍可在主 checkout 補跑。但主 checkout 沒有開工時記錄的比對基準，屬於降級路徑。

## 在主資料夾看 worktree 的進度

政策開啟時，在主 checkout 執行 `speclink list`，有 worktree 的變更會在該行行尾多出「 [worktree]」字樣。這一行的任務計數、狀態與開工資訊都來自 worktree 副本，不是主資料夾的副本。

映射要三個條件同時成立：

- worktree 的分支名是 `speclink/<變更名>`；
- 主工作區有同名且未封存的變更；
- worktree 底下該變更的目錄讀得到。

任一條件不成立，`speclink list` 靜默略過這個 worktree，回讀主資料夾的副本。git 不可用時，清單照常輸出、沒有 worktree 標示，不會失敗。政策文件壞掉時也一樣：清單照常輸出，只是不做 worktree 探索。政策關閉、在 worktree 內執行、或 remote 工作區，都不套用映射。

worktree 移除後再跑 `speclink list`，標示消失、計數回讀主資料夾。

### 桌面 app 的呈現

桌面看板的變更卡片會帶 worktree 標示。打開變更的詳情面板可見分支名與 worktree 路徑。worktree 內勾任務，主看板的計數自動更新，不用手動重整。詳情面板各分頁的內容、狀態報告、分析報告與看板全文搜尋，都以 worktree 副本為準。

worktree 掛著時，桌面上對這個變更的動詞分兩級：

- **拒絕執行**：「封存」「退回提案中」「刪除」。訊息會提示先執行 worktree-merge 收尾。
- **寫進 worktree 副本**：任務勾選、全部勾選、任務拖排、卡片拖排、放棄審查工單。主 checkout 的對應檔案不會被寫入。

這層防護只在桌面 app，CLI 不在此限。看板操作的細節見 [看板與任務](desktop-board.md)。

> [!CAUTION]
> 封存在主 checkout 執行。worktree-merge 收尾完成後，再回主 checkout 走 [封存](archive.md)。

**出處**：`worktree-apply-skill`、`worktree-merge-skill`、`worktree-overlay`、`workflow-config`
