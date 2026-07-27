---
topic: 後台管理主控台重新設計：資訊架構、互動模式與 desktop 風格對齊
slug: admin-console-redesign
status: promoted
promoted_to: admin-console-redesign, drawer-source-chip-overflow
created: 2026-07-25
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 後台管理主控台重新設計：資訊架構、互動模式與 desktop 風格對齊

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者對剛由 `web-service-navigation-redesign`（14/15，僅剩 7.3 終驗）建出的 Server Web Console 提出四點：
(1) 版面不符合後台管理系統要求；(2) 帳號頁一旦進入只剩「上一頁」可退；
(3) 風格最低要求須與 desktop app 一致；(4) desktop 抽屜長標題導致水平捲軸。

**模式**：assumptions——codebase scout 找到 `apps/server-web` 完整 SPA（7 個 admin 頁、3 個 layout、AdminNav、AppRoutes）
與 desktop `App.tsx` 殼，足以形成意見。

**關鍵發現（改變框架）**：色票／字型早已同源——`apps/server-web/src/index.css:5` 與 `apps/desktop/src/index.css:5`
匯入同一份 `packages/ui/src/theme.css`。所以「風格一致」不是視覺 token 問題，而是**版面組成與互動模式**不一致。

**相關變更／規格**：
- `web-service-navigation-redesign`（in-progress 14/15）——本次批評的對象即其產物；經裁定本次重設計**另開新 change**，不 ingest 進該變更。
- `packages/ui`：只有 `Sheet`／`AlertDialog`，**無 Dialog 原語**。
- `openspec/LANGUAGE.md`：原則三「工程詞不出現在使用者可見文案」，明文例外僅設定檔檔名與討論 slug。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-25)

**Focus**: 後台「不像後台」的根因是什麼——視覺 token、還是資訊架構與互動模式？

**Position**: 根因在互動模式與殼的角色裁切，不在視覺 token；五項假設經使用者全數確認，另裁定本次重設計開新 change。
- **視覺已同源**：兩端 `index.css` 皆 `@import packages/ui/src/theme.css`，青綠主色、圓角、字型共用。調 padding／字級救不了。
- **病灶＝所有東西都是常駐表單**：`RegistryPage.tsx`(282 行) 把「建立表單＋列表＋每列常駐更名 input＋新增 repo 表單」平鋪；`UsersPage.tsx`(390 行) 把邀請表單常駐於表格下方，並把「選擇專案／角色／新增」三控制項塞進表格列。Desktop 語彙相反：列表／卡片 → 右側抽屜看細節與動作（`RichDetailDrawer.tsx`）。
- **建立／編輯走 Sheet 抽屜而非 Dialog**：`packages/ui/src/index.ts` 只匯出 `Sheet`／`AlertDialog`，無 Dialog 原語；走 Sheet 既免新增原語、又與 desktop 同語彙，`AlertDialog` 留給破壞性確認。
- **帳號死路＝殼選錯**：`AppRoutes.tsx:157-165` 讓 `/account` 一律套無側欄的 `AccountLayout`，管理員由 `AdminNav.tsx:52-61` 進入即失去全站導覽。修法為依角色決定殼（admin 仍用 `AdminLayout`），並把「帳號」自側欄底部移至右上使用者選單（現 header 僅一顆登出，看不出自己是誰）。
- **總覽是裝飾品**：`OverviewPage.tsx` 僅 45 行、6 張巨卡佔滿寬度、整頁約 85% 空白，數字不可點、無時間維度。應「數字即入口」＋下半頁放可行動內容。
- **詞彙漂移**：「建立 project」「Project key」「Repo key」「Personal Access Tokens」「Web Sessions」「Schema 版本」皆為使用者可見文案中的工程詞，不在 `LANGUAGE.md` 明文例外內。
- **範圍裁定**：`web-service-navigation-redesign` 僅剩 7.3 終驗，本次重設計另開新 change，不把快收工的變更再撐大。

**Ruled out**:
- 「換色票／調間距」——兩端 theme 早已同源，視覺層無可換之物。
- 「帳號頁加一顆返回鈕」——治標；admin 在該頁仍失去全站導覽。
- 「新增 Dialog 原語承載建立表單」——會在兩端養出「彈窗 vs 抽屜」雙心智模型，且 Sheet 已足夠。
- 「ingest 進 web-service-navigation-redesign」——該變更 14/15，撐大它會拖延終驗。

