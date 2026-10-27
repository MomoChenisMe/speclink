## Summary

把變更、討論、已封存三種詳情頁的骨架改成手冊式三欄：左欄 240px 文件樹（身分區、文件節點、狀態區）取代內容卡上的分頁列與「狀態」卡，中欄 768px 閱讀卡加上標題區、出處行與數字磚、提示框的容器，右欄 200px 本頁目錄；變更層動作搬到標題列右端，文件層的「開啟 ▾」搬進閱讀卡標題列；十份文件先以現有渲染器落進新骨架，原始 Markdown 與 schema 一個字不動。

## Motivation

- 討論 detail-content-redesign（結論 cut 1）裁定：詳情頁的內容呈現要和手冊頁同一套三欄分工——左欄管頁與頁之間、右欄管頁內——使用者在手冊頁學一次就通。目前詳情頁是「標題列＋狀態卡＋分頁列＋內容卡」直向堆疊，九份文件的段落沒有頁內目錄，文件之間只能靠分頁列切換。
- 討論同時裁定三件版面事：底部不放「上一份／下一份」（變更文件沒有閱讀順序，且與左樹重複）；分析、封存、⋯ 是變更層動作，歸標題列右端與 ‹ n／m › 同列；「開啟 ▾」是文件層動作，歸閱讀卡標題列，不收進 ⋯。
- 這一刀只做骨架，讓 cut 2（變更文件渲染器）與 cut 3（討論文件渲染器）能在同一個殼上平行進行、spec delta 互不重疊；數字磚與提示框先只放容器，內容由後兩刀各自推導。
- 目標使用者：在桌面 app 看變更與討論內容的開發者與 PM；對應 apps/desktop 的詳情頁（packages/ui 的 ChangeDetailPage、DiscussionDetailPage、ArchivedDetailPage 與 apps/desktop 的標題列宿主）。不涉及 CLI 指令、`--json` 輸出、設定欄位與技能檔；規格詳情頁與手冊頁不在本刀範圍。
- 互動原型：https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底三列十張畫板（變更側資料取自 tray-panel-restyle、討論側取自 desktop-ui-redesign），各畫板右側 Tweaks 有深色模式。

## Proposed Solution

- 新增共用骨架元件：三欄外框、左欄文件樹（身分區、「文件」節點清單、「狀態」列）、閱讀卡標題區（24px 標題＋籤＋右端「開啟 ▾」、出處行）、數字磚列與提示框的容器、右欄本頁目錄（沿用規格詳情頁已在用的 ManualToc，掃閱讀卡內的標題錨點，頁寬 ≥1200px 才顯示）。三種詳情頁共用，已封存詳情頁唯讀。
- 左欄身分區承接原標題列與出身列：名稱或 slug（等寬 14px）、階段籤與兩站章、一行出身（建立者、工具、建立日、開工日）、關聯籤各自成行（變更：來自討論籤含「+N」浮層、規格籤；討論：topic 三行截斷與「關聯」一列）。
- 左欄「文件」節點取代分頁列：變更為提案、設計、任務（n/m 徽章）、規格（delta 計數）、排程（波次）、審查與驗證（條件式，末輪序號）；討論為結論、討論過程（輪數）、背景、衍生變更（數）；點節點切換閱讀卡內容，切換中立即高亮並畫 skeleton。左欄「狀態」列承接原狀態卡三列（任務進度、品質關卡、排程；討論為階段步驟列、保留在途、衍生變更）。
- 標題列右端由左而右：‹ n／m › 切換、細線、「分析」（outline）、依階段的主要鈕（實心主色：已就緒「封存」、進行中「退回提案中」、remote 可認領時「認領」）、「⋯」（outline 圖示，封存與刪除紅字項）；頁以 portal 把動作鈕送進宿主標題列的插槽。討論詳情頁的「封存」（concluded 或保留在途時）同樣進標題列；已封存詳情頁標題列無動作。
- 「開啟 ▾」改為 outline 28px，放閱讀卡標題同列右端；排程與審查、驗證沒有對應檔案時不顯示；remote 無 checkout 時只剩「複製內容」的規則不變。
- 內容卡標頭式分頁列（tabs 原語的 card 變體）在詳情頁退場；文件樹以 tabs 原語新增的直向 tree 變體承載文件節點（保留 tab／tabpanel 語意與鍵盤導覽），card 變體仍留給已封存頁的變更／討論切換。GitHub Alert 提示框的樣式抽成共用 Callout 原語，Markdown 的 `> [!NOTE]` 四型與後兩刀的推導提示框共用。
- openspec/LANGUAGE.md 新增「文件樹」「本頁目錄」「數字磚」三條詞彙。
- 相容性影響：只改桌面 app 的呈現層；無 CLI、`--json`、設定欄位、技能檔與 server 變動；既有 vitest 題目依新骨架改錨點，無 golden 回歸對照變動。

