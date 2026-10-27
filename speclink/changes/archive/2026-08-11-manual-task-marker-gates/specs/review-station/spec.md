## MODIFIED Requirements

### Requirement: 蓋章守門與蓋章效果

系統 SHALL 提供 `speclink review stamp <change> [--accept]`。守門條件:(1) change 的寫碼任務全數完成(非 `[M]` 任務全數勾選;手動測試任務不計,見 manual-task-marker 的寫碼任務全完成預測子);(2) 工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免條件 (2) 的必修部分。守門通過時系統 SHALL 於同一原子寫入內:將 `reviewed_at`/`reviewed_by`/`reviewed_with`/`reviewed_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`reviewed_scope`(指紋清單)寫入該 change 的 `.openspec.yaml`,並刪除 `review.md`。不得出現「章已寫入而工單仍存在」的中間狀態。

<!-- REMOVED-SCENARIO: 任務未全完成即拒絕 -->

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
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除,SUGGESTION 紀錄留在工單的 git 歷史

#### Scenario: 帶保留蓋章

- **WHEN** 工單末輪含必修 findings 且執行 `review stamp --accept`
- **THEN** exit code 0,章寫入且工單刪除

#### Scenario: 乾淨蓋章

- **WHEN** 任務 5/5 完成且工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,`.openspec.yaml` 含五個 reviewed 欄位且 `review.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 5 個任務,其中 4 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,工單 Round 2 的 findings 為空
- **WHEN** `review stamp` 成功
- **THEN** `.openspec.yaml` 內 `reviewed_tasks_total` 為 5(全任務總數,含未勾的 `[M]` 任務)

### Requirement: 內容指紋錨與失效判定

蓋章時系統 SHALL 以工單各輪 Scope 的聯集為範圍,逐檔記錄 `{ path, hash }` 至 `reviewed_scope`:path 為 repo-root 相對且以 `/` 分隔(Windows 路徑正規化後寫入);hash 為檔案內容經行尾 CRLF→LF 正規化後的 SHA-256。聯集中已不存在於工作樹的檔(修正過程刪除或改名)SHALL 排除於指紋之外、不入 `reviewed_scope`,蓋章不因死檔而失敗;聯集全數不存在時 SHALL 拒絕蓋章(exit code 非零並列出檔案);存在但無法以 UTF-8 讀取的檔 SHALL 仍使蓋章失敗。remote 模式下工作樹持有者 SHALL 於 stamp 請求明示宣告已不存在的檔(`missing` 清單),server SHALL 驗證「提交指紋的 path 集合 ∪ missing =工單聯集且兩者不相交」,分割不成立即拒;`missing` 缺席讀作空清單(既有嚴格集合相等)。失效判定 SHALL 為:當前全任務總數不再等於蓋章時的 `reviewed_tasks_total`、或任一寫碼任務未完成、或任一 scope 檔內容雜湊不符(含檔案已不存在)→ 該章判為過期(stale);全部相符 → 有效(fresh)。補勾或取消勾 `[M]` 任務 SHALL NOT 影響判定。判定結果 SHALL NOT 以 CLI 專屬查詢欄位曝光;封存守門 SHALL 消費此判定(見 change-lifecycle 的封存的章失效守門),其拒絕訊息得點名失效;desktop 協定曝光維持既有紅線、不在本 change 接線。

#### Scenario: 修正刪除早輪 scope 檔後仍可蓋章

- **WHEN** 工單 Round 1 的 Scope 含檔案 A 與 B,修正過程刪除 B 後寫碼任務全完成且末輪零 findings,執行 `review stamp`
- **THEN** 蓋章成功,`reviewed_scope` 僅含 A 的指紋,不含 B

#### Scenario: 聯集全數消失時拒絕蓋章

- **WHEN** 工單各輪 Scope 的所有檔案皆已不存在於工作樹,執行 `review stamp`
- **THEN** exit code 非零,stderr 列出已消失的檔案並指引還原或 `review discard`

#### Scenario: 蓋章後修改範圍檔

- **WHEN** 蓋章成功後修改任一 scope 檔的內容
- **THEN** 失效判定為 stale

#### Scenario: 蓋章後補勾手動任務不失效

- **WHEN** 寫碼任務全完成、一個 `[M]` 任務未勾時蓋章成功,之後將該 `[M]` 任務勾選
- **THEN** 失效判定仍為 fresh

#### Scenario: 蓋章後新增任務失效

- **WHEN** 蓋章成功後 tasks.md 新增一個任務(總數改變)
- **THEN** 失效判定為 stale

#### Scenario: 行尾差異不觸發失效

- **WHEN** scope 檔內容僅行尾由 LF 變為 CRLF
- **THEN** 失效判定仍為 fresh

##### Example: 指紋比對

- **GIVEN** `reviewed_scope` 含 `{ path: "crates/a/src/lib.rs", hash: H1 }` 且該檔現值雜湊為 H1
- **WHEN** 該檔追加一行後重新判定
- **THEN** 現值雜湊不為 H1,判定 stale
