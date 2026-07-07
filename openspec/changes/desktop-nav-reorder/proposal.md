## Summary

桌面 app 側欄導覽重排:移除備忘佔位鈕,已封存入口自頂欄移入側欄(規格下方、帶計數徽章),設定上移至同群組,側欄順序:變更>規格>已封存>設定。

## Motivation

備忘是無視圖、無資料的佔位鈕,留著持續誤導使用者;已封存入口藏在頂欄,與側欄導覽語意不一致;設定被 spacer 推到側欄底部、與其餘導覽項分離。目標使用者為以桌面 app 瀏覽 SDD 工作區的開發者,情境為專案內的頁面切換。2026-07-07 討論「desktop-導覽與專案首頁重構」定案,該討論扇出兩個變更,本變更為刀一(刀二為設定頁專案設定卡)。

## Proposed Solution

- 移除備忘導覽項與其兩語系 i18n 鍵。
- 已封存改為側欄導覽項:置於規格之下、帶封存數量徽章(導覽項元件擴充選配尾隨元素)、點擊語意為單純切至已封存頁(返回看板改點「變更」),無障礙標籤維持「已封存」;頂欄的已封存切換鈕移除。
- 設定導覽項自底部上移至已封存之下(移除 spacer 定位)。
- 於 desktop-app spec 新增一條側欄導覽結構需求,釘住順序、徽章與切頁行為,作為真實視窗驗證錨點。

## Non-Goals

- 不動規格佔位鈕的無作用現況(規格頁本體屬未來變更)。
- 不保留已封存的 toggle 語意(再點一次跳回看板)——與其他導覽項行為統一。
- 不採「設定固定於側欄底部」的桌面慣例(使用者明確指定順序)。
- 引擎 crates(speclink-core、speclink-cli)零改動,CLI 輸出與回歸對照不受影響。

## Alternatives Considered

- 已封存留在頂欄僅重排側欄:入口分裂於兩處,導覽心智模型不一致,捨棄。
- 拿掉已封存計數徽章以免擴充導覽項元件:資訊降級,擴充成本僅一個選配屬性,捨棄。

## Impact

- Affected specs: desktop-app(ADDED 側欄導覽結構需求)
- Affected code:
  - Modified: apps/desktop/src/App.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/__tests__/App.test.tsx
  - New: (none)
  - Removed: (none)
