## Purpose

desktop「手冊」頁的行為：讀取 `openspec/manual/` 的手冊頁（格式依 manual-pages 契約），以 frontmatter 推導側欄樹、搜尋與上一頁／下一頁，渲染內頁並讓出處可跳規格，標示可能過期的頁與手冊生成後新增且未入冊的規格，處理無手冊與 remote 模式的空狀態，並隨外部寫入即時重載；另涵蓋共用 Markdown 對 GitHub Alert 語法的提示框呈現。邊界止於讀取與呈現，不含手冊的生成、編輯或 remote 投影。

## ADDED Requirements

### Requirement: 手冊頁的側欄樹與閱讀序

手冊頁 SHALL 讀取專案 `openspec/manual/` 下全部 `.md` 頁的 frontmatter，以 `order` 升冪決定閱讀序（同值以檔名決斷），以 `section` 對閱讀序中連續的頁分組為側欄分區，分區順序為分區內最小 `order`。側欄每列 SHALL 顯示該頁 `title`；缺 `title` 時顯示檔名（去副檔名）、缺 `section` 歸入「其他」分區、缺或非整數 `order` 置於所屬分區末。frontmatter 無法解析的頁 SHALL 仍列於側欄（以檔名為標題）且可開啟，SHALL NOT 使頁面報錯。上一頁／下一頁 SHALL 為閱讀序中的相鄰頁，首頁無上一頁、末頁無下一頁。手冊頁 SHALL 為唯讀，SHALL NOT 提供任何寫入操作。

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

### Requirement: 手冊頁的搜尋列

手冊頁 SHALL 提供搜尋列：輸入時以大小寫不敏感的子字串即時比對各頁 `title` 與 `keywords`，側欄只保留命中的頁及其所屬分區；清空輸入 SHALL 還原完整側欄；無命中時側欄 SHALL 顯示無結果文案。搜尋 SHALL NOT 比對內文。

#### Scenario: 以標題或關鍵字過濾

- **WHEN** 頁 A（title「第一次登入」、keywords 含「github」）、頁 B（title「認識畫面」、無 keywords）並於搜尋列輸入「GitHub」
- **THEN** 側欄只剩頁 A 及其分區；清空輸入後 A、B 皆恢復顯示

#### Scenario: 無命中顯示無結果

- **WHEN** 搜尋列輸入任何頁的標題與關鍵字都不含的字串
- **THEN** 側欄顯示無結果文案，內容區維持目前頁

### Requirement: 內頁渲染與出處跳規格

選定頁的內文 SHALL 以共用 Markdown 元件渲染（去除 frontmatter），沿用共用閱讀欄與行寬上限、16px 基準字級、淺色與深色主題。頁尾出處行中的 capability 名 SHALL 可點：點擊 SHALL 切至規格頁並展開該 capability 的規格卡；該 capability 在正典中不存在時 SHALL 呈現為不可點文字。內文載入中 SHALL 以 skeleton 佔位，載入失敗 SHALL 於內容區顯示失敗文案且側欄照常。

#### Scenario: 點出處跳規格頁展開

- **WHEN** 頁尾出處行列有 `github-oauth`，使用者點擊它
- **THEN** 側欄切至規格頁高亮，規格頁滾至並展開 `github-oauth` 的規格卡

#### Scenario: 不存在的出處不可點

- **WHEN** 出處行列有正典中不存在的 capability 名
- **THEN** 該名稱以純文字呈現，點擊無任何效果

#### Scenario: 內文載入失敗

- **WHEN** 開啟某頁時內文讀取失敗
- **THEN** 內容區顯示載入失敗文案，側欄與其他頁的開啟不受影響

### Requirement: 可能過期與未入冊的標示

手冊頁 SHALL 依 manual-pages 契約計算過期：頁的 `sources` 中任一 capability 正典規格內 `@trace updated` 的最大日期晚於該頁 `generated` 時，側欄該頁列 SHALL 帶「可能過期」標記；`sources` 為空、`generated` 缺席或規格不存在時 SHALL NOT 標記。側欄底部 SHALL 在存在「手冊生成後新增且未入冊」的正典規格——其 `@trace updated` 的最小日期晚於全手冊最大 `generated`、且不在任何頁的 `sources`——時顯示計數提示；不存在時該提示 SHALL 缺席。兩種標示 SHALL 僅呈現，SHALL NOT 觸發生成。

