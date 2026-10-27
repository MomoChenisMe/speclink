## Summary

桌面 app 閱讀卡的數字磚改成依格數平分整列，討論過程時間軸改成由新到舊排列；兩項都只改畫面，不動任何 markdown 檔。

## Motivation

目標使用者是在 desktop 透過 AI 代理跑 SDD 流程的開發者（PO／PM 亦同），情境是在詳情頁閱讀變更與討論的文件（propose 之後檢視 artifact、discuss 進行中或收尾後回看討論過程）。

- 數字磚現在固定排一列四欄：提案只出 3 格、任務出 4 格時，同一位置的閱讀卡一個留空欄、一個填滿，看起來像缺了一格資料。
- 討論過程時間軸由舊到新排列：討論越長，最新一輪（使用者最常回看的那一輪、也是「討論已收斂」提示框所指的那一輪）越被推到頁面底部，要捲過全部舊輪才看得到。

兩項由討論 doc-tiles-fill-and-round-order 定案。正式規格原本沒寫數字磚寬度，也沒寫輪的排列方向，所以這次補字釘住新行為，避免日後被改回。

## Proposed Solution

- 數字磚：共用元件 `DocTiles` 由固定四欄改為「有幾格分幾欄、各格等寬、填滿整列」。變更詳情頁、已封存詳情頁、討論詳情頁（含已封存討論）、規格詳情頁、手冊頁五種頁面共用這個元件，一起生效。只有 1 格時同樣填滿整列。
  - 例：提案出 3 格時三格各佔 1/3 列，不留空欄；4 格時與現況相同。
- 討論過程：共用元件 `RoundsDoc` 依輪次由新到舊排列輪卡。討論詳情頁與已封存討論頁共用這個元件，一起生效。右欄本頁目錄照畫面上的標題順序產生，自動跟著倒序。
  - 預設展開維持「第一輪與最後一輪」，倒序後即最上（最新一輪）與最下（Round 1）展開、其餘收合。
  - 例：9 輪的討論由上而下為 Round 9、Round 8 … Round 1，Round 9 與 Round 1 展開。
- 本頁目錄換文件後重掃（apply 期發現的既有問題，順修）：詳情頁切換文件後，右欄本頁目錄會停在前一份文件的錨點——分頁元件換文件時新內容晚一次畫面更新才掛上，目錄剛好在前一次掃描。共用元件 `ManualToc` 改為監看內文的節點增減、一變就重掃。變更、討論、已封存、規格詳情頁與手冊頁共用這個元件，一起生效。正式規格已寫明目錄「自閱讀卡渲染後的 DOM 推導」，不另補字。
- 規格補字（desktop-app 3 條）：
  - 「markdown 文件內容行寬有上限」補「數字磚列內各格等寬、依格數平分並填滿整列」，並加一個 scenario。討論結論原訂補在「詳情頁的文件樹、閱讀卡與本頁目錄」，改放這一條是因為它明列五種頁面（變更、討論、已封存、規格詳情頁與手冊頁）且已規範數字磚列的對齊；前者只涵蓋三種詳情頁，規格詳情頁與手冊頁會漏掉。
  - 「討論輪以卡片呈現」補「依輪次由新到舊排列」，改寫「標準記錄逐輪成卡」的順序並加「新的輪排在最上」scenario。
  - 「詳情頁的文件樹、閱讀卡與本頁目錄」的「討論文件的本頁目錄錨點」scenario 原寫「Round 1 到 Round 20」，改為「Round 20 到 Round 1」。

相容性影響：只改 desktop 與共用 UI 套件的畫面排列。CLI 人眼輸出與 `--json`、討論記錄 markdown、引擎、server API 全部不變；既有使用者不需遷移。

## Non-Goals

- 審查與驗證工單（TicketView）的輪次順序：維持由舊到新、末輪展開，不在本次範圍——討論只要求討論過程。
- 討論記錄 markdown 的輪順序：引擎 `discuss add-round` 仍往檔尾追加，檔案內容不變。
- 分析面板的維度摘要卡（固定四個維度、恆為四張）：不是數字磚，不改。
- 數字磚最多四格的上限、各頁出哪些格：不變。

## Alternatives Considered

- 數字磚最少分 2 欄、單格只佔半列——使用者要依格數填滿，單格長磚可接受。
- 只改提案頁與任務頁——共用元件改一處即全頁一致，加開關是多餘彈性。
- 已封存討論維持舊順序——兩頁共用元件，分開要多一個參數且兩頁閱讀方向不一致。
- 倒序後只展開最新一輪——沿用既有首末輪展開，不改沒被要求的行為。
- 工單輪次一併倒序——使用者只要求討論過程。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「markdown 文件內容行寬有上限」的數字磚列改為各格等寬、依格數填滿整列；「討論輪以卡片呈現」的輪卡改為依輪次由新到舊排列；「詳情頁的文件樹、閱讀卡與本頁目錄」的討論過程本頁目錄跟著改為由新到舊。

## Impact

- Affected specs: desktop-app（3 條 MODIFIED）
- Affected app／package: `packages/ui`（共用 UI 元件，由 apps/desktop 渲染）；speclink-core、speclink-cli、speclink-host、server 皆不受影響
- Affected code:
  - Modified: `packages/ui/src/components/DetailShell.tsx`、`packages/ui/src/components/RoundsDoc.tsx`、`packages/ui/src/components/ManualToc.tsx`、`packages/ui/src/__tests__/detailShell.test.tsx`、`packages/ui/src/__tests__/roundsDoc.test.tsx`、`packages/ui/src/__tests__/discussionDetailPage.test.tsx`、`packages/ui/src/__tests__/changeDetailPage.test.tsx`
  - New: (none)
  - Removed: (none)
