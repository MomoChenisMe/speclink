## ADDED Requirements

### Requirement: 詳情頁的文件樹、閱讀卡與本頁目錄

變更詳情頁、討論詳情頁與已封存詳情頁 SHALL 共用同一個三欄骨架，SHALL NOT 有底部固定列：

- **左欄「文件樹」**（固定 240px、獨立捲動）由上而下為身分區、「文件」節點清單、「狀態」列。身分區 SHALL 呈現名稱或 slug（等寬 14px，過長時折行、SHALL NOT 撐寬欄）、階段籤與審查／驗證章籤、一行出身（建立者首字母圓標＋名字、產生工具、建立日、「開工 YYYY-MM-DD」；缺席的段整段缺席）、關聯列各自成行——變更為「來自」＋來源討論籤（首顆直出、其餘收「+N」浮層）與「規格」＋delta 能力籤（點擊切到規格文件）；討論為 topic（最多三行、溢出截斷）與「關聯」一列（先前討論數，自背景的 `Prior discussions:` 行計數，`none` 時該列缺席；點擊切到背景文件）。建立者與開工者含 email 的完整識別 SHALL 收進主題化提示。「文件」節點 SHALL 每份文件一個，含圖示、名稱與徽章，點擊 SHALL 切換閱讀卡內容，作用中節點 SHALL 以主色淡底標示；節點 SHALL NOT 列出文件內的段落——頁內清單一律由本頁目錄承接。「狀態」列 SHALL 承接原「狀態」卡的內容（左標籤右內容，細線分隔）。
- **中欄閱讀區**：內容欄置中、寬 768px，一張閱讀卡。閱讀卡 SHALL 由上而下為標題區（24px 一般字重文件名、籤槽、同列右端的「開啟 ▾」）、出處行（灰字：變更與已封存變更的文件為該檔案相對變更目錄的路徑，如 `proposal.md`、`specs/<capability>/spec.md`；排程為推算來源「由 speclink plan 推算」；討論與已封存討論的文件 SHALL NOT 顯示出處行）、數字磚列容器（0 到 4 格、磚缺席時整列缺席）、提示框容器、內文。切換文件時閱讀區 SHALL 自頂端呈現；文件載入三態沿用「詳情頁文件載入以 skeleton 呈現」。
- **右欄「本頁目錄」**（200px）SHALL 自閱讀卡渲染後的 DOM 推導目錄：列入帶目錄錨點的標題，點擊 SHALL 捲至該標題，捲動時 SHALL 高亮目前段；沒有錨點時 SHALL NOT 渲染。頁寬 ≥1200px 時 SHALL 顯示，窄於 1200px 時 SHALL 隱藏且閱讀區維持置中。各文件的目錄錨點為其渲染器標出的區段標題：提案與設計的章節標籤、任務的群組標題、規格 delta 的色標區段標頭與需求標題、排程的四卡標題、工單的輪次標題、結論的欄位標籤、討論過程的輪卡頭、背景與衍生變更的區段標題。

變更的文件節點依序 SHALL 為提案、設計、任務（徽章 n/m）、規格（delta 計數徽章）、排程（有 `wave` 時徽章「第 N 波」）、審查、驗證（後兩者的出現條件與「詳情頁的工單分頁」相同，徽章為末輪序號）；討論的文件節點依序 SHALL 為結論、討論過程（徽章為輪數）、背景、衍生變更（徽章為數量），預設文件依「討論詳情頁檢視與轉出變更」；已封存變更為提案、設計、任務、規格與條件式的審查、驗證，已封存討論為結論、討論過程、背景、衍生變更。已封存詳情頁 SHALL 為同一骨架的唯讀變體：身分區帶「已封存」籤與「封存 YYYY-MM-DD」，標題列 SHALL NOT 有動作。

#### Scenario: 三欄骨架與左欄內容

- **WHEN** 使用者於寬 1440px 的視窗開啟 reviewStatus 為 reviewed、verifyStatus 為 verified、來源討論一份、delta 能力一個、9/9 任務的已就緒變更詳情頁
- **THEN** 主區為左欄文件樹、中欄置中 768px 的閱讀卡、右欄本頁目錄，無底部固定列；左欄身分區顯示等寬變更名、「已就緒」「已審查」「已驗證」籤、一行出身、「來自」列的討論籤與「規格」列的能力籤；「文件」節點為提案、設計、任務 9/9、規格、排程、無審查與驗證節點；「狀態」列為任務進度、品質關卡、排程三列

