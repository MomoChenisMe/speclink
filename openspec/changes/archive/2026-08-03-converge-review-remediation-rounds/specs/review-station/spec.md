## MODIFIED Requirements

### Requirement: 審查工單的建立與追加
<!-- BEFORE: 每輪只要求 Scope 與 findings，沒有 phase 或 frozen patch identity。 -->

系統 SHALL 提供 speclink review add-round <change> --stdin：自 stdin 讀入一輪審查內容，於 `openspec/changes/<change>/review.md` 追加 `## Round N` 區段（工單不存在時建立並自 Round 1 起算）。每輪內容 SHALL 含 `**Scope**:` repo-root 相對路徑清單與零或多行分級 findings（severity ∈ CRITICAL／WARNING／SUGGESTION）。

結構化新輪次 SHALL 同時含 `**Phase**: discovery|validation` 與 `**Patch**: sha256:<64 lowercase hex>`；兩欄只出現其一、phase token 無效或 hash 格式無效時 SHALL 非零拒絕且工單零寫入。工單首個結構化 round SHALL 為 discovery；已有結構化 round 後追加者 SHALL 為 validation。既有兩欄皆缺席的 legacy round SHALL 保持可建立、追加與解析；legacy ticket 後 SHALL 能追加 validation round。

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

### Requirement: 審查工單的讀取
<!-- BEFORE: JSON round 只輸出 index、scope、findings，無 frozen patch identity。 -->

系統 SHALL 提供 speclink review show <change> [--json]。人眼路徑 SHALL 將工單原文印至 stdout，`--no-color` 下不含 ANSI。`--json` payload SHALL 含 change:string、rounds:array、lastRound:object；rounds 每項 SHALL 含 index:number、phase:string|null、patchHash:string|null、scope:string[]、findings:array；findings 每項 SHALL 維持 severity:string、path:string、text:string。lastRound SHALL 與 rounds 末項同形。

Local fs、remote CLI、typed client 與 server response SHALL 輸出相同 camelCase 欄位與 null 語意。phase／patchHash 是刻意的 additive shape change；既有欄位名稱、型別與 findings 順序 SHALL 維持。

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
