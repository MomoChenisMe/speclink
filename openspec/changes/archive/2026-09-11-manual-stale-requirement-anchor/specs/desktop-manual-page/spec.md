## MODIFIED Requirements

### Requirement: 內頁渲染與出處跳規格
<!-- BEFORE: 出處列直接以索引 sources 原字串列出，無錨定語意 -->

選定頁的內文 SHALL 以共用 Markdown 元件渲染（去除 frontmatter），沿用共用閱讀欄與行寬上限、16px 基準字級、淺色與深色主題。內容區 SHALL 分三段：頁首固定顯示頁標題——內文第一個非空行為 `# 標題` 時取該行且內文 SHALL NOT 重複呈現該 H1，否則取索引的 `title`；中段為內文捲動區；頁尾固定顯示出處列與上一頁／下一頁。頁首與頁尾 SHALL NOT 隨內文捲動。內文含 h2／h3 標題時，內容區右側 SHALL 顯示錨點列依序列出各標題（h3 縮排一級）：點擊 SHALL 捲至該標題，捲動時 SHALL 高亮目前段；內文無 h2／h3 時錨點列 SHALL 缺席。換頁 SHALL 回到內文頂端；外部改內文觸發的重載 SHALL 維持捲動位置。頁尾出處列 SHALL 依索引 `sources` 推導：每項取第一個 `#` 前的 capability 名並去重，錨定文字 SHALL NOT 出現在出處列。出處列中的 capability 名 SHALL 可點：點擊 SHALL 於手冊頁上開啟該 capability 的唯讀規格抽屜（與規格頁共用同一抽屜），SHALL NOT 切離手冊頁；該 capability 在正典中不存在時 SHALL 呈現為不可點文字。內文載入中 SHALL 以 skeleton 佔位，載入失敗 SHALL 於內容區顯示失敗文案且側欄照常。

#### Scenario: 頁首與頁尾固定

- **WHEN** 開啟一頁內文長於可視區、開頭為 `# Speclink 操作手冊` 的頁，並捲至內文底部
- **THEN** 頁標題「Speclink 操作手冊」仍固定於頂部可見，出處列與上一頁／下一頁仍固定於底部可見，內文捲動區內沒有重複的 H1

#### Scenario: 右側錨點列

- **WHEN** 開啟內文依序含 `## 看板`、`### 卡片`、`## 抽屜` 的頁
- **THEN** 內容區右側錨點列依序列出「看板」「卡片」「抽屜」，「卡片」縮排一級，「看板」為目前段；點擊「抽屜」內文捲至該標題；切到無 h2／h3 的頁後錨點列消失

#### Scenario: 點出處開啟規格抽屜

- **WHEN** 頁尾出處行列有 `github-oauth`，使用者點擊它
- **THEN** 側欄手冊項維持高亮、手冊內文仍在畫面，`github-oauth` 的規格抽屜於手冊頁上滑入並顯示其規格內文

#### Scenario: 帶錨定的出處只列 capability 一次

- **WHEN** 索引中某頁 `sources` 為 `["desktop-app#看板與任務", "desktop-app#系統匣選單", "policy-config"]`，正典存在 `desktop-app` 而不存在 `policy-config`
- **THEN** 出處列恰有一顆可點的 `desktop-app` 與一個不可點的 `policy-config`，文字中不出現 `#`；點 `desktop-app` 開啟其規格抽屜

#### Scenario: 不存在的出處不可點

- **WHEN** 出處行列有正典中不存在的 capability 名
- **THEN** 該名稱以純文字呈現，點擊無任何效果

#### Scenario: 內文載入失敗

- **WHEN** 開啟某頁時內文讀取失敗
- **THEN** 內容區顯示載入失敗文案，側欄與其他頁的開啟不受影響


### Requirement: 可能過期與未入冊的標示
<!-- BEFORE: 以 sources 中 capability 整份規格的任一 @trace updated 判定；無錨定語意 -->

