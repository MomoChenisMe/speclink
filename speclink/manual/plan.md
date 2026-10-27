---
title: 執行順序：plan 與依賴
section: SDD 工作流
order: 125
keywords: [plan, 執行順序, 波次, 依賴, 前置, depends, 阻擋, 並行, 依賴成環, change depends, 封存順序, 同名重疊, 衝突, change rank, 插隊, strict-overlap]
sources: [change-plan]
generated: 2026-09-25T08:39:09+08:00
---

# 執行順序：plan 與依賴

手上同時有好幾個變更時，先做哪一個？哪些可以同時做？封存時誰要等誰？`speclink plan` 替你算出來：它把所有還沒封存的變更排成一個順序，分成一波一波（同一波可以並行），列出每個變更現在被哪些前置擋住，也列出兩個變更動到同一個 requirement 時，封存要照什麼順序。apply、archive、commit 與 worktree 技能都會先看這份結果，桌面看板與系統匣也照它排。

## 順序怎麼算

引擎先排出「基底順序」，再用宣告的依賴修正它。

**基底順序**依三個鍵：

1. 階段：已就緒（任務全勾且總數大於零）在前，進行中（有開工章或任一任務已勾）次之，提案中最後。
2. 同階段內，有看板順序鍵的排前面、依鍵值升冪。沒有順序鍵的排後面，彼此依序比：被依賴的次數多的在前（有幾個作用中變更把它宣告為前置）→ 任務總數少的在前（沒有 tasks.md、或 tasks.md 裡一個任務都沒有的殿後）→ 建立時間早的在前，沒有建立時間的殿後。
3. 同值以變更名的字典序決斷。

**依賴修正**：引擎沿基底順序，反覆挑出「所有作用中的宣告前置都已經排進去」的第一個變更放進配置。宣告前置只算還沒封存、狀態檔讀得到的變更；指向不存在、已封存或狀態檔壞掉的名稱，當成已經滿足。

**波次**：一個變更的波次，是它的作用中宣告前置之中最大的波次加 1。沒有前置的就是第 1 波。

**阻擋清單**：擋住一個變更的，只有它的作用中宣告前置。清單為空就是現在可以做。

兩個變更的 delta 動到同一個 capability，不會把其中一個推到下一波，也不會進阻擋清單。它們可以同時做；真正要排先後的是封存，見下一節。

狀態檔壞掉的變更不參與配置與重疊判定，列在「略過」裡。

| 情況 | 結果 |
| --- | --- |
| 提案中的 x、y、z 都沒有順序鍵；x 被 2 個變更依賴，y 與 z 沒人依賴；y 有 12 個任務、z 有 5 個 | 基底順序 x、z、y |
| 提案中的 empty（08-01 建立，tasks.md 只有一個標題、沒有任務）與 some（09-01 建立，3 個任務），都沒有順序鍵、沒人依賴 | 基底順序 some、empty，下一個可開工是 some |

> [!NOTE]
> 已開工的變更排在沒開工的前面。所以 a 進行中、b 提案中時，就算 b 比 a 早建立，也是 a 在前。兩者修改同一個 requirement 時，兩者同為第 1 波，b 封存要等 a。

## 同名 requirement 重疊與封存順序

兩個變更在同一個 capability 底下，各自新增、修改、移除或改名了同一個 requirement 名，就叫「同名 requirement 重疊」。改名時，舊名與新名都算被它動到，輸出一律記為改名。同一個 capability 但不同的 requirement，不算重疊。

每個變更對那個名稱扮演三種角色之一：

| 角色 | 哪些操作 | 意思 |
| --- | --- | --- |
| 修改 | 修改 | 封存前後這個名稱都在 |
| 拿走 | 移除、改名的舊名 | 封存前在，封存後不在 |
| 帶進 | 新增、改名的新名 | 封存前不在，封存後在 |

角色的先後，看這個名稱此刻在不在正式規格裡：

- 在：修改 → 拿走 → 帶進。
- 不在（包括這個 capability 還沒有正式規格）：帶進 → 修改 → 拿走。

兩個變更的角色不同時，角色在後的那個要等角色在前的那個先封存，不論基底順序。兩邊都是「修改」時，照封存序（見下）決定誰先。兩邊都是「拿走」、或都是「帶進」時，是**衝突**：後封存的那一個會封存失敗（名字重複，或名字已經不在了），要先改掉其中一邊。

