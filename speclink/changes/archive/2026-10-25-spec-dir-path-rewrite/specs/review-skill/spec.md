## MODIFIED Requirements

### Requirement: 審查產出的語言綁定

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能文件 SHALL 將 workflow config 解析出的 locale（守門自檢步驟所取 payload 的 `locale` 欄位）綁定整條審查產出鏈：兩軸 sub-agent 指示 SHALL 攜帶該語言並要求 finding 描述以該語言撰寫；並列呈現與 `review add-round` 寫入工單的 findings 描述 SHALL 與 sub-agent 產出同語言，主線 SHALL NOT 另行翻譯。severity 標籤（CRITICAL／WARNING／SUGGESTION）、軸前綴（`Standards:`／`Correctness:`）、檔案路徑與指令行 SHALL 維持英文；工單固定骨架屬動詞所有文法（見 review-station 規格），不在本條範圍。locale 未設定時，sub-agent 報告、呈現與工單記錄 SHALL 為英文。smell baseline 正典原文（design「D7a」）屬 sub-agent 指示本文，SHALL 維持逐字英文內嵌，不受本條影響。

#### Scenario: locale tw 時工單 findings 為中文

- **WHEN** 專案 speclink/config.yaml 設 locale: tw，審查輪產出 findings 並以 `review add-round` 寫入工單
- **THEN** 該輪 findings 行的描述文字為繁體中文，severity 標籤與 `Standards:`／`Correctness:` 前綴維持英文

#### Scenario: locale 未設定時全英文

- **WHEN** 專案未設定 locale，執行一輪審查
- **THEN** sub-agent 報告、並列呈現與工單 findings 描述皆為英文

### Requirement: 專案審查規則的讀取與套用

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能 SHALL 於開審前以 speclink workflow-config show --json 讀取 review 物件（baseline 與 critical、warning、suggestion 三個清單），SHALL NOT 直接讀取 config 檔；remote 模式經同一動詞取得 server 端設定。讀取失敗（例如 config 無法解析）時技能 SHALL 原樣回報錯誤並停止，SHALL NOT 在缺少專案規則的情況下開審。

Discovery 時 Standards 軸的 sub-agent 簡報 SHALL 依嚴重度分組帶入三個清單的規則原文；Correctness 軸簡報 SHALL NOT 帶入。違反專案規則的 finding SHALL 以該規則所在清單決定嚴重度（critical→CRITICAL、warning→WARNING、suggestion→SUGGESTION），sub-agent 與主線 SHALL NOT 另行升降級；finding 描述 SHALL 原文引用被違反的規則，描述其餘部分依「審查產出的語言綁定」使用解析後的 locale。專案規則與 repo 文件化標準衝突時，專案規則 SHALL 勝出。Validation 輪 SHALL NOT 因專案規則產生新 finding——續輪只判前輪 finding 是否已解與 remediation patch 的直接 regression，與既有語意相同。三個清單皆空且 baseline 為 extend 時，審查行為 SHALL 與未設定 review 時完全相同。

#### Scenario: 違反 critical 清單的規則

- **WHEN** config 的 review.critical 含「所有 SQL 一律用參數化查詢，不得用字串拼接」，frozen patch 中有一段以字串拼接組出 SQL
- **THEN** Standards 軸回報一筆 CRITICAL finding，描述原文引用該規則；主線照該嚴重度入單並分類為必修

##### Example: 工單中的 finding 行（locale 為 tw）

- **GIVEN** review.critical 含「所有 SQL 一律用參數化查詢，不得用字串拼接」，locale 為 tw
- **WHEN** crates/x/src/db.rs 第 42 行以 format! 拼接 WHERE 子句
- **THEN** 工單記為 `- [CRITICAL] crates/x/src/db.rs — Standards: 違反專案審查規則「所有 SQL 一律用參數化查詢，不得用字串拼接」：第 42 行以 format! 拼接 WHERE 子句`

#### Scenario: 規則只進 Standards 軸

