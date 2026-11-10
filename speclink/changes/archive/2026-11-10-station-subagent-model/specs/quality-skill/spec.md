## ADDED Requirements

### Requirement: 三個子代理各帶自己站的子代理模型

技能同一則訊息派出的三個子代理 SHALL 各依自己站的子代理模型設定帶模型：審查站的 Standards 與 Correctness 兩個子代理依 subagentModel.review.<工具名稱>，驗證站的檢查子代理依 subagentModel.verify.<工具名稱>；有值才帶、沒值不帶，取值與例外處理依兩站各自的規則。技能 SHALL NOT 另設 quality 專屬的子代理模型。

三個子代理中任一個因模型名稱被工具拒絕時，技能 SHALL 結束本輪而不記錄任何一站：原文回報工具的錯誤並指出要改的 subagent_model 鍵路徑，SHALL NOT 執行任一站的 add-round、SHALL NOT 蓋章，其他子代理的回報 SHALL NOT 寫入工單——維持「一輪兩站都檢查或都不檢查」。工具的子代理呼叫沒有模型參數、或工具不能派子代理時，兩站照各自的規則沿用主對話的模型或在主對話裡跑，本輪照常進行。

每輪停下時的合併摘要 SHALL 保留兩站各自的子代理模型行（審查站的那一行接在規則行之後）。

#### Scenario: 兩站各帶自己的模型

- **WHEN** config 的 subagent_model.review.claude 為 opus、subagent_model.verify.claude 為 sonnet，在 Claude Code 執行 speclink-quality
- **THEN** 同一則訊息派出的三個子代理中，審查的兩個帶 opus、驗證的一個帶 sonnet；停下時的合併摘要含兩站各自的子代理模型行

#### Scenario: 只有審查站有設定

- **WHEN** config 的 subagent_model 只有 review 的 codex: gpt-6.1-sol，在已開放 spawn_agent 模型欄位的 Codex 執行 speclink-quality
- **THEN** 審查的兩個子代理帶 gpt-6.1-sol，驗證的子代理不帶模型參數；驗證站的子代理模型行寫明沿用主對話

#### Scenario: 任一子代理的模型名稱被拒時本輪不記錄

- **WHEN** config 的 subagent_model.verify.claude 為 sonet（拼錯）、subagent_model.review.claude 為 opus，在 Claude Code 執行 speclink-quality，Agent 工具拒絕 sonet
- **THEN** 技能原文回報工具的錯誤並指出 subagent_model.verify.claude，然後停止；審查與驗證兩張工單都沒有新增一輪，也沒有蓋章
