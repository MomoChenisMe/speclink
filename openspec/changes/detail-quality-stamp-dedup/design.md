## Context

變更詳情頁（packages/ui 的 ChangeDetailPage）的左欄目前有兩套章籤渲染：

- 身分區：TitleStamp 元件，可視只有圖示＋狀態詞，蓋章日期與蓋章者完整識別收進主題化提示；帶 data-review-row／data-verify-row 與 data-review-tone／data-verify-tone 標記。
- 「狀態」區的「品質關卡」列：在列內直接 map 同一份 stamps 資料，畫狀態籤＋蓋章日期，無提示、無 data 標記。

已封存詳情頁（packages/ui 的 ArchivedDetailPage）把同一個 outcomes 片段（帶 data-review-outcome／data-verify-outcome）同時塞進身分區 badges 與「品質關卡」列，DOM 上有兩份。

兩頁都只在 apps/desktop 渲染；server-web 不用這兩個元件。來源討論：detail-quality-stamp-dedup。規格以 doc-tiles-fill-and-round-order 封存後的 desktop-app 正典字面為底（該變更已於 2026-10-13 封存）。

## Goals / Non-Goals

**Goals:**

- 兩種詳情頁的審查／驗證章籤只在「品質關卡」列出現一次。
- 原本身分區章籤的主題化提示（蓋章日期＋含 email 的完整識別）不流失，改掛在「品質關卡」列的章籤上。
- 變更詳情頁的章籤只剩一條渲染路徑。

**Non-Goals:**

- 不動身分區的階段籤、「已封存」籤與討論詳情頁的狀態籤。
- 不動左欄「審查」「驗證」文件節點（含「第 N 輪」徽章）。
- 不動看板卡片的章、已封存清單的結局標示、品質站配色（REVIEW_PILL_TONE／VERIFY_PILL_TONE）與提示的 300ms 延遲。
- 不改 ChangeItem、ArchivedItem 或任何 protocol 資料形狀；不碰 CLI 與 server。
- 已封存側不新增蓋章者提示——已封存資料本來就沒有帶蓋章者，維持現狀。
- 不處理提示觸發點的鍵盤可達性（既有 TitleStamp 的觸發點同樣不可聚焦，屬既有行為）。

## Decisions

### D1 身分區只留階段籤，兩種詳情頁一起改

變更詳情頁的 DetailIdentity badges 只放階段籤；已封存詳情頁的 badges 只放「已封存」籤。

- 理由：身分區回答「這是什麼」，「狀態」區回答「做到哪了」；規格已用同一條線把任務計數放到「狀態」列。
- 否決：保留身分區章籤、拿掉「品質關卡」列——兩站皆 none 時看不到任何品質資訊，日期要停留才看得到。
- 否決：只改變更詳情頁——兩種詳情頁的身分區會長得不一樣。

### D2 品質關卡列的章籤元件合一

TitleStamp 退場，改成 ChangeDetailPage 模組內私有的 QualityStamp 元件（不匯出），變更詳情頁的「品質關卡」列對每一站渲染它：

- 外層一個 inline-flex 的 span（站別 data 標記在此），內含狀態籤（tone 標記在此）與蓋章日期（有 at 時，等寬 11px 灰字，與現行品質關卡列相同）。
- 提示內容維持現行 TitleStamp 的組法：蓋章日期與蓋章者以「 · 」相連；兩者皆缺（進行中、無戳記）時不包 Tooltip。
- 提示觸發點是整個外層 span（狀態籤＋日期），一站一個觸發點。
- 理由：兩條渲染路徑合一後，「品質關卡」列與提示只需要維護一份；觸發點涵蓋日期，使用者停在日期上也看得到蓋章者。
- 取捨：日期在可視文字與提示內各出現一次。接受——提示內容與現行一致，不新增 i18n 詞條，「主題化提示統一延遲」需求的「變更詳情頁的章籤」情境字面不必改。
- 否決：觸發點只包狀態籤——多一層結構，停在日期上看不到蓋章者，沒有好處。

### D3 既有 data 標記跟著章籤搬到品質關卡列

- 變更詳情頁：data-review-row／data-verify-row 落在 D2 的外層 span；data-review-tone／data-verify-tone 落在狀態籤。身分區不再有這四個標記。
- 已封存詳情頁：outcomes 片段只渲染在「品質關卡」列，data-review-outcome／data-verify-outcome 全頁只剩一份。
- 理由：沿用既有標記，reviewBadge／verifyBadge 的配色與同構測試只需改「取點在哪一區」與「列內含不含日期」，不必重寫。

