## MODIFIED Requirements

### Requirement: frontmatter 六欄
<!-- BEFORE: sources 每項只能是 capability 名稱 -->

每頁 SHALL 以 YAML frontmatter 開頭並含下列欄位：`title`（字串，必填，頁的人類標題）、`section`（字串，必填，側欄分區名）、`order`（整數，必填，全手冊唯一的全域序號，慣例以 10 為間隔）、`keywords`（字串陣列，選填，供搜尋）、`sources`（字串陣列，必填，本頁取材的正典規格；首頁與來源頁得為空陣列）、`generated`（必填，本頁最近一次生成的時戳，格式為帶時區偏移量的 RFC 3339，秒級，例 `2026-09-05T23:31:00+08:00`）。`sources` 的每一項 SHALL 為 `<capability>` 或 `<capability>#<Requirement 名>` 兩種形式之一：井號前為 `openspec/specs/` 下既有的 capability 目錄名，井號後為該正典規格中某條 `### Requirement:` 標題的原文（去頭尾空白）；一項至多一個井號錨定，同一 capability 的多條 Requirement SHALL 寫成多項。生成端寫出帶錨定的項時 SHALL 以雙引號包住。除過期判定外，其他以 `sources` 為依據的規則（未入冊、出處行、來源消失）SHALL 只看井號前的 capability 名。讀取端 SHALL 同時接受純日期 `YYYY-MM-DD` 的 `generated`（本契約放寬前生成的頁），既有頁 SHALL NOT 因此需要回改；不帶錨定的既有 `sources` 項 SHALL 維持整份規格的語意，既有頁 SHALL NOT 因此需要回改。frontmatter SHALL NOT 含其他欄位。頁的排序 SHALL 僅由 `order` 決定；分區順序 SHALL 由分區內最小 `order` 決定。

#### Scenario: 合規的 frontmatter

- **WHEN** 檢視一頁的 frontmatter
- **THEN** 六欄齊備（`keywords` 得缺席），`order` 為整數且與其他頁不重複，`generated` 為合法的 RFC 3339 時戳或純日期，`sources` 每項井號前的名稱對應 `openspec/specs/` 下既有的 capability 目錄名

##### Example: 一頁的 frontmatter

- **GIVEN** 頁 `first-login.md`
- **WHEN** 讀其 frontmatter
- **THEN** 內容為 `title: 第一次登入`、`section: 開始使用`、`order: 20`、`keywords: [登入, github, 審核]`、`sources: [github-oauth, user-pending-blocked-pages]`、`generated: 2026-09-05T23:31:00+08:00`

#### Scenario: 帶 Requirement 錨定的 sources

- **WHEN** 一頁只取材 `desktop-app` 的「看板與任務」與「系統匣選單」兩條 Requirement，另整份取材 `policy-config`
- **THEN** 其 `sources` 為 `["desktop-app#看板與任務", "desktop-app#系統匣選單", policy-config]`，頁尾出處行只列 `desktop-app` 與 `policy-config` 兩個名稱各一次

#### Scenario: 純日期的舊頁照舊可讀

- **WHEN** 一頁的 `generated` 為 `2026-09-02`（放寬前生成）
- **THEN** 讀取端把它當作該日曆日的生成紀錄列於側欄並參與過期判定，不視為格式錯誤

#### Scenario: 排序由 order 推導

- **WHEN** 三頁的 `order` 分別為 30、10、20，`section` 依序為「文件協作」「開始使用」「開始使用」
- **THEN** 讀取端的閱讀序為 10、20、30，分區序為「開始使用」在「文件協作」之前；上一頁／下一頁即該序列中的相鄰頁

### Requirement: 過期判定基準
<!-- BEFORE: 以整個 capability 為單位，任一 @trace updated 晚於 generated 即過期 -->