#### Scenario: 點文件節點切換閱讀卡

- **WHEN** 使用者於變更詳情頁點左欄的「任務」節點，再點「規格」節點
- **THEN** 「任務」節點以主色淡底標示且閱讀卡標題為「任務」、內文為任務清單；再點後「規格」節點標示、閱讀卡自頂端呈現規格 delta，左欄捲動位置不變

#### Scenario: 閱讀卡的出處行

- **WHEN** 使用者於變更詳情頁依序看「提案」與「排程」兩份文件，再開啟一筆討論詳情頁
- **THEN** 提案的出處行為 `proposal.md`，排程的出處行為「由 speclink plan 推算」；討論詳情頁的閱讀卡沒有出處行

#### Scenario: 本頁目錄的錨點與高亮

- **WHEN** 使用者於寬 1440px 的視窗開啟含 Why、What Changes、Impact 三個模板章節的提案文件並往下捲到「影響」區段
- **THEN** 右欄本頁目錄列出「為什麼」「變更內容」「影響」三個錨點，捲到「影響」後該錨點高亮；點擊「為什麼」閱讀區捲回該章節

#### Scenario: 窄於 1200px 時無本頁目錄

- **WHEN** 使用者把視窗縮到寬 1100px 再開啟同一份文件
- **THEN** 右欄本頁目錄不顯示，左欄文件樹仍在，閱讀卡維持置中 768px，頁無水平捲軸

#### Scenario: 已封存詳情頁的唯讀變體

- **WHEN** 使用者於已封存頁開啟一筆封存變更的詳情頁
- **THEN** 同一三欄骨架；左欄身分區帶「已封存」籤與「封存 YYYY-MM-DD」；標題列右端只有 ‹ n／m ›、無分析、封存與「⋯」；「文件」節點為提案、設計、任務、規格；任務文件的勾選框不可點

## MODIFIED Requirements

### Requirement: 變更詳情頁的標頭與狀態卡
<!-- BEFORE: 內容欄直向堆疊標題列、出身列、「狀態」卡、分析結果卡、內容卡（分頁列為其標頭）；動作區在標題列同列右端 -->

變更詳情頁 SHALL 採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架：原標題列與出身列的內容 SHALL 落在左欄身分區，原「狀態」卡 SHALL 落在左欄「狀態」列，分析結果（有分析結果時）SHALL 以一張卡呈現於閱讀卡之上。身分區 SHALL 為等寬 14px 變更名（過長折行）緊跟複製名稱鈕，其下為階段籤與章籤：審查或驗證狀態非 `none` 時 SHALL 以狀態籤呈現章籤（審查在前、驗證在後，圖示＋狀態詞，i18n 沿用既有詞條）；蓋章日期與蓋章者完整識別（含 email）SHALL 收進主題化提示，可視文字 SHALL NOT 直出 email 與日期；狀態為 `none` 時不渲染章籤。身分區 SHALL NOT 顯示任務計數——任務進度由「狀態」列與「任務」節點徽章承載。

出身 SHALL 為一行灰字，依序呈現：建立者頭像＋名字、產生工具、建立相對時間、「開工 YYYY-MM-DD」（有開工戳記時）；其下「來自」列呈現來源討論籤（有來源討論時）、「同源」列呈現同源變更籤（有同源變更時）、「規格」列呈現 delta 能力籤（有 delta 時）；缺席的資訊段與缺席的列 SHALL 整段缺席而非留白。建立者與開工者的完整識別（含 email）SHALL 收進主題化提示，可視文字 SHALL NOT 直出 email。出身行於任何資料組合下 SHALL 維持單行、溢出裁切；關聯列各自一行、籤溢出時收「+N」浮層；SHALL NOT 使頁出現水平捲軸。

