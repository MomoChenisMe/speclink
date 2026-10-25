## MODIFIED Requirements

### Requirement: 手冊頁的側欄樹與閱讀序

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

手冊頁 SHALL 讀取專案 `speclink/manual/` 下全部 `.md` 頁的 frontmatter，以兩層排序決定閱讀序：分區依分區內最小 `order` 排列，分區內的頁依 `order` 升冪（同值以檔名決斷）；側欄分區即閱讀序中連續同 `section` 的頁，同一分區的頁恆相鄰。側欄每列 SHALL 顯示該頁 `title`；缺 `title` 時顯示檔名（去副檔名）、缺 `section` 歸入「其他」分區、缺或非整數 `order` 置於所屬分區末。frontmatter 無法解析的頁 SHALL 仍列於側欄（以檔名為標題）且可開啟，SHALL NOT 使頁面報錯。上一頁／下一頁 SHALL 為閱讀序中的相鄰頁，首頁無上一頁、末頁無下一頁。手冊頁 SHALL 為唯讀，SHALL NOT 提供任何寫入操作。

#### Scenario: 依 order 排序並依 section 分組

- **WHEN** 手冊有四頁，`order` 與 `section` 分別為 10「開始使用」、20「開始使用」、30「文件協作」、40「附錄」
- **THEN** 側欄依序呈現「開始使用」（含前兩頁）、「文件協作」、「附錄」三個分區；第二頁的上一頁為第一頁、下一頁為第三頁；第一頁無上一頁、第四頁無下一頁

##### Example: 排序與分組

| 頁（檔名） | order | section | 側欄位置 | 上一頁 | 下一頁 |
| --- | --- | --- | --- | --- | --- |
| index | 10 | 開始使用 | 分區 1 第 1 列 | 無 | first-login |
| first-login | 20 | 開始使用 | 分區 1 第 2 列 | index | editor |
| editor | 30 | 文件協作 | 分區 2 第 1 列 | first-login | about |
| about | 40 | 附錄 | 分區 3 第 1 列 | editor | 無 |

#### Scenario: 缺欄位的頁寬容降級

- **WHEN** 某頁 frontmatter 缺 `title` 與 `section`、`order` 為非整數
- **THEN** 該頁以檔名為標題列於「其他」分區末，點擊可正常開啟內文，畫面無錯誤提示

#### Scenario: frontmatter 壞掉的頁仍可開

- **WHEN** 某頁不以 `---` 開頭或 YAML 無法解析
- **THEN** 該頁以檔名為標題出現在側欄且可開啟顯示全文，其他頁的順序與內容不受影響

### Requirement: 內頁渲染與出處跳規格

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

