## MODIFIED Requirements

### Requirement: 內頁渲染與出處跳規格

手冊頁主區 SHALL 為三欄：240px 目錄樹（側欄淡灰底）、閱讀欄、200px 本頁目錄（錨點列）。選定頁的內文 SHALL 以共用 Markdown 元件渲染（去除 frontmatter），閱讀欄內容寬 768px 置中、沿用行寬上限、16px 基準字級、淺色與深色主題。閱讀欄 SHALL 分三段：頁首固定顯示 24px 一般字重的頁標題——內文第一個非空行為 `# 標題` 時取該行且內文 SHALL NOT 重複呈現該 H1，否則取索引的 `title`——與一行灰字「產生於 {generated}」（`generated` 缺席時該行缺席）；中段為白底閱讀卡內的內文捲動區；頁尾為固定底列（白底、上緣細線、內容對齊 768px）：左為出處列，右為上一頁／下一頁框線鈕（各帶小字標籤與目標頁標題，無目標時該鈕缺席）。頁首與底列 SHALL NOT 隨內文捲動。內文含 h2／h3 標題時，內容區右側 SHALL 顯示錨點列依序列出各標題（h3 縮排一級）：點擊 SHALL 捲至該標題，捲動時 SHALL 高亮目前段；內文無 h2／h3 時錨點列 SHALL 缺席。換頁 SHALL 回到內文頂端；外部改內文觸發的重載 SHALL 維持捲動位置。頁尾出處列 SHALL 依索引 `sources` 推導：每項取第一個 `#` 前的 capability 名並去重，錨定文字 SHALL NOT 出現在出處列。出處列中的 capability 名 SHALL 可點：點擊 SHALL 於手冊頁上開啟該 capability 的唯讀規格抽屜（與規格頁共用同一抽屜），SHALL NOT 切離手冊頁；該 capability 在正式規格中不存在時 SHALL 呈現為不可點文字。內文載入中 SHALL 以 skeleton 佔位，載入失敗 SHALL 於內容區顯示失敗文案且側欄照常。

#### Scenario: 頁首與頁尾固定

- **WHEN** 開啟一頁內文長於可視區、開頭為 `# Speclink 操作手冊` 的頁，並捲至內文底部
- **THEN** 頁標題「Speclink 操作手冊」仍固定於頂部可見，白底上緣細線的底列（出處籤與帶目標頁標題的上一頁／下一頁鈕）仍固定於底部可見且與 768px 閱讀欄對齊，內文捲動區內沒有重複的 H1

#### Scenario: 右側錨點列

- **WHEN** 開啟內文依序含 `## 看板`、`### 卡片`、`## 抽屜` 的頁
- **THEN** 內容區右側錨點列依序列出「看板」「卡片」「抽屜」，「卡片」縮排一級，「看板」為目前段；點擊「抽屜」內文捲至該標題；切到無 h2／h3 的頁後錨點列消失

#### Scenario: 點出處開啟規格抽屜

- **WHEN** 頁尾出處行列有 `github-oauth`，使用者點擊它
- **THEN** 側欄手冊項維持高亮、手冊內文仍在畫面，`github-oauth` 的規格抽屜於手冊頁上滑入並顯示其規格內文

#### Scenario: 帶錨定的出處只列 capability 一次

- **WHEN** 索引中某頁 `sources` 為 `["desktop-app#看板與任務", "desktop-app#系統匣選單", "policy-config"]`，正式規格存在 `desktop-app` 而不存在 `policy-config`
- **THEN** 出處列恰有一顆可點的 `desktop-app` 與一個不可點的 `policy-config`，文字中不出現 `#`；點 `desktop-app` 開啟其規格抽屜

#### Scenario: 不存在的出處不可點

- **WHEN** 出處行列有正式規格中不存在的 capability 名
- **THEN** 該名稱以純文字呈現，點擊無任何效果

#### Scenario: 內文載入失敗

- **WHEN** 開啟某頁時內文讀取失敗
- **THEN** 內容區顯示載入失敗文案，側欄與其他頁的開啟不受影響