變更層動作 SHALL 位於主區標題列右端、‹ n／m › 之後並以細線分隔，由左而右為：「分析」（outline）、依派生階段的主要鈕（實心主色：進行中「退回提案中」、已就緒「封存」、remote 可認領時「認領」）、「⋯」選單（outline 圖示鈕；項目與守門依「詳情頁的封存與刪除依階段守門」）；動作鈕 SHALL NOT 折行、SHALL NOT 撐寬標題列。

「狀態」列 SHALL 為三列細線分隔、左標籤右內容：「任務進度」列呈現進度條與 `n/m`；「品質關卡」列呈現各站章籤各帶蓋章日期（`inReview`／`inVerify` 僅狀態詞；兩站皆 `none` 時灰字「尚未經過品質關卡」）；「排程」列於清單項帶 `wave` 時呈現「第 N 波」與前置說明（`blockedBy` 空為「可以開工」、非空為「等 N 項」），缺 `wave` 時沿用排程文件的說明詞條。

#### Scenario: 標頭結構與任務計數位置

- **WHEN** 使用者開啟任一變更的詳情頁
- **THEN** 左欄由上而下為身分區（變更名、籤、出身、關聯列）、「文件」節點、「狀態」列；身分區可視文字不含任務計數，「狀態」列的「任務進度」與「任務」節點徽章呈現任務進度

#### Scenario: 章籤與品質關卡列

- **WHEN** 使用者開啟 reviewStatus 為 reviewed 且 verifyStatus 為 verified、兩站蓋章者皆含 email 的變更詳情頁
- **THEN** 身分區顯示「已審查」「已驗證」兩枚章籤；「狀態」列的「品質關卡」顯示兩枚章籤各帶蓋章日期；將指標停留於身分區章籤時，主題化提示顯示該站蓋章日期與含 email 的完整識別；頁無水平捲軸

#### Scenario: 品質關卡列的空態與進行中

- **WHEN** 使用者分別開啟 reviewStatus 與 verifyStatus 皆為 none、以及 reviewStatus 為 inReview 的變更詳情頁
- **THEN** 前者身分區無章籤、「品質關卡」列顯示「尚未經過品質關卡」；後者身分區與「品質關卡」列顯示「審查中」且無日期

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

### Requirement: 討論詳情頁檢視與轉出變更
<!-- BEFORE: 標題列（slug＋複製＋狀態籤）右端為封存動作，其下為「狀態」卡，分頁為內容卡的標頭 -->

點擊討論卡或細列 SHALL 開啟討論詳情頁，採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架。左欄身分區 SHALL 以討論 slug 為題（等寬 14px）、緊跟複製 slug 鈕與狀態籤（討論中／已結論／已轉出／已轉出・尚無結論／已轉出・保留中，判準與看板討論卡一致），topic SHALL 為其下最多三行、溢出截斷的文字（slug 為題屬 openspec/LANGUAGE.md 明載的受控例外）；出身 SHALL 一行依序呈現建立者頭像＋名字、「N 輪」與建立日，缺席的段整段缺席；其下「關聯」列呈現先前討論數（自背景的 `Prior discussions:` 行計數；`none` 時該列缺席），點擊 SHALL 切到背景文件。主區標題列右端 SHALL 為動作區：concluded 且未封存的討論、以及已轉出且保留在途（hold 為 true）的討論呈現「封存」動作（outline）；保留在途時「封存」SHALL 停用並以提示說明結論保留在途、最後一刀轉出後自動封存；其餘（討論中、已轉出且未保留、已封存）的討論 SHALL NOT 呈現封存動作。

左欄「狀態」列 SHALL 為：「階段」列為步驟列「討論中 → 已結論 → 已轉出 → 已封存」且現站可辨識（promoted 為已轉出、concluded 為已結論、其餘為討論中；已封存站保留給已封存討論頁）；hold 為 true 時 SHALL 另有「保留在途」列呈現說明；promoted_to 非空時 SHALL 另有「衍生變更」列，每個子變更一顆可點等寬籤並附階段籤（階段派生與配色同「討論於看板第 0 欄兩級呈現」的 chip 矩陣），點籤 SHALL 開啟該變更的詳情頁；無衍生變更時該列缺席。

