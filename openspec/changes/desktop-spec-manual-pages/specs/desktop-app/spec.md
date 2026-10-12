## MODIFIED Requirements

### Requirement: 桌面 app 呈現 change 與 spec 的清單與內容

桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。spec 的選定 SHALL 開啟規格詳情頁——宿主頁（規格頁或手冊頁）維持不變、頁覆蓋主區，開啟期間內容隨外部檔案變更反映；規格清單 SHALL NOT 提供行內展開。

規格詳情頁 SHALL 採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架（以引用共用，本需求只敘述規格詳情頁放進各欄的內容）。左欄身分區 SHALL 為：capability 名（等寬字型，其後緊跟複製名稱鈕，複製後 SHALL 有已複製回饋）、「正式規格」籤、一行出身「N 條需求 · M 個 scenario」與「最近更新 YYYY-MM-DD」（N 為 `### Requirement:` 段數、M 為各需求段 Scenario 數加總、最近更新取全部 `@trace` 的最大 updated；缺席段整段缺席），其下「溯源」關聯列呈現首顆來源變更籤與其餘數「+N」（來源為零時該列缺席），點擊 SHALL 切到溯源文件。「文件」節點依序 SHALL 為「規格」（徽章為需求數）與「溯源」（徽章為 `@trace` 來源變更去重數），預設開啟「規格」。「狀態」列 SHALL 為「需求」N、「溯源」M、「最近更新」日期與變更名（無 `@trace` 時最近更新列缺席）。主區標題列 SHALL NOT 有變更層動作。

「規格」文件的閱讀卡 SHALL 由上而下為：標題區（標題「規格」、需求數籤、同列右端的「搜尋需求」輸入與「開啟 ▾」）、出處行（`openspec/specs/<capability>/spec.md · 正式規格 · 最近更新 YYYY-MM-DD（<變更>）`，無 `@trace` 時最近更新段缺席）、數字磚「需求」「Scenario」「溯源變更」「最近更新 · <變更>」（值為最近更新的日曆日；無 `@trace` 時該格缺席）、提示框、內文。「搜尋需求」SHALL 以大小寫不敏感子字串比對需求標題、只呈現命中的需求段（Purpose 段恆在），無命中時顯示「沒有符合的需求」，字串 SHALL NOT 持久化。內文 SHALL 先呈現 Purpose（標題「Purpose」為本頁目錄錨點、內文為灰底引言段），其後每條需求一段：「Requirement」小標＋需求標題（本頁目錄錨點）＋右端錨點「來自 <變更> · <日期>」（取該需求段尾的 `@trace` source 與 updated；該變更在封存清單中時錨點可點、點擊開啟該封存變更的已封存詳情頁；無 `@trace` 時錨點缺席）、需求文（`SHALL`／`SHALL NOT`／`MUST`／`MUST NOT` 以等寬灰底標示）、每個 Scenario 為左側細線縮排區塊（標題＋內文，`##### Example` 歸屬所在 Scenario）；命中的第一條需求段 SHALL 預設展開、其餘 SHALL 預設收合為一列（小標、標題、錨點與 Scenario 數籤），點擊列 SHALL 切換展開。本頁目錄（右欄，頁寬 ≥1200px 時顯示）SHALL 列出 Purpose 與全部呈現中的需求標題，點擊 SHALL 捲至該段、捲動時 SHALL 高亮目前段；搜尋過濾後目錄只列命中段。需求段之外的內容（`## Requirements` 下的引言、需求之間或之後的其他 `## ` 區段）SHALL 照原文呈現、SHALL NOT 丟棄：引言在 Purpose 段之後、其他區段在全部需求段之後。辨認結構時 fenced code block 內的行一律是內容，收尾圍欄 SHALL 與開頭同字元且不短於開頭（巢狀圍欄內的 `### Requirement:` 不是需求）。規格全文不含任何 `### Requirement:` 標題時 SHALL 整篇以單一 markdown 檢視退回、無數字磚、無提示框、無本頁目錄錨點，不報錯。

「規格」文件的提示框只放推得出的事實：Purpose 段內文以引擎封存流程的佔位前綴「TBD - created by archiving」起頭時 SHALL 有 warning 提示框「Purpose 待補」，內文指引直接編輯正式規格的 Purpose 段、不走 delta；有需求段缺 `@trace` 時 SHALL 有 note 提示框「N 條需求沒有來源變更」，內文列出這些需求的標題並說明溯源文件與「溯源變更」磚不含它們；兩者皆無時 SHALL NOT 有提示框。

