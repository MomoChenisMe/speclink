## Context

`desktop-settings-mode` 後專案說明與產出規則各自成頁：`ContextPage` 與 `RulesPage` 的編輯態仍用 `Textarea`，編輯／取消／儲存控制在 `SettingsPageHeader` 右側，寫回走 `useProjectSettings` 的既有 `saveContext`／`saveRules`（core 以 `write_workflow_content` 寫 config.yaml，逐鍵代換、空即移除）。Streamdown 的 `Markdown` 元件可供預覽。CodeMirror 6 尚未是相依。

## Goals / Non-Goals

**Goals**

- 一個共用編輯器元件：markdown 與純文字兩模式、行號、自動換行、token 配色、⌘S。
- 專案說明：編輯／預覽切換、保真寫回。
- 產出規則：每鍵一個編輯區、一行一條，既有寫入語意不變。

**Non-Goals**

- 所見即所得、工具列、產出流程與 .speclink.yaml 頁。

## Decisions

### D1 相依與元件

`apps/desktop/package.json` 加 `@codemirror/state`、`@codemirror/view`、`@codemirror/language`、`@codemirror/commands`、`@codemirror/lang-markdown`、`@lezer/highlight`。`apps/desktop/src/components/CodeEditor.tsx`：

- props `{ value, onChange, mode: "markdown" | "plain", readOnly?, onSave?, minLines?, ariaLabel, autoFocus? }`（`readOnly` 只設 `EditorState.readOnly`：輸入被忽略、標上 `aria-readonly`，內容區仍可聚焦——不關 `EditorView.editable`，免得 WebView 把焦點移走、儲存失敗後回不來）；以 `EditorView` 掛在 `div` 上，`value` 外部變動時以 transaction 整份替換（游標盡量保留；在 layout effect 同步，不留讓新按鍵插進來再被舊 value 蓋掉的空檔）；`onChange` 於文件變動時回傳 `state.doc.toString()`。
- 擴充：`lineNumbers()`、`highlightActiveLine()`／`highlightActiveLineGutter()`（游標行上色）、`EditorView.lineWrapping`、`history()`、`keymap.of([{ key: "Mod-s", run }, ...defaultKeymap, ...historyKeymap, indentWithTab])`（`run` 在沒有 `onSave` 或編輯器唯讀時回 false、不攔事件，交給頁層）、markdown 模式加 `markdownLanguage` 與 `syntaxHighlighting(HighlightStyle)`（不用 `markdown()`：它預設嵌入 HTML／CSS／JavaScript 語言與自動完成，主程式包多約 190KB；代價是 Enter 不續寫清單符號、行內 HTML 不上色）（標題粗體＋`text-foreground`、行內 code 等寬＋`bg-muted`、連結 `text-primary`、強調粗體）；`EditorView.theme` 只用 CSS 變數（`var(--foreground)`、`var(--border)`、`var(--muted)`、`var(--muted-foreground)`、`var(--primary)`（游標行以 `color-mix` 調淡）、`var(--destructive)`（特殊字元）；外殼底色 `bg-card`），不寫色階；焦點環沿 `--ring`。
- 底部狀態列（元件外殼渲染）：`markdown · 自動換行` 或 `純文字 · 自動換行`，右端「第 N 行，第 M 欄」隨游標更新。
- 整體包在 `rounded-xl border border-border bg-card`；最小高度 `minLines` 行。
- 測試環境：jsdom 無 Range 的版面量測，掛 CodeMirror 需補 `Range.prototype.getClientRects` 與 `getBoundingClientRect`，與其他 jsdom 缺口一起放 `apps/desktop/vitest.setup.ts`；讀寫編輯器與按 Mod-s 的輔助放 `apps/desktop/src/__tests__/helpers/codemirror.ts`。

### D2 專案說明頁