## Non-Goals

- 不改任何 artifact 的 Markdown 格式、schema 模板、discuss scaffold；數字磚與提示框的內容推導、各文件的新畫法（提案列式、設計決策卡、任務三層、規格 delta 需求段、排程與工單的新畫法、結論分刀卡、討論過程時間軸、背景籤化、衍生變更分組）屬 cut 2 與 cut 3。
- 規格詳情頁與手冊頁維持現狀；server-web 不受影響。
- 不加底部「上一份／下一份」；不把「開啟 ▾」收進「⋯」。
- 文件樹只列文件節點，不列段落；頁內清單一律由右欄本頁目錄承接。
- 詳情頁的導覽與返回（麵包屑、‹ ›、Esc、滑動返回）、互斥、載入三態的語意不變。

## Alternatives Considered

- 只在內容卡裡加厚、分頁列不動——討論裁定點名手冊頁，三欄分工就是手冊頁的識別。
- 左樹列文件加段落、拿掉右欄——與手冊頁分工相反，20 輪討論塞進左樹會把文件節點推出可視範圍。
- 讓宿主 App.tsx 自己組標題列動作鈕——分析、封存、認領、退回的守門與原因文字全在 ChangeDetailPage，搬到宿主要複製一份；以 portal 把現成的動作區送進標題列插槽，守門不動。
- 一刀連渲染器一起做——約 15 條需求、十幾個元件，兩站審起來太痛；骨架與渲染器分開，後兩刀平行。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增「詳情頁的文件樹、閱讀卡與本頁目錄」需求（三欄骨架、身分區、文件節點、狀態列、本頁目錄、寬度門檻、唯讀變體）；修改「變更詳情頁的標頭與狀態卡」（標題列動作搬到標題列右端、標頭與狀態卡併入左欄身分區與狀態列）、「討論詳情頁檢視與轉出變更」（分頁列改文件節點、狀態卡改左欄狀態列、封存動作進標題列）、「詳情頁的文件動作」（「開啟 ▾」改閱讀卡標題列右端、outline 樣式）、「詳情頁標頭標記受寬度約束且頁不產生水平捲軸」（約束改套在左欄身分區與標題列動作）、「已封存項目以詳情頁檢視」與「已封存詳情頁的工單分頁」（同骨架、文件節點）、「詳情頁的排程分頁」與「詳情頁的工單分頁」（分頁改文件節點，內容不變）、「詳情頁文件載入以 skeleton 呈現」（文件切換中的即時回饋）。

## Impact

- Affected specs: desktop-app（1 條 ADDED、10 條 MODIFIED）
- Affected code:
  - New: packages/ui/src/components/DetailShell.tsx、packages/ui/src/components/ui/callout.tsx、packages/ui/src/__tests__/detailShell.test.tsx、packages/ui/src/__tests__/tabs.test.tsx
  - Modified: packages/ui/src/components/DetailHeader.tsx、packages/ui/src/components/ChangeDetailPage.tsx、packages/ui/src/components/DiscussionDetailPage.tsx、packages/ui/src/components/ArchivedDetailPage.tsx、packages/ui/src/components/Markdown.tsx、packages/ui/src/components/SectionedDoc.tsx、packages/ui/src/components/DeltaBadges.tsx、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/TicketView.tsx、packages/ui/src/components/SourceDiscussionChip.tsx、packages/ui/src/components/ui/tabs.tsx、packages/ui/src/i18n.tsx、packages/ui/src/index.ts、packages/ui/src/__tests__/changeDetailPage.test.tsx、packages/ui/src/__tests__/discussionDetailPage.test.tsx、packages/ui/src/__tests__/archivedDetailPage.test.tsx、packages/ui/src/__tests__/markdownAlerts.test.tsx、packages/ui/src/__tests__/taskList.test.tsx、apps/desktop/src/App.tsx、apps/desktop/src/components/DetailNav.tsx、apps/desktop/src/components/DocumentActionsMenu.tsx、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/remoteCapabilities.test.tsx、openspec/LANGUAGE.md
  - Removed: (none)
