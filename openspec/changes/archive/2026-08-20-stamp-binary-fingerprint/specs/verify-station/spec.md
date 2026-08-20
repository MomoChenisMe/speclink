## MODIFIED Requirements

### Requirement: 驗證指紋錨與失效判定

<!-- BEFORE: 指紋規則只定義文字（CRLF→LF 後 SHA-256）的位元級同構；非 UTF-8 檔經共用實作使蓋章失敗 -->

蓋章時系統 SHALL 以驗證工單各輪 Scope 聯集記錄 `{ path, hash }` 至 `verified_scope`,路徑正規化與指紋分流規則(可以 UTF-8 讀取的檔為行尾 CRLF→LF 正規化後的 SHA-256;無法以 UTF-8 讀取的檔為原始位元組的 SHA-256)SHALL 與審查站位元級同構(共用同一實作)。失效判定同構:當前全任務總數不再等於蓋章時的 `verified_tasks_total`、或任一寫碼任務未完成、或任一 scope 檔內容不符(含缺檔)→ stale;補勾或取消勾 `[M]` 任務 SHALL NOT 影響判定。判定結果 SHALL NOT 以 CLI 專屬查詢欄位曝光;封存守門 SHALL 消費此判定(見 change-lifecycle 的封存的章失效守門);desktop 協定曝光維持既有紅線、不在本 change 接線。

#### Scenario: 蓋章後修改範圍檔

- **WHEN** 驗證蓋章成功後修改任一 scope 檔內容
- **THEN** 失效判定為 stale

#### Scenario: 非 UTF-8 scope 檔可蓋章

- **WHEN** 驗證工單 Scope 含一個存在的非 UTF-8 檔,其餘守門條件皆過,執行 `verify stamp`
- **THEN** 蓋章成功,`verified_scope` 記該檔原始位元組的 SHA-256

#### Scenario: 蓋章後補勾手動任務不失效

- **WHEN** 寫碼任務全完成、一個 `[M]` 任務未勾時驗證蓋章成功,之後將該 `[M]` 任務勾選
- **THEN** 失效判定仍為 fresh

#### Scenario: 行尾差異不觸發失效

- **WHEN** scope 檔內容僅行尾由 LF 變為 CRLF
- **THEN** 失效判定仍為 fresh
