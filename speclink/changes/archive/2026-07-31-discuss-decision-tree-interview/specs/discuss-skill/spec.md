## ADDED Requirements

### Requirement: interview 模式以決策樹遍歷提問

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 將 interview 模式的預設提問策略規定為決策樹遍歷：開場先攤開決策空間（根節點為「這題到底在決定什麼」，展開子決策與依賴邊），提問 SHALL 依依賴順序進行、一次一題，上游決策先解。停止條件 SHALL 維持使用者主導：one nudge maximum 與結論的 Deferred 欄位續留，技能檔 SHALL NOT 要求「所有分支解完才可收斂」。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含決策樹提問紀律

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 將 interview 提問策略描述為：開場攤開決策空間、依依賴順序一次一題、上游決策先解

#### Scenario: 停止條件維持使用者主導

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的收斂（Convergence）相關段落
- **THEN** one nudge maximum 與 Deferred 機制 SHALL 保留，技能檔 SHALL NOT 含「未走完所有分支即不得收斂」的規定，且 SHALL 規定使用者喊停時未走分支記入結論的 Deferred

### Requirement: interview 每題附建議答案與 Evidence

技能檔 SHALL 規定 interview 模式每個提問附代理人的建議答案，且建議 SHALL 附 Evidence（檔案路徑或查證結果）——assumptions 模式既有的 Evidence 慣例在 interview 模式同為硬規則，SHALL NOT 以軟性指示表述。

#### Scenario: 渲染產物將 Evidence 列為 interview 硬規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 interview 提問規則
- **THEN** 技能檔 SHALL 規定每題附建議答案並附 Evidence（檔案路徑或查證結果），使用者僅需同意或修正

### Requirement: 事實與決策分診及逐節點查證

技能檔 SHALL 規定每個決策節點解決前先分診：環境（程式碼、檔案系統、工具）查得到的事實 SHALL 由代理人沿樹逐節點自行查證，SHALL NOT 拿去問使用者、SHALL NOT 憑印象作答；僅真正的決策（使用者裁定事項）交由使用者。開場 codebase scout SHALL 維持淺掃時間盒且用途 SHALL 僅為模式選擇；深入查證 SHALL 沿樹逐節點進行（確定會走到的分支才深讀）。

#### Scenario: 渲染產物含事實決策分診規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔
- **THEN** 技能檔 SHALL 規定：查得到的事實由代理人自行查證後附為 Evidence，不得提問使用者、不得憑印象作答；決策類問題才交由使用者裁定

#### Scenario: 開場 scout 維持淺掃

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 codebase scout 段落
- **THEN** scout SHALL 維持原時間盒（數秒、至多讀 5 檔）且用途 SHALL 僅為模式選擇；深入查證 SHALL 規定於決策樹逐節點進行

### Requirement: 討論記錄的樹慣例與格式不變

技能檔 SHALL 規定記錄內容慣例：首輪 Position 攤開初始決策空間（得含 ASCII 樹），之後每輪聚焦解掉一個節點，中途發現的新分支記入該輪 Open。討論文件的骨架（Context／Rounds／Conclusion）、輪模板欄位（Focus／Position／Ruled out／Open）與 append-only 規則 SHALL 維持不變，既有討論記錄 SHALL NOT 需要遷移。

#### Scenario: 首輪攤樹且每輪一節點

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的記錄規則
- **THEN** 技能檔 SHALL 規定首輪 Position 含初始決策空間、後續每輪解一個節點、新分支記入該輪 Open

#### Scenario: 既有記錄格式沿用

- **WHEN** 以更新後的技能進行討論並經 speclink discuss 動詞寫入記錄
- **THEN** 產出的討論文件 SHALL 維持 Context／Rounds／Conclusion 骨架與 Focus／Position／Ruled out／Open 欄位，與既有記錄格式一致，無需任何遷移
