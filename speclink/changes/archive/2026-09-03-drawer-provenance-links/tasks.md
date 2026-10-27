## 1. 來源連結籤的不可點狀態（design D3 來源連結籤的不可點狀態）

- [x] 1.1 RED：在 packages/ui/src/__tests__/specDrawer.test.tsx 新增「無封存記錄的來源變更不可點」案例——傳入 archivedChanges 中無對應的來源變更名時，該籤帶 aria-disabled="true"、點擊不觸發 onOpenArchivedChange、Tooltip 副標為「無封存記錄」，且排在可點籤之後；跑 `npm test -w packages/ui` 確認新案例因 SourceLinkItem 尚無 disabled 而失敗 <!-- speclink-task:tsk_01M1K2Z6BTEPFGQ16P6MTG2KR6 -->
- [x] 1.2 GREEN：packages/ui/src/components/SourceDiscussionChip.tsx 的 SourceLinkItem 補 `disabled?: boolean`；SourceDiscussionChip 在 disabled 時灰底灰字、無 hover 樣式、無 onClick、帶 aria-disabled，Tooltip 仍顯示 slug 與副標；SourceChipRow 的「+N」浮層內 disabled 項不可點且不關閉浮層；跑 `npm test -w packages/ui` 確認 1.1 轉綠且既有 archivedDrawer、discussionDrawer 測試不變 <!-- speclink-task:tsk_01M1K2Z6BTEJV0WVJ9M4GTG0ZV -->

## 2. 規格抽屜標頭兩層與溯源籤（design D1 溯源籤搬進規格抽屜標頭並移除內文底部的 footer、D2 封存清單由 host 傳入並在抽屜內解析與排序）

- [x] 2.1 RED：改寫 packages/ui/src/__tests__/specDrawer.test.tsx——刪除四處對內文底部「來源變更：」字面的斷言，改為依需求「桌面 app 呈現 change 與 spec 的清單與內容」斷言：標頭出身列顯示「來自」與第一顆籤為封存日期最早的變更；來源多於一個時顯示「+N」、點開浮層依封存日期升冪列出其餘且副標為封存日期；點擊可點籤呼叫 onOpenArchivedChange 並帶該變更的 datedName；正典全文無 @trace 時出身列缺席；渲染結果不含「來源變更：」文字；點擊標題旁複製名稱鈕把 capability 名寫入剪貼簿並顯示已複製回饋；跑 `npm test -w packages/ui` 確認新斷言失敗 <!-- speclink-task:tsk_01M1K2Z6BTAVRHRAT90BXJXRQY -->
- [x] 2.2 GREEN：packages/ui/src/components/SpecDrawer.tsx 新增 props `archivedChanges: ArchivedItem[]` 與 `onOpenArchivedChange?: (datedName: string) => void`；標頭改為標題列（capability 名＋複製名稱鈕，沿用 ArchivedDrawer 的複製鈕做法）與出身列（SourceChipRow，前綴取 i18n `sdrawer.fromChanges`）；以 parseTraceSources(doc) 取名後對 archivedChanges 以 name 比對——命中者 `{ slug, topic: 封存日期 }` 依 date 升冪（同日依文件首次出現序），未命中者 `{ slug, topic: t("rdrawer.noArchiveRecord"), disabled: true }` 排最後；移除內文底部的溯源 footer；packages/ui/src/i18n.tsx 新增 `sdrawer.fromChanges`（來自／From）與 `rdrawer.noArchiveRecord`（無封存記錄／No archive record），刪除孤兒化的 `specs.sourceChanges` 與 `specs.sourceSep`；跑 `npm test -w packages/ui` 確認 1.1 與 2.1 全綠 <!-- speclink-task:tsk_01M1K2Z6BT9SHJ6XTYVJQ1EVXK -->

## 3. 已封存討論抽屜的衍生列（design D4 已封存討論抽屜的衍生列與三態跳轉）

