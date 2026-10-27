## MODIFIED Requirements

### Requirement: 提案與設計章節以中文標籤呈現

<!-- BEFORE: 只有白名單章節成 20px 粗體標籤區塊、白名單外章節併入 prose、無白名單章節整篇退回；無專屬畫法、無數字磚與提示框。 -->

變更詳情頁與已封存檢視的提案／設計文件 SHALL 以二級標題切段：每個二級標題 SHALL 成為一段、帶本頁目錄錨點、算進數字磚；已知模板章節標題（提案側 Why／What Changes／Non-Goals／Capabilities／New Capabilities／Modified Capabilities／Impact／Problem／Root Cause／Proposed Solution／Success Criteria／Summary／Motivation／Alternatives Considered；設計側 Context／Goals / Non-Goals／Decisions／Implementation Contract／Risks / Trade-offs／Migration Plan／Open Questions；標題尾的「(optional)」於比對前剝除）SHALL 顯示中文標籤與該章節的專屬畫法，英文模板標題 SHALL NOT 以標題文字直出；白名單以外的二級標題 SHALL 以原文為段標題（不翻譯）、內文照 prose 排版。章節標題款式 SHALL 為 16px 粗體（計算字級 16px、大於內文的 15px），與任務群組卡頭、規格 delta 的需求標題、排程四卡頭、工單輪卡頭同款式（色標區段標頭保留各 delta 色彩）；列式章節的標題右端 SHALL 附列點數籤。整份文件沒有任何二級標題時 SHALL 整篇以單一 markdown 檢視退回，不報錯。章節白名單 SHALL 留在前端，SHALL NOT 讀 schema 模板。渲染 SHALL NOT 修改任何來源檔案。

提案側的專屬畫法 SHALL 為：Summary 成灰底引言段；Why／Motivation／Problem／Root Cause／Success Criteria 的頂層列點成細線列（非列點段落照 prose）；What Changes／Proposed Solution 的頂層列點與編號項成編號步驟；Non-Goals 的列點成帶禁止圖示的細線列；Alternatives Considered 的每個列點以首個「——」切成「方案／落敗理由」兩段，切不到時整條當一般列；Capabilities 的 New Capabilities／Modified Capabilities 次級標籤之下每個「`能力名`: 說明」列點成能力籤＋說明，能力名在本變更已載入的 delta 規格中時籤 SHALL 可點——點擊切到規格文件並選定該能力（規格文件捲到該能力段、「開啟 ▾」指向它），不在時為不可點的籤，「(none)」顯示「無」；Impact 的「Affected specs:」列成「規格」列（能力籤規則同上），「Affected code:」之下的 New／Modified／Removed 子列點依序成「新增 N」「修改 N」「移除 N」三組等寬路徑籤（路徑以「、」或「,」分隔，「(none)」顯示「無」），路徑籤 hover 時 SHALL 出現複製鈕，宿主提供本機路徑動作時 SHALL 另出「在 Finder 顯示」鈕（非 macOS 依平台詞條），「移除」組的路徑籤 SHALL NOT 出該鈕（那些檔案在 apply 後已不存在）；Impact 段不符這個形狀時 SHALL 整段當一般細線列。

設計側的專屬畫法 SHALL 為：Context 成細線列；Goals / Non-Goals 以「**Goals:**」與「**Non-Goals:**」兩個粗體標籤切成左右兩欄（各列其下的列點），切不到時照 prose；Decisions 之下每個三級標題一張可收合卡（標題開頭的 D 編號成籤、卡頭右端標列點數；預設收合，點卡頭展開；自任務文件的 D 籤跳來時該卡展開並捲到卡頭）；Implementation Contract 的「- **鍵**：值」列點成鍵值列，切不到時照 prose；Risks / Trade-offs 的每個列點成一個 warning 提示框（以首個「→」切標題與內文，切不到時整條歸標題；標題只取首行並以行內標記呈現，其餘行（巢狀子項、續行）接在內文之前）；Migration Plan 與 Open Questions 照 prose。