一頁 SHALL 視為過期，若其 `sources` 中任一項判為過期。一項的判定 SHALL 依錨定有無分流：不帶錨定的項，取該 capability 正典規格內全部 `@trace updated` 時戳；帶錨定的項，SHALL 把正典規格全文依行首 `### Requirement:` 標題切段（一段自標題起、至下一個 `### Requirement:` 標題或檔尾止），取標題文字去頭尾空白後與錨定相等（區分大小寫）的那一段內的 `@trace updated` 時戳；任一取得的時戳「在該頁 `generated` 之後」即該項過期。帶錨定的項在正典規格中找不到相符標題、或該 capability 的正典規格不存在時，該項 SHALL 視為過期。「在之後」SHALL 依兩邊的格式分段判定：兩邊都是帶時區偏移量的 RFC 3339 時戳時，換算為同一瞬間後規格時戳嚴格晚於頁時戳才算，同一秒 SHALL NOT 算；任一邊只有純日期時，規格的日曆日不早於（晚於或同日）頁的日曆日即算，帶時間的一方取其時戳自身偏移量下的日曆日——生成當天的封存不得漏判。既非 RFC 3339 也非純日期的時戳 SHALL 視為缺席，不參與判定。一個 capability SHALL 視為未入冊，若它被生成端分流為使用者面向、且其名稱不出現在任何頁 `sources` 任一項的井號前。生成端與讀取端 SHALL 採同一基準；`sources` 為空的頁 SHALL NOT 判為過期。

#### Scenario: 過期與未入冊的判定

- **WHEN** 頁 A（`generated: 2026-09-01`，`sources: [x]`）而規格 x 的最新 `@trace updated` 為 2026-09-05；頁 B（`generated: 2026-09-01`，`sources: [y]`）而規格 y 最新為 2026-08-20；使用者面向 capability z 不在任何頁的 sources
- **THEN** A 判為過期、B 不過期、z 列為未入冊

#### Scenario: 錨定項只看該 Requirement 段落

- **WHEN** 頁 `generated` 為純日期 `2026-09-05`，`sources` 為 `["desktop-app#桌面上的品質關卡"]`；規格 `desktop-app` 中「桌面上的品質關卡」段的 `@trace updated` 為 `2026-08-14`，另一段「看板與任務」的為 `2026-09-10T17:15:24+08:00`
- **THEN** 該頁不判為過期；同一頁若 `sources` 改為 `[desktop-app]`，則判為過期

#### Scenario: 錨定找不到即過期

- **WHEN** 頁 `sources` 為 `["desktop-app#已改名的段"]` 而規格 `desktop-app` 無標題為「已改名的段」的 Requirement
- **THEN** 該頁判為過期，不論該規格其他段的時戳為何

#### Scenario: 錨定項不影響未入冊判定

- **WHEN** 使用者面向 capability x 只出現在某頁 `sources` 的 `"x#某段"` 項
- **THEN** x 不列為未入冊

#### Scenario: 同日先封存後生成不判過期

- **WHEN** 頁 `generated` 為 `2026-09-05T23:31:00+08:00`，`sources: [x]`，規格 x 的 `@trace updated` 最新為 `2026-09-05T23:17:28+08:00`
- **THEN** 該頁不判為過期；若規格 x 之後再封存得到 `2026-09-05T23:40:00+08:00`，該頁判為過期

#### Scenario: 任一邊純日期時退回同日規則

- **WHEN** 頁 `generated` 為 `2026-09-05T23:31:00+08:00` 而規格 x 最新 `@trace updated` 為純日期 `2026-09-05`；或頁 `generated` 為純日期 `2026-09-05` 而規格 x 最新為 `2026-09-05T23:17:28+08:00`
- **THEN** 兩種情況都判為過期（同日也算）；頁 `generated` 為 `2026-09-06T00:10:00+08:00` 對規格純日期 `2026-09-05` 則不過期

##### Example: 判定表

