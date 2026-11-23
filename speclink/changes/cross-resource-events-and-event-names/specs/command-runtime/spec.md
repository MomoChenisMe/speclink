## ADDED Requirements

### Requirement: 一次提交的事件涵蓋它寫到的每個變更與討論

一個變更型動詞成功時，它寫入或刪除了哪些進行中的變更與進行中的討論的文件，命令層回報的事件 SHALL 就指到哪些變更與討論：對每一個被寫到的變更，SHALL 至少有一筆主體為該 change 名的事件；對每一個被寫到的討論，SHALL 至少有一筆主體為該 discussion slug 的事件。一筆事件 SHALL 只有一個主體。命令原本的事件 SHALL 排在前面，為其他資源新增的事件 SHALL 排在後面。事件 SHALL 只為真的發生的寫入回報：某個資源的文件這次沒有被改寫時，SHALL NOT 為它回報事件。事件 SHALL NOT 帶文件的內容或欄位的值。

順便寫到其他資源的動詞與它們多回報的事件 SHALL 是：

| 動詞與情況 | 多回報的事件 | 主體與附帶的名稱 |
| --- | --- | --- |
| new change 帶 `--from-discussion` | discussion-promoted | 討論 slug；附帶新 change 名 |
| archive，來源討論被連帶封存 | 每份討論一筆 discussion-archived | 討論 slug |
| discuss conclude，討論在同一次執行被順手封存 | discussion-archived | 討論 slug |
| discard，來源討論被解開 | 每份討論一筆 discussion-unlinked | 討論 slug；附帶被廢棄的 change 名 |
| discuss conclude，已反映的變更新記上待重新反映 | 每個變更一筆 change-provenance-changed | change 名 |
| discuss link，討論第一次被寫進變更的來源 | change-provenance-changed | change 名 |
| discuss seal，變更的待重新反映被清掉 | change-provenance-changed | change 名 |

`discussion-unlinked` 與 `change-provenance-changed` SHALL 是新的事件種類名。這條規則 SHALL 只管進行中的變更與討論；已封存的文件、正式規格、流程設定、詞彙表與看板順序的寫入 SHALL 維持既有的事件。範圍的匯入改的是整個範圍，SHALL 維持只回報它自己的一筆事件。這條規則 SHALL 由一個涵蓋每一種變更型命令的自動化檢查守住：新增一種變更型命令而沒有納入檢查時，檢查 SHALL 失敗。這些動詞的人眼輸出與 `--json` 輸出 SHALL 維持既有的位元級輸出。

#### Scenario: 廢棄變更時解開的討論有自己的事件

- **WHEN** change add-two 由討論 two 轉出，經引擎命令層廢棄 add-two，討論 two 因此被解開
- **THEN** 執行結果依序附帶 change-discarded（主體為 add-two）與 discussion-unlinked（主體為 two，並帶 change 名 add-two）

#### Scenario: 重新結論只為新記上標記的變更回報

- **WHEN** 討論 two 的結論已反映到 change add-two，經引擎命令層對 two 重新結論兩次
- **THEN** 第一次的執行結果附帶 discussion-concluded 與一筆 change-provenance-changed（主體為 add-two）；第二次 add-two 已經帶著標記、沒有被改寫，執行結果只有 discussion-concluded

#### Scenario: 順手封存的結論回報封存事件

- **WHEN** 討論 two 轉出的變更都已封存，經引擎命令層對 two 寫入結論，討論在同一次執行被封存
- **THEN** 執行結果依序附帶 discussion-concluded 與 discussion-archived，主體都是 two

#### Scenario: link 與 seal 只在改到變更時多回報

- **WHEN** 依序執行：把討論 two 連結到 change add-three（第一次）；再連結一次；對帶著待重新反映標記的 add-two 執行 seal；對沒有標記的 add-two 再執行一次 seal
- **THEN** 四次執行的事件依序是：discussion-linked 與 change-provenance-changed（主體為 add-three）；只有 discussion-linked；discussion-sealed 與 change-provenance-changed（主體為 add-two）；只有 discussion-sealed

##### Example: 各種情況的事件

| 執行 | 事件（依序） |
| --- | --- |
| `new change add-two --from-discussion two` | change-created(add-two)、discussion-promoted(two, add-two) |
| `new change solo` | change-created(solo) |
| `archive add-two`，討論 two 還有別的變更在途 | change-archived(add-two) |
| `archive add-two`，討論 two 被連帶封存 | change-archived(add-two)、discussion-archived(two) |
| `discard add-two`，沒有來源討論 | change-discarded(add-two) |
| `discuss promote two --name add-two` | discussion-promoted(two, add-two)、change-created(add-two) |

#### Scenario: 沒有順便寫到其他資源時事件不變

