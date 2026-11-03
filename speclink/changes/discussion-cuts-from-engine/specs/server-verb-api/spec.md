## ADDED Requirements

### Requirement: 討論詳情回應攜帶分刀

GET /discussions/{slug} 的回應 SHALL 攜帶 `cuts`（形狀依 client-protocol「討論詳情回應的分刀 payload」），live 與已封存的記錄皆然。`cuts` SHALL 只由引擎的分刀解析計算，輸入為該次請求讀到的同一份記錄的結論與 `promoted_to`——與回應的 `content` 及 `info.promotedTo` 出自同一次讀取；server SHALL NOT 重讀檔、SHALL NOT 另寫一份分刀解析。記錄沒有結論或結論沒有刀時 `cuts` SHALL 為空陣列。既有的 `info` 與 `content` SHALL 不變；slug 不存在時照舊回 404、無 `cuts`。

#### Scenario: live 記錄帶分刀

- **WHEN** live 討論 panel 的 promoted_to 為 `add-y`，結論的 Decision 有 ``- **cut 2 `add-y`**: 引擎解析`` 與 ``- **cut 3 `add-z`**: 路線圖`` 兩條列點，client 以具讀取權限的身分呼叫 GET /discussions/panel
- **THEN** 回應 200，`cuts` 為兩項：第一項 cut 為 `cut 2`、changes 為 ["add-y"]、unpromoted 為 []；第二項 cut 為 `cut 3`、title 為「路線圖」、changes 為 ["add-z"]、unpromoted 為 ["add-z"]；`info.promotedTo` 為 ["add-y"]

#### Scenario: 已封存記錄帶分刀

- **WHEN** 討論 old-topic 已封存，其結論有一刀 ``- **cut 1 `add-a`**: 骨架``、promoted_to 為 `add-a`，client 呼叫 GET /discussions/old-topic
- **THEN** 回應 200，`info.archived` 為 true，`cuts` 為一項，cut 為 `cut 1`、unpromoted 為 []

#### Scenario: 沒有結論的記錄

- **WHEN** 討論 fresh-topic 只有 Context 與 Rounds、沒有結論，client 呼叫 GET /discussions/fresh-topic
- **THEN** 回應 200，`cuts` 為空陣列，`content` 為記錄原文
