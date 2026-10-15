## MODIFIED Requirements

### Requirement: 審查工單的建立與追加

系統 SHALL 提供 speclink review add-round <change> --stdin：自 stdin 讀入一輪審查內容，於 `openspec/changes/<change>/review.md` 追加 `## Round N` 區段（工單不存在時建立並自 Round 1 起算）。每輪內容 SHALL 含 `**Scope**:` repo-root 相對路徑清單與零或多行分級 findings（severity ∈ CRITICAL／WARNING／SUGGESTION）。

結構化新輪次 SHALL 同時含 `**Phase**: discovery|validation` 與 `**Patch**: sha256:<64 lowercase hex>`；兩欄只出現其一、phase token 無效或 hash 格式無效時 SHALL 非零拒絕且工單零寫入。工單首個結構化 round SHALL 為 discovery；已有結構化 round 後追加者 SHALL 為 validation。既有兩欄皆缺席的 legacy round SHALL 保持可建立、追加與解析；legacy ticket 後 SHALL 能追加 validation round。

輪次內容 SHALL 接受選填的套用規則記錄：至多一行 `**Rules**: extend|replace`（token 區分大小寫），以及零或多行 `**Rule**: <severity> <規則原文>`（severity ∈ CRITICAL／WARNING／SUGGESTION，其後至少一個空白與非空的規則原文；原文去頭尾空白後保存）。`**Rule**:` 行 SHALL 只出現在同輪帶 `**Rules**:` 的輪；帶 `**Rules**:` 的輪 SHALL 為 `**Phase**: discovery` 的結構化輪。違反任一條時 SHALL 非零拒絕、stderr 點名問題且工單零寫入；remote 模式由 server 以同一文法檢查，拒絕時回 `invalid_argument`（HTTP 400），CLI 非零退出，離線與認證失效時的行為與既有 add-round 相同。這兩種行皆為選填：不帶它們的輪次（含 legacy round）SHALL 照舊可建立、追加與解析。審查站與驗證站共用同一文法。

工單 SHALL 為 append-only：既有輪次不因追加而改寫。工單檔 SHALL NOT 註冊進 workflow schema，speclink status／validate 輸出 SHALL 不因工單存在而改變。

#### Scenario: 首輪建立 structured discovery 工單

- **WHEN** 對無工單的 change 執行 review add-round，stdin 含 Phase=discovery、合法 Patch、Scope 與 findings
- **THEN** exit code 0，`review.md` 建立且含 `## Round 1`、phase／patch 原文與 stdout 確認訊息

#### Scenario: 追加 validation 不改寫既有輪

- **WHEN** 對已有 structured Round 1 的工單追加 Phase=validation 的合法 Round 2
- **THEN** exit code 0，`review.md` 新增 `## Round 2` 且 Round 1 位元級不變

#### Scenario: 第二個 discovery 被拒絕

- **WHEN** structured Round 1 已是 discovery，又追加 Phase=discovery
- **THEN** exit code 非零、stderr 說明後續輪只能是 validation，工單位元級不變

#### Scenario: phase 與 patch 必須成對

- **WHEN** stdin 只有 Phase 沒有 Patch
- **THEN** exit code 非零、stderr 說明兩欄必須同時存在，工單零寫入

#### Scenario: legacy round 保持相容

- **WHEN** stdin 只含既有 Scope 與 findings，不含 Phase／Patch
- **THEN** add-round 維持既有成功行為，該輪 phase 與 patchHash 解析為 null

#### Scenario: change 不存在

- **WHEN** 對不存在的 change 執行 review add-round
- **THEN** exit code 非零，stderr 說明找不到變更，無檔案建立

#### Scenario: 內容缺少 Scope

- **WHEN** stdin 不含 `**Scope**:` 行
- **THEN** exit code 非零，stderr 說明格式要求，工單不變

#### Scenario: 首輪記錄套用規則

- **WHEN** 對無工單的 change 執行 review add-round，stdin 依序含 Phase=discovery、合法 Patch、`**Rules**: extend`、`**Rule**: CRITICAL 所有 SQL 一律用參數化查詢，不得用字串拼接`、`**Rule**: WARNING 新增的公開 API 要有對應的測試`、Scope 與 findings
- **THEN** exit code 0，`review.md` 的 `## Round 1` 逐字含這三行規則記錄，review show --json 的 rounds[0].rules 為 `{"baseline":"extend","critical":["所有 SQL 一律用參數化查詢，不得用字串拼接"],"warning":["新增的公開 API 要有對應的測試"],"suggestion":[]}`

#### Scenario: 規則記錄格式錯誤被拒絕

- **WHEN** discovery 輪的 stdin 帶格式錯誤的規則記錄
- **THEN** exit code 非零，stderr 點名該行的問題，工單零寫入（已存在者位元級不變）

##### Example: 拒絕的規則記錄

