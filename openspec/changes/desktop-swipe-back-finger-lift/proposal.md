## Why

在 desktop 詳情頁、設定模式或專案設定模式用觸控板兩指向右滑動返回時，「←」指示轉實心、手指放開後還要再等一小段時間才回到上一頁；快速一甩時等得最久。目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，他們在看板與詳情頁之間反覆切換檢視變更、討論與規格，這段延遲每次返回都會出現。

原因是網頁層的 wheel 事件不帶「手指放開」的訊號：現行 SwipeBack 以「最後一個事件後靜止 120ms」判定手勢結束，而 macOS 的慣性捲動在手指離開後仍約每 16ms 送一個事件，計時器被一路重設，延遲＝慣性尾段＋120ms。返回動作本身（關閉詳情頁）是同步的，沒有延遲。

討論 swipe-back-delay 實查 Codex app 的作法：它的滑動返回由原生層（Chromium 的 HistorySwiper）處理，直接拿手指放開的時機結算，而且只在 macOS 提供。本變更比照這個作法。

## What Changes

- apps/desktop 的 Rust 殼新增一個只在 macOS 編譯的模組：以 NSEvent 本地監聽讀捲動事件的手勢階段，在手指按下與手指放開時各送一個事件給前端，並附上系統設定「在頁面之間滑動」是否允許兩指滑動。
- 前端 SwipeBack 改為只在「手指按下到放開」之間累計滑動距離，收到放開事件的當下結算：已達門檻立即返回、未達收回指示；移除 120ms 靜止計時器；手指離開後的慣性捲動一律忽略。
- 規則對齊 Codex：沒有手勢階段的輸入（一般滑鼠水平滾輪、Shift＋滾輪）不觸發；Magic Mouse 有手勢階段，照樣支援；系統設定不允許兩指滑動時不觸發；一次手勢中垂直移動明顯多於水平移動時整次取消，直到手指放開。
- 指示外觀、120px 門檻、對話框開著時不返回、可水平捲動的區塊先捲等既有規則不變，指示仍由前端繪製。
- **BREAKING**（行為）：Windows 與 Linux 不再提供觸控板滑動返回（不顯示指示、不關頁）。這兩個平台的返回改由返回鈕、Esc、Alt+← 與滑鼠側鍵「上一頁」承擔，這些方式各平台照舊立即返回。

相容性影響：不涉及 CLI 指令、`--json` 輸出、設定欄位或生成的技能；只影響 desktop app 的互動行為。Windows／Linux 使用者若習慣用觸控板滑動返回，改用 Alt+← 或滑鼠側鍵。

## Non-Goals (optional)

（本變更有 design.md，範圍排除與否決的做法記在 design 的 Goals / Non-Goals。）

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 需求「詳情頁的導覽與返回」的觸控板滑動返回改為只在 macOS、以手指離開觸控板為結算時機，並加入不觸發條件（無手勢階段的輸入、系統設定、垂直為主取消、慣性不推進）；需求「側欄導覽結構」中設定模式與專案設定模式的觸控板返回同步改為只在 macOS。

## Impact

- Affected specs: desktop-app（MODIFIED：詳情頁的導覽與返回、側欄導覽結構）
- 影響的 app：apps/desktop（speclink-desktop-core 的階段判讀與事件格式、Tauri 殼的 Rust 端與 React 前端）；不影響 speclink-core、speclink-cli、speclink-host 與 server
- 新增相依：objc2、objc2-app-kit、block2 只加在 macOS target，版本對齊 Cargo.lock 中經 tauri 帶入的版本（不引入新的 crate 版本）
- Affected code:
  - New: apps/desktop/core/src/trackpad_gesture.rs
  - New: apps/desktop/src-tauri/src/trackpad_gesture.rs
  - Modified: apps/desktop/core/src/lib.rs
  - Modified: apps/desktop/src-tauri/src/lib.rs
  - Modified: apps/desktop/src-tauri/Cargo.toml
  - Modified: Cargo.lock
  - Modified: apps/desktop/src/components/SwipeBack.tsx
  - Modified: apps/desktop/src/App.tsx
  - Modified: apps/desktop/src/__tests__/swipeBack.test.tsx
  - Modified: apps/desktop/src/__tests__/App.test.tsx
