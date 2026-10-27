## 1. 輪卡片：討論輪以卡片呈現（design D1 輪卡片切分：前端行掃描解析 scaffold，不動後端；D2 欄位標籤區塊：行首粗體前綴解析，缺欄位不渲染）

- [x] 1.1 撰寫輪切分 helper 的失敗測試（packages/ui/src/__tests__/discussionDrawer.test.tsx）：scaffold 四輪記錄解析出輪陣列（輪次、mode、日期、欄位名→內文對應）；Position 標籤行後的列點多行歸屬 Position 欄位；來源缺 Ruled out 時欄位對應無該鍵；任一輪標題不符「### Round N — <mode> (<date>)」時回 null。驗證：npm test -w packages/ui 出現預期的紅燈案例。
- [x] 1.2 實作輪切分 helper（packages/ui/src/components/DiscussionDrawer.tsx，前端行掃描解析 scaffold、行首粗體前綴解析且僅認 Focus／Position／Ruled out／Open 四詞白名單）。驗證：1.1 全數轉綠。
- [x] 1.3 撰寫輪卡片渲染的失敗測試（packages/ui/src/__tests__/discussionDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx）：討論過程分頁逐輪成卡（卡頭含 Round N、mode、日期）；欄位以標籤區塊呈現且「**Focus**:」粗體前綴原文不出現；缺席欄位不渲染空標籤；非標準格式整篇以單一 markdown 檢視退回；已封存討論檢視同型斷言。驗證：npm test -w packages/ui 紅燈。
- [x] 1.4 實作輪卡片元件並接入 DiscussionDrawer 討論過程分頁與 ArchivedList 討論檢視（packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/ArchivedList.tsx），行為即規格「討論輪以卡片呈現」。驗證：1.3 全數轉綠。
- [x] 1.5 重構：輪切分與既有 splitDiscussionSections 的行掃描共用套路整理、卡片樣式與既有卡片元件的 border／radius 對齊（packages/ui/src/components/DiscussionDrawer.tsx）。驗證：npm test -w packages/ui 維持全綠，無行為變更。

## 2. 文件容器：markdown 文件內容行寬有上限（design D3 文件容器：prose 行寬上限與容器留白）

- [x] 2.1 撰寫失敗測試（packages/ui/src/__tests__/components.test.tsx）：共用 Markdown 容器不再帶 max-w-none、帶固定行寬上限 class（CJK 可讀行長 35-40 全形字區間定值）。驗證：npm test -w packages/ui 紅燈。
- [x] 2.2 實作文件容器（packages/ui/src/components/Markdown.tsx、apps/desktop/src/index.css）：行寬上限＋一致容器留白，抽屜全螢幕（96vw）時行寬不增長；寬表格維持既有容器內橫向捲動——行為即規格「markdown 文件內容行寬有上限」。驗證：2.1 轉綠；npm test -w apps/desktop 全綠。

## 3. 規格分頁色標：規格分頁 delta 區段以色標呈現（design D4 規格分頁色標區段：delta 標題切分、配色對齊 DeltaBadges）

- [x] 3.1 撰寫 delta 區段切分 helper 的失敗測試（packages/ui/src/__tests__/ui.test.tsx 或 delta 既有測試所在檔）：含 ADDED／MODIFIED／REMOVED／RENAMED 區段標題的 delta spec 切出（delta 種類、區段內文）陣列；不含任何 delta 區段標題時回整篇單段。驗證：npm test -w packages/ui 紅燈。
- [x] 3.2 實作 delta 區段切分 helper（packages/ui/src/delta.ts，與既有 specDeltaCounts 同居）。驗證：3.1 轉綠。
- [x] 3.3 撰寫色標區段渲染的失敗測試（packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx）：規格分頁呈現「新增（綠）／修改（琥珀）／移除（紅）／更名（藍）」色標標頭且色彩 class 與 DeltaBadges 引用同一常數來源；原始「## ADDED Requirements」標題文字不直出；無 delta 標記的規格整篇照常渲染；已封存變更檢視規格分頁同型斷言。驗證：npm test -w packages/ui 紅燈。
- [x] 3.4 實作色標區段元件並接入 RichDetailDrawer 規格分頁與 ArchivedList 規格分頁（packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/DeltaBadges.tsx 抽出共用色彩常數），行為即規格「規格分頁 delta 區段以色標呈現」。驗證：3.3 全數轉綠。

## 4. discuss 技能範本（design D5 skill 範本：assets 單一來源、乾淨樹 golden 再生）

- [x] 4.1 更新 add-round 範本（crates/speclink-core/assets/skills/discuss.md）：範例 Position 改為一句總綰＋隨後「- 」列點展開（每點一行），Document rules 補「Position 超過一句時 SHALL 列點分行」引導；Focus／Ruled out／Open 維持單行慣例。驗證：內容審視——範例區塊與引導句存在、其餘章節未動。
- [x] 4.2 乾淨樹同步三處實例與 golden 再生：確認 git status 乾淨後，以本 repo 新建置的 CLI（非 PATH 上舊拷貝）執行 update 同步 .claude/skills/speclink-discuss/SKILL.md 與 .agents/skills/speclink-discuss/SKILL.md，再跑 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生快照。驗證：git diff 逐行審視僅含 discuss 範本措辭變更；cargo test -p speclink-core --lib 與 render_golden 全綠。

## 5. 收尾驗證（design D6 測試策略：jsdom 結構驗證，視覺以真實視窗驗收）

- [x] 5.1 全量迴歸：npm test -w packages/ui、npm test -w apps/desktop、cargo test -p speclink-core --lib 全綠；CLI 人眼與 --json 輸出零變更（本刀未動任何指令路徑，speclink list --json 抽查形狀不變）。
- [x] 5.2 真實視窗驗收：npm run build -w apps/desktop 後 cargo build --release -p speclink-desktop（先關閉執行中 exe），開 release exe 以實際記錄（desktop-reading-and-tasks-ux 四輪）與本變更的 delta 規格截圖確認——輪卡片分界與欄位標籤、全螢幕行寬上限、規格分頁色標區段；操作前確認使用者未在使用螢幕。驗證：截圖人工核對三項皆呈現。

## 6. 結論欄位標籤化（design D7 結論欄位標籤化：六詞白名單共用欄位解析，背景分頁不動）

- [x] 6.1 撰寫結論欄位標籤化的失敗測試（packages/ui/src/__tests__/discussionDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx）：scaffold 結論的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 以標籤區塊呈現（決定／理由／否決替代案／擱置／記錄去向／下一步）且粗體前綴原文不出現；缺席欄位不渲染空標籤；自由格式結論（無任何白名單欄位）整篇以單一 markdown 檢視退回；已封存討論檢視結論區同型斷言。驗證：npm test -w packages/ui 紅燈。
- [x] 6.2 實作結論欄位標籤化：把輪卡片的欄位解析泛化為共用 helper（內文＋標籤白名單→lead＋欄位對應，splitRounds 改走共用實作），新增結論檢視元件接入 DiscussionDrawer 結論分頁與 ArchivedList 結論區（packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/i18n.tsx 補六個標籤 key）——行為即規格「討論結論以欄位標籤呈現」。驗證：6.1 全數轉綠且既有輪卡片測試維持全綠。
- [x] 6.3 迴歸與真實視窗補驗：npm test -w packages/ui 與 npm test -w apps/desktop 全綠；重建前端與 release exe 後開啟實際記錄（drawer-document-readability 的來源討論）截圖確認結論分頁標籤區塊呈現、背景分頁維持 prose 現狀。驗證：截圖人工核對兩項皆符合。