| stdin 中的規則記錄 | stderr 說明 |
| ------------------ | ----------- |
| `**Rules**: Extend` | baseline 只接受 extend 或 replace |
| `**Rules**: extend` 出現兩行 | 一輪只能有一行 `**Rules**:` |
| 只有 `**Rule**: WARNING 要有測試`，沒有 `**Rules**:` | `**Rule**:` 需要同輪的 `**Rules**:` |
| `**Rules**: extend` 與 `**Rule**: NOTE 要有測試` | severity 只接受 CRITICAL、WARNING、SUGGESTION |
| `**Rules**: replace` 與 `**Rule**: CRITICAL` | `**Rule**:` 缺少規則原文 |

#### Scenario: 規則記錄只能出現在 discovery 輪

- **WHEN** 工單已有 structured discovery Round 1，追加的 Phase=validation 輪帶 `**Rules**: extend`；或 stdin 為不含 Phase／Patch 的 legacy 內容卻帶 `**Rules**: extend`
- **THEN** exit code 非零，stderr 說明規則記錄只能出現在 discovery 輪，工單位元級不變

### Requirement: 審查工單的讀取

系統 SHALL 提供 speclink review show <change> [--json]。人眼路徑 SHALL 將工單原文印至 stdout，`--no-color` 下不含 ANSI。`--json` payload SHALL 含 change:string、rounds:array、lastRound:object；rounds 每項 SHALL 含 index:number、phase:string|null、patchHash:string|null、rules:object|null、scope:string[]、findings:array；findings 每項 SHALL 維持 severity:string、path:string、text:string。lastRound SHALL 與 rounds 末項同形。

rules 非 null 時 SHALL 為 `{ baseline:string, critical:string[], warning:string[], suggestion:string[] }`：baseline 為該輪 `**Rules**:` 的 token（extend 或 replace），三個清單依嚴重度收錄該輪 `**Rule**:` 行的規則原文、保持出現順序，沒有對應行的清單為空陣列。該輪沒有 `**Rules**:` 行時（legacy round、驗證站技能寫的輪、複驗輪）rules SHALL 為 null。人眼路徑維持既有行為：原文照印，規則記錄行隨原文出現。

Local fs、remote CLI、typed client 與 server response SHALL 輸出相同 camelCase 欄位與 null 語意。phase／patchHash 與 rules 是刻意的 additive shape change；既有欄位名稱、型別與 findings 順序 SHALL 維持。server 回傳的輪次沒有 rules 欄位時（舊 server），remote CLI SHALL 視為 null；server 回傳的 baseline 不是 extend 或 replace 時，remote CLI SHALL 非零退出，stderr 說明這支 CLI 比 server 的工單格式舊，SHALL NOT 靜默略過。

#### Scenario: 讀取 structured 兩輪 JSON

- **WHEN** 工單含 discovery Round 1 與 validation Round 2，執行 review show <change> --json
- **THEN** exit code 0、stdout 為合法 JSON、rounds 長度為 2、lastRound.index 為 2、lastRound.phase 為 validation、lastRound.patchHash 為 `sha256:` digest

#### Scenario: legacy JSON 使用 null

- **WHEN** legacy 工單 round 不含 Phase／Patch，執行 review show --json
- **THEN** phase 與 patchHash 明確輸出 null，既有 index、scope、findings 內容不變

#### Scenario: local 與 remote payload 同構

- **WHEN** 同一 structured ticket 分別經 local fs 與 remote server 讀取
- **THEN** 兩份 rounds／lastRound 的欄位集合、camelCase 名稱、null 與值逐項相同

#### Scenario: 無工單

- **WHEN** 對無工單的 change 執行 review show
- **THEN** exit code 非零，stderr 說明該 change 無審查工單

#### Scenario: 讀取帶規則記錄的工單

- **WHEN** 工單的 discovery Round 1 帶 `**Rules**: replace` 與一行 `**Rule**: SUGGESTION 偏好以提早返回取代深層巢狀`，validation Round 2 不帶規則記錄，執行 review show <change> --json
- **THEN** rounds[0].rules 為 `{"baseline":"replace","critical":[],"warning":[],"suggestion":["偏好以提早返回取代深層巢狀"]}`，rounds[1].rules 與 lastRound.rules 為 null，其餘欄位與既有輸出相同

#### Scenario: 沒有規則記錄的工單輸出 null

- **WHEN** 工單只含不帶規則記錄的輪次（legacy 或 structured），分別經 review show --json 與 verify show --json 讀取
- **THEN** 每輪都有 rules 鍵且值為 null

#### Scenario: 舊 server 沒有 rules 欄位

- **WHEN** remote CLI 執行 review show --json，server 回傳的輪次不含 rules 欄位
- **THEN** exit code 0，CLI 輸出的每輪 rules 為 null

#### Scenario: server 回傳不認得的 baseline

- **WHEN** remote CLI 執行 review show --json，server 回傳某輪 rules.baseline 為 `strict`
- **THEN** exit code 非零，stderr 說明這支 CLI 比 server 的工單格式舊，stdout 不輸出 JSON