閱讀卡的籤槽與數字磚 SHALL 由文件結構推出：提案的籤槽標出模板形狀——有 Problem 與 Root Cause 為「Bug Fix」、有 Summary 與 Motivation 為「Refactor」、有 Why 與 What Changes 為「Feature」、皆非時籤缺席；提案的數字磚依序取「解法步驟」（What Changes 或 Proposed Solution 的列點與編號項數）、「非目標」、「成功準則」、「能力」（新增與修改的能力數）、「影響的檔案」（新增、修改、移除的路徑總數）、「替代方案」中章節存在的前四格，章節存在而為零時顯示 0，不足四格少放；設計的數字磚依序為「決策」（Decisions 下三級標題數）、「目標／非目標」（兩欄列點數，顯示為「a ／ b」）、「風險與取捨」（列點數）、「開放問題」（列點數，「無」為 0），章節缺席時該格缺席。提案的 Capabilities 段之下 SHALL 有一個 note 提示框，每個已載入 delta 規格的修改能力一行：「能力名 有 N 條需求被改寫」，該 delta 含 REMOVED-SCENARIO 宣告時加「，其中 M 個 scenario 以 REMOVED-SCENARIO 宣告」；沒有任何可推的能力時提示框缺席。

#### Scenario: 提案模板章節成中文標籤

- **WHEN** 檢視含 Why、What Changes、Non-Goals、Capabilities、Impact 章節的提案文件
- **THEN** 呈現「為什麼」「變更內容」「非目標」「能力」「影響」五段，Why 等英文標題文字不出現在渲染結果；本頁目錄依序列出五段；「變更內容」為編號步驟、「非目標」為帶禁止圖示的細線列

##### Example: 章節對照與畫法

| 來源章節標題 | 呈現標籤 | 畫法 |
| ------------ | -------- | ---- |
| ## Summary | 摘要 | 灰底引言段 |
| ## Why | 為什麼 | 細線列 |
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

<!-- REMOVED-SCENARIO: 標籤為大標題且字級大於內文 -->

- **WHEN** 檢視提案文件的章節標題、任務文件的群組卡頭與規格文件的需求標題
- **THEN** 三者皆為粗體且計算字級為 16px，內文計算字級為 15px；章節標題不是 20px

#### Scenario: 白名單外章節照排

- **WHEN** 檢視提案文件，其含白名單外的二級標題「Rollout」與其下兩段文字
- **THEN** 「Rollout」以原文為段標題呈現（不翻譯、款式同其他章節標題）、進本頁目錄、內文照 prose 排版；整份文件不因它退回單一 markdown 檢視

#### Scenario: 無二級標題整篇退回

<!-- REMOVED-SCENARIO: 無白名單章節整篇退回 -->

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

### Requirement: 任務群組標題與章節標籤同款式

<!-- BEFORE: 群組標題為 16px 粗體的次級標籤、任務為平面清單；無群組卡、無三層任務列、無 D 籤、無數字磚與提示框。 -->

變更詳情頁與已封存檢視的任務文件 SHALL 以 tasks.md 的二級標題分群，每個群組一張卡：卡頭為 16px 粗體的群組標題（文字照來源呈現，不翻譯、不改寫；標題開頭的「N.」序號 SHALL 成等寬序號籤）、右端為該群組的小進度「已完成/總數」；卡頭 SHALL 帶本頁目錄錨點；第一個二級標題之前的任務 SHALL 以無卡頭的清單呈現。每條任務列 SHALL 拆成「標題／內文／驗證」三層：描述以首個「驗證：」切成前段與驗證，前段再以首個全形「：」切成標題與內文；切不到「：」時整段為標題、無內文；切不到「驗證：」時無驗證層；標題緊接的「（design D1、D3）」或「(design D1, D3)」括號 SHALL 自標題剝除，其中的 D 編號成可點的 D 籤（點擊切到設計文件並展開、捲到對應決策卡；設計文件沒有該編號的決策卡時只切換文件），括號內其餘文字歸入內文首行。任務列預設 SHALL 只顯示勾選框、序號、標題、D 籤與手動籤，列尾箭頭鈕 SHALL 展開或收合內文與驗證（展開態以 aria-expanded 標示）；標題與內文中的行內程式碼 SHALL 以等寬籤呈現。整列為勾選框的 label、拖曳把手、批次工具列、「下一個未完成」的 n 快捷鍵與定位捲動 SHALL 不受款式變更影響；手動任務列的淡底與手動籤依「任務列的手動任務徽章」；唯讀（已封存）時勾選框停用、展開收合照常。

