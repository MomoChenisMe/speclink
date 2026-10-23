# desktop-app Specification

## Purpose

Speclink 桌面 app 的圖形操作面：直嵌引擎、以本地檔案為真相，呈現變更看板、規格、討論與已封存的清單與內容，並提供動詞操作、任務勾選與拖放排序、搜尋與抽屜檢視、品質站蓋章與驗證標示、自動更新與 CLI 安裝等互動。本 capability 保證圖形介面與 CLI 對同一份資料同語意——GUI 勾任務與 CLI 完成一致、外部變更即時反映——且介面呈現（看板欄位、卡片解剖學、狀態語意色、字型與 markdown 渲染）有統一規則，觸及檔案系統的操作不佔用主執行緒。

## Requirements

### Requirement: 桌面 app 直嵌引擎並以本地檔案為真相
桌面 app SHALL 以 Tauri 殼直接內嵌 speclink-core（非 spawn CLI 子進程），於本地 openspec/ 專案根運作，且 SHALL NOT 改變 fs 模式下 markdown 檔案的真相地位——所有呈現資料由內嵌 core 讀取檔案取得，app 不將任何 change/spec 文件的真相移出檔案系統。

#### Scenario: 於 fs 專案根啟動並讀取本地文件
- **WHEN** 使用者於含 openspec/ 的專案根啟動桌面 app
- **THEN** app 經內嵌 core 讀取本地 markdown 顯示 change 與 spec，未 spawn speclink CLI 子進程，且未寫入或搬移任何文件真相

#### Scenario: 非專案目錄啟動顯示空狀態而非崩潰
- **WHEN** 使用者於不含 speclink 專案標記的目錄啟動桌面 app
- **THEN** app 顯示明確的「非 speclink 專案」空狀態，不崩潰、不產生錯誤彈窗堆疊


<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 桌面 app 呈現 change 與 spec 的清單與內容

桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。spec 的選定 SHALL 開啟規格詳情頁——宿主頁（規格頁或手冊頁）維持不變、頁覆蓋主區，開啟期間內容隨外部檔案變更反映；規格清單 SHALL NOT 提供行內展開。

規格詳情頁 SHALL 採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架（以引用共用，本需求只敘述規格詳情頁放進各欄的內容）。左欄身分區 SHALL 為：capability 名（等寬字型，其後緊跟複製名稱鈕，複製後 SHALL 有已複製回饋）、「正式規格」籤、一行出身「N 條需求 · M 個 scenario」與「最近更新 YYYY-MM-DD」（N 為 `### Requirement:` 段數、M 為各需求段 Scenario 數加總、最近更新取全部 `@trace` 的最大 updated；缺席段整段缺席），其下「溯源」關聯列呈現首顆來源變更籤與其餘數「+N」（來源為零時該列缺席），點擊 SHALL 切到溯源文件。「文件」節點依序 SHALL 為「規格」（徽章為需求數）與「溯源」（徽章為 `@trace` 來源變更去重數），預設開啟「規格」。「狀態」列 SHALL 為「需求」N、「溯源」M、「最近更新」日期與變更名（無 `@trace` 時最近更新列缺席）。主區標題列 SHALL NOT 有變更層動作。

「規格」文件的閱讀卡 SHALL 由上而下為：標題區（標題「規格」、需求數籤、同列右端的「搜尋需求」輸入與「開啟 ▾」）、出處行（`<spec.md 路徑> · 正式規格 · 最近更新 YYYY-MM-DD（<變更>）`：路徑為 spec.md 相對所在 checkout 根的實際位置，spec 目錄名依專案設定、預設為 `openspec/specs/<capability>/spec.md`，remote 專案無本機 checkout 時路徑段缺席；無 `@trace` 時最近更新段缺席）、數字磚「需求」「Scenario」「溯源變更」「最近更新 · <變更>」（值為最近更新的日曆日；無 `@trace` 時該格缺席）、提示框、內文。「搜尋需求」SHALL 以大小寫不敏感子字串比對需求標題、只呈現命中的需求段（Purpose 段恆在），無命中時顯示「沒有符合的需求」，字串 SHALL NOT 持久化。內文 SHALL 先呈現 Purpose（標題「Purpose」為本頁目錄錨點、內文為灰底引言段），其後每條需求一段：「Requirement」小標＋需求標題（本頁目錄錨點）＋右端錨點「來自 <變更> · <日期>」（取該需求段尾的 `@trace` source 與 updated；該變更在封存清單中時錨點可點、點擊開啟該封存變更的已封存詳情頁；無 `@trace` 時錨點缺席）、需求文（`SHALL`／`SHALL NOT`／`MUST`／`MUST NOT` 以等寬灰底標示）、每個 Scenario 為左側細線縮排區塊（標題＋內文，`##### Example` 歸屬所在 Scenario）；命中的第一條需求段 SHALL 預設展開、其餘 SHALL 預設收合為一列（小標、標題、錨點與 Scenario 數籤），點擊列 SHALL 切換展開。本頁目錄（右欄，頁寬 ≥1200px 時顯示）SHALL 列出 Purpose 與全部呈現中的需求標題，點擊 SHALL 捲至該段、捲動時 SHALL 高亮目前段；搜尋過濾後目錄只列命中段。需求段之外的內容（`## Requirements` 下的引言、需求之間或之後的其他 `## ` 區段）SHALL 照原文呈現、SHALL NOT 丟棄：引言在 Purpose 段之後、其他區段在全部需求段之後。辨認結構時 fenced code block 內的行一律是內容，收尾圍欄 SHALL 與開頭同字元且不短於開頭（巢狀圍欄內的 `### Requirement:` 不是需求）。規格全文不含任何 `### Requirement:` 標題時 SHALL 整篇以單一 markdown 檢視退回、無數字磚、無提示框、無本頁目錄錨點，不報錯。

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
- **THEN** 「規格」文件的數字磚之下有 warning 提示框「Purpose 待補」，內文含該規格 spec.md 的實際相對路徑（預設 spec 目錄下為 `openspec/specs/<capability>/spec.md`；remote 無本機 checkout 時不列路徑、改指引直接編輯這份規格檔）並指引直接編輯 Purpose 段；Purpose 引言段仍呈現佔位原文

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


<!-- @trace
source: desktop-spec-manual-pages
updated: 2026-10-12T22:59:10+08:00
-->

---
### Requirement: 桌面 app 提供動詞操作面

桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

詳情頁的動作區 SHALL 以單一「分析」按鈕同時觸發 validate 與 analyze，SHALL NOT 提供獨立的「驗證」按鈕；結果呈現於該 change 詳情頁內、「狀態」卡之下的分析結果卡（分析面板）。分析面板 SHALL 依序呈現：

- 結構驗證列：validate 通過時呈單列通過標示（成功語意配色）；失敗時呈錯誤數並逐條列出錯誤訊息（與 speclink validate 的輸出一致）。
- 維度摘要卡：Coverage、Consistency、Ambiguity、Gaps 四維度各一張摘要卡，維度名以繁體中文呈現（覆蓋度、一致性、模糊度、缺漏），卡上顯示該維度發現數——零發現呈「無問題」（成功語意配色）、非零呈「N 個問題」（警示語意配色）。
- 發現卡：逐條發現各一張卡，呈現嚴重度徽章、來源檔（location）、摘要（summary）與建議行（recommendation），對應 speclink analyze 的 --json 輸出欄位。

分析面板 SHALL 可關閉：再次點按「分析」按鈕或面板的關閉鈕皆 SHALL 收合面板；收合後再點按「分析」SHALL 重新執行並展開。面板狀態 SHALL NOT 跨 change 沿用（切換 change 即清空）。視窗頂欄 SHALL NOT 承載任何操作結果訊息面；看板全域操作的結果回饋 SHALL 依「看板全域操作成功靜默、失敗以 toast 浮層呈現」需求所定語意呈現。

#### Scenario: 分析一鍵呈現驗證列與四維度發現卡

- **WHEN** 使用者於某 change 的詳情頁點按「分析」
- **THEN** 頁內展開分析面板：頂部結構驗證列（通過或錯誤數）、四張繁體中文維度摘要卡（各帶發現數）、逐條發現卡（嚴重度徽章、來源檔、摘要、建議行），內容對應 speclink validate 與 speclink analyze 的 --json 輸出

##### Example: 維度摘要卡的呈現

- **GIVEN** analyze 回傳 Coverage 0、Consistency 0、Ambiguity 18、Gaps 0 個發現

| 維度卡 | 顯示 | 配色語意 |
| ------ | ---- | -------- |
| 覆蓋度 | 無問題 | 成功 |
| 一致性 | 無問題 | 成功 |
| 模糊度 | 18 個問題 | 警示 |
| 缺漏 | 無問題 | 成功 |

#### Scenario: 分析面板可收合

- **WHEN** 分析面板開啟後，使用者再次點按「分析」或點按面板關閉鈕
- **THEN** 面板收合；再次點按「分析」重新執行 validate 與 analyze 並展開面板

#### Scenario: 結構驗證失敗於分析面板呈現錯誤

- **WHEN** 使用者對結構驗證失敗的 change 點按「分析」
- **THEN** 結構驗證列呈現錯誤數並逐條列出 speclink validate 回報的錯誤訊息；維度摘要卡與發現卡照常呈現

#### Scenario: archive 前置未滿足時失敗顯示

- **WHEN** 使用者對尚未滿足歸檔前置的 change 觸發 archive
- **THEN** app 以 toast 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 歸檔清單經衍生快取加速且可重建
桌面 app SHALL 以本地 SQLite 索引快取歸檔（archived）change 的清單資料以加速呈現；此快取 SHALL 為衍生資料——可刪除後由檔案系統真相重建，且 SHALL 帶 schema 版本標記。active change 與 spec 的清單 SHALL NOT 依賴此快取，一律即時經 core 讀取檔案。快取與檔案真相不一致時，檔案真相 SHALL 為準。

#### Scenario: 歸檔清單由快取呈現
- **WHEN** app 呈現歸檔 change 清單
- **THEN** 清單資料自 SQLite 快取讀取，內容與檔案系統中的歸檔目錄一致

#### Scenario: 快取遺失時重建
- **WHEN** SQLite 快取檔不存在或版本不符
- **THEN** app 由歸檔目錄重建快取後呈現清單，不崩潰

#### Scenario: active 清單不經快取
- **WHEN** app 呈現 active change 清單
- **THEN** 清單即時經 core 讀取檔案取得，不讀取 SQLite 快取


<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 前端元件庫與資料源解耦
桌面 app 的呈現元件（change 看板、文件樹、文件檢視）SHALL 封裝為與資料源解耦的共用元件庫：元件經注入的 data adapter 取得資料，adapter 介面 SHALL 以領域語彙（列出 change、列出 spec、取得文件、執行動詞）定義，SHALL NOT 直接依賴 Tauri 專屬全域。桌面 app SHALL 提供以內嵌 core 為後端的 adapter 實作。

#### Scenario: 桌面注入 core adapter 渲染看板
- **WHEN** 桌面 app 以其 core-backed adapter 提供 change 列表
- **THEN** 共用看板元件據此渲染，元件本身未引用任何 Tauri 專屬全域

#### Scenario: adapter 介面以領域語彙定義
- **WHEN** 檢視 adapter 介面定義
- **THEN** 其方法以 change/spec/document/verb 領域語彙表述，使非 Tauri 後端（如後續 HTTP 後端）可提供另一實作而元件不變

<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 外部變更即時反映

桌面 app SHALL 監看目前專案的 openspec/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情頁、討論詳情頁與已封存頁 SHALL 在短時間內（秒級）自動更新呈現；已開啟檢視中「已載入的內容」——任務清單勾選狀態、proposal／design／specs 文件原文、meta 開工歸屬、討論記錄各分頁——SHALL 重載至磁碟現況，SHALL NOT 要求重啟、重開詳情頁或任何 app 內操作。使用者互動進行中（任務勾選寫回、拖曳排序）時外部觸發的內容重載 SHALL 讓路，互動結束後 SHALL 補一次重載至磁碟現況——SHALL NOT 打斷或蓋掉進行中的操作；重載回應交錯時 SHALL 以最新一次為準，較舊回應 SHALL NOT 覆蓋較新內容。app 內操作（勾任務、拖曳、動詞）後 SHALL 重載受影響的已載入內容——含任務清單與 meta。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

#### Scenario: 外部勾選任務後看板自動更新

- **WHEN** 桌面 app 執行中，於外部終端執行 speclink task done 勾掉某 change 的一項任務
- **THEN** 數秒內該 change 的看板卡片任務數與進度條更新；詳情頁若開啟，標頭計數與任務清單中該項的核取方塊皆同步至磁碟狀態，全程無任何 app 內操作

#### Scenario: 外部蓋開工章後詳情頁出現開工歸屬

- **WHEN** 某 change 的詳情頁開啟中，於外部終端執行 speclink in-progress add 該 change
- **THEN** 數秒內詳情頁出現開工者與開工日，無需重開詳情頁

#### Scenario: 外部推進討論後頁內容更新

- **WHEN** 某討論的詳情頁開啟中，於外部終端執行 speclink discuss add-round 該討論
- **THEN** 數秒內詳情頁的回合分頁出現新回合內容，標頭回合數與其一致

#### Scenario: 互動進行中外部重載讓路

- **WHEN** 使用者正在拖曳某 change 的任務（尚未放開），外部寫者同時修改該 change 的文件
- **THEN** 拖曳互動不被打斷、拖曳視覺不重置；放開完成後數秒內，頁內容重載至磁碟現況

#### Scenario: 外部新增與歸檔反映到清單

- **WHEN** 於外部以 CLI 建立新 change，隨後將另一 change 歸檔
- **THEN** 數秒內看板出現新 change 卡片，被歸檔者自看板消失並出現於已封存頁

#### Scenario: 監看不可用時功能照常

- **WHEN** 檔案監看因環境因素無法建立
- **THEN** app 啟動與所有查詢、操作照常運作，錯誤僅記錄於日誌，畫面無錯誤彈窗堆疊


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 看板欄位由生命週期標記驅動

看板欄位 SHALL 依下列優先序判定：任務全完成（總數大於 0 且完成數等於總數）＝已就緒；meta 含 started_at 或任務完成數大於 0＝進行中；其餘＝提案中。剛完成 propose（有任務、全未勾、未標記開工）的 change SHALL 顯示於提案中欄。詳情頁 SHALL 於 meta 含 started_at 時顯示開工者與開工日（started_by、started_at）；meta 無 started_at 時 SHALL NOT 顯示開工資訊——即使該 change 因任務進度列於進行中欄（派生管顯示，歸屬缺席維持缺席）。此判定 SHALL 對 fs 與 remote 資料來源一致：remote 清單 payload 的 startedAt SHALL 進入看板與系統匣的欄位推導，SHALL NOT 以任務完成數替代開工判定（完成數大於 0 的 fallback 保留，涵蓋繞過工具的寫入路徑）。

#### Scenario: 未開工的 change 留在提案中

- **WHEN** 某 change 的 tasks.md 含 28 項任務全未勾、meta 無 started_at
- **THEN** 看板將其顯示於「提案中」欄，卡片任務數為 0/28

#### Scenario: 標記開工後移入進行中

- **WHEN** 對上述 change 執行 speclink in-progress add 後看板更新
- **THEN** 該卡片移至「進行中」欄，詳情頁標頭顯示開工者與開工日

#### Scenario: 無章而有任務進度列於進行中

- **WHEN** 某 change 的 meta 無 started_at，其 tasks.md 經任意途徑（如編輯器直接修改後 git pull 或本機儲存）成為 3/28 已勾，看板刷新
- **THEN** 該卡片顯示於「進行中」欄，詳情頁不顯示開工者與開工日

#### Scenario: remote 已開工零進度列於進行中

- **WHEN** remote 連線的 change 清單中某 change 帶 startedAt 且任務進度為 0/15，看板與系統匣刷新
- **THEN** 該卡片顯示於「進行中」欄、系統匣列於進行中分區——不因完成數為 0 而落回提案中

##### Example: 欄位判定矩陣

| meta started_at | 任務進度 | 看板欄 |
| --------------- | -------- | ------ |
| 無 | 0 任務 | 提案中 |
| 無 | 0/28 | 提案中 |
| 無 | 3/28 | 進行中（詳情頁無開工資訊） |
| 有 | 0/28 | 進行中 |
| 有 | 13/28 | 進行中 |
| 無 | 28/28 | 已就緒（全完成優先） |
| 有 | 28/28 | 已就緒 |


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 任務清單拖放排序與自動重編號

詳情頁任務分頁的每個任務列 SHALL 於列首提供拖曳把手，使用者拖曳把手放開後 SHALL 一次到位完成排序並回寫 tasks.md——僅 checkbox 行被搬移，群組標題與其他行 SHALL NOT 被搬動。寫回時 SHALL 重寫受影響任務的編號前綴：文字以「數字.數字」＋空白開頭、且位於帶「數字.」前綴標題之群組下的任務行，其前綴 SHALL 重寫為「群組編號.組內序」；不符該樣式的任務行與所有非 checkbox 行 SHALL 逐字元保留。拖曳 SHALL 僅由把手觸發——點擊核取方塊或任務文字 SHALL NOT 啟動拖曳。群組標題 SHALL 參與拖曳中的讓位序列——被越過任務的讓位視覺 SHALL NOT 穿越其群組標題（群組歸屬在拖曳預覽中保持可讀）；群組標題為「組界槽」落點——把手放開於群組標題上時：來自標題上方的任務 SHALL 成為該群組的第一個任務；來自標題下方的任務 SHALL 移到標題之前、成為上一群組的末任務（雙向皆重編號；標題該側無任務可錨定時不提供落點）。逐格上下移動按鈕 SHALL NOT 再提供。封存唯讀檢視 SHALL NOT 渲染把手。

#### Scenario: 組內拖放後編號重寫

- **WHEN** 使用者把群組 1 的首任務拖到該群組末位放開
- **THEN** tasks.md 該群組的 checkbox 行依新順序排列，編號前綴依序重寫為 1.1、1.2、…（被拖任務取得末位編號），群組標題與其他群組逐字元不變

##### Example: 組內移動重編號

| 拖放前（群組 1） | 拖放後（把 1.1 甲拖到末位） |
| ---------------- | --------------------------- |
| - [ ] 1.1 甲 | - [x] 1.1 乙 |
| - [x] 1.2 乙 | - [ ] 1.2 丙 |
| - [ ] 1.3 丙 | - [ ] 1.3 甲 |

#### Scenario: 跨群組拖放取得新群組編號

- **WHEN** 使用者把群組 1 的末任務拖到群組 2 首任務之上放開（落點為該任務）
- **THEN** 該任務插於其後、前綴重寫為 2.2，群組 2 其餘任務依序後移（2.3…），群組 1 剩餘任務依序重排（1.1、1.2…）

#### Scenario: 拖放到群組標題成為組首

- **WHEN** 使用者把群組 1 的任務拖放到「## 2.」群組標題上
- **THEN** 該任務成為群組 2 的第一個任務（前綴重寫為 2.1），原組首任務後移為 2.2，群組 1 剩餘任務依序重排

##### Example: 標題落點成組首

| 拖放前 | 拖放後（把 1.2 乙拖到「## 2. 後段」標題上） |
| ------ | ------------------------------------------- |
| ## 1. 前段：1.1 甲、1.2 乙 | ## 1. 前段：1.1 甲 |
| ## 2. 後段：2.1 丙、2.2 丁 | ## 2. 後段：2.1 乙、2.2 丙、2.3 丁 |

#### Scenario: 組首任務拖回標題之前回到上一群組末位

- **WHEN** 使用者把群組 2 的第一個任務（原自群組 1 拖入）拖放到「## 2.」群組標題上
- **THEN** 該任務移到標題之前、成為群組 1 的末任務並重編號（如群組 1 原有 5 項則成為 1.6），群組 2 其餘任務依序前移補位

##### Example: 標題落點雙向

| 操作 | 結果 |
| ---- | ---- |
| 把 1.2 乙拖到「## 2. 後段」標題上（來自上方） | 乙成為 2.1 組首 |
| 再把 2.1 乙拖回「## 2. 後段」標題上（來自下方） | 乙回到群組 1 末位（1.2） |

#### Scenario: 拖曳讓位不穿越群組標題

- **WHEN** 使用者拖曳任務向下越過相鄰群組的首任務（拖曳進行中、尚未放開）
- **THEN** 該首任務的讓位視覺仍位於其群組標題之下方——SHALL NOT 出現在標題上方（上一群組的區域內）

#### Scenario: 無編號前綴的任務文字不被改寫

- **WHEN** 使用者拖動一個文字不以「數字.數字」開頭的任務到新位置
- **THEN** 該任務僅位置改變、文字逐字元保留，其他符合樣式的任務仍正常重編號

#### Scenario: 點擊核取方塊不誤觸拖曳

- **WHEN** 使用者在任務核取方塊上按下並於拖曳啟動閾值內放開
- **THEN** 勾選狀態切換且任務順序不變，無拖曳視覺出現

#### Scenario: 封存唯讀檢視無把手

- **WHEN** 使用者展開已封存變更的任務分頁
- **THEN** 任務列無拖曳把手、無法拖動，核取方塊維持不可點擊


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 監看根解析與專案探索一致

桌面 app 的檔案監看 SHALL 依附於經專案探索解析出的專案根：啟動時 SHALL 自工作目錄向上探索 speclink 專案（與查詢指令同一探索邏輯、同一結果），並監看該專案實際的 spec 目錄樹；SHALL NOT 以未經探索的啟動工作目錄拼接固定目錄名作為監看目標。以非專案根的工作目錄啟動（如自檔案總管雙擊執行檔）時，外部變更的自動刷新 SHALL 照常生效。向上探索不到任何 speclink 專案時 SHALL 維持既有降級行為：app 照常提供其餘功能、僅失去自動刷新、錯誤記錄於日誌。

#### Scenario: 自非專案根 cwd 啟動後外部開工即時反映

- **WHEN** 桌面 app 以專案內子目錄（如建置輸出目錄）為工作目錄啟動，隨後於外部終端對某 change 標記開工（其 metadata 文件寫入 started_at）
- **THEN** 數秒內看板該卡片自「提案中」欄移至「進行中」欄，全程無重啟或任何 app 內操作

#### Scenario: 監看目標尊重非預設 spec 目錄名

- **WHEN** 專案使用非預設的 spec 目錄名，桌面 app 於該專案內啟動，外部寫者修改該 spec 目錄下的文件
- **THEN** 看板與詳情於數秒內自動更新——監看目標為探索出的實際 spec 目錄，而非寫死的目錄名

#### Scenario: 探索不到專案時維持降級行為

- **WHEN** 桌面 app 於任何 speclink 專案之外的目錄啟動
- **THEN** app 照常啟動與運作，僅無自動刷新；監看建立失敗只記錄於日誌，畫面無錯誤彈窗

<!-- @trace
source: desktop-watcher-root-fix
updated: 2026-07-06
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
-->

---
### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，欄內分上下兩區同屏呈現：上區為討論中（active）全卡清單，下區為欄底的「已轉出」常駐收合列；SHALL NOT 以互斥檢視切換兩者。分區判準 SHALL 為「引擎會自動收走的討論才收合」：已轉出（promoted）、已有結論（concluded 為 true）且未保留（hold 非 true）的討論收進欄底收合列；已轉出但尚無結論（concluded 為 false）的討論 SHALL 留在上區為全尺寸卡；已轉出、已有結論但保留在途（hold 為 true）的討論 SHALL 同樣留在上區為全尺寸卡。討論清單資料的 concluded 欄位缺席（向後相容路徑）時 SHALL 退回既有判準——promoted 一律收進收合列，SHALL NOT 把缺席當成尚無結論；hold 欄位缺席時 SHALL 視同未保留，SHALL NOT 把缺席當成保留。看板拖排落點清單 SHALL 與上區卡集共用同一判準（上區卡集與可拖集恆等）。

討論中區 SHALL 顯示 status 為 open 或 concluded 的討論、已轉出但尚無結論的討論、以及已轉出且保留在途的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題（等寬字型）、topic 為卡身描述；複製 slug 鈕 SHALL 行內尾隨於標題最後一個字元後（版面規則見「看板卡片統一解剖學」）；建立者（createdBy，缺席時省略）SHALL 以頭像圓點呈現且 hover 顯示全名；輪數與建立時間 SHALL 並排於卡底 meta 列，狀態 chip 保留；已轉出但尚無結論的卡 SHALL 帶「已轉出・尚無結論」狀態標；已轉出且保留在途的卡 SHALL 帶「已轉出・保留中」狀態標（英文介面為「Promoted · on hold」），與「已轉出・尚無結論」同為 warning 語意色；promoted 且 concluded 為 false 的卡不論 hold 值 SHALL 標「已轉出・尚無結論」。open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）；已轉出但尚無結論的卡與已轉出且保留在途的卡 SHALL NOT 提供任何動詞按鈕（前者結論未寫；後者的收尾由使用者以 CLI 的 discuss archive 明示解除）。上區全卡 SHALL NOT 列出衍生變更清單（衍生進度於討論詳情頁檢視）。

當存在至少一筆已有結論且未保留的 promoted 討論時，欄底 SHALL 呈現「已轉出 N ›」收合列（白底、上緣細線、右端箭頭依「收合時一律 ›、展開時箭頭指向展開出來的內容」：收合時為 ›，展開時朝上、指向展開於該列上方的 promoted 細列，SHALL NOT 朝下；N 為已有結論且未保留的 promoted 計數，預設收合）；點按 SHALL 就地展開 promoted 細列清單（白底細框列），再點按即收合；桌面 app SHALL 把展開狀態存於 app 本機並跨啟動保留，SHALL NOT 寫入任何專案目錄。無任何已有結論且未保留的 promoted 討論時收合列 SHALL 缺席。

promoted 細列 SHALL 以討論 slug 為首行（等寬字型）且帶複製 slug 鈕、topic 為次行描述，其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

slug 為題與複製鈕（討論全卡標題、promoted 細列首行）屬 openspec/LANGUAGE.md 明載的受控例外。討論欄的計數徽章 SHALL 顯示上區全卡數（open、concluded、已轉出但尚無結論、已轉出且保留在途）。當討論欄上區無任何全卡但存在已有結論且未保留的 promoted 討論時，SHALL NOT 顯示「尚無討論」空狀態（欄底收合列已傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by 與 created）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄顯示兩張全卡，各以 slug 為標題（等寬字型）、topic 為描述，複製 slug 鈕行內尾隨於標題末字元後，建立者呈頭像圓點且 hover 顯示全名、卡面無全名直出文字，卡底 meta 列並排「3 輪」與建立時間；open 卡無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，欄底無「已轉出」收合列

#### Scenario: 未結論的已轉出討論留在上區並帶標示

- **WHEN** 討論欄存在一筆已轉出但尚無結論（promoted 且 concluded 為 false）的討論與一筆已轉出且已有結論且無 hold 的討論
- **THEN** 前者呈上區全尺寸卡、帶「已轉出・尚無結論」狀態標、無任何動詞按鈕；後者收進欄底「已轉出 1」收合列；欄計數徽章顯示 1（上區卡數）

#### Scenario: 保留中的已轉出討論留在上區並帶標示

- **WHEN** 討論欄存在一筆已轉出、已有結論且 hold 為 true 的討論（promoted_to 含兩個已封存變更與一個進行中變更），與一筆已轉出、已有結論且 hold 為 false 的討論
- **THEN** 前者呈上區全尺寸卡、帶「已轉出・保留中」狀態標、無任何動詞按鈕、卡面不列衍生變更清單；後者收進欄底「已轉出 1」收合列；欄計數徽章顯示 1（上區卡數）；拖排落點清單含前者不含後者

#### Scenario: concluded 資料缺席時退回既有分區

- **WHEN** 討論清單資料無 concluded 欄位（向後相容路徑），存在一筆 promoted 討論
- **THEN** 該討論收進欄底「已轉出」收合列（既有行為），上區不出現「已轉出・尚無結論」標示

#### Scenario: hold 資料缺席時退回既有分區

- **WHEN** 討論清單資料無 hold 欄位（舊 server），存在一筆已轉出且已有結論的討論
- **THEN** 該討論收進欄底「已轉出」收合列，上區不出現「已轉出・保留中」標示

#### Scenario: 複製討論 slug

- **WHEN** 點討論全卡或已轉出細列的複製鈕
- **THEN** 該討論的 slug 寫入剪貼簿並短暫顯示已複製回饋，且不開啟討論詳情頁

#### Scenario: 欄底收合列就地展開與收合

- **WHEN** 討論欄存在一筆 active 討論與一筆已有結論且無 hold 的 promoted 討論
- **THEN** 上區顯示 active 全卡、欄底呈現「已轉出 1」收合列（預設收合）；點按收合列就地展開 promoted 細列（active 全卡維持可見），再點按即收合；欄計數徽章全程顯示 1（上區卡數）；展開後重啟桌面 app，收合列仍為展開

#### Scenario: 無已轉出討論時收合列缺席

- **WHEN** 討論欄無任何已有結論且無 hold 的 promoted 討論
- **THEN** 欄底不顯示「已轉出」收合列

#### Scenario: 僅有已轉出討論時討論中區不顯空狀態

- **WHEN** 討論欄上區無任何全卡、但存在至少一筆已有結論且無 hold 的 promoted 討論
- **THEN** 討論中區不顯示「尚無討論」文案，欄底收合列傳達存在已轉出討論

#### Scenario: 已轉出細列的 slug 首行與子變更樹配色

- **WHEN** 展開欄底收合列，一筆已有結論且無 hold 的 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
- **THEN** 該細列首行為 slug（等寬字型）帶複製鈕、次行為 topic，其下兩列樹狀子項——前者帶 ├ 前綴與「提案中」chip（呈提案中欄的 teal 濃度）、後者帶 └ 前綴與「已封存」chip（中性色）

##### Example: chip 階段派生與配色矩陣

| promoted_to 子變更的所在 | 階段標示 | chip 配色 |
| ------------------------ | -------- | --------- |
| active 清單，無 started、0/24 | 提案中 | 提案中欄的 teal 濃度 |
| active 清單，有 started、13/24 | 進行中 | 進行中欄的 teal 濃度 |
| active 清單，24/24 | 已就緒 | 已就緒欄的 teal |
| 封存清單（dated name 尾碼命中） | 已封存 | 中性色 |
| 兩清單皆無 | 已刪除（討論維持已轉出） | destructive 加刪除線 |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作

#### Scenario: 欄底收合列箭頭指向展開內容

- **WHEN** 看板討論欄底部有「已轉出 2 ›」收合列，使用者點按展開、再點按收合
- **THEN** 展開後 2 筆 promoted 細列出現在收合列上方、箭頭朝上；收合後細列消失、箭頭恢復 ›


<!-- @trace
source: tray-panel-content-priority
updated: 2026-10-13T12:56:33+08:00
-->

---
### Requirement: 已封存頁含討論節

已封存頁 SHALL 以頁標題區（標題「已封存」、灰字說明、右端全圓搜尋框）與一張白底細框列表卡呈現，列表卡頂部為卡片標頭式分頁「變更」與「討論」兩節，預設顯示「變更」；分頁標籤 SHALL 各帶該節目前（搜尋過濾後）筆數徽章。兩節皆為列式清單（一列一筆，列 anatomy 見「規格與封存卡片收合資訊」），點擊列開啟唯讀的已封存詳情頁（檢視行為見「已封存項目以詳情頁檢視」）；討論節 SHALL 列出封存討論（日期＋slug 標題＋topic 描述），SHALL NOT 提供任何寫入動詞。頁標題區的搜尋框 SHALL 同時過濾兩節——身處任一子頁籤時，另一子頁籤的徽章 SHALL 反映其命中數。兩子頁籤的換頁頁碼 SHALL 互相獨立。討論清單資料缺席（向後相容路徑）時子頁籤列 SHALL 缺席、僅顯示變更清單。隨最後一個子變更歸檔而自動封存的討論、與經 GUI 或 CLI 手動封存的討論 SHALL 一致地出現於討論節。已封存討論詳情頁的分頁 SHALL 使用「背景」「討論過程」「結論」。

#### Scenario: 切換子頁籤直達討論清單

- **WHEN** 已封存頁含 40 筆封存變更與 26 筆封存討論，使用者點列表卡頂部的「討論」分頁
- **THEN** 直接顯示封存討論列第 1 頁（建立日期最新在前），無需捲過任何變更列；分頁列作用中為主色底線

#### Scenario: 搜尋時兩子頁籤徽章顯示各自命中數

- **WHEN** 使用者於「變更」子頁籤輸入僅命中 3 筆封存討論、0 筆封存變更的搜尋字串
- **THEN** 「變更」徽章顯示 0 且清單顯示無結果空狀態；「討論」徽章顯示 3，切換後即見該 3 筆

#### Scenario: 自動封存的討論現身討論節

- **WHEN** 某已轉出討論的最後一個子變更被歸檔（觸發引擎的討論自動封存）且看板更新
- **THEN** 該討論自看板討論欄消失，已封存頁「討論」子頁籤出現該筆，搜尋其 topic 可命中


<!-- @trace
source: desktop-readonly-detail-pages
updated: 2026-10-10T19:39:25+08:00
-->

---
### Requirement: GUI 勾任務與 CLI 完成語意一致

桌面看板勾選任務為完成 SHALL 產生與 speclink task done 相同的檔案效果：tasks.md 該任務勾章、touched-files 記錄（有未被先前任務認領的 git dirty 檔時追加，無則不追加——與 CLI 同語意）、該 change 首次有任務完成且 meta 無 started_* 時蓋開工章（started_at 為當日、started_by 依 git 身分可得性、started_with 缺席）。對已完成任務的重複完成請求 SHALL 視為冪等成功，SHALL NOT 寫入任何檔案、SHALL NOT 對使用者報錯。取消勾選與拖曳排序 SHALL 僅寫 tasks.md，SHALL NOT 寫入 meta 或 touched 記錄。

#### Scenario: 勾選首任務蓋章並移欄

- **WHEN** 使用者於看板勾選某 meta 無 started_* 的 change 的第一項任務
- **THEN** tasks.md 該任務成 [x]，.openspec.yaml 新增 started_at（git 身分可得時含 started_by），看板刷新後卡片移入「進行中」欄且詳情頁顯示開工列

#### Scenario: 取消勾選不動 meta 與 touched

- **WHEN** 使用者取消勾選一項已完成任務
- **THEN** 僅 tasks.md 該行標記變為 [ ]；.openspec.yaml 與 touched 記錄逐字元不變

#### Scenario: 拖曳排序不觸發完成語意

- **WHEN** 使用者拖曳任務改變順序（含跨群組重編號）
- **THEN** 僅 tasks.md 變動；.openspec.yaml 與 touched 記錄逐字元不變


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 看板搜尋過濾卡片

看板 SHALL 於頁標題區右端提供搜尋輸入（全圓、寬 280px、帶搜尋圖示）與同列同高的篩選圖示鈕，兩者位於欄位上方，SHALL NOT 佔用第二列版面。輸入含非空白內容時 SHALL 呈現清除鈕（點按清空字串且輸入保持聚焦）與即時命中數（過濾後各欄卡片總數）；快捷鍵（macOS 為 Cmd+F、其他平台為 Ctrl+F）SHALL 聚焦搜尋輸入。

輸入含非空白內容時，看板 SHALL 僅顯示比對命中的卡片。比對 SHALL 含三層，任一層命中即顯示：

- 欄位子字串：變更卡以名稱與摘要比對、討論卡以主題與 slug 比對；去除頭尾空白、不分大小寫、以子字串命中（與已封存頁的搜尋規則一致）。
- 名稱層模糊比對：變更卡名稱與討論卡 slug 另以 subsequence 比對（查詢字元依序出現於目標即命中）；摘要與主題 SHALL NOT 套用模糊比對。
- 全文比對：非空查詢 SHALL 經桌面 core 的 workspace 全文查詢，以不分大小寫子字串比對 active 變更的 artifacts（提案、設計、任務、delta 規格）與 active 討論記錄全文；查詢以去抖觸發、回應以 latest-wins 收斂；查詢失敗 SHALL 靜默退回欄位比對，SHALL NOT 阻斷輸入或顯示錯誤。

欄位子字串命中 SHALL 於卡片上以高亮標示命中原文；全文命中的卡片 SHALL 於卡身呈現 snippet 行——命中的 artifact 名與命中前後文裁切、命中原文高亮（每卡取首個命中）；僅模糊命中而無連續子字串時 SHALL 顯示卡片但不高亮。

篩選 SHALL 收於篩選開關鈕之後（預設不佔版面）：點按開關 SHALL 於其下方彈出篩選面板，面板內呈三個篩選維度選單——建立者（active 變更與討論的 createdBy 去重清單）、建立時間（近 7 天、近 30 天、更早三擇一；變更卡與討論卡皆以 created 日期比對）、來源討論（promoted_to 非空的討論清單；選定後顯示該討論卡自身與來源討論含該 slug 的變更卡）。再點按開關、點擊面板外或按 Esc SHALL 關閉面板；關閉面板 SHALL NOT 清除已啟用的篩選（過濾持續生效）。各維度 SHALL 可於面板內單獨清除；存在啟用中篩選時面板 SHALL 提供全部清除、且開關鈕 SHALL 呈現啟用計數。多個維度與搜尋字串 SHALL 以 AND 交集過濾。

各欄欄頭計數 SHALL 等於該欄過濾後的卡片數。清空輸入（或輸入僅含空白）且無啟用篩選時 SHALL 還原全部呈現。搜尋字串與篩選狀態 SHALL NOT 持久化，且 SHALL 與已封存頁的搜尋字串各自獨立。

#### Scenario: 輸入關鍵字即時過濾各欄卡片

- **WHEN** 使用者於看板搜尋輸入鍵入非空白字串
- **THEN** 各欄僅顯示任一比對層命中的卡片，命中的欄位字段以高亮標示，且各欄欄頭計數等於該欄過濾後卡片數

##### Example: 比對規則

- **GIVEN** 提案中欄有變更卡 desktop-acp-agent（摘要含「桌面版」）與 web-role-views（摘要含「情境 1」）；討論欄有卡片主題「GUI 勾任務自動蓋開工章」

| 輸入 | 提案中欄顯示（計數） | 討論欄顯示（計數） | Notes |
| ---- | -------------------- | ------------------ | ----- |
| desktop | desktop-acp-agent（1） | 無（0） | 名稱子字串命中並高亮 |
| 桌面 | desktop-acp-agent（1） | 無（0） | 摘要命中並高亮 |
| &nbsp;GUI&nbsp; | 無（0） | GUI 勾任務自動蓋開工章（1） | 去頭尾空白、不分大小寫 |
| dta | desktop-acp-agent（1） | 無（0） | 名稱 subsequence 模糊命中、不高亮 |
| （清空） | 兩張全顯（2） | 全顯（1） | 還原全部 |

#### Scenario: 全文命中呈現 snippet

- **WHEN** 使用者輸入僅出現於某變更 design.md 內文（不出現於名稱與摘要）的字串
- **THEN** 該變更卡顯示且卡身呈現 snippet 行（命中 artifact 名＋命中前後文、命中原文高亮）；欄頭計數含該卡

#### Scenario: 全文查詢失敗靜默退回欄位比對

- **WHEN** workspace 全文查詢因 IPC 錯誤失敗
- **THEN** 看板維持欄位比對的過濾結果，不顯示錯誤訊息、搜尋輸入不受阻斷

#### Scenario: 篩選收於開關鈕的彈出面板

- **WHEN** 使用者點按篩選開關鈕，於面板啟用建立時間「近 7 天」後再點按開關關閉面板
- **THEN** 開啟前不呈現任何篩選控制；面板開啟時三個維度選單可見；關閉後面板消失、過濾持續生效且開關鈕呈現啟用計數 1

#### Scenario: 面板內全部清除

- **WHEN** 使用者於篩選面板啟用兩個維度後點按全部清除
- **THEN** 所有維度回到未啟用、看板還原（僅剩搜尋字串過濾）、開關鈕的啟用計數消失

#### Scenario: 篩選 chip 與搜尋字串交集過濾

- **WHEN** 使用者啟用建立時間「近 7 天」chip 且輸入非空白字串
- **THEN** 僅顯示 created 日期在近 7 天內且命中該字串的卡片；單獨清除該 chip 後回到僅以字串過濾

#### Scenario: 來源討論篩選

- **WHEN** 使用者於來源討論 chip 選定某 promoted 討論
- **THEN** 看板僅顯示該討論卡自身與來源討論含該 slug 的變更卡

#### Scenario: 快捷鍵聚焦與清除鈕

- **WHEN** 使用者按下 Cmd+F（macOS）或 Ctrl+F（其他平台）
- **THEN** 搜尋輸入取得聚焦；輸入非空時搜尋列呈現清除鈕與命中數，點清除鈕後字串清空、還原全部且輸入保持聚焦

#### Scenario: 無命中時顯示空欄與零計數

- **WHEN** 使用者輸入無任何卡片命中的字串
- **THEN** 各欄顯示為空、欄頭計數為 0，欄位結構維持呈現且不顯示錯誤

#### Scenario: 過濾狀態下卡片互動不受影響

- **WHEN** 過濾狀態下使用者點擊卡片、或拖曳已就緒的變更卡至封存落點
- **THEN** 詳情頁正常開啟、封存流程正常觸發，行為與未過濾時一致

#### Scenario: 搜尋字串與篩選不跨啟動保留且與已封存頁獨立

- **WHEN** 使用者於看板輸入字串並啟用任一篩選 chip 後切至已封存頁，再重啟 app
- **THEN** 已封存頁的搜尋輸入不含看板字串；重啟後看板為未過濾狀態、搜尋輸入為空、無啟用中的篩選 chip


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 側欄導覽結構

桌面 app 的左側 SHALL 分兩層:全域的圖示列(約 56px)與每專案的專案欄(固定 200px,SHALL NOT 可收合)。圖示列自左側標題列下方起由上而下 SHALL 為品牌標記、每個開著的專案一個方塊、「新增專案」的「＋」、底部的「設定」;專案欄自左側標題列下方起由上而下 SHALL 為變更、已封存、規格、手冊,底部 SHALL 固定為專案設定,中間為彈性空間。「設定」SHALL NOT 出現在專案欄,其入口為圖示列底部的齒輪。專案欄 SHALL NOT 含備忘項、SHALL NOT 呈現常駐 app 版號文字;app 版號的呈現位置 SHALL 為應用程式設定的更新頁版本卡與關於頁品牌卡。已封存導覽項 SHALL 帶封存變更數量徽章,數字 SHALL 隨封存清單變動即時更新;其無障礙標籤 SHALL 為「已封存」。手冊導覽項的無障礙標籤 SHALL 為「手冊」,點擊進入手冊頁(行為見 desktop-manual-page)。導覽項點擊 SHALL 為單純切頁——切至對應頁並以視覺高亮(主色淡底、主色字)標示現行項,SHALL NOT 具再次點擊返回看板的 toggle 行為。左側頂端 48px SHALL 為一條連續的標題列(圖示列與專案欄共用、同側欄底色、48px 內無直向分隔線,下緣 SHALL 有與主區頂列同高的橫向細線,使頂列橫貫整個視窗、圖示列分隔線自此線往下),承載 macOS 紅綠燈的留白、作用中專案的純文字名稱(左緣 SHALL 與下方專案欄導覽項文字左緣對齊;hover 顯示完整路徑)與右端的「⋯」專案動作鈕;設定模式時左端 SHALL 為「← 設定」返回鈕且無「⋯」;專案設定模式時左端 SHALL 為「← 專案設定」返回鈕、右端為等寬小字的專案名;兩個返回鈕點擊皆 SHALL 回到進入設定前所在的頁面(兩種設定模式之間切換不改變此頁;零專案時為空狀態);零專案且非設定模式時左側標題列 SHALL 留白(只剩圖示列寬,放不下文字)。

設定(齒輪)SHALL 開啟應用程式設定且 SHALL NOT 依賴任何已開啟的專案——零專案時點擊 SHALL 照常進入且頁內功能可操作;設定模式下齒輪 SHALL 為作用中,專案欄 SHALL 換成設定分類欄(200px;頂部全圓搜尋框,以分類名與各頁列標籤做大小寫不敏感比對過濾,無命中時顯示「沒有符合的設定」;其下群組「應用程式」:一般、更新、CLI、伺服器、網路與診斷、安全標準資料,再下群組「關於」:關於 Speclink;群組內沒有命中的分類時連群組標題一併隱藏),左側因此恆為圖示列加分類欄的寬度(零專案亦然),主區麵包屑為「設定 / 分類名」(「設定」可點回一般頁)。設定模式與專案設定模式下,Esc、⌘[、⌘←(非 macOS 為 Alt+←)、滑鼠側鍵「上一頁」與 macOS 上觸控板兩指往返回方向(手指向右)的水平滑動 SHALL 等效於左側標題列的返回鈕;快捷鍵的不作用條件(焦點在文字輸入、按鍵已被浮層處理)與觸控板滑動返回的平台限制、返回指示、門檻、結算時機與不觸發條件同需求「詳情頁的導覽與返回」。專案設定有未儲存的修改時,返回 SHALL 先詢問是否捨棄,確認後才離開;取消時 SHALL 留在原頁且修改保留。有對話框開著時,所有返回操作 SHALL NOT 作用,對話框 SHALL NOT 被連帶收掉。詳情頁、設定模式與專案設定模式以外的頁面沒有返回去處,上述操作 SHALL NOT 作用。專案設定項 SHALL 進入專案設定模式:專案欄換成專案設定分類欄(群組「openspec/config.yaml」:專案說明、產出規則、審查規則、產出流程;群組「專案」:.speclink.yaml,遠端專案無此組;解析失敗的檔所掛分類項帶警示點),主區麵包屑為「專案名 / 專案設定 / 分類名」(前兩段可點:專案名回看板、專案設定回專案說明頁)。兩種分類欄 SHALL 記住上次所在分類。零專案時 SHALL NOT 渲染專案欄,圖示列 SHALL 只有品牌標記、「＋」與設定。有專案且非設定模式時專案欄五個導覽項 SHALL 恆常渲染,SHALL NOT 依頁面狀態顯隱。

#### Scenario: 專案欄順序與設定在圖示列底部

- **WHEN** 開啟任一專案進入桌面 app
- **THEN** 圖示列由上而下為品牌標記、專案方塊、「＋」、底部齒輪;專案欄頂部由上而下依序為變更、已封存、規格、手冊,底部為專案設定(兩者之間為彈性空白);專案欄不存在「設定」與備忘項;左側標題列顯示作用中專案名與「⋯」

#### Scenario: 側欄無常駐版號

- **WHEN** 開啟任一專案檢視圖示列與專案欄(含專案設定下方區域)
- **THEN** 任何位置不存在版號文字;應用程式設定的更新頁版本卡與關於頁品牌卡顯示目前版本

#### Scenario: 零分頁仍可進入應用程式設定

- **WHEN** app 於零專案狀態點擊圖示列底部的齒輪
- **THEN** 進入應用程式設定,左側為圖示列加設定分類欄、麵包屑為「設定 / 一般」、齒輪為作用中;一般頁的顯示語言列與伺服器頁的連線操作皆可正常使用

#### Scenario: 零分頁點專案設定見空狀態

- **WHEN** app 於零專案狀態
- **THEN** 專案欄不存在,專案設定項不可達;主區為「開啟一個專案開始」空狀態

#### Scenario: 已封存徽章隨封存動作更新

- **WHEN** 看板上一個已就緒的變更被封存
- **THEN** 已封存導覽項的徽章數量在數秒內增加一、無需重啟或手動重新整理

#### Scenario: 已封存導覽項為切頁而非 toggle

- **WHEN** 使用者位於已封存頁並再次點擊已封存導覽項
- **THEN** 畫面停留在已封存頁;隨後點擊變更導覽項才返回看板,且變更項高亮、已封存項恢復未選取樣式

#### Scenario: 手冊導覽項切頁與零分頁空狀態

- **WHEN** 使用者點擊專案欄「手冊」項
- **THEN** 切至手冊頁且該項高亮、其他項恢復未選取樣式,麵包屑顯示「專案名 / 手冊」

#### Scenario: 零專案時只剩圖示列

- **WHEN** 使用者關閉最後一個專案
- **THEN** 專案欄消失、左側只剩圖示列(品牌標記、「＋」、齒輪),左側標題列與主區頂列只剩留白,主區呈現空狀態;此時點齒輪,左側擴為圖示列加設定分類欄

#### Scenario: 專案設定模式的分類欄與返回

- **WHEN** 使用者於 local 專案點專案欄「專案設定」,再點左側標題列的「← 專案設定」
- **THEN** 進入時專案欄換成專案設定分類欄(專案說明、產出規則、審查規則、產出流程;.speclink.yaml)、左側標題列為「← 專案設定」與等寬專案名、麵包屑「專案名 / 專案設定 / 專案說明」;點返回後回到看板,專案欄恢復五個導覽項

#### Scenario: 離開設定模式

- **WHEN** 使用者於規格頁點齒輪進入設定模式,再點左側標題列的「← 設定」,或按 Esc、⌘[、滑鼠側鍵「上一頁」,或於 macOS 以觸控板兩指向右滑過門檻後放開
- **THEN** 回到規格頁,專案欄恢復五個導覽項、齒輪恢復未作用;自手冊頁進專案設定模式再以同樣操作返回則回到手冊頁;零專案時回到空狀態,左側只剩圖示列;焦點在分類欄搜尋框時按 Esc 不離開設定模式

#### Scenario: 專案設定有未存修改時返回先確認

- **WHEN** 使用者於專案設定的專案說明頁編輯並改了內容但未儲存,接著按 Esc 或點「← 專案設定」
- **THEN** 出現「要捨棄未儲存的修改嗎？」確認框;選取消則留在專案說明頁且修改仍在,選捨棄則回到進入專案設定前的頁面;只開了編輯而未改動時直接返回、不詢問

#### Scenario: 設定分類欄搜尋

- **WHEN** 使用者於設定分類欄搜尋框輸入「外觀」
- **THEN** 分類欄只剩一般一項(外觀列只在一般頁),「關於」群組標題隱藏;輸入無命中字串時顯示「沒有符合的設定」;清空後兩群組共七項還原(一般、更新、CLI、伺服器、網路與診斷、安全標準資料;關於 Speclink)

#### Scenario: 非 macOS 的設定模式不接觸控板返回

- **WHEN** 使用者於 Windows 自規格頁點齒輪進入設定模式,以觸控板兩指向右滑動超過門檻後放開,接著按 Alt+←
- **THEN** 滑動後仍停在設定模式、左緣不出現「←」指示;按 Alt+← 後回到規格頁


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 視窗預設尺寸與置中

桌面 app 視窗 SHALL 以 1440×900 邏輯尺寸啟動，並 SHALL 於主螢幕置中。此為靜態預設值：app SHALL NOT 記憶上次視窗大小與位置，每次啟動皆以相同預設呈現。

#### Scenario: 啟動視窗尺寸與置中

- **WHEN** 使用者啟動桌面 app
- **THEN** 視窗邏輯尺寸為 1440×900，且視窗於主螢幕水平與垂直置中

##### Example: 1920×1080 主螢幕下的位置

- **GIVEN** 主螢幕解析度 1920×1080、縮放 100%
- **WHEN** 啟動桌面 app
- **THEN** 視窗尺寸 1440×900，左右邊距各約 240、上下邊距各約 90（邏輯像素，實際垂直置中依作業系統工作列語意由視窗系統決定）

---
### Requirement: 介面文字以打包的 Noto Sans TC 呈現

桌面 app SHALL 將 Noto Sans TC 字體隨應用程式打包，並以其為介面與內容文字的第一優先字體；未安裝該字體的機器與離線環境 SHALL 呈現相同字體，SHALL NOT 依賴網路下載字體資產。等寬文字（inline code 與程式碼區塊）SHALL 維持等寬字體，不受此變更影響。

#### Scenario: 未安裝字體的機器呈現打包字體

- **WHEN** 在未安裝 Noto Sans TC 的作業系統上啟動桌面 app 並開啟任一文件檢視
- **THEN** 介面與 markdown 內容文字以打包的 Noto Sans TC 呈現，無對外字體網路請求

#### Scenario: 等寬文字不受字體變更影響

- **WHEN** 檢視含 inline code 或程式碼區塊的文件
- **THEN** 該段文字以等寬字體呈現，與周圍的 Noto Sans TC 內文明顯可辨

<!-- @trace
source: desktop-reading-experience
updated: 2026-07-08
code:
  - apps/desktop/package.json
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/TaskList.tsx
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


<!-- @trace
source: desktop-spec-manual-pages
updated: 2026-10-12T22:59:10+08:00
-->

---
### Requirement: raw HTML 不以原文呈現

桌面 app 渲染 markdown 內容時，來源中的 raw HTML（含 HTML 註解）SHALL NOT 以原始文字出現在渲染結果；渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 討論記錄的 scaffold 註解不顯示

- **WHEN** 開啟討論詳情頁的討論過程分頁，其來源在 Rounds 區段含 CLI scaffold 產生的 HTML 註解行
- **THEN** 渲染結果不含該註解的任何文字，openspec/ 下的討論記錄檔案內容位元不變

#### Scenario: 程式碼區塊內的 HTML 原文照常顯示

- **WHEN** 檢視在 code fence 內含 HTML 標籤範例的文件
- **THEN** code fence 內容以原文完整顯示（過濾僅及於 code fence 之外的 raw HTML）


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 任務分頁提供批次操作工具列

變更詳情頁的任務分頁 SHALL 於任務清單頂部提供工具列：「全部已完成」SHALL 將該變更全部任務標為完成並以單次檔案寫入回寫 tasks.md；「重置任務」SHALL 將全部任務取消勾選並以單次檔案寫入回寫；「下一個未完成」SHALL 使第一個未完成任務捲入可視範圍並短暫高亮，n 快捷鍵 SHALL 等效。批次完成的開工章語意 SHALL 與逐一勾選一致——未開工變更首次完成任務時蓋開工章；重置 SHALL NOT 蓋開工章、SHALL NOT 記錄 touched。目標狀態已達成時重跑 SHALL 為冪等成功且不改檔。全部任務已完成時「全部已完成」與「下一個未完成」SHALL 呈現不可用；唯讀封存檢視 SHALL NOT 顯示工具列。

#### Scenario: 全部已完成單次寫回

- **WHEN** 對含未完成任務的變更按「全部已完成」
- **THEN** tasks.md 全部任務標為完成且僅發生單次檔案寫入，詳情頁進度顯示 100%

##### Example: 批次完成的進度變化

| 操作前 | 操作 | 操作後 |
| ------ | ---- | ------ |
| 3/10 任務完成 | 全部已完成 | 10/10、進度 100% |
| 10/10 任務完成 | 全部已完成 | 不改檔（冪等）、按鈕不可用 |

#### Scenario: 重置任務不蓋開工章

- **WHEN** 對已有完成任務的變更按「重置任務」
- **THEN** tasks.md 全部任務取消勾選且僅發生單次檔案寫入，變更 meta 未新增開工章與 touched 記錄

#### Scenario: 批次完成沿用開工章語意

- **WHEN** 對尚未開工的變更按「全部已完成」
- **THEN** 變更 meta 蓋上開工章（與逐一勾選首次完成任務的行為一致）

#### Scenario: 下一個未完成定位

- **WHEN** 任務清單有多個任務且第一個未完成任務在可視範圍外，按「下一個未完成」或按 n 鍵
- **THEN** 該任務捲入可視範圍並短暫高亮

#### Scenario: 唯讀封存檢視不顯示工具列

- **WHEN** 於已封存頁展開封存變更的任務檢視
- **THEN** 不顯示批次操作工具列，checkbox 維持唯讀


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 勾選任務即時回饋

任務分頁勾選或取消勾選任一任務時，畫面 SHALL 立即反映新勾選狀態（不等待寫回與重載完成），清單其餘任務 SHALL 保持可互動；寫回失敗時 SHALL 回滾至磁碟現況並顯示單行錯誤。寫回成功後的整批重載 SHALL 沿用既有 refresh 世代機制與互動讓路行為。

#### Scenario: 勾選立即反映

- **WHEN** 勾選一個未完成任務且寫回尚未完成
- **THEN** 該 checkbox 立即呈現勾選態與完成刪除線，無可感知等待

#### Scenario: 連續勾選不互鎖

- **WHEN** 快速連續勾選多個任務
- **THEN** 每次勾選皆立即反映，清單全程可互動，最終 tasks.md 與畫面一致

#### Scenario: 寫回失敗回滾

- **WHEN** 勾選任務後寫回失敗（如檔案不可寫）
- **THEN** 該 checkbox 回復原狀態並顯示單行錯誤訊息，tasks.md 內容不變

<!-- @trace
source: desktop-task-interactions
updated: 2026-07-08
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - docs/assets/speclink-logo-concept.png
  - docs/assets/speclink-logo-horizontal.png
  - docs/assets/speclink-logo-lockup.png
  - docs/assets/speclink-logo-mark-redesign.png
  - docs/assets/speclink-logo-mark.png
  - docs/assets/speclink-logo-selected-lockup.png
  - docs/assets/speclink-logo-selected-mark.png
  - docs/assets/speclink-logo-system-sheet.png
  - docs/assets/speclink-logo-vertical.png
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 表單控制項與按鈕以主題化元件呈現

桌面 app 的互動控制項 SHALL 以元件庫的主題化原語呈現：勾選控制項（任務分頁勾選框、初始化對話框的工具多選）SHALL 呈現主題化外觀——未勾選為主題邊框空框、勾選為主色底加勾選圖示，SHALL NOT 呈現作業系統原生控制項外觀；多行文字輸入與程式碼編輯器（專案設定的專案說明與產出規則編輯區）SHALL 取用設計 token 的邊框、底色、游標行、語法上色與 focus 樣式，SHALL NOT 使用編輯器套件的內建配色；動作按鈕 SHALL 統一取用元件庫按鈕變體，鍵盤聚焦時 SHALL 顯示一致的 focus 可視環、disabled 態呈現一致。上述控制項於淺色與深色主題 SHALL 一致取自設計 token。替換 SHALL NOT 改變控制項的行為與無障礙語意：勾選框 SHALL 保留 checkbox 角色、既有標籤與空白鍵切換；按鈕 SHALL 保留既有無障礙名稱、回呼與 disabled 條件；唯讀封存檢視的勾選框 SHALL 維持不可互動。

#### Scenario: 任務勾選框主題化外觀

- **WHEN** 檢視含已完成與未完成任務的任務分頁
- **THEN** 未完成任務的勾選框為主題邊框空框、已完成為主色底加勾選圖示，非作業系統原生繪製，深淺主題下外觀一致

#### Scenario: 勾選框無障礙語意保留

- **WHEN** 以鍵盤或輔助技術操作任務分頁的勾選框
- **THEN** 每個勾選框曝露 checkbox 角色與「任務 N」標籤，空白鍵可切換勾選且觸發與滑鼠點擊相同的寫回

#### Scenario: 初始化對話框工具多選主題化

- **WHEN** 開啟初始化確認對話框
- **THEN** claude 與 codex 選項以主題化勾選框呈現且可獨立勾選，預設勾選狀態與替換前相同

#### Scenario: 設定頁編輯器主題化

- **WHEN** 於深色主題進入專案說明或產出規則的編輯
- **THEN** 編輯器的邊框、底色、游標行、標題與行內 code 上色與 focus 樣式取自設計 token（深色下為深色值），輸入與儲存行為與替換前相同

#### Scenario: 按鈕 focus 可視環一致

- **WHEN** 以 Tab 鍵依序聚焦任務工具列、詳情頁動作列與側欄導覽的按鈕
- **THEN** 每個按鈕顯示一致的 focus 可視環，無障礙名稱與點擊行為與替換前相同


<!-- @trace
source: desktop-settings-editors
updated: 2026-10-11T21:42:42+08:00
-->

---
### Requirement: 任務寫回非阻塞且序列化

任務寫回（單發勾選、批次設定、拖放排序、看板卡片排序）SHALL 於背景執行緒執行，SHALL NOT 阻塞視窗互動——寫回進行中使用者 SHALL 仍可捲動、點擊與觸發其他操作。同一專案的並發任務寫回 SHALL 依提交順序序列化落盤，SHALL NOT 遺失任何一次更新。完成路徑取得 git 身分 SHALL 每專案根至多 spawn 一次 git 並快取（app 存續期內重用），開工章 started_by 的內容 SHALL 與逐次取得時一致。樂觀更新生效後，更早發起而較晚到達的文件載入回應 SHALL NOT 覆蓋樂觀狀態。寫回過程 spawn 的 git 子進程 SHALL NOT 產生可見的主控台視窗（黑窗閃現）。

#### Scenario: 無主控台視窗閃爍

- **WHEN** 勾選任務觸發完成路徑的 git 呼叫（身分首抓或 touched 記錄）
- **THEN** 畫面上不出現任何主控台視窗閃現，git 呼叫結果與行為不變

#### Scenario: 慢寫回不凍結介面

- **WHEN** 勾選任務且後端寫回耗時數秒（如環境使 git 呼叫緩慢）
- **THEN** 視窗全程可操作（捲動、點擊、切換分頁皆有回應），無整窗凍結

#### Scenario: 勾選後立即取消不遺失

- **WHEN** 勾選一任務後在其寫回完成前取消勾選同一任務
- **THEN** 取消立即反映於畫面，兩次寫回依序落盤，最終 tasks.md 該任務為未勾選

#### Scenario: 並發寫回序列化

- **WHEN** 一筆慢寫回進行中又觸發另一任務的寫回
- **THEN** 兩筆寫回皆落盤且互不覆蓋，結果與依序執行一致

#### Scenario: git 身分快取重用

- **WHEN** 同一專案內連續勾選多個任務
- **THEN** git 身分僅首次（或啟動預熱時）取得，後續勾選不再逐次 spawn git，蓋章的 started_by 內容不變

#### Scenario: 舊載入回應不覆蓋樂觀狀態

- **WHEN** 樂觀勾選生效後，一筆更早發起的 tasks.md 載入回應才到達
- **THEN** 畫面維持樂觀勾選狀態不閃爍，寫回完成後由世代重載收斂至磁碟現況

<!-- @trace
source: desktop-toggle-freeze
updated: 2026-07-08
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - crates/speclink-core/src/util.rs
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
-->

---
### Requirement: 討論輪以卡片呈現

桌面 app 的討論過程呈現（討論詳情頁的討論過程文件、已封存討論頁的討論過程文件）SHALL 將符合 scaffold 格式（輪標題形如「### Round N — <mode> (<date>)」）的記錄逐輪呈現為時間軸上的輪卡，輪卡 SHALL 依輪次由新到舊排列（輪次最大者在最上、Round 1 在最下；只改呈現，來源記錄的輪順序不變）：時間軸 SHALL 有一條貫穿的細線，每輪一個節點，節點色點依 mode 區分——`assumptions` 取主色、`interview` 取看板討論欄的色相、其他 mode 取中性色（mode 值 SHALL 按來源字串透傳呈現，不設白名單）。輪卡卡頭 SHALL 顯示輪次籤（本頁目錄錨點）、焦點（Focus 欄位的首行，作為該輪標題；來源缺 Focus 時只有輪次籤）、mode 與日期。卡身 SHALL 將行首的 Position／Ruled out／Open 粗體前綴呈現為欄位（前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行或輪結尾的全部內容；非四詞白名單的粗體前綴行 SHALL 按一般內文歸屬當前欄位）：立場的第一段散文為定論句、以中等字重呈現，其後的頂層列點為細線列；否決在內文有列點時每條列點一列、否則以「；」切列，每列帶「×」圖示；待決以「無」或 none 起頭（其後緊接標點、空白或結尾；「無法…」不算）時 SHALL 呈現為一行帶勾選圖示的灰字原文，否則每條（列點或以「；」切出的段）一顆琥珀色籤。來源缺席的欄位 SHALL NOT 渲染空標籤。第一輪與最後一輪（由新到舊排列後即最下與最上兩張輪卡）SHALL 預設展開，其餘輪 SHALL 預設收合為卡頭一列，點擊卡頭 SHALL 切換展開。任一輪標題不符 scaffold 格式時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

討論過程文件的閱讀卡標題下 SHALL 有數字磚：「輪」（輪數）、「天」（不同日期數），其後依首次出現序每個 mode 一格（值為該 mode 的輪數、標籤為 mode 原字串），最多四格。出處行 SHALL 為日期範圍（首輪日期 – 末輪日期；同日只寫一次）與各 mode 的輪數。結論區段含任一白名單欄位時 SHALL 有 tip 提示框「討論已收斂」，內文為「第 N 輪之後寫入結論」並接末輪待決的狀態（末輪 Open 以「無」起頭時「最後一輪待決：無」、否則「最後一輪仍列 M 項待決」、末輪缺 Open 時不接）；結論未寫入時 SHALL NOT 有提示框。

#### Scenario: 標準記錄逐輪成卡

- **WHEN** 開啟含四輪 scaffold 格式記錄的討論過程文件
- **THEN** 時間軸上由上而下依序呈現第 4、3、2、1 輪四張輪卡，各卡頭顯示輪次、焦點、mode 與日期；第四輪與第一輪展開、第三與第二輪收合為卡頭一列；展開卡的立場以定論句＋細線列呈現，來源的粗體前綴原文不出現在渲染結果

##### Example: 輪標題解析

- **GIVEN** 來源輪標題「### Round 1 — assumptions (2026-07-08)」與 Focus 行「**Focus**: 現況的痛點是哪一種」
- **WHEN** 開啟討論過程文件
- **THEN** 該卡片卡頭顯示輪次 Round 1、焦點「現況的痛點是哪一種」、mode assumptions（主色色點）、日期 2026-07-08

#### Scenario: 新的輪排在最上

- **WHEN** 開啟一份 9 輪、記錄檔 Rounds 區段由 Round 1 依序寫到 Round 9 的討論過程文件（討論詳情頁，或該討論封存後的已封存討論頁）
- **THEN** 輪卡由上而下為 Round 9、Round 8 … Round 1；Round 9 與 Round 1 展開、其餘七輪收合；右欄本頁目錄依同一順序列出輪次；記錄檔位元不變

##### Example: 九輪的排列

- **GIVEN** 記錄檔 Rounds 區段依序為 Round 1 到 Round 9，Round 9 的 Focus 為「結論後補充」
- **WHEN** 開啟討論過程文件
- **THEN** 第一張輪卡的輪次籤為 Round 9、焦點為「結論後補充」、`aria-expanded` 為 true；最後一張輪卡為 Round 1、`aria-expanded` 為 true；其間七張的 `aria-expanded` 為 false

#### Scenario: 缺席欄位不渲染空標籤

- **WHEN** 某輪來源只有 Focus、Position 與 Open 行，無 Ruled out 行
- **THEN** 該卡片展開後呈現立場與待決兩個欄位，無否決欄位

#### Scenario: 欄位涵蓋後續多行內容

- **WHEN** 某輪的 Position 標籤行之後接一句定論與數行列點、其後才是 Ruled out 標籤行
- **THEN** 定論句以中等字重呈現、數行列點全數成為立場的細線列，列數與來源列點數相同

#### Scenario: 非標準格式整篇退回

- **WHEN** 記錄的 Rounds 區段含不符 scaffold 格式的輪標題（手寫或 pre-scaffold 記錄）
- **THEN** 討論過程文件整篇以單一 markdown 檢視呈現，無時間軸、無數字磚、無提示框、無錯誤訊息，來源檔案位元不變

#### Scenario: 否決與待決的畫法

- **WHEN** 某輪的 Ruled out 為「A——理由甲；B——理由乙」、Open 為「點綴色留 teal 還是換紫；切刀」，末輪的 Open 為「無——寫結論」
- **THEN** 該輪否決呈現兩列各帶「×」圖示；待決呈現兩顆琥珀色籤；末輪的待決為一行帶勾選圖示的灰字「無——寫結論」

#### Scenario: 收合輪的展開

- **WHEN** 使用者於 11 輪的討論過程文件點第 5 輪的卡頭
- **THEN** 第 5 輪展開呈現其欄位，卡頭的 `aria-expanded` 為 true；再點一次收合；第 1 輪與第 11 輪的展開狀態不受影響

#### Scenario: 討論過程的數字磚、出處行與已收斂提示框

- **WHEN** 開啟一份 20 輪、全部在 2026-10-08、assumptions 10 輪、interview 10 輪、結論已寫入且末輪 Open 為「無：寫入結論」的討論過程文件
- **THEN** 數字磚依序為 20「輪」、1「天」、10「assumptions」、10「interview」；出處行為「2026-10-08 · assumptions 10 輪 · interview 10 輪」；tip 提示框標題「討論已收斂」、內文含「第 20 輪之後寫入結論」與「最後一輪待決：無」

#### Scenario: 結論未寫入時無提示框

- **WHEN** 開啟一份 3 輪、結論區段只有鷹架註解的討論過程文件
- **THEN** 時間軸與數字磚照常，無「討論已收斂」提示框


<!-- @trace
source: doc-tiles-fill-and-round-order
updated: 2026-10-13T15:43:46+08:00
-->

---
### Requirement: 討論結論以欄位標籤呈現

桌面 app 的討論結論呈現（討論詳情頁的結論文件、已封存討論頁的結論文件）SHALL 將行首的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 粗體前綴切成六個欄位（前綴原文 SHALL NOT 出現在渲染結果；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行或結論結尾的全部內容；非六詞白名單的粗體前綴行 SHALL 按一般內文歸屬當前欄位），並以 16px 粗體的區段標題呈現各欄位（標籤依語系呈現，zh-TW 為決定／理由／否決替代案／擱置／記錄去向／下一步；每個區段標題 SHALL 為本頁目錄錨點）。來源缺席的欄位 SHALL NOT 渲染空區段；結論不含任何白名單欄位時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

各欄位的畫法 SHALL 為：

- 決定：第一段散文為定論句，SHALL 以大於內文的字級呈現；其後的頂層列點為定案細節，SHALL 以細線列呈現且預設只展開前 4 條，其餘以「展開其餘 N 項定案細節」鈕一次展開；區段標題右端 SHALL 顯示細節總數。
- 分刀：決定的列點中，首行以粗體開頭、粗體內（去掉反引號片段後）以 `cut <編號>`、「刀」或「第 N 刀」起頭的列 SHALL 自細節列抽出，成為「分刀」區段（有任一刀時才出現、位於決定之後）；每刀一張卡，卡頭為刀的識別籤（粗體內去掉反引號片段後開頭的 `cut <編號>`、「刀…」或「第 N 刀」）、範圍句與該刀的變更數。範圍句：粗體內在識別籤之後還有文字時為該文字，否則為粗體之後（去掉開頭冒號）、第一個「——」之前的文字；首行其餘的文字（範圍句取自粗體時為粗體之後的全部文字、外圍整組括號拿掉，否則為「——」之後的文字）SHALL 為卡內第一條細線列，不丟內容。粗體內的反引號片段為該刀的變更名籤，粗體內沒有反引號片段時取同一行粗體之後的反引號片段（含空白的片段不算）——名稱在 `promoted_to` 中時籤 SHALL 附階段籤（階段派生與配色同「討論於看板第 0 欄兩級呈現」）且點擊 SHALL 開啟該變更的詳情頁（已封存者開啟已封存詳情頁），不在 `promoted_to` 中時為不可點籤；列的巢狀子項 SHALL 接在其後為卡內的細線列。刀是否轉出 SHALL 只依該刀粗體內的變更名籤是否在 `promoted_to` 判定（變更名只寫在粗體之後的刀列，反引號片段可能是識別字或檔名，視同沒有變更名籤）：刀至少有一個變更名籤且全部不在 `promoted_to` 時為「整刀未轉出」，卡 SHALL 以橄欖色（手動任務語意色）虛線外框與淡橄欖底呈現，卡頭的識別籤之後 SHALL 有橄欖色「尚未轉出」籤；變更名籤部分在、部分不在 `promoted_to` 時，每個不在其中的變更名籤之後 SHALL 有同款「尚未轉出」籤，卡的外框照舊；刀沒有任何變更名籤時 SHALL NOT 有任何轉出標示。
- 理由：照 markdown 呈現。
- 否決替代案與擱置：內文有列點時每條列點一列、否則以「；」切列；每列以第一個分隔號（「——」或前後帶空白的「 — 」，取先出現者；discuss 技能檔的寫法為「方案 — 理由」）拆成「方案」與「落敗理由」兩行，切不到時整條為方案；擱置的每列 SHALL 帶時鐘圖示；兩個區段標題右端 SHALL 顯示列數。
- 記錄去向：內文以「、」或逗號切成籤；籤文字為正式規格名時 SHALL 為可點籤，點擊開啟該規格的規格詳情頁；其餘為不可點籤；切不出籤時照 markdown 呈現。
- 下一步：照 markdown 呈現，其後 SHALL 另列內文中每個 `/speclink-*` 指令（含其旗標與參數）的等寬籤，每顆附複製鈕，點擊 SHALL 把指令原文寫入剪貼簿。

結論文件的閱讀卡標題下 SHALL 有數字磚：「定案細節」（細節數）、「分刀」（刀數，標籤附變更數；無分刀時缺席）、「否決的替代方案」（列數；欄位缺席時缺席）、「擱置」（列數；欄位缺席時缺席）。討論詳情頁的結論文件有任一刀整刀未轉出時，數字磚之下 SHALL 有橄欖色（手動任務語意色）提示框「N 刀尚未轉出」（N 為整刀未轉出的刀數）：每刀一列，呈現識別籤、範圍句與「前往」鈕，點擊 SHALL 把該刀卡捲到閱讀區可見處；提示框末行 SHALL 說明這幾刀還沒有變更，並附一顆可複製的等寬指令籤 `/speclink-propose --from-discussion <slug>`（樣式與「下一步」的指令籤相同）。沒有整刀未轉出的刀時 SHALL NOT 有提示框；已封存討論頁的結論文件 SHALL NOT 有提示框（刀卡的轉出標示照常）。結論文件 SHALL NOT 有其他提示框。

#### Scenario: 標準結論欄位成標籤區塊

- **WHEN** 開啟含 scaffold 格式結論（Decision、Rationale、Capture to、Next 各佔一行起頭）的結論文件
- **THEN** 四個欄位以 16px 粗體區段標題呈現並列入本頁目錄，「**Decision**:」等粗體前綴原文不出現在渲染結果

#### Scenario: 結論缺席欄位不渲染空標籤

- **WHEN** 某結論來源只有 Decision 與 Rationale 行，無 Deferred 行
- **THEN** 結論文件呈現決定與理由兩個區段，無擱置區段，數字磚無「擱置」格

#### Scenario: 自由格式結論整篇退回

- **WHEN** 結論來源為手寫自由段落、不含任何白名單粗體前綴行
- **THEN** 結論文件整篇以單一 markdown 檢視呈現，無區段標題、無數字磚、無錯誤訊息，來源檔案位元不變

#### Scenario: 決定主句放大與細節收合

- **WHEN** 開啟一份 Decision 為「一句定論＋27 條頂層列點（其中 7 條為分刀列）」的結論文件
- **THEN** 定論句以大於內文的字級呈現；其下細線列只列前 4 條非分刀細節，鈕文字為「展開其餘 16 項定案細節」，點擊後 20 條全部可見且鈕消失；「決定」區段標題右端計數為 20

#### Scenario: 分刀卡與變更籤

- **WHEN** 開啟 detail-content-redesign 的結論文件，其 Decision 列點含「- **cut 1 `desktop-detail-doc-tree`**: 骨架——……」等三條分刀列，且 `promoted_to` 含前兩刀的變更而第三刀尚未轉出
- **THEN** 「分刀」區段有三張卡，卡頭依序為 cut 1、cut 2、cut 3 的識別籤與範圍句（「骨架」等，「——」之後的文字不在卡頭）；前兩張卡的變更籤附「已封存」階段籤且點擊開啟該變更的已封存詳情頁，第三張的變更籤不可點且無階段籤，該卡為橄欖色虛線外框、卡頭識別籤之後有「尚未轉出」籤；前兩張卡為一般外框、無「尚未轉出」籤；每張卡內列出該列的巢狀子項；數字磚「分刀」為 3 且標籤含「3 個變更」

##### Example: 分刀列的解析

| 列首粗體 | 識別籤 | 變更名籤 | 範圍句 |
| --- | --- | --- | --- |
| `**cut 1 \`desktop-detail-doc-tree\`**: 骨架——三種詳情頁改三欄` | cut 1 | desktop-detail-doc-tree | 骨架 |
| `**cut 2 \`desktop-shell-redesign\` \`desktop-notice-relocation\`**: 外殼 7 項` | cut 2 | desktop-shell-redesign、desktop-notice-relocation | 外殼 7 項 |
| `**刀三**：系統匣面板` | 刀三 | （無） | 系統匣面板 |
| `**cut 2 外殼**（propose 期拆兩個 change：\`desktop-shell-redesign\` 外殼結構、\`desktop-native-menu\` 原生選單）` | cut 2 | desktop-shell-redesign、desktop-native-menu | 外殼（卡內第一條細線列為括號內的說明） |
| `**範圍與驗收**：只改 UI 渲染` | （不是分刀，留在定案細節） | — | — |

#### Scenario: 否決與擱置拆方案與理由

- **WHEN** 開啟一份 Rejected alternatives 有 14 條「方案 — 理由」列點、Deferred 有 5 條（其中一條沒有分隔號「——」或「 — 」）的結論文件
- **THEN** 「否決替代案」區段計數 14、每列方案一行、理由灰字一行；「擱置」區段計數 5、每列帶時鐘圖示，沒有分隔號的那列只有方案行

#### Scenario: 記錄去向籤與下一步指令籤

- **WHEN** 開啟一份 Capture to 為「proposal、design、spec（desktop-app MODIFIED）、tasks」且 Next 含「/speclink-propose --from-discussion detail-content-redesign」的結論文件，正式規格存在 desktop-app
- **THEN** 「記錄去向」呈現四顆籤，點 desktop-app 籤開啟該規格的規格詳情頁，其餘籤不可點；「下一步」的 markdown 之後有一顆等寬指令籤「/speclink-propose --from-discussion detail-content-redesign」，點複製鈕後剪貼簿為該指令原文

#### Scenario: 結論文件的數字磚與出處行

- **WHEN** 開啟一份 11 輪、Decision 有 20 條細節與 3 刀、Rejected alternatives 有 11 條、Deferred 有 4 條、三刀的變更皆在 `promoted_to` 的 live 討論結論文件
- **THEN** 閱讀卡出處行為 `openspec/discussions/<slug>.md · 11 輪之後寫入`；數字磚依序為 20「定案細節」、3「分刀 · 3 個變更」、11「否決的替代方案」、4「擱置」；無提示框

#### Scenario: 未轉出刀的提示框與前往

- **WHEN** 使用者於討論詳情頁開啟 wad-plugins-merge-or-split 的結論文件，其 Decision 有 cut 1 `add-team-plugin-library`（在 `promoted_to`、進行中）與 cut 2 `remove-builtin-wad-work`（不在 `promoted_to`）兩條分刀列
- **THEN** 數字磚之下有橄欖色提示框，標題「1 刀尚未轉出」，恰一列為 cut 2 的識別籤、範圍句與「前往」鈕；點「前往」後 cut 2 的刀卡捲到閱讀區可見處；提示框末行有指令籤「/speclink-propose --from-discussion wad-plugins-merge-or-split」，點其複製鈕後剪貼簿為該指令原文；cut 2 卡為橄欖色虛線外框且卡頭有「尚未轉出」籤，cut 1 卡為一般外框、變更名籤附「進行中」階段籤

##### Example: 刀的轉出判定

| 刀列粗體內的變更名籤 | 在 `promoted_to` 的 | 判定 | 刀卡 | 提示框列出 |
| --- | --- | --- | --- | --- |
| `a`、`b` | `a`、`b` | 已轉出 | 一般外框、各附階段籤 | 否 |
| `a`、`b` | `a` | 部分轉出 | 一般外框、`b` 之後有「尚未轉出」籤 | 否 |
| `a` | （無） | 整刀未轉出 | 橄欖色虛線外框、卡頭「尚未轉出」籤 | 是 |
| （無變更名籤） | — | 無從判定 | 一般外框、無轉出標示 | 否 |
| （粗體內沒有；粗體之後有 `KeyLines`、`util.rs`） | （無） | 無從判定 | 一般外框、無轉出標示 | 否 |

#### Scenario: 沒有整刀未轉出的刀時無提示框

- **WHEN** 開啟一份分刀全為「已轉出」「部分轉出」或「無變更名籤」的 live 討論結論文件
- **THEN** 結論文件沒有提示框；部分轉出的刀只在未轉出的變更名籤之後有「尚未轉出」籤

#### Scenario: 已封存討論頁不出未轉出提示框

- **WHEN** 使用者於已封存頁開啟一筆已封存討論的結論文件，其中一刀的變更名不在 `promoted_to`
- **THEN** 該刀卡為橄欖色虛線外框且卡頭有「尚未轉出」籤；結論文件沒有提示框


<!-- @trace
source: discussion-stage-and-unpromoted-cuts
updated: 2026-10-13T22:22:52+08:00
-->

---
### Requirement: markdown 文件內容行寬有上限

桌面 app 經共用 markdown 渲染呈現的文件內容 SHALL 有固定行寬上限：閱讀卡內（變更、討論、已封存、規格四種詳情頁與手冊頁）的上限 SHALL 為閱讀卡的內容欄寬（卡寬 768px 扣內距），章節標題、數字磚列、提示框、群組卡、輪卡、需求段與內文 SHALL 同欄等寬對齊；數字磚列內各格 SHALL 等寬，依該列的格數平分並填滿整列（只有 1 格時該格佔滿整列），SHALL NOT 固定為四欄而在格數不足時留下空欄；容器寬於上限時內容欄 SHALL 水平置中、留白均分兩側。主區寬度改變時內文行寬 SHALL NOT 隨之增長。超過行寬的表格 SHALL 於容器內橫向捲動，版面 SHALL NOT 橫向溢出。

#### Scenario: 主區加寬時行寬不增長且內容欄置中

- **WHEN** 視窗拉寬至 1600px 檢視變更詳情頁的提案文件
- **THEN** 閱讀卡維持 768px 置中、左右留白均分，內文行寬不超過卡的內容欄，章節標題、數字磚列與內文同欄對齊

#### Scenario: 寬表格於容器內橫捲

- **WHEN** 檢視含超過行寬上限的寬表格的文件
- **THEN** 表格於容器內橫向捲動，頁版面不橫向溢出

#### Scenario: 規格詳情頁與手冊頁的行寬同卡欄

- **WHEN** 視窗拉寬至 1600px 依序檢視一份規格詳情頁與一頁手冊
- **THEN** 兩者的閱讀卡維持 768px 置中，需求文、Scenario 區塊、手冊內文的行寬都不超過卡的內容欄，數字磚列與內文同欄對齊，右欄本頁目錄在卡的右側

#### Scenario: 數字磚依格數平分整列

- **WHEN** 視窗寬 1440px 時依序檢視數字磚為 3 格的提案文件、4 格的任務文件、1 格（只有「先前討論 0」）的討論背景文件
- **THEN** 3 格時三格等寬、各佔數字磚列寬的三分之一、右端沒有空欄；4 格時四格等寬；1 格時該格佔滿整列；三種情況下數字磚列的左右緣都與內文同欄對齊


<!-- @trace
source: doc-tiles-fill-and-round-order
updated: 2026-10-13T15:43:46+08:00
-->

---
### Requirement: 規格分頁 delta 區段以色標呈現

變更詳情頁規格文件與已封存變更檢視的規格文件 SHALL 每個能力一段：能力頭為等寬能力名、delta 計數徽章（DeltaBadges）與「看正式規格」鈕（宿主提供開規格詳情頁的回呼時渲染，點擊開該能力的規格詳情頁；未提供時不渲染；作用中變更的提案把該能力列在 New Capabilities 時 SHALL NOT 渲染——封存前它沒有正式規格）；能力頭 SHALL 帶本頁目錄錨點。delta spec 的區段標題（ADDED／MODIFIED／REMOVED／RENAMED Requirements）SHALL 呈現為色標區段標頭，原始區段標題行 SHALL NOT 以標題文字直出；色標配色 SHALL 與 delta 計數徽章一致。ADDED 與 MODIFIED 區段內每條「### Requirement:」SHALL 以規格詳情頁同一個需求段呈現（「Requirement」等寬小標、16px 粗體標題帶本頁目錄錨點、需求文、左線縮排的 Scenario；SHALL／MUST 關鍵字等寬灰底）並加該操作色的左色條；Scenario 內文的「- **WHEN**」「- **THEN**」（含 **AND**／**GIVEN**）列點 SHALL 成「標籤／內容」兩欄，其餘內容（如 ##### Example 區塊）照 prose 接在兩欄之下，沒有這類列點時整段照 prose；MODIFIED 區段內「<!-- REMOVED-SCENARIO: 名稱 -->」宣告 SHALL 自內文取出，於該需求的需求文之前以 caution 提示框「本變更移除 N 個 scenario」列出各名稱（刪除線），並說明封存時正式規格的這些 scenario 會被拿掉；「<!-- BEFORE: … -->」註解 SHALL NOT 直出。需求文超過三段時 SHALL 只展開前三段並以「展開其餘 N 段需求文」鈕收合其餘；Scenario 超過三個時 SHALL 只展開前三個並以「還有 N 個 scenario」鈕收合其餘；每個 ADDED／MODIFIED 區段的第一條需求預設展開，其餘需求預設收合為一列（Requirement 小標、標題、Scenario 數籤、有移除 scenario 時「移除 N 個 scenario」籤），點列展開。REMOVED 與 RENAMED 區段內文（**Reason**／**Migration**、FROM:／TO:）SHALL 照 prose 排版於色標標頭之下。不含 delta 區段標題的規格文件 SHALL 整篇照常渲染。

閱讀卡的籤槽 SHALL 為各操作的計數籤（用詞與配色同色標標頭，如「修改 5」；為零的操作不出籤）；數字磚依序 SHALL 為「觸及的能力」（能力數）、「修改的需求」（MODIFIED 需求總數）、「移除的 scenario」（REMOVED-SCENARIO 宣告總數）、「新增／更名」（ADDED 與 RENAMED 需求數，顯示為「a ／ b」）；沒有任何 delta 規格時籤槽與數字磚缺席。

#### Scenario: delta 區段呈現色標標頭

- **WHEN** 檢視含 ADDED 與 MODIFIED 區段的 delta spec，MODIFIED 區段有三條需求
- **THEN** 能力頭顯示等寬能力名、計數徽章與「看正式規格」鈕；呈現綠色「新增」與琥珀色「修改」區段標頭，原始「ADDED Requirements」「MODIFIED Requirements」標題文字不出現在渲染結果；「修改」標頭之下第一條需求展開為需求段且左緣為琥珀色條，其餘兩條收合為一列

##### Example: 四種 delta 區段的色標對應

| 來源區段標題 | 標頭文字 | 色系 |
| ------------ | -------- | ---- |
| ## ADDED Requirements | 新增 | 綠（emerald） |
| ## MODIFIED Requirements | 修改 | 琥珀（amber） |
| ## REMOVED Requirements | 移除 | 紅（red） |
| ## RENAMED Requirements | 更名 | 藍（sky） |

#### Scenario: 新能力不出看正式規格

- **WHEN** 作用中變更的提案在 New Capabilities 列出 `doc-views`、Modified Capabilities 列出 `desktop-app`，兩者的 delta 規格都已載入，宿主提供開規格詳情頁的回呼
- **THEN** `desktop-app` 段的能力頭有「看正式規格」鈕，`doc-views` 段的能力頭沒有

#### Scenario: 無 delta 標記的規格照常渲染

- **WHEN** 檢視不含任何 delta 區段標題的規格文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無色標標頭、無需求段、無數字磚

#### Scenario: 移除 scenario 的提示框與計數

- **WHEN** 檢視 delta spec，其 MODIFIED 區段的需求「面板樣式（macOS）」內含兩行「<!-- REMOVED-SCENARIO: ready workspace 的區塊順序與分割線 -->」「<!-- REMOVED-SCENARIO: 進度條依階段深淺 -->」
- **THEN** 該需求段的需求文之前有 caution 提示框「本變更移除 2 個 scenario」，內列兩個刪除線名稱；註解原文不出現在渲染結果；數字磚「移除的 scenario」為 2；該需求收合時列上帶「移除 2 個 scenario」籤

#### Scenario: WHEN／THEN 兩欄與收合

- **WHEN** 檢視 delta spec 的一條需求，其需求文有五段、Scenario 五個，每個 Scenario 為「- **WHEN** …」「- **THEN** …」兩列點
- **THEN** 需求文只展開前三段並顯示「展開其餘 2 段需求文」鈕；前三個 Scenario 以左線縮排呈現且 WHEN／THEN 為標籤欄與內容欄兩欄；其後顯示「還有 2 個 scenario」鈕，點擊後五個全部展開

#### Scenario: 規格的籤槽與數字磚

- **WHEN** 檢視本變更的規格文件，delta 涵蓋兩個能力：desktop-app（MODIFIED 5 條、REMOVED-SCENARIO 2 個）與 client-protocol（ADDED 1 條）
- **THEN** 籤槽為「修改 5」「新增 1」；數字磚為「觸及的能力 2」「修改的需求 5」「移除的 scenario 2」「新增／更名 1 ／ 0」；點 desktop-app 能力頭的「看正式規格」開 desktop-app 的規格詳情頁


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
### Requirement: 提案與設計章節以中文標籤呈現

變更詳情頁與已封存檢視的提案／設計文件 SHALL 以二級標題切段：每個二級標題 SHALL 成為一段、帶本頁目錄錨點、算進數字磚；已知模板章節標題（提案側 Why／What Changes／Non-Goals／Capabilities／New Capabilities／Modified Capabilities／Impact／Problem／Root Cause／Proposed Solution／Success Criteria／Summary／Motivation／Alternatives Considered；設計側 Context／Goals / Non-Goals／Decisions／Implementation Contract／Risks / Trade-offs／Migration Plan／Open Questions；標題尾的「(optional)」於比對前剝除）SHALL 顯示中文標籤與該章節的專屬畫法，英文模板標題 SHALL NOT 以標題文字直出；白名單以外的二級標題 SHALL 以原文為段標題（不翻譯）、內文照 prose 排版。章節標題款式 SHALL 為 16px 粗體（計算字級 16px、大於內文的 15px），與任務群組卡頭、規格 delta 的需求標題、排程四卡頭、工單輪卡頭同款式（色標區段標頭保留各 delta 色彩）；列式章節的標題右端 SHALL 附列點數籤。整份文件沒有任何二級標題時 SHALL 整篇以單一 markdown 檢視退回，不報錯。章節白名單 SHALL 留在前端，SHALL NOT 讀 schema 模板。渲染 SHALL NOT 修改任何來源檔案。

提案側的專屬畫法 SHALL 為：Summary 成灰底引言段；Why／What Changes／Proposed Solution 的頂層列點與編號項成編號步驟，連續散文區塊亦各作一項，混合內容依來源順序形成同一連續編號序列並顯示項目數；此為所有專案的預設，不依專案名稱或設定，活動／已封存及Local／Remote共用ProposalDoc，顯示不改來源檔；巢狀列點、連結、續行與程式區塊保留，不依標點拆句，空章節不產生空項。Motivation／Problem／Root Cause／Success Criteria 的頂層列點成細線列（非列點段落照 prose）；Non-Goals 的列點成帶禁止圖示的細線列；Alternatives Considered 的每個列點以首個「——」切成「方案／落敗理由」兩段，切不到時整條當一般列；Capabilities 的 New Capabilities／Modified Capabilities 次級標籤之下每個「`能力名`: 說明」列點成能力籤＋說明，能力名在本變更已載入的 delta 規格中時籤 SHALL 可點——點擊切到規格文件並選定該能力（規格文件捲到該能力段、「開啟 ▾」指向它），不在時為不可點的籤，「(none)」顯示「無」；Impact 的「Affected specs:」列成「規格」列（能力籤規則同上），「Affected code:」之下的 New／Modified／Removed 子列點依序成「新增 N」「修改 N」「移除 N」三組等寬路徑籤（路徑以「、」或「,」分隔，「(none)」顯示「無」），路徑籤 hover 時 SHALL 出現複製鈕，宿主提供本機路徑動作時 SHALL 另出「在 Finder 顯示」鈕（非 macOS 依平台詞條），「移除」組的路徑籤 SHALL NOT 出該鈕（那些檔案在 apply 後已不存在）；Impact 段不符這個形狀時 SHALL 整段當一般細線列。

設計側的專屬畫法 SHALL 為：Context 成細線列；Goals / Non-Goals 以「**Goals:**」與「**Non-Goals:**」兩個粗體標籤切成左右兩欄（各列其下的列點），切不到時照 prose；Decisions 之下每個三級標題一張可收合卡（標題開頭的 D 編號成籤、卡頭右端標列點數；預設收合，點卡頭展開；自任務文件的 D 籤跳來時該卡展開並捲到卡頭）；Implementation Contract 的「- **鍵**：值」列點成鍵值列，切不到時照 prose；Risks / Trade-offs 的每個列點成一個 warning 提示框（以首個「→」切標題與內文，切不到時整條歸標題；標題只取首行並以行內標記呈現，其餘行（巢狀子項、續行）接在內文之前）；Migration Plan 與 Open Questions 照 prose。

閱讀卡的籤槽與數字磚 SHALL 由文件結構推出：提案的籤槽標出模板形狀——有 Problem 與 Root Cause 為「Bug Fix」、有 Summary 與 Motivation 為「Refactor」、有 Why 與 What Changes 為「Feature」、皆非時籤缺席；提案的數字磚依序取「解法步驟」（What Changes 或 Proposed Solution 的列點與編號項數）、「非目標」、「成功準則」、「能力」（新增與修改的能力數）、「影響的檔案」（新增、修改、移除的路徑總數）、「替代方案」中章節存在的前四格，章節存在而為零時顯示 0，不足四格少放；設計的數字磚依序為「決策」（Decisions 下三級標題數）、「目標／非目標」（兩欄列點數，顯示為「a ／ b」）、「風險與取捨」（列點數）、「開放問題」（列點數，「無」為 0），章節缺席時該格缺席。提案的 Capabilities 段之下 SHALL 有一個 note 提示框，每個已載入 delta 規格的修改能力一行：「能力名 有 N 條需求被改寫」，該 delta 含 REMOVED-SCENARIO 宣告時加「，其中 M 個 scenario 以 REMOVED-SCENARIO 宣告」；沒有任何可推的能力時提示框缺席。

#### Scenario: 提案模板章節成中文標籤

- **WHEN** 檢視含 Why、What Changes、Non-Goals、Capabilities、Impact 章節的提案文件
- **THEN** 呈現「為什麼」「變更內容」「非目標」「能力」「影響」五段，Why 等英文標題文字不出現在渲染結果；本頁目錄依序列出五段；「變更內容」為編號步驟、「非目標」為帶禁止圖示的細線列

##### Example: 章節對照與畫法

| 來源章節標題 | 呈現標籤 | 畫法 |
| ------------ | -------- | ---- |
| ## Summary | 摘要 | 灰底引言段 |
| ## Why | 為什麼 | 編號步驟（含散文區塊） |
| ## What Changes | 變更內容 | 編號步驟 |
| ## Proposed Solution | 解法 | 編號步驟 |
| ## Non-Goals | 非目標 | 禁止圖示細線列 |
| ## Alternatives Considered | 曾考慮的替代案 | 方案／落敗理由 |
| ## Capabilities | 能力 | 能力籤＋說明 |
| ## Impact | 影響 | 規格列＋路徑籤三組 |
| ## Context | 背景 | 細線列 |
| ## Decisions | 決策 | 可收合決策卡 |
| ## Implementation Contract | 實作契約 | 鍵值列 |
| ## Risks / Trade-offs | 風險與取捨 | warning 提示框逐條 |

#### Scenario: 章節標題為 16px 粗體

- **WHEN** 檢視提案文件的章節標題、任務文件的群組卡頭與規格文件的需求標題
- **THEN** 三者皆為粗體且計算字級為 16px，內文計算字級為 15px；章節標題不是 20px

#### Scenario: 白名單外章節照排

- **WHEN** 檢視提案文件，其含白名單外的二級標題「Rollout」與其下兩段文字
- **THEN** 「Rollout」以原文為段標題呈現（不翻譯、款式同其他章節標題）、進本頁目錄、內文照 prose 排版；整份文件不因它退回單一 markdown 檢視

#### Scenario: 無二級標題整篇退回

- **WHEN** 檢視手寫自由格式（沒有任何二級標題）的提案文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無章節段、無數字磚、無提示框、無錯誤訊息

#### Scenario: 替代方案與影響段的逐項降級

- **WHEN** 檢視提案文件，其 Alternatives Considered 有一條含「——」、一條不含，Impact 段為「- Affected code:」加 New 兩個路徑、Modified 一個路徑、Removed「(none)」
- **THEN** 含「——」的列拆成「方案」與「落敗理由」兩段，不含的列整條為一般列；影響段呈現「新增 2」「修改 1」「移除 無」三組，路徑為等寬籤且 hover 出現複製鈕；數字磚「影響的檔案」為 3

##### Example: 影響段不符形狀時當一般列

- **GIVEN** Impact 段只有兩條散文列點「影響看板與詳情頁」「無 CLI 變動」
- **WHEN** 檢視該提案文件
- **THEN** 「影響」段為兩條細線列、無路徑籤，數字磚不出現「影響的檔案」格

#### Scenario: 移除組的路徑籤沒有顯示鈕

- **WHEN** 宿主提供本機路徑動作，檢視提案文件，其 Impact 段的 Removed 列出 `c/gone.ts`、New 列出 `a/new.ts`
- **THEN** `a/new.ts` 籤 hover 時出現複製鈕與顯示鈕；`c/gone.ts` 籤只出現複製鈕

#### Scenario: 能力籤跳到規格與改寫提示框

- **WHEN** 檢視提案文件，其 Modified Capabilities 列出 `desktop-app`，本變更的 delta 規格 specs/desktop-app/spec.md 已載入且含 3 條 MODIFIED 需求與 1 個 REMOVED-SCENARIO 宣告
- **THEN** 「能力」段的 `desktop-app` 籤可點，點擊後左欄「規格」節點作用中、閱讀卡捲到 desktop-app 段、「開啟 ▾」指向 specs/desktop-app/spec.md；「能力」段之下的 note 提示框顯示「desktop-app 有 3 條需求被改寫，其中 1 個 scenario 以 REMOVED-SCENARIO 宣告」

#### Scenario: 提案的籤槽與數字磚

- **WHEN** 檢視 Refactor 形狀的提案文件（Summary、Motivation、Proposed Solution 六個編號項、Non-Goals 五條、Alternatives Considered 四條、Capabilities 修改一個能力、Impact 新增兩個與修改四個檔案）
- **THEN** 閱讀卡籤槽為「Refactor」；數字磚依序為「解法步驟 6」「非目標 5」「能力 1」「影響的檔案 6」，「替代方案」因已滿四格不出現

#### Scenario: 設計的決策卡與數字磚

- **WHEN** 檢視設計文件，其 Decisions 下有「### D1 面板底與卡片層」等六個三級標題、Goals 五條與 Non-Goals 三條、Risks / Trade-offs 四條、Open Questions 為「無」
- **THEN** 數字磚為「決策 6」「目標／非目標 5 ／ 3」「風險與取捨 4」「開放問題 0」；「決策」段六張卡預設收合、卡頭含「D1」籤與列點數，點 D1 卡頭後其內文展開；「風險與取捨」段為四個 warning 提示框；本頁目錄列出六個決策標題


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
#### Scenario: 不同專案的散文提案預設列式

- **WHEN** 使用者分別開啟專案A的活動提案與專案B的已封存提案，Why含一段散文接兩項列點
- **THEN** 兩者皆依原序顯示三項連續編號，保留原文與巢狀內容，不需設定，不修改任何來源檔案


<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 任務群組標題與章節標籤同款式

變更詳情頁與已封存檢視的任務文件 SHALL 以 tasks.md 的二級標題分群，每個群組一張卡：卡頭為 16px 粗體的群組標題（文字照來源呈現，不翻譯、不改寫；標題開頭的「N.」序號 SHALL 成等寬序號籤）、右端為該群組的小進度「已完成/總數」；卡頭 SHALL 帶本頁目錄錨點；第一個二級標題之前的任務 SHALL 以無卡頭的清單呈現。每條任務列 SHALL 拆成「標題／內文／驗證」三層：描述以首個「驗證：」切成前段與驗證，前段再以首個全形「：」切成標題與內文；切不到「：」時整段為標題、無內文；切不到「驗證：」時無驗證層；標題緊接的「（design D1、D3）」或「(design D1, D3)」括號 SHALL 自標題剝除，其中的 D 編號成可點的 D 籤（點擊切到設計文件並展開、捲到對應決策卡；設計文件沒有該編號的決策卡時只切換文件），括號內其餘文字歸入內文首行。任務列的標題層 SHALL 恆顯示勾選框、序號、標題、D 籤與手動籤；有內文或驗證的任務列 SHALL 有列尾箭頭鈕展開或收合內文與驗證（展開態以 aria-expanded 標示），沒有內文也沒有驗證的任務列 SHALL NOT 有箭頭鈕。任務列的預設展開 SHALL 由勾選狀態決定：未勾選者展開、已勾選者收合。勾選狀態改變時——不論來自點列勾選、工具列「全部已完成」或「重置任務」、勾選寫回失敗的回滾，或外部檔案變更後的重新載入——該列 SHALL 跟著翻：改為已勾選即收合、改為未勾選即展開。使用者以箭頭鈕手動展開或收合的狀態 SHALL 維持到該任務的勾選狀態下次改變為止，拖曳排序與不改變該任務勾選狀態的重新載入 SHALL NOT 重設它，排序後跟著原任務（任務以 tasks.md 行尾的穩定 ID 識別；沒有穩定 ID 的任務以序數識別，排序後展開狀態留在原位置）；離開任務文件再回來時 SHALL 依勾選狀態重新決定。拖曳時浮起的列只畫標題層。標題與內文中的行內程式碼 SHALL 以等寬籤呈現。整列為勾選框的 label、拖曳把手、批次工具列、「下一個未完成」的 n 快捷鍵與定位捲動 SHALL 不受款式變更影響；手動任務列的淡底與手動籤依「任務列的手動任務徽章」；唯讀（已封存）時勾選框停用、展開收合照常，預設展開同樣依勾選狀態決定。

閱讀卡的籤槽 SHALL 為「全部完成」（全部任務已勾選時）或「已完成/總數」；數字磚依序 SHALL 為「任務」（總數）、「已完成」、「手動」（帶手動籤的任務數）、「群組」（二級標題數）；沒有任何任務時數字磚與籤槽缺席。有未勾選的手動任務時 SHALL 有一個以手動色（見「介面狀態語意色分層」）呈現的提示框「N 個手動任務」——N 為未勾選的手動任務數，內文只列出未勾選手動任務的序號與標題（標題的行內程式碼以等寬籤呈現）並說明要由使用者親自做完再勾，已勾選的手動任務 SHALL NOT 出現在提示框；每一列 SHALL 附「前往」鈕，點擊後任務文件捲到該任務列並短暫高亮（與「下一個未完成」同一種高亮），唯讀（已封存）時照常可用。所有手動任務都已勾選、或沒有手動任務時，提示框 SHALL 缺席，SHALL NOT 改以其他提示（例如「手動任務都完成了」）取代。提示框 SHALL 隨勾選狀態更新：不論勾選狀態來自點列勾選、工具列「全部已完成」或「重置任務」、勾選寫回失敗的回滾，或外部檔案變更後的重新載入，提示框的 N 與列表 SHALL 與當下 tasks.md 的勾選狀態一致。「手動」數字磚 SHALL 維持為帶手動籤的任務總數，不隨勾選變動。

#### Scenario: 群組標題款式一致

- **WHEN** 檢視含「## 1. 原語」「## 2. 骨架」兩個群組的任務文件
- **THEN** 兩張群組卡的卡頭為 16px 粗體標題、左端序號籤「1」「2」、右端各自的小進度；標題文字與來源一致（不含序號後的多餘空白）；本頁目錄列出兩個群組標題

#### Scenario: 互動行為不變

- **WHEN** 在款式調整後的任務文件點整列勾選任務、以把手拖曳排序、按工具列「全部已完成」並按 n
- **THEN** 勾選與排序寫回結果與調整前一致；工具列三鍵與 n 快捷鍵行為不變；點任務列尾的展開鈕不切換勾選

#### Scenario: 任務列三層與展開

- **WHEN** 檢視未勾選的任務「- [ ] 1.1 新增 `packages/ui/src/docModel.ts`（design D1）：匯出 splitH2Sections；驗證：npm test -w packages/ui -- docModel 全綠」
- **THEN** 標題層顯示標題「新增 `packages/ui/src/docModel.ts`」（程式碼為等寬籤）與 D 籤「D1」；該列預設展開，標題下方顯示內文「匯出 splitH2Sections」與驗證「npm test -w packages/ui -- docModel 全綠」，列尾鈕的 aria-expanded 為 true；點列尾鈕後內文與驗證收起、aria-expanded 為 false；同一描述改為已勾選（「- [x]」）時，該列預設只顯示標題層、aria-expanded 為 false

##### Example: 三層切分

| 任務描述 | 標題 | 內文 | 驗證 | D 籤 |
| -------- | ---- | ---- | ---- | ---- |
| 新增 `a.ts`（design D1）：匯出 x；驗證：npm test | 新增 `a.ts` | 匯出 x； | npm test | D1 |
| 補 i18n 詞條；驗證：i18n 測試全綠 | 補 i18n 詞條； | （無） | i18n 測試全綠 | （無） |
| 對齊匯出 | 對齊匯出 | （無） | （無） | （無） |
| 改 TaskList（design D5；spec「任務群組標題與章節標籤同款式」）：群組成卡 | 改 TaskList | spec「任務群組標題與章節標籤同款式」＋群組成卡 | （無） | D5 |

#### Scenario: 依勾選狀態決定預設展開

- **WHEN** 開啟任務文件，群組內有 3.1（未勾選，有內文）、3.2（已勾選，有內文）、3.3（未勾選，無內文也無驗證）
- **THEN** 3.1 展開並顯示內文、3.2 只顯示標題層、3.3 沒有列尾箭頭鈕；已封存詳情頁開啟同一份任務文件時，三列的展開狀態相同

##### Example: 勾選狀態與展開

| 勾選狀態 | 有內文或驗證 | 預設 | 勾選狀態改變後 |
| -------- | ------------ | ---- | -------------- |
| 未勾選 | 有 | 展開 | 勾選 → 收合 |
| 已勾選 | 有 | 收合 | 取消勾選 → 展開 |
| 未勾選或已勾選 | 無 | 無箭頭鈕 | 無箭頭鈕 |

#### Scenario: 勾選狀態改變時展開跟著翻

- **WHEN** 使用者點整列勾選展開中的 3.1，接著取消勾選 3.1；之後按工具列「全部已完成」，再按「重置任務」
- **THEN** 勾選 3.1 後 3.1 收合、取消勾選後 3.1 展開；「全部已完成」後所有有內文或驗證的任務列收合，「重置任務」後全部展開；勾選框的勾選與寫回結果與本需求引入前一致

#### Scenario: 寫回失敗與外部變更同樣跟著勾選狀態

- **WHEN** 使用者勾選 3.1 而寫回失敗、勾選框回滾為未勾選；之後 agent 經 CLI 勾掉 3.1，外部檔案變更觸發任務文件重新載入
- **THEN** 勾選當下 3.1 收合、回滾後 3.1 回到展開；重新載入讀到 3.1 已勾選後 3.1 收合

#### Scenario: 手動展開收合維持到勾選狀態改變

- **WHEN** 使用者以列尾箭頭鈕手動收起未勾選的 3.1、手動展開已勾選的 3.2，把 3.1 拖到 3.2 之後，之後 agent 經 CLI 勾掉另一條任務 3.4 而觸發重新載入
- **THEN** 3.1 維持收合、3.2 維持展開，拖曳與重新載入都不重設兩列的展開狀態；使用者接著勾選 3.1 再取消勾選，3.1 展開；使用者切到別的文件再回到任務文件時，3.2 依已勾選回到收合

#### Scenario: 手動提示框前往任務列

- **WHEN** 任務文件的手動提示框列出「3.2 手動驗收 `dev:desktop` 畫面」，使用者點該列的「前往」鈕
- **THEN** 標題中的 `dev:desktop` 以等寬籤呈現；任務文件捲到 3.2 任務列並短暫高亮，勾選狀態不變

#### Scenario: D 籤跳到設計決策卡

- **WHEN** 使用者於任務文件點任務列的「D3」籤
- **THEN** 左欄「設計」節點作用中、閱讀卡呈現設計文件，「D3」決策卡展開且閱讀區捲到該卡頭；設計文件沒有 D3 卡時只切換到設計文件並自頂端呈現

#### Scenario: 數字磚與手動提示框

- **WHEN** 檢視 3 個群組、9 個任務（7 個已勾選）、其中 3.2 帶手動籤的任務文件
- **THEN** 籤槽為「7/9」；數字磚為「任務 9」「已完成 7」「手動 1」「群組 3」；手動色提示框標題「1 個手動任務」且內文列出「3.2」與其標題；全部勾選後籤槽改為「全部完成」、數字磚仍為「手動 1」，且提示框缺席

#### Scenario: 部分勾選時提示框只列未勾選的手動任務

- **WHEN** 任務文件有 4 個帶手動籤的任務 4.3、5.8、6.9、7.6，其中只有 4.3 已勾選；使用者接著點整列勾選 5.8、6.9、7.6
- **THEN** 勾選前提示框標題「3 個手動任務」、只列出 5.8、6.9、7.6，不列 4.3；三條全部勾選後提示框缺席、沒有任何替代提示；兩個時點的數字磚都是「手動 4」；使用者再取消勾選 6.9 時，提示框重新出現，標題「1 個手動任務」且只列出 6.9

##### Example: 勾選狀態與提示框

| 已勾選的手動任務 | 提示框標題 | 提示框列出 | 「手動」數字磚 |
| ---------------- | ---------- | ---------- | -------------- |
| 無 | 4 個手動任務 | 4.3、5.8、6.9、7.6 | 4 |
| 4.3 | 3 個手動任務 | 5.8、6.9、7.6 | 4 |
| 4.3、5.8、6.9、7.6 | （提示框缺席） | （無） | 4 |


<!-- @trace
source: manual-callout-unchecked-only
updated: 2026-10-14T13:51:46+08:00
-->

---
### Requirement: 規格頁提供清單、搜尋與展開檢視

專案欄的「規格」項 SHALL 進入規格頁：頁標題區（標題「規格」、灰字說明、右端全圓搜尋框）之下為一張白底細框列表卡，以列式清單呈現全部正式規格：每列 SHALL 含 spec 名稱（等寬字型）、hover 才顯示的複製名稱鈕、一行截斷的 Purpose 摘要（為封存流程佔位文字時改顯警示樣式「Purpose 待補」）、右端的需求數、溯源變更數、最後修改相對時間（自檔案系統 mtime 衍生，天級：今天／昨天／N 天前；mtime 不可得時該資訊缺席）與 › 收尾。搜尋框 SHALL 以大小寫不敏感的名稱子字串即時過濾清單。點列 SHALL 開啟該規格的詳情頁（與手冊頁出處所開的同一頁，呈現規則見「桌面 app 呈現 change 與 spec 的清單與內容」）；全文 SHALL 於開啟時才載入。無 spec 的專案與搜尋無結果 SHALL 各於列表卡內顯示空狀態文案。規格頁 SHALL 為唯讀，SHALL NOT 提供任何規格寫入操作。

#### Scenario: 進入規格頁顯示卡片清單

- **WHEN** 於含多個正式規格的專案點專案欄「規格」
- **THEN** 導覽項呈 active 樣式，主區為頁標題「規格」與搜尋框，列表卡內一列一份規格，各含等寬名稱、Purpose 摘要、需求數、溯源數與最後修改相對時間，列尾為 ›

#### Scenario: 搜尋列名稱過濾

- **WHEN** 於搜尋框輸入部分 spec 名稱
- **THEN** 清單即時縮至名稱含該子字串（大小寫不敏感）的列；清空輸入後清單還原

##### Example: 過濾行為

| 既有 specs | 輸入 | 顯示 |
| ---------- | ---- | ---- |
| desktop-app、desktop-config、node-sdk | desktop | desktop-app、desktop-config |
| desktop-app、desktop-config、node-sdk | SDK | node-sdk |
| desktop-app、desktop-config、node-sdk | zzz | 無結果空狀態 |

#### Scenario: 點列開啟規格詳情頁

- **WHEN** 點一列規格
- **THEN** 主區換成該 spec 的規格詳情頁（先呈載入態），需求逐條分段、溯源分頁列出來源變更（依 @trace source 去重）；關閉後清單與搜尋字串維持不變

#### Scenario: 複製名稱

- **WHEN** 滑鼠停在一列上並點其複製名稱鈕
- **THEN** spec 名稱寫入剪貼簿並短暫顯示已複製回饋，且不開啟檢視

#### Scenario: 無 spec 專案顯示空狀態

- **WHEN** 於無任何正式規格的專案進入規格頁
- **THEN** 列表卡內顯示空狀態文案而非空白頁

#### Scenario: 外部變更後反映

- **WHEN** 規格頁開啟期間外部寫者修改某 spec 的 spec.md
- **THEN** 世代重載後列的修改時間更新，開啟中的規格詳情頁反映新全文


<!-- @trace
source: desktop-readonly-detail-pages
updated: 2026-10-10T19:39:25+08:00
-->

---
### Requirement: 變更的來源討論多值呈現

變更連結多份討論時(meta 的 from_discussion 為逗號清單),變更卡 SHALL 維持單一討論徽章、以清單第一個(出身)討論為代表,徽章提示 SHALL 列出全部來源討論;變更詳情頁左欄身分區的「來自」列 SHALL 固定顯示清單第一個(出身)討論,其餘 SHALL 收進「+N」數字籤(N 為未直接顯示的數量),點擊 +N SHALL 開啟浮層列出其餘全部來源討論——浮層項 SHALL 呈現該討論的 slug 與 topic,點擊任一項 SHALL 開啟該討論的詳情頁;全部來源討論 SHALL 經直接顯示的籤或浮層可及並可互跳。同源變更清單 SHALL 以「雙方來源討論集合交集非空」判定收錄,其呈現 SHALL 比照來源討論籤(「同源」列顯示第一顆,其餘收「+N」浮層,點擊浮層項開啟該變更的詳情)。僅一份來源討論的變更 SHALL NOT 出現來源討論的「+N」籤。

#### Scenario: 多來源徽章以出身討論為代表

- **WHEN** 看板呈現來源討論清單為兩份(出身在前)的變更卡
- **THEN** 卡片帶單一討論徽章、代表清單第一份(出身)討論,徽章提示列出全部兩份來源討論

#### Scenario: 詳情頁列出全部來源討論

- **WHEN** 開啟該變更的詳情頁
- **THEN** 左欄「來自」列直接顯示出身討論籤與「+N」籤;點擊 +N 開啟浮層,列出其餘來源討論(各項含 slug 與 topic),點擊任一項開啟該討論的詳情頁

#### Scenario: 同源以來源討論交集判定

- **WHEN** 變更 A 的來源討論清單為 d1, d2、變更 B 的來源討論清單僅含 d2
- **THEN** A 與 B 互為同源變更,出現在彼此詳情頁的同源變更清單

##### Example: 交集判定表

| 變更 A 來源 | 變更 B 來源 | 是否同源 |
| ----------- | ----------- | -------- |
| d1, d2      | d2          | 是       |
| d1          | d1          | 是       |
| d1, d2      | d3          | 否       |
| (無)        | d1          | 否       |

#### Scenario: 單一來源無溢出籤

- **WHEN** 看板呈現僅一份來源討論的變更卡並開啟其詳情頁
- **THEN** 卡片維持單一討論徽章;詳情頁「來自」列僅顯示該討論的籤,無「+N」籤


<!-- @trace
source: desktop-detail-doc-tree
updated: 2026-10-12T13:39:16+08:00
-->

---
### Requirement: 看板卡片浮現待重新反映徽章

desktop 看板的變更卡片 SHALL 於該變更 meta 的 restale_from 非空時，顯示一枚「待重新反映」徽章，提示該變更反映的討論已被重新結論、待 re-ingest。徽章的資料源 SHALL 為變更 meta 的 restale_from 欄位，經桌面看板查詢路徑的 Rust 變更序列化曝為資料欄、透過 tauriDataSource 傳至前端、由 packages/ui 的看板卡片元件渲染——全程僅讀既存 meta 欄位，SHALL NOT 於載入時掃描討論記錄。restale_from 為空或缺席時卡片 SHALL NOT 顯示該徽章。徽章 SHALL 與既有卡片視覺語言（主題化樣式）一致。此浮現不改變看板欄位派生規則（全完成＞有 started＞其餘）——徽章與欄位歸屬正交。

#### Scenario: 過期變更卡片顯示徽章

- **WHEN** 看板渲染一個 meta 帶非空 restale_from 的變更卡片
- **THEN** 該卡片顯示「待重新反映」徽章；徽章不影響卡片所在看板欄位

#### Scenario: 非過期變更卡片無徽章

- **WHEN** 看板渲染一個 meta 無 restale_from（或為空）的變更卡片
- **THEN** 該卡片不顯示「待重新反映」徽章，其餘呈現與本變更前一致

<!-- @trace
source: reconclude-restale
updated: 2026-07-09
code:
  - apps/desktop/core/src/query.rs
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/reconclude_restale.rs
  - crates/speclink-core/assets/skills/ingest.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 看板變更卡呈現建立者與關係提示

看板變更卡 SHALL 顯示建立者（createdBy）頭像——以建立者首字母圓標呈現，meta 無 created_by 時省略。變更卡的關係指示（來自討論、待重新反映）SHALL 於 hover 以主題化提示（shadcn Tooltip，取代原生 title）呈現對應資訊：來自討論指示於卡片來自討論時 SHALL 列出全部來源討論，待重新反映指示於卡片帶 restale 旗標時 SHALL 列出待重新反映的來源；無對應關係時該指示 SHALL 缺席。提示內容 SHALL 與原生 title 一致（改以主題化樣式呈現）。

#### Scenario: 變更卡顯示建立者頭像

- **WHEN** 看板呈現一個 meta 含 created_by 的變更卡
- **THEN** 卡片顯示該建立者的首字母圓標頭像

#### Scenario: 無建立者時省略頭像

- **WHEN** 變更卡的 meta 無 created_by
- **THEN** 卡片不顯示建立者頭像

#### Scenario: 關係指示以主題化 hover 提示呈現

- **WHEN** 使用者 hover 一個來自討論之變更卡的來自討論指示
- **THEN** 以主題化提示（shadcn Tooltip）列出全部來源討論，取代原生 title 呈現

<!-- @trace
source: desktop-card-identity
updated: 2026-07-09
code:
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/query.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/discuss.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 拖曳封存落點以浮層呈現

拖曳看板卡片時的封存落點 SHALL 以浮層呈現:疊於看板欄列右緣上方、不參與欄列佈局——落點浮現與消失時各欄寬度 SHALL 維持不變。落點 SHALL 僅於拖曳**已就緒**變更卡時浮現;拖曳非已就緒變更卡或討論卡時 SHALL NOT 浮現,且 SHALL NOT 造成任何佈局變動——非已就緒變更卡的拖曳僅得欄內排序。拖曳靠近落點時 SHALL 呈現可放開的視覺回饋。已就緒卡拖至落點放開的封存確認流程與拖排語意 SHALL 維持 board-card-order 規格所定行為不變。

#### Scenario: 拖曳已就緒變更卡時浮層浮現且欄寬不變

- **WHEN** 使用者開始拖曳一張已就緒的變更卡
- **THEN** 封存落點以浮層疊於看板右緣浮現,各欄(含討論欄)寬度與拖曳前一致;放開或取消拖曳後浮層消失、欄寬仍不變

#### Scenario: 拖曳非已就緒變更卡時落點不浮現

- **WHEN** 使用者開始拖曳一張提案中或進行中的變更卡
- **THEN** 封存落點不浮現,看板佈局無任何變動;放開時僅依欄內拖排語意處理

#### Scenario: 拖曳討論卡時落點不浮現

- **WHEN** 使用者開始拖曳討論卡
- **THEN** 封存落點不浮現,看板佈局無任何變動

#### Scenario: 已就緒卡拖至浮層落點放開觸發既有封存流程

- **WHEN** 使用者拖曳已就緒變更卡至浮層落點放開
- **THEN** 觸發既有封存確認流程,行為與未改版前一致


<!-- @trace
source: archive-readiness-gating
updated: 2026-07-31
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 看板卡片統一解剖學

看板全尺寸卡（變更卡與討論卡）SHALL 採統一三列骨架：識別列（標題＋複製鈕＋右端 meta icons）、描述列（一行截斷，無內容時缺席）、meta 列。標題 SHALL 以等寬字型呈現（變更名稱與討論 slug 同為可複製把手）。標題 SHALL 恆為單行——長於可用寬度時 SHALL 就地截斷，SHALL NOT 折行、SHALL NOT 強制斷字；截斷處 SHALL 以省略號收尾，SHALL NOT 以漸層淡出或硬切呈現；標題未被截斷時 SHALL 完整顯示，SHALL NOT 出現省略號。複製鈕 SHALL 與標題同列尾隨於標題文字之後，SHALL NOT 因標題過長而落至次行、SHALL NOT 被壓縮，meta icons 維持靠右；SHALL NOT 將複製鈕推至列右緣。變更卡描述列 SHALL 顯示 proposal Why 首句（一行截斷）；proposal 缺席、Why 區段缺席或為空時描述列 SHALL 缺席。描述資料 SHALL 由變更清單 payload 一次帶出，SHALL NOT 逐卡讀取 proposal 全文；該欄位屬呈現層輔助欄位，不屬 CLI --json 對齊範圍。建立者 SHALL 以頭像圓點呈現且 hover 顯示全名，SHALL NOT 於卡面直出全名文字；createdBy 缺席時圓點缺席。狀態 chip SHALL 僅在所在位置無法表達狀態時出現：討論卡（討論欄一欄兩態）帶狀態 chip，變更卡（所在欄即階段）SHALL NOT 帶狀態 chip。

#### Scenario: 變更卡三列骨架

- **WHEN** 看板載入一個 proposal Why 首句非空、createdBy 存在、任務 5/21 的變更
- **THEN** 變更卡識別列以等寬字型顯示名稱且複製鈕緊跟名稱文字後（hover 顯現、點擊寫入名稱至剪貼簿且不開詳情頁）、右端呈建立者圓點；描述列一行截斷顯示 Why 首句；meta 列顯示進度條與 5/21；卡上無狀態 chip

#### Scenario: 變更卡無 Why 內容時描述列缺席

- **WHEN** 某變更無 proposal.md（或 Why 區段為空）
- **THEN** 該變更卡不顯示描述列，識別列與 meta 列照常呈現，看板不因該筆缺件而報錯

#### Scenario: 長標題單行截斷時複製鈕仍在同列

- **WHEN** 變更名稱長於卡片可用寬度
- **THEN** 標題維持單行並於可用寬度處截斷、以省略號收尾，複製鈕緊跟標題文字之後留在同一列且維持可點，右端 meta icons 不被擠出卡外

#### Scenario: 討論卡長 slug 與變更卡同一收尾

- **WHEN** 討論 slug 長於卡片可用寬度
- **THEN** slug 維持單行截斷並以省略號收尾、不強制斷字折行，複製 slug 鈕留在同一列，收尾行為與變更卡標題一致

#### Scenario: 短標題完整顯示

- **WHEN** 變更名稱短於卡片可用寬度
- **THEN** 標題完整顯示且不出現省略號，複製鈕緊跟標題最後一個字元後


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 規格與封存卡片收合資訊

規格列 SHALL 於標題文字後緊跟 hover 才顯示的複製名稱鈕，並顯示需求數、溯源變更數與相對修改時間；Purpose 摘要 SHALL 取正式規格 Purpose 區段首個非空行一行截斷顯示，當 Purpose 為封存流程產生的佔位文字時 SHALL 改顯「Purpose 待補」警示樣式而非佔位原文。封存變更列 SHALL 於最左顯示日期、標題（change 名）後緊跟複製鈕、任務數徽章（未全完成 SHALL 以警示樣式呈現，與全完成可辨；無 tasks.md 者不顯示徽章）、觸及規格數、建立者標記（hover 顯示全名）與來源討論標記（無來源討論時缺席）；標題下方 SHALL 有描述列，一行截斷顯示封存 proposal 的 Why 首句（清單資料的 Why 首句欄位缺席時描述列 SHALL 缺席、列退回單行）——與看板變更卡的描述列同構。封存討論列 SHALL 以 slug 為標題（等寬強調、緊跟複製 slug 鈕），topic SHALL 降為標題下方的描述列一行截斷顯示，並顯示日期、輪數與衍生變更數——與看板討論卡的 slug 標題＋topic 描述同構（slug 作為討論識別錨點屬 LANGUAGE.md 受控例外的既有適用範圍）。三種列的計數 meta（需求數、溯源變更數、觸及規格數、衍生變更數）SHALL 以一致的「icon＋數字」樣式呈現——SHALL NOT 混用 pill 底色或無 icon 的圓圈數字；任務數徽章 SHALL 維持 pill 樣式與配色分級（狀態語意例外）。上述資訊 SHALL 於列上（未開啟任何詳情頁）即可見；三種列 SHALL 以共用列元件呈現（整列可點、細線分隔、hover 微灰、右端 › 收尾），資料 SHALL 由清單載入一次帶出，SHALL NOT 逐列讀取文件全文。

#### Scenario: 規格卡收合資訊

- **WHEN** 規格頁載入一份含 7 條 Requirement、Purpose 已填寫、溯源自 3 個變更的規格
- **THEN** 該列顯示需求數 7、溯源變更數 3、Purpose 首行摘要與相對修改時間，全程未讀取其他規格全文

#### Scenario: Purpose 佔位顯示待補提示

- **WHEN** 某規格的 Purpose 區段仍為封存流程產生的佔位文字（精確偵測字串由實作與封存產生器以同一單元測試釘住）
- **THEN** 該規格列以警示樣式顯示「Purpose 待補」，不顯示佔位原文

#### Scenario: 封存變更卡任務徽章配色分級

- **WHEN** 已封存頁含一筆 20/21 未全完成與一筆 48/48 全完成的封存變更
- **THEN** 前者任務徽章以警示樣式顯示 20/21，後者以一般樣式顯示 48/48，列上即可一眼區辨

#### Scenario: 封存討論卡顯示衍生變更數與複製 slug

- **WHEN** 已封存頁討論節含一筆轉出 2 個變更的封存討論
- **THEN** 該列顯示衍生變更數 2 與 hover 才顯示的複製 slug 鈕，點擊複製鈕將 slug 寫入剪貼簿且不開啟詳情頁

#### Scenario: 封存變更卡顯示 Why 首句描述列

- **WHEN** 已封存頁載入一筆封存 proposal 的 Why 首句可得的封存變更、與一筆 proposal 缺席的封存變更
- **THEN** 前者於標題下方一行截斷顯示該 Why 首句；後者描述列缺席、列退回單行；全程未逐卡讀取文件全文（描述資料由清單載入一次帶出）

#### Scenario: 封存討論卡以 slug 為標題

- **WHEN** 已封存頁討論節載入一筆 slug 為 board-search-bar、topic 為「看板搜尋列」的封存討論
- **THEN** 該列標題以等寬強調顯示 board-search-bar 並緊跟複製 slug 鈕，「看板搜尋列」於標題下方以描述列一行截斷顯示


<!-- @trace
source: desktop-readonly-detail-pages
updated: 2026-10-10T19:39:25+08:00
-->

---
### Requirement: 清單最新在前與換頁瀏覽

規格頁與已封存頁的清單 SHALL 以最新在前排序：封存變更依封存日期新→舊（同日由封存目錄名字典序降冪涵蓋）、封存討論依建立日期新→舊（同日以 slug 字母升冪決勝）、規格依最後修改時間新→舊（修改時間不可得者一律排最後、以名稱字母升冪決勝）。上述清單 SHALL 換頁瀏覽，每頁預設 20 筆。清單頁版面 SHALL 填滿主區高度且頁面本身不縱向捲動：頁標題區與搜尋框固定於頂部，列表卡填滿其餘高度、列於卡內縱向捲動（已封存頁的分頁列為列表卡頂部）。清單非空時列表卡底部 SHALL 常駐一條換頁工具列（不隨列捲動移出視窗）：左側為「第 a–b 筆，共 N 筆」與每頁筆數下拉（20／50／100，預設 20）；過濾後筆數超過每頁筆數時右側 SHALL 依序有第一頁鈕、上一頁鈕、頁碼鈕列（作用中頁碼主色淡底；總頁數超過 7 時恆列首尾頁與目前頁前後各一頁、其餘以省略號收攏）、下一頁鈕、最後一頁鈕與「跳到 __ 頁」數字輸入（Enter 跳頁、越界鉗制至 1 與末頁、非數字忽略）；第 1 頁時第一頁鈕與上一頁鈕 SHALL 不可用、末頁時下一頁鈕與最後一頁鈕 SHALL 不可用；筆數不超過每頁筆數時右側 SHALL 缺席。改每頁筆數 SHALL 重算總頁數並把目前頁碼鉗制至末頁。桌面 app SHALL 記住每頁筆數並跨啟動保留：規格頁、已封存「變更」節與「討論」節各自記住，記憶只存在 app 本機、SHALL NOT 寫入任何專案目錄；記住的值缺席、損毀或不在 20／50／100 之內時 SHALL 退回 20，本機儲存不可用時 SHALL 只失去跨啟動記憶、不顯示錯誤。頁碼與搜尋字串 SHALL NOT 跨啟動保留。搜尋字串變更 SHALL 使清單回到第 1 頁；換頁後內部捲動容器 SHALL 捲回頂部；清單縮短使目前頁碼越界時 SHALL 鉗制至末頁而非顯示空頁。已封存頁「變更」「討論」兩子頁籤 SHALL 維持各自獨立的換頁控制列與頁碼。排序與換頁 SHALL 僅屬前端呈現，清單載入行為與清單資料欄位 SHALL NOT 因此改變。

#### Scenario: 封存變更最新在前

- **WHEN** 已封存頁含封存於 2026-07-04、2026-07-11、2026-07-08 的三筆封存變更
- **THEN** 「變更」子頁籤清單依 2026-07-11、2026-07-08、2026-07-04 順序呈現

#### Scenario: 規格修改時間缺席排最後

- **WHEN** 規格頁含修改時間可得的規格 alpha（3 天前）、beta（今天）與修改時間不可得的 zeta、delta
- **THEN** 清單順序為 beta、alpha、delta、zeta——可得者新→舊在前，不可得者殿後並依名稱字母序排列

#### Scenario: 超過 20 筆顯示換頁控制列

- **WHEN** 已封存頁「變更」子頁籤過濾後有 45 筆
- **THEN** 清單顯示第 1 頁的 20 筆，列表卡底部工具列顯示「第 1–20 筆，共 45 筆」、每頁 20、第一頁鈕、上一頁鈕、頁碼 1 2 3（1 為作用中）、下一頁鈕、最後一頁鈕與跳頁輸入，第一頁與上一頁不可用；點頁碼 2 顯示第 21–40 筆並使卡內捲動容器捲回頂部；於跳頁輸入 9 按 Enter 停在第 3 頁

#### Scenario: 換頁控制列不捲動即可觸及

- **WHEN** 已封存頁「變更」子頁籤過濾後超過 20 筆，視窗高度僅能顯示部分卡片，且使用者尚未捲動
- **THEN** 工具列已於列表卡底部可見且可點擊；捲動列（卡內容器）時，頁標題區、搜尋框、分頁列與工具列維持固定位置，不隨內容移出視窗

#### Scenario: 20 筆以內換頁控制列缺席

- **WHEN** 規格頁清單過濾後僅 13 筆
- **THEN** 全部 13 筆單頁呈現，工具列只顯示「第 1–13 筆，共 13 筆」與每頁筆數下拉，無第一頁／上一頁／下一頁／最後一頁鈕、無頁碼鈕與跳頁輸入

#### Scenario: 搜尋字串變更回到第 1 頁

- **WHEN** 使用者於「變更」子頁籤第 3 頁修改搜尋字串
- **THEN** 清單回到第 1 頁顯示新過濾結果的最前 20 筆

#### Scenario: 改每頁筆數鉗制頁碼

- **WHEN** 已封存頁「變更」節過濾後 45 筆、使用者位於每頁 20 的第 3 頁，把每頁筆數改為 50
- **THEN** 清單單頁顯示 45 筆、工具列顯示「第 1–45 筆，共 45 筆」、頁碼鈕消失；改回 20 後回到第 1 頁

#### Scenario: 第一頁與最後一頁鈕

- **WHEN** 已封存頁「變更」節過濾後 45 筆、每頁 20，使用者位於第 2 頁，點最後一頁鈕，再點第一頁鈕
- **THEN** 點最後一頁鈕後清單顯示「第 41–45 筆，共 45 筆」、下一頁鈕與最後一頁鈕不可用；點第一頁鈕後顯示「第 1–20 筆，共 45 筆」、第一頁鈕與上一頁鈕不可用

#### Scenario: 每頁筆數跨啟動記住

- **WHEN** 使用者在規格頁把每頁筆數改為 50，在已封存頁「討論」節改為 100，然後關閉並重新開啟桌面 app
- **THEN** 規格頁每頁 50、已封存「討論」節每頁 100、已封存「變更」節仍為每頁 20；三個清單都從第 1 頁、空搜尋開始；專案目錄內沒有任何檔案因此變動

##### Example: 記住的值讀回

| 本機記憶內容 | 規格頁 | 已封存「變更」節 | 已封存「討論」節 |
| ------------ | ------ | ---------------- | ---------------- |
| （缺席） | 20 | 20 | 20 |
| specs 50、archivedDiscussions 100 | 50 | 20 | 100 |
| specs 0、archivedChanges 37 | 20 | 20 | 20 |
| 無法解析的文字 | 20 | 20 | 20 |


<!-- @trace
source: desktop-list-pages-reskin
updated: 2026-10-09T22:35:44+08:00
-->

---
### Requirement: 看板全域操作成功靜默、失敗以 toast 浮層呈現

桌面 app 的看板全域操作——刪除變更、封存變更、封存討論、拖排卡片、開啟專案、初始化專案——成功時 SHALL NOT 呈現任何文字結果訊息；操作結果由畫面自身表達（卡片自看板消失、側欄「已封存」計數遞增、卡片位於新位置、切換至該專案）。視窗頂欄 SHALL NOT 呈現任何操作結果文字。

上述任一操作失敗時，app SHALL 以 toast 浮層呈現失敗訊息。訊息 SHALL 包含操作對象的主詞與 core 回報的原始錯誤內容，SHALL NOT 吞掉或改寫該錯誤內容。toast SHALL 疊於詳情頁與確認對話框之上——詳情頁或確認框開啟中發生失敗時，toast SHALL 完整可見且不被遮蔽。toast SHALL 於固定時距後自動消失，並 SHALL 提供關閉鈕供提前關閉。同時 SHALL 僅呈現一則 toast；新訊息 SHALL 取代前一則並重置其自動消失時距。

toast 的表面、前景、邊框、圓角、陰影與字體 SHALL 沿用桌面 app 的既有語意設計 tokens。失敗 toast SHALL 保持中性 card 表面與可讀前景，並以 destructive token 強調錯誤圖示與邊框；SHALL NOT 使用獨立於主介面的內建明暗色票或另建 raw 色票。作業系統切換淺色或深色模式時，toast SHALL 由同一組語意 tokens 取得該模式的對應值。

失敗後看板 SHALL 刷新回磁碟現況，SHALL NOT 保留未落檔的假象順序或狀態。

封存變更成功時 SHALL 關閉詳情頁回看板。成功之所以能靜默，前提是畫面可見地表達了結果；詳情頁開啟時看板不可見，該前提即不成立。

#### Scenario: 詳情頁開啟中封存失敗仍完整可見

- **WHEN** 使用者於某 change 的詳情頁點按「封存」，而封存因前置未滿足失敗
- **THEN** toast 疊於詳情頁之上完整呈現，內容含該 change 名稱與 core 回報的錯誤訊息；詳情頁維持開啟

#### Scenario: 失敗 toast 沿用主介面語意色彩

- **WHEN** 桌面 app 在淺色或深色模式呈現任一看板全域操作的失敗 toast
- **THEN** toast 使用該模式的 card 表面、可讀前景、border、radius、陰影與 host 字體，錯誤圖示與邊框使用 destructive 語意；toast 不呈現獨立於主介面的內建明暗色票

#### Scenario: 系統明暗模式切換後 toast 使用對應 tokens

- **WHEN** 作業系統由淺色切換為深色或由深色切換為淺色，之後看板全域操作失敗
- **THEN** 新呈現的 toast 使用同一組語意 token 名稱在目前模式下的值，訊息內容、逾時、關閉與單槽取代行為不變

#### Scenario: 刪除成功不呈現任何文字訊息

- **WHEN** 使用者確認刪除某變更且刪除成功
- **THEN** 該卡片自看板消失、詳情頁關閉；視窗頂欄與 toast 皆無任何文字訊息

#### Scenario: 封存成功關閉詳情頁且不呈現文字訊息

- **WHEN** 使用者於某 change 的詳情頁點按「封存」且封存成功
- **THEN** 詳情頁關閉、卡片自看板消失、側欄「已封存」計數遞增；無任何文字訊息

#### Scenario: toast 自動消失且可提前關閉

- **WHEN** 失敗 toast 呈現後使用者未操作
- **THEN** toast 於固定時距後自動消失；該時距內使用者點按關閉鈕時 toast 立即關閉

#### Scenario: 開啟專案失敗帶主詞呈現且不切換專案

- **WHEN** 使用者選定的資料夾無法開啟為專案
- **THEN** toast 呈現該路徑與失敗原因；目前專案維持不變

#### Scenario: 拖排成功不呈現文字訊息

- **WHEN** 使用者於看板同一欄內拖動卡片至新位置放開且寫回成功
- **THEN** 卡片呈於新位置；無任何文字訊息

##### Example: 六項全域操作的回饋面對照

| 操作 | 成功 | 失敗 |
| ---- | ---- | ---- |
| 刪除變更 | 靜默：卡片消失、詳情頁關閉 | toast：`<變更名稱> · 刪除失敗 ✗ <core 錯誤>` |
| 封存變更 | 靜默：卡片消失、「已封存」計數遞增、詳情頁關閉 | toast：`<變更名稱> · 封存失敗 ✗ <core 錯誤>` |
| 封存討論 | 靜默：卡片消失、詳情頁關閉 | toast：`<討論 slug> · 討論封存失敗 ✗ <core 錯誤>` |
| 拖排卡片 | 靜默：卡片位於新位置 | toast：`<卡片識別> · 排序寫回失敗 ✗ <core 錯誤>` |
| 開啟專案 | 靜默：切換至該專案 | toast：`<選定路徑> · 開啟專案失敗 ✗ <core 錯誤>` |
| 初始化專案 | 靜默：進入該專案 | toast：`<選定目錄> · 初始化失敗 ✗ <core 錯誤>` |

已顯示及新顯示的 toast SHALL 同步套用 app 有效外觀，明示淺色／深色優先於系統；語意 tokens、訊息、逾時、單槽與關閉行為不變。

<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 桌面自動更新

desktop SHALL 於啟動後在背景檢查 GitHub Releases 更新端點,並提供手動「檢查更新」入口。主視窗自失焦變為取得焦點時,若距上次檢查(啟動檢查、手動檢查與前景重檢皆計入)已滿 1 小時,desktop SHALL 再於背景檢查一次;未滿 1 小時 SHALL NOT 檢查。前景重檢的呈現規則 SHALL 與啟動時的背景檢查相同;檢查中、下載中或待重啟時 SHALL NOT 重檢。狀態為待同意的更新提示(已發現新版、尚未同意)或安裝失敗的錯誤時 SHALL NOT 重檢,提示與錯誤訊息 SHALL 留到使用者處置,SHALL NOT 被自動重檢清掉。發現新版時 SHALL 顯示目標版本並徵得使用者同意後才下載套用;SHALL NOT 靜默安裝。更新包簽章驗證失敗時 SHALL 拒絕安裝並顯示錯誤。檢查失敗(離線、端點不可達)SHALL 靜默,不阻擋啟動、不彈出錯誤。節流時間戳 SHALL 只存於執行期記憶體,SHALL NOT 持久化。此行為於 macOS／Windows／Linux 一致,不依平台分支。

呈現:視窗頂端 SHALL NOT 出現更新橫幅。有新版本待同意、下載中、待重啟或安裝失敗時,圖示列底部、設定齒輪上方 SHALL 出現更新鈕——待同意為下載箭頭加琥珀點、下載中為轉圈、待重啟為重啟箭頭加琥珀點、失敗為紅色警示;提示文字 SHALL 依狀態呈現——待同意帶目標版本與「到設定 › 更新」、下載中與待重啟帶目標版本、失敗指向更新頁查看;其他狀態(閒置、檢查中、已是最新、檢查失敗)SHALL NOT 出現該鈕。點更新鈕 SHALL 進入應用程式設定的「更新」頁並捲到「下載與安裝」卡。「更新」頁 SHALL 依序含:頁標題右側「檢查更新」鈕;「版本」卡——「目前版本」列與「最新版本」列(等寬版號加狀態籤:待同意、下載中、待重啟為目標版本加「有新版本」;已是最新為目前版本加「已是最新」;檢查中為「檢查中」;檢查失敗為「無法檢查」;閒置為「尚未檢查」);「下載與安裝」卡(只於待同意、下載中、待重啟、失敗出現)——標題「Speclink <目標版本>」,控制依狀態:待同意為「下載」鈕;下載中為不確定進度列與「取消」(取消即放棄本次同意、回到閒置;其後到達的下載完成 SHALL 被忽略、SHALL NOT 安裝);待重啟為「更新已就緒」與「安裝並重新啟動」;失敗為錯誤訊息與「重試」;「更新日誌」卡(見需求「更新日誌彈窗」);「自動檢查」卡——一列「啟動時在背景檢查新版本」開關,預設開、存 app 本機:關閉時啟動檢查與前景重檢 SHALL NOT 進行,手動檢查照常。應用程式設定中,檢查更新、更新日誌與下載與安裝控制 SHALL 只在更新頁;關於頁品牌卡的更新狀態籤與「最新版本」列為同一份狀態、點擊進更新頁。背景檢查(啟動或前景重檢)發現新版本時 SHALL 另發一次 toast(目標版本、「查看」動作進更新頁),同一目標版本於 app 執行期間 SHALL 只發一次;手動檢查 SHALL NOT 發 toast,結果以「最新版本」列與關於頁版本列的狀態籤呈現。

#### Scenario: 發現新版徵求同意後套用

- **WHEN** 啟動時端點回報的版本高於目前版本,使用者點圖示列更新鈕進更新頁,於「下載與安裝」卡點「下載」,下載完成後點「安裝並重新啟動」
- **THEN** 「最新版本」列為目標版本加「有新版本」;下載期間卡顯示進度與「取消」、圖示鈕為轉圈;下載完成後卡顯示「更新已就緒」、圖示鈕為重啟箭頭;重啟後執行中的版本為新版

#### Scenario: 檢查失敗靜默

- **WHEN** 啟動時無網路或更新端點不可達
- **THEN** desktop 照常啟動,不顯示任何錯誤提示、圖示列無更新鈕;手動檢查時才於「最新版本」列顯示「無法檢查」

#### Scenario: 簽章不符拒裝

- **WHEN** 下載的更新包簽章與內嵌公鑰驗證不符
- **THEN** desktop 拒絕安裝,圖示列更新鈕轉為紅色警示、更新頁「下載與安裝」卡顯示更新失敗錯誤與「重試」,既有安裝不受影響

#### Scenario: 手動檢查已是最新

- **WHEN** 使用者觸發「檢查更新」且目前已是最新版本
- **THEN** 「最新版本」列為目前版本加「已是最新」,不出現「下載與安裝」卡、圖示列無更新鈕、不發 toast

#### Scenario: 回到前景逾一小時即重檢

- **WHEN** app 持續執行且距上次檢查已滿 1 小時,期間發布了新版,使用者把主視窗切回前景
- **THEN** desktop 在背景檢查一次,圖示列出現更新鈕並以 toast 提示目標版本;若已是最新或檢查失敗則不顯示任何提示

##### Example: 開著三小時後切回

- **GIVEN** app 於 09:00 啟動並檢查(當時 0.5.0 為最新),10:30 發布 0.5.1
- **WHEN** 使用者於 12:00 把主視窗切回前景
- **THEN** desktop 背景檢查一次,圖示列更新鈕的提示顯示 0.5.1、toast 顯示 0.5.1,等待使用者到設定頁同意

#### Scenario: 回到前景未滿一小時不重檢

- **WHEN** 使用者把主視窗切回前景,而距上次檢查(含使用者剛按過的手動檢查)不滿 1 小時
- **THEN** desktop 不對端點發出任何請求,介面無變化

##### Example: 節流邊界

| 上次檢查 | 切回前景時刻 | 是否重檢 | 備註 |
|---|---|---|---|
| 09:00 | 09:20 | 否 | 未滿 1 小時 |
| 09:00 | 09:59:59.999 | 否 | 差 1 毫秒 |
| 09:00 | 10:00:00 | 是 | 剛好滿 1 小時 |
| 09:00 啟動、09:40 手動檢查 | 10:20 | 否 | 手動檢查重置時間戳 |
| 從未檢查(啟動檢查尚未進入檢查中) | 任意 | 是 | 無時間戳視為逾時 |
| 09:00,且 09:30 起處於下載中 | 10:30 | 否 | 下載中不重檢 |
| 09:00 檢查到新版、提示待同意 | 10:30 | 否 | 待同意的提示留到使用者處置,不被重檢清掉 |
| 09:00 安裝失敗顯示錯誤 | 10:30 | 否 | 錯誤訊息留到使用者重試 |
| 09:00 下載完成待重啟 | 10:30 | 否 | 待重啟不重檢 |

#### Scenario: 開關系統匣面板不觸發重檢

- **WHEN** macOS 使用者開啟或關閉系統匣面板,主視窗焦點狀態不變
- **THEN** desktop 不進行前景重檢;面板的「詳情／設定／登入」動作聚焦主視窗時才依節流規則判定

#### Scenario: 背景檢查的 toast 只發一次

- **WHEN** 啟動檢查發現 0.5.1,使用者關掉 toast,於設定卡「下載」後又「取消」回到閒置,一小時後前景重檢再次回報 0.5.1
- **THEN** 第一次出現 toast「有新版本 0.5.1」帶「查看」(進更新頁);取消後圖示列更新鈕消失,第二次檢查後更新鈕重新出現但不再出現 toast

#### Scenario: 取消下載回到閒置

- **WHEN** 使用者於「下載與安裝」卡點「取消」
- **THEN** 卡與圖示列更新鈕消失、狀態回到閒置;其後到達的下載完成不安裝、不改變狀態、不出現「安裝並重新啟動」

#### Scenario: 關閉自動檢查

- **WHEN** 使用者於更新頁把「啟動時在背景檢查新版本」關閉後重啟 app,並於一小時後把主視窗切回前景
- **THEN** 啟動與切回前景皆不對端點發出請求;點「檢查更新」仍檢查並更新「最新版本」列;重新開啟開關後下次啟動恢復背景檢查


<!-- @trace
source: desktop-about-page
updated: 2026-10-12T09:12:54+08:00
-->

---
### Requirement: 安裝 CLI 指令到 PATH

desktop SHALL 佈署 CLI 指令並於應用程式設定的「CLI」頁呈現目前狀態（未安裝／已安裝含版本／版本不符）、佈署位置（等寬路徑）、PATH 提示與平台說明；一般頁 SHALL NOT 重複呈現 CLI 狀態。佈署方式依平台：macOS 於 ~/.local/bin 建立指向 app bundle 內 CLI 的 symlink，且 app 啟動偵測到未安裝或版本不符時 SHALL 自動佈署（symlink 冪等，毋須使用者操作）；Windows 由 NSIS 安裝器將安裝目錄寫入使用者 PATH（app 內僅呈現狀態）；Linux AppImage 將 CLI 複製至 ~/.local/bin，且 app 啟動偵測到版本不符時 SHALL 自動重新佈署；Linux 非 AppImage 的執行（開發建置或手動解開的 app）SHALL 只以 ~/.local/bin 的 CLI 呈現狀態、不提供佈署動作、啟動不自動佈署。PATH 設定依平台：macOS 於 ~/.local/bin 不在 PATH 時 SHALL 自動追加一行 PATH 匯出至 ~/.zprofile（帶識別註解、冪等、僅缺席時寫一次）；Linux 佈署目錄不在 PATH 時 SHALL 提示加入方式。

#### Scenario: 安裝後 CLI 與 desktop 同版

- **WHEN** macOS 使用者啟動 app 完成自動佈署後於終端執行 speclink --version
- **THEN** 輸出的版本與 desktop 版本一致，且該指令解析自 ~/.local/bin 的 symlink

#### Scenario: 已安裝呈現狀態

- **WHEN** CLI 已佈署且版本一致時開啟安裝 CLI 介面
- **THEN** 介面顯示已安裝與目前版本，不重複執行佈署

#### Scenario: macOS 啟動自動佈署與自我修復

- **WHEN** macOS 使用者啟動 app 且 ~/.local/bin 無 speclink（或版本與 app 不符）
- **THEN** desktop 自動建立（或重建）symlink，無需使用者操作；佈署失敗不阻斷 app 啟動

#### Scenario: AppImage 版本不符自我修復

- **WHEN** Linux AppImage 更新後啟動，偵測到 ~/.local/bin 的 CLI 版本與 app 不符
- **THEN** desktop 自動以新版重新複製佈署，無需使用者操作

#### Scenario: macOS 自動追加 PATH 且冪等

- **WHEN** macOS 佈署完成而 ~/.local/bin 不在 PATH，且 ~/.zprofile 尚無 speclink 識別註解行
- **THEN** desktop 於 ~/.zprofile 追加帶識別註解的 PATH 匯出一行；重複啟動不重複追加；已在 PATH 或註解行已存在時不寫入

#### Scenario: 佈署目錄不在 PATH 時提示

- **WHEN** Linux 佈署完成但 ~/.local/bin 不在使用者 PATH
- **THEN** 介面提示需將該目錄加入 PATH 及加入方式

#### Scenario: Linux 非 AppImage 執行只呈現狀態

- **WHEN** 在 Linux 上以非 AppImage 方式啟動 app（環境無 APPIMAGE 變數）
- **THEN** 安裝 CLI 介面以 ~/.local/bin/speclink 的偵測結果呈現狀態，不出現佈署動作，啟動不寫入任何檔案


<!-- @trace
source: desktop-about-page
updated: 2026-10-12T09:12:54+08:00
-->

---
### Requirement: 進行中變更可自看板退回提案中

派生階段為進行中的變更,其看板卡片與詳情頁 SHALL 呈現「退回提案中」動作(樣式沿討論卡封存按鈕的動作呈現);非進行中的變更 SHALL NOT 呈現此動作。點擊 SHALL 先經確認,確認後直接呼叫引擎的退回動詞——UI SHALL NOT 預判守門條件(守門僅由引擎裁決一次)。退回成功後 SHALL NOT 手動搬移卡片:重載後開工戳記為空、已勾任務為 0,卡片依階段派生自然回到提案中欄。引擎守門拒絕時 SHALL 開啟守門對話框,列出引擎回傳的證據(已勾任務數與 touched 記錄的檔案清單)與出路說明——已勾任務可於任務分頁取消後重試,touched 需請 agent 判斷;GUI SHALL NOT 提供清理 touched 或批次取消勾選的機械出路。本地與 remote 資料模式下,動作的出現條件、確認流程、成功效果與守門對話框行為 SHALL 一致。跨欄拖曳 SHALL 維持不改變變更階段的既有行為。

#### Scenario: 退回動作僅於進行中變更出現

- **WHEN** 使用者檢視看板上一張提案中的變更卡與一張進行中的變更卡(含各自的詳情頁)
- **THEN** 僅進行中的卡片與其詳情頁呈現「退回提案中」動作,提案中的卡片與詳情頁皆不呈現

#### Scenario: 零痕跡變更確認後退回提案中欄

- **WHEN** 使用者對一張零工作痕跡(無已勾任務、無 touched 記錄)的進行中變更卡點「退回提案中」並於確認對話框確認
- **THEN** 退回成功,卡片自進行中欄消失並出現於提案中欄,詳情頁的開工歸屬顯示清空

#### Scenario: 有工作痕跡時顯示守門對話框

- **WHEN** 使用者對一張已有 3 個已勾任務且 touched 記錄含 2 個檔案的進行中變更點「退回提案中」並確認
- **THEN** 出現守門對話框,列出已勾任務數 3 與該 2 個檔案的清單,並說明取消勾選可重試、touched 需請 agent 判斷;卡片停留在進行中欄,對話框內無任何清理或強制退回按鈕

#### Scenario: remote 模式行為一致

- **WHEN** 使用者於 remote 資料模式對零痕跡的進行中變更執行同一退回操作
- **THEN** 動作出現條件、確認流程、成功後卡片回提案中欄、守門對話框行為皆與本地模式一致


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 桌面刪除變更走 discard 全語意

桌面 app 的本地刪除變更 SHALL 執行引擎 discard 全語意(不帶強制):開工痕跡守門、來源討論解鏈與 touched 紀錄清理——SHALL NOT 直接刪除目錄繞過任一環節。刪除一個由討論轉出的 change 後,來源討論的已轉出清單 SHALL 同步移除該 change 名,清單空時討論狀態 SHALL 回復。對有開工痕跡(meta 含 started_at 或任一任務已勾)的 change,刪除 SHALL 被引擎拒絕且無任何檔案改動。remote session 的刪除 SHALL 以不帶強制的語意呼叫 server 刪除端點,已開工 change 的刪除由 server 拒絕,依既有失敗 toast 語意呈現。

#### Scenario: 刪除轉出 change 連帶解鏈來源討論

- **WHEN** 使用者刪除一個由討論轉出、無開工痕跡的提案中 change(該討論僅轉出此一 change)
- **THEN** change 目錄消失、其 touched 紀錄清除;來源討論的已轉出清單移除該名稱且狀態回復,看板重載後討論卡如實呈現

#### Scenario: 有開工痕跡的本地刪除被拒

- **WHEN** 使用者對 meta 含 started_at 的 change 觸發刪除並確認(UI 停用未及反應的併發情境)
- **THEN** 引擎拒絕,openspec/ 下任何檔案逐位元不變,app 依既有失敗 toast 語意呈現拒絕訊息

#### Scenario: remote 刪除已開工 change 被 server 拒絕

- **WHEN** 使用者於 remote 分頁對已勾任務的 change 觸發刪除並確認
- **THEN** server 以需要強制的拒絕語意回應,scope 內容零改動,app 依既有失敗 toast 語意呈現

<!-- @trace
source: archive-readiness-gating
updated: 2026-07-31
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 指令檔過期提示

桌面 app SHALL 於本地專案成為作用中時執行指令檔過期探測,並於更新動作完成後與 workspace-changed 事件到達時重新探測。探測回報過期、缺失或較新、且該專案未略過當前產物層版號時,SHALL 於專案欄底部、專案設定項上方呈現一張琥珀淡底的提示卡(琥珀圖示、一句標題、一句說明、右端 ›);提示卡 SHALL 於該專案的所有頁面(變更、已封存、規格、手冊、專案設定)常駐,SHALL NOT 佔用主區、SHALL NOT 以 modal 阻斷操作,SHALL NOT 於 remote 專案觸發探測。探測回報現版或無法判定時 SHALL 無任何提示;無法判定 SHALL NOT 記入略過、SHALL NOT 視同現版。其他開著的本地專案上次探測為過期、缺失或較新時,其圖示列方塊角落 SHALL 亮琥珀點,提示附「技能檔需要處理」;背景專案 SHALL NOT 主動探測。

點提示卡 SHALL 開啟確認框:琥珀圖示、標題、說明,一張細框卡三列(這個專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數——較新態第三列為會被換回舊內容的檔案數),按鈕依探測狀態:過期為「稍後」「保留現狀」「更新技能檔」、缺失為「稍後」「保留現狀」「安裝技能檔」(從未安裝的專案不以「更新」稱之)、較新為「保留現狀」「更新 Speclink」。較新時 SHALL 以「app 本體是舊版、專案檔案較新」語意呈現,SHALL NOT 提供任何改寫專案檔案的動作,「更新 Speclink」SHALL 開啟應用程式設定的更新頁。「稍後」SHALL 只關閉確認框、提示卡維持。

「更新技能檔」與「安裝技能檔」SHALL 經引擎既有的指令檔再生入口整套再生受管檔,成功後提示卡與確認框消失;失敗時錯誤 SHALL 呈現於確認框內原位且可重試,SHALL NOT 阻斷專案其他功能。「保留現狀」SHALL 將該專案與當前產物層版號記入 app 本機持久化,SHALL NOT 寫入專案內任何檔案;同版不再提示(缺失、過期與較新共用同一略過記憶),產物層版號變動後 SHALL 重新提示。提示文案 SHALL 遵循 openspec/LANGUAGE.md 詞彙原則,SHALL NOT 出現工程詞。主區各視圖的高度 SHALL NOT 因提示存在而改變。

#### Scenario: 開啟過期專案出現提示並更新

- **WHEN** 以新版 desktop 開啟指令檔為舊版的本地專案,點專案欄底部的提示卡,於確認框點「更新技能檔」
- **THEN** 提示卡先於專案欄底部呈現(標題「技能檔是舊版」、說明含將被改寫的檔案數);確認框三列顯示專案技能檔版本、Speclink 版本與檔案數;更新後受管檔再生為現版、確認框與提示卡消失

#### Scenario: 檔案領先 app 時不提供改寫動作

- **WHEN** 以舊版 desktop(引擎產物層版號舊於專案指令檔標記版號)開啟該本地專案並點提示卡
- **THEN** 提示卡以「技能檔比 Speclink 新」呈現;確認框說明以「app 本體是舊版」語意引導更新 app,按鈕只有「保留現狀」與「更新 Speclink」,無「更新技能檔」「安裝技能檔」;「更新 Speclink」開設定的更新頁;專案內無任何檔案因此變動

#### Scenario: 較新提示保留現狀後同版不再問

- **WHEN** 使用者於較新態的確認框點「保留現狀」,其後關閉並重新開啟同一專案
- **THEN** 提示卡不再出現;app 升版使產物層版號變動後重新探測、依新狀態重新裁決

#### Scenario: 保留現狀同版不再問

- **WHEN** 使用者於過期態的確認框點「稍後」再點提示卡改點「保留現狀」,其後關閉並重新開啟同一專案
- **THEN** 「稍後」後提示卡仍在;「保留現狀」後提示卡消失且重開不再出現,專案內無任何檔案因此變動

#### Scenario: 產物層版號變動後重新提示

- **WHEN** 曾保留現狀的專案,在 desktop 升版且產物層版號變動後再次開啟
- **THEN** 提示卡重新出現

#### Scenario: 標記移除不提示

- **WHEN** 開啟指令檔已整塊移除 SPECLINK 標記的本地專案
- **THEN** 無任何提示卡

#### Scenario: 從未安裝的專案提示安裝

- **WHEN** 開啟 tools 清單宣告 claude 但 CLAUDE.md 不存在(如 clone 後指令檔未進版控)的本地專案,點提示卡並於確認框點「安裝技能檔」
- **THEN** 提示卡以「還沒安裝技能檔」呈現、說明含將被新建的檔案數;安裝後受管檔生成為現版、提示卡消失

#### Scenario: 更新失敗原位呈現錯誤

- **WHEN** 更新動作因受管檔無法寫入而失敗
- **THEN** 錯誤訊息呈現於確認框內容卡下方、動作鍵恢復可按可重試,專案其餘功能不受影響

#### Scenario: remote 分頁不觸發

- **WHEN** 開啟 remote 專案
- **THEN** 不執行過期探測、專案欄無提示卡

#### Scenario: 外部更新自然消提示

- **WHEN** 提示卡顯示期間,使用者於終端執行 speclink update 使受管檔更新為現版
- **THEN** workspace-changed 事件觸發重新探測後提示卡消失,無需重開專案

#### Scenario: 提示卡跨頁常駐且主區不扣高度

- **WHEN** 技能檔提示卡存在時依序切到規格頁、手冊頁、已封存頁、專案設定頁
- **THEN** 提示卡始終在專案欄底部同一位置;主區各視圖的高度與無提示時相同,看板欄底緣仍貼齊主區底緣

#### Scenario: 其他專案的狀況以琥珀點提示

- **WHEN** 開著專案 A(技能檔舊版)與 B(現版),作用中切到 B
- **THEN** A 的圖示列方塊角落亮琥珀點、提示附「技能檔需要處理」;B 無點;專案欄無提示卡;切回 A 後重新探測並顯示提示卡


<!-- @trace
source: desktop-settings-mode
updated: 2026-10-10T23:13:14+08:00
-->

---
### Requirement: 看板卡片的審查標示

change 卡片 SHALL 依審查狀態渲染行內小章（不增加文字列、維持卡片極簡）：`none` 無標示；`inReview` 顯示「審查中」章；`reviewed` 顯示「已審查」章；`reviewedStale` 顯示降級樣式章。章 SHALL 附 tooltip 顯示狀態詞；文案取自 i18n 詞條——tw 使用共用詞彙的正典詞（審查中／已審查／已審查·其後有變動），en 使用對應英文詞條，未設定語言時依應用預設語言。

#### Scenario: 已審查卡片

- **WHEN** 看板載入且某 change 的協定資料為 `reviewStatus: "reviewed"`
- **THEN** 該卡片名稱旁出現審查章，tooltip 為「已審查」（tw）

#### Scenario: 降級章

- **WHEN** 協定資料為 `reviewStatus: "reviewedStale"`
- **THEN** 卡片章以降級樣式呈現，tooltip 為「已審查·其後有變動」（tw）

#### Scenario: 無審查痕跡

- **WHEN** 協定資料為 `reviewStatus: "none"`
- **THEN** 卡片無任何審查相關元素

<!-- @trace
source: code-review-stage
updated: 2026-08-02
code:
  - AGENTS.md
  - CLAUDE.md
  - README.en.md
  - README.md
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/review_verbs.rs
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-node/src/store_bridge.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/read_api.rs
  - crates/speclink-server/tests/review_api.rs
  - packages/ui/src/__tests__/reviewBadge.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedDrawer.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ReviewArchiveDialog.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/reviewStyle.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 已封存側的審查標示

已封存頁的清單與詳情頁 SHALL 依封存時的審查結局標示：帶章者顯示「已審查」；帶化石工單而無章者顯示「曾審查未通過」；皆無者無標示。

#### Scenario: 曾審查未通過

- **WHEN** 已封存 change 的目錄含 review.md 而 metadata 無 reviewed 欄位
- **THEN** 已封存清單項與詳情頁顯示「曾審查未通過」標示


<!-- @trace
source: desktop-readonly-detail-pages
updated: 2026-10-10T19:39:25+08:00
-->

---
### Requirement: 封存入口的未結工單三選項

desktop 的封存入口（卡片封存鈕、詳情頁封存動作、拖曳封存）SHALL 於目標 change 為 `inReview` 時彈出三選項對話框：前往完成蓋章（導引執行審查收尾）、放棄審查（等同 review discard 後封存）、照樣帶走（等同 `--carry-review` 封存，並說明將永久顯示「曾審查未通過」）。非 `inReview` 的封存行為 SHALL 維持現行不變。

#### Scenario: 封存審查中的 change

- **WHEN** 對 `reviewStatus: "inReview"` 的 change 觸發封存
- **THEN** 出現三選項對話框，未選擇前不執行封存

#### Scenario: 選擇照樣帶走

- **WHEN** 於對話框選擇「照樣帶走」
- **THEN** change 封存成功且已封存側顯示「曾審查未通過」


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 品質站蓋章配色與主色分離

品質站狀態標示的配色 SHALL 與主題主色分離:「已審查」蓋章 SHALL 以紫色系呈現,SHALL NOT 引用主題主色 token;「審查中」「已審查·其後有變動」「曾審查未通過」三態維持藍/琥珀/紅系不變。紫色系 SHALL 為品質站蓋章專屬——品質站標示以外的介面元素 SHALL NOT 使用紫色系。此約束 SHALL 套用於呈現審查標示的全部表面——「已審查」蓋章出現於看板卡片、變更詳情頁、已封存清單、已封存詳情頁四處;封存對話框共用同一份配色對照,但僅取用「曾審查未通過」,不呈現已審查蓋章。各表面 SHALL 引用同一份配色對照,SHALL NOT 各自定義。

#### Scenario: 已審查蓋章與主色可區辨

- **WHEN** 使用者開啟 reviewStatus 為 reviewed 的變更卡片與其詳情頁
- **THEN** 「已審查」蓋章以紫色系呈現,與同視野的主色元素(進度條、分頁、連結籤)可區辨;深淺主題下皆然

#### Scenario: 其餘三態配色不變

- **WHEN** 使用者分別檢視 inReview、reviewedStale、reviewedNotPassed 的審查標示
- **THEN** 三態仍以藍、琥珀、紅系呈現,詞條與圖示與本變更前一致

#### Scenario: 紫色系為蓋章專屬

- **WHEN** 使用者檢視看板、詳情頁與已封存各表面
- **THEN** 紫色系僅出現於品質站蓋章,不出現在其他介面元素


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 介面狀態語意色分層

桌面 app 的狀態呈現 SHALL 依語意上色:進行中(分頁還原、遷移進行、更新下載、看板卡片 worktree 標示)為藍系;成功結果(遷移完成)為綠系;警示(登入過期 stale、needs-reauth、政策衝突、待重新反映)為琥珀系;錯誤(分頁 error、更新檢查失敗、設定檔解析錯誤、遠端工作區 access-denied/not-found/unknown)與危險動作為紅系;待手動(看板「待手動」章、任務文件的手動任務列淡底與徽章、手動任務提示框)為暖黃系——暖黃與警示琥珀是兩個色相,待手動 SHALL NOT 以琥珀或藍呈現。主題主色 SHALL NOT 用於表達狀態;錯誤 SHALL NOT 以琥珀呈現;靜態小標與裝飾面 SHALL 為中性。

生命週期與討論 SHALL 各有一個色相,由單一來源供給並為看板與系統匣面板共用:提案中為主色(teal)、進行中為藍系(與進行中語意同色)、已就緒為綠系(與成功語意同色)、討論為桃紫系(fuchsia);看板欄頂條、欄標頭圖示、欄計數徽章、卡片波次章與系統匣面板的生命週期分區、討論分區的圖示與計數徽章 SHALL 取所在階段或分區的色相;「已轉出」分區、收合列及其計數 SHALL 為中性。琥珀、紅與紫色系 SHALL NOT 用於任何生命週期或討論的欄位與分區。豁免項 SHALL 維持主色:複製成功勾號(互動即時回饋)、可點連結籤與導航元素。

色值 SHALL 只存在於共用 theme 樣式表(基底 token、狀態語意 token、品質站蓋章 token、改進標示 token、生命週期 token),淺色與深色各一套;集中常數檔 SHALL 只保存「狀態→token class」的對照;元件原始碼 SHALL 只以 token class 取色。原生色階字面(含語意色階與中性色階)SHALL NOT 出現在共用元件庫、桌面 app 與 server web 的任何元件或常數檔原始碼;主題守門測試 SHALL 於違規時失敗並指名檔案與 class,SHALL NOT 設白名單。

#### Scenario: 錯誤態以紅呈現

- **WHEN** 專案分頁進入 error、更新檢查失敗或設定檔解析錯誤
- **THEN** 對應訊息與圖示以紅系呈現,非琥珀

#### Scenario: 進行中以藍呈現

- **WHEN** 分頁還原中或遷移進行中
- **THEN** spinner 與狀態面以藍系呈現

#### Scenario: worktree 標示以藍呈現

- **WHEN** 看板卡片的 change 有活躍 worktree 映射
- **THEN** 該卡片的 worktree 圖示以藍系(進行中語意)呈現,非主色;變更詳情頁的分支與路徑維持中性

#### Scenario: 成功以綠呈現

- **WHEN** 工作區遷移完成
- **THEN** 成功卡與勾號圓標以綠系呈現

#### Scenario: 待手動以暖黃呈現

- **WHEN** 看板有一張帶「待手動」章的變更卡,開啟其詳情頁的任務文件,文件含一個未勾的 `[M]` 任務
- **THEN** 卡片的「待手動」章、任務列的手動徽章與淡底、閱讀卡的手動任務提示框皆取同一個暖黃語意 token,非琥珀、非藍

#### Scenario: 生命週期四欄各一色相

- **WHEN** 檢視看板的討論、提案中、進行中、已就緒四欄與系統匣面板的對應分區
- **THEN** 欄頂條、標頭圖示與計數徽章分別為桃紫、主色、藍、綠;系統匣面板的討論分區與三個生命週期分區同色;「已轉出」分區與收合列為中性

##### Example: 生命週期與討論的色相對照

| 欄位／分區 | 色相 | 與之同色的既有語意 |
| ---------- | ---- | ------------------ |
| 討論       | 桃紫 | 無(專屬)          |
| 提案中     | 主色 | 連結、主要操作      |
| 進行中     | 藍   | 進行中狀態          |
| 已就緒     | 綠   | 成功結果            |
| 已轉出     | 中性 | 靜態 meta          |

#### Scenario: 卡片波次章隨所在欄色相

- **WHEN** 檢視提案中欄與進行中欄各一張帶波次章的變更卡
- **THEN** 提案中卡的波次章為主色、進行中卡的波次章為藍,與各自欄的計數徽章同色

#### Scenario: 換一個色值全面生效

- **WHEN** 開發者只改共用 theme 樣式表中已就緒的生命週期 token 值
- **THEN** 看板已就緒欄、卡片、系統匣面板已就緒分區與 server web 任何取用該 token 的元件同步變色,不需改動任何 TS 或 TSX 原始碼

#### Scenario: 深色主題取深色 token

- **WHEN** 系統偏好為深色時檢視看板與系統匣面板
- **THEN** 生命週期、狀態與蓋章色取自深色 token 值,元件原始碼不含任何 dark: 色階成對寫法

#### Scenario: 原生色階守門無白名單

- **WHEN** 共用元件庫、桌面 app 或 server web 任一元件或常數檔原始碼出現原生語意色階或中性色階字面
- **THEN** 主題守門測試失敗並列出違規檔名與 class,任何檔案皆不豁免


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
### Requirement: 看板卡片的驗證標示

change 卡片 SHALL 依驗證狀態渲染第二顆行內小章，與審查章並排且順序固定（審查章在前、驗證章在後）：`none` 無標示；`inVerify` 顯示「驗證中」章；`verified` 顯示「已驗證」章；`verifiedStale` 顯示降級樣式章。tooltip 文案取自 i18n 詞條——tw 使用正典詞（驗證中／已驗證／已驗證·其後有變動），en 使用對應詞條。驗證章各狀態的配色 SHALL 與審查章對應狀態同值（色承載狀態、圖示形狀承載站別），SHALL NOT 另建配色；驗證章圖示 SHALL 採盾牌形系，與審查章的徽章形系可區辨。

#### Scenario: 兩章並排

- **WHEN** 某 change 的協定資料為 `reviewStatus: "reviewed"` 且 `verifyStatus: "verified"`
- **THEN** 卡片依序顯示審查章與驗證章，各自 tooltip 正確

#### Scenario: 兩站同狀態同色、異形可辨

- **WHEN** 某 change 同時為 `reviewStatus: "reviewed"` 與 `verifyStatus: "verified"`
- **THEN** 兩章配色同值（皆為品質站蓋章色），審查章為徽章形、驗證章為盾牌形，站別可一眼區辨

#### Scenario: 僅驗證章

- **WHEN** 協定資料為 `reviewStatus: "none"` 且 `verifyStatus: "verifiedStale"`
- **THEN** 卡片僅顯示降級樣式的驗證章，tooltip 為「已驗證·其後有變動」（tw）

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 已封存側的驗證標示

已封存頁的清單與詳情頁 SHALL 依封存時的驗證結局標示：帶章者「已驗證」；帶化石驗證工單而無章者「曾驗證未通過」；皆無者無標示。與審查結局標示可並存。

#### Scenario: 曾驗證未通過

- **WHEN** 已封存 change 的目錄含 verify.md 而 metadata 無 verified 欄位
- **THEN** 已封存清單項與詳情頁顯示「曾驗證未通過」標示


<!-- @trace
source: desktop-readonly-detail-pages
updated: 2026-10-10T19:39:25+08:00
-->

---
### Requirement: 封存入口三選項擴及驗證工單

desktop 封存入口 SHALL 於目標 change 存在未結驗證工單時彈三選項對話框（前往完成驗證蓋章／放棄驗證／照樣帶走並警示永久標示）；review 與 verify 工單並存時 SHALL 讓使用者對兩種工單分別處置後才執行封存。

#### Scenario: 封存驗證中的 change

- **WHEN** 對 `verifyStatus: "inVerify"` 的 change 觸發封存
- **THEN** 出現驗證三選項對話框，未選擇前不執行封存

#### Scenario: 雙工單並存的封存

- **WHEN** 對同時為 `inReview` 與 `inVerify` 的 change 觸發封存
- **THEN** 對話流程涵蓋兩種工單的處置選擇，全部選定後才封存

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 主題化提示統一延遲

共用 UI 元件庫的主題化提示 SHALL 採單一共用延遲預設：指標停留達 300ms 顯示提示；個別介面 SHALL NOT 另行覆寫延遲值。此預設 SHALL 同源套用於 desktop 與 server-web console（兩者共用同一 UI 元件庫）。於同一提示群組內的多個觸發點間連續移動時，後續提示 SHALL 立即顯示（沿用元件庫既有 skipDelay 行為；跨群組移動時延遲重新計算）。系統匣面板的原生 title 提示不屬主題化提示，SHALL 維持原生行為、不受本延遲約束。

#### Scenario: 卡片章提示於統一延遲後顯示

- **WHEN** 使用者將指標停留於看板卡片的審查章達 300ms
- **THEN** 主題化提示顯示狀態詞（tw 為正典詞「已審查」等）

#### Scenario: 詳情頁與卡片同一延遲

- **WHEN** 使用者分別將指標停留於變更詳情頁的章籤與看板卡片的章
- **THEN** 兩處提示皆於停留達 300ms 時顯示，無任一處立即顯示或採用其他延遲值


<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 指令檔過期提示捲動釘選

指令檔過期提示卡 SHALL 位於專案欄,SHALL NOT 屬於主區任何捲動容器;主區於任何頁面捲動時提示卡 SHALL 維持可見且位置不變,SHALL NOT 被捲過的內容覆蓋或推動。

#### Scenario: 捲動時提示保持可見

- **WHEN** 專案設定頁顯示提示卡且使用者向下捲動頁面
- **THEN** 提示卡仍在專案欄底部同一位置、完整可見,頁面內容只在主區內捲動


<!-- @trace
source: desktop-notice-relocation
updated: 2026-10-10T00:25:32+08:00
-->

---
### Requirement: 截斷省略號的統一字形

介面上任何以省略號收尾的截斷，SHALL 於全部表面呈現同一字形——拉丁半形、貼基線。省略號字元（U+2026）的字形 SHALL 由共用 UI 元件庫的 theme 以限定該碼位的字型宣告統一供應，desktop 與 server-web 同源生效。此統一 SHALL NOT 改變任何一段文字本身的字型歸屬：等寬把手（變更名稱、討論 slug、分支名、路徑）SHALL 維持等寬字型，一般文字 SHALL 維持既有中文字型，markdown 程式碼區塊 SHALL 維持等寬字型。當執行環境找不到任何可供應該字形的字型時，該宣告 SHALL 失效並退回既有字型呈現，SHALL NOT 造成缺字或破圖。

#### Scenario: 等寬把手與中文文字的截斷收尾同形

- **WHEN** 同一張看板卡片上，等寬標題與中文描述列同時因過長而截斷
- **THEN** 兩處省略號為同一字形（拉丁半形、貼基線），標題仍以等寬字型呈現、描述列仍以既有中文字型呈現

<!-- @trace
source: desktop-ui-stamp-and-overflow-polish
updated: 2026-08-07
-->

---
### Requirement: 看板討論卡片的改進標示

kind 為 improve 的討論卡片 SHALL 於卡片上渲染行內小章(不增加文字列、維持卡片極簡),小章 SHALL 附 tooltip 顯示狀態詞——文案取自 i18n 詞條,tw 使用共用詞彙的正典詞「改進討論」、en 使用對應英文詞條,未設定語言時依應用預設語言。一般討論(無 kind)SHALL NOT 渲染任何新增元素。標示 SHALL 隨 kind 恆定:已轉出變更與已封存側的改進討論 SHALL 維持同一標示,SHALL NOT 隨生命週期狀態變化或消失。

#### Scenario: 改進討論卡片

- **WHEN** 看板顯示一筆 kind 為 improve 的討論(語言 tw)
- **THEN** 該卡片出現改進小章,tooltip 為「改進討論」

#### Scenario: 一般討論卡片無標示

- **WHEN** 看板顯示一筆無 kind 欄位的討論
- **THEN** 卡片不出現改進小章,與本變更引入前的渲染一致

#### Scenario: 已封存的改進討論維持標示

- **WHEN** 已封存頁顯示一筆 kind 為 improve 的已封存討論
- **THEN** 該筆項目仍顯示改進小章,tooltip 詞彙與看板側一致

<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->

---
### Requirement: 觸及檔案系統的 command 不佔用主執行緒

桌面 app 中凡會觸及檔案系統或 spawn 子進程（git 等）的 Tauri command SHALL 以 async 形式將工作委派至背景執行緒（spawn_blocking）執行；主執行緒 SHALL NOT 執行任何檔案 IO 或子進程等待。純記憶體讀取與純視窗操作的 command（如行程環境讀取、連線健康狀態、系統匣面板開閉、結束 app）不受此限。command 的對外契約（名稱、參數、回傳資料形狀）SHALL 與同步時代完全一致。

#### Scenario: 引擎讀取耗時期間 UI 保持可回應

- **WHEN** 使用者切換 workspace（主視窗分頁或 tray 面板皆同），且引擎讀取因 repo 忙碌（如 agent 同時在同一 repo 操作）而耗時數秒
- **THEN** 主視窗與系統匣仍可互動——視窗事件與 tray 圖示點擊即時回應，資料以載入中狀態呈現，不發生整窗凍結

#### Scenario: async 化不改變 command 對外契約

- **WHEN** 前端以既有名稱與參數 invoke 任一改為 async 的 command
- **THEN** 成功路徑的回傳資料形狀與同步時代完全一致，前端無需任何配合改動；回傳資料的 command 於背景執行本身失敗（如關閉期取消）時 SHALL 以 invoke 錯誤回報，不得讓呼叫懸置不決（fire-and-forget 的監看類 command 維持既有降級語意：失敗僅記錄、呼叫正常結束）

<!-- @trace
source: desktop-async-commands
updated: 2026-08-11
-->

---
### Requirement: 分頁切換中即時回饋

點擊圖示列上的非作用中本地專案方塊後、專案探測(probe)完成前,目標方塊 SHALL 立即以切換中 spinner 取代首字母,且原專案畫面 SHALL 照常顯示並可互動;探測成功翻頁或失敗轉錯誤態時 spinner SHALL 消失。切換順序(探測成功才切換作用中專案)與既有錯誤呈現 SHALL 維持不變。spinner SHALL 帶有切換中語意的 aria-label。

#### Scenario: 點擊分頁立即出現 spinner

- **WHEN** 點擊非作用中的本地專案方塊且專案探測尚未完成
- **THEN** 目標方塊立即顯示 spinner,畫面停留在原專案且可互動

#### Scenario: 探測成功後切頁

- **WHEN** 專案探測成功完成
- **THEN** spinner 消失,作用中方塊切換為目標專案,左側標題列與麵包屑的專案名同步更新

#### Scenario: 探測失敗回饋照舊

- **WHEN** 專案探測失敗(如目錄已不存在)
- **THEN** spinner 消失,方塊以錯誤態(角落紅點)呈現,作用中專案不變


<!-- @trace
source: desktop-shell-redesign
updated: 2026-10-09T12:33:58+08:00
-->

---
### Requirement: 看板首訪以 skeleton 佔位

活躍 workspace 的快照尚未完成首次載入時，看板 SHALL 以 skeleton 佔位卡呈現卡片區，欄位標題 SHALL 照常顯示真實文字；快照已載入（含上次造訪的舊快取）時 SHALL 直接顯示既有資料，整批重載完成後靜默更新，SHALL NOT 於重載期間顯示 skeleton。skeleton 區塊 SHALL 標記 aria-busy，其動畫 SHALL 於 prefers-reduced-motion 下停用。

#### Scenario: 首訪 workspace 顯示骨架卡

- **WHEN** 切換至首次造訪的 workspace 且整批載入尚未完成
- **THEN** 看板欄名照常顯示，卡片區為 skeleton 佔位卡，與空 workspace 的空態呈現可區分

#### Scenario: 載入完成換為真資料

- **WHEN** 首訪 workspace 的整批載入完成
- **THEN** skeleton 消失，顯示實際卡片；workspace 確實無內容時顯示既有空態呈現

#### Scenario: 已訪 workspace 不閃 skeleton

- **WHEN** 切換至先前造訪過的 workspace
- **THEN** 立即顯示上次快照的資料，全程無 skeleton，重載完成後畫面靜默更新

<!-- @trace
source: desktop-loading-skeleton-ux
updated: 2026-08-12
-->

---
### Requirement: 首訪載入失敗終態呈現

活躍 workspace 尚無已載入快照（首訪）且整批載入以失敗收場時，看板卡片區 SHALL 顯示載入失敗提示文案，SHALL NOT 顯示與真空 workspace 相同的空態文案，SHALL NOT 停留在 skeleton。提示 SHALL 於下一次成功載入後消失並恢復正常呈現。失敗記錄 SHALL 隨 workspace 快照存續——切走再切回、重試再度失敗後仍顯示失敗提示而非空態；切回觸發的重試在途期間 SHALL 呈現載入骨架（載入中優先於失敗提示，且不顯示空態文案）。已有舊快取的 workspace 重載失敗 SHALL 維持既有行為（沿用最後成功快照靜默呈現，不顯示失敗提示）。

#### Scenario: 首訪載入失敗顯示失敗提示

- **WHEN** 切換至首次造訪的 workspace 且整批載入失敗（如離線或目錄不可讀）
- **THEN** 骨架收掉，卡片區顯示載入失敗提示文案，與空 workspace 的空態呈現可區分

#### Scenario: 失敗後成功載入恢復正常

- **WHEN** 顯示失敗提示的 workspace 隨後一次整批載入成功
- **THEN** 失敗提示消失，顯示實際資料（或真空態文案）

#### Scenario: 失敗記錄隨快照存續

- **WHEN** 自顯示失敗提示的 workspace 切至他處再切回，期間無成功載入
- **THEN** 切回觸發的重試在途期間顯示骨架（載入中優先），重試仍失敗後回到失敗提示；全程不顯示空態文案

#### Scenario: 舊快取重載失敗不受影響

- **WHEN** 已載入過的 workspace 於重載時失敗
- **THEN** 照常顯示最後一次成功快照，無失敗提示、無骨架

<!-- @trace
source: desktop-refreshing-inflight-set
updated: 2026-08-12
-->

---
### Requirement: 任務列的手動任務徽章

任務文件（變更詳情頁與已封存檢視）的 `[M]` 任務列 SHALL 以兩個訊號標出手動任務：未勾選時整列以手動色淡底呈現；列尾（D 籤之後、展開鈕之前）SHALL 有一枚「手動」小徽章——圖示＋文字的 chip（圖示取 lucide 的手部同義圖示），文案取自 i18n 詞條（tw 用「手動」二字——共用詞彙正典詞「手動任務」的徽章縮寫，不寫「手動測試」以免使用者誤解標記僅限測試；en 用對應詞條），文字、外框與淡底取手動色。徽章 SHALL NOT 插在勾選框、序號與標題之間，也 SHALL NOT 獨佔一行：手動列的勾選框、序號與標題 SHALL 與無標記列同一行、同一起始欄，長標題換行時各行左緣不位移。手動色 SHALL 為專屬的暖黃語意色（見「介面狀態語意色分層」），與看板「待手動」章、任務文件的手動任務提示框同色。任務勾選後 SHALL 維持文字劃線的既有樣式並退掉整列淡底，徽章 SHALL 保留且不劃線。剝離後的 `[P]` 舊標記 SHALL NOT 產生任何徽章或淡底。無 `[M]` 任務的清單呈現 SHALL 與本需求引入前一致。

#### Scenario: 手動任務列的徽章呈現

- **WHEN** 任務文件載入含「- [ ] [M] 3.2 手動驗證匯入結果」與「- [x] 3.1 跑全套測試」的 tasks.md
- **THEN** 3.2 列整列為手動色淡底、列尾有「手動」徽章，勾選框、序號籤「3.2」與標題同一行且序號起始欄與 3.1 同位；3.1 列無徽章、無淡底

#### Scenario: 長描述換行不動徽章

- **WHEN** 該 `[M]` 任務的描述長到換成多行
- **THEN** 徽章仍在列尾、不佔描述的行，描述各行的左緣與無標記列一致

#### Scenario: 勾完徽章保留

- **WHEN** 使用者勾選該 `[M]` 任務
- **THEN** 任務文字轉劃線樣式、整列淡底退掉，徽章保留於列尾且不劃線

#### Scenario: 舊 [P] 行無徽章

- **WHEN** 任務文件載入含「- [x] [P] 舊任務」的封存前舊檔
- **THEN** 該列顯示剝離後文字「舊任務」，無徽章、無淡底


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
### Requirement: 看板卡片的待手動標示

change 卡片 SHALL 於「寫碼任務全完成且尚有未勾 `[M]` 任務」(codeTotal 大於 0、codeRemaining 為 0 且 remaining 大於 0)時渲染一枚「待手動」行內小章——沿看板卡片的審查標示家族樣式:不增加文字列、維持卡片極簡,附 tooltip 載明剩餘項數(tw「待手動·剩 N 項」),文案取自 i18n 詞條。codeTotal 為 0 的變更(尚無任何寫碼任務)SHALL NOT 顯示章——「寫碼全完成」的空真值不視為待手動。寫碼任務未全完成、或全任務完成的卡片 SHALL 與本需求引入前逐位元一致。章與看板欄位派生 SHALL 正交(不影響卡片所在欄位)。判定 SHALL 收斂於 packages/ui 的階段派生模組單一入口,卡片元件只讀結果。資料源缺寫碼進度欄位時(remote 變更摘要)章 SHALL 缺席——與審查標示的 local-only 行為同構。

#### Scenario: 待手動卡片浮現章

- **WHEN** 看板載入一個 codeTotal=9、codeComplete=9、total=10、complete=9 的變更
- **THEN** 該卡片出現「待手動」行內小章,tooltip 為「待手動·剩 1 項」(tw);卡片所在欄位不變

#### Scenario: 寫碼未完成無章

- **WHEN** 看板載入一個 codeRemaining 大於 0 的變更
- **THEN** 該卡片無待手動章,呈現與本需求引入前一致

#### Scenario: 全完成無章

- **WHEN** 看板載入一個 complete=total 的變更
- **THEN** 該卡片無待手動章,呈現與本需求引入前一致

#### Scenario: remote 模式無章

- **WHEN** remote 看板載入變更清單(摘要 payload 不含寫碼進度欄位)
- **THEN** 卡片一律無待手動章,其餘呈現不變

##### Example: 浮現判定

| codeTotal | codeRemaining | remaining | 待手動章 |
| --------- | ------------- | --------- | -------- |
| 9 | 0 | 1 | 顯示(剩 1 項) |
| 7 | 0 | 3 | 顯示(剩 3 項) |
| 8 | 2 | 3 | 無 |
| 10 | 0 | 0 | 無 |
| 0 | 0 | 2 | 無(尚無寫碼任務,空真值不浮現) |

<!-- @trace
source: manual-marker-scope-beyond-tests
updated: 2026-08-14
-->

---
### Requirement: 更新日誌彈窗

desktop SHALL 打包 release-notes.json 並於啟動取得 app 版號後判定是否彈出更新日誌：app 版號等於 JSON 頂端版號、且本機記住的已看過版號不等於 app 版號時，SHALL 以 modal 對話框彈出「X.Y.Z 更新內容」，列出比已看過版號新的全部條目（已看過版號不在清單內時只列頂端一筆），每筆以「版號（日期）」為小標、其下依「新功能／修正／改善」分組列點；使用者關閉（主鈕「知道了」）時 SHALL 記下 app 版號為已看過。本機無已看過記錄（首次安裝）時 SHALL NOT 彈出，SHALL 直接記下 app 版號。dev 建置 SHALL NOT 自動彈出、SHALL NOT 寫入記錄。app 版號不等於 JSON 頂端版號時 SHALL NOT 彈出。更新頁 SHALL 有「更新日誌」卡：最新版本為第一列並預設展開其分組列點，較舊版本每版一列（版號、日期、首條摘要）附「檢視」，底列為「全部更新日誌」；兩者皆開啟同一對話框的瀏覽模式（標題「更新日誌」、列出全部條目、主鈕「關閉」）——「全部更新日誌」自頂端呈現，某版列的「檢視」SHALL 於開啟後把該版小標捲到捲動區頂端；瀏覽模式關閉 SHALL NOT 改變已看過記錄。應用程式設定中更新日誌的瀏覽入口 SHALL 只在更新頁（一般頁與關於頁不設）；macOS 原生選單「說明 › 更新日誌」照常開啟瀏覽模式。已看過記錄 SHALL 存於 app 本機持久化，SHALL NOT 進入任何專案檔案。

#### Scenario: 更新後首次啟動彈出且看過即止

- **WHEN** 使用者由 0.2.0 更新到 0.3.0 後首次啟動，已看過記錄為 0.2.0
- **THEN** 彈出「0.3.0 更新內容」對話框只含 0.3.0 條目；按「知道了」後重新啟動不再彈出

#### Scenario: 跳版更新一次看齊

- **WHEN** 已看過記錄為 0.2.0，使用者直接更新到 0.3.1 且 JSON 依序有 0.3.1、0.3.0、0.2.0
- **THEN** 對話框標題為「0.3.1 更新內容」，依序列出 0.3.1 與 0.3.0 兩筆，不含 0.2.0

##### Example: 彈出判定

| app 版號 | JSON 頂端 | 已看過 | dev | 結果 |
| -------- | --------- | ------ | --- | ---- |
| 0.3.0 | 0.3.0 | 0.2.0 | 否 | 彈出，含 0.3.0 |
| 0.3.1 | 0.3.1 | 0.2.0 | 否 | 彈出，含 0.3.1、0.3.0 |
| 0.3.1 | 0.3.1 | 0.1.9（不在清單） | 否 | 彈出，只含 0.3.1 |
| 0.3.0 | 0.3.0 | 無記錄 | 否 | 不彈，記下 0.3.0 |
| 0.3.0 | 0.3.0 | 0.3.0 | 否 | 不彈 |
| 0.3.0 | 0.2.0 | 0.2.0 | 否 | 不彈 |
| 0.3.0 | 0.3.0 | 0.2.0 | 是 | 不彈、不寫記錄 |

#### Scenario: 首次安裝不彈

- **WHEN** 全新安裝 0.3.0 後首次啟動
- **THEN** 不彈出對話框，已看過記錄變為 0.3.0；之後更新到 0.3.1 才彈出

#### Scenario: 更新頁瀏覽全部版本

- **WHEN** 使用者在更新頁「更新日誌」卡按「全部更新日誌」
- **THEN** 對話框以「更新日誌」為標題列出 JSON 全部條目；按「關閉」後已看過記錄不變，下次啟動的自動彈出判定不受影響

#### Scenario: 更新頁檢視單一版本

- **WHEN** 使用者在更新頁「更新日誌」卡的 0.6.0 列按「檢視」
- **THEN** 對話框以「更新日誌」為標題列出 JSON 全部條目，開啟後 0.6.0 的小標位於捲動區頂端；按「關閉」後已看過記錄不變

#### Scenario: 已看過記錄壞值視為無記錄

- **WHEN** 本機持久化的已看過記錄不是字串
- **THEN** 視為無記錄：不彈出、記下 app 版號


<!-- @trace
source: desktop-about-page
updated: 2026-10-12T09:12:54+08:00
-->

---
### Requirement: 看板卡片的波次與阻擋標示

change 卡片 SHALL 於清單項帶 `wave` 欄位時，在進度列（進度條與任務數所在的那一列）最左渲染一枚波次行內小章：圓圈內為波次數字（同波同號即可並行）；`blockedBy` 為空時 tooltip 為「第 N 波」（tw），非空時 tooltip 為「第 N 波 · 等待：a、b」（tw；前置名稱以「、」相連，en 以「, 」）。`blockedBy` 非空時卡片 SHALL 整卡降透明度並帶 `data-blocked="true"` 屬性（與系統匣面板被擋列同一語意：淡＝現在不能做），SHALL NOT 另渲染「等 N 項」文字章。排程資訊 SHALL NOT 在標題列（名稱所在列）新增任何章——名稱的空間不被排程資訊佔用。判定 SHALL 收斂於 packages/ui 的階段派生模組單一入口（planWave／planBlockedBy），卡片元件只讀結果。清單項缺 `wave`（remote 清單未取得 plan：舊 server、請求失敗或成環；或 local plan 成環）時波次章 SHALL 缺席、卡片 SHALL NOT 變淡，其餘呈現 SHALL 與本需求引入前逐位元一致。波次章 SHALL NOT 增加文字列、SHALL NOT 影響卡片所在欄位。

#### Scenario: 第一波無阻擋

- **WHEN** 看板載入一個 wave=1、blockedBy=[] 的變更
- **THEN** 該卡片進度列最左顯示圓圈「1」波次章，tooltip 為「第 1 波」，卡片不變淡且無 data-blocked

#### Scenario: 第二波被兩項擋住

- **WHEN** 看板載入一個 wave=2、blockedBy=["add-a","add-b"] 的變更
- **THEN** 該卡片進度列最左顯示圓圈「2」，tooltip 為「第 2 波 · 等待：add-a、add-b」，整卡降透明度並帶 data-blocked="true"，標題列沒有任何排程章

#### Scenario: 缺欄位無章

- **WHEN** remote 看板載入變更清單，但 server 沒有 plan 端點（清單項不含 wave）
- **THEN** 卡片無波次章、不變淡，其餘呈現不變


<!-- @trace
source: add-change-plan-remote
updated: 2026-09-17T15:34:04+08:00
-->

---
### Requirement: 拖排時不合法落點灰化

看板拖動變更卡 X 期間，同欄中「X 的 dependsOn 成員」及其上方所有卡片位置、「dependsOn 含 X 的卡」及其下方所有卡片位置 SHALL 標為不合法落點：該些卡片 SHALL 降透明度並帶 `data-drop-invalid="true"` 屬性；放開於不合法落點 SHALL NOT 觸發任何寫回，卡片回到原位。不合法落點的判定 SHALL 只用宣告依賴（dependsOn），SHALL NOT 用 delta 重疊。判定 SHALL 為 packages/ui 的純函式（invalidDropTargets），拖曳接線只讀結果；接線餵入的欄 SHALL 為同欄全序（不經搜尋過濾），被過濾隱藏的前置仍決定其上方位置不合法。planError 非 null 或欄位缺席時 SHALL 無不合法落點。

#### Scenario: 拖動依賴者時前置灰化

- **WHEN** 同欄依序為 a、b、c，c 的 dependsOn 含 a，使用者開始拖動 c
- **THEN** a 帶 data-drop-invalid 且降透明度，b 正常；放開於 a 上不觸發寫回，c 回到原位

#### Scenario: 拖動前置時依賴者灰化

- **WHEN** 同欄依序為 a、b、c，c 的 dependsOn 含 a，使用者開始拖動 a
- **THEN** c 帶 data-drop-invalid，b 正常；放開於 b 上觸發寫回（a 落在 b 之後、c 之前）

#### Scenario: 重疊夥伴不灰化

- **WHEN** a 與 b 只因 delta 重疊而依序、無宣告依賴，使用者拖動 b
- **THEN** a 不帶 data-drop-invalid，放開於 a 上觸發寫回

#### Scenario: 搜尋過濾隱藏前置

- **WHEN** 同欄全序為 b、a、c，c 的 dependsOn 含 a，搜尋過濾只留 b 與 c，使用者開始拖動 c
- **THEN** b 帶 data-drop-invalid（a 隱藏但仍在 b 之後），放開於 b 上不觸發寫回

<!-- @trace
source: add-change-plan-desktop
updated: 2026-09-16T14:57:15+08:00
-->

---
### Requirement: 依賴成環時看板提示

清單 payload 的 `planError` 非 null 時，看板 SHALL 於欄位上方顯示一行提示：tw「依賴成環：」加訊息原文（如 a -> b -> a），en「Dependency cycle: 」加原文；卡片 SHALL 無波次章與阻擋章；欄內順序為 plan 退回序（local 為基底序；remote 為 board resource overlay 序，見 remote-board-order）；拖排 SHALL 照常可用。planError 為 null 時 SHALL NOT 渲染此提示列。

#### Scenario: 成環提示

- **WHEN** 看板載入 planError 為 `dependency cycle: a -> b -> a` 的清單
- **THEN** 欄位上方出現「依賴成環：dependency cycle: a -> b -> a」一行，所有卡片無波次章，拖排可用

#### Scenario: 無成環不提示

- **WHEN** 看板載入 planError 為 null 的清單
- **THEN** 不出現提示列，呈現與本需求引入前一致


<!-- @trace
source: add-change-plan-remote
updated: 2026-09-17T15:34:04+08:00
-->

---
### Requirement: 共用元件唯一來源

桌面 app 與 server web 的共用介面元件(含確認框、複製鈕、空狀態、區段標題、品牌標記與字標)SHALL 只在共用元件庫有一份實作;桌面 app、系統匣面板與 server web SHALL 自共用元件庫引用,SHALL NOT 各自定義與共用元件庫匯出同名的元件。兩鍵式確認框(取消＋確認)SHALL 以共用確認框元件呈現,其標題、說明、按鈕文案、危險變體與 disabled 行為由呼叫端以屬性提供;三選項以上的對話框不受此限。品牌標記與字標的影像資產 SHALL 只存放於共用元件庫一處;品牌標記 SHALL 以向量內嵌並取主色 token 的顏色,字標 SHALL 以橫式鎖版呈現並依系統偏好切換淺色版與深色版;系統匣面板 SHALL NOT 顯示品牌標記。守門測試 SHALL 於桌面 app 或 server web 原始碼頂層出現與共用元件庫匯出同名的函式或常數定義時失敗並指名檔案與名稱。

#### Scenario: 系統匣面板的複製鈕來自共用元件庫

- **WHEN** 檢視系統匣面板任一變更列或討論列的複製鈕
- **THEN** 其實作與看板卡片、已封存清單、server web 的複製鈕為同一元件,複製成功後呈現同一款主色勾號 1.5 秒

#### Scenario: 兩鍵式確認框行為不變

- **WHEN** 使用者在桌面 app 觸發初始化確認、啟用確認、封存確認、刪除確認任一,或在 server web 觸發停用使用者確認
- **THEN** 確認框顯示與改版前相同的標題、說明、確認與取消文案;危險動作的確認鈕為紅系;進行中時兩鍵皆不可按

#### Scenario: 同名本地定義被守門擋下

- **WHEN** 桌面 app 或 server web 任一原始碼檔頂層定義了名為 CopyButton 的函式
- **THEN** 守門測試失敗並列出該檔案路徑與名稱

#### Scenario: 品牌資產只有一處

- **WHEN** 檢視桌面 app 零專案空狀態與 server web 主控台殼的品牌字標,以及桌面 app 圖示列頂端的品牌標記
- **THEN** 字標兩處引用共用元件庫的同一個字標元件與同一份影像資產;品牌標記引用共用元件庫的品牌標記元件

#### Scenario: 品牌標記隨主題取主色

- **WHEN** 系統偏好在淺色與深色之間切換
- **THEN** 品牌標記的顏色與主色 token 的對應值一致(淺色為青綠、深色為淺青綠),字標切換為對應的淺色版或深色版鎖版,兩者皆不需改動任何原始碼


<!-- @trace
source: desktop-shell-redesign
updated: 2026-10-09T12:33:58+08:00
-->

---
### Requirement: 隱藏原生標題列與三平台視窗殼

桌面 app SHALL 隱藏原生標題列的空白條並以自繪的兩條 48px 頂列取代:左側標題列(圖示列與專案欄之上)與主區頂列,兩者 SHALL 皆可拖曳移動視窗。macOS SHALL 以 Overlay 標題列隱藏標題文字、紅綠燈置於左側標題列左端(x=16)且與標題列內容垂直置中;Windows SHALL 關閉原生裝飾(保留陰影與圓角)並於主區頂列右端自繪三顆視窗鈕——縮到最小、最大化／還原(圖示依視窗狀態切換)、關閉,各 46×48、hover 灰底、關閉 hover 紅底白字;Linux SHALL 維持原生標題列,其下兩條頂列與 Windows 同一版面。三平台的左側標題列專案名左緣 SHALL 一律對齊下方專案欄導覽項文字左緣。主區頂列 SHALL 顯示麵包屑:有專案時為「專案名 / 頁名」(頁名為變更、已封存、規格、手冊或專案設定;專案名可點返回看板),設定模式為「設定」,零專案時為空;麵包屑 SHALL NOT 承載任何操作結果訊息。主區的通知列(如更新通知)SHALL 位於主區頂列之下,主區頂列 SHALL 維持在視窗最上緣、與左側標題列同高。看板頁主區 SHALL 以頁標題區開頭:24px 一般字重標題「變更」、灰字一句說明,右側同一列為既有的搜尋與篩選工具列(仍位於欄位上方)。

#### Scenario: macOS 紅綠燈落在左側標題列

- **WHEN** 於 macOS 啟動桌面 app 並開啟任一專案
- **THEN** 視窗頂端無原生標題列文字,紅綠燈位於左側標題列左端(x=16),其右側為專案名與「⋯」;拖曳左側標題列或主區頂列可移動視窗;主區頂列右端無視窗鈕

#### Scenario: Windows 自繪視窗鈕

- **WHEN** 於 Windows 啟動桌面 app
- **THEN** 無原生標題列,主區頂列右端有縮到最小、最大化／還原、關閉三顆鈕,按下分別使視窗縮到最小、切換最大化(鈕圖示隨之換成還原)、關閉;左側標題列的專案名左緣對齊下方導覽項文字

#### Scenario: Linux 維持原生標題列

- **WHEN** 於 Linux 啟動桌面 app
- **THEN** 原生標題列照舊,其下的左側標題列與主區頂列仍為 48px,專案名左緣對齊下方導覽項文字,無自繪視窗鈕

#### Scenario: 麵包屑隨頁面與專案切換

- **WHEN** 使用者於專案 A 的看板切到規格頁,再切到專案 B
- **THEN** 麵包屑依序為「A / 變更」→「A / 規格」→「B / 規格」;點麵包屑的專案名回到看板

#### Scenario: 看板頁的頁標題區

- **WHEN** 檢視任一專案的看板
- **THEN** 主區頂列下方為 24px 標題「變更」與灰字說明,同一列右端為搜尋輸入與篩選鈕,其下才是四欄;搜尋與篩選行為與「看板搜尋過濾卡片」需求一致

<!-- @trace
source: desktop-shell-redesign
updated: 2026-10-09T12:33:58+08:00
-->

---
### Requirement: 專案層檔案系統動作

桌面 app SHALL 於左側標題列的「⋯」與圖示列專案方塊的右鍵提供同一份專案動作選單:在檔案管理員顯示(名稱隨平台——macOS「在 Finder 顯示」、Windows「在檔案總管顯示」、Linux「在檔案管理員顯示」)、在終端機開啟、以編輯器開啟、複製路徑、分隔線、重新整理(快捷鍵 ⌘R／Ctrl+R)、關閉專案(紅字,快捷鍵 ⌘W／Ctrl+W)。本機專案的檔案系統動作 SHALL 作用於專案根目錄;remote 專案帶 checkout 時 SHALL 作用於 checkout 目錄,無 checkout 時選單 SHALL 只有重新整理與關閉專案。在終端機開啟 SHALL 依平台啟動:macOS 為系統 Terminal;Windows 為 Windows Terminal,沒有時退回 cmd;Linux 依序嘗試 x-terminal-emulator、gnome-terminal、konsole、xfce4-terminal。以編輯器開啟 SHALL 依序嘗試 PATH 上的 code、cursor、zed、subl。啟動後立即以失敗結束的候選(如不認得工作目錄參數)SHALL 視同無法啟動,改試下一個。動作成功 SHALL 靜默(無任何文字訊息);終端機或編輯器候選全部無法啟動時 SHALL 以 toast 說明(終端機:「找不到可用的終端機」;編輯器:「找不到可用的編輯器(支援 VS Code、Cursor、Zed、Sublime)」);在終端機開啟或以編輯器開啟時,專案目錄已不存在 SHALL 以 toast「專案目錄已不存在」說明且 SHALL NOT 嘗試任何候選(在檔案管理員顯示屬「其他失敗」,toast 錯誤原文);其他失敗 SHALL 以 toast 呈現錯誤原文。在非作用中專案的選單選「重新整理」SHALL 切換到該專案(切換即重新載入)。選單標示的快捷鍵 SHALL 觸發同一動作:⌘R(macOS)／Ctrl+R(Windows、Linux)重新整理作用中專案,Ctrl+W(Windows、Linux)關閉作用中專案;macOS 的 ⌘W 由系統選單先攔截,SHALL NOT 由網頁層處理。複製路徑 SHALL 把目錄路徑寫入系統剪貼簿。選單 SHALL 以共用元件庫的下拉選單與右鍵選單原語呈現(12px 圓角、浮層陰影、項目 hover 灰底、危險項紅字、右端等寬快捷鍵)。

#### Scenario: 在 Finder 顯示本機專案

- **WHEN** macOS 使用者按左側標題列「⋯」選「在 Finder 顯示」
- **THEN** Finder 開啟並選取該專案根目錄,app 畫面無任何文字訊息

#### Scenario: 右鍵方塊得到同一份選單

- **WHEN** 使用者在圖示列的本機專案方塊按右鍵
- **THEN** 彈出與標題列「⋯」相同的七個項目(六個動作與一條分隔線),「關閉專案」為紅字且右端顯示快捷鍵

#### Scenario: remote 專案無 checkout 時只剩兩項

- **WHEN** 作用中專案為未連接本機 checkout 的 remote 專案,使用者按「⋯」
- **THEN** 選單只有「重新整理」與「關閉專案」

#### Scenario: 找不到編輯器以 toast 說明

- **WHEN** PATH 上沒有 code、cursor、zed、subl 任一指令,使用者選「以編輯器開啟」
- **THEN** 出現 toast「找不到可用的編輯器(支援 VS Code、Cursor、Zed、Sublime)」,畫面其餘不變

##### Example: 終端機候選順序

| 平台 | 依序嘗試 | 全部失敗 |
| ---- | -------- | -------- |
| macOS | open -a Terminal <路徑> | toast「找不到可用的終端機」 |
| Windows | wt -d <路徑>;cmd /c start cmd /K cd /d <路徑> | 同上 |
| Linux | x-terminal-emulator、gnome-terminal、konsole、xfce4-terminal(各帶工作目錄) | 同上 |

#### Scenario: 專案目錄已不存在以 toast 說明

- **WHEN** remote 專案綁定的 checkout 目錄已被刪除,使用者選「在終端機開啟」
- **THEN** 出現 toast「專案目錄已不存在」,不啟動任何終端機

#### Scenario: 背景專案的重新整理切到該專案

- **WHEN** 使用者在非作用中的專案方塊按右鍵選「重新整理」
- **THEN** 該專案成為作用中專案並重新載入,原作用中專案不重新載入

#### Scenario: 快捷鍵觸發選單動作

- **WHEN** Windows 使用者按 Ctrl+R,再按 Ctrl+W
- **THEN** 作用中專案先重新載入,再被關閉

#### Scenario: 複製路徑寫入剪貼簿

- **WHEN** 使用者選「複製路徑」
- **THEN** 系統剪貼簿內容為該專案根目錄的完整路徑,畫面無任何文字訊息

<!-- @trace
source: desktop-shell-redesign
updated: 2026-10-09T12:33:58+08:00
-->

---
### Requirement: 看板欄與卡片的容器外觀

看板的欄 SHALL 等寬並填滿主區(四欄、間距 12px;每欄最小 220px,再窄時欄列水平捲動),每欄 SHALL 為白底細框 16px 圓角的容器:頂端為 3px 生命週期色相條(討論桃紫、提案中主色、進行中藍、已就緒綠,取自生命週期 token),其下為白底標頭(色相圖示、12px 中粗欄名、色相淡底計數徽章)與一條細線,卡片區 SHALL 鋪側欄同款淡灰底並於欄內縱向捲動。過濾後零卡且非載入中的欄 SHALL 於卡片區置中顯示灰字(提案中「沒有提案中的變更」、進行中「沒有進行中的變更」、已就緒「沒有已就緒的變更」、討論「尚無討論」)。變更卡與討論卡 SHALL 為 12px 圓角白底細框、SHALL NOT 帶陰影,hover 時框線變深、底色微灰;拖曳中的浮動卡 SHALL 帶陰影(浮層);封存落點浮層 SHALL 為主色淡底加主色虛線框,不合法落點 SHALL 以降透明度呈現;上述顏色 SHALL 只取 theme token。欄與卡的結構變更 SHALL NOT 改變欄位判定、拖排、搜尋與卡片三列骨架的任何語意。

#### Scenario: 四欄等寬且為容器卡

- **WHEN** 於 1440px 寬的視窗檢視含討論、提案中、進行中、已就緒四欄的看板
- **THEN** 四欄等寬、間距 12px、填滿主區;每欄為白底細框 16px 圓角容器,頂端色相條依序為桃紫、主色、藍、綠,標頭白底、卡片區淡灰

#### Scenario: 空欄顯示置中灰字

- **WHEN** 已就緒欄沒有任何變更,且搜尋字串使進行中欄無命中
- **THEN** 已就緒欄卡片區置中顯示「沒有已就緒的變更」、進行中欄顯示「沒有進行中的變更」,欄頭計數分別為 0

#### Scenario: 卡片無陰影而浮動卡有陰影

- **WHEN** 看板顯示變更卡,使用者開始拖曳其中一張
- **THEN** 靜置的卡片無陰影、hover 時框線變深底色微灰;拖曳中的浮動卡帶陰影;放開後浮動卡消失、卡片回到無陰影

<!-- @trace
source: desktop-board-reskin
updated: 2026-10-09T18:18:49+08:00
-->

---
### Requirement: macOS 原生選單

桌面 app 於 macOS SHALL 以自己的選單列取代 Tauri 預設選單,共六組:Speclink(關於 Speclink、檢查更新…、設定… ⌘,、服務、隱藏 Speclink ⌘H、隱藏其他 ⌥⌘H、顯示全部、結束 Speclink ⌘Q)、檔案(開啟專案… ⌘O、關閉專案 ⌘W、安裝 CLI…)、編輯(復原、重做、剪下、複製、貼上、全選)、檢視(變更 ⌘1、已封存 ⌘2、規格 ⌘3、手冊 ⌘4、專案設定、搜尋看板 ⌘F、重新整理 ⌘R、下一個專案 ⌃Tab)、視窗(縮到最小 ⌘M、縮放、全部移到最前)、說明(手冊、更新日誌、GitHub、回報問題)。選單文案 SHALL 依 UI 語言偏好呈現 zh-TW 或 en,語言切換後 SHALL 立即重建;沒有作用中專案時關閉專案、檢視組全部與說明的手冊 SHALL 停用。各項動作 SHALL 為:開啟專案開「新增專案」對話框;關閉專案於主視窗為焦點時關閉作用中專案(系統匣面板在前景時不動作);變更／已封存／規格／手冊／專案設定切至對應頁;搜尋看板聚焦看板搜尋輸入(不在看板時先切回看板);重新整理重讀作用中專案;下一個專案循環切換作用中專案;檢查更新執行手動檢查並進入應用程式設定頁的軟體更新卡;設定與安裝 CLI 進入應用程式設定頁;關於 Speclink 於「關於」頁落地前進入應用程式設定頁;更新日誌開啟更新日誌對話框;GitHub 與回報問題以系統瀏覽器開啟專案的 GitHub 頁與新 issue 頁;視窗組與編輯組為系統預設行為。Windows 與 Linux SHALL NOT 顯示選單列,既有鍵盤快捷鍵行為不變。

#### Scenario: 選單文案跟隨 UI 語言

- **WHEN** macOS 使用者於設定頁把 UI 語言自 zh-TW 切為 en
- **THEN** 選單列六組標題與項目立即變為英文(File、Edit、View、Window、Help 與 Open Project…、Close Project、Changes、Archived、Specs、Manual 等),切回 zh-TW 後立即變回繁中

#### Scenario: 檢視組快捷鍵切頁

- **WHEN** 有作用中專案時按 ⌘3
- **THEN** 主區切至規格頁、專案欄「規格」為作用中;按 ⌘1 回到看板

#### Scenario: 搜尋看板經選單聚焦

- **WHEN** 看板顯示中,使用者自檢視選單選「搜尋看板」或按 ⌘F
- **THEN** 看板搜尋輸入取得焦點,行為與「看板搜尋過濾卡片」需求的快捷鍵一致

#### Scenario: 非看板頁經選單搜尋看板

- **WHEN** 規格頁顯示中,使用者按 ⌘F 或自檢視選單選「搜尋看板」
- **THEN** 主區切回看板,看板搜尋輸入取得焦點

#### Scenario: 系統匣面板在前景時不關專案

- **WHEN** 系統匣面板為作用中視窗時使用者按 ⌘W
- **THEN** 主視窗的作用中專案維持開啟

#### Scenario: 無專案時相關項目停用

- **WHEN** app 於零專案狀態展開檔案與檢視選單
- **THEN** 「關閉專案」與檢視組全部項目呈停用、不可觸發;「開啟專案…」可用,按下開「新增專案」對話框

#### Scenario: 說明組開外部網址

- **WHEN** 使用者自說明選單選「回報問題」
- **THEN** 系統瀏覽器開啟 https://github.com/MomoChenisMe/speclink/issues/new,app 畫面無變化

#### Scenario: Windows 無選單列

- **WHEN** 於 Windows 啟動桌面 app
- **THEN** 視窗無選單列;Ctrl+Tab 與 Ctrl+1..9 仍可切換專案、Ctrl+F 仍聚焦看板搜尋

<!-- @trace
source: desktop-native-menu
updated: 2026-10-09T21:29:17+08:00
-->

---
### Requirement: 設定頁的鍵盤快捷鍵卡

應用程式設定的「一般」頁 SHALL 含「鍵盤快捷鍵」卡,置於該頁所有卡片之後;卡內每列左為動作名、右為按鍵;卡片 SHALL 不依賴任何已開啟的專案(零專案時照常顯示),且 SHALL NOT 提供編輯或自訂快捷鍵的操作。卡片 SHALL 逐列呈現「動作名稱+按鍵」,按鍵以等寬按鍵樣式呈現,列出的內容 SHALL 依平台而定且只列 app 實際生效的快捷鍵:

- **macOS** SHALL 依序列出原生選單上帶快捷鍵的自訂項目(設定… ⌘,、開啟專案… ⌘O、關閉專案 ⌘W、變更 ⌘1、已封存 ⌘2、規格 ⌘3、手冊 ⌘4、搜尋看板 ⌘F、重新整理 ⌘R、下一個專案 ⌃Tab),最後一列為「跳到第 1–9 個專案 ⌃1–9」;按鍵 SHALL 以 ⌘、⌃ 符號表示修飾鍵,且內容 SHALL 與「macOS 原生選單」需求的快捷鍵一致。系統預設項(⌘Q、⌘H、⌥⌘H、⌘M 與編輯組)SHALL NOT 列入。
- **Windows 與 Linux** SHALL 依序列出關閉專案 Ctrl+W、搜尋看板 Ctrl+F、重新整理 Ctrl+R、下一個專案 Ctrl+Tab、跳到第 1–9 個專案 Ctrl+1–9;SHALL NOT 出現 ⌘ 等 macOS 符號,也 SHALL NOT 列出只經 macOS 原生選單才生效的項目(設定、開啟專案、切頁)。

卡片標題與動作名稱 SHALL 依 UI 語言呈現 zh-TW 或 en,動作名稱 SHALL 與原生選單同一動作的文案相同。

#### Scenario: macOS 列出選單快捷鍵與跳專案

- **WHEN** macOS 使用者以 zh-TW 介面開啟應用程式設定的一般頁
- **THEN** 最後一張卡標題為「鍵盤快捷鍵」,依序列出 11 列動作與按鍵,按鍵皆為 ⌘／⌃ 符號

##### Example: macOS 的卡片內容

| 動作 | 按鍵 |
| ---- | ---- |
| 設定… | ⌘, |
| 開啟專案… | ⌘O |
| 關閉專案 | ⌘W |
| 變更 | ⌘1 |
| 已封存 | ⌘2 |
| 規格 | ⌘3 |
| 手冊 | ⌘4 |
| 搜尋看板 | ⌘F |
| 重新整理 | ⌘R |
| 下一個專案 | ⌃Tab |
| 跳到第 1–9 個專案 | ⌃1–9 |

#### Scenario: Windows 只列實際生效的 Ctrl 快捷鍵

- **WHEN** Windows 使用者以 zh-TW 介面開啟應用程式設定的一般頁
- **THEN** 「鍵盤快捷鍵」卡依序列出 5 列,畫面上沒有 ⌘ 符號,也沒有「設定…」「開啟專案…」與變更／已封存／規格／手冊的切頁列

##### Example: Windows 與 Linux 的卡片內容

| 動作 | 按鍵 |
| ---- | ---- |
| 關閉專案 | Ctrl+W |
| 搜尋看板 | Ctrl+F |
| 重新整理 | Ctrl+R |
| 下一個專案 | Ctrl+Tab |
| 跳到第 1–9 個專案 | Ctrl+1–9 |

#### Scenario: 卡片文案跟隨 UI 語言

- **WHEN** macOS 使用者於一般頁把 UI 語言切為 en
- **THEN** 卡片標題立即變為「Keyboard Shortcuts」,動作名稱變為 Settings…、Open Project…、Close Project、Changes、Archived、Specs、Manual、Search Board、Refresh、Next Project、Go to Project 1–9,按鍵不變

#### Scenario: 零專案時照常顯示

- **WHEN** app 於零專案狀態點圖示列的設定齒輪
- **THEN** 一般頁照常顯示「鍵盤快捷鍵」卡與完整列表


<!-- @trace
source: desktop-settings-mode
updated: 2026-10-10T23:13:14+08:00
-->

---
### Requirement: 詳情頁互斥

桌面 app SHALL 保證同時僅一個詳情頁（變更詳情、規格、已封存、討論）開啟：任一詳情頁開啟中，自任何入口（看板卡片、規格清單、已封存頁、系統匣選單與面板、頁內跳轉、‹ › 切換）開啟另一個詳情頁時，先開者 SHALL 關閉、由後開者取代顯示，SHALL NOT 疊加顯示、SHALL NOT 拒開。此互斥 SHALL 由狀態層的開啟動作保證——每個開啟動作於設定自身選定狀態時同時清除其他詳情頁的選定狀態；SHALL NOT 依賴各呼叫端入口自行先關再開。

變更詳情頁與討論詳情頁的開啟動作 SHALL 同時收掉可取消浮層：新增專案對話框、初始化確認框、啟用確認框、封存確認框（含品質關卡工單三選項框）、封存討論確認框、刪除確認框、退回提案中確認框、退回被擋說明框、更新日誌對話框、技能檔提示確認框。收掉的語意 SHALL 與使用者按該浮層的「取消」或「關閉」相同——SHALL NOT 送出任何未確認的動作；更新日誌於 whatsNew 模式被收掉時 SHALL NOT 記為已看過。此保證 SHALL 同樣由狀態層的開啟動作承擔，自任何入口開啟結果 SHALL 一致。

不可取消浮層（遷移對話框、遠端衝突對話框）開啟中，變更詳情頁與討論詳情頁的開啟動作 SHALL NOT 改變任何狀態：浮層維持、頁不開、宿主頁面不切換；自系統匣觸發時主視窗 SHALL 仍顯示到前景。規格詳情頁與已封存詳情頁的開啟動作 SHALL NOT 受本段約束（其宿主頁為規格頁與已封存頁）。

#### Scenario: 討論詳情頁開啟中自系統匣開啟變更詳情

- **WHEN** 討論詳情頁開啟中，使用者自系統匣開啟某 change 的詳情頁
- **THEN** 討論詳情頁關閉，僅變更詳情頁可見

#### Scenario: 任兩種詳情頁交錯開啟

- **WHEN** 任一詳情頁開啟中，使用者自任何入口開啟另一種詳情頁
- **THEN** 先開者關閉、後開者顯示，同時僅一個詳情頁可見

##### Example: 四種詳情頁互斥狀態轉移

| 開啟中的詳情頁 | 開啟動作         | 結果                       |
| -------------- | ---------------- | -------------------------- |
| 討論           | 開啟變更詳情     | 討論關閉，僅變更詳情可見   |
| 變更詳情       | 開啟規格         | 變更詳情關閉，僅規格可見   |
| 規格           | 開啟已封存       | 規格關閉，僅已封存可見     |
| 已封存         | 開啟討論         | 已封存關閉，僅討論可見     |

#### Scenario: 新增專案對話框開啟中自系統匣開啟變更詳情

- **WHEN** 新增專案對話框開啟中，使用者自系統匣面板或原生選單選「開啟此變更」
- **THEN** 對話框關閉（未開啟任何專案）、變更詳情頁開啟、宿主為看板，畫面沒有任何浮層

#### Scenario: 可取消確認框開啟中開啟討論詳情頁

- **WHEN** 初始化確認、啟用確認、封存確認（含工單三選項框）、封存討論確認、刪除確認、退回提案中確認、退回被擋說明或技能檔提示確認任一開啟中，使用者自任何入口開啟某討論
- **THEN** 該確認框關閉且未送出任何動作，討論詳情頁開啟

##### Example: 可取消浮層與開啟動作的狀態轉移

| 開啟中的浮層 | 開啟動作     | 結果                                   |
| ------------ | ------------ | -------------------------------------- |
| 新增專案     | 開啟變更詳情 | 對話框關閉、詳情頁開啟、無專案被開啟   |
| 封存確認     | 開啟討論     | 確認框關閉、詳情頁開啟、無封存發生     |
| 工單三選項框 | 開啟變更詳情 | 三選項框關閉、詳情頁開啟、工單維持未結 |
| 刪除確認（change A） | 開啟討論 | 確認框關閉、詳情頁開啟、change A 未被刪除 |
| 更新日誌（whatsNew） | 開啟討論 | 對話框關閉、詳情頁開啟、下次啟動仍彈出 |
| 技能檔提示確認 | 開啟變更詳情 | 確認框關閉、詳情頁開啟、技能檔未改寫 |

#### Scenario: 更新日誌對話框開啟中自系統匣開啟變更詳情

- **WHEN** 更新日誌對話框開啟中，使用者自系統匣開啟某 change 的詳情頁
- **THEN** 更新日誌對話框關閉、變更詳情頁開啟；whatsNew 模式下 SHALL NOT 記為已看過

#### Scenario: 遷移進行中自系統匣開啟變更詳情

- **WHEN** 遷移對話框開啟中，使用者自系統匣面板選「開啟此變更」
- **THEN** 主視窗顯示到前景、遷移對話框維持、變更詳情頁不開、宿主頁面不變

#### Scenario: 遠端衝突對話框開啟中開啟討論

- **WHEN** 遠端衝突對話框開啟中，使用者自任何入口開啟某討論
- **THEN** 衝突對話框維持、討論詳情頁不開、任何狀態不變

<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 討論詳情頁檢視與轉出變更

點擊討論卡或細列 SHALL 開啟討論詳情頁，採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架。左欄身分區 SHALL 以討論 slug 為題（等寬 14px）、緊跟複製 slug 鈕與狀態籤（討論中／已結論／已轉出／已轉出・尚無結論／已轉出・保留中，判準與看板討論卡一致），topic SHALL 為其下最多三行、溢出截斷的文字（slug 為題屬 openspec/LANGUAGE.md 明載的受控例外）；出身 SHALL 一行依序呈現建立者頭像＋名字、「N 輪」與建立日，缺席的段整段缺席；其下「關聯」列呈現先前討論數（自背景的 `Prior discussions:` 行計數；`none` 時該列缺席），點擊 SHALL 切到背景文件。主區標題列右端 SHALL 為動作區：concluded 且未封存的討論、以及已轉出且保留在途（hold 為 true）的討論呈現「封存」動作（outline）；保留在途時「封存」SHALL 停用並以提示說明結論保留在途、最後一刀轉出後自動封存；其餘（討論中、已轉出且未保留、已封存）的討論 SHALL NOT 呈現封存動作。

左欄「狀態」列 SHALL 為：「階段」列為步驟列「討論中 → 已結論 → 已轉出 → 已封存」且現站可辨識（promoted 為已轉出、concluded 為已結論、其餘為討論中；已封存站保留給已封存討論頁）；現站籤的色調 SHALL 與身分區狀態籤一致——狀態籤為「已轉出・保留中」或「已轉出・尚無結論」時現站為 warning 語意色，其餘為主色；現站之前的站 SHALL 呈現為已過站（打勾），但已轉出且尚無結論（concluded 為 false）時「已結論」站 SHALL 呈現為未到站（灰圈、無勾）；hold 為 true 時 SHALL 另有「保留在途」列呈現說明；promoted_to 非空時 SHALL 另有「衍生變更」列，每個子變更一顆可點等寬籤並附階段籤（階段派生與配色同「討論於看板第 0 欄兩級呈現」的 chip 矩陣），點籤 SHALL 開啟該變更的詳情頁；無衍生變更時該列缺席。

左欄「文件」節點依序 SHALL 為：結論、討論過程 N、背景、衍生變更 N——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯，此時節點為「記錄」與「衍生變更」）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」，結論為空時預設「背景」。結論與討論過程的畫法見「討論結論以欄位標籤呈現」與「討論輪以卡片呈現」。

背景文件 SHALL 先把 Context 區段收尾的 `Related specs:`／`Related changes:`／`Prior discussions:`／`Source doc:` 四行自內文剝離，其餘分三段（各為本頁目錄錨點）：「起因與範圍」——第一段散文為引言、以灰底引言段呈現，其餘照 markdown，內文中與正式規格名、作用中變更名、已封存變更名、其他討論 slug 完全相同的字詞（前後皆非英數、連字號與底線；反引號內的不算）SHALL 自動成為可點籤，點擊分別開啟該規格的規格詳情頁、該變更的詳情頁、該封存變更的已封存詳情頁、該討論的詳情頁；「關聯」——鍵值列「正式規格」「在途變更」「已封存變更」「先前討論」「來源文件」，值為前述四行解析出的名稱（去掉括號備註、以逗號、頓號或分號切項、`none` 為空）與內文命中名稱的聯集籤（可點規則同上），值為空的列 SHALL 缺席（來源文件為 `none` 時顯示「無」、行缺席時列缺席），全部缺席時「關聯」段缺席；「記錄」——「建立」（建立日與建立者名字）、「種類」（kind 為 improve 時「改進討論」、否則「一般討論」）、「檔案」（live 討論為 `openspec/discussions/<slug>.md` 等寬文字附複製鈕；已封存討論頁該列缺席）。背景文件的數字磚 SHALL 為「命中的正式規格」「在途變更」「相關已封存變更」（各為關聯列的籤數，0 時缺席）與「先前討論」（`Prior discussions:` 行缺席時缺席，`none` 為 0）；出處行為「記錄建立時寫入一次」與建立日；首輪 mode 為 `assumptions` 時 SHALL 有 note 提示框「沒有 grill 階段」，否則無提示框。

衍生變更文件 SHALL 列出各子變更現況與跳轉且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。結論的 Decision 列點可解析出分刀（見「討論結論以欄位標籤呈現」）時 SHALL 依刀分組：每組組頭為刀的識別籤、範圍句與該組變更數（組頭為本頁目錄錨點），組內列為該刀變更名籤中存在於 `promoted_to` 的變更、依籤序；不屬任何刀的子變更接在最後、無組頭。解析不出分刀時 SHALL 退回依轉出順序的平面清單、無組頭。每列 SHALL 呈現：階段籤（作用中變更依看板階段；已封存為「已封存」中性；兩者皆無為「已刪除」警示色加刪除線）、等寬變更名、任務進度（細進度條與 n/m；作用中取其任務完成數／總數、已封存取封存清單的任務數；無資料時缺席）、審查章與驗證章（作用中變更帶審查章或驗證章、已封存變更的結局為已審查或已驗證時各自出現）、日期（已封存為封存日期；作用中為最近蓋章日；皆無時缺席）、開啟鈕（作用中開啟該變更的詳情頁、已封存開啟其已封存詳情頁；已刪除無鈕）。衍生變更文件的數字磚 SHALL 為「變更」（總數）、「已封存」、「作用中」、「任務」（有任務資料的列加總 n/m；皆無資料時缺席）；出處行為「依結論的分刀分組」（有分刀時）或「依轉出順序」；討論為已轉出、hold 為 false 且恰有一個作用中子變更時 SHALL 有 tip 提示框「只差最後一刀」，內文點名該變更與其階段並說明它封存後討論會自動隨之封存；其他情況 SHALL NOT 有提示框。無衍生變更時衍生變更文件顯示既有空態文案、無數字磚。已封存討論頁的衍生變更文件 SHALL 同款（無「只差最後一刀」提示框）。

concluded 且未封存討論的封存動詞 SHALL 於討論卡與討論詳情頁皆可及，兩處 SHALL 走同一確認流程，經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情頁 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 頁標題以 slug 為題且可複製

- **WHEN** 使用者開啟任一討論的詳情頁並點按左欄身分區 slug 旁的複製鈕
- **THEN** 身分區呈現該討論的 slug（等寬字型）、其下呈現 topic；slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的詳情頁
- **THEN** 左欄「文件」節點為 結論／討論過程 N／背景／衍生變更 N，且預設閱讀卡呈現結論內容；「狀態」列的階段以「已結論」為現站

#### Scenario: 狀態卡的保留與衍生列

- **WHEN** 使用者開啟一筆已轉出、已有結論、hold 為 true、promoted_to 含一個已封存與一個進行中子變更的討論詳情頁
- **THEN** 身分區狀態籤為「已轉出・保留中」；「狀態」列的階段現站為「已轉出」且為 warning 語意色、與狀態籤同色，「討論中」與「已結論」兩站打勾；「保留在途」列呈現說明；「衍生變更」列兩顆籤分別附「已封存」與「進行中」階段籤，點進行中那顆開啟該變更的詳情頁；標題列右端的「封存」停用並有提示

#### Scenario: 尚無結論的已轉出討論的步驟列

- **WHEN** 使用者開啟一筆已轉出、concluded 為 false 的討論詳情頁
- **THEN** 身分區狀態籤為 warning 語意色的「已轉出・尚無結論」；「狀態」列的步驟列「討論中」打勾、「已結論」為灰圈且無勾、現站「已轉出」為 warning 語意色、「已封存」為未到站

#### Scenario: 已轉出且未保留的討論現站為主色

- **WHEN** 使用者開啟一筆已轉出、已有結論、hold 為 false 的討論詳情頁
- **THEN** 身分區狀態籤為主色的「已轉出」；步驟列現站「已轉出」為主色，「討論中」與「已結論」兩站打勾

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的衍生變更文件
- **THEN** 閱讀卡列出各子變更現況與開啟鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論詳情頁或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情頁
- **THEN** 左欄身分區的「來自」列顯示來源討論的 slug 籤（topic 於提示呈現）、「同源」列顯示同源變更籤，點擊同源籤開啟該變更的詳情頁

#### Scenario: 頁內封存 concluded 討論

- **WHEN** 使用者開啟一筆已結論、未封存且無 hold 討論的詳情頁，點標題列右端的「封存」並於確認對話框確認
- **THEN** 該討論移入封存：自討論欄消失、出現於已封存頁討論節，與自討論卡封存的效果一致；詳情頁關閉回看板

#### Scenario: 背景文件的引言、關聯與記錄列

- **WHEN** 使用者開啟一筆 live 討論（2026-10-08 由 MomoChen 建立、無 kind）的背景文件，其 Context 內文提到正式規格 desktop-app 與 tray-status-menu、作用中變更 add-copilot-tool、已封存變更 tray-panel-card-design，收尾行為「Related specs: desktop-app（79 條）、tray-status-menu」「Prior discussions: card-drawer-header-colors, admin-console-redesign」「Source doc: none」
- **THEN** 「起因與範圍」的第一段為灰底引言，內文中 desktop-app、tray-status-menu、add-copilot-tool、tray-panel-card-design 各成可點籤且收尾四行不出現在內文；「關聯」列為正式規格（desktop-app、tray-status-menu）、在途變更（add-copilot-tool）、已封存變更（tray-panel-card-design）、先前討論（兩顆）、來源文件「無」；「記錄」列為建立「2026-10-08 · MomoChen」、種類「一般討論」、檔案 `openspec/discussions/<slug>.md` 附複製鈕；數字磚為 2「命中的正式規格」、1「在途變更」、1「相關已封存變更」、2「先前討論」

#### Scenario: 背景的名稱籤跳轉與沒有 grill 階段提示框

- **WHEN** 使用者於上述背景文件點內文的 desktop-app 籤，再回到該討論點 add-copilot-tool 籤；該討論第一輪 mode 為 assumptions
- **THEN** 第一次開啟 desktop-app 的規格詳情頁（討論詳情頁依「詳情頁互斥」關閉），第二次開啟 add-copilot-tool 的變更詳情頁；背景文件有 note 提示框「沒有 grill 階段」，出處行為「記錄建立時寫入一次 · 2026-10-08」

#### Scenario: 背景無關聯行時關聯段缺席

- **WHEN** 使用者開啟一筆 Context 為兩段純散文、無收尾四行且內文未提及任何規格、變更或討論名稱的討論背景文件
- **THEN** 只有「起因與範圍」與「記錄」兩段，無「關聯」段；數字磚整列缺席；無名稱籤

#### Scenario: 衍生變更依分刀分組

- **WHEN** 使用者開啟 desktop-ui-redesign（結論含 cut 0～cut 5 共 7 張分刀卡、promoted_to 13 個變更，12 個已封存、1 個已就緒且兩站蓋章）的衍生變更文件
- **THEN** 列依 7 組呈現，各組組頭為 cut 識別籤、範圍句與變更數（cut 2 為 3 個變更），組頭列入本頁目錄；每列有階段籤、等寬名稱、任務進度 n/m、「審」「驗」章、日期與開啟鈕；數字磚為 13「變更」、12「已封存」、1「作用中」與「133 / 133」「任務」；出處行為「依結論的分刀分組」

##### Example: 列的欄位來源

| 子變更所在 | 階段籤 | 任務 | 審查章／驗證章 | 日期 | 開啟鈕 |
| --- | --- | --- | --- | --- | --- |
| 作用中（已就緒、已審查、已驗證） | 已就緒 | completedTasks/totalTasks | 兩章皆有 | 最近蓋章日 | 開啟變更詳情頁 |
| 作用中（進行中、審查中） | 進行中 | completedTasks/totalTasks | 無 | 缺席 | 開啟變更詳情頁 |
| 已封存（結局 reviewed、verified） | 已封存 | tasksDone/tasksTotal | 兩章皆有 | 封存日期 | 開啟已封存詳情頁 |
| 已封存（結局 reviewedNotPassed） | 已封存 | tasksDone/tasksTotal | 無審查章 | 封存日期 | 開啟已封存詳情頁 |
| 兩者皆無 | 已刪除（刪除線） | 缺席 | 無 | 缺席 | 無 |

#### Scenario: 讀不到分刀時退回平面清單

- **WHEN** 使用者開啟一筆結論的 Decision 沒有任何以 cut 或刀起頭的粗體列點、promoted_to 含兩個變更的討論衍生變更文件
- **THEN** 兩列依轉出順序平面呈現、無組頭；出處行為「依轉出順序」；數字磚照常

#### Scenario: 只差最後一刀提示框

- **WHEN** 使用者開啟一筆已轉出、hold 為 false、promoted_to 含兩個已封存與一個已就緒變更 tray-panel-restyle 的討論衍生變更文件
- **THEN** tip 提示框標題「只差最後一刀」、內文點名 tray-panel-restyle 與「已就緒」並說明它封存後討論會自動隨之封存；同一討論的 hold 為 true 時、或作用中子變更有兩個時，無此提示框


<!-- @trace
source: discussion-stage-and-unpromoted-cuts
updated: 2026-10-13T22:22:52+08:00
-->

---
### Requirement: 詳情頁標頭標記受寬度約束且頁不產生水平捲軸

變更詳情頁與已封存詳情頁的左欄身分區中,來源討論標記 SHALL 以討論 slug 直出(等寬字型),topic SHALL NOT 於標記面直出。標記 SHALL 設寬度上限:slug 超出上限時 SHALL 以截斷呈現,SHALL NOT 使標記寬度超出左欄。標記 SHALL 以主題化提示呈現完整 slug 與 topic,使用者 SHALL 能取得未截斷的原始內容。多份來源討論 SHALL NOT 各自換行——僅第一份直接顯示,其餘收進「+N」數字籤與其浮層(呈現規則見「變更的來源討論多值呈現」)。詳情頁 SHALL NOT 於水平方向產生捲軸——不論身分區標記、標題列動作或閱讀卡內文包含多長的單一元素,左欄、標題列與閱讀卡 SHALL 維持在可視寬度內。此約束 SHALL 同時套用於變更詳情頁與已封存詳情頁,SHALL NOT 只在其中一處成立。

來源討論標記的資料來源、數量、排序,以及點擊(標記或浮層項)後開啟對應討論的行為 SHALL 維持不變。

#### Scenario: 長討論 topic 不撐寬詳情頁

- **WHEN** 使用者開啟一個來源討論的 topic 長度遠超過詳情頁寬度的變更,其詳情頁
- **THEN** 該來源討論標記顯示 slug 而非 topic,寬度不超出左欄,詳情頁沒有水平捲軸,左欄、標題列與閱讀卡皆維持在可視寬度內

#### Scenario: 截斷後仍可取得完整 topic

- **WHEN** 使用者將指標停留在來源討論標記上
- **THEN** 主題化提示呈現該討論完整的 slug 與 topic 文字

#### Scenario: 已封存詳情頁適用同一約束

- **WHEN** 使用者於已封存頁開啟同一個變更的詳情頁
- **THEN** 來源討論標記同樣以 slug 直出、多份時溢出收「+N」籤,詳情頁沒有水平捲軸

#### Scenario: 超長 slug 截斷呈現

- **WHEN** 使用者開啟來源討論 slug 長於標記寬度上限的變更,其詳情頁
- **THEN** 該標記以截斷呈現且寬度不超出上限,提示保有完整 slug 與 topic;點擊標記開啟對應討論的詳情頁

##### Example: 標記呈現與詳情頁捲軸

| 情境                 | 標記面呈現                  | 提示           | 詳情頁水平捲軸 |
| -------------------- | --------------------------- | -------------- | ------------ |
| slug 短於寬度上限    | slug 完整                   | slug 與 topic  | 無           |
| slug 超過寬度上限    | slug 截斷                   | slug 與 topic  | 無           |
| 多份來源討論         | 首份 slug+「+N」籤,單行     | 各籤 slug 與 topic | 無       |


<!-- @trace
source: desktop-detail-doc-tree
updated: 2026-10-12T13:39:16+08:00
-->

---
### Requirement: 詳情頁的封存與刪除依階段守門

變更詳情頁「⋯」選單的「封存」項 SHALL 僅於該 change 派生階段為已就緒時可選；非已就緒時 SHALL 呈現停用並於項目下方以灰字說明原因（載明任務進度與「完成後才能封存」的出路）。已就緒時標題列另 SHALL 呈現「封存」主要鈕，與選單項同一守門。「刪除」項 SHALL 置於選單底部、以警示色呈現，僅於派生階段為提案中時可選；非提案中時 SHALL 呈現停用並說明原因（載明已有開工痕跡與「先退回提案中」的出路）。remote session 下對應能力缺失時，能力缺失原因 SHALL 優先於階段原因呈現。守門為呈現層過濾：可按期間階段已於外部改變（併發寫入）時，引擎拒絕 SHALL 為最終裁決，依既有失敗 toast 語意呈現。桌面 SHALL NOT 提供任何略過守門的強制通道。

#### Scenario: 非已就緒的封存項停用附原因

- **WHEN** 使用者開啟提案中（0/11）與進行中（5/19）change 的詳情頁並展開「⋯」選單
- **THEN** 兩者「封存」項皆停用，項目下方各載明任務進度與完成後才能封存；標題列無「封存」主要鈕。已就緒 change 的「封存」項與標題列主要鈕照常可按並走既有確認流程

#### Scenario: 非提案中的刪除項停用附原因

- **WHEN** 使用者開啟進行中 change 的詳情頁並展開「⋯」選單
- **THEN** 「刪除」項停用，項目下方載明已有開工痕跡並指向先退回提案中；提案中 change 的「刪除」項照常可按

#### Scenario: 併發階段變化由引擎拒絕保底

- **WHEN** 封存可按期間另一 session 於 tasks.md 取消勾選一個任務，使用者隨後確認封存
- **THEN** 引擎拒絕封存，app 依既有失敗 toast 語意呈現拒絕訊息，change 仍在看板、詳情頁維持開啟

<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 變更詳情頁的標頭與狀態卡

變更詳情頁 SHALL 採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架：原標題列與出身列的內容 SHALL 落在左欄身分區，原「狀態」卡 SHALL 落在左欄「狀態」列，分析結果（有分析結果時）SHALL 以一張卡呈現於閱讀卡之上。身分區 SHALL 為等寬 14px 變更名（過長折行）緊跟複製名稱鈕，其下為階段籤。身分區 SHALL NOT 顯示審查與驗證章籤——品質關卡由「狀態」列的「品質關卡」列承載；身分區 SHALL NOT 顯示任務計數——任務進度由「狀態」列與「任務」節點徽章承載。

出身 SHALL 為一行灰字，依序呈現：建立者頭像＋名字、產生工具、建立相對時間、「開工 YYYY-MM-DD」（有開工戳記時）；其下「來自」列呈現來源討論籤（有來源討論時）、「同源」列呈現同源變更籤（有同源變更時）、「規格」列呈現 delta 能力籤（有 delta 時）；缺席的資訊段與缺席的列 SHALL 整段缺席而非留白。建立者與開工者的完整識別（含 email）SHALL 收進主題化提示，可視文字 SHALL NOT 直出 email。出身行於任何資料組合下 SHALL 維持單行、溢出裁切；關聯列各自一行、籤溢出時收「+N」浮層；SHALL NOT 使頁出現水平捲軸。

變更層動作 SHALL 位於主區標題列右端、‹ n／m › 之後並以細線分隔，由左而右為：「分析」（outline）、依派生階段的主要鈕（實心主色：進行中「退回提案中」、已就緒「封存」、remote 可認領時「認領」）、「⋯」選單（outline 圖示鈕；項目與守門依「詳情頁的封存與刪除依階段守門」）；動作鈕 SHALL NOT 折行、SHALL NOT 撐寬標題列。

「狀態」列 SHALL 為三列細線分隔、左標籤右內容：「任務進度」列呈現進度條與 `n/m`；「品質關卡」列於審查或驗證狀態非 `none` 時 SHALL 以狀態籤呈現各站章籤（審查在前、驗證在後，圖示＋狀態詞，i18n 沿用既有詞條），已蓋章的章籤旁直出蓋章日期（`inReview`／`inVerify` 僅狀態詞、無日期；兩站皆 `none` 時灰字「尚未經過品質關卡」）；蓋章日期與蓋章者完整識別（含 email）SHALL 收進該章籤的主題化提示，進行中或無戳記時提示缺席，可視文字 SHALL NOT 直出 email；「排程」列於清單項帶 `wave` 時呈現「第 N 波」與前置說明（`blockedBy` 空為「可以開工」、非空為「等 N 項」），缺 `wave` 時沿用排程文件的說明詞條。

#### Scenario: 標頭結構與任務計數位置

- **WHEN** 使用者開啟任一變更的詳情頁
- **THEN** 左欄由上而下為身分區（變更名、階段籤、出身、關聯列）、「文件」節點、「狀態」列；身分區可視文字不含任務計數與審查／驗證章籤，「狀態」列的「任務進度」與「任務」節點徽章呈現任務進度

#### Scenario: 章籤與品質關卡列

- **WHEN** 使用者開啟 reviewStatus 為 reviewed（reviewedAt 2026-10-12）且 verifyStatus 為 verified（verifiedAt 2026-10-13）、兩站蓋章者皆含 email 的已就緒變更詳情頁
- **THEN** 身分區只有「已就緒」階段籤，無「已審查」「已驗證」章籤；「狀態」列的「品質關卡」顯示「已審查 2026-10-12」「已驗證 2026-10-13」兩枚章籤，可視文字不含 email；將指標停留於「品質關卡」列的「已審查」章籤時，主題化提示顯示 2026-10-12 與審查者含 email 的完整識別，停留於「已驗證」時顯示驗證站自己的日期與識別；頁無水平捲軸

#### Scenario: 品質關卡列的空態與進行中

- **WHEN** 使用者分別開啟 reviewStatus 與 verifyStatus 皆為 none、以及 reviewStatus 為 inReview 的變更詳情頁
- **THEN** 兩者身分區皆無章籤；前者「品質關卡」列顯示「尚未經過品質關卡」；後者「品質關卡」列顯示「審查中」且無日期，將指標停留於該章籤時無主題化提示

#### Scenario: 排程列

- **WHEN** 使用者開啟 wave=2、blockedBy=["add-a"] 的變更詳情頁
- **THEN** 「狀態」列的「排程」顯示「第 2 波」與「等 1 項」；wave 缺席的變更改顯排程文件的說明詞條

#### Scenario: email 收進提示

- **WHEN** 使用者開啟 created_by 含 email 的變更詳情頁，並將指標停留於建立者名字
- **THEN** 出身行可視文字僅顯示頭像與名字，主題化提示呈現含 email 的完整識別；開工者的完整識別亦僅於提示呈現

#### Scenario: 出身列恆定單行

- **WHEN** 使用者開啟一個有 4 份來源討論、有開工戳記且有同源變更的變更詳情頁
- **THEN** 出身行維持單行；「來自」列直接顯示出身討論籤與「+3」籤、「同源」列顯示同源籤，各列不撐寬左欄，頁無水平捲軸

#### Scenario: 標題列的變更層動作

- **WHEN** 使用者開啟一個已就緒、未蓋章的變更詳情頁
- **THEN** 主區標題列右端為 ‹ n／m ›、細線、「分析」outline 鈕、「封存」實心主色鈕、「⋯」outline 圖示鈕；左欄與閱讀卡內沒有這些動作鈕


<!-- @trace
source: detail-quality-stamp-dedup
updated: 2026-10-13T23:23:36+08:00
-->

---
### Requirement: 討論詳情頁的改進標示

討論詳情頁 SHALL 於 kind 為 improve 時顯示改進標示,文案與卡片 tooltip 使用同一 i18n 詞條;一般討論的詳情頁 SHALL NOT 顯示該標示。

#### Scenario: 改進討論詳情頁

- **WHEN** 開啟 kind 為 improve 的討論詳情頁(語言 tw)
- **THEN** 詳情頁顯示「改進討論」標示

#### Scenario: 一般討論詳情頁無標示

- **WHEN** 開啟無 kind 欄位的討論詳情頁
- **THEN** 詳情頁不顯示改進標示,與本變更引入前的渲染一致

<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 變更與討論詳情頁的宿主為看板

變更詳情頁與討論詳情頁的開啟動作 SHALL 於設定選定狀態的同一次狀態更新內，把宿主頁切回看板——不論開啟入口（看板卡片、系統匣選單與面板、頁內跳轉、封存前確認對話框的「去蓋章」）與開啟當下所在頁面（規格頁、已封存頁、專案設定頁、應用程式設定頁）。此歸位 SHALL 由狀態層的開啟動作保證，SHALL NOT 依賴各呼叫端入口自行切頁。規格詳情頁與已封存詳情頁 SHALL NOT 在此列——其宿主為開啟當下所在的規格頁、已封存頁或手冊頁，開啟時宿主頁面 SHALL 維持不變。

詳情頁開啟期間宿主頁 SHALL 保持掛載但不可見、不可互動（鍵盤與指標皆不可達）；關閉詳情頁後宿主頁 SHALL 以開啟前的捲動位置、搜尋字串與篩選狀態呈現，SHALL NOT 重新載入或重置。保持掛載的宿主頁 SHALL NOT 讓主區產生超出可視高度的可捲範圍——宿主頁內容（如看板卡片上複製鈕的隱藏朗讀區）的定位 SHALL 落在其所屬捲動區或定位容器之內；因此詳情頁內的定位捲動（手動提示框「前往」、「下一個未完成」與 n 快捷鍵、本頁目錄、D 籤跳轉）只捲閱讀區，詳情頁外框（主區標題列、左欄文件樹、閱讀區的上下緣）SHALL NOT 位移，主區底部 SHALL NOT 露出空白。

#### Scenario: 已封存頁自系統匣開啟變更詳情

- **WHEN** 使用者身處已封存頁，自系統匣點擊某活躍 change 的「開啟此變更」
- **THEN** 主視窗前置、宿主頁切回看板、該 change 的詳情頁開啟——關閉詳情頁後所見為看板而非已封存頁

#### Scenario: 規格頁開啟討論詳情頁落回看板

- **WHEN** 使用者身處規格頁，經任一入口開啟某活躍討論的詳情頁
- **THEN** 宿主頁切回看板、討論詳情頁開啟

#### Scenario: 已封存頁開啟已封存詳情頁不切頁

- **WHEN** 使用者於已封存頁點擊一筆封存變更列
- **THEN** 已封存詳情頁開啟，宿主頁面維持已封存頁

#### Scenario: 關閉詳情頁後看板狀態保留

- **WHEN** 使用者於看板輸入搜尋字串、啟用一個篩選並把「進行中」欄捲到底，點一張卡開詳情頁，再按 Esc
- **THEN** 看板重新可見且搜尋字串、篩選與該欄捲動位置與開頁前相同；詳情頁開啟期間看板不可見且無法以 Tab 鍵聚焦

#### Scenario: 宿主頁不推動詳情頁外框

- **WHEN** 看板有 30 張卡、「進行中」欄的卡片多到欄內需要捲動，使用者開啟某變更詳情頁的任務文件，點手動任務提示框中清單最後一個手動任務的「前往」
- **THEN** 閱讀區捲到底、該任務列短暫高亮；左欄文件樹頂端仍看得到變更名、主區標題列與閱讀區的上下緣位置與點擊前相同，主區底部沒有空白；關閉詳情頁後看板也沒有上移


<!-- @trace
source: copy-button-live-region-pin
updated: 2026-10-14T13:25:41+08:00
-->

---
### Requirement: 詳情頁文件載入以 skeleton 呈現

詳情頁（變更、規格、討論、已封存）的各文件 SHALL 統一區分文件三態：載入中 SHALL 渲染文件 skeleton（標題條與數行內文條），載入完成且檔案不存在才 SHALL 顯示該文件的空態文案，載入完成且有內容 SHALL 渲染內容。任何文件 SHALL NOT 於載入中顯示「無文件」類空態文案。點擊左欄文件節點時，節點 SHALL 立即標示為作用中、閱讀卡標題區 SHALL 立即換成該文件的標題，內文尚未抵達時 SHALL 畫 skeleton。

#### Scenario: 文件載入中顯示骨架

- **WHEN** 開啟詳情頁的任一文件且文件內容尚未抵達
- **THEN** 該文件的閱讀卡顯示文件 skeleton，不顯示任何空態文案

#### Scenario: 載入完成檔案不存在

- **WHEN** 文件載入完成且該檔案不存在
- **THEN** skeleton 消失，顯示該文件既有的空態文案

#### Scenario: 載入完成有內容

- **WHEN** 文件載入完成且有內容
- **THEN** skeleton 消失，內容以既有渲染路徑呈現

#### Scenario: 切換文件節點的即時回饋

- **WHEN** 使用者於變更詳情頁點左欄的「設計」節點而 design.md 的內容尚未抵達
- **THEN** 「設計」節點立即為作用中、閱讀卡標題立即為「設計」且內文為 skeleton；內容抵達後取代 skeleton


<!-- @trace
source: desktop-detail-doc-tree
updated: 2026-10-12T13:39:16+08:00
-->

---
### Requirement: 詳情頁與浮層的進出動畫

桌面 app 的詳情頁（變更、討論、規格、已封存）開啟時 SHALL 以 150ms 淡入並自下方 4px 位移到位；關閉時 SHALL 直接移除節點，無退場動畫。浮層（popover、tooltip、對話框、選單）SHALL 沿用元件庫既有的進出動畫。使用者系統偏好 prefers-reduced-motion 時，上述動畫 SHALL 停用，詳情頁與浮層立即出現與消失。動畫 SHALL NOT 改變詳情頁的焦點、關閉與內容行為。

#### Scenario: 詳情頁進場

- **WHEN** 使用者在看板開啟變更詳情頁，再按 Esc 關閉
- **THEN** 詳情頁淡入並自下方微位移到位；關閉時立即消失、看板立即可見

#### Scenario: 減少動態偏好下無動畫

- **WHEN** 系統設定 prefers-reduced-motion 後開啟任一詳情頁
- **THEN** 詳情頁立即出現、無淡入與位移；浮層亦立即出現與消失

<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 詳情頁的排程分頁

change 詳情頁 SHALL 於左欄「文件」節點「規格」之後提供「排程」節點（tw 標籤「排程」、en「Plan」），點擊 SHALL 於閱讀卡呈現排程文件。清單項帶 `wave` 時閱讀卡 SHALL 依序呈現四張卡片（與審查／驗證文件同一種邊框卡片：卡頭與內容區以分隔線分格；卡頭為 16px 粗體標題帶本頁目錄錨點，重疊卡與封存順序卡的卡頭右端附筆數籤），data-plan-section 依序為 wave、depends、overlaps、archive：波次卡（「第 N 波」＋同波其他變更名稱清單，無其他時顯示「本波只有這個變更」）；前置卡（卡頭右側 SHALL 有狀態徽章——`blockedBy` 空時「可以開工」、非空時「等 N 項」且 title 列出前置名；內容為 dependsOn 每項一列，資料源 capability `setDepends` 為 true 時每列附移除鈕、底部附下拉選單列出候選變更供新增，選擇即寫入；空時「尚無前置」；`blockedBy` 含 `dependsOn` 以外的名稱（舊 server 以目錄級重疊判定阻擋）時 SHALL 另以一列唯讀列出這些名稱（tw「另需等待（server 依重疊判定）：」，不附移除鈕），讓徽章數字對得上卡片內容）；重疊卡（`requirementOverlap` 每項一列：對方變更名、`capability › requirement` 文字、雙方操作各一枚小標籤（用詞與配色同規格文件的 delta 區段，如 MODIFIED 顯示「修改」；title 為該操作所屬的變更名），`conflict` 為 true 時加「同名衝突」標籤；空時「無重疊」；SHALL NOT 顯示目錄級 `overlaps`）；封存順序卡（`archiveAfter` 每項變更名，下方一句「先封存它們，再對照正式規格重寫同名 requirement 後封存本變更」；其中有變更也把本變更列入它的 `archiveAfter`（互相等待）時 SHALL 另加一句警示（tw「與 {names} 互相等待：兩邊都要先等對方封存，須調整拆分」）；`requirementOverlap` 有 `conflict` 為 true 的列時 SHALL 加一句同名衝突警示（tw「與 {names} 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」）；`archiveAfter` 為空且無同名衝突時「可直接封存」）。排程文件 SHALL NOT 渲染獨立的「阻擋」段。新增候選 SHALL 為該變更所在名冊中的其他作用中變更、扣除已宣告的前置：資料源提供名冊查詢時（local）SHALL 以查詢結果為準——該變更有 worktree 映射時名冊為其 worktree 副本、無映射時為主 checkout，與前置寫入作用的 store 相同；名冊查詢未完成時 SHALL NOT 渲染新增下拉；資料源未提供名冊查詢（remote，scope 只有一份名冊）或首次查詢（開啟詳情頁、切換變更）失敗時 SHALL 自作用中變更清單派生；看板刷新後的重新查詢失敗時 SHALL 沿用前一次的查詢結果。新增或移除前置 SHALL 經資料源 `setDepends` 寫回該變更 meta 的 depends_on，成功後看板與排程文件刷新；引擎拒絕（自依賴、不存在、已封存、成環）時 SHALL 以單行錯誤呈現引擎訊息、畫面不變。capability 為 false 時 SHALL NOT 渲染移除鈕與下拉選單。清單項缺 `wave` 且缺 `dependsOn`（remote 清單未取得 plan：舊 server、依賴成環或請求失敗）時閱讀卡 SHALL 只顯示一句說明（tw「此模式尚未提供排程資訊」），不渲染四卡；缺 `wave` 但帶 `dependsOn`（local plan 成環）時 SHALL 顯示一句說明（tw「依賴成環，先移除一條前置才有排程資訊」）並只渲染前置卡（無狀態徽章；`dependsOn` 為空時照樣顯示「尚無前置」；capability 為 true 時附移除鈕、SHALL NOT 渲染新增下拉），讓使用者解環。`requirementOverlap` 與 `archiveAfter` 缺席但 `wave` 存在（舊 server）時重疊卡與封存順序卡 SHALL 顯示各自的空態文案。判定 SHALL 收斂於 packages/ui 階段派生模組的 planWave／planBlockedBy／planRequirementOverlap／planArchiveAfter 單一入口。

清單項帶 `wave` 時閱讀卡 SHALL 另有：籤槽為前置卡同一枚狀態徽章（「可以開工」或「等 N 項」）；數字磚依序「波次」（N）、「宣告的前置」（`dependsOn` 筆數）、「同名需求重疊」（`requirementOverlap` 筆數）、「封存前要等」（`archiveAfter` 筆數）；一個提示框，標題由兩個子句組成——`blockedBy` 空為「可以開工」、非空為「等 N 項前置」；`archiveAfter` 空且無同名衝突為「可直接封存」、`archiveAfter` 非空為「封存前要等 M 個變更」、有同名衝突為「有同名衝突要先改」——兩個子句皆為綠態時提示框為 tip、否則為 warning；內文列出前置名、要先封存的變更名與同名衝突的對方變更名，並說明排程於看板刷新時重算。缺 `wave` 的兩種情況 SHALL NOT 渲染籤槽、數字磚與提示框。

#### Scenario: 排程分頁四段

- **WHEN** 開啟 wave=2、dependsOn=["add-a"]、blockedBy=["add-a"]、requirementOverlap=[{change:"add-b",capability:"desktop-app",requirement:"看板與任務",ownOperation:"MODIFIED",otherOperation:"MODIFIED",conflict:false}]、archiveAfter=["add-b"] 的變更詳情頁並點左欄「排程」節點
- **THEN** 閱讀卡依序顯示波次卡「第 2 波」、前置卡（徽章「等 1 項」、前置列 add-a 附移除鈕、新增下拉）、重疊卡一列（add-b、`desktop-app › 看板與任務`、兩枚「修改」標籤）、封存順序卡（add-b 與提示句），頁面無 data-plan-section="blocked"；四卡頭為 16px 粗體且進本頁目錄

#### Scenario: 新增前置寫回

- **WHEN** 於排程文件的下拉選單選擇 add-c
- **THEN** 該變更 .openspec.yaml 的 depends_on 含 add-c，前置卡出現 add-c，看板順序刷新

#### Scenario: 引擎拒絕新增

- **WHEN** 於排程文件選擇會成環的變更
- **THEN** 顯示引擎的成環單行錯誤，前置卡不變，meta 檔不變

#### Scenario: 成環時可移除前置

- **WHEN** 清單 planError 非 null，開啟 dependsOn=["add-a"]（無 wave）的變更詳情頁並點「排程」節點
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置卡的 add-a 列（附移除鈕），無狀態徽章、無新增下拉、無波次／重疊／封存順序卡；按移除鈕即寫回

#### Scenario: remote 分頁唯讀

- **WHEN** 於 capability setDepends 為 false 的資料源開啟詳情頁的排程文件
- **THEN** 不出現移除鈕與下拉選單，四卡照常顯示

#### Scenario: worktree 映射時候選來自副本名冊

- **WHEN** 主 checkout 且 worktree 政策開啟，add-dark-mode 已映射到 worktree，add-auth 在分支前建立、add-late 在分支後才於主 checkout 建立，開啟 add-dark-mode 的排程文件
- **THEN** 新增下拉選單含 add-auth、不含 add-late，看板清單仍列出 add-late

#### Scenario: 重新查詢名冊失敗沿用前次結果

- **WHEN** 排程文件已以名冊 add-a、add-b 呈現候選（作用中變更清單另有 add-d，add-a 已是前置），看板刷新後名冊查詢失敗
- **THEN** 新增下拉的候選仍只有 add-b，不出現 add-d

#### Scenario: 可開工徽章與同名衝突標籤

- **WHEN** 開啟 wave=1、blockedBy=[]、requirementOverlap=[{change:"add-x",capability:"auth",requirement:"登入",ownOperation:"ADDED",otherOperation:"ADDED",conflict:true}]、archiveAfter=[] 的變更詳情頁並點「排程」節點
- **THEN** 前置卡徽章為「可以開工」，重疊卡的 add-x 列帶「同名衝突」標籤與兩枚「新增」標籤，封存順序卡顯示「與 add-x 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」且不顯示「可直接封存」

#### Scenario: 舊 server 缺新欄位的空態

- **WHEN** remote 清單項帶 wave 與 blockedBy 但無 requirementOverlap 與 archiveAfter
- **THEN** 重疊卡顯示「無重疊」、封存順序卡顯示「可直接封存」，其餘兩卡照常

#### Scenario: 舊 server 的重疊阻擋列在前置卡

- **WHEN** remote 清單項 wave=2、dependsOn=[]、blockedBy=["add-a"]（舊 server 把先配置、共用 capability 的 add-a 算進阻擋），無 requirementOverlap 與 archiveAfter
- **THEN** 前置卡徽章「等 1 項」、內容「尚無前置」與一列「另需等待（server 依重疊判定）：add-a」，該列無移除鈕；本機與新 server 的 blockedBy 只含宣告前置，不出現這一列

#### Scenario: 互相等待時封存卡警示

- **WHEN** add-a 的 archiveAfter=["add-b"]、add-b 的 archiveAfter=["add-a"]，開啟 add-a 的排程文件
- **THEN** 封存順序卡列 add-b 與提示句，另顯示「與 add-b 互相等待：兩邊都要先等對方封存，須調整拆分」；單向的 archiveAfter 不出現這句警示

#### Scenario: 成環時環外變更的前置卡空態

- **WHEN** 清單 planError 非 null，開啟 dependsOn=[]（無 wave）的環外變更詳情頁並點「排程」節點
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置卡「尚無前置」，無狀態徽章、無新增下拉

#### Scenario: 排程的數字磚與提示框兩態

- **WHEN** 開啟排程分頁四段 scenario 的變更（wave=2、dependsOn 與 blockedBy 各一、重疊一列、archiveAfter 一個）並點「排程」節點；再開啟 wave=1、dependsOn=[]、blockedBy=[]、requirementOverlap=[]、archiveAfter=[] 的變更
- **THEN** 第一個變更的籤槽為「等 1 項」、數字磚「波次 2」「宣告的前置 1」「同名需求重疊 1」「封存前要等 1」、warning 提示框標題「等 1 項前置，封存前要等 1 個變更」且內文列出 add-a 與 add-b；第二個變更的籤槽為「可以開工」、數字磚「波次 1」「宣告的前置 0」「同名需求重疊 0」「封存前要等 0」、tip 提示框標題「可以開工，也可直接封存」

##### Example: 提示框標題的兩個子句

| blockedBy | archiveAfter | 同名衝突 | 標題 | 型 |
| --------- | ------------ | -------- | ---- | -- |
| [] | [] | 無 | 可以開工，也可直接封存 | tip |
| [add-a] | [] | 無 | 等 1 項前置，可直接封存 | warning |
| [] | [add-b] | 無 | 可以開工，封存前要等 1 個變更 | warning |
| [add-a] | [add-b] | 無 | 等 1 項前置，封存前要等 1 個變更 | warning |
| [] | [] | add-x | 可以開工，有同名衝突要先改 | warning |

#### Scenario: 缺 wave 時無數字磚與提示框

- **WHEN** 開啟 remote 清單項缺 wave 且缺 dependsOn 的變更詳情頁並點「排程」節點
- **THEN** 閱讀卡只顯示「此模式尚未提供排程資訊」，籤槽、數字磚與提示框皆缺席


<!-- @trace
source: desktop-change-doc-views
updated: 2026-10-12T16:18:33+08:00
-->

---
### Requirement: 詳情頁的工單分頁

change 詳情頁 SHALL 於左欄「文件」節點「排程」之後提供兩個條件式節點：清單項 `reviewStatus` 為 `inReview` 時 SHALL 出現「審查」節點（tw「審查」、en「Review」），`verifyStatus` 為 `inVerify` 時 SHALL 出現「驗證」節點（tw「驗證」、en「Verify」），審查節點在前；兩者皆非時 SHALL NOT 渲染這兩個節點，文件節點維持提案／設計／任務／規格／排程。工單文件的閱讀卡 SHALL 為該站工單的結構化呈現：籤槽為「第 N 輪」（N 為末輪序號）與末輪階段詞籤（`phase` 為 `discovery` 顯示 tw「首輪」en「First pass」、`validation` 顯示 tw「複驗」en「Re-check」、`null` 不出階段詞籤）；數字磚依序為「輪」（輪數）、「CRITICAL（末輪）」、「WARNING（末輪）」、「SUGGESTION（末輪）」（末輪三級各自的條數，標籤維持英文）；末輪 CRITICAL 與 WARNING 皆為 0 時 SHALL 有 tip 提示框「可以蓋章」，內文為「末輪只剩 N 條 SUGGESTION，不擋章」（N 為 0 時「末輪無發現」）並點名以 CLI 或 agent 跑該站的 stamp 指令蓋章；否則提示框缺席；內文 SHALL NOT 另畫段標題列。輪次依序號升序逐輪一張卡（與排程文件同一種邊框卡片），末輪預設展開、其餘預設收合；卡頭為 16px 粗體的「第 N 輪」帶本頁目錄錨點、階段詞、「範圍 N 檔」、findings 條數與各嚴重度的計數籤（為零的不出）；展開後卡內先列「套用規則」列（僅該輪 `rules` 非 null 時，見下段），再列範圍路徑（等寬，原樣顯示、不做平台轉換），再列每條 finding：嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、軸向籤（描述以「Correctness」「Standards」「Coherence」加冒號開頭時取出該詞成籤，描述去掉前綴；無前綴時無籤、描述原樣）、路徑（等寬字）與描述原文（不翻譯、不截斷）；描述以結構 token `(accepted)` 結尾時 SHALL 去掉 token 並附「已接受」籤，token 出現於行中時 SHALL 原樣顯示；零 findings 的輪 SHALL 顯示「本輪無發現」。工單 SHALL 於節點出現時載入，並隨外部檔案變更重新載入；載入三態依「詳情頁文件載入以 skeleton 呈現」；載入完成而無工單（不存在、遠端 404、讀取或解析失敗）時 SHALL 顯示空態「工單尚未抵達或已被刪除」而非錯誤，籤槽、數字磚與提示框缺席。工單資料 SHALL 為 `{ rounds: [{ index, phase, patchHash, rules, scope, findings: [{ severity, path, text }] }], reviewBaselineSmells: string[] }`：`rounds` 的欄位名與型別與 CLI `speclink review show --json` 的 `rounds` 相同（`rules` 為 `{ baseline, critical, warning, suggestion }` 或 null）；`reviewBaselineSmells` 為 desktop 內建引擎提供的預設 smell 名稱（依審查技能範本順序），兩站的工單資料都帶此欄位；本機（含 worktree 覆蓋層）與遠端、活變更與已封存 SHALL 回同一形狀。遠端活變更 SHALL 以 `GET /changes/{name}/review`／`verify` 取得結構化輪次，SHALL NOT 放寬 `artifact cat` 的 artifact 名白名單。看板刷新後對應狀態翻為非進行中時，該節點 SHALL 消失；使用者正停在該文件時 SHALL 切回「提案」。工單文件 SHALL 為唯讀，SHALL NOT 提供蓋章、放棄或追加輪的操作。

「套用規則」列：輪次 `rules` 非 null 時，該輪卡展開後 SHALL 先渲染此列（tw「套用規則」、en「Rules applied」），再列範圍路徑；`rules` 為 null（舊工單、驗證站工單、複驗輪）時 SHALL NOT 渲染此列。列頭 SHALL 為帶 `aria-expanded` 的展開切換按鈕，與輪卡卡頭同一種 chevron 樣式，預設收合。收合時列頭依 baseline 與專案規則條數呈現模式文字與條數籤：條數籤以嚴重度色章呈現「CRITICAL 2」形式、條數為零的嚴重度不出；沒有專案規則時 SHALL NOT 顯示任何條數籤；baseline 為 replace 且三個清單皆空時，列 SHALL 以琥珀底（WARNING 語意色）呈現。展開後 SHALL 依序列出：baseline 為 extend 時，SUGGESTION 色章、「預設 N 條 code smell」與 N 個 smell 名稱（等寬字，取自 `reviewBaselineSmells`）；baseline 為 replace 時，一行「沒有套用預設 N 條 code smell」；接著每個非空的嚴重度一組——嚴重度色章、「專案規則 N 條」與規則原文清單（不翻譯、不截斷）；末尾一行註記「記錄首輪當時的設定；之後改了審查規則，這裡不會跟著變。」N 為 `reviewBaselineSmells` 的長度，SHALL NOT 寫死。smell 名稱清單 SHALL 與設定頁審查規則頁的名稱清單為同一個共用元件。

#### Scenario: 審查工單分頁出現並結構化呈現

- **WHEN** 開啟 `reviewStatus: "inReview"`、`verifyStatus: "none"` 的變更詳情頁，工單含 Round 1（`phase: "discovery"`，scope 3 檔，findings 兩條 WARNING）與 Round 2（`phase: "validation"`，scope 3 檔，findings 一條 WARNING 描述結尾 `(accepted)`、一條 SUGGESTION）
- **THEN** 左欄文件節點為提案／設計／任務／規格／排程／審查，無「驗證」節點；點「審查」後閱讀卡標題「審查」、籤槽「第 2 輪」「複驗」，數字磚「輪 2」「CRITICAL（末輪）0」「WARNING（末輪）1」「SUGGESTION（末輪）1」，無「可以蓋章」提示框；第 2 輪卡展開列兩條 finding，WARNING 那條附「已接受」籤且描述不含 `(accepted)`；第 1 輪卡收合、卡頭顯示「第 1 輪 · 首輪 · 範圍 3 檔 · 2 條」與「WARNING 2」計數籤

##### Example: 階段詞與 accepted token

| round.phase | 顯示階段詞（tw） | finding.text | 顯示描述 | 已接受籤 |
| ----------- | ---------------- | ------------ | -------- | -------- |
| discovery | 首輪 | Correctness: 未處理空清單 | 未處理空清單（軸向籤 Correctness） | 無 |
| validation | 複驗 | Standards: 命名不一致 (accepted) | 命名不一致（軸向籤 Standards） | 有 |
| null | （不顯示） | Standards: (accepted) 語意不清 | (accepted) 語意不清（軸向籤 Standards） | 無 |

#### Scenario: 兩站皆進行中出現兩個分頁

- **WHEN** 開啟 `reviewStatus: "inReview"` 且 `verifyStatus: "inVerify"` 的變更詳情頁
- **THEN** 左欄文件節點於「排程」後依序為「審查」「驗證」，各自載入對應站的工單，兩份文件內容互不混入

#### Scenario: 無工單時分頁缺席

- **WHEN** 開啟 `reviewStatus: "reviewed"`、`verifyStatus: "none"` 的變更詳情頁
- **THEN** 文件節點維持提案／設計／任務／規格／排程五個，無「審查」與「驗證」節點，且未發出工單讀取請求

#### Scenario: 工單被刪時分頁退場

- **WHEN** 使用者停在 `inReview` 變更的「審查」文件，其後工單被蓋章刪除、看板刷新後該變更 `reviewStatus` 翻為 `reviewed`
- **THEN** 「審查」節點自左欄消失，詳情頁切回「提案」文件，「狀態」列「品質關卡」列的章籤改為「已審查」

#### Scenario: 遠端工單 404 顯示空態

- **WHEN** 於 remote 資料源開啟 `inReview` 變更詳情頁並點「審查」節點，server 對 `GET /changes/{name}/review` 回 404
- **THEN** 節點仍在，skeleton 消失後顯示「工單尚未抵達或已被刪除」，無錯誤彈窗，籤槽與數字磚缺席

#### Scenario: 工單追加輪後分頁更新

- **WHEN** 「審查」文件開啟中，外部以 `speclink review add-round` 追加 Round 3
- **THEN** 檔案變更觸發重新載入，籤槽改為「第 3 輪」，數字磚「輪 3」，第 3 輪卡展開、第 2 輪與第 1 輪收合

#### Scenario: 可蓋章提示框與軸向籤

- **WHEN** 開啟 `inReview` 變更的「審查」文件，末輪為 Round 2（`validation`）只有一條 SUGGESTION，描述為「Standards: possible Duplicated Code。讀寫加 try/catch 的寫法…」
- **THEN** tip 提示框標題「可以蓋章」、內文「末輪只剩 1 條 SUGGESTION，不擋章」並點名 `speclink review stamp`；該條 finding 顯示 SUGGESTION 色章、「Standards」軸向籤、等寬路徑與去掉前綴的描述；末輪 CRITICAL 或 WARNING 任一非零時提示框缺席

#### Scenario: 沒有專案規則時套用規則列只寫預設清單

- **WHEN** 開啟 `inReview` 變更的「審查」文件，Round 1（`discovery`）的 `rules` 為 `{ baseline: "extend", critical: [], warning: [], suggestion: [] }`，`reviewBaselineSmells` 有 12 個名稱，展開第 1 輪卡
- **THEN** 範圍路徑之前有一列收合的「套用規則」，列頭文字為「預設 12 條 code smell」，沒有任何條數籤；點列頭展開後顯示 SUGGESTION 色章、「預設 12 條 code smell」、12 個 smell 名稱與末尾註記，沒有任何「專案規則」組

#### Scenario: 套用規則列的四種呈現

- **WHEN** 開啟 `inReview` 變更的「審查」文件，Round 1 的 `rules` 分別為下表四種，`reviewBaselineSmells` 有 12 個名稱
- **THEN** 收合時列頭文字、條數籤與底色如下表；展開後 extend 列出 12 個名稱、replace 寫「沒有套用預設 12 條 code smell」，再列非空嚴重度的規則原文組與末尾註記

##### Example: 列頭呈現

| rules | 列頭文字（tw） | 列頭文字（en） | 條數籤 | 底色 |
| ----- | -------------- | -------------- | ------ | ---- |
| extend，三清單皆空 | 預設 12 條 code smell | 12 baseline code smells | 無 | 一般 |
| extend，critical 2、warning 1、suggestion 0 | 預設 12 條＋專案規則 | 12 baseline smells + project rules | CRITICAL 2、WARNING 1 | 一般 |
| replace，suggestion 1 | 只用專案規則（沒有套用預設 12 條） | Project rules only (baseline not applied) | SUGGESTION 1 | 一般 |
| replace，三清單皆空 | 整份換掉且沒有專案規則：只依 repo 文件審查 | Baseline replaced, no project rules: judged against repo documents only | 無 | 琥珀 |

#### Scenario: 展開後列出專案規則原文

- **WHEN** Round 1 的 `rules` 為 `{ baseline: "extend", critical: ["所有 SQL 一律用參數化查詢，不得用字串拼接", "不得把金鑰或密碼寫死在程式碼裡"], warning: ["新增的公開 API 要有對應的測試"], suggestion: [] }`，使用者展開「套用規則」列
- **THEN** 依序顯示 12 個 smell 名稱、CRITICAL 色章與「專案規則 2 條」及兩條原文、WARNING 色章與「專案規則 1 條」及一條原文，沒有 SUGGESTION 組，末尾為「記錄首輪當時的設定；之後改了審查規則，這裡不會跟著變。」

#### Scenario: 沒有規則記錄時不顯示套用規則列

- **WHEN** 開啟「審查」文件，工單所有輪的 `rules` 皆為 null（審查技能升版前寫的工單）；或開啟「驗證」文件
- **THEN** 任何輪卡展開後都直接從範圍路徑開始，沒有「套用規則」列


<!-- @trace
source: review-ticket-applied-rules
updated: 2026-10-15T09:31:43+08:00
-->

---
### Requirement: 詳情頁的導覽與返回

詳情頁開啟時主區頂列的麵包屑 SHALL 為「專案名 / 頁名 / 項目名」三段：專案名與頁名 SHALL 可點、點擊 SHALL 關閉詳情頁回宿主頁；項目名 SHALL 以等寬字型呈現且不可點。麵包屑左側 SHALL 有返回鈕（「←」圖示，無障礙名稱為「返回<頁名>」，提示另標示 Esc 與返回快捷鍵），點擊 SHALL 關閉詳情頁回宿主頁。麵包屑右端 SHALL 呈現 ‹ n／m ›：m 為宿主清單目前呈現順序的項目數、n 為目前項目的序位，順序 SHALL 為宿主清單的呈現順序含搜尋與篩選——看板的變更依欄序（提案中、進行中、已就緒）再依欄內順序，看板的討論為上區全卡依序、其後接欄底「已轉出」細列，規格頁的規格、已封存頁的封存變更與封存討論各為該清單過濾後的全部頁順序；宿主不是該項目所屬的清單頁時（手冊頁開啟的規格詳情頁、規格詳情頁溯源列開啟的已封存詳情頁），序列為該清單未過濾的全部項目（規格依規格頁的預設排序）；到頭時對應鈕 SHALL 停用，SHALL NOT 循環；目前項目不在序列中時整組 SHALL 缺席。快捷鍵 ⌥↑／⌥↓（非 macOS 為 Alt+↑／Alt+↓）SHALL 等效於 ‹ ›，Esc 與 ⌘[、⌘←（非 macOS 為 Alt+←；以實體按鍵判定，不受輸入法影響）SHALL 等效於返回鈕；這些快捷鍵於焦點在文字輸入（input、textarea、contenteditable）時 SHALL NOT 作用，且 SHALL NOT 攔截已被浮層（選單、對話框、提示）處理掉的按鍵。滑鼠側鍵「上一頁」SHALL 等效於返回鈕。macOS 上，觸控板兩指往返回方向（手指向右）的水平滑動 SHALL 等效於返回鈕；非 macOS 平台 SHALL NOT 提供觸控板滑動返回——觸控板水平滑動 SHALL NOT 顯示指示、SHALL NOT 關頁，返回由返回鈕、返回快捷鍵與滑鼠側鍵承擔。只有帶手勢階段（手指按下、移動、放開）的輸入算滑動手勢：觸控板與 Magic Mouse 適用，一般滑鼠滾輪的水平捲動（含按住 Shift 的滾輪捲動）SHALL NOT 觸發；macOS 系統設定「在頁面之間滑動」不含兩指滑動（關閉或設為只用三指）時 SHALL NOT 觸發。滑動期間主區左緣 SHALL 顯示隨滑動距離推出的「←」返回指示，達門檻時指示轉為實心；結算時機為手指離開觸控板的當下——已達門檻 SHALL 立即關閉詳情頁回宿主頁，SHALL NOT 等待慣性捲動結束；放開前反向滑回門檻以下時指示 SHALL 收回且不關頁；手指離開後的慣性捲動 SHALL NOT 推進指示、SHALL NOT 開始新的一次滑動；一次手勢中累計的垂直移動超過 10px 且多於累計的水平移動時，整次手勢 SHALL 取消（指示收回、放開也不關頁），直到手指離開後的下一次手勢才重新判定；滑動期間整個畫面 SHALL NOT 隨手指水平平移（桌面關閉根層的水平彈性回彈）；手勢開始於仍可沿滑動方向捲動的水平捲動區塊（如寬表格、程式碼區塊；自然捲動關閉時手指向右是把區塊往右捲）時 SHALL 交給該區塊捲動、SHALL NOT 觸發返回；只裁切不捲動的區塊（如截斷的長名稱）不算捲動區塊。切換項目 SHALL 經與開啟相同的狀態層動作（互斥與浮層收掉規則同樣適用），切換後頁 SHALL 自頂端呈現；封存、刪除、退回或討論封存的確認框開著或確認在途時，切換與返回（含返回快捷鍵、滑鼠側鍵與觸控板滑動）SHALL NOT 作用——確認框屬於目前詳情頁，SHALL NOT 被連帶收掉。詳情頁開啟時經專案欄、應用程式選單、系統匣或連線失效橫幅的「重新登入」切換主區頁面 SHALL 先關閉詳情頁再呈現該頁。

#### Scenario: 麵包屑三段與返回

- **WHEN** 使用者於專案 speclink 的看板點開變更 add-auth 的詳情頁，再點麵包屑的「變更」
- **THEN** 開頁時麵包屑為「speclink / 變更 / add-auth」且 add-auth 為等寬不可點；點「變更」後詳情頁關閉、看板可見

#### Scenario: ‹ › 依看板順序含過濾

- **WHEN** 看板搜尋後提案中欄剩 a、b，進行中欄剩 c，使用者開啟 b 的詳情頁
- **THEN** 麵包屑右端顯示「2 / 3」；按 › 切到 c、顯示「3 / 3」且 › 停用；按 ‹ 兩次回到 a 且 ‹ 停用

#### Scenario: 快捷鍵與焦點在輸入時不作用

- **WHEN** 使用者於變更詳情頁按 ⌥↓、再把焦點放進排程分頁的下拉輸入後按 ⌥↓ 與 Esc
- **THEN** 第一次 ⌥↓ 切到下一個變更；焦點在輸入時 ⌥↓ 與 Esc 皆不切換、不關頁

#### Scenario: Esc 先給浮層

- **WHEN** 使用者於詳情頁開啟「⋯」選單後按 Esc，再按一次 Esc
- **THEN** 第一次 Esc 只關閉選單、詳情頁維持；第二次 Esc 關閉詳情頁

#### Scenario: 切換主區頁面關閉詳情頁

- **WHEN** 使用者停在變更 add-auth 的詳情頁，點專案欄的「規格」
- **THEN** 詳情頁關閉、主區為規格頁，麵包屑為「專案名 / 規格」

#### Scenario: 目前項目被外部刪除

- **WHEN** 使用者停在變更 x 的詳情頁，外部以 CLI 刪除 x 後看板刷新
- **THEN** 詳情頁關閉、看板可見且不含 x；過程無錯誤彈窗

#### Scenario: 返回鈕、返回快捷鍵與滑鼠側鍵回宿主頁

- **WHEN** 使用者於規格頁搜尋「desk」後開啟 desktop-app 的規格詳情頁，點麵包屑左側的返回鈕；再開啟同一頁後按 ⌘[（非 macOS 為 Alt+←）；再開啟後按滑鼠側鍵「上一頁」
- **THEN** 返回鈕的名稱為「返回規格」；三次皆關閉詳情頁回到規格頁，搜尋字串仍為「desk」

#### Scenario: 觸控板滑動返回

- **WHEN** 使用者於 macOS 的規格詳情頁以觸控板兩指向右滑動，滑動距離先未達門檻即放開，再滑一次超過門檻後放開
- **THEN** 第一次左緣出現「←」指示後收回、詳情頁維持；第二次指示轉為實心，手指放開的當下詳情頁關閉、回到規格頁

#### Scenario: 確認框開著時返回與切換不作用

- **WHEN** 使用者於變更詳情頁點「封存」叫出確認框，尚未按確認時按 ⌘[（非 macOS 為 Alt+←）、滑鼠側鍵「上一頁」、於 macOS 以觸控板兩指向右滑過門檻後放開、再按 ⌥↓
- **THEN** 確認框維持開著、詳情頁仍為同一個變更，左緣「←」指示未出現

#### Scenario: 手冊頁開啟的規格詳情頁序列為全部規格

- **WHEN** 專案有三份正式規格，使用者於手冊頁經出處開啟其中一份的規格詳情頁
- **THEN** 麵包屑右端的 m 為 3，‹ › 依規格頁的預設排序切換，宿主維持手冊頁

#### Scenario: 快甩放開當下即返回、慣性不延後

- **WHEN** 使用者於 macOS 的變更詳情頁以觸控板兩指快速向右甩過門檻後放開，手指離開後慣性捲動仍持續送出捲動約一秒
- **THEN** 手指放開的當下詳情頁關閉、看板可見；其後的慣性捲動不在看板推出「←」指示，也不觸發任何返回

##### Example: 放開時的結算（門檻 120px）

| 放開前的手勢 | 放開時的結果 | 說明 |
| --- | --- | --- |
| 向右滑 200px 後放開 | 立即關閉詳情頁 | 已達門檻 |
| 向右滑 80px 後放開 | 指示收回、詳情頁維持 | 未達門檻 |
| 向右滑 150px、未放開再往回滑到 80px 後放開 | 指示收回、詳情頁維持 | 放開前滑回門檻以下 |
| 向右滑 200px 後放開，慣性又捲了 300px | 放開當下關頁，慣性不影響 | 慣性不推進指示 |

#### Scenario: 一般滑鼠滾輪與系統設定關閉時不觸發

- **WHEN** 使用者於 macOS 的規格詳情頁按住 Shift 以一般滑鼠滾輪往返回方向水平捲動超過門檻；接著把系統設定「在頁面之間滑動」改為「以三指滑動」，再以觸控板兩指向右滑過門檻後放開
- **THEN** 兩次左緣皆不出現「←」指示，詳情頁維持；把設定改回「以兩指向左或向右捲動」後同樣的觸控板滑動即返回規格頁

#### Scenario: 垂直為主的手勢整次取消

- **WHEN** 使用者於 macOS 的變更詳情頁以觸控板兩指先向下捲動 40px，手指不放開接著向右滑 200px 後放開；再以觸控板兩指單純向右滑過門檻後放開
- **THEN** 第一次頁面照常向下捲動，左緣不出現「←」指示、詳情頁維持；第二次指示轉為實心，手指放開的當下詳情頁關閉

#### Scenario: 非 macOS 不提供觸控板滑動返回

- **WHEN** 使用者於 Windows 的規格詳情頁以觸控板兩指向右滑動超過門檻後放開，接著按 Alt+←
- **THEN** 滑動期間左緣不出現「←」指示、放開後詳情頁維持；按 Alt+← 後詳情頁關閉、回到規格頁


<!-- @trace
source: desktop-swipe-back-finger-lift
updated: 2026-10-14T14:34:02+08:00
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
- **THEN** 選單首列為 `openspec/manual/tray.md`，其後三項為以預設 App 開啟、在 Finder 顯示（macOS）、複製路徑；剪貼簿寫入 `/work/app/openspec/manual/tray.md`；規格詳情頁的「溯源」文件無選單；remote 模式的手冊頁顯示既有空狀態，無閱讀卡與選單


<!-- @trace
source: desktop-detail-doc-tree
updated: 2026-10-12T13:39:16+08:00
-->


<!-- @trace
source: desktop-spec-manual-pages
updated: 2026-10-12T22:59:10+08:00
-->

---
### Requirement: 任務列整列可點

變更詳情頁任務分頁的每個任務列 SHALL 整列為其勾選框的 label：點擊列上任一處（勾選框、編號、文字、空白）SHALL 切換該任務的完成狀態並依既有語意寫回；拖曳把手 SHALL NOT 觸發切換；拖選列上文字後放開 SHALL NOT 觸發切換（直接點勾選框不受此限）；雙擊選字時兩下點擊 SHALL 互相抵銷、勾選狀態不變。唯讀檢視（已封存）與批次操作或拖排寫回進行中的列 SHALL NOT 回應點擊；單一任務的勾選寫回不鎖列（勾選後立即取消不遺失）。勾選框 SHALL 保留 checkbox 角色、「任務 N」標籤與空白鍵切換；拖曳仍 SHALL 僅由把手啟動，拖曳進行中按 Esc SHALL 只取消拖曳、SHALL NOT 關閉詳情頁。有寫入能力時工具列 SHALL 附一行灰字提示「點整列即可勾選；拖曳把手改順序」；無拖排能力（不顯示把手）時提示 SHALL 為「點整列即可勾選」。

#### Scenario: 點任務文字切換

- **WHEN** 使用者點某未完成任務列的文字部分
- **THEN** 該任務切換為完成並寫回 tasks.md，效果與點勾選框相同

#### Scenario: 點把手不切換

- **WHEN** 使用者點某任務列的拖曳把手
- **THEN** 勾選狀態不變

#### Scenario: 選取任務文字不切換

- **WHEN** 使用者在某任務列的文字上拖選一段文字後放開
- **THEN** 勾選狀態不變、不發出寫回，選取的文字可複製

#### Scenario: 唯讀與批次寫回中不回應

- **WHEN** 使用者於已封存變更的任務分頁點任務列，或於「全部已完成」等批次操作、拖排的寫回尚未完成時點任務列
- **THEN** 勾選狀態不變、不發出寫回

<!-- @trace
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 已封存項目以詳情頁檢視

已封存頁的變更列與討論列 SHALL 以列式清單呈現，點擊列 SHALL 開啟唯讀的已封存詳情頁（宿主維持已封存頁），SHALL NOT 提供行內展開。已封存詳情頁 SHALL 與變更／討論詳情頁同骨架（「詳情頁的文件樹、閱讀卡與本頁目錄」：左欄身分區、文件節點、「狀態」列，中欄閱讀卡，右欄本頁目錄）且唯讀——SHALL NOT 提供任務勾選、動詞執行或任何寫入操作；身分區 SHALL NOT 呈現進度條，標題列 SHALL NOT 有動作區——封存是唯讀定格；開啟期間內容 SHALL 隨外部檔案變更反映；所請求的文件不存在時對應閱讀卡 SHALL 顯示空狀態而非錯誤。

封存變更的詳情頁 SHALL 至少含提案、設計、任務、規格文件節點（內容來自封存目錄的實體文件），閱讀卡標題列右端提供「開啟 ▾」（對應 `openspec/changes/archive/<封存目錄名>/` 下的檔案）；任務文件的勾選框不可點擊、無拖曳把手、無批次工具列、整列不回應點擊。身分區 SHALL 為變更名（等寬）緊跟複製鈕（複製含日期前綴的封存目錄名，複製後 SHALL 有已複製回饋）與「已封存」籤，審查／驗證結局籤 SHALL NOT 出現在身分區——結局只由「狀態」列的品質關卡結局承載；出身 SHALL 一行依序呈現建立者（首字母圓標＋名字，完整識別收 hover 提示）、建立日期、「封存 YYYY-MM-DD」，其下「來自」列呈現來源討論籤（有來源討論時；點擊 SHALL 開啟該討論的已封存詳情頁，記錄以 live 優先、封存後備載入），出身行恆定單行、溢出裁切，任一欄位資料缺席時該欄位 SHALL 缺席、其餘照常。「狀態」列 SHALL 為三列：任務 n/m（無 tasks.md 時「—」）、品質關卡結局（皆無時「未經品質關卡」）、封存日期。

封存討論的詳情頁身分區 SHALL 以 slug 為題（等寬）緊跟複製 slug 鈕與「已封存」籤（kind 為 improve 時另帶改進籤），topic 為其下最多三行；出身為建立者、建立日期與「封存 YYYY-MM-DD」，任一欄位資料缺席時該欄位 SHALL 缺席、其餘照常（清單資料不帶封存討論的封存日期時，「封存」欄缺席）。「狀態」列 SHALL 含階段列（步驟列與討論詳情頁相同、現站為「已封存」）；promoted_to 非空時 SHALL 另有「衍生變更」列：每顆籤依子變更所在呈現三態——在封存清單中者為可點籤、副標為封存日期、點擊開啟該封存變更的已封存詳情頁；在活躍變更清單中者為可點籤、副標為其看板階段詞、點擊開啟該變更的詳情頁（宿主依「變更與討論詳情頁的宿主為看板」切回看板）；兩者皆無者為不可點籤、副標「無封存記錄」；順序依轉出順序；無衍生變更時該列缺席，封存變更的詳情頁 SHALL NOT 顯示該列。文件節點 SHALL 為「結論」「討論過程」「背景」「衍生變更」；「開啟 ▾」SHALL 只有「複製內容」。

#### Scenario: 點擊封存變更列開啟四分頁詳情頁

- **WHEN** 使用者於已封存頁點擊一個含完整 artifacts 的封存變更列
- **THEN** 主區換成唯讀的已封存詳情頁，左欄文件節點為提案／設計／任務／規格，各文件的閱讀卡呈現封存目錄內對應文件的內容，任務文件的勾選框不可點擊、無把手與工具列；宿主維持已封存頁，清單無行內展開

#### Scenario: 點擊封存討論列開啟唯讀詳情頁

- **WHEN** 使用者於已封存頁討論節點擊一筆封存討論列
- **THEN** 主區換成該討論的已封存詳情頁：身分區有「已封存」籤、「狀態」列的階段現站為「已封存」、文件節點為結論／討論過程／背景／衍生變更，無轉為變更、封存或任何寫入按鈕

#### Scenario: 缺件文件顯示空狀態

- **WHEN** 使用者開啟一個無 design.md 的封存變更詳情頁並點「設計」節點
- **THEN** 閱讀卡顯示空狀態文字，無錯誤彈窗，其餘文件照常可用

#### Scenario: 自封存變更詳情頁跳轉來源討論

- **WHEN** 使用者開啟一個帶來源討論的封存變更詳情頁，點擊身分區「來自」列的來源討論籤
- **THEN** 該討論的已封存詳情頁取代顯示（記錄以 live 優先、封存後備載入），宿主維持已封存頁；無來源討論的封存變更詳情頁不顯示此列

#### Scenario: 封存變更詳情頁的標題列、出身列與狀態卡

- **WHEN** 使用者開啟一筆建立者與建立日期俱在、20/21 任務、reviewStatus 為 reviewed 的封存變更詳情頁並點擊身分區變更名旁的複製鈕
- **THEN** 剪貼簿寫入含日期前綴的封存目錄名並顯示已複製回饋；身分區只帶「已封存」籤、無「已審查」籤；出身行顯示建立者首字母圓標＋名字、建立日期與「封存 YYYY-MM-DD」；「狀態」列三列為「20/21」、「已審查」籤、封存日期；無進度條，標題列無動作區

#### Scenario: 出身列欄位缺席時其餘照常

- **WHEN** 使用者開啟一筆建立日期不可得、建立者可得的封存變更詳情頁
- **THEN** 出身行顯示建立者與封存日期，建立日期欄位缺席，無錯誤與空欄佔位

#### Scenario: 封存討論詳情頁列出衍生變更並跳轉封存變更

- **WHEN** 使用者開啟一筆 promoted_to 含兩個已封存子變更的封存討論詳情頁，點擊「狀態」列中「衍生變更」的第一顆籤
- **THEN** 列有兩顆依轉出順序的籤、副標各為封存日期；點擊後該子變更的已封存詳情頁取代顯示，宿主頁面維持已封存頁

#### Scenario: 衍生變更籤的三態

- **WHEN** 使用者開啟一筆 promoted_to 含三個子變更的封存討論詳情頁——一個已封存、一個仍在看板進行中、一個已刪除
- **THEN** 三顆籤依轉出順序呈現，各自的副標與可點性如下表；點擊活躍者開啟其詳情頁且宿主切回看板，點擊已刪除者無任何動作

##### Example: 三態對照

| 子變更所在 | 副標 | 可點 | 點擊結果 |
| --- | --- | --- | --- |
| 封存清單 | 封存日期（如 2026-09-02） | 是 | 開啟該封存變更的已封存詳情頁，宿主不切頁 |
| 活躍變更清單 | 看板階段詞（如「進行中」） | 是 | 開啟該變更的詳情頁，宿主落回看板 |
| 兩者皆無 | 無封存記錄 | 否 | 無 |

#### Scenario: 無衍生變更時衍生列缺席

- **WHEN** 使用者開啟一筆 promoted_to 為空的封存討論詳情頁
- **THEN** 「狀態」列只有階段列，無「衍生變更」列；封存變更詳情頁亦無該列

#### Scenario: 已封存討論的文件動作只有複製內容

- **WHEN** 使用者於封存討論詳情頁展開閱讀卡標題列的「開啟 ▾」
- **THEN** 選單只有「複製內容」，選後剪貼簿為該記錄原文


<!-- @trace
source: detail-quality-stamp-dedup
updated: 2026-10-13T23:23:36+08:00
-->

---
### Requirement: 已封存詳情頁的工單分頁

已封存變更詳情頁 SHALL 於左欄「文件」節點「規格」之後提供兩個條件式節點：封存清單項 `reviewStatus` 為 `reviewedNotPassed` 時 SHALL 出現「審查」節點，`verifyStatus` 為 `verifiedNotPassed` 時 SHALL 出現「驗證」節點，審查在前；其他狀態（`none`、`reviewed`、`verified`）SHALL NOT 渲染對應節點。文件內容、載入三態、空態文案與工單資料形狀 SHALL 與「詳情頁的工單分頁」相同，資料來自封存目錄內的 `review.md`／`verify.md`（本機讀封存目錄；遠端以 `GET /archived/{datedName}/artifacts/review.md` 或 `verify.md` 取原文後解析）。工單文件 SHALL 為唯讀。

#### Scenario: 曾審查未通過的已封存詳情頁顯示審查分頁

- **WHEN** 開啟 `reviewStatus: "reviewedNotPassed"`、`verifyStatus: "verified"` 的封存變更詳情頁
- **THEN** 左欄文件節點為提案／設計／任務／規格／審查，無「驗證」節點；「審查」文件以結構化方式呈現封存目錄內 review.md 的各輪，末輪展開

#### Scenario: 蓋章封存的詳情頁無工單分頁

- **WHEN** 開啟 `reviewStatus: "reviewed"`、`verifyStatus: "none"` 的封存變更詳情頁
- **THEN** 文件節點維持提案／設計／任務／規格四個，未發出工單讀取請求

#### Scenario: 遠端封存工單原文解析

- **WHEN** 於 remote 資料源開啟 `verifiedNotPassed` 的封存變更詳情頁並點「驗證」節點，server 對 `GET /archived/{datedName}/artifacts/verify.md` 回傳工單原文
- **THEN** 閱讀卡呈現與本機相同形狀的結構化輪次（`rounds[].index`、`phase`、`patchHash`、`scope`、`findings[].severity`／`path`／`text`）


<!-- @trace
source: desktop-detail-doc-tree
updated: 2026-10-12T13:39:16+08:00
-->

---
### Requirement: 關於頁

應用程式設定的分類欄 SHALL 於「應用程式」群組之後有群組「關於」，唯一一項「關於 Speclink」；主區麵包屑為「設定 / 關於 Speclink」。macOS 原生選單的「關於 Speclink」SHALL 直接開啟此頁。頁內由上而下 SHALL 為：

- **品牌卡**：橫式鎖版（高 40px，深色主題用深色版）；其下一行為灰字版本列「app 版號 · 引擎 引擎版號 · 平台 架構」、更新狀態籤與「複製版本資訊」鈕；再一句產品描述；SHALL NOT 另放裸標記。更新狀態籤（已是最新、有新版本、檢查中、無法檢查、尚未檢查）SHALL 與更新頁「最新版本」列為同一份狀態，且 SHALL 可點——點擊進入更新頁。「複製版本資訊」SHALL 把三行文字寫入剪貼簿——Speclink app 版號、Engine 引擎版號、Platform 平台與架構——並短暫顯示已複製；版本資訊尚未取得時版本列顯示「—」且不顯示複製鈕。
- **連結卡**三列整列可點並帶外開圖示：「原始碼」（說明為等寬倉庫網址）、「說明文件」、「回報問題」（開啟新 issue 頁並帶入版本三行）；皆以系統瀏覽器開啟，失敗以失敗 toast 呈現。
- **授權卡**兩列：「授權條款」（說明「MIT License · © 2026 MomoChen」；「檢視」開對話框顯示隨 app 打包的 LICENSE 全文）、「第三方授權」（「檢視」開對話框列出桌面 app 與 CLI 用到的開源套件：npm 與 cargo 兩節，每筆名稱、版本與授權識別字，依名稱排序；清單於建置時由腳本自相依清單產生，缺檔時顯示「清單尚未產生」）。

關於頁 SHALL NOT 含「檢查更新」與「更新日誌」的入口——兩者在應用程式設定中只在更新頁（見需求「桌面自動更新」「更新日誌彈窗」）。

版本資訊 SHALL 由桌面 core 一次回傳（app 版號、引擎版號、平台與架構），前端 SHALL NOT 自行拼湊；引擎版號 SHALL 與 CLI `--version` 所示的 engine 版號相同（ASSET_VERSION）。頁內文案 SHALL 跟隨 UI 語言。

#### Scenario: 選單關於直達頁

- **WHEN** macOS 使用者點原生選單「Speclink › 關於 Speclink」
- **THEN** 主區為關於頁、麵包屑「設定 / 關於 Speclink」、分類欄「關於 Speclink」為作用中；品牌卡顯示鎖版、版本列與產品描述

#### Scenario: 複製版本資訊

- **WHEN** 使用者於關於頁點品牌卡版本列旁的「複製版本資訊」鈕
- **THEN** 剪貼簿為三行：`Speclink <app 版號>`、`Engine <引擎版號>`、`Platform <平台> <架構>`，鈕短暫顯示已複製

#### Scenario: 版本列狀態籤與更新頁同步並可點入

- **WHEN** 使用者於更新頁按「檢查更新」且目前已是最新，接著切到關於頁並點品牌卡版本列的狀態籤
- **THEN** 關於頁狀態籤為「已是最新」且頁內沒有「檢查更新」與「更新日誌」列；點擊後主區為更新頁、麵包屑「設定 / 更新」，「最新版本」列同為已是最新

#### Scenario: 連結外開

- **WHEN** 使用者點「回報問題」列
- **THEN** 系統瀏覽器開啟倉庫的新 issue 頁，網址含版本三行；開啟失敗時以失敗 toast 呈現錯誤

#### Scenario: 授權對話框

- **WHEN** 使用者點「授權條款」的「檢視」，關閉後再點「第三方授權」的「檢視」
- **THEN** 第一次對話框顯示 LICENSE 全文；第二次對話框分 npm 與 cargo 兩節列出套件名稱、版本與授權識別字且依名稱排序；清單檔缺席時顯示「清單尚未產生」而非錯誤

<!-- @trace
source: desktop-about-page
updated: 2026-10-12T09:12:54+08:00
-->

---
### Requirement: 詳情頁的文件樹、閱讀卡與本頁目錄

變更詳情頁、討論詳情頁與已封存詳情頁 SHALL 共用同一個三欄骨架，SHALL NOT 有底部固定列：

- **左欄「文件樹」**（固定 240px、獨立捲動）由上而下為身分區、「文件」節點清單、「狀態」列。身分區 SHALL 呈現名稱或 slug（等寬 14px，過長時折行、SHALL NOT 撐寬欄）、階段籤（SHALL NOT 含審查／驗證章籤——品質關卡由「狀態」列承載）、一行出身（建立者首字母圓標＋名字、產生工具、建立日、「開工 YYYY-MM-DD」；缺席的段整段缺席）、關聯列各自成行——變更為「來自」＋來源討論籤（首顆直出、其餘收「+N」浮層）與「規格」＋delta 能力籤（點擊切到規格文件）；討論為 topic（最多三行、溢出截斷）與「關聯」一列（先前討論數，自背景的 `Prior discussions:` 行計數，`none` 時該列缺席；點擊切到背景文件）。建立者與開工者含 email 的完整識別 SHALL 收進主題化提示。「文件」節點 SHALL 每份文件一個，含圖示、名稱與徽章，點擊 SHALL 切換閱讀卡內容，作用中節點 SHALL 以主色淡底標示；節點 SHALL NOT 列出文件內的段落——頁內清單一律由本頁目錄承接。「狀態」列 SHALL 承接原「狀態」卡的內容（左標籤右內容，細線分隔）。
- **中欄閱讀區**：內容欄置中、寬 768px，一張閱讀卡。閱讀卡 SHALL 由上而下為標題區（24px 一般字重文件名、籤槽、同列右端的「開啟 ▾」）、出處行（灰字：變更與已封存變更的文件為該檔案相對變更目錄的路徑，如 `proposal.md`、`specs/<capability>/spec.md`；排程為推算來源「由 speclink plan 推算」；討論與已封存討論的文件為自記錄推導的出處——結論為記錄路徑與「N 輪之後寫入」、討論過程為日期範圍與各 mode 輪數、背景為「記錄建立時寫入一次」與建立日、衍生變更為「依結論的分刀分組」或「依轉出順序」；已封存討論的文件 SHALL NOT 出記錄路徑段）、數字磚列容器（0 到 4 格、磚缺席時整列缺席）、提示框容器、內文。切換文件時閱讀區 SHALL 自頂端呈現；文件載入三態沿用「詳情頁文件載入以 skeleton 呈現」。
- **右欄「本頁目錄」**（200px）SHALL 自閱讀卡渲染後的 DOM 推導目錄：列入帶目錄錨點的標題，點擊 SHALL 捲至該標題，捲動時 SHALL 高亮目前段；沒有錨點時 SHALL NOT 渲染。頁寬 ≥1200px 時 SHALL 顯示，窄於 1200px 時 SHALL 隱藏且閱讀區維持置中。各文件的目錄錨點為其渲染器標出的區段標題：提案與設計的章節標籤、任務的群組標題、規格 delta 的色標區段標頭與需求標題、排程的四卡標題、工單的輪次標題、結論的欄位區段標題（含「分刀」）、討論過程的輪次籤（依輪卡的畫面順序，由新到舊）、背景的三個區段標題、衍生變更的分刀組頭。

變更的文件節點依序 SHALL 為提案、設計、任務（徽章 n/m）、規格（delta 計數徽章）、排程（有 `wave` 時徽章「第 N 波」）、審查、驗證（後兩者的出現條件與「詳情頁的工單分頁」相同，徽章為末輪序號）；討論的文件節點依序 SHALL 為結論、討論過程（徽章為輪數）、背景、衍生變更（徽章為數量），預設文件依「討論詳情頁檢視與轉出變更」；已封存變更為提案、設計、任務、規格與條件式的審查、驗證，已封存討論為結論、討論過程、背景、衍生變更。已封存詳情頁 SHALL 為同一骨架的唯讀變體：身分區帶「已封存」籤與「封存 YYYY-MM-DD」，標題列 SHALL NOT 有動作。

#### Scenario: 三欄骨架與左欄內容

- **WHEN** 使用者於寬 1440px 的視窗開啟 reviewStatus 為 reviewed、verifyStatus 為 verified、來源討論一份、delta 能力一個、9/9 任務的已就緒變更詳情頁
- **THEN** 主區為左欄文件樹、中欄置中 768px 的閱讀卡、右欄本頁目錄，無底部固定列；左欄身分區顯示等寬變更名、「已就緒」籤（無「已審查」「已驗證」章籤）、一行出身、「來自」列的討論籤與「規格」列的能力籤；「文件」節點為提案、設計、任務 9/9、規格、排程、無審查與驗證節點；「狀態」列為任務進度、品質關卡（「已審查」「已驗證」章籤）、排程三列

#### Scenario: 點文件節點切換閱讀卡

- **WHEN** 使用者於變更詳情頁點左欄的「任務」節點，再點「規格」節點
- **THEN** 「任務」節點以主色淡底標示且閱讀卡標題為「任務」、內文為任務清單；再點後「規格」節點標示、閱讀卡自頂端呈現規格 delta，左欄捲動位置不變

#### Scenario: 閱讀卡的出處行

- **WHEN** 使用者於變更詳情頁依序看「提案」與「排程」兩份文件，再開啟一筆 11 輪、2026-10-12 建立的 live 討論詳情頁依序看結論、討論過程、背景、衍生變更，最後開啟一筆已封存討論的結論文件
- **THEN** 提案的出處行為 `proposal.md`，排程的出處行為「由 speclink plan 推算」；討論的結論出處行為 `openspec/discussions/<slug>.md · 11 輪之後寫入`、討論過程為日期範圍與各 mode 輪數、背景為「記錄建立時寫入一次 · 2026-10-12」、衍生變更為「依結論的分刀分組」；已封存討論的結論出處行只有「N 輪之後寫入」、無路徑段

#### Scenario: 本頁目錄的錨點與高亮

- **WHEN** 使用者於寬 1440px 的視窗開啟含 Why、What Changes、Impact 三個模板章節的提案文件並往下捲到「影響」區段
- **THEN** 右欄本頁目錄列出「為什麼」「變更內容」「影響」三個錨點，捲到「影響」後該錨點高亮；點擊「為什麼」閱讀區捲回該章節

#### Scenario: 窄於 1200px 時無本頁目錄

- **WHEN** 使用者把視窗縮到寬 1100px 再開啟同一份文件
- **THEN** 右欄本頁目錄不顯示，左欄文件樹仍在，閱讀卡維持置中 768px，頁無水平捲軸

#### Scenario: 已封存詳情頁的唯讀變體

- **WHEN** 使用者於已封存頁開啟一筆封存變更的詳情頁
- **THEN** 同一三欄骨架；左欄身分區帶「已封存」籤與「封存 YYYY-MM-DD」；標題列右端只有 ‹ n／m ›、無分析、封存與「⋯」；「文件」節點為提案、設計、任務、規格；任務文件的勾選框不可點

#### Scenario: 討論文件的本頁目錄錨點

- **WHEN** 使用者於寬 1440px 的視窗開啟一筆 20 輪、結論含六欄位與 7 刀的討論詳情頁，依序看結論、討論過程、衍生變更
- **THEN** 結論的本頁目錄依序為決定、分刀、理由、否決替代案、擱置、記錄去向、下一步；討論過程的本頁目錄依序為 Round 20 到 Round 1；衍生變更的本頁目錄為 7 個分刀組頭；點任一錨點閱讀區捲至該處


<!-- @trace
source: detail-quality-stamp-dedup
updated: 2026-10-13T23:23:36+08:00
-->

---
### Requirement: 可點元素的游標與 hover 回饋

桌面 app（主視窗與系統匣面板）的可點元素——按鈕與圖示鈕、清單列、看板卡片、收合卡頭、名稱籤與關聯籤、分頁與文件樹節點、專案欄與設定分類欄的導覽列、選單項與下拉選項——滑鼠移入時 SHALL 維持箭頭游標，SHALL NOT 顯示手指游標。可點性 SHALL 以 hover 回饋表示：整列、卡頭與無外框的可點元素 SHALL 於 hover 時顯示前景色 5% 的淡底；自帶底色的籤與按鈕 SHALL 於 hover 時換成另一個可辨識的底色，SHALL NOT 換成比原底色更淡、肉眼難辨的底色。

例外與不適用：Markdown 正文內的超連結 SHALL 維持手指游標；拖曳把手的抓取游標、文字輸入的文字游標與調整尺寸的游標不受本需求約束，維持各自的游標。

停用或唯讀的元素（停用的按鈕、選擇卡與最近開啟列，已封存變更與批次寫回中的任務列，不可開啟的列與籤）SHALL 為箭頭游標，SHALL NOT 顯示禁止游標，SHALL NOT 顯示 hover 底色。

游標 SHALL 由共用樣式的單一設定集中決定：共用樣式的預設值為手指，桌面 app 的根樣式把它覆寫為箭頭；可用的原生按鈕與 `role="button"` 元素 SHALL 由共用樣式的全域規則取用該設定。元件與元件庫原語 SHALL NOT 自行指定手指或禁止游標；守門測試 SHALL 掃描 packages/ui、桌面 app 與網頁後台的元件原始碼，出現寫死的手指或禁止游標 class 即失敗並列出檔案。

#### Scenario: 桌面版 hover 收合卡頭與清單列維持箭頭並出現淡底

- **WHEN** 使用者在桌面 app 把滑鼠移到討論過程未展開的輪次卡頭，再移到規格頁清單的任一列
- **THEN** 兩者游標皆為箭頭，且滑過的卡頭與列各自出現前景色 5% 的淡底；移開後淡底消失

#### Scenario: 桌面版按鈕、文件樹與導覽列維持箭頭

- **WHEN** 使用者在桌面 app 依序 hover 詳情頁的工具列按鈕、文件樹節點、專案欄的「規格」導覽列與「開啟 ▾」選單裡的一個項目
- **THEN** 每個元素的游標皆為箭頭，且各自出現 hover 底色

#### Scenario: Markdown 正文的超連結維持手指

- **WHEN** 使用者在桌面 app 的文件閱讀卡把滑鼠移到 Markdown 正文內的超連結上
- **THEN** 游標為手指

#### Scenario: 唯讀任務列與停用選擇卡沒有 hover

- **WHEN** 使用者在桌面 app hover 已封存變更任務文件的任一任務列，再 hover 開啟專案畫面中停用的選擇卡
- **THEN** 兩者游標皆為箭頭（非禁止游標），且底色與未 hover 時相同

#### Scenario: 自帶底色的籤 hover 換成可辨識的底色

- **WHEN** 使用者在桌面 app 的規格詳情頁 hover 溯源關聯列的「+N」籤
- **THEN** 游標為箭頭，籤的底色由灰底換成主色淡底

#### Scenario: 寫死手指游標的元件被守門擋下

- **WHEN** 開發者在 packages/ui 的元件原始碼新增一個帶 `cursor-pointer` class 的元素並執行前端測試
- **THEN** 游標守門測試失敗，失敗訊息列出該檔案的 repo 相對路徑；把 class 改成取用共用游標設定後測試通過

#### Scenario: 共用游標設定被移除時守門擋下

- **WHEN** 共用樣式缺少游標設定的預設值或全域規則，或桌面 app 根樣式缺少箭頭覆寫
- **THEN** 游標守門測試失敗並指出缺少的是哪一項

<!-- @trace
source: clickable-cursor-hover-rule
updated: 2026-10-13T11:49:40+08:00
-->

---
### Requirement: 詳情頁狀態列的標籤欄依最長標籤對齊

變更、討論、規格與已封存詳情頁左欄的「狀態」列 SHALL 讓同一頁的全部列共用一個標籤欄：標籤欄寬度 SHALL 為該頁最長標籤的寬度，下限 56px、上限 112px；超過上限的標籤 SHALL 在字間折行。任何介面語言下，標籤 SHALL NOT 與同列的數值重疊，同一頁各列數值的左緣 SHALL 對齊。繁中介面的標籤都不超過 56px，標籤欄 SHALL 維持 56px。

#### Scenario: 英文介面的規格詳情頁狀態列不重疊

- **WHEN** 使用者以英文介面開啟一份 2 條需求、1 個溯源變更的規格詳情頁
- **THEN** 狀態列顯示「Requirements」與「2」、「Provenance」與「1」，標籤與數值之間有間距、不重疊，兩列數值的左緣對齊

#### Scenario: 繁中介面版面不變

- **WHEN** 使用者以繁中介面開啟同一份規格詳情頁
- **THEN** 狀態列標籤欄寬 56px，「需求」「溯源」「最近更新」三列的數值左緣對齊

<!-- @trace
source: fix-english-status-labels-and-plurals
updated: 2026-10-15T15:31:48+08:00
-->

---
### Requirement: 更新日誌依介面語言呈現

desktop 的更新日誌對話框（自動彈出的「X.Y.Z 更新內容」與瀏覽模式的「更新日誌」）與更新頁「更新日誌」卡（含每版一列的首條摘要）SHALL 依目前的介面語言呈現 release-notes.json 的條目：介面語言為英文時 SHALL 顯示每個條目的 `en` 文字，分組標題 SHALL 為 New features、Fixes、Improvements；介面語言為繁中時 SHALL 顯示 `zh-TW` 文字，分組標題 SHALL 為新功能、修正、改善。切換介面語言後，已開啟的對話框與更新頁 SHALL 改以新語言呈現。彈出判定與已看過記錄 SHALL NOT 受介面語言影響。

#### Scenario: 英文介面看到英文更新內容

- **WHEN** 介面語言為英文的使用者由 0.8.0 更新到 0.9.0 後首次啟動
- **THEN** 彈出「0.9.0」更新內容對話框，條目為英文、分組標題為 New features 等英文標題

#### Scenario: 繁中介面維持繁中

- **WHEN** 介面語言為繁中的使用者在更新頁按「全部更新日誌」
- **THEN** 對話框列出全部版本的繁中條目，分組標題為新功能、修正、改善

#### Scenario: 切換語言不影響已看過記錄

- **WHEN** 使用者看過 0.9.0 的更新內容後把介面語言改為英文並重新啟動
- **THEN** 不再彈出 0.9.0 的更新內容

<!-- @trace
source: bilingual-release-notes
updated: 2026-10-15T17:28:24+08:00
-->

---
### Requirement: 全介面配色一致套用有效外觀

桌面 app 的背景、文字、邊框、表單控制項、狀態徽章與互動提示 SHALL 使用 desktop-config 所定義的有效外觀。深淺色票 SHALL 採官方 0.9，既有狀態色角色 SHALL 保留；手動淺色時 SHALL NOT 混入系統深色變體，手動深色時亦 SHALL NOT 混入系統淺色變體。共用樣式未收到外觀覆寫的宿主 SHALL 維持依系統配色，SHALL NOT 因桌面偏好改變其他 app 的選擇。

#### Scenario: 系統與手動選擇相反仍一致

- **WHEN** 系統為深色而 app 選淺色，或系統為淺色而 app 選深色
- **THEN** 看板、設定、詳情面板、狀態徽章與表單控制項皆使用所選模式且文字可讀，既有狀態色意義不變

#### Scenario: 共用樣式宿主無覆寫

- **WHEN** server-web 未設定外觀覆寫，使用者切換系統深淺模式
- **THEN** server-web 的共用背景、文字與狀態色跟隨系統，不受桌面 app 儲存偏好影響

<!-- @trace
source: desktop-theme-and-project-directory
updated: 2026-10-15T22:40:22+08:00
-->

---
### Requirement: 裝置層專案圖示底色

Desktop SHALL 在既有專案設定提供預設及六個固定色票，Local／Remote SHALL 依 locatorKey 儲存在裝置偏好，不修改團隊文件。全部色票的設定預覽 SHALL 固定深底白字，只以外框標示目前選項；實際專案圖示選取 SHALL 深底白字，未選取 SHALL 淺底黑字，包含預設色票，主視窗與系統匣 SHALL 一致且亮暗主題使用同一配色。預設深／淺底 SHALL 為#374151／#E5E7EB，青綠#0F766E／#99F6E4，藍#1D4ED8／#BFDBFE，紫#9400D3／#DA70D6，粉紅#BE185D／#FBCFE8，橘#C2410C／#FED7AA，綠#15803D／#BBF7D0；不新增色票。重開 SHALL 保留；失敗 SHALL 顯示錯誤且不變更已採用值。Remote reader SHALL 能改自己的裝置偏好。

#### Scenario: 同首字母的專案以色彩區分

- **WHEN** Spec A 設紫色且被選取，Spec B 設粉紅色且未選取
- **THEN** 兩者皆顯示S，A為#9400D3深紫底白字、B為#FBCFE8淺粉紅底黑字，切換選取後A為#DA70D6淺紫底黑字、B為#BE185D深粉紅底白字；關閉重開仍保留，清回預設只影響該專案

#### Scenario: 儲存失敗不改舊值

- **WHEN** 藍色專案改紫色但裝置儲存拒絕
- **THEN** 顯示失敗原因，圖示維持藍色；Remote政策沒有寫入

#### Scenario: 設定預覽全部深底但外框仍反映選擇
- **WHEN** 使用者在專案設定選紫色並查看七個選項
- **THEN** 七個預覽全部深底白字，只有紫色選項的外框標示選取，紫色預覽為#9400D3且非選中的預設預覽為#374151

#### Scenario: 預設色票在兩種主題一致
- **WHEN** 沒有自訂色的專案從選取轉為未選取，並切換亮暗主題
- **THEN** 選取為#374151底白字，未選取為#E5E7EB底黑字，主視窗與系統匣同色，主題切換不改此配對


<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 設定頁點專案返回看板

Desktop SHALL 在應用程式設定及專案設定點左側專案圖示或使用專案切換快捷鍵時，於專案啟用成功後進入該專案看板，包含目前已作用中的專案。專案設定有未儲存修改時 SHALL 先沿用捨棄確認；取消 SHALL 保留原頁、作用中專案與修改且不開始切換。切換或重連失敗 SHALL 維持原頁與專案並顯示錯誤，不顯示新專案看板。SHALL 共用同一離開設定入口，既有返回鈕回進入前頁面的行為不變。

#### Scenario: 兩種設定模式點目前或其他專案
- **WHEN** 使用者在任一設定模式點有效專案A的圖示，A是目前或其他專案
- **THEN** 啟用成功後關閉設定分類、顯示A看板，齒輪不再作用中，詳情或浮層不殘留

#### Scenario: 未儲存取消與確認
- **WHEN** 專案A設定有草稿，使用者點專案B或使用專案切換快捷鍵
- **THEN** 先顯示捨棄確認；取消後仍為A設定且草稿保留，確認後才切B，成功後顯示B看板

#### Scenario: 切換失敗留在設定
- **WHEN** 使用者從設定點目錄失效的本機專案或需要重新登入的Remote專案
- **THEN** 原頁與作用中專案維持，顯示既有錯誤／復原操作，不冒稱已切換

<!-- @trace
source: desktop-colors-network-and-offline-bundles
updated: 2026-10-19T23:00:19+08:00
-->

---
### Requirement: 本分支更新與原作者版本

更新頁主要版本檢查、更新候選與下載入口 SHALL 以 https://github.com/mosutw/speclink 為來源，原作者版本 SHALL 在獨立列顯示並可點往 https://github.com/MomoChenisMe/speclink。原作者版本 SHALL NOT 成為本分支安裝候選，也 SHALL NOT 與目前分支版號混用判定更新。About／原生選單的本專案原始碼與回報問題 SHALL 指向 mosutw，原作者 SHALL 另保留清楚署名與專案連結，zh-TW／en 行為一致。

本分支或原作者的已發布桌面版本 SHALL 從固定專案的 GitHub Releases metadata 查詢，排除 draft、prerelease、CLI-only 發行；單次最多讀 100 筆，沒有合格桌面版本 SHALL 顯示未取得而非已是最新。兩者各自顯示版本、狀態與查詢時間；失敗時舊版本只能標為上次取得，未曾成功則顯示未取得。403／429、404、TLS、Proxy 或解析錯誤 SHALL 不清除內附版本與日誌，不將失敗當成最新，不以另一個專案的資料補位。兩者 SHALL 共用已選 Proxy 政策且不偷偷直連。

只有本分支提供與信任公鑰相符的合法簽章更新包時 SHALL 使用既有同意、下載及安裝流程；缺少 latest.json／對應平台或沒有可驗簽套件時 SHALL 呈現手動更新狀態及本分支 Release 連結，不靜默降級成未簽章安裝。簽章不符 SHALL 保持拒裝錯誤。版本查詢 SHALL NOT 要求使用者提供 GitHub token 或由前端直接連 GitHub。

#### Scenario: 本分支有新版而原作者版號更高

- **WHEN** 本機版本為 0.10.0-build.3，本分支有較新版桌面 Release，原作者另有更高版本
- **THEN** 更新候選只對照本分支，原作者版本顯示在獨立列，點原作者連結導向原作專案，不以原作者包覆蓋本分支

#### Scenario: 只有未簽章安裝檔

- **WHEN** 本分支已發布桌面安裝檔，但未發布 latest.json 或有效簽章更新包
- **THEN** 顯示取得的本分支版本與手動下載入口，沒有可直接安裝的自動更新動作，不把描述檔缺失說成已是最新

#### Scenario: 原作者查詢失敗且本分支正常

- **WHEN** 本分支查詢成功，原作者查詢遇到 403 或 TLS 失敗
- **THEN** 本分支結果不變，原作者列顯示其獨立錯誤及上次成功時間或未取得，內附版本與日誌仍可讀

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 專案貢獻與原作署名

中英 README SHALL 清楚列出 mosutw 為本分支維護者、Codex 為 AI 開發協作工具，並保留 MomoChenisMe 原作者及原作專案連結與既有授權。貢獻說明 SHALL 區分 README 的人工署名與 GitHub 側欄 Contributors 的自動統計，說明作者 email 與 GitHub 帳號關聯、預設分支及真實共同作者的 Co-authored-by 用法；SHALL NOT 承諾 README 文字可控制側欄或偽造 Codex 帳號／email。此交付 SHALL 不修改 Git 身分、歷史或推送遠端。

#### Scenario: 查看本分支貢獻說明

- **WHEN** 使用者檢視任一語言 README 與 CONTRIBUTING.md
- **THEN** 可辨認維護者、AI 協作與原作者，並理解側欄需由符合 GitHub 規則的實際提交統計，不將 AI 工具標示為已驗證 GitHub 帳號

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 左側專案排序與裝置持久化

Desktop SHALL 提供垂直拖曳及右鍵上移／下移來調整左側專案順序，沿用 locator 身分與裝置 tabs 順序。成功保存後重開、Tray 及專案切換快捷鍵 SHALL 使用相同順序；新增專案附尾、關閉只刪指定項。排序 SHALL NOT 切換作用中專案、丟失草稿或更動專案文件；失效／離線專案仍可排序。取消、未知或重複項目、超出範圍 SHALL 不更動原順序，保存失敗 SHALL 顯示錯誤並維持舊值。第一／末項對應的移動項 SHALL 停用。

#### Scenario: 本機與遠端混排後重開

- **WHEN** 使用者把順序 A（本機）、B（Remote離線）、C 的 C 拖到 A 前並重開 Desktop
- **THEN** 左側列與Tray皆為 C、A、B，目前專案及三者身分不變，沒有專案文件寫入

#### Scenario: 取消與保存失敗

- **WHEN** 拖曳後按 Escape、拖到無效位置，或上移操作的裝置儲存寫入失敗
- **THEN** 順序仍是操作前值；儲存失敗顯示錯誤，無誤觸專案切換

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 更新頁內附資訊不依賴網路成功

Desktop 更新頁 SHALL 在 updater 尚未就緒、檢查失敗、Proxy／TLS 錯誤時仍呈現已知目前版本與程式內附更新日誌，不用網路空結果取代內附內容。未就緒 SHALL 顯示說明並停用更新動作；失敗 SHALL 顯示最後一次檢查狀態、重試及前往網路與診斷的入口。背景檢查不主動打擾的行為 SHALL 保留，但設定頁 SHALL NOT 將背景失敗顯示為已是最新。內附日誌 SHALL 隨zh-TW／en切換。

#### Scenario: 更新元件缺席

- **WHEN** updater 未完成初始化而使用者進入更新頁
- **THEN** 日誌及已知版本可讀，顯示更新服務尚未就緒，不能觸發無效下載

#### Scenario: Proxy拒絕與背景失敗

- **WHEN** 更新請求遭Proxy拒絕，或背景檢查失敗後回到設定頁
- **THEN** 日誌仍可讀，顯示失敗狀態及診斷入口，不把失敗當作最新版本或只顯示頁標題

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 文件區只顯示進行中的審查與驗證

Desktop 提案詳情的文件區 SHALL 只顯示目前進行中的 `review.md` 與 `verify.md` 工單；蓋章後的歷史紀錄 SHALL NOT 出現在文件區、文件樹或獨立「檢查紀錄」清單。已審查與已驗證的歷史紀錄 SHALL 僅能從品質關卡各自的狀態控制項展開與查閱，並依開始時間由新到舊排列。已封存詳情 SHALL NOT 另顯示審查／驗證歷史清單。切換專案或變更時 SHALL 清除舊文件與清單選取，且不得跨專案顯示內容。

#### Scenario: 文件區只顯示進行中的工單

- **WHEN** 提案同時有進行中的 `review.md`、`verify.md` 及過往完成紀錄
- **THEN** 文件區只呈現兩份進行中的工單；完成紀錄不出現在文件區或文件樹

#### Scenario: 蓋章後從品質關卡查詢歷史

- **WHEN** 使用者展開品質關卡的「已審查」或「已驗證」
- **THEN** 該類完成紀錄依開始時間由新到舊顯示；歷史紀錄不額外出現在文件區或已封存詳情

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->

---
### Requirement: 品質關卡展開審查與驗證紀錄

Desktop SHALL 讓提案詳情品質關卡中的「已審查」與「已驗證」狀態分別作為展開控制項。展開一類 SHALL 從同一 Store 載入該類已保存紀錄，依紀錄開始時間由新到舊列出開始時間、結果及紀錄 ID；使用者選取一筆 SHALL 在提案詳情中顯示相應紀錄內容。兩類展開狀態互相獨立且控制項可用鍵盤操作。空清單 SHALL 顯示無紀錄；讀取失敗 SHALL 顯示錯誤與重試入口；載入中的紀錄 SHALL 不跨專案或變更顯示。此入口 SHALL 是審查與驗證歷史紀錄的唯一 Desktop 查閱入口，不觸發重跑。

#### Scenario: 展開並查看審查紀錄

- **WHEN** 使用者在提案詳情點選「已審查」並選取一筆審查紀錄
- **THEN** 只列出審查紀錄且顯示所選完整紀錄內容，「已驗證」清單維持原狀

#### Scenario: 展開並查看驗證紀錄

- **WHEN** 使用者在提案詳情以滑鼠或鍵盤點選「已驗證」並選取一筆驗證紀錄
- **THEN** 只列出驗證紀錄且顯示所選完整紀錄內容，「已審查」清單維持原狀

#### Scenario: 空清單、失敗與切換變更

- **WHEN** 清單為空、查詢失敗，或讀取期間切換到另一個變更
- **THEN** 分別顯示空狀態、可重試錯誤，或只顯示新變更的資料，且不啟動審查／驗證

<!-- @trace
source: desktop-workflow-and-offline-security-review
updated: 2026-10-23T14:12:44+08:00
-->