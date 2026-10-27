## MODIFIED Requirements

### Requirement: 變更型動詞的領域事件

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
| new change | change-created |
| new artifact | artifact-created |
| task done | task-completed |
| task undone | task-uncompleted |
| task move | task-moved |
| claim | change-claimed |
| in-progress add | change-marked-in-progress |
| archive | change-archived |
| discard | change-discarded |
| discuss new | discussion-created |
| discuss context | discussion-context-set |
| discuss add-round | discussion-round-added |
| discuss conclude | discussion-concluded |
| discuss promote | discussion-promoted 與 change-created |
| discuss link | discussion-linked |
| discuss seal | discussion-sealed |
| discuss archive | discussion-archived |
| discuss discard | discussion-discarded |
