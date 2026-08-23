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
