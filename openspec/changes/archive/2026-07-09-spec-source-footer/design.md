## Context

桌面規格頁（SpecList）展開卡片時以 Markdown 元件渲染 spec.md 全文，該元件以 skipHtml 丟棄所有 raw HTML，連帶把封存時注入的 @trace 註解濾掉。@trace 每個 requirement 一塊，含 source（最後動這條需求的變更）、updated、code。source 是溯源資訊，其原始字串已隨 loadDocument 送達前端。本變更只在既有展開檢視下方加一行來源 footer，不觸及 core 與 IPC。承 discuss `spec-trace-display` 結論：updated 不顯示、code 檔案樹 defer、原始註解維持過濾。

## Goals / Non-Goals

**Goals:**

- 規格頁展開檢視底部顯示該 spec 的來源變更（@trace source 去重集合）。
- 純前端解析，零 core／IPC 變更。

**Non-Goals:**

- 不顯示 @trace 的 updated 日（卡頭 mtime 已涵蓋）。
- 不渲染 code 相關檔案樹（discuss 已 defer 至桌面具開檔能力後再議）。
- footer 不可點擊、不做跳轉至變更的導航。
- 不改變 raw HTML／@trace 註解仍被 skipHtml 過濾的現況。
- 不逐 requirement 就地標註 source（見 D2）。

## Decisions

### D1：於前端解析 @trace source，不動 core 與 IPC

trace 原始字串已在 loadDocument 回傳的 spec.md 全文內，前端以正則抽出 @trace 區塊的 source 行即可。新增 packages/ui/src/trace.ts 承載解析（比照既有 delta.ts 的定位）。

- 替代：於 core 解析並經 adapter 曝露結構化 trace——否決：唯一消費者是規格頁，跨 core／CLI／IPC 三處改動屬過度設計。

### D2：聚合至 spec 層級去重呈現，非逐 requirement 標註

一份 spec.md 有多個 @trace（每 requirement 一塊），同一變更的多條需求 source 相同。footer 取全檔 source 去重、依首次出現順序，成一行「來源變更：A、B」。

- 替代：逐 requirement 就地顯示 source——否決：需與 Markdown 渲染交錯、複雜；discuss 結論要的是「一行輕量 footer」的溯源，非精確到每條需求。

### D3：footer 僅顯示不可點擊且不顯示 updated

呼應 discuss 的「溯源 vs 導航」取捨：本變更只做溯源顯示。source 名以純文字呈現，不連結、不開變更；updated 不顯示（與卡頭 mtime 重疊）。

- 替代：source 可點擊開變更卡——否決：導航是 discuss 明確 defer 的一支，且規格頁與變更頁分屬不同導覽區，跳轉體驗另需設計。

## Implementation Contract

- 行為：使用者在規格頁展開一張其 spec.md 含至少一個 @trace source 的卡片時，全文下方顯示一行來源 footer——內容為該檔所有 @trace 區塊 source 值去重後、依首次出現順序、以分隔符連接，前置在地化標籤（如「來源變更：」）。spec.md 無任何 @trace（或無 source）時不顯示 footer。
- 介面／資料形狀：新增 packages/ui/src/trace.ts 匯出純函式，輸入 raw markdown 字串、回傳去重且保序的 source 名字串陣列。SpecList 的展開區在 Markdown 之後條件渲染 footer。i18n 新增 footer 標籤鍵。
- 失敗模式：解析對缺 source 或畸形的 @trace 區塊靜默略過該塊（不丟例外、不顯示空 footer）；空結果即不渲染 footer。
- 驗收：packages/ui 的 trace 解析單元測試涵蓋「單一 source／多 source 去重保序／無 @trace／畸形區塊略過」；specList 測試驗「有 source 顯示 footer、無 source 不顯示」。`npm test -w packages/ui` 綠。
- 範圍邊界：in scope＝packages/ui 的解析工具、SpecList 展開檢視 footer、i18n 標籤、對應測試。out of scope＝core／IPC／adapter 改動、可點擊導航、updated 或 code 檔案樹呈現、規格頁以外任何檢視。

## Risks / Trade-offs

- [聚合掩蓋逐需求差異] → 可接受：footer 定位為 spec 層級溯源、非精確索引，discuss 已認可。
- [@trace 格式若未來變動，前端正則須同步] → 緩解：解析集中於 trace.ts 單點，格式由 archive.rs 的 trace_block 決定、變更頻率低；測試涵蓋畸形略過。
