## 1. 前端測試先行

- [x] 1.1 紅:於 apps/desktop/src/__tests__/App.test.tsx 撰寫失敗測試,覆蓋需求「側欄導覽結構」——側欄依序呈現變更/規格/已封存/設定四項且無備忘項、頂欄無已封存鈕;已封存導覽項帶封存數量徽章且無障礙標籤為「已封存」;於已封存頁再點已封存項仍停留、點變更項返回看板(切頁語意,非 toggle);i18n 兩語系鍵集合相等(備忘鍵已移除)。驗證:npm test -w apps/desktop 出現預期紅燈。

## 2. 導覽重排實作

- [x] 2.1 綠:apps/desktop/src/App.tsx 實作——NavItem 擴充選配尾隨元素以承載徽章;移除備忘 NavItem 與頂欄已封存鈕;新增已封存 NavItem(規格之下、計數徽章、aria-label 已封存、單純切頁);設定 NavItem 上移(移除 spacer 定位);apps/desktop/src/i18n/messages.ts 移除備忘鍵(zh-TW 與 en),1.1 測試全綠。驗證:npm test -w apps/desktop 全綠,npm test -w packages/ui 全綠(確認未波及共用元件)。

## 3. 建置與真實視窗驗證

- [x] 3.1 前端與桌面殼建置成功。驗證:npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop 皆 exit 0(重建前先關閉執行中的 speclink-desktop exe)。
- [x] 3.2 真實視窗驗證(操作前先確認使用者未在使用螢幕;於臨時測試工作區啟動 release exe):側欄順序為變更/規格/已封存/設定且無備忘、頂欄無已封存鈕;點已封存進入已封存頁且徽章數與清單一致;於外部終端封存一個變更後徽章數秒內 +1;再點已封存仍停留、點變更返回看板。驗證:CopyFromScreen 截圖逐項核對相符。
