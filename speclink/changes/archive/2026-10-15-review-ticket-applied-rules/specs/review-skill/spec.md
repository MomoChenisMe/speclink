## ADDED Requirements

### Requirement: 首輪工單記錄套用的規則

技能於 discovery 輪執行 speclink review add-round 時，stdin SHALL 在 `**Patch**:` 之後、`**Scope**:` 之前寫入套用規則記錄：一行 `**Rules**: <baseline>`，baseline 取自開審前以 speclink workflow-config show --json 讀到的 review 物件（未設定時為 extend）；接著依 critical、warning、suggestion 的順序，每條專案規則一行 `**Rule**: <SEVERITY> <規則原文>`，SEVERITY 為該規則所在清單對應的 CRITICAL、WARNING 或 SUGGESTION，各清單內保持原順序。記錄的內容 SHALL 與 Standards 軸簡報實際帶入的規則相同，SHALL NOT 在記錄前重新讀取設定。規則原文含換行時，技能 SHALL 以單一空白取代每個換行連同其前後空白，寫成一行。validation 輪 SHALL NOT 寫入套用規則記錄。

這些行不隨解析後的 locale 改變：`**Rules**:`、`**Rule**:`、extend、replace 與嚴重度 token 維持英文，規則原文照設定中的文字寫入、不翻譯（tw、ja、en 與未設定皆同）。本記錄與「審查呈現標明套用的規則」的呈現開頭行是兩件事：開頭行照舊只出現在呈現中、不寫入工單。

#### Scenario: 有專案規則時的首輪記錄

- **WHEN** review 為 baseline 未設定、critical 一條「所有 SQL 一律用參數化查詢，不得用字串拼接」、suggestion 一條「偏好以提早返回取代深層巢狀」，執行 speclink-review 的 discovery
- **THEN** add-round 的 stdin 在 Patch 行之後依序含 `**Rules**: extend`、`**Rule**: CRITICAL 所有 SQL 一律用參數化查詢，不得用字串拼接`、`**Rule**: SUGGESTION 偏好以提早返回取代深層巢狀`，之後才是 Scope 行

#### Scenario: 沒有 review 區塊時只記模式

- **WHEN** openspec/config.yaml 沒有 review 區塊，執行 speclink-review 的 discovery
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
