---
topic: desktop 詳情頁點「前往」任務後，主區底部出現白色空白區塊（整個詳情頁被往上推）
slug: detail-locate-white-gap
status: promoted
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: copy-button-live-region-pin
---

# Discussion: desktop 詳情頁點「前往」任務後，主區底部出現白色空白區塊（整個詳情頁被往上推）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報 desktop 變更詳情頁的任務文件裡，任務 5.4（未勾的手動任務、展開中）位在清單尾端，畫面上整個詳情頁（左欄文件樹與閱讀區）被往上推約 180px，主區底部露出一塊白底；標題列與專案欄不動。截圖專案為 wadpilot（看板卡片多）。需求本身可驗證（定位後詳情頁外框不位移、無白底），不需要 grill 階段，直接以假設推進。
相關規格與變更：desktop-app（「任務分頁提供批次操作工具列」的下一個未完成、「任務群組標題與章節標籤同款式」的手動提示框「前往」、「變更與討論詳情頁的宿主為看板」的宿主頁保持掛載但不可見）；進行中的 delta：manual-callout-unchecked-only: desktop-app、desktop-swipe-back-finger-lift: desktop-app、review-rules-config: desktop-app。
重現方式：scratchpad 以 vite 掛真實 ChangeDetailPage＋KanbanBoard、仿 App.tsx 的 main／詳情頁層外框，用原生 WKWebView（swiftc 編的小程式送 NSEvent 點擊）實測；Chromium 無頭瀏覽器同樣可跑。
Prior discussions: archived-page-fill-height, tray-panel-scroll-and-vibrancy

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: 白底的成因與修法落點
**Position**: 成因是 CopyButton 的隱藏朗讀區（sr-only、position:absolute）跳出看板欄的捲動裁切、把 main 撐高，修法是在 CopyButton 把它釘在定位容器左上角；使用者確認四條假設全對。
- 成因鏈：詳情頁開著時看板仍掛載但隱形（desktop-app「變更與討論詳情頁的宿主為看板」）→ 看板卡片的複製鈕旁有 `<span role="status" aria-live="polite" class="sr-only">`（packages/ui/src/components/CopyButton.tsx:66）→ absolute 元素的定位容器在欄與 grid 捲動區之外，靜態位置落在欄內容深處、不被裁切 → main（overflow-hidden，程式仍可捲）的 scrollHeight 大於 clientHeight → TaskList highlightRow 的 scrollIntoView({block:"center"}) 在閱讀區捲到底後續捲 main → 詳情頁層（main 內 absolute inset-0）跟著上移、底部露白
- 實測（WKWebView）：假看板讓 main scrollHeight 890／clientHeight 852，點「前往」後 main.scrollTop=38、詳情頁上移 38px；main 的 scrollHeight 恰等於最低一個 sr-only span 的底緣；換成真實專案卡片多時位移更大（截圖約 180px）
- 引入點：2026-10-08 bdf1d7cc（desktop-design-foundation）為 CopyButton 加上朗讀區
- 修法：朗讀區加 `top-0 left-0`；實測 19 個朗讀區釘住後 main 無多餘高度、「前往」後詳情頁不位移；朗讀不受位置影響；15 處 CopyButton（desktop 與 server-web）一併受惠
- 看板其他 absolute 元素（搜尋列圖示、篩選浮層、封存落點浮層）都有自己的 relative 外框，不跑出裁切
- 規格：在「變更與討論詳情頁的宿主為看板」補一個情境（宿主頁內容不推動詳情頁外框）；避開「任務群組標題與章節標籤同款式」，因 manual-callout-unchecked-only 正改寫它
- 測試：jsdom 不算版面，自動測試只斷言朗讀區帶 top-0／left-0；另加一個 [M] 由使用者在 Mac 實機確認；desktop 沒有 e2e（apps/desktop/e2e 不存在）
**Ruled out**: main 改 overflow:clip——實測 main 不再是捲動區、被內容撐高，連標題列一起被推上去，且舊版 macOS WebKit 不支援 clip；改寫 TaskList 等 6 處以上的 scrollIntoView 為只捲閱讀區——來源修掉後外層無可捲高度，屬額外工；用 relative span 包住 CopyButton——傳入的 className 會落到內層按鈕、改變既有版面

## Conclusion

**Decision**: 在共用元件 CopyButton 把隱藏朗讀區（`<span role="status" aria-live="polite" class="sr-only">`）釘在定位容器左上角（加 `top-0 left-0`），讓它不再跳出捲動裁切撐高 desktop 的 main，「前往」等定位捲動就推不動詳情頁。
- 改動只在 packages/ui/src/components/CopyButton.tsx 的朗讀區 class；朗讀行為不變；desktop 與 server-web 的 15 處 CopyButton 一併受惠
- 不改 apps/desktop/src/App.tsx 的 main／詳情頁層外框，不改 TaskList、ManualToc、DesignDoc 等處的 scrollIntoView
- 規格：desktop-app「變更與討論詳情頁的宿主為看板」補一個情境——看板卡片多到欄內需要捲動時，於變更詳情頁任務文件點清單尾端手動任務的「前往」，閱讀區捲到底，詳情頁外框（主區標題列、左欄文件樹頂端的變更名、閱讀區上下緣）位置不變、主區底部無空白
- 測試：packages/ui 的 CopyButton 單元測試斷言朗讀區帶 top-0 與 left-0（jsdom 不算版面，只能鎖 class）；另加一個 [M] 任務，由使用者在 Mac 以 root `npm run dev:desktop` 對卡片多的專案點「前往」確認
- 範例：看板有 30 張卡、打開某變更的任務文件、點最後一個手動任務的「前往」→ 閱讀區捲到底、該列短暫高亮；文件樹頂端仍看得到變更名，主區底部沒有白底
**Rationale**: 位移需要外層有可捲高度，而那段高度全來自跳出裁切的朗讀區；從來源釘住它，一行 class 就讓所有定位捲動（前往、下一個未完成、n 鍵、目錄、D 籤跳轉）都不再推動外框，不必逐一改呼叫點，也不碰版面結構。
**Rejected alternatives**:
- main 改 overflow:clip — 實測 main 不再是捲動區、被內容撐高，連標題列一起被推上去；舊版 macOS 的 WebKit 也不支援 clip
- 把 6 處以上的 scrollIntoView 改成只捲閱讀區 — 來源修掉後外層沒有可捲高度，屬額外工；日後再出現跳出裁切的元素時再加這層保護
- 用 relative span 包住 CopyButton — 呼叫端傳入的 className 會落到內層按鈕、改變既有版面
- 補自動化版面測試（e2e） — desktop 沒有 e2e 基建，為一行修正新建一套成本過高
**Deferred**: none
**Capture to**: proposal | spec | tasks
**Next**: /speclink-propose --from-discussion detail-locate-white-gap
