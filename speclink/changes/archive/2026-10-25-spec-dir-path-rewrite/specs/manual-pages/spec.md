## MODIFIED Requirements

### Requirement: 手冊頁的落點與檔名

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

手冊頁 SHALL 位於工作區的 `speclink/manual/` 目錄，每頁一個檔案，檔名 SHALL 為 kebab-case 的 ASCII 英文並以 `.md` 結尾（例：`first-login.md`）。生成端 SHALL 只寫入此目錄，目錄不存在時 SHALL 建立；SHALL NOT 在其他路徑放置手冊頁或索引檔。

#### Scenario: 首次生成建立目錄與頁

- **WHEN** 工作區尚無 `speclink/manual/` 而生成端寫出手冊
- **THEN** `speclink/manual/` 被建立，其下每個檔案皆為 kebab-case ASCII 檔名的 `.md`，工作區其他路徑無新增檔案

#### Scenario: 檔名不合規即違反契約

- **WHEN** 檢視 `speclink/manual/` 下任一頁的檔名
- **THEN** 不含空白、大寫字母、非 ASCII 字元或 `.md` 以外的副檔名

### Requirement: frontmatter 六欄

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

每頁 SHALL 以 YAML frontmatter 開頭並含下列欄位：`title`（字串，必填，頁的人類標題）、`section`（字串，必填，側欄分區名）、`order`（整數，必填，全手冊唯一的全域序號，慣例以 10 為間隔）、`keywords`（字串陣列，選填，供搜尋）、`sources`（字串陣列，必填，本頁取材的正式規格；首頁與來源頁得為空陣列）、`generated`（必填，本頁最近一次生成的時戳，格式為帶時區偏移量的 RFC 3339，秒級，例 `2026-09-05T23:31:00+08:00`）。`sources` 的每一項 SHALL 為 `<capability>` 或 `<capability>#<Requirement 名>` 兩種形式之一：井號前為 `speclink/specs/` 下既有的 capability 目錄名，井號後為該正式規格中某條 `### Requirement:` 標題的原文（去頭尾空白）；一項至多一個井號錨定，同一 capability 的多條 Requirement SHALL 寫成多項。生成端寫出帶錨定的項時 SHALL 以雙引號包住。除過期判定外，其他以 `sources` 為依據的規則（未入冊、出處行、來源消失）SHALL 只看井號前的 capability 名。讀取端 SHALL 同時接受純日期 `YYYY-MM-DD` 的 `generated`（本契約放寬前生成的頁），既有頁 SHALL NOT 因此需要回改；不帶錨定的既有 `sources` 項 SHALL 維持整份規格的語意，既有頁 SHALL NOT 因此需要回改。frontmatter SHALL NOT 含其他欄位。頁的排序 SHALL 僅由 `order` 決定；分區順序 SHALL 由分區內最小 `order` 決定。

#### Scenario: 合規的 frontmatter

- **WHEN** 檢視一頁的 frontmatter
- **THEN** 六欄齊備（`keywords` 得缺席），`order` 為整數且與其他頁不重複，`generated` 為合法的 RFC 3339 時戳或純日期，`sources` 每項井號前的名稱對應 `speclink/specs/` 下既有的 capability 目錄名

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

### Requirement: 必產的首頁與來源頁

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

每份手冊 SHALL 含 `index.md`（`title` 為手冊名、`section` 為第一個分區、`order` 為全手冊最小值；內容為系統一句話定位、三個以內的核心概念、依角色分流的入口連結）與 `about.md`（`title`「本手冊的來源」、`order` 為全手冊最大值；內容 SHALL 載明取材範圍為 `speclink/specs/`、規格內部新舊描述矛盾的清單（無矛盾時明寫「未發現」）、已知侷限（無截圖、以實機為準）與編成日期）。兩頁為衍生頁：任一其他頁新增或重生時 SHALL 隨之重生（`section` 與 `order` 保留），且 SHALL NOT 被判為過期、未入冊或來源消失。

#### Scenario: 生成後兩頁存在

- **WHEN** 生成端完成任一次生成
- **THEN** `speclink/manual/index.md` 與 `speclink/manual/about.md` 存在，`index.md` 的 `order` 為全手冊最小、`about.md` 為最大，且 `about.md` 含矛盾清單段（或「未發現」字樣）

#### Scenario: 無可入冊能力時仍產兩頁

- **WHEN** 規格中無任何被分流為使用者面向的 capability
- **THEN** 仍產出 `index.md` 與 `about.md`，`about.md` 載明「尚無可入冊的使用者面向能力」，無其他頁

### Requirement: 重生時保留既有順序

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

生成端重生手冊時 SHALL 先讀取既有各頁的 frontmatter；檔名已存在的頁，其 `section` 與 `order` SHALL 逐字保留（除非使用者明示要求重排）；新頁的 `order` SHALL 取相鄰頁之間的整數（例：20 與 30 之間填 25）而 SHALL NOT 重排既有頁；相鄰序號之間無整數可用（含新頁須排在 `about.md` 之後）時，生成端 SHALL NOT 自行重排，SHALL 將該頁留待下一輪並列入報告，待使用者明示要求重排後再入冊；未被判為過期的頁 SHALL 逐位元不變；被判為過期的頁，重生後（含依取材範圍重新推導的 `sources`）除 `generated` 行外其餘內容與磁碟上逐位元相同時，生成端 SHALL 只把 `generated` 換成本次時戳而其餘位元不動；因錨定找不到而過期的頁，重新推導的 `sources` 必然不同，SHALL 整頁重寫並寫入現行的 Requirement 標題；`sources` 非空且所列規格全部不復存在的頁 SHALL 列入報告而 SHALL NOT 自動刪除。

#### Scenario: 只重生過期頁

- **WHEN** 手冊有五頁、其中兩頁過期，生成端以預設方式重生
- **THEN** 兩頁的內文與 `generated` 更新而 `section`、`order` 不變；其餘三頁逐位元不變

#### Scenario: 過期但內文不變的頁只換時戳

- **WHEN** 一頁被判為過期，重生後除 `generated` 行外與磁碟上的內容逐位元相同
- **THEN** 該頁只有 `generated` 行變為本次時戳，其餘位元（含 `sources` 與內文）不變；下次以同一規格判定時該頁不再過期；`index.md` 與 `about.md` 隨之重生

#### Scenario: 錨定失效的頁整頁重寫

- **WHEN** 一頁 `sources` 為 `["desktop-app#舊段名"]`，正式規格 `desktop-app` 已把該 Requirement 改名為「新段名」，頁的其餘內文與重生結果相同
- **THEN** 該頁整頁重寫，`sources` 變為 `["desktop-app#新段名"]`、`generated` 為本次時戳；下次判定不再過期

#### Scenario: 插入新頁不重排

- **WHEN** 既有頁 `order` 為 20 與 30，生成端為新出現的使用者面向能力新增一頁並置於兩者之間
- **THEN** 新頁 `order` 為 21 至 29 之間的整數，既有兩頁的 `order` 不變

#### Scenario: 來源消失的頁不自動刪

- **WHEN** 某頁 `sources` 所列的 capability（取井號前名稱）已全部自 `speclink/specs/` 移除
- **THEN** 該頁保留於磁碟，生成端於報告中列出該頁並說明來源已消失