**封存要等誰**以兩個變更整體來判：

1. 任何一個同名 requirement 上，我要等對方，對方就列進我的「封存要等」清單。
2. 否則，任何一個同名 requirement 上，對方要等我，就不列。
3. 否則，有兩邊都是「修改」的同名 requirement，而且對方在封存序裡排在前面，才列。

**封存序**是配置順序經「等」修正後的順序：沿配置順序，反覆取出「它要等的變更都已經取出」的第一個變更；彼此互等、取不出來時，取配置最前面的那個。衝突不會讓任何一方列進對方的「封存要等」，也不改變其他同名 requirement 的先後。兩個方向都有「等」時，雙方互相列入：封存互相卡住，要調整拆分。delta 讀不到或是空白，當成什麼都沒動到。

| 情況 | 結果 |
| --- | --- |
| a 與 b 各自修改 desktop-app 的「看板與任務」，a 排在前面，都沒有宣告前置 | 兩者同為第 1 波、都沒被擋；b 封存要等 a |
| a 修改「Run rewind point」、b 修改「Conversation pin」，同一個 capability | 不算重疊 |
| a 新增「匯出」、b 修改「匯出」，b 排在前面 | b 封存要等 a |
| a 與 b 都新增「匯出」 | 衝突；兩邊的「封存要等」都是空的 |
| a 把「舊名」改名成「匯出」、b 新增「匯出」 | 衝突；如果 b 改成修改「匯出」，b 封存要等 a |
| 正式規格已有「Y」；c 移除「Y」、d 新增「Y」，d 排在前面 | d 封存要等 c |
| 正式規格已有「X」；a 移除「X」、b 修改「X」，a 排在前面 | a 封存要等 b |
| 正式規格已有「X」；a 移除「X」、b 把「X」改名成「W」 | 衝突 |
| a 與 b 都新增「R1」、都修改「R2」，a 排在前面 | 「R1」衝突；b 封存要等 a |
| a（3 個任務）修改「R」「S」；b（10 個）修改「S」「T」；c（20 個）新增「R」並修改「T」 | a 等 b 與 c、c 等 b；封存序 b、c、a，不繞成環 |

封存的先後為什麼重要：兩個變更都改了同一個 requirement，後封存的那一個手上的內文是舊的，直接封存會把前一個併進正式規格的內容蓋掉。所以前一個封存後，後一個要先重讀自己對那個 requirement 的區塊、對照正式規格重寫（走 `/speclink-ingest`），再封存。只跑 drift 抓不到這種覆蓋：drift 只查 requirement 名在不在。archive 與 commit 技能會替你提醒，見[封存](archive.md)。

下表是五個變更的完整例子：

| 變更 | 階段 | 順序鍵 | 建立 | 宣告前置 | delta 動到 | 基底序 | 波次 | 被誰擋住 | 封存要等 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| r1 | 已就緒 | 無 | 09-01 | 無 | tray-status-menu 修改「列首波次」 | 1 | 1 | 無 | 無 |
| p2 | 進行中 | n | 09-03 | 無 | desktop-app 修改「看板與任務」 | 2 | 1 | 無 | 無 |
| n5 | 提案中 | f | 09-02 | 無 | archive-skill 新增「候選清單」 | 3 | 1 | 無 | 無 |
| n3 | 提案中 | 無 | 09-10 | 無 | desktop-app 修改「看板與任務」 | 4 | 1 | 無 | p2 |
| n4 | 提案中 | 無 | 09-12 | n3 | discuss-skill 修改「輪次」 | 5 | 2 | n3 | 無 |

上表的結果：第 1 波是 r1、p2、n5、n3，第 2 波是 n4。「下一個可開工」是 n5：第一個提案中、而且沒被擋住的變更。n3 可以與 p2 同時做，但封存要等 p2。

## 看結果：speclink plan

```
speclink plan
speclink plan --json
```

plan 只讀不寫。人眼輸出逐波印 `Wave N` 標題（同一波有兩個以上變更時附 `(parallel)`），每個變更一行寫名稱與階段，行尾依序附上：

1. 被擋住時：`blocked by: …`
2. 封存要等別人時：`archive after: …`
3. 有衝突時：`conflicts with: …`

沒有的就不印。最後一行是 `next: <變更名>` 或 `next: none`。加 `--no-color` 就沒有顏色碼。例如 e 宣告了前置 a、與 b 修改同一個 requirement、與 c 新增同名 requirement，那一行長這樣：

