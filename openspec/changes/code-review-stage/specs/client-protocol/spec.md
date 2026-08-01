## ADDED Requirements

### Requirement: 變更清單的審查狀態欄位

desktop 協定的 change 清單項 SHALL 增列 `reviewStatus` 欄位（字串 enum：`"none"`／`"inReview"`／`"reviewed"`／`"reviewedStale"`），且於章存在時附 `reviewedAt`（字串）與 `reviewedBy`（字串）。狀態判定：工單存在 → `inReview`；章存在且任務錨與內容指紋錨皆相符 → `reviewed`；章存在而任一錨不符 → `reviewedStale`；皆無 → `none`。凍結度重算 SHALL 於有工作樹的 client 端執行。CLI `speclink list --json` SHALL NOT 包含上述任何欄位（相容性釘住歸 review-station 規格）。

#### Scenario: 四態判定

- **WHEN** desktop 載入變更清單
- **THEN** 每個 change 項含 `reviewStatus`，其值依「工單存在／章存在／雙錨相符」的組合為四態之一

##### Example: 章在但指紋不符

- **GIVEN** 某 change 的 metadata 含全套 reviewed 欄位，且 `reviewed_scope` 中一個檔的現值雜湊與記錄不符
- **WHEN** desktop 取得該 change 的清單項
- **THEN** `reviewStatus` 為 `"reviewedStale"`，`reviewedAt`／`reviewedBy` 仍存在

#### Scenario: 審查中的清單項

- **WHEN** 某 change 有 review.md 而無章
- **THEN** 清單項 `reviewStatus` 為 `"inReview"`，無 `reviewedAt`／`reviewedBy`

### Requirement: 已封存清單的審查結局欄位

desktop 協定的已封存清單項 SHALL 增列 `reviewStatus`（字串 enum：`"none"`／`"reviewed"`／`"reviewedNotPassed"`）：封存目錄含章 → `reviewed`；含工單而無章 → `reviewedNotPassed`；皆無 → `none`。已封存側 SHALL NOT 做凍結度重算（封存即定格）。

#### Scenario: 化石工單的封存項

- **WHEN** desktop 載入已封存清單且某項的封存目錄含 review.md 而 metadata 無 reviewed 欄位
- **THEN** 該項 `reviewStatus` 為 `"reviewedNotPassed"`
