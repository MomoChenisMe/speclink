## MODIFIED Requirements

### Requirement: workflow-config set 政策欄位寫入

<!-- BEFORE: 寫入為 mapping 重新序列化的 read-modify-write——僅保證鍵值保留，明文記載「原檔的模板註解於重寫後喪失（read-modify-write 取捨）」，且新增鍵一律附加檔尾；本次改為文字層手術：非目標行逐位元保留、缺鍵按正典序插於 schema 之下、寫後解析等值驗證 -->

CLI SHALL 提供 speclink workflow-config set <key> <value>（旗標 --dry-run 與 --no-color，無 stdin），寫入政策欄位：key SHALL 限 locale、spec_locale、tdd、audit 四者，其他 key SHALL 以非零 exit code 拒絕且無檔案效果；tdd 與 audit 的 value SHALL 僅接受 true 或 false，其他值 SHALL 以非零 exit code 拒絕。locale 的 value SHALL 僅接受語系代碼 tw、ja、en，spec_locale 的 value SHALL 僅接受 tw、ja、en、auto；比對 SHALL 大小寫敏感（TW、Auto 均非法）；空字串（移除鍵、回到未設定＝預設）SHALL 仍合法；其他值——含顯示名稱字串（如「繁體中文」）——SHALL 以非零 exit code 拒絕，stderr SHALL 指出欄位名、收到的值與合法代碼集合，且無任何檔案效果；帶 --dry-run 時非法值 SHALL 同樣拒絕且 SHALL NOT 印出 diff。

寫入 SHALL 為文字層 read-modify-write：僅變動目標鍵所在的行或區塊，其餘行——含註解行、空行、未知頂層鍵與 schema 鍵下方的使用者自加內容——SHALL 逐位元保留（模板註解不再喪失）。tdd 或 audit 設為 false、locale 或 spec_locale 設為空字串時 SHALL 移除該鍵行（維持未設定＝預設語意），SHALL NOT 一併刪除其上方的註解行。新增原先不存在的政策鍵時 SHALL 依正典序（locale、spec_locale、tdd、audit）以連續區塊插於 schema 鍵行之後，區塊與前後內容行之間 SHALL 各恰一空行（相鄰處原已有空行時不重複補）；schema 鍵不存在時 SHALL 插於檔案最頂端。既有政策鍵 SHALL 原位改值、SHALL NOT 移動位置。寫入結果 SHALL 於落檔前重新解析並與目標狀態逐鍵等值比對，不等值 SHALL fail-closed 拒絕寫入且原檔逐位元不變。

帶 --dry-run 時 SHALL 於 stdout 印出變更前後的 unified diff、SHALL NOT 寫入任何檔案、exit code 0；無變更時 diff 為空。fs 模式寫入 openspec/config.yaml；remote 模式 SHALL 先經連線讀取 server 端 config 現行內容與版本，套用同一改寫、寫回時附帶讀得的版本——server 端版本已前進（他人並行改寫）時 SHALL 以非零 exit code 失敗、stderr 提示重新執行，SHALL NOT 覆蓋他人寫入；版本識別 SHALL NOT 出現在指令介面。連線離線或認證失效時 SHALL 以非零 exit code 失敗並輸出語義化訊息，SHALL NOT 暫存或排隊寫入。config 文件無法解析時 SHALL fail-closed 拒絕寫入（重寫壞檔會毀掉使用者內容）。成功 exit code 0、stdout 單行成功訊息。

#### Scenario: 設定 locale 保留其他鍵

- **WHEN** openspec/config.yaml 含 schema、tdd: true 與 context，執行 speclink workflow-config set locale tw
- **THEN** exit code 0；config.yaml 的 locale 為 tw，schema、tdd、context 的值不變

#### Scenario: 缺鍵插於 schema 之下且空行區隔

