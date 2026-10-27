## MODIFIED Requirements

### Requirement: 審查工單的建立與追加

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

系統 SHALL 提供 speclink review add-round <change> --stdin：自 stdin 讀入一輪審查內容，於 `speclink/changes/<change>/review.md` 追加 `## Round N` 區段（工單不存在時建立並自 Round 1 起算）。每輪內容 SHALL 含 `**Scope**:` repo-root 相對路徑清單與零或多行分級 findings（severity ∈ CRITICAL／WARNING／SUGGESTION）。

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

### Requirement: 蓋章守門與蓋章效果

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

系統 SHALL 提供 `speclink review stamp <change> [--accept]`。守門條件:(1) change 的寫碼任務全數完成(非 `[M]` 任務全數勾選;手動測試任務不計,見 manual-task-marker 的寫碼任務全完成預測子);(2) 工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免條件 (2) 的必修部分。守門通過時系統 SHALL 於同一原子寫入內:將 `reviewed_at`/`reviewed_by`/`reviewed_with`/`reviewed_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`reviewed_scope`(指紋清單)寫入該 change 的 `.meta.yaml`,並刪除 `review.md`。不得出現「章已寫入而工單仍存在」的中間狀態。工單刪除後其文字於 fs 模式僅存於 git 歷史;remote 模式的 store 不保留已刪文件內容,蓋章後工單文字不可回讀。

#### Scenario: 寫碼任務未全完成即拒絕

- **WHEN** change 的寫碼任務為 4/5 完成時執行 `review stamp`
- **THEN** exit code 非零,stderr 說明寫碼任務未全數完成並列計數(4/5),metadata 與工單皆不變

#### Scenario: 僅餘手動任務可蓋章

- **WHEN** change 的寫碼任務 4/4 全勾、一個 `[M]` 任務未勾,工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 且執行 `review stamp`(無 `--accept`)
- **THEN** exit code 非零,stderr 點名未解必修數並提示 `--accept` 或先修正重審

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 寫碼任務全數完成且工單末輪僅含 SUGGESTION 級 findings 時執行 `review stamp`(無 `--accept`)
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除;fs 模式下 SUGGESTION 紀錄留在工單的 git 歷史,remote 模式下工單文字不保留

#### Scenario: 帶保留蓋章

- **WHEN** 工單末輪含必修 findings 且執行 `review stamp --accept`
- **THEN** exit code 0,章寫入且工單刪除

#### Scenario: 乾淨蓋章

- **WHEN** 任務 5/5 完成且工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,`.meta.yaml` 含五個 reviewed 欄位且 `review.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 5 個任務,其中 4 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,工單 Round 2 的 findings 為空
- **WHEN** `review stamp` 成功
- **THEN** `.meta.yaml` 內 `reviewed_tasks_total` 為 5(全任務總數,含未勾的 `[M]` 任務)

### Requirement: 放棄審查

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

系統 SHALL 提供 `speclink review discard <change>`：刪除工單、不寫任何 metadata。

#### Scenario: 放棄既有工單

- **WHEN** 對有工單的 change 執行 `review discard`
- **THEN** exit code 0，`review.md` 不存在，`.meta.yaml` 不變

#### Scenario: 無工單可放棄

- **WHEN** 對無工單的 change 執行 `review discard`
- **THEN** exit code 非零，stderr 說明無工單

### Requirement: CLI 清單輸出的相容性釘住

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink list --json` 的輸出 SHALL 不因 metadata 含任何 reviewed 欄位而改變形狀：含全套 reviewed 欄位的 change 與不含者 SHALL 序列化出同形（位元級同構的欄位集合）。

#### Scenario: 帶章的 change 不外洩新欄位

- **WHEN** 某 change 的 `.meta.yaml` 含全套 reviewed 欄位時執行 `speclink list --json`
- **THEN** 該 change 的 JSON 項目與無 reviewed 欄位的 change 具有相同的欄位集合

### Requirement: finding 路徑落在工單範圍內

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink review add-round <change> --stdin` SHALL 在寫入之前檢查本輪每一條 finding 的路徑。這項檢查排在既有的 change 存在檢查與輪次序列檢查之前：同一份輸入同時違反多項時，先回報路徑違規；格式錯誤照舊回報同一則錯誤。規則如下：