```
• e [proposed] — blocked by: a — archive after: b — conflicts with: c
```

`--json` 輸出同樣的內容給程式讀：各波的變更；每個變更的波次、階段、宣告前置、目錄級的重疊夥伴（delta 動到同一個 capability 的變更，原樣保留）、阻擋清單、是否就緒、同名 requirement 重疊（對方、capability、requirement 名、雙方各自的操作、是否衝突），以及封存要等的變更；下一個可開工；被略過的變更與原因。

沒有作用中變更時，波次與變更都是空的，沒有下一個可開工。

在主 checkout、worktree 政策開啟時，plan 判階段會讀各 worktree 裡的開工章與任務進度，與 `speclink list` 同一套聚合。

> [!WARNING]
> 宣告依賴成環（a 等 b、b 等 a）時，plan 以錯誤結束：stderr 印 `dependency cycle: a -> b -> a`，stdout 什麼都不印。解法是移除環上的一條前置（見下面的「宣告依賴」）。

### 退回舊規則：--strict-overlap

```
speclink plan --strict-overlap
```

加這個旗標，plan 退回同名 requirement 規則之前的算法：delta 動到同一個 capability 的變更會被推到下一波（波次取已配置的重疊者的最大波次加 1），也會進阻擋清單；沒有順序鍵的變更只依建立時間排（沒有建立時間的殿後），不看被依賴次數與任務數。波次、順序、阻擋清單、目錄級重疊與下一個可開工，都與舊版逐位元一致。同名 requirement 重疊與封存要等照常算。

| 情況 | 不帶旗標 | 帶 `--strict-overlap` |
| --- | --- | --- |
| a 與 b 都動到 desktop-app，但是不同的 requirement | 同為第 1 波 | a 第 1 波、b 第 2 波，b 被 a 擋住 |
| 提案中的 a（12 個任務、09-01 建立）與 b（5 個任務、09-02 建立），都沒有順序鍵，都動到 desktop-app | b 排在 a 前、同為第 1 波 | a 第 1 波、b 第 2 波，b 被 a 擋住 |

這個旗標只能在本機模式用，見下面的「remote 模式」。

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

## 調整順序：speclink change rank

想讓某個變更排到另一個變更的前面或後面，用 `speclink change rank`：

```
speclink change rank <變更名> --before <另一個變更>
speclink change rank <變更名> --after <另一個變更>
```

指令改寫變更的看板順序鍵，plan 的配置順序與看板欄內的順序會一起變。`--before` 與 `--after` 必須二選一。

指令依序做這些檢查，任何一項不過就以錯誤結束，而且一個檔案都不寫（所有檢查都在第一筆寫入之前做完）：

| 情況 | 結果 |
| --- | --- |
| 任一方不是作用中變更、狀態檔壞掉，或兩者是同一個 | 拒絕 |
| 兩者階段不同 | 拒絕：順序鍵只在同一欄內有意義 |
| 變更已經有順序鍵，而且沒帶 `--force` | 拒絕，訊息含 `already has a board rank` 與 `--force` |
| 新位置違反宣告依賴 | 拒絕，列出必須排在它前面與後面的變更，例如 `cannot move 'c' there` 與 `it depends on a` |
| 要寫入的變更缺 `.openspec.yaml` | 拒絕，訊息含該檔路徑與 `is missing` |

已經有順序鍵、確定要覆寫時，加 `--force`：

```
speclink change rank <變更名> --after <另一個變更> --force
```

新的順序鍵取「另一個變更」與它鄰居的中間值。欄內的順序與鄰居，看的是看板上的顯示順序（plan 的配置順序投影到那一欄）。那一欄除了這個變更以外，有任何一張卡缺順序鍵、或鍵的順序與顯示順序不一致時，指令先依顯示順序替整欄補上順序鍵，再寫這個變更。補鍵、中間值與依賴檢查，與桌面看板拖排是同一套引擎實作。寫入順序是先逐一補鍵、最後寫這個變更；中途失敗就停下並回報，已經補上的鍵保留（補到一半的欄仍是合法的順序，重跑可以接著做）。每次寫入只動順序那一行，其他內容逐位元不變。

成功時 stdout 一行 `✓ <變更名> ranked before <另一個變更>`（或 `after`）；`--json` 回變更名、另一個變更與前後位置，不含順序鍵的值。

