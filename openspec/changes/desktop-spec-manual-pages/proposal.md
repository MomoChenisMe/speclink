## Summary

規格詳情頁整個換成詳情頁共用的三欄骨架（文件樹、閱讀卡、本頁目錄），手冊頁只對齊閱讀卡（頁標題進卡、出處行、數字磚、提示框、15px 內文、「開啟 ▾」）；原始規格與手冊的 Markdown 都不動。來自討論 spec-manual-page-alignment，單一變更，排在 cut 2 `desktop-change-doc-views` 封存之後、與 cut 3 `desktop-discussion-doc-views` 平行。

## Motivation

- 現況：detail-content-redesign 三刀把變更、討論、已封存三種詳情頁改成手冊式三欄閱讀頁，規格詳情頁（`packages/ui/src/components/SpecDetailPage.tsx`）是四種詳情頁裡唯一還是舊骨架的——`DetailHeader` 標頭＋卡片標頭分頁列「規格 N／溯源 M」＋768px 欄左側空白處的需求目錄，內文 16px、行寬 96ch。手冊頁（`packages/ui/src/components/ManualPage.tsx`）本來就是三欄加固定底列，差別都在閱讀卡內部：頁標題與「產生於」固定在卡外、沒有出處行、沒有數字磚、內文 16px、沒有「開啟 ▾」；`DocumentRef` 只有變更、討論、規格、已封存四種，宿主與 Rust 都不認手冊頁。
- 目標使用者：透過 AI 代理跑 SDD 的開發者、PO 與 PM——從規格頁、手冊出處或變更文件的能力籤進到一份正式規格時，要用與其他詳情頁同一套位置（左欄換文件、右欄跳段落、標題下看計數）；讀手冊時要在同一張卡上看到這頁從哪些規格來、產生於何時、是否可能過期，並能開檔案。對應 `/speclink-manual` 產出後的閱讀與規格溯源情境。
- 為什麼是現在：cut 1、cut 2 已於 2026-10-12 封存，cut 2 的規格明寫「規格詳情頁與手冊頁的 markdown 內文維持 16px」「上限維持 96ch」是為了不與本變更重疊而留的例外；討論 spec-manual-page-alignment 的原型兩張畫板（ContentSpecDetail、ContentManual，資料取自 tray-status-menu 與 openspec/manual/tray.md）已由使用者確認。
- 影響的 app 與 crate：`packages/ui`（兩頁、需求段、Markdown 的匯出退場、文案）、`apps/desktop`（`App.tsx` 接線、`adapter/fsActions.ts` 的 locate 分支）、`apps/desktop/core`（`speclink-desktop-core` 新增 `manual_location_at`）與 `apps/desktop/src-tauri`（Tauri command `manual_location` 單行委派）。不改 CLI、不改 `--json`、不改 server——相容性影響為零。

## Proposed Solution

