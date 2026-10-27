## MODIFIED Requirements

### Requirement: 介面狀態語意色分層

<!-- BEFORE: 內文以「品質站」稱呼品質關卡 -->

桌面 app 的狀態呈現 SHALL 依語意上色:進行中(分頁還原、遷移進行、更新下載、看板卡片 worktree 標示)為藍系;成功結果(遷移完成)為綠系;警示(登入過期 stale、needs-reauth、政策衝突、待重新反映)為琥珀系;錯誤(分頁 error、更新檢查失敗、設定檔解析錯誤、遠端工作區 access-denied/not-found/unknown)與危險動作為紅系;待手動(看板「待手動」章、任務文件的手動任務列淡底與徽章、手動任務提示框)為暖黃系——暖黃與警示琥珀是兩個色相,待手動 SHALL NOT 以琥珀或藍呈現。主題主色 SHALL NOT 用於表達狀態;錯誤 SHALL NOT 以琥珀呈現;靜態小標與裝飾面 SHALL 為中性。

生命週期與討論 SHALL 各有一個色相,由單一來源供給並為看板與系統匣面板共用:提案中為主色(teal)、進行中為藍系(與進行中語意同色)、已就緒為綠系(與成功語意同色)、討論為桃紫系(fuchsia);看板欄頂條、欄標頭圖示、欄計數徽章、卡片波次章與系統匣面板的生命週期分區、討論分區的圖示與計數徽章 SHALL 取所在階段或分區的色相;「已轉出」分區、收合列及其計數 SHALL 為中性。琥珀、紅與紫色系 SHALL NOT 用於任何生命週期或討論的欄位與分區。豁免項 SHALL 維持主色:複製成功勾號(互動即時回饋)、可點連結籤與導航元素。

色值 SHALL 只存在於共用 theme 樣式表(基底 token、狀態語意 token、品質關卡蓋章 token、改進標示 token、生命週期 token),淺色與深色各一套;集中常數檔 SHALL 只保存「狀態→token class」的對照;元件原始碼 SHALL 只以 token class 取色。原生色階字面(含語意色階與中性色階)SHALL NOT 出現在共用元件庫、桌面 app 與 server web 的任何元件或常數檔原始碼;主題守門測試 SHALL 於違規時失敗並指名檔案與 class,SHALL NOT 設白名單。

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

### Requirement: 看板卡片的驗證標示

<!-- BEFORE: 內文以「品質站」稱呼品質關卡 -->

change 卡片 SHALL 依驗證狀態渲染第二顆行內小章，與審查章並排且順序固定（審查章在前、驗證章在後）：`none` 無標示；`inVerify` 顯示「驗證中」章；`verified` 顯示「已驗證」章；`verifiedStale` 顯示降級樣式章。tooltip 文案取自 i18n 詞條——tw 使用正典詞（驗證中／已驗證／已驗證·其後有變動），en 使用對應詞條。驗證章各狀態的配色 SHALL 與審查章對應狀態同值（色承載狀態、圖示形狀承載站別），SHALL NOT 另建配色；驗證章圖示 SHALL 採盾牌形系，與審查章的徽章形系可區辨。

#### Scenario: 兩章並排

- **WHEN** 某 change 的協定資料為 `reviewStatus: "reviewed"` 且 `verifyStatus: "verified"`
- **THEN** 卡片依序顯示審查章與驗證章，各自 tooltip 正確

#### Scenario: 兩站同狀態同色、異形可辨

- **WHEN** 某 change 同時為 `reviewStatus: "reviewed"` 與 `verifyStatus: "verified"`
- **THEN** 兩章配色同值（皆為品質關卡蓋章色），審查章為徽章形、驗證章為盾牌形，站別可一眼區辨

#### Scenario: 僅驗證章

- **WHEN** 協定資料為 `reviewStatus: "none"` 且 `verifyStatus: "verifiedStale"`
- **THEN** 卡片僅顯示降級樣式的驗證章，tooltip 為「已驗證·其後有變動」（tw）

## REMOVED Requirements

### Requirement: 品質站蓋章配色與主色分離

**Reason**: 需求名與內文的「品質站」是 LANGUAGE.md「品質關卡」的 avoid 同義詞

**Migration**: 由「品質關卡蓋章配色與主色分離」承接全部條款，只把「品質站」改為「品質關卡」，行為不變

## ADDED Requirements

### Requirement: 品質關卡蓋章配色與主色分離

品質關卡狀態標示的配色 SHALL 與主題主色分離:「已審查」蓋章 SHALL 以紫色系呈現,SHALL NOT 引用主題主色 token;「審查中」「已審查·其後有變動」「曾審查未通過」三態維持藍/琥珀/紅系不變。紫色系 SHALL 為品質關卡蓋章專屬——品質關卡標示以外的介面元素 SHALL NOT 使用紫色系。此約束 SHALL 套用於呈現審查標示的全部表面——「已審查」蓋章出現於看板卡片、變更詳情頁、已封存清單、已封存詳情頁四處;封存對話框共用同一份配色對照,但僅取用「曾審查未通過」,不呈現已審查蓋章。各表面 SHALL 引用同一份配色對照,SHALL NOT 各自定義。

#### Scenario: 已審查蓋章與主色可區辨

- **WHEN** 使用者開啟 reviewStatus 為 reviewed 的變更卡片與其詳情頁
- **THEN** 「已審查」蓋章以紫色系呈現,與同視野的主色元素(進度條、分頁、連結籤)可區辨;深淺主題下皆然

#### Scenario: 其餘三態配色不變

- **WHEN** 使用者分別檢視 inReview、reviewedStale、reviewedNotPassed 的審查標示
- **THEN** 三態仍以藍、琥珀、紅系呈現,詞條與圖示與本變更前一致

#### Scenario: 紫色系為蓋章專屬

- **WHEN** 使用者檢視看板、詳情頁與已封存各表面
- **THEN** 紫色系僅出現於品質關卡蓋章,不出現在其他介面元素
