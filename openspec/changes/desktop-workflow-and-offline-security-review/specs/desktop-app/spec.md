## ADDED Requirements

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

### Requirement: 專案貢獻與原作署名

中英 README SHALL 清楚列出 mosutw 為本分支維護者、Codex 為 AI 開發協作工具，並保留 MomoChenisMe 原作者及原作專案連結與既有授權。貢獻說明 SHALL 區分 README 的人工署名與 GitHub 側欄 Contributors 的自動統計，說明作者 email 與 GitHub 帳號關聯、預設分支及真實共同作者的 Co-authored-by 用法；SHALL NOT 承諾 README 文字可控制側欄或偽造 Codex 帳號／email。此交付 SHALL 不修改 Git 身分、歷史或推送遠端。

#### Scenario: 查看本分支貢獻說明

- **WHEN** 使用者檢視任一語言 README 與 CONTRIBUTING.md
- **THEN** 可辨認維護者、AI 協作與原作者，並理解側欄需由符合 GitHub 規則的實際提交統計，不將 AI 工具標示為已驗證 GitHub 帳號

### Requirement: 左側專案排序與裝置持久化

Desktop SHALL 提供垂直拖曳及右鍵上移／下移來調整左側專案順序，沿用 locator 身分與裝置 tabs 順序。成功保存後重開、Tray 及專案切換快捷鍵 SHALL 使用相同順序；新增專案附尾、關閉只刪指定項。排序 SHALL NOT 切換作用中專案、丟失草稿或更動專案文件；失效／離線專案仍可排序。取消、未知或重複項目、超出範圍 SHALL 不更動原順序，保存失敗 SHALL 顯示錯誤並維持舊值。第一／末項對應的移動項 SHALL 停用。

#### Scenario: 本機與遠端混排後重開

- **WHEN** 使用者把順序 A（本機）、B（Remote離線）、C 的 C 拖到 A 前並重開 Desktop
- **THEN** 左側列與Tray皆為 C、A、B，目前專案及三者身分不變，沒有專案文件寫入

#### Scenario: 取消與保存失敗

- **WHEN** 拖曳後按 Escape、拖到無效位置，或上移操作的裝置儲存寫入失敗
- **THEN** 順序仍是操作前值；儲存失敗顯示錯誤，無誤觸專案切換

### Requirement: 更新頁內附資訊不依賴網路成功

Desktop 更新頁 SHALL 在 updater 尚未就緒、檢查失敗、Proxy／TLS 錯誤時仍呈現已知目前版本與程式內附更新日誌，不用網路空結果取代內附內容。未就緒 SHALL 顯示說明並停用更新動作；失敗 SHALL 顯示最後一次檢查狀態、重試及前往網路與診斷的入口。背景檢查不主動打擾的行為 SHALL 保留，但設定頁 SHALL NOT 將背景失敗顯示為已是最新。內附日誌 SHALL 隨zh-TW／en切換。

#### Scenario: 更新元件缺席

- **WHEN** updater 未完成初始化而使用者進入更新頁
- **THEN** 日誌及已知版本可讀，顯示更新服務尚未就緒，不能觸發無效下載

#### Scenario: Proxy拒絕與背景失敗

- **WHEN** 更新請求遭Proxy拒絕，或背景檢查失敗後回到設定頁
- **THEN** 日誌仍可讀，顯示失敗狀態及診斷入口，不把失敗當作最新版本或只顯示頁標題

### Requirement: 提案與已封存的檢查紀錄文件樹

Desktop SHALL 在提案詳情及已封存詳情的文件樹提供檢查紀錄群組，可依審查／驗證／安全審查、時間與結果選取每次已保存文件。蓋章後活動工單消失 SHALL NOT 隱藏已完成紀錄；讀取 SHALL 經同一Store查詢，不依賴另一台裝置的資料目錄。未知／缺失紀錄 SHALL 顯示未保存或查詢錯誤，不重跑檢查；Remote的reader可查、離線／401保留已載入唯讀內容並明示狀態，不跨專案混入結果。歷史Markdown SHALL 遮罩並視為不可信內容渲染。

#### Scenario: 蓋章後與封存後查詢

- **WHEN** 審查及驗證蓋章完成，使用者先於提案、再於保留紀錄的已封存詳情選取其中一筆紀錄
- **THEN** 兩處皆顯示相同完整輪次、當時規則與接受紀錄，不要求存在活動工單

#### Scenario: 切換專案與遠端失敗

- **WHEN** A的歷史讀取尚未完成時切到B，或Remote紀錄查詢回401
- **THEN** B不顯示A紀錄，Remote明示需重新登入且不把既有內容誤標為本次新讀取成功

## MODIFIED Requirements

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