手冊頁主區 SHALL 為三欄：240px 目錄樹（側欄淡灰底）、閱讀欄、200px 本頁目錄（錨點列）。目錄樹的頁列 SHALL 與詳情頁文件樹的節點同款（30px 列、圓角、作用中為主色淡底與主色字、hover 淡底），每列顯示該頁 `title`。選定頁的內文 SHALL 以共用 Markdown 元件渲染（去除 frontmatter），閱讀欄內容寬 768px 置中、沿用「markdown 文件內容行寬有上限」的閱讀卡上限、15px 基準字級與 1.75 行高、h2 章節 16px 粗體、h3 小節 14px 粗體、淺色與深色主題。閱讀欄 SHALL 分兩段：中段為捲動區內的白底閱讀卡，卡由上而下為標題區（24px 一般字重的頁標題——內文第一個非空行為 `# 標題` 時取該行且內文 SHALL NOT 重複呈現該 H1，否則取索引的 `title`；籤槽為分區籤（缺 `section` 時「其他」）與「可能過期」籤（見「可能過期與未入冊的標示」）；同列右端為「開啟 ▾」，行為依 desktop-app「詳情頁的文件動作」對應 `speclink/manual/<slug>.md`）、出處行（灰字：頁檔相對所在 checkout 根的實際路徑（spec 目錄名依專案設定、預設為 `speclink/manual/<slug>.md`；查不到時該段缺席）、「產生於 {generated}」（`generated` 缺席時該段缺席）、「出處」與出處籤）、數字磚「章節」（內文 h2 數）、「小節」（h3 數）、「關鍵字」（索引 `keywords` 數）、「出處規格」（出處去重數）（內文未載入或載入失敗時數字磚整列缺席）、提示框、內文；頁尾為固定底列（白底、上緣細線、內容對齊 768px）：左為出處列，右為上一頁／下一頁框線鈕（各帶小字標籤與目標頁標題，無目標時該鈕缺席）。底列 SHALL NOT 隨內文捲動；標題區隨閱讀卡捲動。內文含 h2／h3 標題時，內容區右側 SHALL 顯示錨點列依序列出各標題（h3 縮排一級；閱讀卡標題區的頁標題 SHALL NOT 列入）：點擊 SHALL 捲至該標題，捲動時 SHALL 高亮目前段；內文無 h2／h3 時錨點列 SHALL 缺席。換頁 SHALL 回到內文頂端；外部改內文觸發的重載 SHALL 維持捲動位置。出處行與頁尾出處列 SHALL 依索引 `sources` 推導：每項取第一個 `#` 前的 capability 名並去重，錨定文字 SHALL NOT 出現。兩處的 capability 名 SHALL 可點：點擊 SHALL 於手冊頁上開啟該 capability 的規格詳情頁（與規格頁共用同一頁；宿主維持手冊頁，Esc 或麵包屑回手冊），SHALL NOT 切離手冊頁；該 capability 在正式規格中不存在時 SHALL 呈現為不可點文字。內文載入中 SHALL 以 skeleton 佔位，載入失敗 SHALL 於內容區顯示失敗文案且側欄照常。

#### Scenario: 標題進卡與頁尾固定

- **WHEN** 開啟一頁內文長於可視區、開頭為 `# Speclink 操作手冊` 的頁，並捲至內文底部
- **THEN** 頁標題「Speclink 操作手冊」在閱讀卡標題區、隨卡捲出可視區，捲動區外沒有固定頁首；白底上緣細線的底列（出處籤與帶目標頁標題的上一頁／下一頁鈕）仍固定於底部可見且與 768px 閱讀欄對齊，內文捲動區內沒有重複的 H1

#### Scenario: 右側錨點列

- **WHEN** 開啟內文依序含 `## 看板`、`### 卡片`、`## 抽屜` 的頁
- **THEN** 內容區右側錨點列依序列出「看板」「卡片」「抽屜」，「卡片」縮排一級，「看板」為目前段，閱讀卡標題區的頁標題不在列中；點擊「抽屜」內文捲至該標題；切到無 h2／h3 的頁後錨點列消失

#### Scenario: 點出處開啟規格詳情頁

- **WHEN** 頁尾出處行列有 `github-oauth`，使用者點擊它
- **THEN** 側欄手冊項維持高亮，`github-oauth` 的規格詳情頁覆蓋主區、麵包屑為「專案名 / 手冊 / github-oauth」；按 Esc 回到手冊頁且內文捲動位置不變

#### Scenario: 帶錨定的出處只列 capability 一次

- **WHEN** 索引中某頁 `sources` 為 `["desktop-app#看板與任務", "desktop-app#系統匣選單", "policy-config"]`，正式規格存在 `desktop-app` 而不存在 `policy-config`
- **THEN** 出處行與頁尾出處列各恰有一顆可點的 `desktop-app` 與一個不可點的 `policy-config`，文字中不出現 `#`；點 `desktop-app` 開啟其規格詳情頁；數字磚「出處規格」為 2

#### Scenario: 不存在的出處不可點

- **WHEN** 出處行列有正式規格中不存在的 capability 名
- **THEN** 該名稱以純文字呈現，點擊無任何效果

