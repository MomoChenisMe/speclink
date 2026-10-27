## ADDED Requirements

### Requirement: createEngine 的建構期 actor 注入

createEngine SHALL 接受選填 actor 欄位（"Name <email>" 格式字串），兩種儲存形式皆於建構期收下並綁定於該 engine 實例；dispatch 的操作者身分解析 SHALL 為：建構期 actor 有給值（trim 後非空）一律優先，fs 形式未給時回退 workspace 的 git identity（現行為），宿主 Store 形式未給時維持無章（現行為）。trim 後為空字串的 actor SHALL 視同未給。actor SHALL NOT 能經 dispatch 的 argv 或其他呼叫期參數傳入或覆寫——一個實例一個身分，多身分宿主以多個 engine 實例表達。

#### Scenario: fs 形式明給 actor 優先於 git identity

- **WHEN** 在設有 git user.name 與 user.email 的 fixture 專案，以 createEngine({ store: { type: 'fs', root }, actor: 'Alice <alice@example.com>' }) 建構並 dispatch(['new', 'change', 'demo'])
- **THEN** demo 的 metadata created_by 為 Alice <alice@example.com>，而非 git identity

#### Scenario: fs 形式未給 actor 維持現行回退

- **WHEN** 同一 fixture 專案以不帶 actor 的 createEngine fs 形式建構並 dispatch(['new', 'change', 'demo2'])
- **THEN** demo2 的 created_by 與在該專案執行 CLI speclink new change 的蓋章逐位元一致（git identity 回退不變）

#### Scenario: 宿主 Store 形式帶 actor 落章

- **WHEN** 以 JS Store 物件＋actor: 'Bob <bob@example.com>' 建構引擎並 dispatch(['new', 'change', 'demo3'])
- **THEN** 宿主 Store 收到的 metadata 寫入含 created_by: Bob <bob@example.com>；同引擎後續的 review／verify 蓋章動詞亦以同值落 _by 欄位

#### Scenario: 宿主 Store 形式未給 actor 維持無章

- **WHEN** 以 JS Store 物件建構引擎（不帶 actor）並 dispatch(['new', 'change', 'demo4'])
- **THEN** 寫入的 metadata 不含 created_by（與現行無章行為一致）

#### Scenario: 呼叫期無從覆寫身分

- **WHEN** 檢視 dispatch 的輸入契約並嘗試以任意 argv 影響蓋章身分
- **THEN** dispatch 不存在 actor 參數，蓋章內容只隨建構期 actor（或其回退）改變

### Requirement: dispatch 的蓋章動詞

dispatch SHALL 認得 `review add-round`、`review stamp`、`verify add-round`、`verify stamp` 四個動詞，argv 沿用 CLI 詞彙（`--accept`、`--agent <tool>`、`--stdin`）。add-round 的輪次內容 SHALL 由 dispatch 的 stdin 參數帶入；stamp 的 scope 指紋與 missing 清單 argv 承載不了，SHALL 由 stdin 參數以 JSON 帶入（`{ "scope": [{ "path", "hash" }], "missing": [] }`，兩欄缺席讀作空清單）。蓋章落下的 `reviewed_by`／`verified_by` SHALL 為該 engine 建構期綁定的 actor（未給時依儲存形式回退，與 created_by 同一條解析）。引擎既有的守門（任務未全完成、末輪未解必修 findings、scope ∪ missing 與工單聯集的分割）SHALL 原封傳遞，拒絕時以語義化例外呈現。蓋章會刪除工單文件並改寫 change 的 metadata 原文，宿主 Store 因此 SHALL 可提供三個選填的前置方法：`deleteArtifact(change, artifact)`、`readChangeMeta(name)`、`writeChangeMeta(name, content)`；缺任何一個時蓋章 SHALL 在動手前以語義化訊息拒絕（工單與 metadata 皆不動），其餘動詞不受影響。

#### Scenario: review 蓋章鏈落 actor

- **WHEN** 以 actor: 'Rev <rev@example.com>' 建構引擎，對一個任務全完成的 change 依序 dispatch(['review', 'add-round', 'beta', '--stdin'], { stdin: 只含 SUGGESTION 的輪次內容 }) 與 dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: JSON.stringify({ missing: [輪次 Scope 的檔路徑] }) })
- **THEN** add-round 解析為 { change: 'beta', round: 1 }；stamp 解析為 { change: 'beta' }，且 change 的 metadata 落下 reviewed_by: Rev <rev@example.com>

#### Scenario: verify 蓋章鏈落 actor

- **WHEN** 同引擎（同一 actor）對同一 change 走 verify add-round 與 verify stamp
- **THEN** metadata 落下 verified_by 為同一個 actor 值

#### Scenario: 蓋章守門的拒絕原封傳遞

- **WHEN** 對末輪帶 CRITICAL finding 的工單 dispatch(['review', 'stamp', ...]) 而不帶 --accept
- **THEN** dispatch 以 Error 拒絕，message 為引擎的語義化守門訊息，未落任何 reviewed_* 欄位

#### Scenario: 宿主 Store 未實作 deleteArtifact 時蓋章失敗

- **WHEN** 以未實作 deleteArtifact 的 JS Store 建構引擎，走完 add-round 後 dispatch(['review', 'stamp', ...])
- **THEN** 以 Error 拒絕，訊息指名 deleteArtifact 是蓋章所需的方法；同一個 store 的 list／status／new 動詞不受影響

#### Scenario: 未支援的子動詞明確拒絕

- **WHEN** dispatch(['review', 'show', 'beta'])
- **THEN** 以 code 為 invalid_argv 的 Error 拒絕，訊息指出 review 只支援 add-round 與 stamp