- [x] 3.1 RED：在 packages/ui/src/__tests__/archivedDrawer.test.tsx 新增案例，依需求「已封存項目以抽屜檢視」斷言：討論型別且 promotedChanges 非空時出身列之下顯示「衍生」、第一顆籤與「+N」籤，順序沿傳入順序；點擊可點籤呼叫 onOpenPromotedChange 並帶變更名；disabled 項點擊不呼叫；promotedChanges 為空或缺席時「衍生」列缺席；變更型別的抽屜即使傳入 promotedChanges 也不顯示「衍生」列；跑 `npm test -w packages/ui` 確認新案例失敗 <!-- speclink-task:tsk_01M1K2Z6BTGRTQVG4X4PH66CJS -->
- [x] 3.2 GREEN：packages/ui/src/components/ArchivedDrawer.tsx 新增 props `promotedChanges?: SourceLinkItem[]` 與 `onOpenPromotedChange?: (name: string) => void`，於討論型別且清單非空時在出身列之下渲染 SourceChipRow（前綴取 i18n `adrawer.promotedTo`），位置與變更型別的「來自」列相同；packages/ui/src/i18n.tsx 新增 `adrawer.promotedTo`（衍生／Derived）；跑 `npm test -w packages/ui` 確認 3.1 轉綠 <!-- speclink-task:tsk_01M1K2Z6BTWCC8KZCN51CG29JD -->

## 4. desktop 接線（design D2 封存清單由 host 傳入並在抽屜內解析與排序、D4 已封存討論抽屜的衍生列與三態跳轉）

- [x] 4.1 RED：在 apps/desktop/src/__tests__/App.test.tsx 新增案例：(a) 於手冊頁開啟規格抽屜後點擊一顆溯源籤，store 的 detailArchived 為 `{ kind: "change", datedName }`、detailSpec 為 null、boardView 維持 "manual"；(b) 開啟一筆 promotedTo 含已封存、活躍、已刪除三個子變更的封存討論抽屜，三顆籤副標分別為封存日期、看板階段詞、「無封存記錄」，點擊已封存者開啟該封存變更抽屜且 boardView 不變，點擊活躍者開啟其詳情抽屜且 boardView 切為 "board"，已刪除者不可點；跑 `npm test -w apps/desktop` 確認新案例失敗 <!-- speclink-task:tsk_01M1K2Z6BT45VGPHDA4RN80Z32 -->
- [x] 4.2 GREEN：apps/desktop/src/App.tsx 對 SpecDrawer 傳入 `archivedChanges={s.archived}` 與 `onOpenArchivedChange={(datedName) => s.openArchived({ kind: "change", datedName })}`；對 ArchivedDrawer 於討論型別時自 `s.discussions.archived` 該筆的 promotedTo 派生 promotedChanges——`s.archived` 命中者副標封存日期、`s.changes` 命中者副標取 discussionChipStage 的階段詞、皆無者副標「無封存記錄」且 disabled——並以 `onOpenPromotedChange` 分流：封存命中→openArchived、活躍命中→openDetail；跑 `npm test -w apps/desktop` 確認 4.1 轉綠且既有 App 測試不變 <!-- speclink-task:tsk_01M1K2Z6BTR2D0VNQ4F5SDYB58 -->

## 5. 詞彙、守門與驗收（design D5 標籤字與詞彙）

- [x] 5.1 openspec/LANGUAGE.md「衍生變更」詞條的 definition 補一句「抽屜出身列的標籤縮寫為『衍生』，與『來自』『同源』同為兩字關係詞」；不新增詞條、不動 avoid 欄；內容檢閱確認詞條其餘欄位逐字不變，並跑 `node --test scripts/*.test.mjs` 確認詞彙守門與連結檢查全綠 <!-- speclink-task:tsk_01M1K2Z6BT8GDHZJ54YJY2XVA7 -->
- [x] 5.2 對新增的元件 props（archivedChanges 空陣列、promotedChanges 缺席、disabled 與 onOpen 同時存在）套用 sharp-edges 稽核清單（`speclink instructions --skill audit`），確認每個組合有測試涵蓋或明文無行為；然後全面跑 `npm test -w packages/ui`、`npm test -w apps/desktop` 與 `node --test scripts/*.test.mjs`，三者皆綠 <!-- speclink-task:tsk_01M1K2Z6BT01S84GZSQ02S6186 -->
- [x] [M] 5.3 於 desktop 實機開啟任一規格抽屜與一筆有衍生變更的封存討論抽屜，目視確認標頭籤列與變更詳情抽屜的出身列同款、點籤跳轉正確、規格抽屜內文底部無溯源文字；不符時回報截圖 <!-- speclink-task:tsk_01M1K2Z6BTQAHHNCJM9C9JX8KY -->
