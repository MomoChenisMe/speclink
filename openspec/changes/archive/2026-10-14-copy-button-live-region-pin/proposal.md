## Why

在 desktop 用變更詳情頁追任務的開發者（PO／PM 或透過 AI 代理跑 SDD 的工程師，apply 階段逐條勾任務、找手動任務時），點手動任務提示框的「前往」、工具列「下一個未完成」或按 n 鍵，目標列在清單尾端時，整個詳情頁（左欄文件樹與閱讀區）會被往上推，主區底部露出一塊白底，而且推上去後不會自己回來。看板卡片越多，位移越大（回報的專案約 180px）。

成因：
- 詳情頁開著時，宿主頁（看板）保持掛載但不可見（desktop-app「變更與討論詳情頁的宿主為看板」）。
- 看板卡片的複製名稱鈕（共用元件 CopyButton）旁有一個給螢幕閱讀器朗讀「已複製」的隱藏區塊，它是 position:absolute。它的定位容器在看板欄與 grid 的捲動區之外，所以它不被捲動區裁切，停在欄內容深處的靜態位置，把主區 main 撐出多餘的可捲高度。main 設的是 overflow:hidden，使用者捲不動它，程式仍然可以捲。
- 任務列表定位時用 scrollIntoView 把目標列捲到中央。閱讀區捲到底仍不夠時，瀏覽器會接著捲 main，main 裡的詳情頁層就跟著上移。
- 這個朗讀區是 2026-10-08 的 desktop-design-foundation 加進來的，所以這個問題是最近才出現。

這個成因已在原生 WKWebView（與 Tauri 同引擎）以真實元件重現：main 的可捲高度恰等於最低一個朗讀區的底緣；把朗讀區釘在左上角後，位移消失。

## What Changes

- 共用元件 CopyButton 的隱藏朗讀區加上 top-0 與 left-0，釘在它定位容器的左上角。這樣它不再停在捲動區外的深處，也就不會撐高外層容器。朗讀內容與時機不變。
- 看板的捲動區（各欄卡片區、討論欄的已轉出展開區、整個 grid）加上 relative，成為定位容器。日後卡片裡再加沒釘位置的 absolute 元素，也只會被所在的捲動區裁切，不會跳出去撐高 main。這條來自品質關卡 review round 1 的建議，從版面結構上根治同一類問題。
- 修正一次涵蓋所有定位捲動入口（「前往」、「下一個未完成」、n 鍵、本頁目錄、D 籤跳轉），也涵蓋 desktop 與 server-web 共 15 處 CopyButton。
- desktop-app「變更與討論詳情頁的宿主為看板」補一個情境：宿主頁的內容不推動詳情頁外框，定位捲動後主區底部沒有空白。
- CopyButton 單元測試補一條：朗讀區帶 top-0 與 left-0。看板測試補一條：grid、各欄卡片區與已轉出展開區都帶 relative。

相容性影響：不新增也不變更任何 CLI 指令、旗標、`--json` 輸出、設定欄位或生成的技能；只改共用元件庫裡四個元件的 class。

## Non-Goals

- 不改 apps/desktop/src/App.tsx 的 main 與詳情頁層外框。把 main 改成 overflow:clip 已實測更糟：main 不再是捲動區、被內容撐高，連標題列一起被推上去；舊版 macOS 的 WebKit 也不支援 clip。
- 不改寫 TaskList、ManualToc、DesignDoc 等處的 scrollIntoView 為只捲閱讀區。來源修掉後外層已無可捲高度；日後若又出現跳出裁切的元素，再加這層保護。
- 不用 relative 外框包住 CopyButton：呼叫端傳入的 className 會落到內層按鈕，改變既有版面。
- 不新建 desktop 的自動化版面測試（e2e）。jsdom 不算版面，實際位移由手動任務在 Mac 實機確認。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「變更與討論詳情頁的宿主為看板」補「宿主頁不推動詳情頁外框」情境。掃描到的相關規格只有 desktop-app（宿主頁保持掛載、手動提示框「前往」、「下一個未完成」都在這裡）；server-web-console 也用同一個 CopyButton，但沒有宿主頁疊詳情頁的版面，行為不變、不需要 delta。

## Impact

- Affected specs: desktop-app
- Affected apps: packages/ui（共用元件庫）；受惠者為 apps/desktop 與 apps/server-web，兩者程式不改
- Affected code:
  - Modified: packages/ui/src/components/CopyButton.tsx、packages/ui/src/__tests__/copyButton.test.tsx、packages/ui/src/components/BoardColumn.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/__tests__/discussionColumn.test.tsx