**Open**: 改造深度未定——保留現有七頁只換互動模式，或連資訊架構一併重排（例如合併憑證／資料操作／系統狀態）？

### Round 2 — assumptions (2026-07-25)

**Focus**: 改造深度——保留現有七頁只換互動模式，還是連資訊架構一併重排？

**Position**: 採中度重排：資訊架構只動一刀（七個目的地降為六個），其餘靠互動模式與殼的修正解決；完整 ASCII 版面已逐頁定案並經使用者確認。
- **唯一的 IA 刀**：合併「資料操作」＋「系統狀態」為「系統」——兩頁各印一份 Store 驅動／契約／等級／能力／健康，是兩份真相。合併後區段為 執行環境／儲存狀態／匯出／危險區（遷移）。
- **憑證頁保留獨立**：與「使用者抽屜的憑證分頁」雖重疊，但跨使用者稽核（一眼看完全站金鑰）是獨立用途；改以分頁（存取金鑰｜裝置）取代兩塊堆疊區段。
- **殼**：admin 與帳號共用同一個殼、依角色裁切——admin 進 `/account` 側欄整條保留且無高亮，一般成員同殼但不渲染側欄。「帳號」移出側欄，改由右上使用者選單（頭像＋email → 帳號／登出）進入，順帶解決 header 只有一顆登出、看不出自己是誰。
- **對齊 desktop 的三個數字**：header `h-14 → h-12`、側欄 `w-56 → w-[200px]`、main `px-4 py-6 → p-5`（值取自 `apps/desktop/src/App.tsx:407,482,527`）。
- **互動模式**：列表為主體，建立／編輯一律右滑 Sheet 抽屜（＋邀請使用者、＋建立專案、使用者細節、專案細節），破壞性動作走 AlertDialog；表格列不含任何輸入控制項，整列可點開抽屜。
- **不可變欄位視覺分離**：專案／儲存庫代號建立後不可改，抽屜內以唯讀樣式呈現；更名走「按鈕 → 才變輸入框」，取代常駐 input。
- **總覽**：6 張巨卡 → 4 張可點的緊湊卡（使用者／專案／憑證／待啟用），下半頁為 需要處理（無事項整塊不出現）／系統健康／最近活動；「Schema 版本」不再充當 KPI。
- **空狀態**：不再只印一行「尚無 X。」，改為 圖示＋一句「為什麼你需要它」＋主行動鈕。
- **窄螢幕**：表格改卡片列不橫捲，抽屜改全寬。
- **詞彙替換表**（`LANGUAGE.md` 現僅 11 條，以下皆為新增詞條而非改既有正典）：建立 project→建立專案、Project key→專案代號、Repo key→儲存庫代號、Personal Access Tokens／PAT→存取金鑰、Web Sessions→登入工作階段、Schema 版本→資料結構版本、Store 狀態／驅動／等級／能力→儲存後端…、Outbox backlog→待送佇列。
- **架構縫接深度檢查**：本次無新 IPC、無新儲存抽象；唯一可能新縫是「把殼抽到 `packages/ui` 共用」——**判定不抽**，刪除測試過不了（刪掉後兩端各留自己的殼，什麼都不會壞，該層只轉發版面）。對齊靠三個數字＋已同源的 `theme.css`。`NavItem` 因藏有 active／inactive class 矩陣（已實際漂移）而有抽的價值，列為選配。

**Ruled out**:
- 「保留七頁只換互動模式」——會讓 Store 健康的兩份真相原封不動留著。
- 「連憑證頁一併砍成五個目的地」——跨使用者稽核視角會消失。
- 「抽 AppShell 到 packages/ui」——未通過刪除測試，是純轉發的淺縫。
- 「新增 Dialog 原語」——沿用第一輪裁定，Sheet 已足夠且與 desktop 同語彙。

**Open**: 無。抽屜水平捲軸 bug（`RichDetailDrawer.tsx:303-313` chip 缺 `max-w`／`truncate`，`ArchivedDrawer.tsx:141` 疑同病）經裁定切為獨立變更，不併入本次重設計。

## Conclusion

**Decision**: 重做 Speclink Server Web Console 的版面組成與互動模式，使其符合後台管理系統慣例並與 desktop app 對齊；資訊架構只動一刀（合併「資料操作」＋「系統狀態」為「系統」，七個目的地降為六個），其餘改動集中在殼、互動模式與文案。範圍拆為兩個變更：

