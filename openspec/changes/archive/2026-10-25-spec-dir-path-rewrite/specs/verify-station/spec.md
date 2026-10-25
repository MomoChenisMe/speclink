## MODIFIED Requirements

### Requirement: 驗證工單的建立與追加

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

系統 SHALL 提供 `speclink verify add-round <change> --stdin`:自 stdin 讀入一輪驗證內容,於 `speclink/changes/<change>/verify.md` 追加 `## Round N` 區段(工單不存在時建立)。每輪內容 SHALL 含 `**Scope**:` 檔案清單與零或多行分級 findings(CRITICAL/WARNING/SUGGESTION),骨架與審查工單同構、append-only。寫碼任務未全完成時 SHALL 拒絕(引擎守門,採 manual-task-marker 的寫碼任務全完成預測子)——工單語意限定為成品驗證(代碼成品),盤點輪不落工單;僅餘 `[M]` 手動測試任務未勾時 SHALL 放行。

新技能產生的 structured round SHALL 於 Scope 前同時包含 `**Phase**: discovery|validation` 與 `**Patch**: sha256:<hex>`。Phase 與 Patch 必須成對且格式合法;structured Round 1 只能是 discovery,後續 structured round 只能是 validation。只含既有 Scope/findings 的 legacy stdin SHALL 維持可讀,該輪 phase/patchHash 解析為 null。任何格式或輪次序列錯誤 SHALL 非零拒絕且工單零寫入。

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

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

系統 SHALL 提供 `speclink verify stamp <change> [--accept]`,守門與審查站同一條:寫碼任務全數完成(manual-task-marker 預測子)＋工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免必修條件。通過時 SHALL 於同一原子寫入內:將 `verified_at`/`verified_by`/`verified_with`/`verified_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`verified_scope` 寫入 `.meta.yaml` 並刪除 `verify.md`,不得出現「章已寫而工單仍在」的中間狀態;canonical mutation 成功後 SHALL 依「驗證 frozen scope 與續輪 snapshot」清理 verify snapshots。工單刪除後其文字於 fs 模式僅存於 git 歷史;remote 模式的 store 不保留已刪文件內容,蓋章後工單文字不可回讀。

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
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除;fs 模式下 SUGGESTION 紀錄留在工單的 git 歷史,remote 模式下工單文字不保留

#### Scenario: 乾淨蓋章

- **WHEN** 寫碼任務全數完成且末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,`.meta.yaml` 含五個 verified 欄位且 `verify.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 8 個任務,其中 7 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,驗證工單 Round 1 的 findings 為空
- **WHEN** `verify stamp` 成功
- **THEN** `.meta.yaml` 內 `verified_tasks_total` 為 8(全任務總數)

### Requirement: 放棄驗證

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

系統 SHALL 提供 `speclink verify discard <change>`：刪除驗證工單、不寫任何 metadata，並依「驗證 frozen scope 與續輪 snapshot」清理 verify snapshots；無工單時非零 exit code。

#### Scenario: 放棄既有工單

- **WHEN** 對有驗證工單的 change 執行 `verify discard`
- **THEN** exit code 0，`verify.md` 不存在，`.meta.yaml` 不變

### Requirement: CLI 清單輸出的驗證欄位釘住

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink list --json` 的輸出 SHALL 不因 metadata 含任何 verified 欄位而改變形狀，與審查欄位的釘住規則同構。

#### Scenario: 帶驗證章的 change 不外洩新欄位

- **WHEN** 某 change 的 `.meta.yaml` 含全套 verified 欄位時執行 `speclink list --json`
- **THEN** 該 change 的 JSON 項目與無 verified 欄位的 change 具有相同的欄位集合

### Requirement: 驗證 finding 路徑落在工單範圍內

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink verify add-round <change> --stdin` SHALL 在寫入之前檢查本輪每一條 finding 的路徑，規則與審查工單相同。這項檢查排在既有的 change 存在檢查、寫碼任務守門與輪次序列檢查之前：同一份輸入同時違反多項時，先回報路徑違規；格式錯誤照舊回報同一則錯誤。規則如下：