### D4 規格改寫四條 desktop-app 需求

- 「變更詳情頁的標頭與狀態卡」：身分區改為只有階段籤；章籤、日期與提示的描述移到「品質關卡」列；改寫情境「章籤與品質關卡列」「品質關卡列的空態與進行中」，「標頭結構與任務計數位置」補上身分區不含章籤。
- 「已封存項目以詳情頁檢視」：身分區去掉結局籤；情境「封存變更詳情頁的標題列、出身列與狀態卡」改成身分區只帶「已封存」籤。
- 「詳情頁的文件樹、閱讀卡與本頁目錄」：身分區描述與情境「三欄骨架與左欄內容」去掉章籤。
- 「詳情頁的工單分頁」：情境「工單被刪時分頁退場」的「身分區章籤改為『已審查』」改為「品質關卡列的章籤改為『已審查』」。
- 「主題化提示統一延遲」不改：「變更詳情頁的章籤」仍成立，只是位置換到「品質關卡」列。
- 所有既有 scenario 名稱保留，沒有刪除或改名。

## Implementation Contract

**落點與邊界**：只改 @speclink/ui（packages/ui）的 ChangeDetailPage、ArchivedDetailPage 與四份測試。apps/desktop 透過 @speclink/ui 取得新行為，自身程式碼不改。沒有 Rust、CLI、protocol 或 server 變更。用既有原語 StatusPill、Tooltip、DetailIdentity、StatusRow，不新增共用匯出。

**可觀察行為**：

- 變更詳情頁、reviewed（2026-08-04、蓋章者含 email）＋verified（2026-08-06）：身分區（data-detail-identity）只有階段籤「已就緒」，文字不含「已審查」「已驗證」；「品質關卡」列依序顯示「已審查」＋「2026-08-04」、「已驗證」＋「2026-08-06」，文字不含 email；停留在「品質關卡」列的「已審查」上，提示含 2026-08-04 與 email；停留在「已驗證」上，提示含驗證站自己的日期與識別。
- 變更詳情頁、inReview＋inVerify：身分區無章籤；「品質關卡」列顯示「審查中」「驗證中」、無任何日期；停留時無提示。
- 變更詳情頁、兩站 none：身分區與全頁都沒有 data-review-row／data-verify-row；「品質關卡」列為「尚未經過品質關卡」。
- 已封存詳情頁、reviewed＋verifiedNotPassed：身分區只有「已封存」籤，沒有 data-review-outcome／data-verify-outcome；「品質關卡」列有「已審查」「曾驗證未通過」；全頁 data-review-outcome 只有一個。
- 已封存詳情頁、兩站皆無：「品質關卡」列為「未經品質關卡」。

**DOM 標記**：data-review-row／data-verify-row 在「品質關卡」列每站的外層 span，其內含狀態籤與日期；data-review-tone／data-verify-tone 在狀態籤；兩站的外層 span className 相同，審查在前。

**驗收**：

- `npm test -w packages/ui` 全綠，其中 changeDetailPage、archivedDetailPage、reviewBadge、verifyBadge 四份測試依上述行為改寫。
- `npm test -w apps/desktop` 全綠（App.test 只渲染詳情頁、不查章籤，預期不必改）。
- 用 rg 檢查 packages/ui/src 已無 TitleStamp。
- 人工在 desktop 開一個已蓋章的變更詳情頁與一個已封存變更詳情頁，看身分區無章籤、「品質關卡」列有章籤，停留在章籤上有提示。

**範圍外**：見 Non-Goals。

## Risks / Trade-offs

- [其他測試依賴身分區章籤] → 已用 rg 掃過 packages/ui 與 apps/desktop：只有上述四份測試查這些 data 標記；apps/desktop 的 App.test 不查。實作時跑兩個 workspace 的全部測試確認。
- [回歸對照：golden 與 CLI 測試] → 不受影響，本變更沒有 Rust 或 CLI 輸出變更。
- [跨平台] → 純 React 渲染，Windows／macOS／Linux 行為相同；提示觸發沿用既有 Tooltip。
- [提示裡的日期與可視日期重複] → 接受（D2），換來提示內容與規格字面不必改。
- [平行變更改到同一需求] → discussion-stage-and-unpromoted-cuts 只改「討論詳情頁檢視與轉出變更」「討論結論以欄位標籤呈現」，不重疊；doc-tiles-fill-and-round-order 已封存，delta 以其封存後字面為底。
