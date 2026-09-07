## MODIFIED Requirements

### Requirement: 生成模式的讀取策略

生成模式 SHALL 只以正式規格為內容來源：先以 speclink list --specs --json 取得全部 capability，再以 speclink show 讀各 capability 的 Purpose，將其分流為使用者面向與引擎內部；Purpose 為空或為 TBD 佔位時 SHALL 改以該 capability 的 Requirement 標題判斷。旅程骨幹 SHALL 依序優先取自劇本型規格（驗收劇本）、路由交棒型規格（技能交棒表）、使用者文件型規格；三者皆無時 SHALL 按功能域從使用者面向的能力規格重建旅程，並在 about 頁載明屬重建。技能 SHALL NOT 以 README、docs 目錄或程式碼作為手冊內容來源。已有手冊時，每個 capability 的 Purpose 與其全部 @trace updated 時戳仍 SHALL 讀取（分流與過期判定所需；時戳得為 RFC 3339 或純日期，依 manual-pages 契約「過期判定基準」分段比較），Requirement 內文 SHALL 只讀過期頁與未入冊能力所涉的規格。

#### Scenario: 有劇本型規格的專案

- **WHEN** 規格中存在驗收劇本與交棒表型的 capability
- **THEN** 手冊的旅程章節順序與站別取自該等規格，about 頁載明旅程「轉寫自」哪些規格

#### Scenario: 無劇本型規格且 Purpose 為 TBD 的專案

- **WHEN** 規格皆無劇本型內容且多數 Purpose 為 TBD 佔位
- **THEN** 技能以 Requirement 標題分流、按功能域重建旅程，about 頁載明屬重建並列出規格內部的新舊矛盾

#### Scenario: 不讀規格以外的來源

- **WHEN** 工作區同時存在 README 與 docs 目錄
- **THEN** 生成的任何頁的內容與出處只引用 openspec/specs/ 的 capability，不引用 README 或 docs

#### Scenario: 同日先封存後生成的頁不列為過期

- **WHEN** 手冊已存在，一頁 generated 為 2026-09-05T23:31:00+08:00，其 sources 規格最新 @trace updated 為 2026-09-05T23:17:28+08:00
- **THEN** 技能的過期報告不列該頁，也不重讀該規格的 Requirement 內文

### Requirement: 生成模式的輸出與報告

生成模式 SHALL 依 manual-pages 契約寫頁，每頁 frontmatter 的 generated SHALL 為生成當下帶時區偏移量的 RFC 3339 時戳（秒級，例 2026-09-05T23:31:00+08:00），技能檔 SHALL 給 agent 一條取得該時戳的建議指令；已有手冊時預設 SHALL 只重生過期頁並為未入冊能力新增頁，使用者明示要求時方全量重生。結束時 SHALL 於對話輸出摘要：新增、重生、未動的頁數；可能過期的頁清單；未入冊能力清單；about 頁記錄的矛盾數。無過期頁且無未入冊能力時 SHALL 零檔案寫入並如實回報。摘要末尾 SHALL 建議以一般提交收尾手冊異動——僅建議、SHALL NOT 代跑。

#### Scenario: 首次全量生成

- **WHEN** 工作區無 openspec/manual/ 而執行生成模式
- **THEN** 產出含 index.md 與 about.md 的完整手冊，每頁 generated 為 RFC 3339 時戳，摘要列出新增頁數且重生與未動為零

#### Scenario: 二次只重生過期頁

- **WHEN** 手冊已存在且兩頁過期、一個能力未入冊
- **THEN** 僅該兩頁被重寫（generated 換成本次時戳）、新增一頁，其餘頁逐位元不變，摘要列出可能過期的頁與新入冊能力

#### Scenario: 無異動時零寫入

- **WHEN** 手冊已存在且無任何過期頁或未入冊能力
- **THEN** 無檔案被寫入，摘要明示手冊已是最新