- 路徑在最後一個冒號之後只剩行號寫法（例如 `src/a.rs:42`、`src/a.rs:42-50`、`src/a.rs:42-`、`src/a.rs:L42`），或以單獨的冒號結尾時 SHALL 拒絕。
- 路徑（`\` 正規化為 `/` 後）不在「工單既有各輪 `**Scope**:` 的聯集，加上本輪 `**Scope**:`」之內時 SHALL 拒絕。路徑位於規格目錄（預設 `speclink/`）或 `.speclink/` 之下時 SHALL 免除這一項——驗證站常把 Requirement 層級的 finding 錨定在 delta 規格檔上，這類路徑不進驗證 patch，複驗的範圍解析會略過它們。
- 與工單上一輪某條 finding 的 severity、路徑與描述完全相同的行（描述結尾的 `(accepted)` 標記不計），視為沿用的未解項，SHALL NOT 受上述兩項檢查。

違規時 SHALL 以非零 exit code 結束、stderr 逐一列出違規的路徑與原因，`verify.md` SHALL 零寫入、位元級不變。合法內容的成功行為與 stdout SHALL 維持既有輸出不變。remote 模式下 CLI SHALL 以同一套規則先讀取工單再檢查，違規時於送出前非零結束、stderr 帶違規路徑，server 端工單零寫入；離線、認證失效與 revision 衝突沿用既有 remote 動詞行為。

#### Scenario: 帶行號的路徑被拒

- **WHEN** 對任務全數完成、無工單的 change 寫入 discovery 輪，Scope 為 `src/a.rs`，finding 路徑寫成 `src/a.rs:7-9`
- **THEN** exit code 非零，stderr 點名 `src/a.rs:7-9` 帶有行號，`verify.md` 不建立

#### Scenario: 範圍外的程式碼路徑被拒

- **WHEN** 工單各輪與本輪 Scope 都不含 `src/caller.rs`，新 finding 的路徑為該檔
- **THEN** exit code 非零，stderr 點名該路徑不在工單範圍內，`verify.md` 位元級不變

#### Scenario: 錨定在 delta 規格檔的 finding 可接受

- **WHEN** 本輪 Scope 只含程式碼檔，新 finding 寫成 `- [CRITICAL] speclink/changes/demo/specs/auth/spec.md — Completeness: Requirement not found: 登入逾時`
- **THEN** exit code 0，該 finding 照原文寫入

#### Scenario: 早輪 Scope 內的路徑可接受

- **WHEN** Round 1 的 Scope 含 `src/b.rs`，Round 2 寫入一條路徑為 `src/b.rs` 的新 finding
- **THEN** exit code 0，`verify.md` 新增 `## Round 2`

#### Scenario: 沿用的已接受項不受檢查

- **WHEN** Round 1 有一條帶 `(accepted)` 的 finding，其路徑在檢查上線前已寫成帶行號，Round 2 原文沿用
- **THEN** exit code 0，該行照原文寫入 Round 2

### Requirement: 蓋章記錄驗證輪數

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink verify stamp <change>` 守門通過時，系統 SHALL 在寫入其他章欄位的同一次原子寫入中，把 `verified_rounds` 寫入該 change 的 `.meta.yaml`：值為原有 `verified_rounds`（缺席視為 0）加上本次蓋章的驗證工單回合數，檔內 SHALL 只有一行 `verified_rounds`。帶 `--accept` 的蓋章 SHALL 同樣記錄。守門失敗時 SHALL NOT 改動 `verified_rounds`；放棄驗證（只刪工單、未蓋章）SHALL NOT 改動 `verified_rounds`。原有 `verified_rounds` 不是非負整數時，metadata 視為無法解析，蓋章 SHALL 以非零 exit code 拒絕且零寫入。

#### Scenario: 首次驗證蓋章記錄回合數

- **WHEN** change 的驗證工單有 2 個回合、寫碼任務全數完成、末輪零必修 findings，執行 `speclink verify stamp <change>`
- **THEN** exit code 為 0，`.meta.yaml` 含 `verified_rounds: 2` 與 `verified_at` 等章欄位，`verify.md` 已刪除

#### Scenario: 驗證章失效後重新蓋章累加輪數

- **WHEN** change 的 `.meta.yaml` 已有 `verified_rounds: 2`，章失效後重開的驗證工單有 3 個回合，再次執行 `speclink verify stamp <change>` 且守門通過
- **THEN** exit code 為 0，`.meta.yaml` 只有一行 `verified_rounds`，值為 5

#### Scenario: 驗證守門失敗不改輪數

- **WHEN** 驗證工單末輪有一條未解的 CRITICAL finding，執行 `speclink verify stamp <change>` 且未帶 `--accept`
- **THEN** exit code 非零，`.meta.yaml` 的 `verified_rounds` 與其他欄位皆不變，`verify.md` 仍存在

#### Scenario: 原有驗證輪數不是整數時拒絕

- **WHEN** change 的 `.meta.yaml` 含 `verified_rounds: -1`，執行 `speclink verify stamp <change>`
- **THEN** exit code 非零，stderr 說明該 change 的 metadata 無法解析，`.meta.yaml` 與 `verify.md` 皆不變
