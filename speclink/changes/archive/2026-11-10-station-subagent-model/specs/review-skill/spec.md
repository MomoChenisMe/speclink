## ADDED Requirements

### Requirement: 審查子代理的模型設定

技能 SHALL 從第 2 步已執行的 speclink workflow-config show --json 一併取得 subagentModel.review 物件，SHALL NOT 直接讀取 config 檔；remote 模式經同一動詞取得 server 端設定。生成的技能檔 SHALL 以渲染時代入的工具名稱指出要讀的格子（例：claude 的技能檔讀 subagentModel.review.claude，codex 的技能檔讀 subagentModel.review.codex），技能檔 SHALL NOT 寫入任何模型值。改 subagent_model 設定 SHALL NOT 需要重新生成技能檔即生效。

派出審查子代理時（discovery 與 validation 皆同）：

- 該格有值 → Standards 與 Correctness 兩個子代理的呼叫 SHALL 都帶這個模型（兩軸共用一個值）。
- 該格沒設定 → 子代理呼叫 SHALL NOT 帶模型參數，子代理沿用主對話的模型。
- 該格有值，但工具的子代理呼叫沒有模型參數（例如 Codex 未在 ~/.codex/config.toml 設 expose_spawn_agent_model_overrides = true）→ 技能 SHALL 照常派出子代理、沿用主對話的模型，並在呈現中寫明設定沒有套用；工具為 Codex 時 SHALL 附上該設定的開法。
- 該格有值，但工具拒絕這個模型名稱 → 技能 SHALL 停在派子代理那一步：原文回報工具的錯誤，指出要改 speclink/config.yaml 的 subagent_model.review.<工具名稱>（例如以 speclink workflow-config subagent-model review <工具名稱> <模型名稱> 改寫），SHALL NOT 改用主對話的模型重派，SHALL NOT 執行 review add-round、SHALL NOT 蓋章。
- 工具不能派子代理 → 技能依既有退路在主對話裡依序跑兩軸，設定 SHALL NOT 套用，呈現中寫明這一點。

派出子代理的那一輪，技能於呈現開頭的規則行之後 SHALL 恰印一行子代理模型行，寫明本輪子代理用的模型或上述不套用的原因。該行 SHALL 依「審查產出的語言綁定」以解析後的 locale 撰寫（tw、ja、en；未設定時為英文），模型名稱、subagent_model 鍵路徑與設定名稱維持原文。該行只出現在呈現中，SHALL NOT 寫入工單。

#### Scenario: 有設定時兩軸都帶模型

- **WHEN** config 的 subagent_model.review.claude 為 opus，在 Claude Code 執行 speclink-review 的 discovery
- **THEN** Standards 與 Correctness 兩個子代理呼叫都帶 opus；呈現開頭的規則行之後一行寫明子代理模型為 opus；工單不含此行

##### Example: 各情況的子代理模型行（locale 為 tw）

| 情況 | 子代理模型行 |
| ---- | ------------ |
| subagent_model.review.claude 為 opus | 子代理模型：opus（subagent_model.review.claude） |
| subagent_model.review.claude 沒有設定 | 子代理模型：沿用主對話（subagent_model.review.claude 沒有設定） |
| Codex、subagent_model.review.codex 為 gpt-6.1-sol、未開放 model 欄位 | 子代理模型：沿用主對話——Codex 沒有開放子代理指定模型，本輪用主對話的模型。要開放，請在 ~/.codex/config.toml 設 expose_spawn_agent_model_overrides = true。 |
| 工具不能派子代理 | 子代理模型：不適用——這個工具不能派子代理，本輪檢查在主對話裡跑 |

#### Scenario: 沒有設定時不帶模型

- **WHEN** config 沒有 subagent_model 區塊，執行 speclink-review
- **THEN** 兩個子代理呼叫都不帶模型參數；呈現的子代理模型行寫明沿用主對話

#### Scenario: 只設了其他工具時沿用主對話

- **WHEN** config 的 subagent_model.review 只有 claude: opus，在 Codex 執行 speclink-review
- **THEN** 子代理呼叫不帶模型參數；呈現的子代理模型行寫明 subagent_model.review.codex 沒有設定、沿用主對話

#### Scenario: 工具拒絕模型名稱時停下

- **WHEN** config 的 subagent_model.review.claude 為 opux（拼錯），在 Claude Code 執行 speclink-review，Agent 工具拒絕這個模型名稱
- **THEN** 技能原文回報工具的錯誤，指出要改 subagent_model.review.claude，然後停止；不以主對話的模型重派、不執行 review add-round、不蓋章

#### Scenario: 技能檔寫明要讀的格子

- **WHEN** 分別檢視生成的 claude 與 codex 版 speclink-review 技能檔
- **THEN** claude 版寫明讀 subagentModel.review.claude，codex 版寫明讀 subagentModel.review.codex

#### Scenario: 改設定不改技能檔

- **WHEN** 把 subagent_model.review.claude 由 opus 改為 sonnet，再執行 speclink update
- **THEN** 生成的技能檔逐位元不變；下一次 speclink-review 的子代理帶 sonnet
