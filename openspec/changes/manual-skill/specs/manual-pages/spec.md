## Purpose

手冊頁的格式與落點契約：`openspec/manual/` 下由技能生成、給人閱讀的 Markdown 頁——檔名、frontmatter 六欄（即索引與過期比對的唯一依據）、內文慣例、必產的首頁與來源頁、過期判定基準，以及重生時保留既有順序的規則。本 capability 是生成端（manual 技能）與所有讀取端（desktop 手冊頁、SSG、其他工具）共用的契約；不涵蓋任何讀取端的呈現行為。

## ADDED Requirements

### Requirement: 手冊頁的落點與檔名

手冊頁 SHALL 位於工作區的 `openspec/manual/` 目錄，每頁一個檔案，檔名 SHALL 為 kebab-case 的 ASCII 英文並以 `.md` 結尾（例：`first-login.md`）。生成端 SHALL 只寫入此目錄，目錄不存在時 SHALL 建立；SHALL NOT 在其他路徑放置手冊頁或索引檔。

#### Scenario: 首次生成建立目錄與頁

- **WHEN** 工作區尚無 `openspec/manual/` 而生成端寫出手冊
- **THEN** `openspec/manual/` 被建立，其下每個檔案皆為 kebab-case ASCII 檔名的 `.md`，工作區其他路徑無新增檔案

#### Scenario: 檔名不合規即違反契約

- **WHEN** 檢視 `openspec/manual/` 下任一頁的檔名
- **THEN** 不含空白、大寫字母、非 ASCII 字元或 `.md` 以外的副檔名

### Requirement: frontmatter 六欄

每頁 SHALL 以 YAML frontmatter 開頭並含下列欄位：`title`（字串，必填，頁的人類標題）、`section`（字串，必填，側欄分區名）、`order`（整數，必填，全手冊唯一的全域序號，慣例以 10 為間隔）、`keywords`（字串陣列，選填，供搜尋）、`sources`（capability 名稱陣列，必填，本頁取材的正典規格；首頁與來源頁得為空陣列）、`generated`（`YYYY-MM-DD` 日期，必填，本頁最近一次生成日）。frontmatter SHALL NOT 含其他欄位。頁的排序 SHALL 僅由 `order` 決定；分區順序 SHALL 由分區內最小 `order` 決定。

#### Scenario: 合規的 frontmatter

- **WHEN** 檢視一頁的 frontmatter
- **THEN** 六欄齊備（`keywords` 得缺席），`order` 為整數且與其他頁不重複，`generated` 為合法日期，`sources` 每項對應 `openspec/specs/` 下既有的 capability 目錄名

##### Example: 一頁的 frontmatter

- **GIVEN** 頁 `first-login.md`
- **WHEN** 讀其 frontmatter
- **THEN** 內容為 `title: 第一次登入`、`section: 開始使用`、`order: 20`、`keywords: [登入, github, 審核]`、`sources: [github-oauth, user-pending-blocked-pages]`、`generated: 2026-09-02`

#### Scenario: 排序由 order 推導

- **WHEN** 三頁的 `order` 分別為 30、10、20，`section` 依序為「文件協作」「開始使用」「開始使用」
- **THEN** 讀取端的閱讀序為 10、20、30，分區序為「開始使用」在「文件協作」之前；上一頁／下一頁即該序列中的相鄰頁

### Requirement: 內文慣例

頁內文 SHALL 使用 GitHub Flavored Markdown。提示框 SHALL 用 GitHub Alert 語法（`> [!NOTE]`、`> [!TIP]`、`> [!WARNING]`、`> [!CAUTION]`）。跨頁連結 SHALL 用相對檔名（例：`[認識畫面](layout.md)`）。每頁結尾 SHALL 有一行以 `**出處**：` 開頭的出處行，逐一以反引號列出與 frontmatter `sources` 相同的 capability 名。內文 SHALL NOT 含 HTML 標籤；SHALL NOT 出現 `--json` 欄位名、旗標或程式碼細節，除非該內容是使用者實際輸入的指令或技能名。

#### Scenario: 出處行與 sources 一致

