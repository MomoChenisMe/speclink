## Context

desktop 的觸控板滑動返回由前端元件 SwipeBack 實作（封存變更 desktop-readonly-detail-pages 的 design D8）：監聽 window 的 wheel 事件，水平為主且朝返回方向時累計位移，左緣「←」指示隨之推出，120px 達門檻轉實心；因 DOM wheel 事件沒有手指放開的相位，以「最後一個事件後靜止 120ms」判定手勢結束。macOS 慣性捲動在手指離開後仍約每 16ms 送一個事件，計時器一路被重設，使用者放開後要等「慣性尾段＋120ms」才返回。

討論 swipe-back-delay 實查 Codex app：它的滑動返回由原生層的 Chromium HistorySwiper 處理——手指離開觸控板（touchesEnded）時進度到門檻就導覽；phase 與 momentumPhase 皆為 None 的捲動（一般滑鼠滾輪）不處理；慣性事件不會開始一次滑動；尊重系統設定「在頁面之間滑動」；垂直移動壓過水平時整次取消；只在 macOS 提供。

現況可用的基礎：Cargo.lock 已經由 tauri 帶入 objc2 0.6.4、objc2-app-kit 0.3.2、block2 0.6.2，objc2-app-kit 0.3.2 提供 NSEvent 的 addLocalMonitorForEventsMatchingMask_handler、phase、momentumPhase 與 isSwipeTrackingFromScrollEventsEnabled。前端已有 detectPlatform() 判定平台（App 用它切換快捷鍵標籤）。

## Goals / Non-Goals

**Goals:**

- macOS 上手指離開觸控板的當下結算滑動返回，延遲不再受慣性捲動長度影響。
- 輸入來源與不觸發條件對齊 Codex：只認帶手勢階段的輸入、慣性不推進也不開始滑動、尊重系統設定、垂直為主整次取消。
- Windows 與 Linux 不提供觸控板滑動返回。
- 指示外觀、120px 門檻、對話框開著不返回、可水平捲動區塊先捲等既有規則與測試語意保留。

**Non-Goals:**

