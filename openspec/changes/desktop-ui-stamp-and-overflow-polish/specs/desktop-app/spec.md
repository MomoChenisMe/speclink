## ADDED Requirements

### Requirement: 主題化提示統一延遲

共用 UI 元件庫的主題化提示 SHALL 採單一共用延遲預設：指標停留達 300ms 顯示提示；個別介面 SHALL NOT 另行覆寫延遲值。此預設 SHALL 同源套用於 desktop 與 server-web console（兩者共用同一 UI 元件庫）。於多個觸發點間連續移動時，後續提示 SHALL 立即顯示（沿用元件庫既有 skipDelay 行為）。系統匣面板的原生 title 提示不屬主題化提示，SHALL 維持原生行為、不受本延遲約束。

#### Scenario: 卡片章提示於統一延遲後顯示

- **WHEN** 使用者將指標停留於看板卡片的審查章達 300ms
- **THEN** 主題化提示顯示狀態詞（tw 為正典詞「已審查」等）

#### Scenario: 詳情抽屜與卡片同一延遲

- **WHEN** 使用者分別將指標停留於變更詳情抽屜的章籤與看板卡片的章
- **THEN** 兩處提示皆於停留達 300ms 時顯示，無任一處立即顯示或採用其他延遲值

### Requirement: 指令檔過期提示捲動釘選

指令檔過期提示 SHALL 於其分頁內容捲動時釘選於可視區頂部維持可見；釘選時提示 SHALL 以不透明底呈現，下層捲過的內容 SHALL NOT 透出。未捲動時版面 SHALL 與提示原位呈現一致，不因釘選機制改變原始位置與間距。

#### Scenario: 捲動時提示保持可見

- **WHEN** 專案設定頁顯示過期提示且使用者向下捲動頁面
- **THEN** 提示固定於可視區頂部持續可見，底色不透明、下層內容不透出，頁面其餘內容自提示下方捲過

## MODIFIED Requirements

### Requirement: 變更詳情抽屜標頭的四層結構

<!-- BEFORE: 狀態列於審查／驗證資訊直出日期與含 email 的作者身分且不可壓縮，超寬時被抽屜水平裁切 -->

變更詳情抽屜的標頭 SHALL 由上而下為四層固定結構:標題列(變更名+複製名稱鈕)、狀態列、出身列、動作列。狀態列 SHALL 呈現進度條與完成百分比;審查或驗證狀態非 `none` 時,同列 SHALL 呈現對應站章籤(圖示+狀態詞,審查章在前、驗證章在後,i18n 沿用既有詞條);蓋章日期與蓋章者完整識別(含 email)SHALL 收進主題化提示,狀態列可視文字 SHALL NOT 直出 email 與日期。狀態列於任何資料組合下 SHALL 維持單行,SHALL NOT 被水平裁切、SHALL NOT 撐寬抽屜。標頭 SHALL NOT 顯示任務計數字樣——任務數由任務分頁徽章與進度條承載。出身列 SHALL 為單行,依序呈現:建立者頭像+名字、產生工具、建立相對時間、開工日期(有開工戳記時)、「來自」與來源討論籤(有來源討論時)、「同源」與同源變更籤(有同源變更時);缺席的資訊段 SHALL 整段缺席而非留白。建立者與開工者的完整識別(含 email)SHALL 收進主題化提示,標頭可視文字 SHALL NOT 直出 email。出身列於任何資料組合下 SHALL 維持單行,SHALL NOT 折行,SHALL NOT 撐寬抽屜。

#### Scenario: 四層結構與任務計數缺席

- **WHEN** 使用者開啟任一變更的詳情抽屜
- **THEN** 標頭由上而下依序為標題列、狀態列、出身列、動作列;標頭可視文字不含任務計數字樣,任務分頁徽章與進度條仍呈現任務進度

#### Scenario: 狀態列章籤與提示

- **WHEN** 使用者開啟 reviewStatus 為 reviewed 且 verifyStatus 為 verified、兩站蓋章者皆含 email 的變更詳情抽屜
- **THEN** 狀態列顯示進度條、完成百分比與「已審查」「已驗證」兩枚章籤;將指標停留於章籤時,主題化提示顯示該站蓋章日期與含 email 的完整識別;狀態列維持單行、無水平裁切,抽屜無水平捲軸