「溯源」文件的閱讀卡 SHALL 為：標題區（標題「溯源」、來源變更數籤；無「開啟 ▾」）、出處行「自全文的 @trace 註解推導，不是檔案 · 封存清單命中者在前、依封存日期升冪」、數字磚「來源變更」「最早 · <變更>」「最近 · <變更>」（分別為最早與最近 `@trace` updated 的日曆日與其 source）、「有來源的需求」（`有 @trace 的需求數 ／ 需求數`；全文無 `@trace` 時數字磚整列缺席），其後列出正式規格全文 `@trace` 來源變更名的去重清單，一列一個變更：封存清單中有對應者為可點列、副標為封存日期、依封存日期升冪在前，點擊 SHALL 開啟該封存變更的已封存詳情頁（規格詳情頁依「詳情頁互斥」關閉、宿主頁 SHALL 維持不變）；無對應者為不可點列、副標「無封存記錄」，排在所有可點列之後、依文件首次出現序。全文無 `@trace` 來源時節點徽章為 0 且卡內顯示「此規格沒有來源變更」。清單與狀態資料的欄位與值 SHALL 與對應 CLI `--json` 輸出一致；自檔案系統衍生的呈現層輔助欄位（如 spec 的最後修改時間、需求數、Purpose 摘要、溯源變更數）不屬此對齊範圍，SHALL NOT 出現在 CLI 輸出對照要求中。

#### Scenario: 顯示 change 清單與狀態

- **WHEN** app 於含多個 active change 的專案啟動
- **THEN** 每個 change 以其名稱與 proposal/tasks 狀態呈現，欄位與值對應 speclink list 與 speclink status 的 --json 輸出

#### Scenario: 選定 change 顯示其文件內容

- **WHEN** 使用者在清單中選定一個 change
- **THEN** app 顯示該 change 的 proposal 內容，並可切換檢視其 design 與 tasks（若存在）

#### Scenario: 選定 spec 以詳情頁顯示需求分段

- **WHEN** 使用者於規格頁點一列含 Purpose、三條需求（各帶兩個 Scenario 與 `@trace`）的規格
- **THEN** 主區換成三欄骨架的規格詳情頁：左欄身分區為等寬名稱＋複製鈕、「正式規格」籤、「3 條需求 · 6 個 scenario」與「最近更新 …」、「溯源」關聯列；「文件」節點為規格 3／溯源 2、「規格」作用中；閱讀卡依序為 Purpose 引言段與三個需求段，第一段展開（「Requirement」小標、標題、右端「來自 <變更> · 日期」錨點、SHALL 關鍵字等寬灰底、Scenario 左線縮排區塊）、第二與第三段收合為一列並帶「2 個 scenario」籤；規格頁清單無行內展開、宿主維持規格頁

#### Scenario: 需求目錄與搜尋需求

- **WHEN** 視窗寬 1600px 時使用者於規格詳情頁閱讀卡標題列的「搜尋需求」輸入只命中一條需求標題的字串
- **THEN** 右欄本頁目錄只列 Purpose 與該條需求，閱讀卡只呈現 Purpose 段與該需求段且該段展開；清空後全部還原（第一條展開、其餘收合）；視窗縮至 1100px 時本頁目錄缺席、閱讀卡照常

#### Scenario: 溯源分頁依封存日期排序

- **WHEN** 使用者開啟一份正式規格全文含三個 `@trace` 來源變更、且三者皆在封存清單中的規格詳情頁並點左欄的「溯源」節點
- **THEN** 節點徽章為 3，閱讀卡標題「溯源」帶籤 3，三列依封存日期升冪排列，每列主行為變更名、副標為封存日期

##### Example: 三個來源變更的排序

| 正式規格全文出現序 | 變更名 | 封存日期 | 呈現位置 |
| --- | --- | --- | --- |
| 1 | drawer-polish | 2026-08-04 | 第 2 列 |
| 2 | spec-archive-drawer | 2026-07-11 | 第 1 列 |
| 3 | desktop-archived-parity | 2026-08-11 | 第 3 列 |

#### Scenario: 點擊溯源列開啟已封存詳情頁且宿主不切頁

