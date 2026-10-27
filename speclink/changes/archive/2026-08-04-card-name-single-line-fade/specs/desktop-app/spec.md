## MODIFIED Requirements

### Requirement: 看板卡片統一解剖學

<!-- BEFORE: 標題不截斷、過長時自然折行，複製鈕行內尾隨於標題末行最後一個字元後 -->
<!-- REMOVED-SCENARIO: 長標題折行時複製鈕仍緊跟末字元 -->

看板全尺寸卡（變更卡與討論卡）SHALL 採統一三列骨架：識別列（標題＋複製鈕＋右端 meta icons）、描述列（一行截斷，無內容時缺席）、meta 列。標題 SHALL 以等寬字型呈現（變更名稱與討論 slug 同為可複製把手）。標題 SHALL 恆為單行——長於可用寬度時 SHALL 就地截斷，SHALL NOT 折行、SHALL NOT 強制斷字；截斷處 SHALL 以尾端漸層淡出呈現，SHALL NOT 以省略號或硬切收尾；標題未被截斷時 SHALL NOT 套用淡出，末尾字元不得被誤淡。複製鈕 SHALL 與標題同列尾隨於標題文字之後，SHALL NOT 因標題過長而落至次行、SHALL NOT 被壓縮，meta icons 維持靠右；SHALL NOT 將複製鈕推至列右緣。變更卡描述列 SHALL 顯示 proposal Why 首句（一行截斷）；proposal 缺席、Why 區段缺席或為空時描述列 SHALL 缺席。描述資料 SHALL 由變更清單 payload 一次帶出，SHALL NOT 逐卡讀取 proposal 全文；該欄位屬呈現層輔助欄位，不屬 CLI --json 對齊範圍。建立者 SHALL 以頭像圓點呈現且 hover 顯示全名，SHALL NOT 於卡面直出全名文字；createdBy 缺席時圓點缺席。狀態 chip SHALL 僅在所在位置無法表達狀態時出現：討論卡（討論欄一欄兩態）帶狀態 chip，變更卡（所在欄即階段）SHALL NOT 帶狀態 chip。

#### Scenario: 變更卡三列骨架

- **WHEN** 看板載入一個 proposal Why 首句非空、createdBy 存在、任務 5/21 的變更
- **THEN** 變更卡識別列以等寬字型顯示名稱且複製鈕緊跟名稱文字後（hover 顯現、點擊寫入名稱至剪貼簿且不開詳情抽屜）、右端呈建立者圓點；描述列一行截斷顯示 Why 首句；meta 列顯示進度條與 5/21；卡上無狀態 chip

#### Scenario: 變更卡無 Why 內容時描述列缺席

- **WHEN** 某變更無 proposal.md（或 Why 區段為空）
- **THEN** 該變更卡不顯示描述列，識別列與 meta 列照常呈現，看板不因該筆缺件而報錯

#### Scenario: 長標題單行截斷時複製鈕仍在同列

- **WHEN** 變更名稱長於卡片可用寬度
- **THEN** 標題維持單行並於可用寬度處截斷、截斷處漸層淡出，複製鈕緊跟標題文字之後留在同一列且維持可點，右端 meta icons 不被擠出卡外

#### Scenario: 討論卡長 slug 與變更卡同一收尾

- **WHEN** 討論 slug 長於卡片可用寬度
- **THEN** slug 維持單行截斷並於截斷處漸層淡出、不強制斷字折行，複製 slug 鈕留在同一列，收尾行為與變更卡標題一致

#### Scenario: 短標題不套淡出

- **WHEN** 變更名稱短於卡片可用寬度
- **THEN** 標題完整顯示且末尾字元不被淡化，複製鈕緊跟標題最後一個字元後