1. **規格詳情頁換骨架**：改用 `DetailShell`——左欄身分區為能力名（等寬＋複製）、「正式規格」籤、出身行「N 條需求 · M 個 scenario」與「最近更新 日期」、「溯源」關聯列（首顆來源變更籤＋「+N」，點擊切到溯源文件）；「文件」節點為規格（需求數徽章）與溯源（來源變更數徽章），溯源節點取代原本的分頁；「狀態」列為需求、溯源、最近更新三列。閱讀卡「規格」：標題＋需求數籤＋「搜尋需求」輸入＋「開啟 ▾」，出處行 `openspec/specs/<capability>/spec.md · 正式規格 · 最近更新 日期（變更）`，數字磚需求／Scenario／溯源變更／最近更新，Purpose 成灰底引言段（標題進本頁目錄），需求段沿用現有畫法並加上 cut 2 的收合規則（第一條展開、其餘收合為一列：Requirement 小標、標題、來源變更與日期、Scenario 數籤）。需求目錄從左側空白處搬到右欄本頁目錄（Purpose＋各需求標題，自渲染後的 DOM 推導，搜尋過濾後只剩命中段）。閱讀卡「溯源」：出處行寫明自 `@trace` 推導、不是檔案，數字磚來源變更／最早／最近／有來源的需求，列規則與現在相同。
2. **需求段的收合模式**：`RequirementBlock` 在非 delta 檢視也能收合（第一條展開、其餘一列），標題帶本頁目錄錨點；規格詳情頁與變更文件的 delta 檢視共用同一個元件、兩種畫法只差色條與移除提示框。
3. **規格詳情頁的提示框**只放推得出的事實：Purpose 為封存流程佔位文字（與引擎 `PURPOSE_TBD_PREFIX` 同一前綴）時 warning「Purpose 待補」，有需求缺 `@trace` 時 note「N 條需求沒有來源變更」列出名稱；兩者皆無時不出現。數字磚、出處行與提示框的推導集中在純函式模組，可獨立測試。
4. **手冊頁對齊閱讀卡**：三欄與固定底列（出處籤、上一頁／下一頁）不動；頁標題與「產生於」從卡外搬進閱讀卡標題區（24px 標題、分區籤、「可能過期」籤、「開啟 ▾」）；出處行 `openspec/manual/<slug>.md · 產生於 時間 · 出處 規格籤`（可點開規格詳情頁，規則同底列）；數字磚章節（h2 數）／小節（h3 數）／關鍵字／出處規格；可能過期時 warning 提示框點名各出處規格的最近 `@trace updated` 與來源變更、本頁產生時間，並指引用 `/speclink-manual` 重生（規格最近更新由頁自行讀該規格全文推出；過期與否仍以索引的 `stale` 為準，不在前端重判）；章節 16px 粗體、內文 15px 行高 1.75；左樹節點改用文件樹同款（30px 列、主色淡底作用中），可能過期的頁在節點尾標「過期」籤。
5. **手冊頁的「開啟 ▾」**：`DocumentRef` 新增手冊頁種類（`{ manual: slug }`），宿主 `locate` 解析為 `openspec/manual/<slug>.md`——`speclink-desktop-core` 新增 `manual_location_at(root, slug)`（與 `manual_page_at` 同一條 slug 守門與路徑解析），Tauri 殼新增 command `manual_location` 單行委派並註冊；remote 無 checkout 時只剩「複製內容」（既有選單規則）。
6. **退場**：規格詳情頁的 `DetailHeader`、`DetailPageFrame`、`DetailContentCard` 與 Markdown 的 `READING_COLUMN_CLS` 失去最後消費者即刪除（`IdentityName`、`StatusRow`、`DocumentRef` 等同檔其餘匯出不動）。
7. **規格**：`desktop-app` 四條 MODIFIED——「桌面 app 呈現 change 與 spec 的清單與內容」（規格詳情頁段落改寫為三欄骨架、溯源文件、數字磚、提示框）、「markdown 內容保留文件結構呈現」與「markdown 文件內容行寬有上限」（規格詳情頁與手冊頁納入 15px 與卡欄上限，不再例外）、「詳情頁的文件動作」（手冊頁納入「開啟 ▾」）；`desktop-manual-page` 兩條 MODIFIED——「內頁渲染與出處跳規格」（標題進卡、出處行、數字磚、15px、節點款式）、「可能過期與未入冊的標示」（籤進籤槽、節點「過期」籤、提示框）。不改「詳情頁的文件樹、閱讀卡與本頁目錄」（規格詳情頁以引用方式共用骨架）——cut 3 `desktop-discussion-doc-views` 改那一條與討論側三條，兩個變更沒有同名需求。

## Non-Goals

- 不改規格頁清單、已封存清單、手冊側欄的搜尋與閱讀序、固定底列與上一頁／下一頁。
- 不改「詳情頁的文件樹、閱讀卡與本頁目錄」需求（避免與 cut 3 同名重疊；規格詳情頁以引用共用骨架）。
- 不在前端重判手冊頁的過期（索引的 `stale` 是唯一真相）；提示框只列各出處規格的最近 `@trace` 時戳與來源變更。
- 不改 Markdown 元件的 96ch 預設（其他消費者沿用），不改 GitHub Alert。
- 不做 remote 模式的手冊；不碰 server-web、CLI 與 `--json`。
- 未入冊規格的標示維持在側欄底部的計數提示（它是規格層的事實，不屬於任一頁的籤槽）。

