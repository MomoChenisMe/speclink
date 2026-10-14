---
topic: 詳情頁觸控板滑動返回後有一小段延遲
slug: swipe-back-delay
status: promoted
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: desktop-swipe-back-finger-lift
---

# Discussion: 詳情頁觸控板滑動返回後有一小段延遲

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 desktop 詳情頁用觸控板兩指向右滑動返回，指示轉實心、放開後要再等一小段時間才回到宿主頁，想知道原因並參考 Codex app 的做法改善。目標明確（去掉滑動返回的延遲），不需要 grill 階段，直接以假設推進。相關正典：desktop-app 的需求「詳情頁的導覽與返回」（觸控板滑動返回的指示、門檻、手勢停止時返回）與設定模式返回那條需求（引用同一套規則）；相關封存變更：desktop-readonly-detail-pages（design D8 定下以「最後一個 wheel 事件後靜止 120ms」判定手勢結束、含慣性尾段，並排除 WKWebView 原生前後頁滑動）；目前沒有在途變更。
Prior discussions: spec-manual-page-alignment, desktop-ui-redesign, remote-login-ux-gaps, admin-console-redesign, desktop-導覽與專案首頁重構

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: 延遲從哪裡來、修法有哪幾條路，以及 Codex app 怎麼做
**Position**: 延遲來自手勢結束的判定方式；Codex 用原生層拿「手指放開」訊號，方向傾向照這個做（修法 C）。
- 原因（事實）：DOM wheel 事件沒有手指放開的相位，SwipeBack.tsx 以「最後一個事件後靜止 120ms」判定結束（IDLE_MS，每個事件都重設計時器）；macOS 慣性捲動在手指離開後仍約每 16ms 送一個事件，延遲＝慣性尾段＋120ms，快甩最久
- 返回動作本身沒有延遲：closeDetailPage 是同步狀態切換，宿主頁保持掛載（inert）
- 決策樹：A 一過門檻就返回（純前端，失去過門檻後反悔，需改規格「手勢停止時」字面）／B 前端猜慣性開始／C 原生層提供手指放開訊號（保留反悔，規格不用改）
- Codex 實查（/Applications/ChatGPT.app 的 app.asar）：前端只把 canGoBack、canGoForward、指示範圍與主色送給主程序，主程序呼叫自家 Electron 分支的 BrowserWindow.setHistorySwipeNavigationState；手勢辨識、指示繪製與放開判定都在原生層，所以指示跟手、放開立刻消失
- Codex 只在 macOS 開這個功能（platform 判斷＋功能旗標），Windows 沒有滑動返回
- Tauri 可行性：Cargo.lock 已有 objc2 0.6.4、objc2-app-kit 0.3.2、block2 0.6.2（經 tauri 帶入），Rust 端可用 NSEvent 本地監聽讀 phase／momentumPhase
**Ruled out**: 調小 IDLE_MS — 慣性事件約每 16ms 一個，門檻多小都會被一直重設；B 前端猜慣性 — 「手指放慢」與「慣性遞減」的事件形狀幾乎一樣，會偶發太早或太晚觸發；WKWebView 原生前後頁滑動 — D8 已排除（要寫瀏覽歷史、呈現是整頁滑動不是左緣指示），本輪沒有新理由重開
**Open**: 確認採 C 且指示留在前端畫（不像 Codex 連指示都原生畫）；Windows 保留 120ms 判定還是另做；放開後剩下的慣性事件要不要吞掉、吞到什麼時候

### Round 2 — interview (2026-10-14)

**Focus**: 採修法 C 時，指示要像 Codex 一樣由原生層畫，還是留在前端
**Position**: 採 C，原生層只提供「手指按下／放開」訊號，指示與所有規則留在前端（使用者同意）。
- Rust 端：desktop crate 一個只在 macOS 編譯的小模組，以 NSEvent 本地監聽讀捲動事件的 phase，手指按下（Began／MayBegin）與放開（Ended／Cancelled）各送一個事件給前端
- 前端：SwipeBack.tsx 收到放開事件立刻結算（達門檻返回、未達收回），不再等 120ms；指示外觀、門檻、對話框開著不返回、寬表格先捲等規則照舊
- 慣性事件的處理（預設、不另問）：只在原生的按下與放開之間累計位移，放開後到下一次按下之間的 wheel 事件一律忽略，所以慣性尾段不會延後結算，也不會重新開始一個手勢
- 沒有原生相位的輸入（例如一般滑鼠的水平滾輪）收不到按下事件，退回現在的 120ms 判定
- 介面深度：一層轉接；藏住 macOS 手勢階段與慣性細節；刪掉只會退回 120ms 判定
**Ruled out**: 指示由原生層畫（照抄 Codex） — 指示、門檻、對話框與捲動區塊規則都要在 Rust 重寫，Codex 能這樣做是因為它的分支 Electron 本來就提供這個功能；修法 A 一過門檻就返回 — 會失去過門檻後反悔、還要改規格，C 不犧牲這點就能讓延遲歸零
**Open**: Windows 保留 120ms 判定還是另做；規格「手勢停止時」要不要明寫成「手指離開觸控板時、不等慣性」

