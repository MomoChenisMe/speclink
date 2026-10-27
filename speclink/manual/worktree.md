---
title: 平行實作與合回：worktree
section: SDD 工作流
order: 190
keywords: [worktree, 平行實作, apply-with-worktree, worktree-merge, 合併]
sources: [worktree-apply-skill, worktree-merge-skill, worktree-overlay, workflow-config]
generated: 2026-10-27T11:03:57+08:00
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

1. **一次只收一個變更。** 你給了兩個以上的變更名時，技能先執行 `speclink plan --json`，把沒被擋住的變更列成並行配方（其餘變更各開一個新 session 執行本技能），被擋住的逐一列出前置並說明要等前置落地再開，然後停下來請你從並行名單擇一。恰好只有一個沒被擋住時，技能改問你要不要就在這裡做它，不印配方。一個都沒有時，技能列出要等的與不可用的名字，然後停止。它不會默默依序做完。
2. **檢查政策。** 有效的 worktree 政策不是 true 時，技能拒絕執行，說明「本專案未啟用 worktree 流程」，並告訴你用 `workflow-config set worktree true` 開啟。它不會改在主資料夾執行實作。
3. **以 plan 選變更並守門。** 技能在主 checkout 執行 `speclink plan --json`。沒指名時取「下一個可開工」；沒有可開工的變更時列出每個變更的阻擋清單並停止。指名時讀該變更的阻擋清單，非空就印出前置並停止。指名的變更不在 plan 裡（已封存、不存在、或因 metadata 壞掉被略過）時，停止並列出可用的變更名。plan 失敗（依賴成環、專案未初始化）時顯示錯誤並停止。任何一種停止都不會提交產物、也不會建立 worktree。順序與阻擋怎麼算，見[執行順序：plan 與依賴](plan.md)。
4. **確認變更的產物已提交進 HEAD。** worktree 由 HEAD 建出來。產物不在 HEAD，worktree 裡就沒有這個變更。未提交時，技能只提交該變更目錄本身，不夾帶其他髒檔。
5. **檢查進度與程式碼有沒有分家。** 技能讀變更目錄裡的證據檔（.evidence.json）記錄的觸及檔案，對主資料夾查 git 狀態。任一檔案在主資料夾是髒的，技能停下列出髒檔，依推薦順序給你三個選項：「先執行 speclink-commit 將本 change 的程式碼提交進 HEAD 再回來」、「照樣繼續（明知 worktree 缺這些實作）」、「停止」。沒有證據檔或清單為空時，靜默續行。
6. **建立 worktree。** 分支名為 `speclink/<變更名>`，位置在主資料夾旁邊的 `<repo 資料夾名>.worktrees/<變更名>/`。分支或 worktree 已存在時，沿用既有的續作，不重複建立。
7. **印出建置成本提示。** worktree 是完整的原始碼副本，你要自行安裝依賴與建置產物。
8. **在 worktree 資料夾內執行實作流程。** 之後的步驟與 [實作：完成任務](apply.md) 相同。實作流程第一步在 worktree 內再跑的 plan 只是複查：worktree 裡的 plan 讀不到其他 worktree 的進度，也讀不到主線後來的封存。複查結果與第 3 步不同時，以主 checkout 的判定為準，第二步（開審查工單、蓋開工章）照常執行。

多個變更名時，技能怎麼分組：

| 你給的變更名 | plan 的阻擋清單 | 技能的處置 |
| --- | --- | --- |
| add-a、add-b、add-c | add-a 與 add-b 沒被擋；add-c 等 add-a | 並行配方列 add-a、add-b；add-c 說明「等 add-a 落地再開」；請你從 add-a、add-b 擇一 |
| add-a、add-c | add-a 沒被擋；add-c 等 add-a | add-c 說明「等 add-a 落地再開」；不印配方，問你要不要就在這裡做 add-a |
| add-b、add-c | 兩個都等 add-a | 各說明「等 add-a 落地再開」；沒有可選的變更，停止；不提交產物、不建立 worktree |
| add-c | add-c 等 add-a | 印出前置 add-a 並停止；不提交產物、不建立 worktree |
| （沒指名） | 下一個可開工是 add-a | 選 add-a，宣告「Using change: add-a」後續行第 4 步 |
| （沒指名） | 沒有可開工的變更 | 列出每個變更的阻擋清單並停止 |

### 實作完成後

技能在 worktree 內完成這個變更的提交，沿用 [提交單一變更的檔案](commit.md) 的歸屬慣例。它不會合併回主分支，也不會移除 worktree。

提交之後、交棒之前，技能會把分支 rebase 到主分支的最新位置，並重設品質關卡的起點。起點是品質關卡用來比對「這個變更動了哪些東西」的那個 commit。

1. 只有審查與驗證兩站都還沒有工單時才做（技能用 `speclink review show` 與 `speclink verify show` 判斷）。任一站已經有工單，整步略過：品質關卡跑到一半時不 rebase。
2. 合併目標是主 checkout 目前所在的分支。那個分支是 `speclink/*` 或 detached 時，技能略過這一步並說明原因。
3. 技能記下合併目標當下的 commit，在 worktree 內 rebase 上去。遇到衝突時，技能執行 `git rebase --abort` 中止 rebase、回報衝突檔案，並略過重設起點。分支沒有變，原本的起點仍然有效。技能不代你解衝突。
4. rebase 成功後，技能在 worktree 內執行 `speclink review prepare <變更名> --base <記下的 commit>`，把起點改成剛才記下的 commit。