## Alternatives Considered

- 規格詳情頁只換字級、不換骨架——它是唯一長得不一樣的詳情頁，換字級解不了目錄在左、分頁列在卡頭的差異（討論期否決）。
- 需求目錄留在左側空白處——與其他三種詳情頁相反，左欄管頁與頁之間、右欄管頁內的分工就斷了（討論期否決）。
- 手冊頁拿掉固定底列——desktop-ui-redesign 第 18 輪裁定底列固定、手冊有閱讀順序（討論期否決）。
- 手冊頁不做「開啟 ▾」以避開 Rust——畫板已含且使用者整張接受，少它手冊頁就與其他頁的閱讀卡不同款（討論期否決）。
- 由 Rust 在手冊索引多回一欄「各出處規格的最近 `@trace`」給提示框——多一段 protocol 與索引欄位；改由頁在過期時才讀該規格全文（既有 `getSpecDocument`）推出，Rust 零改動。
- 修改「詳情頁的文件樹、閱讀卡與本頁目錄」把規格詳情頁寫進去——cut 3 改同一條，兩刀會同名重疊；以引用共用骨架即可。

## Capabilities

### New Capabilities

（none）——規格掃描命中 `desktop-app`（規格詳情頁、兩條 markdown 需求、文件動作）與 `desktop-manual-page`（內頁渲染、過期標示）；`manual-pages`（手冊格式契約）不改。

### Modified Capabilities

- `desktop-app`：「桌面 app 呈現 change 與 spec 的清單與內容」（規格詳情頁改三欄骨架：左欄身分區與文件節點、閱讀卡標題區與搜尋需求、出處行、數字磚、Purpose 引言段、需求段收合、右欄本頁目錄、溯源文件的數字磚與出處行、兩種提示框）、「markdown 內容保留文件結構呈現」（規格詳情頁與手冊頁的內文納入 15px，不再維持 16px）、「markdown 文件內容行寬有上限」（規格詳情頁與手冊頁的上限改為閱讀卡內容欄，不再維持 96ch）、「詳情頁的文件動作」（手冊頁的閱讀卡標題列納入「開啟 ▾」，對應 `openspec/manual/<slug>.md`）。
- `desktop-manual-page`：「內頁渲染與出處跳規格」（頁標題與產生時間進閱讀卡標題區、分區籤、出處行、數字磚、15px 內文與 16px 章節、目錄樹節點改文件樹同款）、「可能過期與未入冊的標示」（可能過期籤進閱讀卡籤槽、側欄節點改「過期」籤、warning 提示框列出各出處規格的最近 `@trace` 時戳與來源變更）。

## Impact

- Affected specs: desktop-app（4 條 MODIFIED）、desktop-manual-page（2 條 MODIFIED）
- Affected code:
  - New: packages/ui/src/specSummary.ts、packages/ui/src/__tests__/specSummary.test.ts
  - Modified: packages/ui/src/components/SpecDetailPage.tsx、packages/ui/src/components/ManualPage.tsx、packages/ui/src/components/ManualTree.tsx、packages/ui/src/components/RequirementBlock.tsx、packages/ui/src/components/DetailHeader.tsx、packages/ui/src/components/Markdown.tsx、packages/ui/src/manualDoc.ts、packages/ui/src/specSections.ts、packages/ui/src/i18n.tsx、packages/ui/src/index.ts、packages/ui/src/__tests__/specDetailPage.test.tsx、packages/ui/src/__tests__/manualPage.test.tsx、packages/ui/src/__tests__/specSections.test.ts、packages/ui/src/__tests__/deltaSpecView.test.tsx、apps/desktop/src/App.tsx、apps/desktop/src/adapter/fsActions.ts、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/fsActions.test.ts、apps/desktop/core/src/manual.rs、apps/desktop/src-tauri/src/lib.rs
  - Removed: （none）
