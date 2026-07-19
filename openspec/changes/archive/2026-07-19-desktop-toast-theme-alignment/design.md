## Context

`packages/ui` 的 `Toaster` 目前只固定 Sonner 的 system theme、6 秒逾時、關閉鈕與單槽數量。Sonner 仍以套件內建的 `--normal-bg`、`--normal-border`、`--normal-text`、字型與 8px 圓角繪製 toast；桌面 app 則由 `apps/desktop/src/index.css` 的 card、card-foreground、border、destructive、radius 等語意 tokens 驅動淺色與深色介面。結果是 toast 的明暗方向正確，但色彩語言與主介面脫節。

本變更只落在 React 呈現邊界 `packages/ui`。speclink-core、speclink-cli、Desktop Rust core、Tauri adapter、儲存、序列化、設定與 git 行為均不受影響；此處沒有需要抽入 storage 解耦引擎的領域邏輯。

## Goals / Non-Goals

**Goals:**

- 讓 toast 的表面、前景、邊框、圓角、陰影與字體使用主介面既有語意 tokens。
- 讓 error toast 在中性 card 表面上以 destructive 語意清楚標示失敗。
- 讓同一套映射自然跟隨既有淺色／深色 token 值，不另維護兩套 toast 色票。
- 保留呼叫者傳入的 `ToasterProps` 與 `toastOptions` 擴充能力，且不改動既有 toast 行為。

**Non-Goals:**

- 不新增、重命名或調整 `apps/desktop/src/index.css` 的全域 tokens。
- 不啟用 Sonner 內建 rich colors，不建立 toast 專用 raw hex／HSL 色票。
- 不改訊息內容、持續時間、單槽取代、關閉、hover 暫停、z-index 或成功靜默語意。
- 不改其他 shadcn 元件、Rust crates、CLI、設定、技能與文件。

## Decisions

### D1: 由 Toaster wrapper 橋接主介面語意 tokens

`packages/ui/src/components/ui/sonner.tsx` SHALL 在 Sonner root 設定其公開 CSS custom properties，使一般 toast 的背景、文字、邊框與圓角分別引用主介面的 card、card-foreground、border 與 radius tokens；字型 SHALL 繼承 host app，陰影 SHALL 使用共用元件既有的 elevation 語意。映射放在共用 wrapper，所有呼叫端自動取得一致外觀，且不需要在 `apps/desktop/src/index.css` 以 Sonner 私有 DOM selector 做全域覆寫。

替代方案是在 desktop CSS 直接選取 `data-sonner-*` attributes。此法把第三方 DOM 細節散落到 host app、共用元件本身仍未封裝完整，因此不採用。另一替代是只傳 `theme="system"`，只能切換明暗而無法接上 Speclink tokens，也不採用。

### D2: Error toast 使用中性 card 表面與 destructive 局部強調

error toast SHALL 保留 card surface 與 card foreground，並以 destructive token 強調錯誤 icon 與邊框；關閉鈕 SHALL 使用同一表面、邊框與可讀前景。這讓錯誤可辨識，但不把長篇 core 錯誤包在高飽和紅底中，延續目前介面以中性表面承載內容、語意色局部標示狀態的做法。

替代方案是開啟 Sonner `richColors`。它會套用 Sonner 自有 light/dark HSL error 色票而非 Speclink OKLCH destructive token，正是本變更要移除的第二套色彩系統，因此不採用。另一替代是整張 destructive 實色背景；長錯誤文字的對比與視覺重量過高，也不採用。

### D3: Token defaults 與呼叫端樣式採可預測合併

wrapper SHALL 先提供設計系統 defaults，再保留呼叫者傳入的 root `style`、`toastOptions.style` 與 `toastOptions.classNames`；巢狀 classNames SHALL 逐欄合併，不得因新增 defaults 而整包覆蓋呼叫端設定。既有固定的 i18n 關閉鈕標籤、system theme、6 秒逾時、關閉鈕與單槽語意維持不變。

替代方案是完全鎖死樣式、不接受呼叫端擴充。`Toaster` 已公開 `ToasterProps`，破壞既有擴充面沒有必要，因此不採用。也不新增 theme adapter 或專用 hook；兩個檔案內的薄 wrapper 與測試足以完成需求。

## Implementation Contract

**Behavior:** 桌面 app 呈現失敗 toast 時，toast 與 card 使用同一表面、前景、邊框、圓角、陰影與 host 字型；錯誤 icon／邊框使用 destructive 語意。作業系統切換淺色或深色時，toast 由同名語意 tokens 自動取得對應值。訊息、位置、層級、6 秒逾時、關閉鈕、hover 暫停與單槽取代皆與變更前相同。

**Interface:** `Toaster(props: ToasterProps)` 與 `toast.error(...)` 呼叫形狀不變，`packages/ui` 的公開匯出不變。實作只可消費既有 CSS variables 與 Sonner props，不新增依賴、設定、IPC、JSON 或檔案格式。

**Failure modes:** token 映射不得依賴 Sonner 的 light/dark raw 色值；呼叫者附加 root style 或 toast classNames 時不得遺失。長錯誤、空 core 錯誤與 HTML-like 文字仍以既有文字路徑呈現，不因樣式調整改寫或注入 HTML。

**Acceptance criteria:**

- `packages/ui/src/__tests__/sonner.test.tsx` 先以失敗測試斷言一般表面與 error 語意的 token 映射、host 字型繼承，以及呼叫端 style/classNames 保留，再由 wrapper 實作轉綠。
- `npm test -w packages/ui` 全綠，既有逾時、關閉與單槽測試不改語意。
- `npm test -w apps/desktop` 與 `npm run build -w apps/desktop` 通過，證明整合與 bundle 無回歸。
- release app 中觸發封存失敗，眼見 toast 在抽屜遮罩上方使用主介面 card surface 與 destructive 強調；淺色與深色由同一組 token 映射成立。

**Scope boundaries:** in scope 僅 `Toaster` 視覺 token bridge 及其元件測試；out of scope 為 store 訊息、其他回饋面、全域 token 定義、Sonner 版本、所有 Rust／CLI／storage 路徑。

## Risks / Trade-offs

- [Sonner 內建 stylesheet specificity 蓋過 utility class] → 一般表面與圓角優先透過 Sonner 公開 CSS custom properties橋接；針對 error 局部 class 以 DOM 測試與 release app 實視雙重確認。
- [深色模式 destructive 強調對比不足或過重] → 僅使用既有 dark destructive、card 與border tokens，不創造 raw 色；實視同時確認文字可讀與狀態可辨。
- [巢狀 `toastOptions` 合併時覆蓋呼叫端設定] → 加入帶自訂 root style、toast class 與 error class 的測試，明確釘住合併順序。
- [跨平台 host 字型不同] → 使用 inherit 而非指定 macOS 字型，讓 Windows、macOS、Linux 各自沿用 desktop app 已配置的字型堆疊。
- [UI 全套測試既有 timer teardown 偶發干擾] → 先跑 `sonner.test.tsx` 精準驗證，再單獨跑完整 UI suite；不把無關的 ArchivedList timer 修復塞入本 change。

## Migration Plan

不需要資料或設定遷移。發布時隨 Desktop 前端 bundle 生效；回滾只需還原 wrapper 與測試，不影響使用者資料或規格文件。

## Open Questions

無。