1. **admin-console-redesign**（本討論主體）
   - 殼：admin 與帳號共用同一個殼、依角色裁切；admin 進 `/account` 側欄整條保留且無高亮，一般成員同殼但不渲染側欄——解決「帳號頁只能上一頁」的死路。
   - 「帳號」移出側欄，改由右上使用者選單（頭像＋email → 帳號／登出）進入。
   - 對齊 desktop 三個數字：header `h-12`、側欄 `w-[200px]`、main `p-5`。
   - 互動模式：列表為主體；建立／編輯一律右滑 Sheet 抽屜；破壞性動作走 AlertDialog；表格列不含輸入控制項，整列可點開抽屜。
   - 總覽：6 張巨卡 → 4 張可點緊湊卡＋需要處理／系統健康／最近活動；「Schema 版本」降級不再充當 KPI。
   - 合併「資料操作」＋「系統狀態」為「系統」（區段：執行環境／儲存狀態／匯出／危險區）。
   - 憑證頁保留獨立，改以分頁（存取金鑰｜裝置）取代兩塊堆疊區段。
   - 代號（project key／repo key）建立後不可改，抽屜內以唯讀樣式呈現；更名走「按鈕 → 才變輸入框」。
   - 空狀態改為 圖示＋一句「為什麼你需要它」＋主行動鈕；窄螢幕表格改卡片列、抽屜全寬。
   - 詞彙收斂並回寫 `LANGUAGE.md`（新增詞條）。
   - 完整 ASCII 版面（殼／總覽／使用者＋兩個抽屜／專案與儲存庫＋抽屜／憑證／系統／稽核／帳號／窄螢幕）須原樣寫入 proposal。

2. **抽屜水平捲軸修正**（獨立小變更，先修）
   - `packages/ui/src/components/RichDetailDrawer.tsx:303-313` 的來源討論 chip 直接塞整段 `src.topic`，無 `max-w`／`truncate`；外層 `flex-wrap` 救不了「單一子項寬於容器」，而 `SheetContent`（`ui/sheet.tsx:36`）帶 `overflow-y-auto`，CSS 連帶把 x 軸算成 auto → 橫向捲軸。修法：chip 加 `max-w-full truncate` 並以 `title` 保留全文。`ArchivedDrawer.tsx:141` 同結構須一併檢查。

**Rationale**: 色票與字型早已同源（兩端 `index.css` 皆 `@import packages/ui/src/theme.css`），所以「風格不一致」的實體是版面組成與互動模式，不是視覺 token——這決定了整個變更的重心放在殼與抽屜語彙，而非樣式微調。IA 只動一刀是因為唯一站得住的合併理由是「同一件事兩頁兩份真相」，其餘分頁各有獨立用途。

**Rejected alternatives**:
- 換色票／調間距——兩端 theme 同源，視覺層無可換之物。
- 帳號頁加返回鈕——治標，admin 在該頁仍失去全站導覽。
- 新增 Dialog 原語承載建立表單——會養出「彈窗 vs 抽屜」雙心智模型，Sheet 已足夠且與 desktop 同語彙。
- ingest 進 `web-service-navigation-redesign`——該變更 14/15 僅剩終驗，撐大它會拖延收工。
- 保留七頁只換互動模式——Store 健康的兩份真相會原封不動留著。
- 砍掉憑證頁成五個目的地——跨使用者稽核視角會消失。
- 抽 AppShell 到 `packages/ui`——未通過刪除測試，是純轉發的淺縫；`NavItem` 列為選配。

**Deferred**: `NavItem` 是否抽到 `packages/ui` 共用（有 active／inactive class 矩陣漂移的實據，但非本次必要）——留待實作時視重複程度決定。

**Capture to**: proposal（兩個變更）＋ design（殼角色裁切、抽屜語彙、IA 合併理由）＋ specs（導覽、帳號殼、管理頁互動契約）＋ LANGUAGE.md（詞彙漂移：建立 project／Project key／Repo key／Personal Access Tokens／Web Sessions／Schema 版本／Store ＊／Outbox backlog 皆為使用者可見文案中的工程詞，不在明文例外內）

**Next**: /speclink-propose --from-discussion admin-console-redesign