左欄「文件」節點依序 SHALL 為：結論、討論過程 N、背景、衍生變更 N——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯，此時節點為「記錄」與「衍生變更」）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」，結論為空時預設「背景」。衍生變更文件 SHALL 列出各子變更現況與跳轉，且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。concluded 且未封存討論的封存動詞 SHALL 於討論卡與討論詳情頁皆可及，兩處 SHALL 走同一確認流程，經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情頁 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 頁標題以 slug 為題且可複製

- **WHEN** 使用者開啟任一討論的詳情頁並點按左欄身分區 slug 旁的複製鈕
- **THEN** 身分區呈現該討論的 slug（等寬字型）、其下呈現 topic；slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的詳情頁
- **THEN** 左欄「文件」節點為 結論／討論過程 N／背景／衍生變更 N，且預設閱讀卡呈現結論內容；「狀態」列的階段以「已結論」為現站

#### Scenario: 狀態卡的保留與衍生列

- **WHEN** 使用者開啟一筆已轉出、已有結論、hold 為 true、promoted_to 含一個已封存與一個進行中子變更的討論詳情頁
- **THEN** 身分區狀態籤為「已轉出・保留中」；「狀態」列的階段現站為「已轉出」；「保留在途」列呈現說明；「衍生變更」列兩顆籤分別附「已封存」與「進行中」階段籤，點進行中那顆開啟該變更的詳情頁；標題列右端的「封存」停用並有提示

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的衍生變更文件
- **THEN** 閱讀卡列出各子變更現況與跳轉按鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論詳情頁或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情頁
- **THEN** 左欄身分區的「來自」列顯示來源討論的 slug 籤（topic 於提示呈現）、「同源」列顯示同源變更籤，點擊同源籤開啟該變更的詳情頁

#### Scenario: 頁內封存 concluded 討論

- **WHEN** 使用者開啟一筆已結論、未封存且無 hold 討論的詳情頁，點標題列右端的「封存」並於確認對話框確認
- **THEN** 該討論移入封存：自討論欄消失、出現於已封存頁討論節，與自討論卡封存的效果一致；詳情頁關閉回看板

### Requirement: 詳情頁的文件動作
<!-- BEFORE: 「開啟 ▾」在內容卡標頭（分頁列）右端 -->

變更詳情頁與討論詳情頁的閱讀卡標題列右端 SHALL 提供「開啟 ▾」選單（outline 樣式、與標題同列），對目前文件對應的檔案提供：以預設 App 開啟、在 Finder 顯示（標籤隨平台：Finder／檔案總管／檔案管理員）、複製路徑；選單首列 SHALL 以等寬灰字顯示該檔案相對所在 checkout 根的路徑。文件與檔案的對應 SHALL 為：提案 `openspec/changes/<name>/proposal.md`、設計 `design.md`、任務 `tasks.md`、規格 `specs/<capability>/spec.md`（目前選定的 capability）、審查 `review.md`、驗證 `verify.md`；討論各文件皆為該討論的記錄檔。檔案的實際位置 SHALL 與詳情頁讀取內容的位置相同：綁定 worktree 的變更指向 worktree 副本、spec 目錄名依專案設定、已封存的討論指向 `openspec/discussions/archive/<封存日>-<slug>.md`。沒有對應檔案的文件（排程）SHALL NOT 顯示選單。remote 專案無本機 checkout 時選單 SHALL 只有「複製內容」（該文件已載入的原文；未載完時項目停用）且首列路徑缺席；工單文件（審查、驗證）不載原文，此時 SHALL NOT 顯示選單。「以預設 App 開啟」SHALL 只開啟 markdown 文件：傳入的檔名與符號連結的實際指向皆須為 markdown。動作失敗時 SHALL 以既有失敗 toast 語意呈現錯誤，SHALL NOT 彈出錯誤視窗。「開啟 ▾」SHALL NOT 出現在主區標題列，也 SHALL NOT 收進「⋯」選單。

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

### Requirement: 詳情頁標頭標記受寬度約束且頁不產生水平捲軸
<!-- BEFORE: 約束套在內容欄的標頭標記與動作列 -->

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

### Requirement: 變更的來源討論多值呈現
<!-- BEFORE: 詳情頁的來源討論籤在標頭出身列 -->

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