- **WHEN** 檢視生成的 speclink-review 技能檔關於 discovery 簡報的段落
- **THEN** 段落明定 Standards 簡報依嚴重度分組帶入專案規則原文、Correctness 簡報不帶入、嚴重度由規則所在清單決定且不得另行升降級

#### Scenario: 讀不到設定即停止

- **WHEN** speclink/config.yaml 的 review 區塊含未知子鍵，執行 speclink-review
- **THEN** 技能原樣回報 workflow-config show 的錯誤並停止，不呼叫 review scope、不派出 sub-agent、不寫工單

#### Scenario: 續輪不因規則新增 finding

- **WHEN** Round 1 後使用者在 review.critical 新增一條規則，接著跑 validation 輪
- **THEN** validation 只判定前輪 finding 是否已解與 remediation patch 的直接 regression，不依新規則回報新 finding

### Requirement: 首輪工單記錄套用的規則

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

技能於 discovery 輪執行 speclink review add-round 時，stdin SHALL 在 `**Patch**:` 之後、`**Scope**:` 之前寫入套用規則記錄：一行 `**Rules**: <baseline>`，baseline 取自開審前以 speclink workflow-config show --json 讀到的 review 物件（未設定時為 extend）；接著依 critical、warning、suggestion 的順序，每條專案規則一行 `**Rule**: <SEVERITY> <規則原文>`，SEVERITY 為該規則所在清單對應的 CRITICAL、WARNING 或 SUGGESTION，各清單內保持原順序。記錄的內容 SHALL 與 Standards 軸簡報實際帶入的規則相同，SHALL NOT 在記錄前重新讀取設定。規則原文含換行時，技能 SHALL 以單一空白取代每個換行連同其前後空白，寫成一行。validation 輪 SHALL NOT 寫入套用規則記錄。

這些行不隨解析後的 locale 改變：`**Rules**:`、`**Rule**:`、extend、replace 與嚴重度 token 維持英文，規則原文照設定中的文字寫入、不翻譯（tw、ja、en 與未設定皆同）。本記錄與「審查呈現標明套用的規則」的呈現開頭行是兩件事：開頭行照舊只出現在呈現中、不寫入工單。

#### Scenario: 有專案規則時的首輪記錄

- **WHEN** review 為 baseline 未設定、critical 一條「所有 SQL 一律用參數化查詢，不得用字串拼接」、suggestion 一條「偏好以提早返回取代深層巢狀」，執行 speclink-review 的 discovery
- **THEN** add-round 的 stdin 在 Patch 行之後依序含 `**Rules**: extend`、`**Rule**: CRITICAL 所有 SQL 一律用參數化查詢，不得用字串拼接`、`**Rule**: SUGGESTION 偏好以提早返回取代深層巢狀`，之後才是 Scope 行

#### Scenario: 沒有 review 區塊時只記模式

- **WHEN** speclink/config.yaml 沒有 review 區塊，執行 speclink-review 的 discovery
- **THEN** add-round 的 stdin 含一行 `**Rules**: extend`，不含任何 `**Rule**:` 行

#### Scenario: 複驗輪不寫記錄

- **WHEN** 工單已有 discovery Round 1，修正後執行 speclink-review 的 validation 輪
- **THEN** add-round 的 stdin 不含 `**Rules**:` 與 `**Rule**:` 行

#### Scenario: 多行規則寫成一行

- **WHEN** review.warning 的一條規則在 config 中以 YAML 區塊純量寫成兩行「新增的公開 API」與「要有對應的測試」，執行 discovery
- **THEN** add-round 的 stdin 含一行 `**Rule**: WARNING 新增的公開 API 要有對應的測試`

#### Scenario: 技能檔載明記錄規則

- **WHEN** 檢視生成的 speclink-review 技能檔關於記錄輪次的步驟
- **THEN** 步驟明定 discovery 輪在 Patch 與 Scope 之間寫入 `**Rules**:` 與每條專案規則的 `**Rule**:` 行、validation 輪不寫，且三種工具（claude、codex、copilot）生成的技能檔都含這段
