## MODIFIED Requirements

### Requirement: 變更清單的審查狀態欄位

<!-- BEFORE: 凍結度重算的內容指紋一律讀 workspace 主 checkout 的檔案現值，未定義 worktree 映射存在時的讀取根，worktree 中蓋章的 change 於合併前恆被誤判 reviewedStale。 -->

desktop 協定的 change 清單項 SHALL 增列 `reviewStatus` 欄位（字串 enum：`"none"`／`"inReview"`／`"reviewed"`／`"reviewedStale"`），且於章存在時附 `reviewedAt`（字串）與 `reviewedBy`（字串）。狀態判定：工單存在 → `inReview`；章存在且任務錨與內容指紋錨皆相符 → `reviewed`；章存在而任一錨不符 → `reviewedStale`；皆無 → `none`。凍結度重算 SHALL 於有工作樹的 client 端執行。內容指紋錨的檔案現值 SHALL 逐 change 解析讀取根：該 change 有 worktree 映射時讀該 worktree 副本的檔案，無映射時讀主 checkout——與同一清單項的任務錨同源，SHALL NOT 出現任務錨取自 worktree、指紋錨取自主 checkout 的劈半。scope 檔於解析後的根下不存在時維持「缺檔即不符 → Stale」語意。CLI `speclink list --json` SHALL NOT 包含上述任何欄位（相容性釘住歸 review-station 規格）。

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

#### Scenario: worktree 中蓋章的凍結度以 worktree 現值判定

- **WHEN** 某 change 有 worktree 映射，`reviewed_scope` 各檔於 worktree 副本內的現值雜湊與記錄相符，主 checkout 的同名檔仍為蓋章前舊內容
- **THEN** 清單項 `reviewStatus` 為 `"reviewed"`

##### Example: worktree 內蓋章後又改檔才轉 stale

- **GIVEN** change fix-auth 有 worktree 映射，蓋章時 `reviewed_scope` 記錄 src/auth.rs 的雜湊；主 checkout 的 src/auth.rs 為未實作的舊內容
- **WHEN** worktree 副本的 src/auth.rs 與蓋章時一致 → 清單項判定；其後於 worktree 內再修改該檔 → 再次判定
- **THEN** 前者 `reviewStatus` 為 `"reviewed"`，後者為 `"reviewedStale"`
