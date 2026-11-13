## ADDED Requirements

### Requirement: 變更回應攜帶關卡狀態與落地記錄

變更清單端點與單筆變更讀取端點 SHALL 依 Client Protocol 的定義回填 `review` 與 `verify`（工單是否存在、章的欄位）以及分支與落地記錄；已封存清單端點 SHALL 回填兩個關卡的結局。回填 SHALL 來自該範圍的儲存：工單是否存在由該變更底下的文件鍵判定，章與落地記錄由 metadata 判定；它 SHALL NOT 為此多發儲存呼叫（清單端點對儲存的呼叫數與回填前相同）。metadata 無法解析的變更 SHALL NOT 帶關卡物件，其餘欄位照既有行為。server SHALL 提供落地回報端點，以引擎的落地命令執行，editor 以上角色可用。蓋章端點 SHALL 把請求的 headCommit 交給引擎；開工標記端點 SHALL 把請求的 branch 交給引擎。

#### Scenario: 清單回填三態

- **WHEN** 範圍有三個變更：一個沒有工單也沒有章、一個有審查工單、一個審查已蓋章，請求變更清單
- **THEN** 三個項目的 review.state 依序是 `none`、`inProgress`、`stamped`

#### Scenario: 回填不多發儲存呼叫

- **WHEN** 以計數的儲存比較回填前後變更清單端點對儲存的呼叫
- **THEN** 列出次數與讀取次數都沒有增加

#### Scenario: 壞 metadata 的變更

- **WHEN** 一個變更的 metadata 無法解析，請求變更清單
- **THEN** 該項目沒有 review 與 verify，仍帶既有的 metaError；其他項目照常

#### Scenario: 落地回報端點

- **WHEN** editor 對 changes/demo/landed 送出合法的 commit
- **THEN** 回應是 200；之後單筆變更回應的 landedCommit 是該值，landedBy 是該使用者的顯示名稱

#### Scenario: reader 不能回報

- **WHEN** reader 送出落地回報
- **THEN** 回應是 403