閱讀卡的籤槽 SHALL 為「全部完成」（全部任務已勾選時）或「已完成/總數」；數字磚依序 SHALL 為「任務」（總數）、「已完成」、「手動」（帶手動籤的任務數）、「群組」（二級標題數）；沒有任何任務時數字磚與籤槽缺席。有手動任務時 SHALL 有一個以手動色（見「介面狀態語意色分層」）呈現的提示框「N 個手動任務」，內文列出各手動任務的序號與標題（標題的行內程式碼以等寬籤呈現）並說明要由使用者親自做完再勾；每一列 SHALL 附「前往」鈕，點擊後任務文件捲到該任務列並短暫高亮（與「下一個未完成」同一種高亮），唯讀（已封存）時照常可用；沒有手動任務時提示框缺席。

#### Scenario: 群組標題款式一致

- **WHEN** 檢視含「## 1. 原語」「## 2. 骨架」兩個群組的任務文件
- **THEN** 兩張群組卡的卡頭為 16px 粗體標題、左端序號籤「1」「2」、右端各自的小進度；標題文字與來源一致（不含序號後的多餘空白）；本頁目錄列出兩個群組標題

#### Scenario: 互動行為不變

- **WHEN** 在款式調整後的任務文件點整列勾選任務、以把手拖曳排序、按工具列「全部已完成」並按 n
- **THEN** 勾選與排序寫回結果與調整前一致；工具列三鍵與 n 快捷鍵行為不變；點任務列尾的展開鈕不切換勾選

#### Scenario: 任務列三層與展開

- **WHEN** 檢視任務「- [ ] 1.1 新增 `packages/ui/src/docModel.ts`（design D1）：匯出 splitH2Sections；驗證：npm test -w packages/ui -- docModel 全綠」
- **THEN** 列預設只顯示標題「新增 `packages/ui/src/docModel.ts`」（程式碼為等寬籤）與 D 籤「D1」；點列尾展開鈕後顯示內文「匯出 splitH2Sections」與驗證「npm test -w packages/ui -- docModel 全綠」，鈕的 aria-expanded 為 true

##### Example: 三層切分

| 任務描述 | 標題 | 內文 | 驗證 | D 籤 |
| -------- | ---- | ---- | ---- | ---- |
| 新增 `a.ts`（design D1）：匯出 x；驗證：npm test | 新增 `a.ts` | 匯出 x； | npm test | D1 |
| 補 i18n 詞條；驗證：i18n 測試全綠 | 補 i18n 詞條； | （無） | i18n 測試全綠 | （無） |
| 對齊匯出 | 對齊匯出 | （無） | （無） | （無） |
| 改 TaskList（design D5；spec「任務群組標題與章節標籤同款式」）：群組成卡 | 改 TaskList | spec「任務群組標題與章節標籤同款式」＋群組成卡 | （無） | D5 |

#### Scenario: 手動提示框前往任務列

- **WHEN** 任務文件的手動提示框列出「3.2 手動驗收 `dev:desktop` 畫面」，使用者點該列的「前往」鈕
- **THEN** 標題中的 `dev:desktop` 以等寬籤呈現；任務文件捲到 3.2 任務列並短暫高亮，勾選狀態不變

#### Scenario: D 籤跳到設計決策卡

- **WHEN** 使用者於任務文件點任務列的「D3」籤
- **THEN** 左欄「設計」節點作用中、閱讀卡呈現設計文件，「D3」決策卡展開且閱讀區捲到該卡頭；設計文件沒有 D3 卡時只切換到設計文件並自頂端呈現

#### Scenario: 數字磚與手動提示框

- **WHEN** 檢視 3 個群組、9 個任務（7 個已勾選）、其中 3.2 帶手動籤的任務文件
- **THEN** 籤槽為「7/9」；數字磚為「任務 9」「已完成 7」「手動 1」「群組 3」；手動色提示框標題「1 個手動任務」且內文列出「3.2」與其標題；全部勾選後籤槽改為「全部完成」

### Requirement: 任務列的手動任務徽章

<!-- BEFORE: 徽章獨佔描述正上方一行、左緣與描述切齊，配色取進行中藍；SHALL NOT 置於編號之前、編號與描述之間或該列行尾。 -->

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

### Requirement: 介面狀態語意色分層

<!-- BEFORE: 語意分層只有進行中藍、成功綠、警示琥珀、錯誤紅；手動任務徽章與看板待手動章取進行中藍。 -->

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

### Requirement: 規格分頁 delta 區段以色標呈現

<!-- BEFORE: 色標區段標頭之下的需求與 scenario 內文照 prose 排版；無能力頭、無需求段、無 REMOVED-SCENARIO 提示框、無收合、無數字磚。 -->

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

