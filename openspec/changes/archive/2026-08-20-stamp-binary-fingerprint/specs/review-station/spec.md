## MODIFIED Requirements

### Requirement: 內容指紋錨與失效判定

<!-- BEFORE: hash 只定義文字規則（CRLF→LF 正規化後 SHA-256），存在但無法以 UTF-8 讀取的 scope 檔一律使蓋章失敗 -->

蓋章時系統 SHALL 以工單各輪 Scope 的聯集為範圍,逐檔記錄 `{ path, hash }` 至 `reviewed_scope`:path 為 repo-root 相對且以 `/` 分隔(Windows 路徑正規化後寫入);hash 依內容分流——可以 UTF-8 讀取的檔為行尾 CRLF→LF 正規化後的 SHA-256(與既有章逐位元組相容),無法以 UTF-8 讀取的檔為原始位元組的 SHA-256(不做行尾正規化)。聯集中已不存在於工作樹的檔(修正過程刪除或改名)SHALL 排除於指紋之外、不入 `reviewed_scope`,蓋章不因死檔而失敗;聯集全數不存在時 SHALL 拒絕蓋章(exit code 非零並列出檔案);存在但讀取發生 I/O 錯誤的檔 SHALL 仍使蓋章失敗(非 UTF-8 內容不是讀取失敗,走位元組指紋)。remote 模式下工作樹持有者 SHALL 於 stamp 請求明示宣告已不存在的檔(`missing` 清單),server SHALL 驗證「提交指紋的 path 集合 ∪ missing =工單聯集且兩者不相交」,分割不成立即拒;`missing` 缺席讀作空清單(既有嚴格集合相等)。失效判定 SHALL 為:當前全任務總數不再等於蓋章時的 `reviewed_tasks_total`、或任一寫碼任務未完成、或任一 scope 檔內容雜湊不符(含檔案已不存在)→ 該章判為過期(stale);全部相符 → 有效(fresh)。補勾或取消勾 `[M]` 任務 SHALL NOT 影響判定。判定結果 SHALL NOT 以 CLI 專屬查詢欄位曝光;封存守門 SHALL 消費此判定(見 change-lifecycle 的封存的章失效守門),其拒絕訊息得點名失效;desktop 協定曝光維持既有紅線、不在本 change 接線。

#### Scenario: 修正刪除早輪 scope 檔後仍可蓋章

- **WHEN** 工單 Round 1 的 Scope 含檔案 A 與 B,修正過程刪除 B 後寫碼任務全完成且末輪零 findings,執行 `review stamp`
- **THEN** 蓋章成功,`reviewed_scope` 僅含 A 的指紋,不含 B

#### Scenario: 聯集全數消失時拒絕蓋章

- **WHEN** 工單各輪 Scope 的所有檔案皆已不存在於工作樹,執行 `review stamp`
- **THEN** exit code 非零,stderr 列出已消失的檔案並指引還原或 `review discard`

#### Scenario: 非 UTF-8 scope 檔可蓋章

- **WHEN** 工單 Scope 聯集含一個存在的非 UTF-8 檔,其餘守門條件皆過,執行 `review stamp`
- **THEN** 蓋章成功,`reviewed_scope` 記該檔原始位元組的 SHA-256

#### Scenario: 蓋章後修改範圍檔

- **WHEN** 蓋章成功後修改任一 scope 檔的內容
- **THEN** 失效判定為 stale

#### Scenario: 蓋章後 binary 內容變動失效

- **WHEN** 含非 UTF-8 scope 檔蓋章成功後,該檔的位元組內容變動
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

##### Example: 指紋分流表

| 檔案內容 | hash 規則 |
|----------|-----------|
| UTF-8 文字(LF) | CRLF→LF 正規化後 SHA-256 |
| UTF-8 文字(CRLF) | 與同內容 LF 版雜湊相同 |
| 非 UTF-8 位元組(如 PNG) | 原始位元組 SHA-256 |
