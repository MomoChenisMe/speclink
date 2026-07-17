## MODIFIED Requirements

### Requirement: 看板全域操作成功靜默、失敗以 toast 浮層呈現

桌面 app 的看板全域操作——刪除變更、封存變更、封存討論、拖排卡片、開啟專案、初始化專案——成功時 SHALL NOT 呈現任何文字結果訊息；操作結果由畫面自身表達（卡片自看板消失、側欄「已封存」計數遞增、卡片位於新位置、切換至該專案）。視窗頂欄 SHALL NOT 呈現任何操作結果文字。

上述任一操作失敗時，app SHALL 以 toast 浮層呈現失敗訊息。訊息 SHALL 包含操作對象的主詞與 core 回報的原始錯誤內容，SHALL NOT 吞掉或改寫該錯誤內容。toast SHALL 疊於詳情抽屜與確認對話框的遮罩之上——抽屜或確認框開啟中發生失敗時，toast SHALL 完整可見且不被遮蔽。toast SHALL 於固定時距後自動消失，並 SHALL 提供關閉鈕供提前關閉。同時 SHALL 僅呈現一則 toast；新訊息 SHALL 取代前一則並重置其自動消失時距。

失敗後看板 SHALL 刷新回磁碟現況，SHALL NOT 保留未落檔的假象順序或狀態。

封存變更成功時 SHALL 關閉詳情抽屜。成功之所以能靜默，前提是畫面可見地表達了結果；詳情抽屜開啟時遮蔽看板，該前提即不成立。

#### Scenario: 抽屜開啟中封存失敗仍完整可見

- **WHEN** 使用者於某 change 的詳情抽屜點按「封存」，而封存因前置未滿足失敗
- **THEN** toast 疊於抽屜遮罩之上完整呈現，內容含該 change 名稱與 core 回報的錯誤訊息；詳情抽屜維持開啟

#### Scenario: 刪除成功不呈現任何文字訊息

- **WHEN** 使用者確認刪除某變更且刪除成功
- **THEN** 該卡片自看板消失、詳情抽屜關閉；視窗頂欄與 toast 皆無任何文字訊息

#### Scenario: 封存成功關閉詳情抽屜且不呈現文字訊息

- **WHEN** 使用者於某 change 的詳情抽屜點按「封存」且封存成功
- **THEN** 詳情抽屜關閉、卡片自看板消失、側欄「已封存」計數遞增；無任何文字訊息

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
| 刪除變更 | 靜默：卡片消失、抽屜關閉 | toast：`<變更名稱> · 刪除失敗 ✗ <core 錯誤>` |
| 封存變更 | 靜默：卡片消失、「已封存」計數遞增、抽屜關閉 | toast：`<變更名稱> · 封存失敗 ✗ <core 錯誤>` |
| 封存討論 | 靜默：卡片消失、抽屜關閉 | toast：`<討論 slug> · 討論封存失敗 ✗ <core 錯誤>` |
| 拖排卡片 | 靜默：卡片位於新位置 | toast：`<卡片識別> · 排序寫回失敗 ✗ <core 錯誤>` |
| 開啟專案 | 靜默：切換至該專案 | toast：`<選定路徑> · 開啟專案失敗 ✗ <core 錯誤>` |
| 初始化專案 | 靜默：進入該專案 | toast：`<選定目錄> · 初始化失敗 ✗ <core 錯誤>` |

### Requirement: 桌面 app 提供動詞操作面

<!-- BEFORE: 分析面板規範之末句為「視窗頂列狀態列 SHALL 保留供看板全域操作（刪除、封存、拖排失敗）之結果訊息」 -->

桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

詳情抽屜的動作列 SHALL 以單一「分析」按鈕同時觸發 validate 與 analyze，SHALL NOT 提供獨立的「驗證」按鈕；結果呈現於該 change 詳情抽屜內的分析面板。分析面板 SHALL 依序呈現：

- 結構驗證列：validate 通過時呈單列通過標示（成功語意配色）；失敗時呈錯誤數並逐條列出錯誤訊息（與 speclink validate 的輸出一致）。
- 維度摘要卡：Coverage、Consistency、Ambiguity、Gaps 四維度各一張摘要卡，維度名以繁體中文呈現（覆蓋度、一致性、模糊度、缺漏），卡上顯示該維度發現數——零發現呈「無問題」（成功語意配色）、非零呈「N 個問題」（警示語意配色）。
- 發現卡：逐條發現各一張卡，呈現嚴重度徽章、來源檔（location）、摘要（summary）與建議行（recommendation），對應 speclink analyze 的 --json 輸出欄位。

分析面板 SHALL 可關閉：再次點按「分析」按鈕或面板的關閉鈕皆 SHALL 收合面板；收合後再點按「分析」SHALL 重新執行並展開。面板狀態 SHALL NOT 跨 change 沿用（切換 change 即清空）。視窗頂欄 SHALL NOT 承載任何操作結果訊息面；看板全域操作的結果回饋 SHALL 依「看板全域操作成功靜默、失敗以 toast 浮層呈現」需求所定語意呈現。

#### Scenario: 分析一鍵呈現驗證列與四維度發現卡

- **WHEN** 使用者於某 change 的詳情抽屜點按「分析」
- **THEN** 抽屜內展開分析面板：頂部結構驗證列（通過或錯誤數）、四張繁體中文維度摘要卡（各帶發現數）、逐條發現卡（嚴重度徽章、來源檔、摘要、建議行），內容對應 speclink validate 與 speclink analyze 的 --json 輸出

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