手冊頁 SHALL 依 manual-pages 契約「過期判定基準」計算過期：頁的 `sources` 中任一項判為過期時，側欄該頁列 SHALL 帶「可能過期」標記。一項的判定 SHALL 依錨定分流：不帶井號的項取該 capability 正典規格內全部 `@trace updated` 時戳；`<capability>#<Requirement 名>` 形式的項，SHALL 只取正典規格中標題與錨定相等（去頭尾空白、區分大小寫）的那條 `### Requirement:` 段落內的 `@trace updated` 時戳，找不到相符標題或規格不存在時該項 SHALL 視為過期。「在之後」SHALL 分段判定：兩邊都是帶時區偏移量的 RFC 3339 時戳時，換算同一瞬間後規格時戳嚴格晚於頁時戳才算（同秒不算）；任一邊只有純日期時，規格日曆日不早於頁日曆日（同日也算）即算，帶時間的一方取其自身偏移量下的日曆日。`sources` 為空、`generated` 缺席或既非 RFC 3339 也非 `YYYY-MM-DD` 時 SHALL NOT 標記；不帶錨定的項其規格不存在時 SHALL NOT 標記；規格內無法解析的 `updated` 時戳 SHALL 視為缺席。側欄底部 SHALL 在存在「手冊生成後新增且未入冊」的正典規格——其每一個 `@trace updated` 時戳都在每一頁 `generated` 之後（依同一分段判定）、且其名稱不在任何頁 `sources` 任一項的井號前——時顯示計數提示；不存在時該提示 SHALL 缺席。索引中每頁的 `generated` 欄位 SHALL 為 frontmatter 原字串，無法解析時為 null；`sources` 欄位 SHALL 為 frontmatter 原字串陣列（含錨定原樣），井號前名稱不合路徑守門的項 SHALL 整項略去。兩種標示 SHALL 僅呈現，SHALL NOT 觸發生成。

#### Scenario: 來源更新後標示可能過期

- **WHEN** 頁 `generated` 為 2026-09-01、`sources` 含 x，而規格 x 的 `@trace updated` 最大為 2026-09-05
- **THEN** 側欄該頁列出現「可能過期」標記；規格 y 最大為 2026-08-20 的另一頁無標記

#### Scenario: 錨定項只看該段落

- **WHEN** 頁 `generated` 為純日期 2026-09-05、`sources` 為 `["desktop-app#桌面上的品質關卡"]`，規格 desktop-app 中該段 `@trace updated` 為 2026-08-14、「看板與任務」段為 `2026-09-10T17:15:24+08:00`
- **THEN** 側欄該頁列無標記；把 `sources` 改為 `[desktop-app]` 後，該列於數秒內出現「可能過期」標記

#### Scenario: 錨定找不到即標記

- **WHEN** 頁 `sources` 為 `["desktop-app#已改名的段"]` 而規格 desktop-app 無此標題的 Requirement
- **THEN** 側欄該頁列出現「可能過期」標記，索引中該頁 `sources` 仍回傳 `["desktop-app#已改名的段"]` 原字串

#### Scenario: 同日先封存後生成不標記

- **WHEN** 頁 `generated` 為 `2026-09-05T23:31:00+08:00`、`sources` 含 x，規格 x 的 `@trace updated` 最新為 `2026-09-05T23:17:28+08:00`
- **THEN** 側欄該頁列無標記；規格 x 再封存得到 `2026-09-05T23:40:00+08:00` 後，該列於數秒內出現「可能過期」標記

#### Scenario: 純日期一方退回同日規則

- **WHEN** 頁 `generated` 為 `2026-09-05T23:31:00+08:00` 而規格 x 最新 `@trace updated` 為純日期 `2026-09-05`
- **THEN** 側欄該頁列出現「可能過期」標記

#### Scenario: 生成後新增的規格計入未入冊

- **WHEN** 全手冊最大 `generated` 為 2026-09-01，規格 z 的最小 `@trace updated` 為 2026-09-03 且不在任何頁的 `sources`
- **THEN** 側欄底部顯示未入冊規格數為 1 的提示；z 被加入某頁 `sources`（以 `z` 或 `"z#某段"` 任一形式）後提示消失

##### Example: 判定表

| 頁 generated | sources 項 | 該項取得的最大 updated | 頁標記 |
| --- | --- | --- | --- |
| 2026-09-01 | x | 2026-09-05 | 可能過期 |
| 2026-09-01 | x | 2026-08-20 | 無 |
| 2026-09-01 | x | 2026-09-01 | 可能過期（同日） |
| 2026-09-01 | （sources 為空） | — | 無 |
| （缺席） | x | 2026-09-05 | 無 |
| 2026-09-05 | "desktop-app#桌面上的品質關卡" | 2026-08-14（該段） | 無 |
| 2026-09-05 | desktop-app | 2026-09-10T17:15:24+08:00（整份） | 可能過期 |
| 2026-09-05 | "desktop-app#不存在的段" | （找不到段） | 可能過期 |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T23:17:28+08:00 | 無 |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T23:40:00+08:00 | 可能過期 |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T23:31:00+08:00 | 無（同秒） |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05T15:40:00Z | 可能過期 |
| 2026-09-05T23:31:00+08:00 | x | 2026-09-05 | 可能過期（退回同日） |
| 2026-09-05 | x | 2026-09-05T23:17:28+08:00 | 可能過期（退回同日） |
| 2026-09-06T00:10:00+08:00 | x | 2026-09-05 | 無 |
| 2026-09-05T23:31:00+08:00 | x | not-a-date | 無 |
