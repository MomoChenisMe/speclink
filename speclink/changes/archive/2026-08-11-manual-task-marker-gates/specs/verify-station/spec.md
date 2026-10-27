## MODIFIED Requirements

### Requirement: 驗證工單的建立與追加

系統 SHALL 提供 `speclink verify add-round <change> --stdin`:自 stdin 讀入一輪驗證內容,於 `openspec/changes/<change>/verify.md` 追加 `## Round N` 區段(工單不存在時建立)。每輪內容 SHALL 含 `**Scope**:` 檔案清單與零或多行分級 findings(CRITICAL/WARNING/SUGGESTION),骨架與審查工單同構、append-only。寫碼任務未全完成時 SHALL 拒絕(引擎守門,採 manual-task-marker 的寫碼任務全完成預測子)——工單語意限定為成品驗證(代碼成品),盤點輪不落工單;僅餘 `[M]` 手動測試任務未勾時 SHALL 放行。

新技能產生的 structured round SHALL 於 Scope 前同時包含 `**Phase**: discovery|validation` 與 `**Patch**: sha256:<hex>`。Phase 與 Patch 必須成對且格式合法;structured Round 1 只能是 discovery,後續 structured round 只能是 validation。只含既有 Scope/findings 的 legacy stdin SHALL 維持可讀,該輪 phase/patchHash 解析為 null。任何格式或輪次序列錯誤 SHALL 非零拒絕且工單零寫入。

<!-- REMOVED-SCENARIO: 任務未全完成即拒絕落工單 -->

#### Scenario: 寫碼任務未完成即拒絕落工單

- **WHEN** 對寫碼任務 4/5 的 change 執行 `verify add-round`
- **THEN** exit code 非零,stderr 說明驗證工單要求寫碼任務全數完成,無檔案建立

#### Scenario: 僅餘手動任務可落工單

- **WHEN** 對寫碼任務 4/4 全勾、一個 `[M]` 任務未勾且無工單的 change 執行 `verify add-round` 且 stdin 合法
- **THEN** exit code 0,`verify.md` 建立且含 `## Round 1`

#### Scenario: 首輪建立工單

- **WHEN** 對任務全數完成且無工單的 change 執行 `verify add-round` 且 stdin 合法
- **THEN** exit code 0,`verify.md` 建立且含 `## Round 1`

#### Scenario: 追加輪次不改寫既有輪

- **WHEN** 對已有 Round 1 的驗證工單再次執行 `verify add-round`
- **THEN** exit code 0,新增 `## Round 2` 且 Round 1 位元級不變

#### Scenario: 追加 structured validation

- **WHEN** 對已有 structured discovery Round 1 的工單追加 Phase=validation 且 Patch 合法的 Round 2
- **THEN** exit code 0,新增 `## Round 2`,stdout 確認 phase/patch 且 Round 1 位元級不變

#### Scenario: 第二個 discovery 被拒絕

- **WHEN** structured Round 1 已是 discovery,又追加 Phase=discovery
- **THEN** exit code 非零、stderr 說明後續輪只能是 validation,工單位元級不變

#### Scenario: phase 與 patch 必須成對

- **WHEN** stdin 只有 Phase 沒有 Patch
- **THEN** exit code 非零、stderr 說明兩欄必須同時存在,工單零寫入

#### Scenario: legacy round 保持相容

- **WHEN** stdin 只含既有 Scope 與 findings,不含 Phase/Patch
- **THEN** add-round 維持成功行為,該輪 phase 與 patchHash 解析為 null

#### Scenario: 內容缺少 Scope

- **WHEN** stdin 內容不含 `**Scope**:` 行
- **THEN** exit code 非零,stderr 說明格式要求,工單不變

### Requirement: 驗證蓋章守門與蓋章效果

系統 SHALL 提供 `speclink verify stamp <change> [--accept]`,守門與審查站同一條:寫碼任務全數完成(manual-task-marker 預測子)＋工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免必修條件。通過時 SHALL 於同一原子寫入內:將 `verified_at`/`verified_by`/`verified_with`/`verified_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`verified_scope` 寫入 `.openspec.yaml` 並刪除 `verify.md`,不得出現「章已寫而工單仍在」的中間狀態;canonical mutation 成功後 SHALL 依「驗證 frozen scope 與續輪 snapshot」清理 verify snapshots。

#### Scenario: 寫碼任務未完成即拒絕蓋章

- **WHEN** 對寫碼任務 4/5 的 change 執行 `verify stamp`
- **THEN** exit code 非零,stderr 說明寫碼任務未全數完成,metadata 與工單皆不變

#### Scenario: 僅餘手動任務可蓋章

- **WHEN** 寫碼任務全勾、一個 `[M]` 任務未勾且驗證工單末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 驗證工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 時執行 `verify stamp`
- **THEN** exit code 非零,stderr 點名未解必修數並提示 `--accept` 或先修正重驗

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 寫碼任務全數完成且驗證工單末輪僅含 SUGGESTION 級 findings 時執行 `verify stamp`(無 `--accept`)
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除,SUGGESTION 紀錄留在工單的 git 歷史

#### Scenario: 乾淨蓋章

- **WHEN** 寫碼任務全數完成且末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,`.openspec.yaml` 含五個 verified 欄位且 `verify.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 8 個任務,其中 7 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,驗證工單 Round 1 的 findings 為空
- **WHEN** `verify stamp` 成功
- **THEN** `.openspec.yaml` 內 `verified_tasks_total` 為 8(全任務總數)

### Requirement: 驗證指紋錨與失效判定

蓋章時系統 SHALL 以驗證工單各輪 Scope 聯集記錄 `{ path, hash }` 至 `verified_scope`,路徑正規化與行尾 CRLF→LF 後 SHA-256 規則 SHALL 與審查站位元級同構(共用同一實作)。失效判定同構:當前全任務總數不再等於蓋章時的 `verified_tasks_total`、或任一寫碼任務未完成、或任一 scope 檔內容不符(含缺檔)→ stale;補勾或取消勾 `[M]` 任務 SHALL NOT 影響判定。判定結果 SHALL NOT 以 CLI 專屬查詢欄位曝光;封存守門 SHALL 消費此判定(見 change-lifecycle 的封存的章失效守門);desktop 協定曝光維持既有紅線、不在本 change 接線。

#### Scenario: 蓋章後修改範圍檔

- **WHEN** 驗證蓋章成功後修改任一 scope 檔內容
- **THEN** 失效判定為 stale

#### Scenario: 蓋章後補勾手動任務不失效

- **WHEN** 寫碼任務全完成、一個 `[M]` 任務未勾時驗證蓋章成功,之後將該 `[M]` 任務勾選
- **THEN** 失效判定仍為 fresh

#### Scenario: 行尾差異不觸發失效

- **WHEN** scope 檔內容僅行尾由 LF 變為 CRLF
- **THEN** 失效判定仍為 fresh
