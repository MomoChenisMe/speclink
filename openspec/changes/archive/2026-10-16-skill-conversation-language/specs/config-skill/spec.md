## ADDED Requirements

### Requirement: 技能規定起草內容的撰寫語言依 locale

渲染產出的 speclink-config 技能檔 SHALL 在顯示 diff 並寫入的步驟規定撰寫語言：以政策欄位確認步驟（Step 3）確認後的 `locale` 為準（其初值來自固定輸入來源中 `speclink workflow-config show --json` 的 `locale` 欄位），技能起草或改寫的 `context` 與 `rules` 行 SHALL 以該語言撰寫——`tw` 為繁體中文、`ja` 為日文、`en` 或未設定為英文；確認步驟改了 `locale` 時，SHALL 先把已起草的行改寫成新語言，再產生 diff。未改動的既有行 SHALL 維持原文，SHALL NOT 僅為統一語言而翻譯改寫。review 規則（`critical`、`warning`、`suggestion` 三個清單）SHALL 照使用者原話記錄，SHALL NOT 翻譯。指令、檔案路徑、測試名、設定鍵與程式碼識別字 SHALL 維持原樣。結尾自我檢查清單 SHALL 以一句話指回此規則。對話本身的語言依共用對話語言段落，SHALL NOT 由 `locale` 決定。

此為刻意變更：變更前技能未規定起草語言；變更後依 `locale`，未設定 `locale` 時為英文。落地方式不變，仍經 `--dry-run` diff 由使用者核可。

#### Scenario: 渲染產物含起草語言規則

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** speclink-config 技能檔的顯示 diff 步驟含依 Step 3 確認後的 `locale` 撰寫 context 與 rules 新行、確認步驟改了 `locale` 時先改寫草稿、未改動的既有行維持原文、review 規則照使用者原話的規定，且此規則在技能檔中只完整陳述一次

#### Scenario: 確認步驟改了 locale 時先改寫草稿

- **WHEN** 專案原本未設定 `locale`，config 技能以英文起草 context 新行後，使用者在 Step 3 把 `locale` 改為 `tw`
- **THEN** 技能先把那些新行改寫成繁體中文，再以 `--dry-run` 顯示 diff；diff 中 `locale` 行與 context 新行同時出現

#### Scenario: locale tw 時起草的新行為繁體中文

- **WHEN** 專案設 `locale: tw`，config 技能為 context 起草一條新行並以 `speclink workflow-config context --dry-run` 顯示 diff
- **THEN** diff 中新增的行為繁體中文，行內的指令、路徑與 crate 名稱維持原樣

#### Scenario: review 規則照原話記錄

- **WHEN** 專案設 `locale: tw`，使用者以英文口述一條 warning 級 review 規則
- **THEN** 寫入 review 清單的規則為使用者的英文原話

##### Example: 起草內容的撰寫語言

| locale | 新起草或改寫的 context／rules 行 | 未改動的既有行 | review 規則 |
| ------ | ------------------------------- | -------------- | ----------- |
| tw | 繁體中文 | 原文 | 使用者原話 |
| ja | 日文 | 原文 | 使用者原話 |
| en | 英文 | 原文 | 使用者原話 |
| 未設定 | 英文 | 原文 | 使用者原話 |
