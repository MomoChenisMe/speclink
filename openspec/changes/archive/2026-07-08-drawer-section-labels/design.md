## Context

drawer-document-readability 刀為討論側建立了結構化視覺語言：小字大寫 muted 標籤區塊（輪的焦點／立場、結論的決定／理由）、卡片邊界、delta 色標，且欄位解析已有「白名單＋整篇退回」的既定套路（splitLabeledFields、splitDeltaSections 前例）。變更抽屜的提案／設計分頁仍以單一 Markdown 渲染，英文模板章節名（Why、Context…）以 h2 直出；任務分頁群組標題為粗體 h4。提案／設計的章節名由 schema 模板固定產生（spec-driven 的 feature／bugfix／refactor 三型提案模板與 design 模板），與 conclude 六欄位同屬「可靠可解析的 scaffold 結構」。

本刀純前端（packages/ui），無 Rust 變更、無技能資產變更、無 golden 再生。

## Goals / Non-Goals

**Goals:**

- 提案／設計分頁的已知模板章節以中文標籤區塊呈現，與討論側同款——三種分頁同一視覺語言（使用者明示原則）。
- 變更抽屜與已封存檢視重用同一章節檢視元件。
- 任務分頁群組標題與章節標籤同款式。

**Non-Goals:**

- 章節卡片化（連續論述被切碎，已否決）；只調樣式不映射中文（英文工程詞仍直出，已否決）。
- 規格分頁與討論抽屜各分頁（前刀已定案）。
- 回寫任何 markdown 來源檔；模板本身（engine 端 scaffold）不動。

## Decisions

### D1 章節切分：頂層標題白名單映射，未知照排

新增 SectionedDoc 元件（packages/ui/src/components/SectionedDoc.tsx）：行掃描把文件按 `## ` 頂層標題切段——標題文字（trim 後）命中白名單即渲染為中文標籤區塊（標籤＋該段內文交給共用 Markdown prose）；未命中的標題連同其內文原樣併入 prose 段照排（含設計的 D1／D2 決策標題、手寫自訂章節）。`### New Capabilities`／`### Modified Capabilities` 兩個 h3 模板詞同列白名單（提案 Capabilities 節內的次級標籤）。整份文件無任何白名單命中時整篇照現行單一 Markdown 渲染（與輪／結論同型 fallback）。

替代方案：重用 splitLabeledFields——它解析的是行首粗體前綴欄位，章節是 markdown 標題結構，形狀不同，硬套會扭曲兩者；沿用其「白名單＋fallback」精神、另寫標題版切分，與 splitDeltaSections 同套路。

### D2 對照表與標籤文案：i18n 收斂、涵蓋三型提案模板與設計模板

章節名→i18n key 對照表放 SectionedDoc 內（單一來源）；中文標籤進 i18n（zh-TW／en 雙語系，en 即原文）。涵蓋聯集：提案 feature 型（Why 為什麼、What Changes 變更內容、Non-Goals 非目標、Capabilities 能力、New Capabilities 新增能力、Modified Capabilities 修改能力、Impact 影響）、bugfix 型（Problem 問題、Root Cause 根因、Proposed Solution 解法、Success Criteria 成功準則）、refactor 型（Summary 摘要、Motivation 動機、Alternatives Considered 曾考慮的替代案）、設計模板（Context 背景、Goals / Non-Goals 目標與非目標、Decisions 決策、Implementation Contract 實作契約、Risks / Trade-offs 風險與取捨、Migration Plan 遷移計畫、Open Questions 未解問題）。標題比對取 trim 後全字串精確比對（Non-Goals (optional) 形式的模板附註「 (optional)」於比對前剝除）。

替代方案：標籤直接硬編碼中文於元件——app 既有 i18n 慣例（rounds.*、conclusion.*、delta.* 皆走 i18n），不另闢蹊徑。

### D3 接線：RichDetailDrawer 與 ArchivedList 的提案／設計分頁換用 SectionedDoc

兩處的提案／設計 TabsContent 由 Markdown 換為 SectionedDoc（空狀態 empty 文案照舊傳遞）；規格分頁維持 DeltaSpecView、討論側維持 RoundsView／ConclusionView，互不相擾。

### D4 任務群組標題款式對齊

