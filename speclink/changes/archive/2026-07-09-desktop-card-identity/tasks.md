## 1. 引擎蓋討論建立者

- [x] 1.1 [Red] 於 crates/speclink-core/src/discuss.rs 測試寫失敗測試，實現 D2：引擎替討論蓋 createdBy，比照 change 的 git_identity 機制，使「討論記錄蓋建立者章」成立——discuss new 於有 git 身分時 frontmatter 含 created_by、無身分時省略，show --json 對應帶／不帶 createdBy。驗證：`cargo test -p speclink-core`（Windows 如遇 cdylib 連結問題以 `--lib` 限縮）見紅。
- [x] 1.2 [Green] discuss new 以 util::git_identity 蓋 created_by frontmatter（無身分省略），DiscussionItem 與 --json 以 camelCase createdBy 曝露，apps/desktop/core query 讀取並傳遞。令 1.1 轉綠。驗證：`cargo test -p speclink-core`、`npm test -w apps/desktop`。

## 2. discuss 卡身分與建立者

- [x] 2.1 [Red] 為「討論於看板第 0 欄兩級呈現」寫失敗測試（discussionColumn.test），實現 D1：discuss 卡以 slug 為題、topic 為描述，並記為詞彙受控例外——全卡以 slug 為標題、topic 為卡身描述、帶複製 slug 鈕與建立者顯示。驗證：`npm test -w packages/ui` 見紅。
- [x] 2.2 [Green] DiscussionColumn 全卡改以 slug 為標題、topic 降為描述、加複製 slug 鈕與 createdBy 顯示，DiscussionDrawer 顯示建立者，packages/ui/src/i18n.tsx 增對應鍵。令 2.1 轉綠。驗證：`npm test -w packages/ui`。
- [x] 2.3 於 openspec/LANGUAGE.md 記「discuss 卡以 slug 為題」為受控例外（比照 config.yaml 頁簽先例、範圍限 discuss 卡標題）。驗證：內容審視確認例外條目存在且範圍明確。

## 3. change 卡建立者與關係提示

- [x] 3.1 [Red] 為「看板變更卡呈現建立者與關係提示」寫失敗測試（changeListItem.test），實現 D3：change 卡加建立者頭像，關係徽章以 hover 提示呈現——有 created_by 顯示首字母頭像、無則省略，來自討論／同源以 shadcn Tooltip hover 呈現。驗證：`npm test -w packages/ui` 見紅。
- [x] 3.2 [Green] ChangeCard 加 createdBy 頭像，來自討論／同源指示改用 shadcn Tooltip（取代原生 title），提示內容與詳情抽屜一致。令 3.1 轉綠。驗證：`npm test -w packages/ui`。

## 4. 重構與回歸

- [x] 4.1 [Refactor] 檢視引擎蓋章與卡片變更，去除重複、確認 --json createdBy 為 camelCase、無孤兒 imports，並套用 sharp-edges 稽核確認 created_by 曝露於邊界無型別混淆或非預期洩漏。驗證：`cargo test -p speclink-core`、`npm test -w packages/ui`、`npm test -w apps/desktop` 全綠，且 `npm run build -w apps/desktop` 通過。