接著技能建議下一步，三個品質關卡入口都會列出：`/speclink-review`（工藝品質）、`/speclink-verify`（規格符合度），或 `/speclink-quality`（兩站合跑）。要不要跑由你判斷。依變更的狀態，建議分兩條路：

| 變更的狀態 | 建議的路 |
| --- | --- |
| 沒有未勾的手動任務 | 在 worktree 內跑品質關卡。蓋章會寫入變更的中介資料，技能會提示你補提交，再交給 `/speclink-worktree-merge` |
| 還有未勾的手動任務，或你想在蓋章前看到 CI 結果 | 先用 `/speclink-worktree-merge` 合回，再到主 checkout 跑品質關卡、做完手動任務後封存。合併時，品質關卡的起點會承接到主 checkout |

想看 CI 結果要走第二條路，是因為 worktree 分支只在你的電腦上、從不 push，CI 看不到它。

## 合回主分支：/speclink-worktree-merge

合併是你明確觸發的一步。技能依序做：

1. **前置檢查。** 主資料夾當前分支不能是 `speclink/*`，也不能是 detached。技能先向你宣告合併目標分支，不代你切換分支。主資料夾的工作樹必須乾淨，該變更的 worktree 分支必須全數提交。任一條件不成立，技能停下說明缺什麼。它不代你 stash，也不代你提交主資料夾的變更。
2. **先 rebase，再 fast-forward。** 合併開始前，技能先記下主資料夾目前的 HEAD commit（合併前 HEAD）。接著在 worktree 內把 `speclink/<變更名>` rebase 到合併目標分支。成功後，在主資料夾以 fast-forward 限定方式合併，不產生合併節點。
3. **rebase 衝突時退回一般 merge。** 技能中止 rebase，讓分支完整復原，改在主資料夾做一般 merge。fast-forward 被拒時走同一條出口，並告訴你本次會留下合併節點。fast-forward 被拒的常見原因：另一個 worktree 先合回，目標分支前進了。
4. **一般 merge 衝突時立即停止。** 技能回報衝突檔案清單，中止合併。它不代你編輯衝突內容，也不留下未完成的合併狀態。
5. **把品質關卡的起點承接到主資料夾。** 合併成功後（fast-forward 或留下合併節點都一樣）、移除 worktree 之前，技能在主資料夾執行 `speclink review prepare <變更名> --base <合併前 HEAD> --from <worktree 路徑>`。這會把品質關卡的起點寫到主資料夾，並把 worktree 裡審查與驗證兩站的快照搬過來。指令失敗時，技能回報並停止，不移除 worktree。
6. **清理。** 技能移除該 worktree 並刪除分支。
7. **確認收尾。** 成功輸出標示本次以 fast-forward 或合併節點落地，並依變更的狀態提示下一步：

| 變更的狀態 | 下一步 |
| --- | --- |
| 品質關卡已在 worktree 內跑完 | 回主 checkout [封存](archive.md) |
| 品質關卡還沒跑 | 直接在主 checkout 跑品質關卡，再封存 |
| 還有未勾的手動任務 | 在主 checkout 跑完品質關卡、做完手動任務後再封存 |

> [!NOTE]
> 合併後才在主 checkout 跑品質關卡，不是降級路徑：第 5 步已經把品質關卡的起點承接到主資料夾。

## 在主資料夾看 worktree 的進度

政策開啟時，在主 checkout 執行 `speclink list`，有 worktree 的變更會在該行行尾多出「 [worktree]」字樣。這一行的任務計數、狀態與開工資訊都來自 worktree 副本，不是主資料夾的副本。

映射要三個條件同時成立：

- worktree 的分支名是 `speclink/<變更名>`；
- 主工作區有同名且未封存的變更；
- worktree 底下該變更的目錄讀得到。

任一條件不成立，`speclink list` 靜默略過這個 worktree，回讀主資料夾的副本。git 不可用時，清單照常輸出、沒有 worktree 標示，不會失敗。政策文件壞掉時也一樣：清單照常輸出，只是不做 worktree 探索。政策關閉、在 worktree 內執行、或 remote 工作區，都不套用映射。

worktree 移除後再跑 `speclink list`，標示消失、計數回讀主資料夾。

### 桌面 app 的呈現

桌面看板的變更卡片會帶 worktree 標示。打開變更的詳情頁可見分支名與 worktree 路徑。worktree 內勾任務，主看板的計數自動更新，不用手動重整。詳情頁各分頁的內容、狀態報告、分析報告與看板全文搜尋，都以 worktree 副本為準。

worktree 掛著時，桌面上對這個變更的動詞分兩級：

- **拒絕執行**：「封存」「退回提案中」「刪除」。訊息會提示先執行 worktree-merge 收尾。
- **寫進 worktree 副本**：任務勾選、全部勾選、任務拖排、卡片拖排、放棄審查工單。主 checkout 的對應檔案不會被寫入。

這層防護只在桌面 app，CLI 不在此限。看板操作的細節見 [看板與任務](desktop-board.md)。

> [!CAUTION]
> 封存在主 checkout 執行。worktree-merge 收尾完成後，再回主 checkout 走 [封存](archive.md)。

**出處**：`worktree-apply-skill`、`worktree-merge-skill`、`worktree-overlay`、`workflow-config`