TaskList 的群組標題（SortableGroupHeading）由 text-base font-bold 改為章節標籤同款（text-xs font-semibold uppercase tracking-wider text-muted-foreground）；群組文字是使用者內容，不翻譯不改寫；拖曳讓位行為（disabled sortable）不動。

替代方案：任務分頁不動——使用者已裁定一併調，且三分頁同語言正是本刀目的。

### D6 標籤款式：粗體大標題、單一常數全面套用

（實作驗收後由使用者比對裁定，取代首版的小字大寫 muted 標籤。）標籤款式家族改為粗體大標題——計算字級大於 16px 內文（text-xl 級、font-bold、前景色），不再 uppercase、不再 muted。款式抽為單一共用常數（SectionedDoc 匯出），五處引用主標題款：SectionedDoc 章節標籤、RoundsView 輪欄位標籤、ConclusionView 結論欄位標籤、DeltaSpecView 色標區段標頭（沿用各 delta 色彩、其餘款式同）、ArchivedList 討論檢視的區段標題。次級款（text-base font-bold，同族、字級同內文基準）兩處引用：Capabilities 內的次級標籤（新增能力／修改能力）與 TaskList 群組標題——後者為使用者第二次比對裁定（大標題在任務清單過重），恰為 Spectra 任務清單的原尺寸。輪卡片內的欄位標題視覺權重高於卡頭 chip 屬可接受取捨——一致性優先，真實視窗驗收把關。

替代方案：兩級制（章節大標題、欄位小標籤）——字體層級論上更正統，但使用者兩度要求跨分頁一致且明示偏好大標題，否決。

### D5 測試策略：jsdom 結構驗證，視覺以真實視窗驗收

TDD 紅綠重構。jsdom 驗：白名單章節渲染中文標籤且英文標題不直出、未知章節照排、無命中整篇退回、已封存同型、任務群組標題款式 class。視覺（間距、與討論側並排一致感）以 release exe 真實視窗截圖驗收。

## Implementation Contract

**可觀察行為：**

- 變更抽屜提案／設計分頁與已封存檢視提案／設計分頁：已知模板章節以中文標籤區塊呈現（為什麼／變更內容／非目標／能力／影響／背景／決策／實作契約／風險與取捨／未解問題等），Why、Context 等英文模板標題不以標題文字直出；白名單外的標題（決策標題、自訂章節）照 prose 排；整份無白名單命中時整篇照現行渲染。
- 任務分頁：群組標題以章節標籤同款式呈現，文字內容不變；勾選、拖曳、工具列行為不變。
- 標籤款式（六處同源）：粗體大標題、計算字級大於內文 16px；規格色標區段標頭保留各 delta 色彩；討論輪／結論欄位標籤同款式（結構與切分邏輯不動）。
- 來源檔案在任何路徑下位元不變；CLI 人眼與 --json 輸出逐位元不變。

**介面／資料形狀：** SectionedDoc 輸入文件全文與空狀態文案、輸出章節化渲染；章節切分為 packages/ui 內部純函式（頂層標題→白名單命中段／prose 段的序列），無 IPC、無新依賴。

**失敗模式：** 白名單零命中→整篇單一 Markdown 檢視退回；渲染不報錯、不留空白。

**驗收：** npm test -w packages/ui 全綠（新增章節標籤、未知照排、fallback、已封存同型、任務群組款式測試）；npm test -w apps/desktop 全綠；release exe 並排開變更抽屜提案分頁與討論抽屜結論分頁截圖，人工核對標籤款式一致。

**範圍邊界：** in scope＝提案／設計分頁章節呈現（兩檢視面）、任務群組標題款式、i18n 標籤、六處標籤家族的款式統一（含討論側欄位標籤與色標標頭的「款式」）；out of scope＝討論側與規格分頁的切分結構、markdown 來源回寫、engine 模板、CLI。

## Risks / Trade-offs

- [schema 模板日後新增章節名，白名單漏收→該章英文直出] → 對照表集中單處、補一詞即收；未知章節照排保證不炸版。
- [使用者手寫章節恰與模板詞同名（如自寫 ## Impact）] → 命中即標籤化——語意相同，視為正確行為。
- [大標題款式使輪卡片內欄位標題視覺權重高於卡頭 chip] → 一致性優先的既定取捨；真實視窗驗收把關，過重再回報調整。
- [回歸對照] → 純前端呈現層，CLI 零變更；packages/ui 既有 156 測試為護欄。

## Open Questions

（無）