### Requirement: 詳情頁文件載入以 skeleton 呈現
<!-- BEFORE: 以「分頁」稱呼文件；無文件切換中的即時回饋 -->

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

### Requirement: 詳情頁的排程分頁
<!-- BEFORE: 排程為內容卡的「排程」分頁，位於「規格」分頁之後 -->

change 詳情頁 SHALL 於左欄「文件」節點「規格」之後提供「排程」節點（tw 標籤「排程」、en「Plan」），點擊 SHALL 於閱讀卡呈現排程文件。清單項帶 `wave` 時閱讀卡 SHALL 依序呈現四張卡片（與審查／驗證文件同一種邊框卡片：標題列與內容區以分隔線分格），data-plan-section 依序為 wave、depends、overlaps、archive：波次卡（「第 N 波」＋同波其他變更名稱清單，無其他時顯示「本波只有這個變更」）；前置卡（標題列右側 SHALL 有狀態徽章——`blockedBy` 空時「可以開工」、非空時「等 N 項」且 title 列出前置名；內容為 dependsOn 每項一列，資料源 capability `setDepends` 為 true 時每列附移除鈕、底部附下拉選單列出候選變更供新增，選擇即寫入；空時「尚無前置」；`blockedBy` 含 `dependsOn` 以外的名稱（舊 server 以目錄級重疊判定阻擋）時 SHALL 另以一列唯讀列出這些名稱（tw「另需等待（server 依重疊判定）：」，不附移除鈕），讓徽章數字對得上卡片內容）；重疊卡（`requirementOverlap` 每項一列：對方變更名、`capability › requirement` 文字、雙方操作各一枚小標籤（用詞與配色同規格文件的 delta 區段，如 MODIFIED 顯示「修改」；title 為該操作所屬的變更名），`conflict` 為 true 時加「同名衝突」標籤；空時「無重疊」；SHALL NOT 顯示目錄級 `overlaps`）；封存順序卡（`archiveAfter` 每項變更名，下方一句「先封存它們，再對照正式規格重寫同名 requirement 後封存本變更」；其中有變更也把本變更列入它的 `archiveAfter`（互相等待）時 SHALL 另加一句警示（tw「與 {names} 互相等待：兩邊都要先等對方封存，須調整拆分」）；`requirementOverlap` 有 `conflict` 為 true 的列時 SHALL 加一句同名衝突警示（tw「與 {names} 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」）；`archiveAfter` 為空且無同名衝突時「可直接封存」）。排程文件 SHALL NOT 渲染獨立的「阻擋」段。新增候選 SHALL 為該變更所在名冊中的其他作用中變更、扣除已宣告的前置：資料源提供名冊查詢時（local）SHALL 以查詢結果為準——該變更有 worktree 映射時名冊為其 worktree 副本、無映射時為主 checkout，與前置寫入作用的 store 相同；名冊查詢未完成時 SHALL NOT 渲染新增下拉；資料源未提供名冊查詢（remote，scope 只有一份名冊）或首次查詢（開啟詳情頁、切換變更）失敗時 SHALL 自作用中變更清單派生；看板刷新後的重新查詢失敗時 SHALL 沿用前一次的查詢結果。新增或移除前置 SHALL 經資料源 `setDepends` 寫回該變更 meta 的 depends_on，成功後看板與排程文件刷新；引擎拒絕（自依賴、不存在、已封存、成環）時 SHALL 以單行錯誤呈現引擎訊息、畫面不變。capability 為 false 時 SHALL NOT 渲染移除鈕與下拉選單。清單項缺 `wave` 且缺 `dependsOn`（remote 清單未取得 plan：舊 server、依賴成環或請求失敗）時閱讀卡 SHALL 只顯示一句說明（tw「此模式尚未提供排程資訊」），不渲染四卡；缺 `wave` 但帶 `dependsOn`（local plan 成環）時 SHALL 顯示一句說明（tw「依賴成環，先移除一條前置才有排程資訊」）並只渲染前置卡（無狀態徽章；`dependsOn` 為空時照樣顯示「尚無前置」；capability 為 true 時附移除鈕、SHALL NOT 渲染新增下拉），讓使用者解環。`requirementOverlap` 與 `archiveAfter` 缺席但 `wave` 存在（舊 server）時重疊卡與封存順序卡 SHALL 顯示各自的空態文案。判定 SHALL 收斂於 packages/ui 階段派生模組的 planWave／planBlockedBy／planRequirementOverlap／planArchiveAfter 單一入口。