- 唯讀：既有 markdown 渲染，**拿掉收合**——專案說明自 `desktop-settings-mode` 起自成一頁，整頁只有這段內容，收合只會把它藏起來（收合是舊版「多張卡同頁」的遺留）；連帶移除 `isLongContext`、展開狀態與只剩此處使用的 `settings.showMore` 文案。
- 點「編輯」：頁標題右側變為分段切換「編輯｜預覽」（`packages/ui` 的 `TabsList` 新增 `segmented` 變體：h-7 膠囊底、作用中分頁卡片底加陰影；呼叫端只指定變體，不以 className 覆蓋）＋ghost「取消」＋主要「儲存」（右側 `Kbd variant="inherit"` 跟隨按鈕字色，標籤以共用的 `modKey(platform)` 組出 ⌘S／Ctrl+S）；內容卡換成 `CodeEditor mode="markdown"`（autoFocus）；切「預覽」時卡內改以 `Markdown` 渲染草稿（空草稿顯示空態文案），切回「編輯」保留草稿與游標位置（編輯器維持掛載、以 `hidden` 切換）。
- 儲存：草稿原字串寫入 context——不去頭尾空白、不改檔尾換行、不改縮排；core 既有序列化以 YAML 區塊字串保留換行（檔尾有換行用 `|`、無則 `|-`）。清空（全空白）仍視為移除鍵（既有語意）。
- ⌘S 在編輯器內與頁內皆觸發儲存（編輯器 keymap 先攔；頁層 `useSaveShortcut` 補焦點在編輯器外或編輯器唯讀的情況）。是否真的送出由 `useProjectSettings` 的寫入守門決定：停用（解析失敗、檢視者、衝突對照中——此時 adapter 已採用最新 revision，再送會繞過對照卡）或同一份寫入進行中都不送（衝突狀態另有同步的 ref 鏡像，409 設下衝突到下一次 render 之間也擋得住）；寫入中旗標在四頁共用狀態裡，儲存中切頁再切回也不重送。儲存進行中按鈕停用、編輯器唯讀（避免存檔後才打的字在編輯器關閉時遺失）。
- 取消：捨棄草稿、回唯讀。

### D3 產出規則頁

- 編輯：每個固定鍵一張小卡：卡頭為本地組合（文件圖示＋等寬鍵名＋「N 條」膠囊，N 為非空行數即時計算；`SettingsGroupHeader` 的鍵名不能設等寬、計數只顯示數字，故不沿用）＋`CodeEditor mode="plain" minLines={3}`（其 `rounded-xl` 外殼即小卡）；頁標題右側 ghost「取消」＋主要「儲存 ⌘S」；無「預覽」。
- 儲存：逐行去頭尾空白、空行略過、依行序整份寫回 rules（既有語意與引號處理不變）；某鍵全空即移除該鍵、全部鍵空即移除 rules 鍵。
- 唯讀：既有僅列有條目的鍵。

### D4 文案

i18n 新增 `settings.editor.edit`、`settings.editor.preview`、`settings.editor.markdownMode`、`settings.editor.plainMode`、`settings.editor.wrap`、`settings.editor.cursor`（「第 {line} 行，第 {col} 欄」）；「N 條」沿用既有 `settings.rulesCount`；兩語系鍵集合相等。

## Risks / Trade-offs

- **CodeMirror 在 jsdom**：需 polyfill 才能掛載；測試聚焦在元件外殼行為（value 回傳、onSave、模式標籤、狀態列）與頁層整合（草稿、預覽、儲存參數），不測算繪。
- **套件體積**：實測主程式包增加約 320KB（minified；gzip 約 110KB），只進桌面 bundle。
- **保真與空白**：既有「清空即移除」只看全空白；非空草稿一律原樣寫，使用者多打的尾端空行會被保留——與「保真」一致。

## Migration Plan

1. D1 元件與測試 helper。
2. D2、D3 接線，搬既有編輯測試並補預覽與保真案例。

## Open Questions

- 無。
