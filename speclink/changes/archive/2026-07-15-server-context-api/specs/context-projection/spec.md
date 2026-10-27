## ADDED Requirements

### Requirement: 遠端投影以 Context API 為來源

remote 動詞流程的投影供應者 SHALL 以 Context API 的一致快照為來源，SHALL NOT 以逐 artifact 分次請求拼裝快照；投影內容 SHALL 因此涵蓋正典 specs、該 change 的 delta specs、artifacts、config 與 LANGUAGE（既有佈局需求的完整實現）。manifest 現值的 snapshot id 與 server 現值相同時，refresh SHALL 免重寫投影；Context API 失敗時 SHALL 維持既有韌性語意——響亮警告、動詞照常完成、既有投影標記 stale。

#### Scenario: 投影含正典與 delta specs

- **WHEN** 對含正典 specs 與 delta specs 的 remote scope 執行 apply 階段動詞後檢視投影
- **THEN** 投影鏡像含正典 specs、該 change 的 delta specs 與 artifacts；manifest 的 snapshot id 為 server 回應的識別；verify 通過

#### Scenario: 未變免重寫

- **WHEN** 連續兩次執行同一 remote 動詞且期間 server 無任何 commit
- **THEN** 第二次不重寫投影（檔案不變動）；期間發生 commit 後再執行則投影更新為新快照

#### Scenario: API 失敗不阻斷動詞

- **WHEN** Context API 回 503 期間執行 remote 動詞
- **THEN** 動詞以既有 exit code 完成；stderr 含投影未刷新警告；既有投影被標記 stale