#### Scenario: 排程分頁四段

- **WHEN** 開啟 wave=2、dependsOn=["add-a"]、blockedBy=["add-a"]、requirementOverlap=[{change:"add-b",capability:"desktop-app",requirement:"看板與任務",ownOperation:"MODIFIED",otherOperation:"MODIFIED",conflict:false}]、archiveAfter=["add-b"] 的變更詳情頁並點左欄「排程」節點
- **THEN** 閱讀卡依序顯示波次卡「第 2 波」、前置卡（徽章「等 1 項」、前置列 add-a 附移除鈕、新增下拉）、重疊卡一列（add-b、`desktop-app › 看板與任務`、兩枚「修改」標籤）、封存順序卡（add-b 與提示句），頁面無 data-plan-section="blocked"

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

### Requirement: 詳情頁的工單分頁
<!-- BEFORE: 工單為內容卡的條件式分頁，位於「排程」分頁之後 -->

change 詳情頁 SHALL 於左欄「文件」節點「排程」之後提供兩個條件式節點：清單項 `reviewStatus` 為 `inReview` 時 SHALL 出現「審查」節點（tw「審查」、en「Review」），`verifyStatus` 為 `inVerify` 時 SHALL 出現「驗證」節點（tw「驗證」、en「Verify」），審查節點在前；兩者皆非時 SHALL NOT 渲染這兩個節點，文件節點維持提案／設計／任務／規格／排程。工單文件的閱讀卡 SHALL 為該站工單的結構化呈現：段標題含站名、「第 N 輪」（N 為末輪序號）、末輪階段詞與末輪三級計數（CRITICAL／WARNING／SUGGESTION 各自的條數，標籤維持英文）；輪次依序號升序逐輪可收合，末輪預設展開、其餘預設收合；每輪標題含「第 N 輪」、階段詞（`phase` 為 `discovery` 顯示 tw「首輪」en「First pass」、`validation` 顯示 tw「複驗」en「Re-check」、`null` 不顯示階段詞）、「範圍 N 檔」（展開後列出 scope 路徑，原樣顯示、不做平台轉換）與 findings 條數；每條 finding 一列，含嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、路徑（等寬字）與描述原文（不翻譯、不截斷）；描述以結構 token `(accepted)` 結尾時 SHALL 去掉 token 並附「已接受」籤，token 出現於行中時 SHALL 原樣顯示；零 findings 的輪 SHALL 顯示「本輪無發現」。工單 SHALL 於節點出現時載入，並隨外部檔案變更重新載入；載入三態依「詳情頁文件載入以 skeleton 呈現」；載入完成而無工單（不存在、遠端 404、讀取或解析失敗）時 SHALL 顯示空態「工單尚未抵達或已被刪除」而非錯誤。工單資料 SHALL 為 `{ rounds: [{ index, phase, patchHash, scope, findings: [{ severity, path, text }] }] }`，欄位名與型別與 CLI `speclink review show --json` 的 `rounds` 相同；本機（含 worktree 覆蓋層）與遠端、活變更與已封存 SHALL 回同一形狀。遠端活變更 SHALL 以 `GET /changes/{name}/review`／`verify` 取得結構化輪次，SHALL NOT 放寬 `artifact cat` 的 artifact 名白名單。看板刷新後對應狀態翻為非進行中時，該節點 SHALL 消失；使用者正停在該文件時 SHALL 切回「提案」。工單文件 SHALL 為唯讀，SHALL NOT 提供蓋章、放棄或追加輪的操作。

#### Scenario: 審查工單分頁出現並結構化呈現