#### Scenario: email 收進提示

- **WHEN** 使用者開啟 created_by 含 email 的變更詳情抽屜,並將指標停留於建立者名字
- **THEN** 標頭可視文字僅顯示頭像與名字,主題化提示呈現含 email 的完整識別;開工者的完整識別亦僅於提示呈現

#### Scenario: 出身列恆定單行

- **WHEN** 使用者開啟一個有 4 份來源討論、有開工戳記且有同源變更的變更詳情抽屜
- **THEN** 出身列維持單行——直接顯示出身討論籤與「+N」籤,不折行,抽屜無水平捲軸

### Requirement: 看板卡片統一解剖學

<!-- BEFORE: 截斷處以尾端漸層淡出呈現、SHALL NOT 以省略號收尾（card-name-single-line-fade，2026-08-04）——本次經使用者裁定刻意翻案 -->

看板全尺寸卡（變更卡與討論卡）SHALL 採統一三列骨架：識別列（標題＋複製鈕＋右端 meta icons）、描述列（一行截斷，無內容時缺席）、meta 列。標題 SHALL 以等寬字型呈現（變更名稱與討論 slug 同為可複製把手）。標題 SHALL 恆為單行——長於可用寬度時 SHALL 就地截斷，SHALL NOT 折行、SHALL NOT 強制斷字；截斷處 SHALL 以省略號收尾，SHALL NOT 以漸層淡出或硬切呈現；標題未被截斷時 SHALL 完整顯示，SHALL NOT 出現省略號。複製鈕 SHALL 與標題同列尾隨於標題文字之後，SHALL NOT 因標題過長而落至次行、SHALL NOT 被壓縮，meta icons 維持靠右；SHALL NOT 將複製鈕推至列右緣。變更卡描述列 SHALL 顯示 proposal Why 首句（一行截斷）；proposal 缺席、Why 區段缺席或為空時描述列 SHALL 缺席。描述資料 SHALL 由變更清單 payload 一次帶出，SHALL NOT 逐卡讀取 proposal 全文；該欄位屬呈現層輔助欄位，不屬 CLI --json 對齊範圍。建立者 SHALL 以頭像圓點呈現且 hover 顯示全名，SHALL NOT 於卡面直出全名文字；createdBy 缺席時圓點缺席。狀態 chip SHALL 僅在所在位置無法表達狀態時出現：討論卡（討論欄一欄兩態）帶狀態 chip，變更卡（所在欄即階段）SHALL NOT 帶狀態 chip。

#### Scenario: 變更卡三列骨架

- **WHEN** 看板載入一個 proposal Why 首句非空、createdBy 存在、任務 5/21 的變更
- **THEN** 變更卡識別列以等寬字型顯示名稱且複製鈕緊跟名稱文字後（hover 顯現、點擊寫入名稱至剪貼簿且不開詳情抽屜）、右端呈建立者圓點；描述列一行截斷顯示 Why 首句；meta 列顯示進度條與 5/21；卡上無狀態 chip

#### Scenario: 變更卡無 Why 內容時描述列缺席

- **WHEN** 某變更無 proposal.md（或 Why 區段為空）
- **THEN** 該變更卡不顯示描述列，識別列與 meta 列照常呈現，看板不因該筆缺件而報錯

#### Scenario: 長標題單行截斷時複製鈕仍在同列

- **WHEN** 變更名稱長於卡片可用寬度
- **THEN** 標題維持單行並於可用寬度處截斷、以省略號收尾，複製鈕緊跟標題文字之後留在同一列且維持可點，右端 meta icons 不被擠出卡外

#### Scenario: 討論卡長 slug 與變更卡同一收尾

- **WHEN** 討論 slug 長於卡片可用寬度
- **THEN** slug 維持單行截斷並以省略號收尾、不強制斷字折行，複製 slug 鈕留在同一列，收尾行為與變更卡標題一致

<!-- REMOVED-SCENARIO: 短標題不套淡出 -->

#### Scenario: 短標題完整顯示

- **WHEN** 變更名稱短於卡片可用寬度
- **THEN** 標題完整顯示且不出現省略號，複製鈕緊跟標題最後一個字元後