- **WHEN** 使用者於手冊頁經出處開啟規格詳情頁，再於「溯源」文件點一個可點列
- **THEN** 規格詳情頁關閉、該封存變更的已封存詳情頁開啟，宿主頁面維持手冊頁；按 Esc 回到手冊頁

#### Scenario: 無封存記錄的來源變更不可點

- **WHEN** 使用者開啟一份正式規格全文含兩個 `@trace` 來源變更、其中一個不在封存清單中的規格詳情頁並點「溯源」節點
- **THEN** 有對應者為第一列可點，無對應者排在其後、副標「無封存記錄」且不可點，點擊無任何效果

#### Scenario: 無溯源來源時溯源分頁空態

- **WHEN** 使用者開啟一份正式規格全文不含任何 `@trace` 註解的規格詳情頁
- **THEN** 身分區出身行無「最近更新」段且無「溯源」關聯列，「狀態」列無最近更新列；「規格」文件的出處行無最近更新段、數字磚無「最近更新」格、各需求段無錨點，note 提示框「N 條需求沒有來源變更」列出全部需求；「溯源」節點徽章為 0、文件無數字磚且顯示「此規格沒有來源變更」

#### Scenario: 需求段之外的內容照樣呈現

- **WHEN** 使用者開啟一份規格詳情頁，其 `## Requirements` 下有一段引言、最後一條需求之後另有 `## 附錄` 區段
- **THEN** 「規格」文件依序為 Purpose 段、引言、需求段、附錄區段，引言與附錄的內文都可見，「附錄」列入本頁目錄

#### Scenario: 無需求標題的規格整篇退回

- **WHEN** 使用者開啟一份不含任何 `### Requirement:` 標題的規格
- **THEN** 「規格」文件整篇以單一 markdown 檢視呈現、無數字磚、無提示框、右欄無本頁目錄、無錯誤訊息

#### Scenario: 複製名稱鈕

- **WHEN** 使用者於規格詳情頁點擊左欄身分區標題旁的複製名稱鈕
- **THEN** capability 名寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 規格文件的出處行、數字磚與狀態列

- **WHEN** 使用者開啟 tray-status-menu（15 條需求、74 個 scenario、9 個 `@trace` 來源變更、最近更新 2026-10-12 來自 tray-panel-restyle）的規格詳情頁
- **THEN** 「規格」文件出處行為 `openspec/specs/tray-status-menu/spec.md · 正式規格 · 最近更新 2026-10-12（tray-panel-restyle）`；數字磚依序為 15「需求」、74「Scenario」、9「溯源變更」、2026-10-12「最近更新 · tray-panel-restyle」；左欄「狀態」列為需求 15、溯源 9、最近更新 2026-10-12；無提示框

#### Scenario: 溯源關聯列切到溯源文件

- **WHEN** 使用者於上述規格詳情頁點左欄「溯源」關聯列的首顆變更籤或「+8」籤
- **THEN** 「溯源」節點作用中、閱讀卡換成溯源文件，出處行為「自全文的 @trace 註解推導，不是檔案 · 封存清單命中者在前、依封存日期升冪」，數字磚為 9「來源變更」、最早與最近的日曆日各附變更名、「15 ／ 15」「有來源的需求」

#### Scenario: Purpose 待補提示框

- **WHEN** 使用者開啟一份 Purpose 段內文以「TBD - created by archiving」起頭的規格詳情頁
- **THEN** 「規格」文件的數字磚之下有 warning 提示框「Purpose 待補」，內文含 `openspec/specs/<capability>/spec.md` 並指引直接編輯 Purpose 段；Purpose 引言段仍呈現佔位原文

#### Scenario: 缺來源變更的需求提示框

- **WHEN** 使用者開啟一份 15 條需求中「選單內容與看板同源」「分區溢出摺疊」兩條沒有 `@trace` 的規格詳情頁
- **THEN** note 提示框標題「2 條需求沒有來源變更」、內文列出兩個標題；兩條需求段無錨點；溯源文件的「有來源的需求」磚為「13 ／ 15」

#### Scenario: 收合的需求段展開

- **WHEN** 使用者於規格詳情頁點第二條（收合中）需求段的列
- **THEN** 該段展開呈現需求文與 Scenario 區塊、列的 `aria-expanded` 為 true，第一段維持展開；再點一次收合；右欄本頁目錄的錨點數不變


<!-- @trace
source: desktop-readonly-detail-pages
updated: 2026-10-10T19:39:25+08:00
-->