在主 checkout、worktree 政策開啟時，欄內順序與檢查看的是與 plan 相同的範圍：主 checkout 的變更名冊，對應到 worktree 的變更取它 worktree 副本的值；worktree 分出去之後才在主 checkout 新建的變更，也在同一欄裡。每一筆寫入落在該變更自己的副本：有對應 worktree 的寫 worktree 副本，沒有的寫主 checkout，與桌面拖排相同。

| 情況 | 結果 |
| --- | --- |
| 同為提案中的 a（有順序鍵 n）與 c（沒有順序鍵），欄內只有這兩個，執行 `speclink change rank c --before a` | c 多一個比 n 小的順序鍵，印 `✓ c ranked before a`，plan 的順序 c 在 a 前 |
| c 已有順序鍵，執行 `speclink change rank c --after a` | 拒絕，c 的檔案不變 |
| 同上，加 `--force` | c 的順序鍵改成排在 a 之後 |
| c 依賴 a，執行 `speclink change rank c --before a` | 拒絕，零寫入 |
| 提案中欄的 a、b、c 都沒有順序鍵，顯示順序 a、b、c，執行 `speclink change rank c --before b` | a、b 先補鍵（a 小於 b），c 落在兩者之間，plan 的順序變成 a、c、b |
| a 進行中、c 提案中，執行 `speclink change rank c --before a` | 拒絕：兩者不在同一欄 |

這條指令通常也不用你手動打：propose 與 ingest 收尾判定一個小而急的提案中變更該插隊時，會替你執行，見[提案：建立變更與產物](propose.md)。

## 拖排看板卡片時的依賴檢查

在看板同一欄拖動卡片，等於改它的順序鍵。引擎在寫回前用新的順序鍵重算同階段的基底順序：任何一個宣告前置會排到這張卡之後、或任何一個依賴它的變更會排到它之前，就拒絕，並列出這兩組名稱。只因 delta 重疊而要依序的變更不構成拒絕。這個檢查不寫任何檔案。`speclink change rank` 用的是同一個檢查。畫面上的表現見[看板與任務](desktop-board.md)。

## 哪些技能會看 plan

| 技能 | 怎麼用 plan |
| --- | --- |
| `/speclink-apply`、`/speclink-apply-with-worktree` | 挑變更的第一步先跑 plan：沒指名就取下一個可開工；指名的變更被擋住就印出前置並停止，不開審查工單、不蓋開工章。不再用「只有一個變更就自動選」繞過。見[實作：完成任務](apply.md)與[平行實作與合回](worktree.md) |
| `/speclink-archive` | 封存前跑 plan：目標封存要等別人時，提醒先封存它們、等它們封存後對照正式規格重寫再封存；有衝突時提醒先改掉其中一邊；宣告前置還沒封存時另外提一句。都只是建議，不擋封存。封存後再提醒排在後面、動到同名 requirement 的變更先重寫。見[封存](archive.md) |
| `/speclink-commit` 的先封存子流程 | 與 archive 相同的封存前提示與封存後提醒。見[提交單一變更的檔案](commit.md) |
| `/speclink-propose`、`/speclink-ingest` | 收尾判定軟依賴、落檔；小而急的提案中變更用 `speclink change rank` 插隊；再以 plan 呈現順序。見[提案：建立變更與產物](propose.md)與[續作與需求變更](drift-ingest.md) |

## remote 模式

remote 模式下 `speclink plan` 向 server 查詢，`speclink change depends` 向 server 寫入，人眼與 `--json` 輸出的長相都與本機模式相同，不會讀寫本機的 openspec 資料夾。server 回報依賴成環時，plan 同樣以錯誤結束，stderr 是 server 轉發的 `dependency cycle: …`。change depends 被 server 拒絕（找不到變更、守門失敗）時，stderr 印 server 轉發的一行訊息，以錯誤結束。

有兩件事只能在本機模式做。remote 模式下它們以錯誤結束，而且不向 server 發出任何請求：

- `speclink plan --strict-overlap`：stderr 印 `plan --strict-overlap is not available in remote mode`，stdout 是空的。
- `speclink change rank`：stderr 說明 remote 的順序由 server 上的看板順序文件決定。

remote 看板拖排也做同一套依賴檢查，只計涉及被拖卡的配對；違反時不送出寫回，畫面顯示與本地相同的一行錯誤。server 沒有 plan（舊 server、成環或請求失敗）時略過檢查。

**出處**：`change-plan`
