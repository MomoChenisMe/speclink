## ADDED Requirements

### Requirement: 討論輪以卡片呈現

桌面 app 的討論過程呈現（討論抽屜的討論過程分頁、已封存討論檢視的討論過程分頁）SHALL 將符合 scaffold 格式（輪標題形如「### Round N — <mode> (<date>)」）的記錄逐輪呈現為獨立卡片：卡頭 SHALL 顯示輪次、mode 與日期；卡身 SHALL 將行首的 Focus／Position／Ruled out／Open 粗體前綴呈現為欄位標籤區塊，前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行（或輪結尾）的全部內容。來源缺席的欄位 SHALL NOT 渲染空標籤；mode 值 SHALL 按來源字串透傳呈現（不設白名單）。任一輪標題不符 scaffold 格式時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 標準記錄逐輪成卡

- **WHEN** 開啟含四輪 scaffold 格式記錄的討論過程分頁
- **THEN** 呈現四張輪卡片，各卡頭顯示輪次、mode 與日期；卡身的 Focus 與 Position 以標籤區塊呈現，來源的粗體前綴原文不出現在渲染結果

##### Example: 輪標題解析

- **GIVEN** 來源輪標題「### Round 1 — assumptions (2026-07-08)」
- **WHEN** 開啟討論過程分頁
- **THEN** 該卡片卡頭顯示輪次 Round 1、mode assumptions、日期 2026-07-08

#### Scenario: 缺席欄位不渲染空標籤

- **WHEN** 某輪來源只有 Focus、Position 與 Open 行，無 Ruled out 行
- **THEN** 該卡片呈現 Focus、Position、Open 三個標籤區塊，無 Ruled out 標籤

#### Scenario: 欄位涵蓋後續多行內容

- **WHEN** 某輪的 Position 標籤行之後接數行列點、其後才是 Ruled out 標籤行
- **THEN** 該數行列點全數呈現於 Position 欄位區塊內，列表符號保留

#### Scenario: 非標準格式整篇退回

- **WHEN** 記錄的 Rounds 區段含不符 scaffold 格式的輪標題（手寫或 pre-scaffold 記錄）
- **THEN** 討論過程分頁整篇以單一 markdown 檢視呈現，無卡片、無錯誤訊息，來源檔案位元不變

### Requirement: 討論結論以欄位標籤呈現

桌面 app 的討論結論呈現（討論抽屜的結論分頁、已封存討論檢視的結論區）SHALL 將行首的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 粗體前綴呈現為欄位標籤區塊（標籤依語系呈現，zh-TW 為決定／理由／否決替代案／擱置／記錄去向／下一步），前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行（或結論結尾）的全部內容。來源缺席的欄位 SHALL NOT 渲染空標籤；非六詞白名單的粗體前綴行 SHALL 按一般內文歸屬當前欄位。結論不含任何白名單欄位時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 標準結論欄位成標籤區塊

- **WHEN** 開啟含 scaffold 格式結論（Decision、Rationale、Capture to、Next 各佔一行起頭）的結論分頁
- **THEN** 各欄位以標籤區塊呈現，「**Decision**:」等粗體前綴原文不出現在渲染結果

#### Scenario: 結論缺席欄位不渲染空標籤

- **WHEN** 某結論來源只有 Decision 與 Rationale 行，無 Deferred 行
- **THEN** 結論分頁呈現決定與理由兩個標籤區塊，無擱置標籤

#### Scenario: 自由格式結論整篇退回

- **WHEN** 結論來源為手寫自由段落、不含任何白名單粗體前綴行
- **THEN** 結論分頁整篇以單一 markdown 檢視呈現，無標籤區塊、無錯誤訊息，來源檔案位元不變

### Requirement: markdown 文件內容行寬有上限

桌面 app 經共用 markdown 渲染呈現的文件內容（變更抽屜的提案／設計／規格分頁、討論抽屜各分頁、已封存檢視）SHALL 有固定行寬上限與一致的容器留白；抽屜寬度改變（含全螢幕）時內文行寬 SHALL NOT 隨之增長；超過行寬的表格 SHALL 於容器內橫向捲動，版面 SHALL NOT 橫向溢出。

#### Scenario: 全螢幕下行寬不增長

- **WHEN** 變更抽屜切換至全螢幕（96vw）檢視提案分頁
- **THEN** 內文行寬維持固定上限，不隨抽屜變寬而增長

#### Scenario: 寬表格於容器內橫捲

- **WHEN** 檢視含超過行寬上限的寬表格的文件
- **THEN** 表格於容器內橫向捲動，抽屜版面不橫向溢出

### Requirement: 規格分頁 delta 區段以色標呈現

變更抽屜規格分頁與已封存變更檢視的規格分頁 SHALL 將 delta spec 的區段標題（ADDED／MODIFIED／REMOVED／RENAMED Requirements）呈現為色標區段標頭，原始區段標題行 SHALL NOT 以標題文字直出；區段內的 requirement 與 scenario 內文 SHALL 照 prose 排版呈現。不含 delta 區段標題的規格文件 SHALL 整篇照常渲染。色標配色 SHALL 與 delta 計數徽章（DeltaBadges）一致。

#### Scenario: delta 區段呈現色標標頭

- **WHEN** 檢視含 ADDED 與 MODIFIED 區段的 delta spec
- **THEN** 呈現綠色「新增」與琥珀色「修改」區段標頭，原始「ADDED Requirements」「MODIFIED Requirements」標題文字不出現在渲染結果

##### Example: 四種 delta 區段的色標對應

| 來源區段標題 | 標頭文字 | 色系 |
| ------------ | -------- | ---- |
| ## ADDED Requirements | 新增 | 綠（emerald） |
| ## MODIFIED Requirements | 修改 | 琥珀（amber） |
| ## REMOVED Requirements | 移除 | 紅（red） |
| ## RENAMED Requirements | 更名 | 藍（sky） |

#### Scenario: 無 delta 標記的規格照常渲染

- **WHEN** 檢視不含任何 delta 區段標題的規格文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無色標標頭
