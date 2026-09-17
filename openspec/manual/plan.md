---
title: 執行順序：plan 與依賴
section: SDD 工作流
order: 125
keywords: [plan, 執行順序, 波次, 依賴, 前置, depends, 阻擋, 並行, 依賴成環, change depends]
sources: [change-plan]
generated: 2026-09-17T16:05:41+08:00
---

# 執行順序：plan 與依賴

手上同時有好幾個變更時，先做哪一個？哪些可以同時做？`speclink plan` 替你算出來：它把所有還沒封存的變更排成一個順序，分成一波一波（同一波可以並行），並列出每個變更現在被誰擋住。apply、archive、commit 與 worktree 技能都會先看這份結果，桌面看板與系統匣也照它排。

## 順序怎麼算

引擎先排出「基底順序」，再用依賴修正它。

**基底順序**依三個鍵：

1. 階段：已就緒（任務全勾且總數大於零）在前，進行中（有開工章或任一任務已勾）次之，提案中最後。
2. 同階段內，有看板順序鍵的排前面、依鍵值升冪；沒有的排後面、依建立時間升冪；沒有建立時間的殿後。
3. 同值以變更名的字典序決斷。

**依賴修正**：引擎沿基底順序，反覆挑出「所有作用中的宣告前置都已經排進去」的第一個變更放進配置。宣告前置只算還沒封存、狀態檔讀得到的變更；指向不存在、已封存或狀態檔壞掉的名稱，當成已經滿足。

**波次**：一個變更的波次，是它的宣告前置與「排在它前面、且 delta 動到同一個 capability」的變更之中最大的波次加 1。沒有前置、沒有重疊的就是第 1 波。delta 重疊不是有向邊：兩個重疊的變更誰先誰後由基底順序決定，只是不能同一波做。

**阻擋清單**：擋住一個變更的，是它的作用中宣告前置，加上排在它前面的重疊夥伴。清單為空就是現在可以做。

狀態檔壞掉的變更不參與配置與重疊判定，列在「略過」裡。

| 變更 | 階段 | 順序鍵 | 建立 | 宣告前置 | delta | 基底序 | 波次 | 被誰擋住 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| r1 | 已就緒 | 無 | 09-01 | 無 | tray-status-menu | 1 | 1 | 無 |
| p2 | 進行中 | n | 09-03 | 無 | desktop-app | 2 | 1 | 無 |
| n5 | 提案中 | f | 09-02 | 無 | archive-skill | 3 | 1 | 無 |
| n3 | 提案中 | 無 | 09-10 | 無 | desktop-app | 4 | 2 | p2 |
| n4 | 提案中 | 無 | 09-12 | n3 | discuss-skill | 5 | 3 | n3 |

上表的結果：第 1 波是 r1、p2、n5，第 2 波是 n3，第 3 波是 n4。「下一個可開工」是 n5：第一個提案中、而且沒被擋住的變更。

> [!NOTE]
> 已開工的變更排在沒開工的前面。所以 a 進行中、b 提案中、兩者重疊時，就算 b 比 a 早建立，也是 a 在前、b 被 a 擋住。

## 看結果：speclink plan

```
speclink plan
speclink plan --json
```

plan 只讀不寫。人眼輸出逐波印 `Wave N` 標題（同一波有兩個以上變更時附 `(parallel)`），每個變更一行寫名稱與階段，被擋住的附 `blocked by: …`，最後一行是 `next: <變更名>` 或 `next: none`。加 `--no-color` 就沒有顏色碼。`--json` 輸出同樣的內容給程式讀：各波的變更、每個變更的波次、階段、宣告前置、重疊夥伴與阻擋清單、下一個可開工，以及被略過的變更與原因。

沒有作用中變更時，波次與變更都是空的，沒有下一個可開工。

在主 checkout、worktree 政策開啟時，plan 判階段會讀各 worktree 裡的開工章與任務進度，與 `speclink list` 同一套聚合。

> [!WARNING]
> 宣告依賴成環（a 等 b、b 等 a）時，plan 以錯誤結束：stderr 印 `dependency cycle: a -> b -> a`，stdout 什麼都不印。解法是移除環上的一條前置（見下一節）。

## 宣告依賴：speclink change depends

「這個變更要等那個變更先做完」用 `speclink change depends` 宣告：

