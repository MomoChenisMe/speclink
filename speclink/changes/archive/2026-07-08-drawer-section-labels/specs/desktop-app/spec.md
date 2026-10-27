## ADDED Requirements

### Requirement: 提案與設計章節以中文標籤呈現

變更抽屜與已封存檢視的提案／設計分頁 SHALL 將已知模板章節標題（提案側 Why／What Changes／Non-Goals／Capabilities／New Capabilities／Modified Capabilities／Impact／Problem／Root Cause／Proposed Solution／Success Criteria／Summary／Motivation／Alternatives Considered；設計側 Context／Goals / Non-Goals／Decisions／Implementation Contract／Risks / Trade-offs／Migration Plan／Open Questions）呈現為中文標籤區塊，英文模板標題 SHALL NOT 以標題文字直出；標籤款式 SHALL 為粗體大標題——計算字級 SHALL 大於內文基準字級（16px），且 SHALL 與討論側欄位標籤（輪的焦點／立場、結論的決定／理由）及規格分頁色標區段標頭同款式（色標標頭保留各 delta 色彩）。白名單以外的章節標題 SHALL 連同內文照 prose 排版呈現。整份文件無任何白名單章節時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 提案模板章節成中文標籤

- **WHEN** 檢視含 Why、What Changes、Non-Goals、Capabilities、Impact 章節的提案分頁
- **THEN** 呈現「為什麼」「變更內容」「非目標」「能力」「影響」標籤區塊，Why 等英文標題文字不出現在渲染結果

##### Example: 章節對照

| 來源章節標題 | 呈現標籤 |
| ------------ | -------- |
| ## Why | 為什麼 |
| ## What Changes | 變更內容 |
| ## Non-Goals | 非目標 |
| ## Context | 背景 |
| ## Decisions | 決策 |
| ## Risks / Trade-offs | 風險與取捨 |

#### Scenario: 標籤為大標題且字級大於內文

- **WHEN** 檢視提案分頁的章節標籤與討論抽屜結論分頁的欄位標籤
- **THEN** 兩者款式一致，皆為粗體且計算字級大於內文的 16px

#### Scenario: 白名單外章節照排

- **WHEN** 檢視設計分頁，其 Decisions 章節內含自訂決策標題（如 D1 起頭的三級標題）
- **THEN** 決策標題照 prose 標題樣式渲染，不被標籤化、不被翻譯

#### Scenario: 無白名單章節整篇退回

- **WHEN** 檢視手寫自由格式（無任何模板章節標題）的提案文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無標籤區塊、無錯誤訊息

### Requirement: 任務群組標題與章節標籤同款式

變更抽屜任務分頁的群組標題 SHALL 以標籤家族的次級款式呈現——粗體、計算字級與內文基準（16px）一致，與 Capabilities 次級標籤同款、與章節主標題同族但小一級；群組標題文字 SHALL 照來源呈現（不翻譯、不改寫）；任務勾選、拖曳排序與工具列行為 SHALL 不受款式變更影響。

#### Scenario: 群組標題款式一致

- **WHEN** 檢視含群組標題的任務分頁
- **THEN** 群組標題為粗體、計算字級為 16px（與任務文字同級、小於章節主標題），標題文字與來源一致

#### Scenario: 互動行為不變

- **WHEN** 在款式調整後的任務分頁勾選任務並拖曳排序
- **THEN** 勾選與排序行為與調整前一致，寫回結果正確
