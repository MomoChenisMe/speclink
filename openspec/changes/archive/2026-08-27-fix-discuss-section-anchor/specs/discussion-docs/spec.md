## ADDED Requirements

### Requirement: 討論記錄結構錨定與撞名內容跳脫

引擎解析討論記錄時 SHALL 只把整行為「## Context」「## Rounds」「## Conclusion」的行視為結構標題，並保留 pre-scaffold 容忍：未經跳脫、行首為「## Round 」的行亦視為區段邊界（pre-scaffold 版面的既有輪與結論不得因區段替換而遺失）；輪內文中其他以「## 」開頭的行 SHALL NOT 截斷區段、SHALL NOT 改變 add-round 的插入點。討論內容寫入動詞（context、add-round、conclude）SHALL 在落盤前把撞名內容行（整行為結構標題，或行首為「### Round 」「## Round 」前綴）以 markdown 反斜線跳脫；成對 fenced code block（``` 或 ~~~ 圍欄）內的行 SHALL NOT 跳脫、SHALL NOT 視為結構或輪標題，且寫入動詞 SHALL 使落盤內容的圍欄行成對——內容含奇數個圍欄行時，最後一個落單的圍欄行一併以反斜線跳脫。輪計數 SHALL 只認合法輪標題形狀（「### Round <編號> — <mode> (<日期>)」，與 UI 輪切分同形；並保留 pre-scaffold「## Round 」前綴的容忍），SHALL NOT 因內文撞名而膨脹或跳號。桌面與 Web 的區段切分 SHALL 與引擎採同一結構標題白名單與容忍規則。

#### Scenario: 輪內文含二級標題行時新輪仍落於既有輪之後

- **WHEN** 既有 Round 1 的內文含一行「## 背景」，執行 add-round 追加第二輪
- **THEN** Round 2 標題插在 Round 1 完整內文之後、Conclusion 結構標題之前；Round 1 的內文（含該「## 背景」行）完整留在 Round 1 標題之下

##### Example: 兩輪順序與內文歸屬

- **GIVEN** Round 1 內文兩行——「## 背景」與「首輪本文」
- **WHEN** add-round 追加內文為「次輪本文」的 Round 2
- **THEN** 文件順序為：Round 1 標題、「## 背景」、「首輪本文」、Round 2 標題、「次輪本文」、Conclusion 結構標題

#### Scenario: 結論寫入不落入輪內

- **WHEN** 某輪內文原始輸入含整行「## Conclusion」，其後執行 conclude
- **THEN** 該內容行以跳脫形式留在該輪內文中；結論內容寫入文件的結構 Conclusion 區段；既有輪內文不被改寫

#### Scenario: 撞名輪標題行不膨脹輪計數

- **WHEN** 某輪內文含行首為「### Round 」的行，隨後 add-round 追加新輪
- **THEN** 新輪編號等於既有合法輪數加一，無跳號；discuss list 回報的輪數等於合法輪標題數

#### Scenario: 區段切分不被輪內文截斷

- **WHEN** 桌面或 Web 檢視的討論記錄，其某輪內文含以「## 」開頭的行
- **THEN** Rounds 區段完整切分至 Conclusion 結構標題為止，該輪內文完整顯示於該輪之下
