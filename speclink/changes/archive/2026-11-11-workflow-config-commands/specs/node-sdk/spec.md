## ADDED Requirements

### Requirement: dispatch 的流程設定動詞

dispatch SHALL 認得 workflow-config 的全部動詞（show、set、context、rules、review 的四個、subagent-model 的三個），argv 與 stdin 沿用 CLI 的詞彙，回傳內容與 CLI 同一動詞的輸出同源。宿主 Store 物件 SHALL 可提供一個選填方法 writeWorkflowConfig(content)，回傳值或 Promise 皆可。宿主 Store 沒有提供這個方法時，會寫入的 workflow-config 動詞 SHALL 在動手前以語義化訊息拒絕、code 為 refused，宿主 Store 的任何寫入方法 SHALL NOT 被呼叫；show 與帶 --dry-run 的動詞 SHALL 不受影響。argv 帶 --stdin 而 dispatch 沒有收到 stdin 內容時，workflow-config 的動詞 SHALL 以 code invalid_argv 拒絕，宿主 Store 的任何寫入方法 SHALL NOT 被呼叫；空字串是有效的內容（清掉該值）。fs 形式建構的引擎 SHALL 直接寫入規格資料夾的 config.yaml。

#### Scenario: 經 dispatch 設定政策欄位

- **WHEN** 宿主 Store 提供 writeWorkflowConfig，現有流程設定含 tdd: false，執行 dispatch(['workflow-config', 'set', 'tdd', 'true'])
- **THEN** writeWorkflowConfig 被呼叫正好一次，收到的內容裡 tdd 是 true；Promise 解析為成功結果

#### Scenario: 沒有寫入方法時拒絕

- **WHEN** 宿主 Store 沒有 writeWorkflowConfig，執行 dispatch(['workflow-config', 'set', 'tdd', 'true'])
- **THEN** Promise 以 Error 拒絕，code 為 refused，message 指出宿主 Store 缺少 writeWorkflowConfig；宿主 Store 沒有任何寫入方法被呼叫

#### Scenario: 讀取與 dry-run 不需要寫入方法

- **WHEN** 宿主 Store 沒有 writeWorkflowConfig，分別執行 dispatch(['workflow-config', 'show']) 與 dispatch(['workflow-config', 'set', 'tdd', 'true', '--dry-run'])
- **THEN** 兩者都成功

#### Scenario: 帶 --stdin 卻沒有 stdin 內容時拒絕

- **WHEN** 宿主 Store 的流程設定含 rules.tasks 一條規則，執行 dispatch(['workflow-config', 'rules', 'tasks', '--stdin']) 而沒有傳 stdin
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv；流程設定沒有被改動。改傳 stdin 為空字串時成功，rules.tasks 被移除

#### Scenario: 壞的流程設定經 dispatch 拒絕

- **WHEN** 宿主 Store 的流程設定讀取方法回傳無法解析的 YAML，執行 dispatch(['workflow-config', 'show'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_config
