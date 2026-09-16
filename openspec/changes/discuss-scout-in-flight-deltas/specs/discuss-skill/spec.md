## MODIFIED Requirements

### Requirement: 事實與決策分診及逐節點查證

技能檔 SHALL 規定每個決策節點解決前先分診：環境（程式碼、檔案系統、工具）查得到的事實 SHALL 由代理人沿樹逐節點自行查證，SHALL NOT 拿去問使用者、SHALL NOT 憑印象作答；僅真正的決策（使用者裁定事項）交由使用者。開場偵察 SHALL 為漏斗式：先跑 speclink list --specs --json（候選 ≤5、讀 Purpose ≤3、主題直接動到的 capability 才讀全文、零命中靜默略過）；命中 capability 名後 SHALL 比對 openspec/changes/*/specs/<capability>/spec.md 是否存在，存在者列為「進行中 delta 命中」並記下路徑中的變更名，只讀該 delta 的 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 區段標記、至多 3 份、不讀全文；正式規格零命中時 SHALL 跳過 delta 比對；本機沒有 openspec/changes/ 目錄（remote 模式）時 SHALL 靜默跳過。之後以命中的 capability 名與正典詞彙轉譯搜尋詞後再掃原始碼（至多讀 5 檔）；主題已含具體檔名或符號時，程式碼軌 SHALL 直接開跑不等正式規格。偵察三段順序 SHALL 維持「正式規格 → 舊討論查核 → 程式碼」，進行中 delta 規格 SHALL 屬規格段的一部分、SHALL NOT 另立一段。偵察用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 沿樹逐節點進行（確定會走到的分支才深讀）。

#### Scenario: 渲染產物含事實決策分診規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔
- **THEN** 技能檔 SHALL 規定：查得到的事實由代理人自行查證後附為 Evidence，不得提問使用者、不得憑印象作答；決策類問題才交由使用者裁定

#### Scenario: 開場 scout 維持淺掃

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察段落
- **THEN** 偵察 SHALL 維持時間盒（正式規格讀取至多 3 份 Purpose、進行中 delta 規格至多 3 份且只讀 Requirement 標題與區段標記、原始碼至多 5 檔），用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 規定於決策樹逐節點進行

#### Scenario: 正式規格先行的漏斗偵察

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察順序規定
- **THEN** 技能檔 SHALL 規定先查正式規格（speclink list --specs --json）、以命中詞彙轉譯搜尋詞後再掃原始碼；主題含具體檔名或符號時程式碼軌直接開跑；正式規格零命中時靜默退回關鍵字掃描

#### Scenario: 規格段納入進行中 delta 規格

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 Canon pass 段落
- **THEN** 技能檔 SHALL 規定命中 capability 名後比對 openspec/changes/*/specs/<capability>/spec.md，存在者列為進行中 delta 命中並記變更名，只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED／RENAMED 標記、至多 3 份；正式規格零命中或本機無 openspec/changes/ 目錄時靜默跳過；三段順序描述仍為「正式規格 → 舊討論查核 → 程式碼」且 SHALL NOT 出現獨立的「進行中變更」偵察段

##### Example: 命中 client-protocol 且有進行中 delta

- **GIVEN** 正式規格 client-protocol 命中，且 openspec/changes/add-change-plan-remote/specs/client-protocol/spec.md 存在
- **WHEN** 代理人依技能檔執行 Canon pass
- **THEN** 代理人記下「進行中 delta 命中：add-change-plan-remote: client-protocol」與該 delta 的 Requirement 標題清單，不讀該 delta 全文

### Requirement: 正式規格接地與三分對照

技能檔 SHALL 規定：偵察命中相關正式規格時，假設清單 SHALL 對使用者需求逐項做三分對照——正式規格已涵蓋（附 spec 證據）、與正式規格衝突（指出衝突內容並附證據）、正式規格沒講（新地盤，順帶檢查 capability 命名鄰近既有規格）。「正式規格已涵蓋」與「與正式規格衝突」兩類在該 capability 有進行中 delta 命中時，該條假設的 Evidence SHALL 附加標記「（進行中變更 <name> 將改動）」並列出該 delta 動到的 Requirement 名；技能檔 SHALL 明文不得以此擋下討論方向。對照表 SHALL NOT 為進行中 delta 另增一類。紀律 SHALL 明文為「使用者需求是目標，正式規格是證據、不是裁決」：偏離正式規格 SHALL 為允許的結論方向，但 SHALL 記入討論記錄成為有意識的決定。

#### Scenario: 需求與正式規格衝突時列為假設

- **WHEN** 偵察發現使用者需求與正式規格既有承諾衝突
- **THEN** 技能檔 SHALL 規定該衝突以假設形式呈現（附 spec 證據），由使用者裁定改正式規格或改需求，SHALL NOT 逕行擋下或否決討論方向

#### Scenario: 正式規格零命中時流程照舊

- **WHEN** 主題與任何正式規格 capability 無關（工具鏈、依賴選型類主題）
- **THEN** 技能檔 SHALL 規定不提及規格掃描，後續流程與純程式碼偵察相同

#### Scenario: 進行中 delta 命中時附加將改動標記

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的假設清單對照規則
- **THEN** 對照表 SHALL 仍為四列（正式規格已涵蓋、與正式規格衝突、正式規格沒講、舊討論已定案）；前兩列 SHALL 規定 delta 命中時附加「（進行中變更 <name> 將改動）」標記與該 delta 動到的 Requirement 名，並含不得以此擋下討論方向的字句

##### Example: 已涵蓋且將被改動

- **GIVEN** 使用者需求對應正式規格 client-protocol 的既有承諾，且 add-change-plan-remote 帶 client-protocol 的 delta、其 MODIFIED 區段含 Requirement「討論資訊 payload」
- **WHEN** 代理人列假設清單
- **THEN** 該條假設寫為「Covered by canon（進行中變更 add-change-plan-remote 將改動：討論資訊 payload）」，Evidence 同時指向正式規格與該 delta 路徑

## ADDED Requirements

### Requirement: Context 相關變更句列出進行中 delta

技能檔 SHALL 規定 Context 段既有的「related changes/specs」句同時列出偵察命中的進行中 delta，格式為 `<name>: <capability>`、多筆以逗號分隔；零命中時該句只記正式規格與變更名，SHALL NOT 加任何空標記。`Prior discussions:` 行、Source doc 慣例與 Context／Rounds／Conclusion 骨架 SHALL 不變；SHALL NOT 新增獨立的機械標記行。既有討論記錄 SHALL NOT 需要遷移。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: Context 記錄進行中 delta 命中

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 Context 撰寫規定
- **THEN** 技能檔 SHALL 規定相關變更句以 `<name>: <capability>` 形式列出 delta 命中，SHALL NOT 規定新增獨立標記行，且 `Prior discussions:` 行的規定 SHALL 逐字不變

#### Scenario: 零 delta 命中時 Context 照舊

- **WHEN** 偵察的正式規格命中皆無進行中 delta
- **THEN** 技能檔 SHALL 規定 Context 相關變更句與今天相同，不出現空的 delta 標記