### Round 3 — interview (2026-10-14)

**Focus**: Windows 要不要保留觸控板滑動返回
**Position**: Windows 拿掉觸控板滑動返回，只有 macOS 保留，並照 Codex 的方式以原生的手指放開訊號結算（使用者裁定）。
- 依據：Codex 的 Windows 版沒有滑動返回——前端只在 macOS 且功能旗標開啟時回報可返回，主程序的原生呼叫前有 process.platform === "darwin" 判斷；bundle 內也沒有 app-command／browser-backward 的處理
- 前端以既有的 detectPlatform()（App.tsx:542）把 SwipeBack 限定在 macOS 啟用，Windows 不顯示指示也不返回
- Windows 的返回改由 Esc、Alt+← 與滑鼠側鍵「上一頁」承擔，這三者照舊、各平台都有
- 規格要改：desktop-app 的「詳情頁的導覽與返回」與設定模式返回那條需求，觸控板滑動返回改為只在 macOS、且以手指離開觸控板為結算時機
**Ruled out**: Windows 保留 120ms 判定 — 使用者選擇與 Codex 對齊；Windows 的延遲無法實測，鍵盤與滑鼠側鍵已能立即返回，保留一條跑不準的路徑沒有實際好處
**Open**: macOS 上沒有手勢階段的輸入（滑鼠水平滾輪、Shift＋滾輪）要不要也拿掉、不再退回 120ms 判定

### Round 4 — interview (2026-10-14)

**Focus**: 最終行為表與 Codex 實際規則逐項比對，以及 macOS 上沒有手勢階段的輸入要不要拿掉
**Position**: 行為表五列都與 Codex 一致，使用者條件式同意（對就 OK）；滑鼠水平滾輪拿掉，並補齊兩條 Codex 有、表上沒寫的規則。
- 依據：Codex Framework 的 binary 含 Chromium HistorySwiper 的全部方法與欄位名（handleScrollWheelEvent:、touchesEndedWithEvent:、_firstScrollUnconsumed、_overscrollTriggeredByRenderer 等）與 isSwipeTrackingFromScrollEventsEnabled，推定沿用 Chromium chrome/browser/renderer_host/chrome_render_widget_host_view_mac_history_swiper.mm 的規則，只把指示換成自家 OwlHistorySwipeOverlayView；OpenAI 有沒有改數值無法從 binary 確認
- 放開即結算：touchesEndedWithEvent 時進度達 1 就導覽、隨即收掉指示，不等慣性
- 放開前往回滑：進度以目前位置對起點計算，放開時未達 1 就收回不導覽
- 滑鼠滾輪：phase 與 momentumPhase 都是 None 的事件直接不處理，Shift＋滾輪同屬此類；Magic Mouse 有手勢階段，照樣支援
- 慣性：慣性事件永遠不會開始一次滑動，放開後的慣性事件被吞掉
- 補齊一：尊重 macOS 系統設定「在頁面之間滑動」（NSEvent.swipeTrackingFromScrollEventsEnabled 為否就不觸發）
- 補齊二：手勢累計的垂直移動明顯大於水平時，整次手勢取消直到手指放開（Chromium 用 yDelta > 2×xDelta 等條件）
- 刻意不對齊：Codex 的門檻是手指在觸控板上移動 8% 寬度，我們維持 120px 捲動量；要對齊得在原生層讀觸控點
**Ruled out**: macOS 上沒有手勢階段的輸入退回 120ms 判定 — Codex 不處理一般滑鼠滾輪，現況 Shift＋滾輪會誤觸返回，拿掉後計時器可整個刪除
**Open**: 門檻改用手指實際移動距離——延後，見結論

## Conclusion