### Requirement: 詳情頁的排程分頁

<!-- BEFORE: 四卡為小寫間距卡頭、無數字磚、無提示框、籤槽空；其餘資料流不變。 -->

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

### Requirement: 詳情頁的工單分頁

<!-- BEFORE: 內文開頭有段標題列（站名、第 N 輪、階段詞、三級計數）、輪次為輕量列；無籤槽、無數字磚、無提示框、無軸向籤。 -->

change 詳情頁 SHALL 於左欄「文件」節點「排程」之後提供兩個條件式節點：清單項 `reviewStatus` 為 `inReview` 時 SHALL 出現「審查」節點（tw「審查」、en「Review」），`verifyStatus` 為 `inVerify` 時 SHALL 出現「驗證」節點（tw「驗證」、en「Verify」），審查節點在前；兩者皆非時 SHALL NOT 渲染這兩個節點，文件節點維持提案／設計／任務／規格／排程。工單文件的閱讀卡 SHALL 為該站工單的結構化呈現：籤槽為「第 N 輪」（N 為末輪序號）與末輪階段詞籤（`phase` 為 `discovery` 顯示 tw「首輪」en「First pass」、`validation` 顯示 tw「複驗」en「Re-check」、`null` 不出階段詞籤）；數字磚依序為「輪」（輪數）、「CRITICAL（末輪）」、「WARNING（末輪）」、「SUGGESTION（末輪）」（末輪三級各自的條數，標籤維持英文）；末輪 CRITICAL 與 WARNING 皆為 0 時 SHALL 有 tip 提示框「可以蓋章」，內文為「末輪只剩 N 條 SUGGESTION，不擋章」（N 為 0 時「末輪無發現」）並點名以 CLI 或 agent 跑該站的 stamp 指令蓋章；否則提示框缺席；內文 SHALL NOT 另畫段標題列。輪次依序號升序逐輪一張卡（與排程文件同一種邊框卡片），末輪預設展開、其餘預設收合；卡頭為 16px 粗體的「第 N 輪」帶本頁目錄錨點、階段詞、「範圍 N 檔」、findings 條數與各嚴重度的計數籤（為零的不出）；展開後卡內先列範圍路徑（等寬，原樣顯示、不做平台轉換），再列每條 finding：嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、軸向籤（描述以「Correctness」「Standards」「Coherence」加冒號開頭時取出該詞成籤，描述去掉前綴；無前綴時無籤、描述原樣）、路徑（等寬字）與描述原文（不翻譯、不截斷）；描述以結構 token `(accepted)` 結尾時 SHALL 去掉 token 並附「已接受」籤，token 出現於行中時 SHALL 原樣顯示；零 findings 的輪 SHALL 顯示「本輪無發現」。工單 SHALL 於節點出現時載入，並隨外部檔案變更重新載入；載入三態依「詳情頁文件載入以 skeleton 呈現」；載入完成而無工單（不存在、遠端 404、讀取或解析失敗）時 SHALL 顯示空態「工單尚未抵達或已被刪除」而非錯誤，籤槽、數字磚與提示框缺席。工單資料 SHALL 為 `{ rounds: [{ index, phase, patchHash, scope, findings: [{ severity, path, text }] }] }`，欄位名與型別與 CLI `speclink review show --json` 的 `rounds` 相同；本機（含 worktree 覆蓋層）與遠端、活變更與已封存 SHALL 回同一形狀。遠端活變更 SHALL 以 `GET /changes/{name}/review`／`verify` 取得結構化輪次，SHALL NOT 放寬 `artifact cat` 的 artifact 名白名單。看板刷新後對應狀態翻為非進行中時，該節點 SHALL 消失；使用者正停在該文件時 SHALL 切回「提案」。工單文件 SHALL 為唯讀，SHALL NOT 提供蓋章、放棄或追加輪的操作。

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
- **THEN** 「審查」節點自左欄消失，詳情頁切回「提案」文件，身分區章籤改為「已審查」

#### Scenario: 遠端工單 404 顯示空態

- **WHEN** 於 remote 資料源開啟 `inReview` 變更詳情頁並點「審查」節點，server 對 `GET /changes/{name}/review` 回 404
- **THEN** 節點仍在，skeleton 消失後顯示「工單尚未抵達或已被刪除」，無錯誤彈窗，籤槽與數字磚缺席

#### Scenario: 工單追加輪後分頁更新