```
speclink change depends <變更名> --on <前置變更>...
speclink change depends <變更名> --on <前置變更>... --remove
```

指令把前置寫進變更狀態檔（`.openspec.yaml`）頂層的前置那一行，逗號分隔；其餘內容逐位元保留，換行形式沿用原檔。多個 `--on` 一次寫完。加已經存在的邊、或移除不存在的邊，不改檔、直接成功。移除到清單為空時整行拿掉。

成功時 stdout 一行 `✓ <變更名> depends on: …`（移除時是 `✓ <變更名> no longer depends on: …`）；`--json` 時回寫入後的完整前置清單。

以下情況指令拒絕，以錯誤結束、零寫入：

| 情況 | 結果 |
| --- | --- |
| 變更名不是作用中的變更（含名稱大小寫不同、空字串、`archive`） | 拒絕，stderr 說明沒有這個作用中變更 |
| 前置指向自己 | 拒絕 |
| 前置已封存或不存在（不帶 `--remove` 時） | 拒絕，訊息分辨兩者 |
| 加了這條邊會經過這個變更成環 | 拒絕，印出環（從這個變更起） |
| 變更的狀態檔壞掉 | 拒絕 |

帶 `--remove` 時，允許移除指向已封存或不存在名稱的殘留項，只拒絕不合法的名稱（空白、含路徑分隔符或 `..`）。別處既有的環不擋這條指令，由 plan 回報。

| 步驟 | 狀態檔的前置那一行 |
| --- | --- |
| 對沒有前置的 c 執行 `--on a` | `depends_on: a` |
| 再執行 `--on b` | `depends_on: a, b` |
| 再執行 `--on a --remove` | `depends_on: b` |
| 再執行 `--on b --remove` | 整行移除 |

在主 checkout、worktree 政策開啟時，變更已經對應到 worktree 的話，守門、成環檢查與寫入都作用在 worktree 副本，主 checkout 的副本不變。

宣告前置這件事，通常不用你手動做：propose 收尾與 ingest 收尾都會判定軟依賴並替你落檔，見[提案：建立變更與產物](propose.md)與[續作與需求變更](drift-ingest.md)。桌面 app 的排程分頁也能加減前置，見[看板與任務](desktop-board.md)。

## 拖排看板卡片時的依賴檢查

在看板同一欄拖動卡片，等於改它的順序鍵。引擎在寫回前用新的順序鍵重算同階段的基底順序：任何一個宣告前置會排到這張卡之後、或任何一個依賴它的變更會排到它之前，就拒絕，並列出這兩組名稱。只因 delta 重疊而要依序的變更不構成拒絕。這個檢查不寫任何檔案。畫面上的表現見[看板與任務](desktop-board.md)。

## 哪些技能會看 plan

| 技能 | 怎麼用 plan |
| --- | --- |
| `/speclink-apply`、`/speclink-apply-with-worktree` | 挑變更的第一步先跑 plan：沒指名就取下一個可開工；指名的變更被擋住就印出前置並停止，不開審查工單、不蓋開工章。不再用「只有一個變更就自動選」繞過。見[實作：完成任務](apply.md)與[平行實作與合回](worktree.md) |
| `/speclink-archive` | 封存前跑 plan：目標被擋住時提醒先封存前置。只是建議，不擋封存。見[封存](archive.md) |
| `/speclink-commit` 的先封存子流程 | 與 archive 相同的封存前提示與封存後提醒。見[提交單一變更的檔案](commit.md) |
| `/speclink-propose`、`/speclink-ingest` | 收尾判定軟依賴、落檔，再以 plan 呈現順序 |

## remote 模式

remote 模式下 `speclink plan` 向 server 查詢，`speclink change depends` 向 server 寫入，人眼與 `--json` 輸出的長相都與本機模式相同，不會讀寫本機的 openspec 資料夾。server 回報依賴成環時，plan 同樣以錯誤結束，stderr 是 server 轉發的 `dependency cycle: …`。change depends 被 server 拒絕（找不到變更、守門失敗）時，stderr 印 server 轉發的一行訊息，以錯誤結束。

remote 看板拖排也做同一套依賴檢查，只計涉及被拖卡的配對；違反時不送出寫回，畫面顯示與本地相同的一行錯誤。server 沒有 plan（舊 server、成環或請求失敗）時略過檢查。

**出處**：`change-plan`
