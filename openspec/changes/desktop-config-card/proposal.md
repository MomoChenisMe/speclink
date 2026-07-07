## Summary

設定頁的「專案說明」與「產出規則」兩區段重構為頁面頂部的「專案設定」卡:唯讀優先呈現(專案說明 markdown 渲染、超長收合;產出規則僅列有條目的鍵)、右上編輯就地切換(取消/儲存);產出規則自逐項輸入框編輯器改為整份文字編輯——每個 schema artifact 鍵一個多行文字區、一行即一條規則。

## Motivation

現行兩區段是永遠可編輯的表單:專案說明長文以編輯框呈現不利閱讀,產出規則的逐項輸入框加上下移按鈕操作繁瑣。參考 Spectra 專案設定頁的「唯讀優先＋就地編輯」概念,同時保持 Speclink 自身設計語言(teal 主色、既有版面骨架)。目標使用者為以桌面 app 維護專案層工作流設定的開發者,情境為檢視與編修 config.yaml 的專案說明與產出規則。2026-07-07 討論「desktop-導覽與專案首頁重構」定案,本變更為該討論扇出的刀二(刀一為 desktop-nav-reorder)。

## Proposed Solution

- 設定頁頂部新增「專案設定」卡(標註 config.yaml),含「專案說明」「產出規則」兩分頁,預設唯讀。
- 唯讀呈現:專案說明以 markdown 渲染(重用 packages/ui 既有 Markdown 元件)、超過固定高度收合並提供顯示更多;產出規則僅列出有條目的 artifact 鍵,鍵名作小節標題、條目為清單。
- 就地編輯:卡右上編輯鈕切換為取消/儲存;專案說明為 raw markdown 多行文字區;產出規則列出活躍 schema 的全部 artifact 鍵、每鍵一個多行文字區,一行一條規則——調序即搬行、刪除即刪行、新增即打新行。
- 寫入語意全數沿用既有契約:逐行修剪頭尾空白、空行滌除、行序即寫入順序、清空即移除鍵、雙重解析驗證、保留字元自動加引號;config.yaml 解析失敗時卡片編輯停用。底層重用既有 desktop-core 讀寫橋接與 speclink-core 純函式,引擎與橋接層零改動。
- 下方 App 區與專案政策區(locale、spec_locale、tdd、audit 等)維持現行表單形態不動。

## Non-Goals

- 不做獨立專案首頁視圖與統計卡(看板即首頁,通道標題已有計數,討論定案否決)。
- 不保留逐項輸入框與上下移按鈕的編輯器形態(被整份文字編輯取代)。
- 不採單一大文字區編輯全部規則(需手寫鍵名分節、易格式錯,討論定案否決)。
- 不動 App 區與專案政策區的表單形態與行為。
- 引擎 crates(speclink-core、speclink-cli)與 desktop-core 橋接零改動,CLI 輸出與回歸對照不受影響。

## Impact

- Affected specs: desktop-config(MODIFIED 設定頁編輯專案說明與產出規則)
- Affected code:
  - Modified: apps/desktop/src/views/SettingsView.tsx、apps/desktop/src/__tests__/settingsView.test.tsx、apps/desktop/src/i18n/messages.ts
  - New: (none)
  - Removed: (none)