- **WHEN** 開啟 `reviewStatus: "inReview"`、`verifyStatus: "none"` 的變更詳情頁，工單含 Round 1（`phase: "discovery"`，scope 3 檔，findings 兩條 WARNING）與 Round 2（`phase: "validation"`，scope 3 檔，findings 一條 WARNING 描述結尾 `(accepted)`、一條 SUGGESTION）
- **THEN** 左欄文件節點為提案／設計／任務／規格／排程／審查，無「驗證」節點；點「審查」後段標題顯示「審查 · 第 2 輪（複驗）CRITICAL 0 · WARNING 1 · SUGGESTION 1」；第 2 輪展開列兩條 finding，WARNING 那條附「已接受」籤且描述不含 `(accepted)`；第 1 輪收合、標題顯示「第 1 輪 · 首輪 · 範圍 3 檔 · 2 條」

##### Example: 階段詞與 accepted token

| round.phase | 顯示階段詞（tw） | finding.text | 顯示描述 | 已接受籤 |
| ----------- | ---------------- | ------------ | -------- | -------- |
| discovery | 首輪 | Correctness: 未處理空清單 | Correctness: 未處理空清單 | 無 |
| validation | 複驗 | Standards: 命名不一致 (accepted) | Standards: 命名不一致 | 有 |
| null | （不顯示） | Standards: (accepted) 語意不清 | Standards: (accepted) 語意不清 | 無 |

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
- **THEN** 節點仍在，skeleton 消失後顯示「工單尚未抵達或已被刪除」，無錯誤彈窗

#### Scenario: 工單追加輪後分頁更新

- **WHEN** 「審查」文件開啟中，外部以 `speclink review add-round` 追加 Round 3
- **THEN** 檔案變更觸發重新載入，段標題改為「第 3 輪」，第 3 輪展開、第 2 輪與第 1 輪收合

### Requirement: 已封存項目以詳情頁檢視
<!-- BEFORE: 已封存詳情頁與變更／討論詳情頁同骨架（標題列、出身列、「狀態」卡、內容卡標頭分頁） -->

已封存頁的變更列與討論列 SHALL 以列式清單呈現，點擊列 SHALL 開啟唯讀的已封存詳情頁（宿主維持已封存頁），SHALL NOT 提供行內展開。已封存詳情頁 SHALL 與變更／討論詳情頁同骨架（「詳情頁的文件樹、閱讀卡與本頁目錄」：左欄身分區、文件節點、「狀態」列，中欄閱讀卡，右欄本頁目錄）且唯讀——SHALL NOT 提供任務勾選、動詞執行或任何寫入操作；身分區 SHALL NOT 呈現進度條，標題列 SHALL NOT 有動作區——封存是唯讀定格；開啟期間內容 SHALL 隨外部檔案變更反映；所請求的文件不存在時對應閱讀卡 SHALL 顯示空狀態而非錯誤。

封存變更的詳情頁 SHALL 至少含提案、設計、任務、規格文件節點（內容來自封存目錄的實體文件），閱讀卡標題列右端提供「開啟 ▾」（對應 `openspec/changes/archive/<封存目錄名>/` 下的檔案）；任務文件的勾選框不可點擊、無拖曳把手、無批次工具列、整列不回應點擊。身分區 SHALL 為變更名（等寬）緊跟複製鈕（複製含日期前綴的封存目錄名，複製後 SHALL 有已複製回饋）與「已封存」籤及審查／驗證結局籤；出身 SHALL 一行依序呈現建立者（首字母圓標＋名字，完整識別收 hover 提示）、建立日期、「封存 YYYY-MM-DD」，其下「來自」列呈現來源討論籤（有來源討論時；點擊 SHALL 開啟該討論的已封存詳情頁，記錄以 live 優先、封存後備載入），出身行恆定單行、溢出裁切，任一欄位資料缺席時該欄位 SHALL 缺席、其餘照常。「狀態」列 SHALL 為三列：任務 n/m（無 tasks.md 時「—」）、品質關卡結局（皆無時「未經品質關卡」）、封存日期。

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
- **THEN** 剪貼簿寫入含日期前綴的封存目錄名並顯示已複製回饋；身分區帶「已封存」與「已審查」籤；出身行顯示建立者首字母圓標＋名字、建立日期與「封存 YYYY-MM-DD」；「狀態」列三列為「20/21」、「已審查」籤、封存日期；無進度條，標題列無動作區

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

### Requirement: 已封存詳情頁的工單分頁
<!-- BEFORE: 工單為已封存詳情頁內容卡的條件式分頁 -->

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
