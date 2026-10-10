## Summary

把桌面 app（apps/desktop）專案設定裡「專案說明」與「產出規則」的多行文字區換成 CodeMirror 6 編輯器：專案說明為 markdown 模式（行號、標題與行內 code 上色、自動換行）並有「編輯／預覽」切換；產出規則每個產出物一個純文字編輯區、一行一條；兩者共用同一個編輯器元件，儲存走 ⌘S，寫回逐位元保真。

## Motivation

專案說明是餵給 AI 的設定文字，多行文字區沒有行號與語法上色，長文編輯容易看錯層級；產出規則一行一條卻只有一塊 textarea。結論選定 CodeMirror 6（否決 TipTap 所見即所得，因為 Markdown 往返契約要自扛、文字需逐位元保真），並要求專案說明有預覽、產出規則與專案說明共用同一個編輯器。使用者在互動原型確認了「專案說明」與「產出規則」兩張畫板。

目標使用者是維護 openspec/config.yaml 專案說明與產出規則的開發者與 PO／PM；對應 config 技能。

本 change 依賴 `desktop-settings-mode`（專案說明頁與產出規則頁、頁標題右側的編輯控制）。

## Proposed Solution

- **編輯器元件**：`CodeEditor`（apps/desktop）包 CodeMirror 6：`markdown` 與 `plain` 兩種模式、行號、自動換行、以設計 token 配色（底、框、游標行、標題、行內 code）、唯讀態、`onSave` 綁 ⌘S／Ctrl+S、底部狀態列（模式 · 自動換行 · 第 N 行，第 M 欄）。
- **專案說明頁**：唯讀時沿用 markdown 渲染、不再收合（自成一頁後收合只會藏起內容）；點頁標題右側「編輯」後內容卡換成編輯器，頁標題右側變成「編輯／預覽」分段切換＋「取消」＋「儲存 ⌘S」；預覽以共用 Markdown 元件渲染草稿；儲存把編輯器內容原樣寫回 context（不去頭尾空白、不補不刪檔尾換行、縮排保留），取消還原。
- **產出規則頁**：編輯時每個固定鍵一張小卡：群組標題（圖示＋等寬鍵名＋「N 條」）＋純文字編輯器；一行一條、行序即注入順序、儲存時逐行去頭尾空白、空行略過、整份寫回 rules（既有語意不變）；頁標題右側「取消」「儲存 ⌘S」。
- **退場**：兩頁不再使用 `Textarea`；`Textarea` 原語留給其他消費者。

## Non-Goals

- 所見即所得編輯、Markdown 工具列、拖放圖片。
- 產出規則單條增刪改的獨立寫回動詞（結論延後）。
- 產出流程頁與 .speclink.yaml 頁的編輯方式。
- server-web。

## Alternatives Considered

- TipTap 所見即所得——結論已否決。
- 產出規則改清單卡逐條編輯——結論已否決：使用者偏好同一個編輯器、一行一條。
- 編輯器放 packages/ui——目前只有桌面消費者，CodeMirror 相依會進 server-web 的套件樹；放 apps/desktop，待第二個消費者出現再上提。

## Impact

- Affected specs: desktop-config（設定頁編輯專案說明與產出規則 MODIFIED）、desktop-app（表單控制項與按鈕以主題化元件呈現 MODIFIED）
- Affected code:
  - New: `apps/desktop/src/components/CodeEditor.tsx`、`apps/desktop/src/__tests__/codeEditor.test.tsx`、`apps/desktop/src/__tests__/helpers/codemirror.ts`
  - Modified: `apps/desktop/package.json`、`package-lock.json`、`apps/desktop/src/views/project-settings/ContextPage.tsx`、`apps/desktop/src/views/project-settings/RulesPage.tsx`、`apps/desktop/src/views/project-settings/useProjectSettings.ts`、`apps/desktop/src/views/project-settings/parts.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`、`apps/desktop/src/__tests__/App.test.tsx`
  - Removed: （無）
