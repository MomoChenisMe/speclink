## MODIFIED Requirements

### Requirement: 生成模式的輸出與報告
<!-- BEFORE: 重生後內文逐位元相同的頁視為未動、不寫檔；sources 只寫 capability 名；摘要無「只換時戳」列 -->

生成模式 SHALL 依 manual-pages 契約寫頁，每頁 frontmatter 的 generated SHALL 為生成當下帶時區偏移量的 RFC 3339 時戳（秒級，例 2026-09-05T23:31:00+08:00），技能檔 SHALL 給 agent 一條取得該時戳的建議指令；已有手冊時預設 SHALL 只重生過期頁並為未入冊能力新增頁，使用者明示要求時方全量重生。過期判定 SHALL 與 manual-pages 契約「過期判定基準」同基準，含 sources 項的 Requirement 錨定：帶錨定的項只看該 Requirement 段落的 @trace updated，錨定找不到即過期。新頁與整頁重寫的頁，其 sources 每項 SHALL 依取材範圍寫出：該頁只取材該 capability 的部分 Requirement 時，每條寫一項帶錨定的 `capability#Requirement 名`（以雙引號包住）；取材該 capability 全部 Requirement 時寫不帶錨定的名稱。被判過期的頁重生後（含重新推導的 sources）除 generated 行外與磁碟內容逐位元相同時，SHALL 只把 generated 換成本次時戳、其餘位元不動，並在摘要的「只換時戳」列計入該頁；內文或 sources 不同時整頁重寫並計入「重生」——錨定找不到而過期的頁 SHALL NOT 走只換時戳。index.md 與 about.md SHALL 在任一頁新增、重生或只換時戳時隨之重生。技能檔 SHALL 把 sources 錨定的語意集中寫成一段，其他提及 sources 的規則引用該段。結束時 SHALL 於對話輸出摘要：新增、重生、只換時戳、未動的頁數；可能過期的頁清單；未入冊能力清單；about 頁記錄的矛盾數。無過期頁且無未入冊能力時 SHALL 零檔案寫入並如實回報。摘要末尾 SHALL 建議以一般提交收尾手冊異動——僅建議、SHALL NOT 代跑。

#### Scenario: 首次全量生成

- **WHEN** 工作區無 openspec/manual/ 而執行生成模式
- **THEN** 產出含 index.md 與 about.md 的完整手冊，每頁 generated 為 RFC 3339 時戳，只取材部分 Requirement 的頁其 sources 帶錨定，摘要列出新增頁數且重生、只換時戳與未動為零

#### Scenario: 二次只重生過期頁

- **WHEN** 手冊已存在且兩頁過期、一個能力未入冊
- **THEN** 僅該兩頁被重寫（generated 換成本次時戳）、新增一頁，其餘頁逐位元不變，摘要列出可能過期的頁與新入冊能力

#### Scenario: 過期但內文不變只換時戳

- **WHEN** 手冊已存在，一頁 sources 為 `[desktop-app]`、generated 為 2026-09-05，規格 desktop-app 有 2026-09-10T17:15:24+08:00 的 @trace updated，但該頁重生後內文與磁碟相同
- **THEN** 該頁只有 generated 行改為本次時戳，sources 仍為 `[desktop-app]`，摘要「只換時戳：1 頁」列出該頁，「重生」不計入；index.md 與 about.md 隨之重生

#### Scenario: 錨定失效的頁不走只換時戳

- **WHEN** 手冊已存在，一頁 sources 為 `["desktop-app#舊段名"]` 而正典已把該 Requirement 改名
- **THEN** 該頁整頁重寫、sources 改為現行標題並計入「重生」，不計入「只換時戳」

#### Scenario: 錨定頁不受他段封存影響

- **WHEN** 手冊已存在，一頁 sources 為 `["desktop-app#桌面上的品質關卡"]`、generated 為 2026-09-05，規格 desktop-app 只有「看板與任務」段帶 2026-09-10T17:15:24+08:00 的 @trace updated
- **THEN** 技能的過期報告不列該頁，也不重讀該規格的 Requirement 內文

#### Scenario: 無異動時零寫入

- **WHEN** 手冊已存在且無任何過期頁或未入冊能力
- **THEN** 無檔案被寫入，摘要明示手冊已是最新
