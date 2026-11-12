## ADDED Requirements

### Requirement: createHost 的技能清單

createHost SHALL 接受選填的 `skills` 欄位（字串陣列：這個宿主提供給連進來的用戶端的技能名稱）。省略時 SHALL 提供引擎的全部技能。陣列含不認得的技能名稱時 createHost SHALL 同步拋出錯誤並列出這些名稱，不產生宿主套件；空陣列時 createHost SHALL 同樣同步拋出錯誤（要提供全部技能就省略這個欄位）。host.handleRequest SHALL 依這個清單回答技能清單的請求，並在握手宣告技能版號。這個欄位 SHALL NOT 提供改寫技能文字的途徑。它 SHALL NOT 影響 skills.render 與 host.dispatch。

#### Scenario: 只提供兩支技能

- **WHEN** 以 `skills: ['discuss', 'propose']` 建立 host，經 handleRequest 送出 claude 的技能清單請求
- **THEN** 回應的 skills 只有 discuss 與 propose

#### Scenario: 省略時提供全部

- **WHEN** 不帶 skills 建立 host，送出 claude 的技能清單請求
- **THEN** 回應含引擎給 Claude 的每一支技能（worktree 的兩支依範圍的流程設定）

#### Scenario: 不認得的名稱

- **WHEN** 以 `skills: ['discuss', 'no-such-skill']` 呼叫 createHost
- **THEN** 同步拋出錯誤，訊息列出 no-such-skill

#### Scenario: 空陣列

- **WHEN** 以 `skills: []` 呼叫 createHost
- **THEN** 同步拋出錯誤，訊息說明清單是空的

#### Scenario: skills.render 不受影響

- **WHEN** 以 `skills: ['discuss']` 建立 host 之後呼叫 skills.render('archive', …)
- **THEN** 照常回傳 archive 的渲染結果
