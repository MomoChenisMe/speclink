## ADDED Requirements

### Requirement: 變更清單的驗證狀態欄位

desktop 協定的 change 清單項 SHALL 增列 `verifyStatus`（字串 enum：`"none"`／`"inVerify"`／`"verified"`／`"verifiedStale"`），章存在時附 `verifiedAt`／`verifiedBy`（字串）。判定規則與審查狀態同構：工單存在 → `inVerify`；章在且雙錨相符 → `verified`；章在而任一錨不符 → `verifiedStale`；皆無 → `none`。CLI `speclink list --json` SHALL NOT 包含上述欄位。

#### Scenario: 驗證四態判定

- **WHEN** desktop 載入變更清單
- **THEN** 每個 change 項含 `verifyStatus`，其值依「工單存在／章存在／雙錨相符」為四態之一，與 `reviewStatus` 各自獨立判定

##### Example: 兩站狀態獨立

- **GIVEN** 某 change 有審查章（雙錨相符）且存在未結驗證工單
- **WHEN** desktop 取得該 change 的清單項
- **THEN** `reviewStatus` 為 `"reviewed"` 且 `verifyStatus` 為 `"inVerify"`

### Requirement: 已封存清單的驗證結局欄位

desktop 協定的已封存清單項 SHALL 增列 `verifyStatus`（`"none"`／`"verified"`／`"verifiedNotPassed"`）：封存目錄含驗證章 → `verified`；含驗證工單而無章 → `verifiedNotPassed`；皆無 → `none`。已封存側不重算凍結度。

#### Scenario: 化石驗證工單的封存項

- **WHEN** desktop 載入已封存清單且某項封存目錄含 verify.md 而 metadata 無 verified 欄位
- **THEN** 該項 `verifyStatus` 為 `"verifiedNotPassed"`