- **WHEN** config.yaml 依序含 schema、context、rules 而無任何政策鍵，執行 speclink workflow-config set locale tw
- **THEN** exit code 0；locale: tw 行位於 schema 鍵行之後、context 之前，與前後內容行之間各恰一空行；context 與 rules 的所有行逐位元不變

#### Scenario: 註解與空行逐位元保留

- **WHEN** config.yaml 含模板註解與使用者自加的註解行，執行任一合法的 set 寫入
- **THEN** exit code 0；除目標鍵所在行與插入區塊外，檔案所有行（含全部註解與空行）逐位元不變

#### Scenario: 檔尾既有鍵原位改值不搬家

- **WHEN** config.yaml 的 locale 鍵位於檔案最尾（rules 之後），執行 speclink workflow-config set locale ja
- **THEN** exit code 0；locale 行仍在檔尾原位、值改為 ja，其餘行逐位元不變

#### Scenario: schema 缺席時插於檔案最頂端

- **WHEN** config.yaml 無 schema 鍵且無政策鍵，執行 speclink workflow-config set tdd true
- **THEN** exit code 0；tdd: true 位於檔案最頂端，與後續內容之間恰一空行

#### Scenario: 內部改寫驗證失敗拒絕寫入

- **WHEN** 文字層手術產出的結果重新解析後與目標狀態不等值（防呆觸發）
- **THEN** exit code 非 0，stderr 單行錯誤指明內部改寫驗證失敗；openspec/config.yaml 逐位元不變

#### Scenario: 未知 key 拒絕

- **WHEN** 執行 speclink workflow-config set theme dark
- **THEN** exit code 非 0，stderr 指出 key 限 locale、spec_locale、tdd、audit；openspec/config.yaml 逐位元不變

#### Scenario: 非法布林值拒絕

- **WHEN** 執行 speclink workflow-config set tdd yes
- **THEN** exit code 非 0，stderr 指出 tdd 僅接受 true 或 false；無任何檔案效果

#### Scenario: 非法 locale 值拒絕

- **WHEN** 執行 speclink workflow-config set locale 繁體中文
- **THEN** exit code 非 0，stderr 指出 locale 欄位、收到的值「繁體中文」與合法代碼 tw、ja、en；openspec/config.yaml 逐位元不變

#### Scenario: 非法值帶 dry-run 同樣拒絕

- **WHEN** 執行 speclink workflow-config set spec_locale 繁體中文 --dry-run
- **THEN** exit code 非 0，stdout 無 diff 輸出，stderr 列出 spec_locale 的合法代碼（tw、ja、en、auto）；無任何檔案效果

##### Example: locale 值域判定

| key | value | 結果 |
| --- | ----- | ---- |
| locale | tw | 成功 |
| locale | 空字串 | 成功（移除鍵） |
| locale | 繁體中文 | 拒絕 |
| locale | TW | 拒絕（大小寫敏感） |
| spec_locale | auto | 成功 |
| spec_locale | zh-Hant | 拒絕 |

#### Scenario: 設 false 移除鍵

- **WHEN** openspec/config.yaml 含 audit: true，執行 speclink workflow-config set audit false
- **THEN** exit code 0；config.yaml 不含 audit 鍵（未設定＝預設關閉），該鍵上方的註解行仍在

#### Scenario: dry-run 印 diff 不落檔

- **WHEN** 執行 speclink workflow-config set locale ja --dry-run
- **THEN** exit code 0；stdout 為 unified diff 且僅含 locale 相關行的變更（不含其他行的重排）；openspec/config.yaml 逐位元不變

#### Scenario: remote 版本衝突提示重跑

- **WHEN** 於 remote 模式執行 speclink workflow-config set tdd true，且 server 端 config 在讀取後、寫回前已被他人改寫
- **THEN** exit code 非 0，stderr 說明設定已被他人更新、請重新執行；server 端內容維持他人寫入的版本

#### Scenario: 壞 config 拒絕寫入

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config set locale tw
- **THEN** exit code 非 0，stderr 指出該檔無法解析、寫入已拒絕；檔案逐位元不變