#### Scenario: 來源更新後標示可能過期

- **WHEN** 頁 `generated` 為 2026-09-01、`sources` 含 x，而規格 x 的 `@trace updated` 最大為 2026-09-05
- **THEN** 側欄該頁列出現「可能過期」標記；規格 y 最大為 2026-08-20 的另一頁無標記

#### Scenario: 生成後新增的規格計入未入冊

- **WHEN** 全手冊最大 `generated` 為 2026-09-01，規格 z 的最小 `@trace updated` 為 2026-09-03 且不在任何頁的 `sources`
- **THEN** 側欄底部顯示未入冊規格數為 1 的提示；z 被加入某頁 `sources` 後提示消失

##### Example: 判定表

| 頁 generated | sources 規格最大 updated | 頁標記 |
| --- | --- | --- |
| 2026-09-01 | 2026-09-05 | 可能過期 |
| 2026-09-01 | 2026-08-20 | 無 |
| 2026-09-01 | （sources 為空） | 無 |
| （缺席） | 2026-09-05 | 無 |

### Requirement: 無手冊與 remote 模式的空狀態

`openspec/manual/` 不存在或其中無任何 `.md` 時，手冊頁 SHALL 顯示空狀態文案：說明尚無手冊、可用 manual 技能從規格生成；目錄不可讀時亦呈此空狀態且錯誤只記錄於日誌。分頁為 remote 資料源時，手冊頁 SHALL 顯示「remote 模式尚不支援手冊」的空狀態，SHALL NOT 嘗試讀取遠端。零分頁時點擊側欄「手冊」SHALL 呈現與變更頁相同的空狀態引導頁。

#### Scenario: 無手冊目錄

- **WHEN** 專案沒有 `openspec/manual/` 而使用者進入手冊頁
- **THEN** 主內容顯示尚無手冊的空狀態文案，側欄「手冊」項高亮，無錯誤彈窗

#### Scenario: remote 分頁

- **WHEN** 活躍分頁綁定 remote scope 而使用者進入手冊頁
- **THEN** 主內容顯示 remote 模式尚不支援手冊的空狀態，無任何網路請求發出

### Requirement: 手冊頁隨外部變更即時更新

手冊視圖活躍時，app 之外的寫者（manual 技能、手動編輯器）新增、修改或刪除 `openspec/manual/` 下的頁後，側欄索引與已開啟頁的內文 SHALL 於秒級自動重載至磁碟現況，SHALL NOT 要求重啟或重新進入頁面；重載回應交錯時 SHALL 以最新一次為準。監看不可用時手冊頁 SHALL 照常可讀，僅失去自動刷新。

#### Scenario: 外部重生一頁後內容更新

- **WHEN** 使用者正在閱讀某頁，外部以 manual 技能重生該頁
- **THEN** 數秒內內容區顯示新內文，側欄該頁的過期標記依新 `generated` 重算

#### Scenario: 外部新增頁後側欄出現

- **WHEN** 外部於 `openspec/manual/` 新增一頁（`order` 落於既有兩頁之間）
- **THEN** 數秒內側欄於對應位置出現該頁，其餘頁順序不變

### Requirement: Markdown 的 GitHub Alert 提示框

共用 Markdown 元件 SHALL 將首段以 `[!NOTE]`、`[!TIP]`、`[!WARNING]`、`[!CAUTION]` 之一開頭的 blockquote 呈現為對應類型的提示框：移除標記文字、顯示類型標籤、配色取自介面狀態語意色（資訊、成功、警告、危險）且不佔主色，其餘內容照常渲染；不符此形式的 blockquote SHALL 維持既有渲染逐位元不變。此呈現 SHALL 於淺色與深色主題一致生效，並適用於所有使用共用 Markdown 元件的檢視。

#### Scenario: 四型提示框

- **WHEN** 內文含 `> [!WARNING]` 開頭的 blockquote
- **THEN** 該段呈現為警告色提示框，標記文字 `[!WARNING]` 不出現，blockquote 內其餘文字完整顯示

#### Scenario: 一般引言不受影響

- **WHEN** 內文含首段不以四種標記開頭的 blockquote
- **THEN** 該段以既有 blockquote 樣式呈現，與本變更前逐位元一致