**Decision**: 觸控板滑動返回只在 macOS 保留，改由原生層提供「手指按下／放開」訊號、放開即結算，規則對齊 Codex（Chromium HistorySwiper）；Windows 拿掉觸控板滑動返回。
- 延遲原因：DOM wheel 事件沒有手指放開的相位，SwipeBack.tsx 以「最後一個事件後靜止 120ms」判定結束，macOS 慣性捲動在手指離開後仍約每 16ms 送事件、一直重設計時器，延遲＝慣性尾段＋120ms；返回動作本身（closeDetailPage）是同步的
- Rust 端：desktop crate 新增只在 macOS 編譯的小模組，以 NSEvent 本地監聽讀捲動事件的 phase，手指按下（MayBegin／Began）與放開（Ended／Cancelled）各送一個事件給前端，並帶上系統設定「在頁面之間滑動」是否開啟（NSEvent.swipeTrackingFromScrollEventsEnabled）
- 相依：objc2、objc2-app-kit、block2 只加在 macOS target，版本對齊 Cargo.lock 中經 tauri 帶入的 0.6.4／0.3.2／0.6.2
- 前端：SwipeBack.tsx 只在原生的按下與放開之間累計位移；收到放開事件立刻結算（達門檻返回、未達收回），120ms 計時器整個移除；指示仍由前端畫，外觀、120px 門檻、對話框開著不返回、可水平捲動的區塊先捲等規則照舊
- 對齊 Codex 的規則：放開即結算、不等慣性；放開前往回滑到門檻以下就收回不返回；慣性事件永遠不會開始一次滑動，放開後的慣性事件一律忽略；沒有手勢階段的輸入（一般滑鼠水平滾輪、Shift＋滾輪）不觸發；Magic Mouse 有手勢階段，照樣支援；系統設定「在頁面之間滑動」關閉時不觸發；手勢累計的垂直移動明顯大於水平時，整次手勢取消直到手指放開
- 平台：以既有 detectPlatform()（App.tsx:542）把滑動返回限定在 macOS；Windows 不顯示指示也不返回；Esc、⌘[／Alt+←、滑鼠側鍵「上一頁」各平台照舊立即返回
- 規格：desktop-app 的「詳情頁的導覽與返回」與設定模式返回那條需求（spec.md:855 引用同一套規則）改為只在 macOS、以手指離開觸控板為結算時機，並寫入上列規則；既有 Scenario「觸控板滑動返回」沿用原名修改內容，另增快甩放開立即返回、Shift＋滾輪不觸發、Windows 不觸發、系統設定關閉不觸發等情境
- 驗收例：macOS 快甩 200px 放開→放開當下返回；滑到 150px 再往回到 80px 放開→收回不返回；macOS Shift＋滾輪→無指示不返回；Windows 觸控板向右滑→無指示不返回；任何平台按 Esc、⌘[／Alt+←、滑鼠側鍵→立即返回
- 介面深度：一層轉接（監聽→送事件）；藏住 macOS 手勢階段與慣性細節；刪掉這個模組，macOS 會失去觸控板滑動返回，其他返回方式不受影響
**Rationale**: 延遲的根源是網頁層拿不到「手指放開」這個訊號，任何純前端做法都只能用猜的（等靜止、猜慣性）或犧牲反悔（一過門檻就返回）；Codex 的答案是讓原生層直接提供這個訊號，而 Tauri 的 Rust 端用 NSEvent 就拿得到，需要的 objc 套件也已在相依樹裡。指示與規則留在前端，原生層只多一條很薄的訊號通道，成本最小；Windows 沒有對應訊號、Codex 也不做，鍵盤與滑鼠側鍵已能立即返回，因此拿掉而不是保留一條跑不準的路徑。
**Rejected alternatives**:
- 調小 IDLE_MS — 慣性事件約每 16ms 一個，門檻多小都會被一直重設
- 一過門檻就立刻返回（純前端） — 失去過門檻後往回滑反悔，且仍要改規格；原生訊號不犧牲這點就能讓延遲歸零
- 前端猜慣性開始 — 「手指放慢」與「慣性遞減」的事件形狀幾乎一樣，會偶發太早或太晚觸發
- 指示也由原生層畫（照抄 Codex） — 指示、門檻、對話框與捲動區塊規則都要在 Rust 重寫；Codex 能這樣做是因為它的分支 Electron 本來就提供
- Windows 保留 120ms 判定 — 使用者選擇與 Codex 對齊；Windows 的延遲無法實測，鍵盤與滑鼠側鍵已能立即返回
- macOS 上沒有手勢階段的輸入退回 120ms 判定 — Codex 不處理一般滑鼠滾輪；現況 Shift＋滾輪會誤觸返回
- WKWebView 原生前後頁滑動 — desktop-readonly-detail-pages design D8 已排除（要寫瀏覽歷史、呈現是整頁滑動不是左緣指示），本次沒有新理由重開
**Deferred**:
- 門檻改用手指在觸控板上的實際移動距離（Codex 為觸控板寬度的 8%） — 要在原生層讀觸控點，WKWebView 能否收到 touch 事件未驗證；120px 捲動量目前沒有手感問題
**Capture to**: spec | design | tasks
**Next**: /speclink-propose --from-discussion swipe-back-delay