- 路徑在最後一個冒號之後只剩行號寫法（例如 `src/a.rs:42`、`src/a.rs:42-50`、`src/a.rs:42-`、`src/a.rs:L42`），或以單獨的冒號結尾時 SHALL 拒絕。
- 路徑（`\` 正規化為 `/` 後）不在「工單既有各輪 `**Scope**:` 的聯集，加上本輪 `**Scope**:`」之內時 SHALL 拒絕。此聯集與蓋章指紋範圍採同一定義。路徑位於規格目錄（預設 `speclink/`）或 `.speclink/` 之下時 SHALL 免除這一項——這類路徑本來就不進驗證 patch，複驗的範圍解析會略過它們。
- 與工單上一輪某條 finding 的 severity、路徑與描述完全相同的行（描述結尾的 `(accepted)` 標記不計），視為沿用的未解項，SHALL NOT 受上述兩項檢查。

違規時 SHALL 以非零 exit code 結束、stderr 逐一列出違規的路徑與原因，`review.md` SHALL 零寫入、位元級不變。合法內容的成功行為與 stdout SHALL 維持既有輸出不變。remote 模式下 CLI SHALL 以同一套規則先讀取工單再檢查，違規時於送出前非零結束、stderr 帶違規路徑，server 端工單零寫入；離線、認證失效與 revision 衝突沿用既有 remote 動詞行為。

#### Scenario: 帶行號的路徑被拒

- **WHEN** 對已有 Round 1（Scope 含 `src/a.rs`）的工單追加 validation 輪，新 finding 寫成 `- [WARNING] src/a.rs:42 — Correctness: 邊界未處理`
- **THEN** exit code 非零，stderr 點名 `src/a.rs:42` 帶有行號，`review.md` 位元級不變

#### Scenario: 範圍外的路徑被拒

- **WHEN** 本輪 Scope 為 `src/a.rs`、既有各輪 Scope 也不含 `src/caller.rs`，新 finding 的路徑為 `src/caller.rs`
- **THEN** exit code 非零，stderr 點名 `src/caller.rs` 不在工單範圍內，`review.md` 位元級不變

#### Scenario: 早輪 Scope 內的路徑可接受

- **WHEN** Round 1 的 Scope 含 `src/b.rs`，Round 2 驗證 patch 只改 `src/a.rs`，Round 2 寫入一條路徑為 `src/b.rs` 的新 finding
- **THEN** exit code 0，`review.md` 新增 `## Round 2`

#### Scenario: 規格目錄下的路徑免除範圍檢查

- **WHEN** Scope 只含 `src/a.rs`，新 finding 的路徑為 `speclink/changes/demo/design.md`
- **THEN** exit code 0，該 finding 照原文寫入

#### Scenario: 沿用的未解項不受檢查

- **WHEN** 舊工單 Round 1 已有 `- [WARNING] src/a.rs:12 — Correctness: x`，Round 2 原文沿用該行
- **THEN** exit code 0，該行照原文寫入 Round 2，不因帶行號被拒

#### Scenario: 反斜線路徑正規化後比對

- **WHEN** Scope 含 `src/a.rs`，新 finding 的路徑寫成 `src\a.rs`
- **THEN** 路徑正規化為 `src/a.rs` 後落在範圍內，exit code 0

#### Scenario: remote 模式同樣拒絕

- **WHEN** remote 模式下對工單追加一條範圍外路徑的 finding
- **THEN** CLI 非零結束、stderr 帶該路徑，不送出寫入，server 端工單零寫入

### Requirement: 蓋章記錄審查輪數

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink review stamp <change>` 守門通過時，系統 SHALL 在寫入其他章欄位的同一次原子寫入中，把 `reviewed_rounds` 寫入該 change 的 `.meta.yaml`：值為原有 `reviewed_rounds`（缺席視為 0）加上本次蓋章的審查工單回合數，檔內 SHALL 只有一行 `reviewed_rounds`。帶 `--accept` 的蓋章 SHALL 同樣記錄。守門失敗時 SHALL NOT 改動 `reviewed_rounds`；放棄審查（只刪工單、未蓋章）SHALL NOT 改動 `reviewed_rounds`。原有 `reviewed_rounds` 不是非負整數時，metadata 視為無法解析，蓋章 SHALL 以非零 exit code 拒絕且零寫入。

#### Scenario: 首次蓋章記錄回合數

- **WHEN** change 的審查工單有 3 個回合、寫碼任務全數完成、末輪零必修 findings，執行 `speclink review stamp <change>`
- **THEN** exit code 為 0，`.meta.yaml` 含 `reviewed_rounds: 3` 與 `reviewed_at` 等章欄位，`review.md` 已刪除

#### Scenario: 章失效後重新蓋章累加輪數

- **WHEN** change 的 `.meta.yaml` 已有 `reviewed_rounds: 3`，章失效後重開的審查工單有 2 個回合，再次執行 `speclink review stamp <change>` 且守門通過
- **THEN** exit code 為 0，`.meta.yaml` 只有一行 `reviewed_rounds`，值為 5

#### Scenario: 守門失敗不改輪數

- **WHEN** 審查工單末輪有一條未解的 WARNING finding，執行 `speclink review stamp <change>` 且未帶 `--accept`
- **THEN** exit code 非零，`.meta.yaml` 的 `reviewed_rounds` 與其他欄位皆不變，`review.md` 仍存在

#### Scenario: 原有輪數不是整數時拒絕

- **WHEN** change 的 `.meta.yaml` 含 `reviewed_rounds: many`，執行 `speclink review stamp <change>`
- **THEN** exit code 非零，stderr 說明該 change 的 metadata 無法解析，`.meta.yaml` 與 `review.md` 皆不變