- 不由原生層繪製指示（Codex 的 OwlHistorySwipeOverlayView 作法）：指示與規則都要在 Rust 重寫，Codex 能這樣做是因為它的分支 Electron 本來就提供。
- 不把門檻改成手指在觸控板上的實際移動距離（Codex 為觸控板寬度的 8%）：要在原生層讀觸控點，WKWebView 能否收到 touch 事件未驗證；延後處理。
- 不改用 WKWebView 原生前後頁滑動（D8 已排除：要寫瀏覽歷史、呈現是整頁滑動）。
- 不新增前進方向的滑動、不改返回鈕／Esc／⌘[／Alt+←／滑鼠側鍵的行為。
- 不動 server、CLI、speclink-core 與任何 `--json` 輸出。

## Decisions

### 原生層以 NSEvent 本地監聽提供手指按下與放開訊號

macOS 的捲動 NSEvent 帶有 phase（MayBegin、Began、Changed、Ended、Cancelled）與 momentumPhase。Tauri 殼在 setup 時以 `NSEvent::addLocalMonitorForEventsMatchingMask_handler`（mask 為 ScrollWheel）掛一個本地監聽：phase 含 MayBegin 或 Began 時送「began」，含 Ended 或 Cancelled 時送「ended」，其餘（Changed、慣性事件、一般滑鼠滾輪的 None）不送。監聽只讀不攔，handler 原樣回傳事件，WKWebView 與頁面捲動不受影響。

替代方案與否決理由：
- 調小 120ms：慣性事件約每 16ms 一個，門檻多小都會被重設。
- 一過門檻就返回（純前端）：失去放開前滑回的反悔能力。
- 前端以事件遞減特徵猜慣性：手指放慢與慣性遞減形狀幾乎一樣，會偶發誤判。

### 階段判讀與事件格式放 speclink-desktop-core，殼只做 AppKit 黏合

依桌面分層規則，可測的邏輯放 speclink-desktop-core（不依賴 Tauri、可獨立 cargo test）。新模組 trackpad_gesture 提供：

- 事件名常數 `TRACKPAD_GESTURE_EVENT = "trackpad-gesture"`。
- `TrackpadGesturePhase`（`Began`／`Ended`，serde 序列化為 `"began"`／`"ended"`）。
- `TrackpadGestureEvent { phase, swipe_enabled }`，以 `#[serde(rename_all = "camelCase")]` 輸出 `swipeEnabled`。
- 純函式 `classify_scroll_phase(phase_bits: u64, momentum_bits: u64) -> Option<TrackpadGesturePhase>`：momentum_bits 非 0 回 None；phase_bits 含 MayBegin（0x20）或 Began（0x1）回 Began；含 Ended（0x8）或 Cancelled（0x10）回 Ended；其餘回 None。位元值與 AppKit 的 NSEventPhase 相同，殼端以 `.bits()` 傳入。

Tauri 殼的 trackpad_gesture 模組（`#[cfg(target_os = "macos")]`）只負責：掛監聽、把 phase／momentumPhase 的位元交給 classify_scroll_phase、在 Began 時讀 `NSEvent::isSwipeTrackingFromScrollEventsEnabled()`（每次讀，使用者執行中改設定也會生效；Ended 時固定帶 false，前端不讀）、以 `app.emit(TRACKPAD_GESTURE_EVENT, event)` 廣播。系統匣面板不訂閱此事件，廣播無副作用。

本變更與「storage 解耦的規格驅動引擎」無關：純屬桌面殼的輸入處理，不碰規格儲存或流程抽象。

### 監聽的生命週期與相依

- 監聽在 `run()` 的 setup 中、僅 macOS 編譯的區塊安裝；回傳的 monitor 物件放進 app 管理的狀態持有到 app 結束，不提供移除（app 生命週期內恆在）。安裝回傳 None 時只記錄，app 照常，失去觸控板滑動返回。
- apps/desktop/src-tauri 的 Cargo.toml 在既有 `[target.'cfg(target_os = "macos")'.dependencies]` 區塊加 objc2 0.6、block2 0.6、objc2-app-kit 0.3（features：NSEvent、block2），版本對齊 Cargo.lock 中經 tauri 帶入的 0.6.4／0.6.2／0.3.2，不得引入第二個版本（重複連結 objc 類別符號的同一類風險，見同區塊 window-vibrancy 的註記）。Windows 與 Linux 建置不攜入。

### 前端 SwipeBack 以手勢階段界定累計區間、放開即結算

SwipeBack 在 enabled 時以 `@tauri-apps/api/event` 的 listen 訂閱 `trackpad-gesture`（訂閱失敗，例如非 Tauri 環境，靜默：沒有階段事件就不會觸發），並保留 window wheel 監聽。狀態：

- **idle**（手指未按下）：wheel 事件一律忽略——一般滑鼠滾輪、Shift＋滾輪與手指放開後的慣性捲動都落在這裡。
- 收到 began：若 swipeEnabled 為 false 維持 idle；否則進入 **tracking**，歸零累計。已在 tracking 時再收到 began（MayBegin 之後的 Began）只更新 swipeEnabled，不歸零。
- **tracking** 中的 wheel：ctrlKey（縮放）忽略；累計 `sumX += deltaX`、`sumAbsY += |deltaY|` 供垂直判定；返回位移沿用現行規則（起點只認朝返回方向、且不在仍可沿 deltaX 捲動的區塊內；開始後反向扣減、下限 0）；指示進度＝返回位移／120。每個事件後檢查垂直取消：`sumAbsY > 10 && sumAbsY > |sumX|` 時進入 **cancelled**，進度歸零。
- **cancelled**：wheel 忽略，直到 ended。
- 收到 ended：tracking 且返回位移 ≥ 120 時立即呼叫 onBack；不論結果歸零進度並回到 idle。

移除 IDLE_MS 與 setTimeout。`SWIPE_THRESHOLD = 120`、`backDelta`（依 webkitDirectionInvertedFromDevice 換算手指方向）與 `canScrollX` 保留。

替代方案：以 props 注入訂閱函式（session.ts 的 deps 注入式）。否決：SwipeBack 只有一個呼叫端，測試以 vi.mock 模組即可，不為一次性用途加抽象。

### 平台限定在 App 層

App 以既有 `detectPlatform()` 的結果把 SwipeBack 的 enabled 改為「平台為 macOS 且有返回去處」；Windows 與 Linux 的 SwipeBack 不訂閱、不監聽，指示保持隱藏。根層 `overscroll-behavior-x: none` 保留（各平台無害）。

## Implementation Contract

**行為（使用者可觀察）：**

- macOS：詳情頁、設定模式、專案設定模式下，觸控板或 Magic Mouse 兩指向右滑，指示隨距離推出、120px 轉實心；手指離開的當下，已達門檻立即返回、未達收回；放開前滑回門檻以下則收回不返回；放開後的慣性不推進指示、不開始新滑動、不造成第二次返回。
- macOS：一般滑鼠水平滾輪與 Shift＋滾輪不出現指示、不返回；系統設定「在頁面之間滑動」不含兩指時不出現指示、不返回；同一次手勢先垂直捲動超過 10px 且垂直多於水平時整次取消。
- Windows／Linux：觸控板水平滑動不出現指示、不返回；返回鈕、Esc、Alt+←、滑鼠側鍵照舊。
- 對話框開著時不返回、可水平捲動區塊先捲、只裁切的區塊不擋——與現行相同。

**介面／資料形狀：**

- Tauri 事件名 `trackpad-gesture`，payload JSON：`{ "phase": "began" | "ended", "swipeEnabled": boolean }`（ended 時 swipeEnabled 恆為 false，前端不讀）。
- speclink-desktop-core 匯出 `trackpad_gesture` 模組：`TRACKPAD_GESTURE_EVENT`、`TrackpadGesturePhase`、`TrackpadGestureEvent`、`classify_scroll_phase`。
- SwipeBack 的 props 維持 `{ enabled, onBack }`；`SWIPE_THRESHOLD` 仍匯出。

**失敗模式：**

- 監聽安裝失敗：記錄一行錯誤，app 照常，macOS 失去觸控板滑動返回（其他返回方式不受影響）。
- 前端訂閱失敗：靜默，同上。
- began 事件晚於最初幾個 wheel 事件抵達：那幾個事件不計入（見 Risks）。

**驗收：**

- `cargo test -p speclink-desktop-core trackpad_gesture`：classify_scroll_phase 對 MayBegin、Began、Changed、Ended、Cancelled、慣性（phase None＋momentum Changed）、一般滑鼠（兩者皆 None）的回傳，以及 payload 序列化為 `{"phase":"began","swipeEnabled":true}`。
- `cargo build -p speclink-desktop`（macOS）通過；Windows CI 建置不含此模組與 objc 相依。
- `npm test -w apps/desktop`：swipeBack.test.tsx 涵蓋放開即結算（無計時器）、未達門檻、放開前滑回、無 began 的 wheel 忽略、swipeEnabled false 忽略、ended 後慣性忽略且不二次返回、垂直取消、往前方向忽略、水平捲動區塊兩種自然捲動設定、只裁切區塊、停用時不訂閱不監聽；App.test.tsx 的觸控板測試改以 began／wheel／ended 驅動並固定平台為 macOS，另加 Windows 平台滑動不作用。
- 手動：使用者在 Mac 觸控板實測快甩即返回、Shift＋滾輪不觸發、系統設定改三指不觸發、先垂直再水平不觸發、設定模式滑動返回。

**範圍：**

- In scope：apps/desktop 的 Rust 殼（新監聽模組、setup 安裝、macOS 相依）、speclink-desktop-core 的 trackpad_gesture 模組、前端 SwipeBack 與 App 的平台限定、對應測試、desktop-app 兩條需求的規格。
- Out of scope：原生繪製指示、門檻單位改為手指移動距離、前進手勢、其他返回方式、server／CLI／core 引擎。

## Risks / Trade-offs

- [began 的 IPC 事件與 DOM wheel 事件分屬兩條管道，抵達順序不保證] → MayBegin 在手指觸碰、尚未移動時就送出，通常早於第一個位移事件；即使晚到，只少計最初幾 px，門檻 120px 不受實質影響。ended 晚到同理只多計幾 px。
- [ended 遺失（例如手勢中視窗失焦而系統沒送 Cancelled）使指示停在半途] → 下一次 began 會歸零；enabled 轉為 false（詳情頁關閉、離開設定模式或專案設定模式）時，SwipeBack 解除訂閱與監聽並把進度歸零。
- [垂直取消的 10px 死區與比例是自訂數值，與 Chromium 的正規化座標不同] → 以單元測試釘住「先垂直 40px 再水平 200px 取消」「純水平 200px 帶少量垂直晃動不取消」兩例；實測手感不對時只調常數。
- [Windows／Linux 使用者失去觸控板滑動返回（刻意的行為變更）] → proposal 標 BREAKING；返回鈕、Alt+←、滑鼠側鍵照舊立即返回。
- [objc 相依版本漂移造成重複連結] → 版本對齊 Cargo.lock 既有版本，加入後檢查 Cargo.lock 沒有出現第二個 objc2／objc2-app-kit／block2 條目。
- 回歸對照：不涉及 golden 與 CLI 測試；前端既有觸控板測試改寫為階段驅動，語意（門檻、反向扣減、捲動區塊、對話框）逐條保留。
- 跨平台：監聽與 objc 相依只在 macOS 編譯；Windows CI 會驗證非 macOS 建置與前端的平台限定測試。
