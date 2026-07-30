## ADDED Requirements

### Requirement: 技能規定政策語系欄位寫入代碼

渲染後的 speclink-config 技能文件（所有 tool flavor）SHALL 於政策欄位段落明文規定：locale 僅接受語系代碼 tw、ja、en，spec_locale 僅接受 tw、ja、en、auto；SHALL 要求執行技能的 agent 把使用者的自然語言回答映射為代碼後寫入，並 SHALL 附至少一組映射示例（「繁體中文」→ tw）；SHALL 明文禁止把顯示名稱字串當作值寫入。內嵌資產與 repo 技能實例的同步一致性歸既有需求「內嵌 speclink-config 技能的渲染與保護」管轄，本需求 SHALL NOT 另立同步機制。

#### Scenario: 渲染文件含代碼指引

- **WHEN** 於啟用 claude 工具的專案渲染 speclink-config 技能
- **THEN** 產出的技能文件含 locale 與 spec_locale 的合法代碼集合、「繁體中文」→ tw 的映射示例，以及禁止寫入顯示名稱的指示

#### Scenario: 技能執行不再寫入顯示名稱

- **WHEN** agent 依更新後的技能文件執行設定流程，使用者以「繁體中文」回答語系偏好
- **THEN** agent 對 workflow-config set 寫入的值為 tw 而非「繁體中文」