- **WHEN** 「審查」文件開啟中，外部以 `speclink review add-round` 追加 Round 3
- **THEN** 檔案變更觸發重新載入，籤槽改為「第 3 輪」，數字磚「輪 3」，第 3 輪卡展開、第 2 輪與第 1 輪收合

#### Scenario: 可蓋章提示框與軸向籤

- **WHEN** 開啟 `inReview` 變更的「審查」文件，末輪為 Round 2（`validation`）只有一條 SUGGESTION，描述為「Standards: possible Duplicated Code。讀寫加 try/catch 的寫法…」
- **THEN** tip 提示框標題「可以蓋章」、內文「末輪只剩 1 條 SUGGESTION，不擋章」並點名 `speclink review stamp`；該條 finding 顯示 SUGGESTION 色章、「Standards」軸向籤、等寬路徑與去掉前綴的描述；末輪 CRITICAL 或 WARNING 任一非零時提示框缺席

### Requirement: markdown 內容保留文件結構呈現

<!-- BEFORE: 內容基準字級 16px、任務文字 16px，行高 1.7。 -->

桌面 app 渲染 markdown 內容（變更詳情頁的提案／設計／規格文件、討論詳情頁各文件、已封存檢視）SHALL 保留來源的文件結構：無序清單 SHALL 顯示列表符號、有序清單 SHALL 顯示編號、段落之間 SHALL 有可辨識的垂直間距、來源中的單一換行 SHALL 呈現為換行。詳情頁閱讀卡內（變更、討論、已封存三種詳情頁的內文容器）的內文基準字級 SHALL 為 15px、行高為字級的 1.75 倍，任務文件的任務列文字 SHALL 同為 15px；規格詳情頁與手冊頁的 markdown 內文維持 16px。排版與內容色彩 SHALL 於淺色與深色主題（跟隨系統偏好）一致生效。

#### Scenario: 清單顯示列表符號與編號

- **WHEN** 檢視含無序清單與有序清單的提案文件
- **THEN** 無序清單項目前顯示列表符號、有序清單項目前顯示編號，清單相對內文有縮排

#### Scenario: 單一換行呈現為換行

- **WHEN** 檢視討論記錄的討論過程文件，其中一輪的 Focus 與 Position 行在來源中各佔一行、以單一換行分隔
- **THEN** 兩行在渲染結果中分行呈現，不塌成同一段連續文字

#### Scenario: 閱讀卡內文字級為 15px

<!-- REMOVED-SCENARIO: 內容基準字級為 16px -->

- **WHEN** 檢視變更詳情頁的提案文件與任務文件，再開啟一份規格詳情頁
- **THEN** 閱讀卡內 markdown 內文與任務列文字的計算字級皆為 15px、行高為 15px 的 1.75 倍；規格詳情頁的需求文計算字級為 16px

#### Scenario: 深色主題下排版一致

- **WHEN** 系統偏好為深色時檢視同一份文件
- **THEN** 列表符號、段落間距、字級與淺色主題一致，內容色彩取自深色 token 且可讀

### Requirement: markdown 文件內容行寬有上限

<!-- BEFORE: 所有文件內容固定上限 96ch、容器寬於上限時置中留白。 -->

桌面 app 經共用 markdown 渲染呈現的文件內容 SHALL 有固定行寬上限：詳情頁閱讀卡內（變更、討論、已封存三種詳情頁）的上限 SHALL 為閱讀卡的內容欄寬（卡寬 768px 扣內距），章節標題、數字磚列、提示框、群組卡、輪卡與內文 SHALL 同欄等寬對齊；規格詳情頁與手冊頁的上限維持 96ch，容器寬於上限時內容欄 SHALL 水平置中、留白均分兩側。主區寬度改變時內文行寬 SHALL NOT 隨之增長。超過行寬的表格 SHALL 於容器內橫向捲動，版面 SHALL NOT 橫向溢出。

#### Scenario: 主區加寬時行寬不增長且內容欄置中

- **WHEN** 視窗拉寬至 1600px 檢視變更詳情頁的提案文件
- **THEN** 閱讀卡維持 768px 置中、左右留白均分，內文行寬不超過卡的內容欄，章節標題、數字磚列與內文同欄對齊

#### Scenario: 寬表格於容器內橫捲

- **WHEN** 檢視含超過行寬上限的寬表格的文件
- **THEN** 表格於容器內橫向捲動，頁版面不橫向溢出