---
### Requirement: markdown 內容保留文件結構呈現

桌面 app 渲染 markdown 內容（變更詳情頁的提案／設計／規格文件、討論詳情頁各文件、已封存檢視、規格詳情頁、手冊頁）SHALL 保留來源的文件結構：無序清單 SHALL 顯示列表符號、有序清單 SHALL 顯示編號、段落之間 SHALL 有可辨識的垂直間距、來源中的單一換行 SHALL 呈現為換行。閱讀卡內（變更、討論、已封存、規格四種詳情頁與手冊頁的內文容器）的內文基準字級 SHALL 為 15px、行高為字級的 1.75 倍，任務文件的任務列文字 SHALL 同為 15px；手冊頁內文的 h2 章節標題 SHALL 為 16px 粗體、h3 小節標題 SHALL 為 14px 粗體。排版與內容色彩 SHALL 於淺色與深色主題（跟隨系統偏好）一致生效。

#### Scenario: 清單顯示列表符號與編號

- **WHEN** 檢視含無序清單與有序清單的提案文件
- **THEN** 無序清單項目前顯示列表符號、有序清單項目前顯示編號，清單相對內文有縮排

#### Scenario: 單一換行呈現為換行

- **WHEN** 檢視討論記錄的討論過程文件，其中一輪的 Focus 與 Position 行在來源中各佔一行、以單一換行分隔
- **THEN** 兩行在渲染結果中分行呈現，不塌成同一段連續文字

#### Scenario: 閱讀卡內文字級為 15px

- **WHEN** 檢視變更詳情頁的提案文件與任務文件，再開啟一份規格詳情頁與一頁手冊
- **THEN** 四處閱讀卡內 markdown 內文（含規格詳情頁的需求文與手冊內文）與任務列文字的計算字級皆為 15px、行高為 15px 的 1.75 倍；手冊內文的 h2 為 16px 粗體、h3 為 14px 粗體

#### Scenario: 深色主題下排版一致

- **WHEN** 系統偏好為深色時檢視同一份文件
- **THEN** 列表符號、段落間距、字級與淺色主題一致，內容色彩取自深色 token 且可讀


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
### Requirement: markdown 文件內容行寬有上限

桌面 app 經共用 markdown 渲染呈現的文件內容 SHALL 有固定行寬上限：閱讀卡內（變更、討論、已封存、規格四種詳情頁與手冊頁）的上限 SHALL 為閱讀卡的內容欄寬（卡寬 768px 扣內距），章節標題、數字磚列、提示框、群組卡、輪卡、需求段與內文 SHALL 同欄等寬對齊；容器寬於上限時內容欄 SHALL 水平置中、留白均分兩側。主區寬度改變時內文行寬 SHALL NOT 隨之增長。超過行寬的表格 SHALL 於容器內橫向捲動，版面 SHALL NOT 橫向溢出。

#### Scenario: 主區加寬時行寬不增長且內容欄置中

- **WHEN** 視窗拉寬至 1600px 檢視變更詳情頁的提案文件
- **THEN** 閱讀卡維持 768px 置中、左右留白均分，內文行寬不超過卡的內容欄，章節標題、數字磚列與內文同欄對齊

#### Scenario: 寬表格於容器內橫捲

- **WHEN** 檢視含超過行寬上限的寬表格的文件
- **THEN** 表格於容器內橫向捲動，頁版面不橫向溢出

#### Scenario: 規格詳情頁與手冊頁的行寬同卡欄

- **WHEN** 視窗拉寬至 1600px 依序檢視一份規格詳情頁與一頁手冊
- **THEN** 兩者的閱讀卡維持 768px 置中，需求文、Scenario 區塊、手冊內文的行寬都不超過卡的內容欄，數字磚列與內文同欄對齊，右欄本頁目錄在卡的右側


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
### Requirement: 詳情頁的文件動作

