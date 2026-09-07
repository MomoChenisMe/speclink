## MODIFIED Requirements

### Requirement: frontmatter 六欄

每頁 SHALL 以 YAML frontmatter 開頭並含下列欄位：`title`（字串，必填，頁的人類標題）、`section`（字串，必填，側欄分區名）、`order`（整數，必填，全手冊唯一的全域序號，慣例以 10 為間隔）、`keywords`（字串陣列，選填，供搜尋）、`sources`（capability 名稱陣列，必填，本頁取材的正典規格；首頁與來源頁得為空陣列）、`generated`（必填，本頁最近一次生成的時戳，格式為帶時區偏移量的 RFC 3339，秒級，例 `2026-09-05T23:31:00+08:00`）。讀取端 SHALL 同時接受純日期 `YYYY-MM-DD` 的 `generated`（本契約放寬前生成的頁），既有頁 SHALL NOT 因此需要回改。frontmatter SHALL NOT 含其他欄位。頁的排序 SHALL 僅由 `order` 決定；分區順序 SHALL 由分區內最小 `order` 決定。

#### Scenario: 合規的 frontmatter

- **WHEN** 檢視一頁的 frontmatter
- **THEN** 六欄齊備（`keywords` 得缺席），`order` 為整數且與其他頁不重複，`generated` 為合法的 RFC 3339 時戳或純日期，`sources` 每項對應 `openspec/specs/` 下既有的 capability 目錄名

##### Example: 一頁的 frontmatter

- **GIVEN** 頁 `first-login.md`
- **WHEN** 讀其 frontmatter
- **THEN** 內容為 `title: 第一次登入`、`section: 開始使用`、`order: 20`、`keywords: [登入, github, 審核]`、`sources: [github-oauth, user-pending-blocked-pages]`、`generated: 2026-09-05T23:31:00+08:00`

#### Scenario: 純日期的舊頁照舊可讀

- **WHEN** 一頁的 `generated` 為 `2026-09-02`（放寬前生成）
- **THEN** 讀取端把它當作該日曆日的生成紀錄列於側欄並參與過期判定，不視為格式錯誤

#### Scenario: 排序由 order 推導

- **WHEN** 三頁的 `order` 分別為 30、10、20，`section` 依序為「文件協作」「開始使用」「開始使用」
- **THEN** 讀取端的閱讀序為 10、20、30，分區序為「開始使用」在「文件協作」之前；上一頁／下一頁即該序列中的相鄰頁

### Requirement: 過期判定基準

一頁 SHALL 視為過期，若其 `sources` 中任一 capability 的正典規格內存在任一 `@trace updated` 時戳「在該頁 `generated` 之後」。「在之後」SHALL 依兩邊的格式分段判定：兩邊都是帶時區偏移量的 RFC 3339 時戳時，換算為同一瞬間後規格時戳嚴格晚於頁時戳才算，同一秒 SHALL NOT 算；任一邊只有純日期時，規格的日曆日不早於（晚於或同日）頁的日曆日即算，帶時間的一方取其時戳自身偏移量下的日曆日——生成當天的封存不得漏判。既非 RFC 3339 也非純日期的時戳 SHALL 視為缺席，不參與判定。一個 capability SHALL 視為未入冊，若它被生成端分流為使用者面向、且不出現在任何頁的 `sources`。生成端與讀取端 SHALL 採同一基準；`sources` 為空的頁 SHALL NOT 判為過期。

#### Scenario: 過期與未入冊的判定

- **WHEN** 頁 A（`generated: 2026-09-01`，`sources: [x]`）而規格 x 的最新 `@trace updated` 為 2026-09-05；頁 B（`generated: 2026-09-01`，`sources: [y]`）而規格 y 最新為 2026-08-20；使用者面向 capability z 不在任何頁的 sources
- **THEN** A 判為過期、B 不過期、z 列為未入冊

#### Scenario: 同日先封存後生成不判過期

- **WHEN** 頁 `generated` 為 `2026-09-05T23:31:00+08:00`，`sources: [x]`，規格 x 的 `@trace updated` 最新為 `2026-09-05T23:17:28+08:00`
- **THEN** 該頁不判為過期；若規格 x 之後再封存得到 `2026-09-05T23:40:00+08:00`，該頁判為過期

#### Scenario: 任一邊純日期時退回同日規則

- **WHEN** 頁 `generated` 為 `2026-09-05T23:31:00+08:00` 而規格 x 最新 `@trace updated` 為純日期 `2026-09-05`；或頁 `generated` 為純日期 `2026-09-05` 而規格 x 最新為 `2026-09-05T23:17:28+08:00`
- **THEN** 兩種情況都判為過期（同日也算）；頁 `generated` 為 `2026-09-06T00:10:00+08:00` 對規格純日期 `2026-09-05` 則不過期

##### Example: 判定表

| 頁 generated | sources 最新 @trace updated | 結果 |
| ------------ | -------------------------- | ---- |
| 2026-09-01   | 2026-09-05                 | 過期 |
| 2026-09-01   | 2026-08-20                 | 未過期 |
| 2026-09-01   | 2026-09-01                 | 過期（同日） |
| 2026-09-01   | （sources 為空）           | 未過期 |
| 2026-09-05T23:31:00+08:00 | 2026-09-05T23:17:28+08:00 | 未過期（同日但更早） |
| 2026-09-05T23:31:00+08:00 | 2026-09-05T23:40:00+08:00 | 過期 |
| 2026-09-05T23:31:00+08:00 | 2026-09-05T23:31:00+08:00 | 未過期（同秒） |
| 2026-09-05T23:31:00+08:00 | 2026-09-05T15:40:00Z      | 過期（同一瞬間為 23:40+08:00） |
| 2026-09-05T23:31:00+08:00 | 2026-09-05                 | 過期（退回同日） |
| 2026-09-05   | 2026-09-05T23:17:28+08:00  | 過期（退回同日） |
| 2026-09-06T00:10:00+08:00 | 2026-09-05                 | 未過期 |
| 2026-09-05T23:31:00+08:00 | not-a-date                 | 未過期（時戳視為缺席） |