- **WHEN** 一頁的 `sources` 為 `[github-oauth, user-pending-blocked-pages]`
- **THEN** 該頁最後一個非空段落為 `**出處**：` 開頭且恰含這兩個反引號名稱

#### Scenario: 提示框與連結形式

- **WHEN** 檢視任一頁內文
- **THEN** 提示框皆為 GitHub Alert 語法、跨頁連結皆為相對 `.md` 檔名，且全文無 HTML 標籤

### Requirement: 必產的首頁與來源頁

每份手冊 SHALL 含 `index.md`（`title` 為手冊名、`section` 為第一個分區、`order` 為全手冊最小值；內容為系統一句話定位、三個以內的核心概念、依角色分流的入口連結）與 `about.md`（`title`「本手冊的來源」、`order` 為全手冊最大值；內容 SHALL 載明取材範圍為 `openspec/specs/`、規格內部新舊描述矛盾的清單（無矛盾時明寫「未發現」）、已知侷限（無截圖、以實機為準）與編成日期）。

#### Scenario: 生成後兩頁存在

- **WHEN** 生成端完成任一次生成
- **THEN** `openspec/manual/index.md` 與 `openspec/manual/about.md` 存在，`index.md` 的 `order` 為全手冊最小、`about.md` 為最大，且 `about.md` 含矛盾清單段（或「未發現」字樣）

#### Scenario: 無可入冊能力時仍產兩頁

- **WHEN** 規格中無任何被分流為使用者面向的 capability
- **THEN** 仍產出 `index.md` 與 `about.md`，`about.md` 載明「尚無可入冊的使用者面向能力」，無其他頁

### Requirement: 過期判定基準

一頁 SHALL 視為過期，若其 `sources` 中任一 capability 的正典規格內最新的 `@trace updated` 日期晚於該頁的 `generated`。一個 capability SHALL 視為未入冊，若它被生成端分流為使用者面向、且不出現在任何頁的 `sources`。生成端與讀取端 SHALL 採同一基準；`sources` 為空的頁 SHALL NOT 判為過期。

#### Scenario: 過期與未入冊的判定

- **WHEN** 頁 A（`generated: 2026-09-01`，`sources: [x]`）而規格 x 的最新 `@trace updated` 為 2026-09-05；頁 B（`generated: 2026-09-01`，`sources: [y]`）而規格 y 最新為 2026-08-20；使用者面向 capability z 不在任何頁的 sources
- **THEN** A 判為過期、B 不過期、z 列為未入冊

##### Example: 判定表

| 頁 generated | sources 最新 @trace updated | 結果 |
| ------------ | -------------------------- | ---- |
| 2026-09-01   | 2026-09-05                 | 過期 |
| 2026-09-01   | 2026-08-20                 | 未過期 |
| 2026-09-01   | （sources 為空）           | 未過期 |

### Requirement: 重生時保留既有順序

生成端重生手冊時 SHALL 先讀取既有各頁的 frontmatter；檔名已存在的頁，其 `section` 與 `order` SHALL 逐字保留（除非使用者明示要求重排）；新頁的 `order` SHALL 取相鄰頁之間的整數（例：20 與 30 之間填 25）而 SHALL NOT 重排既有頁；未被重生的頁 SHALL 逐位元不變；`sources` 所列規格全部不復存在的頁 SHALL 列入報告而 SHALL NOT 自動刪除。

#### Scenario: 只重生過期頁

- **WHEN** 手冊有五頁、其中兩頁過期，生成端以預設方式重生
- **THEN** 兩頁的內文與 `generated` 更新而 `section`、`order` 不變；其餘三頁逐位元不變

#### Scenario: 插入新頁不重排

- **WHEN** 既有頁 `order` 為 20 與 30，生成端為新出現的使用者面向能力新增一頁並置於兩者之間
- **THEN** 新頁 `order` 為 21 至 29 之間的整數，既有兩頁的 `order` 不變

#### Scenario: 來源消失的頁不自動刪

- **WHEN** 某頁 `sources` 所列的 capability 已全部自 `openspec/specs/` 移除
- **THEN** 該頁保留於磁碟，生成端於報告中列出該頁並說明來源已消失