#### Scenario: 內文載入失敗

- **WHEN** 開啟某頁時內文讀取失敗
- **THEN** 內容區顯示載入失敗文案、無數字磚、無提示框，側欄與其他頁的開啟不受影響

#### Scenario: 閱讀卡標題區、出處行與數字磚

- **WHEN** 開啟〈系統匣選單〉（`speclink/manual/tray.md`，`section` 為「桌面 app」、`generated` 為 `2026-09-17T16:05:41+08:00`、`keywords` 7 個、`sources` 為 `["tray-status-menu"]`、內文 3 個 h2 與 11 個 h3）
- **THEN** 閱讀卡標題區為「系統匣選單」、分區籤「桌面 app」、同列右端「開啟 ▾」；出處行為 `speclink/manual/tray.md · 產生於 2026-09-17T16:05:41+08:00 · 出處 tray-status-menu`（tray-status-menu 為可點籤）；數字磚依序為 3「章節」、11「小節」、7「關鍵字」、1「出處規格」；捲動區外沒有頁首

#### Scenario: 手冊頁的開啟選單與字級

- **WHEN** 使用者於本機專案的手冊頁展開閱讀卡的「開啟 ▾」並選「在 Finder 顯示」
- **THEN** Finder 顯示該專案的 `speclink/manual/<slug>.md`（選單首列為同一相對路徑）；內文的計算字級為 15px、行高 15px 的 1.75 倍，h2 為 16px 粗體、h3 為 14px 粗體

#### Scenario: 目錄樹節點同文件樹款式

- **WHEN** 使用者開啟手冊頁並把指標移到非作用中的頁列
- **THEN** 頁列高 30px、圓角、hover 淡底；作用中的頁列為主色淡底與主色字、`aria-current` 為 page

### Requirement: 無手冊與 remote 模式的空狀態

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink/manual/` 不存在或其中無任何 `.md` 時，手冊頁 SHALL 顯示空狀態文案：說明尚無手冊、可用手冊技能從規格生成；目錄不可讀時亦呈此空狀態且錯誤只記錄於日誌。分頁為 remote 資料源時，手冊頁 SHALL 顯示「remote 模式尚不支援手冊」的空狀態，SHALL NOT 嘗試讀取遠端。零分頁時點擊側欄「手冊」SHALL 呈現與變更頁相同的空狀態引導頁。

#### Scenario: 無手冊目錄

- **WHEN** 專案沒有 `speclink/manual/` 而使用者進入手冊頁
- **THEN** 主內容顯示尚無手冊的空狀態文案，側欄「手冊」項高亮，無錯誤彈窗

#### Scenario: remote 分頁

- **WHEN** 活躍分頁綁定 remote scope 而使用者進入手冊頁
- **THEN** 主內容顯示 remote 模式尚不支援手冊的空狀態，無任何網路請求發出

### Requirement: 手冊頁隨外部變更即時更新

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

手冊視圖活躍時，app 之外的寫者（手冊技能、手動編輯器）新增、修改或刪除 `speclink/manual/` 下的頁後，側欄索引與已開啟頁的內文 SHALL 於秒級自動重載至磁碟現況，SHALL NOT 要求重啟或重新進入頁面；重載回應交錯時 SHALL 以最新一次為準。監看不可用時手冊頁 SHALL 照常可讀，僅失去自動刷新。

#### Scenario: 外部重生一頁後內容更新

- **WHEN** 使用者正在閱讀某頁，外部以手冊技能重生該頁
- **THEN** 數秒內內容區顯示新內文，側欄該頁的過期標記依新 `generated` 重算

#### Scenario: 外部新增頁後側欄出現

- **WHEN** 外部於 `speclink/manual/` 新增一頁（`order` 落於既有兩頁之間）
- **THEN** 數秒內側欄於對應位置出現該頁，其餘頁順序不變
