## ADDED Requirements

### Requirement: 驗證子代理的模型設定

技能 SHALL 於讀完 artifacts 與任務進度之後、派出檢查子代理之前，以 speclink workflow-config show --json 取得 subagentModel.verify 物件，SHALL NOT 直接讀取 config 檔；remote 模式經同一動詞取得 server 端設定。該指令失敗（例如 config 無法解析）時技能 SHALL 原樣回報錯誤並停止，SHALL NOT 派出子代理、SHALL NOT 寫工單。生成的技能檔 SHALL 以渲染時代入的工具名稱指出要讀的格子（例：claude 的技能檔讀 subagentModel.verify.claude），技能檔 SHALL NOT 寫入任何模型值；改 subagent_model 設定 SHALL NOT 需要重新生成技能檔即生效。

派出檢查子代理時（成品驗證、中途盤點與 validation 皆同）：

- 該格有值 → 子代理呼叫 SHALL 帶這個模型。
- 該格沒設定 → 子代理呼叫 SHALL NOT 帶模型參數，子代理沿用主對話的模型。
- 該格有值，但工具的子代理呼叫沒有模型參數（例如 Codex 未在 ~/.codex/config.toml 設 expose_spawn_agent_model_overrides = true）→ 技能 SHALL 照常派出子代理、沿用主對話的模型，並在報告中寫明設定沒有套用；工具為 Codex 時 SHALL 附上該設定的開法。
- 該格有值，但工具拒絕這個模型名稱 → 技能 SHALL 停在派子代理那一步：原文回報工具的錯誤，指出要改 speclink/config.yaml 的 subagent_model.verify.<工具名稱>（例如以 speclink workflow-config subagent-model verify <工具名稱> <模型名稱> 改寫），SHALL NOT 改用主對話的模型重派，SHALL NOT 執行 verify add-round、SHALL NOT 蓋章。
- 工具不能派子代理 → 技能依既有退路在主對話裡跑檢查，設定 SHALL NOT 套用，報告中寫明這一點。

派出子代理的那一次，驗證報告開頭（報告標題之後）SHALL 恰有一行子代理模型行，寫明本次子代理用的模型或上述不套用的原因；寫法與審查站的子代理模型行相同，鍵路徑為 subagent_model.verify.<工具名稱>。該行 SHALL 以解析後的 locale 撰寫（tw、ja、en；未設定時為英文），模型名稱、subagent_model 鍵路徑與設定名稱維持原文。該行只出現在報告中，SHALL NOT 寫入工單。

#### Scenario: 有設定時帶模型

- **WHEN** config 的 subagent_model.verify.claude 為 sonnet，在 Claude Code 對程式任務全數完成的變更執行 speclink-verify
- **THEN** 檢查子代理呼叫帶 sonnet；驗證報告標題之後一行寫明子代理模型為 sonnet；工單不含此行

##### Example: 子代理模型行（locale 為 tw）

| 情況 | 子代理模型行 |
| ---- | ------------ |
| subagent_model.verify.claude 為 sonnet | 子代理模型：sonnet（subagent_model.verify.claude） |
| subagent_model.verify.claude 沒有設定 | 子代理模型：沿用主對話（subagent_model.verify.claude 沒有設定） |

#### Scenario: 沒有設定時不帶模型

- **WHEN** config 的 subagent_model 只有 review 的 claude: opus，在 Claude Code 執行 speclink-verify
- **THEN** 檢查子代理呼叫不帶模型參數；報告的子代理模型行寫明 subagent_model.verify.claude 沒有設定、沿用主對話

#### Scenario: 工具拒絕模型名稱時停下

- **WHEN** config 的 subagent_model.verify.claude 為 sonet（拼錯），在 Claude Code 執行 speclink-verify，Agent 工具拒絕這個模型名稱
- **THEN** 技能原文回報工具的錯誤，指出要改 subagent_model.verify.claude，然後停止；不以主對話的模型重派、不執行 verify add-round、不蓋章

#### Scenario: Codex 未開放模型欄位時照常驗證

- **WHEN** config 的 subagent_model.verify.codex 為 gpt-6.1-sol，在未開放 spawn_agent 模型欄位的 Codex 執行 speclink-verify
- **THEN** 技能照常派出檢查子代理並完成本輪；報告的子代理模型行寫明設定沒有套用，並附上 ~/.codex/config.toml 的 expose_spawn_agent_model_overrides = true 開法

#### Scenario: 讀不到設定即停止

- **WHEN** speclink/config.yaml 的 subagent_model 區塊含未知子鍵，執行 speclink-verify
- **THEN** 技能原樣回報錯誤並停止，不派出子代理、不寫工單