| 頁 generated | sources 項 | 該項取得的最新 @trace updated | 結果 |
| ------------ | ---------- | -------------------------- | ---- |
| 2026-09-01   | x          | 2026-09-05                 | 過期 |
| 2026-09-01   | x          | 2026-08-20                 | 未過期 |
| 2026-09-01   | x          | 2026-09-01                 | 過期（同日） |
| 2026-09-01   | （sources 為空） | —                    | 未過期 |
| 2026-09-05   | "desktop-app#桌面上的品質關卡" | 2026-08-14（該段；他段有 2026-09-10T17:15:24+08:00） | 未過期 |
| 2026-09-05   | desktop-app | 2026-09-10T17:15:24+08:00（整份） | 過期 |
| 2026-09-05   | "desktop-app#不存在的段" | （找不到段） | 過期 |
| 2026-09-05   | "not-exist#段" | （規格不存在） | 過期 |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T23:17:28+08:00 | 未過期（同日但更早） |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T23:40:00+08:00 | 過期 |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T23:31:00+08:00 | 未過期（同秒） |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T15:40:00Z      | 過期（同一瞬間為 23:40+08:00） |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05                 | 過期（退回同日） |
| 2026-09-05   | x          | 2026-09-05T23:17:28+08:00  | 過期（退回同日） |
| 2026-09-06T00:10:00+08:00 | x | 2026-09-05                 | 未過期 |
| 2026-09-05T23:31:00+08:00 | x | not-a-date                 | 未過期（時戳視為缺席） |

### Requirement: 重生時保留既有順序
<!-- BEFORE: 未被重生的頁一律逐位元不變，沒有「只推進 generated」的路徑 -->

生成端重生手冊時 SHALL 先讀取既有各頁的 frontmatter；檔名已存在的頁，其 `section` 與 `order` SHALL 逐字保留（除非使用者明示要求重排）；新頁的 `order` SHALL 取相鄰頁之間的整數（例：20 與 30 之間填 25）而 SHALL NOT 重排既有頁；相鄰序號之間無整數可用（含新頁須排在 `about.md` 之後）時，生成端 SHALL NOT 自行重排，SHALL 將該頁留待下一輪並列入報告，待使用者明示要求重排後再入冊；未被判為過期的頁 SHALL 逐位元不變；被判為過期的頁，重生後（含依取材範圍重新推導的 `sources`）除 `generated` 行外其餘內容與磁碟上逐位元相同時，生成端 SHALL 只把 `generated` 換成本次時戳而其餘位元不動；因錨定找不到而過期的頁，重新推導的 `sources` 必然不同，SHALL 整頁重寫並寫入現行的 Requirement 標題；`sources` 非空且所列規格全部不復存在的頁 SHALL 列入報告而 SHALL NOT 自動刪除。

#### Scenario: 只重生過期頁

- **WHEN** 手冊有五頁、其中兩頁過期，生成端以預設方式重生
- **THEN** 兩頁的內文與 `generated` 更新而 `section`、`order` 不變；其餘三頁逐位元不變

#### Scenario: 過期但內文不變的頁只換時戳

- **WHEN** 一頁被判為過期，重生後除 `generated` 行外與磁碟上的內容逐位元相同
- **THEN** 該頁只有 `generated` 行變為本次時戳，其餘位元（含 `sources` 與內文）不變；下次以同一規格判定時該頁不再過期；`index.md` 與 `about.md` 隨之重生

#### Scenario: 錨定失效的頁整頁重寫

- **WHEN** 一頁 `sources` 為 `["desktop-app#舊段名"]`，正典 `desktop-app` 已把該 Requirement 改名為「新段名」，頁的其餘內文與重生結果相同
- **THEN** 該頁整頁重寫，`sources` 變為 `["desktop-app#新段名"]`、`generated` 為本次時戳；下次判定不再過期

#### Scenario: 插入新頁不重排

- **WHEN** 既有頁 `order` 為 20 與 30，生成端為新出現的使用者面向能力新增一頁並置於兩者之間
- **THEN** 新頁 `order` 為 21 至 29 之間的整數，既有兩頁的 `order` 不變

#### Scenario: 來源消失的頁不自動刪

- **WHEN** 某頁 `sources` 所列的 capability（取井號前名稱）已全部自 `openspec/specs/` 移除
- **THEN** 該頁保留於磁碟，生成端於報告中列出該頁並說明來源已消失