- **WHEN** 經引擎命令層建立沒有來源討論的 change solo，並廢棄一個沒有來源討論的 change
- **THEN** 兩次執行各只有一筆事件（change-created、change-discarded），與這條需求之前相同

#### Scenario: 輸出不變

- **WHEN** 比對這條需求前後 `speclink discard add-two`、`speclink archive add-two` 與 `speclink discuss conclude two --stdin` 的人眼輸出與 `--json` 輸出
- **THEN** 每一個輸出逐位元相同

### Requirement: 引擎提供全部事件名稱的清單

引擎 SHALL 提供一份清單，列出每一種領域事件的種類名，每個名稱正好一次；命令層回報的每一筆事件的種類名 SHALL 在這份清單裡。清單裡名稱的順序 SHALL NOT 是契約的一部分。清單 SHALL 與事件種類的定義保持一致：新增一種事件而沒有把名稱加進清單時，自動化檢查 SHALL 失敗。清單 SHALL NOT 帶事件的其他性質。

#### Scenario: 清單涵蓋每一種事件

- **WHEN** 取得引擎的事件名稱清單
- **THEN** 它含 37 個不重複的名稱，包括 `change-created`、`task-completed`、`review-stamped`、`discussion-promoted`、`workflow-config-changed`、`discussion-unlinked` 與 `change-provenance-changed`

#### Scenario: 回報的事件名稱都在清單裡

- **WHEN** 依序執行建立變更、勾選任務、從討論轉出變更、廢棄變更，收集每次執行回報的事件種類名
- **THEN** 每一個名稱都在清單裡

## MODIFIED Requirements

### Requirement: 變更型動詞的領域事件

<!-- BEFORE: new change、archive、discard、discuss conclude、discuss link、discuss seal 各只回報一筆自己主體的事件 -->

覆蓋表內每個變更型動詞成功時，命令層 SHALL 隨執行結果回報一至多筆領域事件，每筆事件 SHALL 含種類名、主體識別（change 名或 discussion slug）與 UTC 時間戳；查詢型動詞與失敗的執行 SHALL NOT 產生事件。本能力 SHALL NOT 含事件持久化與訂閱；事件契約標示為 experimental，於事件持久化能力落地前 SHALL 允許不相容調整。

#### Scenario: 建立變更回報 change-created

- **WHEN** 經引擎命令層成功建立名為 add-auth 的 change
- **THEN** 執行結果附帶恰一筆 change-created 事件，主體為 add-auth 且含 UTC 時間戳

#### Scenario: 失敗的命令不產生事件

- **WHEN** 以已存在的名稱再次建立 change 而失敗
- **THEN** 執行結果為錯誤且不附帶任何事件

#### Scenario: 複合動詞回報多筆事件

- **WHEN** 經引擎命令層將已結論的討論 promote 成新 change
- **THEN** 執行結果附帶 discussion-promoted（主體為討論 slug）與 change-created（主體為新 change 名）兩筆事件

##### Example: 變更型動詞與事件種類對應

| 動詞 | 事件種類 |
| --- | --- |
| new change | change-created；帶 `--from-discussion` 時另有 discussion-promoted |
| new artifact | artifact-created |
| task done | task-completed |
| task undone | task-uncompleted |
| task move | task-moved |
| claim | change-claimed |
| in-progress add | change-marked-in-progress |
| archive | change-archived；每連帶封存一份討論另有一筆 discussion-archived |
| discard | change-discarded；每解開一份討論另有一筆 discussion-unlinked |
| discuss new | discussion-created |
| discuss context | discussion-context-set |
| discuss add-round | discussion-round-added |
| discuss conclude | discussion-concluded；順手封存討論時另有 discussion-archived；每個新記上待重新反映的變更另有一筆 change-provenance-changed |
| discuss promote | discussion-promoted 與 change-created |
| discuss link | discussion-linked；真的寫進變更的來源時另有 change-provenance-changed |
| discuss seal | discussion-sealed；真的清掉變更的待重新反映時另有 change-provenance-changed |
| discuss archive | discussion-archived |
| discuss discard | discussion-discarded |

#### Scenario: 從討論建立變更回報兩筆事件

- **WHEN** 經引擎命令層以 `--from-discussion two` 成功建立名為 add-two 的 change
- **THEN** 執行結果依序附帶 change-created（主體為 add-two）與 discussion-promoted（主體為討論 two，並帶 change 名 add-two）兩筆事件

#### Scenario: 新增的事件排在原本的事件之後

- **WHEN** 經引擎命令層封存 change add-two，它的來源討論 two 被連帶封存
- **THEN** 執行結果依序附帶 change-archived（主體為 add-two）與 discussion-archived（主體為 two）