變更詳情頁、討論詳情頁、規格詳情頁與手冊頁的閱讀卡標題列右端 SHALL 提供「開啟 ▾」選單（outline 樣式、與標題同列），對目前文件對應的檔案提供：以預設 App 開啟、在 Finder 顯示（標籤隨平台：Finder／檔案總管／檔案管理員）、複製路徑；選單首列 SHALL 以等寬灰字顯示該檔案相對所在 checkout 根的路徑。文件與檔案的對應 SHALL 為：提案 `openspec/changes/<name>/proposal.md`、設計 `design.md`、任務 `tasks.md`、規格 `specs/<capability>/spec.md`（目前選定的 capability）、審查 `review.md`、驗證 `verify.md`；討論各文件皆為該討論的記錄檔；正式規格為 `openspec/specs/<capability>/spec.md`；手冊頁為 `openspec/manual/<slug>.md`（spec 目錄名依專案設定）。檔案的實際位置 SHALL 與詳情頁讀取內容的位置相同：綁定 worktree 的變更指向 worktree 副本、spec 目錄名依專案設定、已封存的討論指向 `openspec/discussions/archive/<封存日>-<slug>.md`。沒有對應檔案的文件（排程、規格詳情頁的溯源）SHALL NOT 顯示選單。remote 專案無本機 checkout 時選單 SHALL 只有「複製內容」（該文件已載入的原文；未載完時項目停用）且首列路徑缺席；工單文件（審查、驗證）不載原文，此時 SHALL NOT 顯示選單。「以預設 App 開啟」SHALL 只開啟 markdown 文件：傳入的檔名與符號連結的實際指向皆須為 markdown。動作失敗時 SHALL 以既有失敗 toast 語意呈現錯誤，SHALL NOT 彈出錯誤視窗。「開啟 ▾」SHALL NOT 出現在主區標題列，也 SHALL NOT 收進「⋯」選單。

#### Scenario: 提案分頁的開啟選單

- **WHEN** 使用者於本機專案 /work/app 的變更 add-auth 詳情頁提案文件展開閱讀卡標題列的「開啟 ▾」並選「複製路徑」
- **THEN** 選單首列為 `openspec/changes/add-auth/proposal.md`，其後三項為以預設 App 開啟、在 Finder 顯示（macOS）、複製路徑；剪貼簿寫入 `/work/app/openspec/changes/add-auth/proposal.md`

#### Scenario: worktree 中的變更指向 worktree 副本

- **WHEN** 變更 add-auth 綁定 worktree /work/app-wt/add-auth，使用者於其詳情頁任務文件展開「開啟 ▾」並選「在 Finder 顯示」
- **THEN** 選單首列為 `openspec/changes/add-auth/tasks.md`，Finder 顯示 `/work/app-wt/add-auth/openspec/changes/add-auth/tasks.md`（詳情頁所讀的那一份）

#### Scenario: 已封存討論指向封存檔

- **WHEN** 使用者自變更詳情頁的來源討論籤開啟已封存的討論 old-topic（封存檔為 `openspec/discussions/archive/2026-01-03-old-topic.md`）並展開「開啟 ▾」
- **THEN** 選單首列為 `openspec/discussions/archive/2026-01-03-old-topic.md`，複製路徑得到該封存檔的絕對路徑

#### Scenario: 排程分頁無選單

- **WHEN** 使用者點左欄的「排程」節點
- **THEN** 閱讀卡標題列右端不顯示「開啟 ▾」，主區標題列也沒有「開啟 ▾」

#### Scenario: remote 只剩複製內容

- **WHEN** 使用者於無本機 checkout 的 remote 專案開啟討論詳情頁並展開「開啟 ▾」
- **THEN** 選單只有「複製內容」且無路徑首列；選後剪貼簿為該討論記錄的原文

#### Scenario: 在 Finder 顯示失敗以 toast 呈現

- **WHEN** 使用者選「在 Finder 顯示」而該檔案已被外部刪除
- **THEN** 以失敗 toast 呈現 opener 回報的錯誤，詳情頁維持開啟，無錯誤視窗

#### Scenario: 手冊頁的開啟選單

- **WHEN** 使用者於本機專案 /work/app 的手冊頁閱讀〈系統匣選單〉（檔案 `openspec/manual/tray.md`），展開閱讀卡標題列的「開啟 ▾」並選「複製路徑」
- **THEN** 選單首列為 `openspec/manual/tray.md`，其後三項為以預設 App 開啟、在 Finder 顯示（macOS）、複製路徑；剪貼簿寫入 `/work/app/openspec/manual/tray.md`；規格詳情頁的「溯源」文件與 remote 無 checkout 的手冊頁分別無選單、只有「複製內容」


<!-- @trace
source: desktop-detail-doc-tree
updated: 2026-10-12T13:39:16+08:00
-->
