## 1. 前端測試先行

- [x] 1.1 紅:於 apps/desktop/src/__tests__/settingsView.test.tsx 撰寫失敗測試,覆蓋 MODIFIED 需求「設定頁編輯專案說明與產出規則」與 design D1 專案設定卡唯讀優先與就地編輯——開啟設定頁時卡片唯讀(專案說明 markdown 渲染、超長收合顯示更多、未設定顯示空狀態;產出規則僅列有條目鍵)、點編輯就地切換為取消/儲存、取消還原且不觸發寫入、parseError 時浮出說明並停用編輯;並覆蓋 design D2 產出規則整份文字編輯與行序語意——編輯態文字區恰為活躍 schema 固定鍵各一、行↔條目轉換(儲存 payload 逐行 trim、空行滌除、行序即順序、清空文字區=移除鍵、全部空=移除 rules 鍵、未動分頁原樣保留)。驗證:npm test -w apps/desktop 出現預期紅燈。

## 2. 卡片實作

- [x] 2.1 綠:apps/desktop/src/views/SettingsView.tsx 實作專案設定卡——卡層級編輯態、行↔條目轉換內部純函式、沿用既有讀寫 payload 與錯誤浮出;依 design D3 專案說明 markdown 渲染與收合接上 packages/ui 的 Markdown 元件與顯示更多;依 design D4 卡片文案與 i18n 於 apps/desktop/src/i18n/messages.ts 補齊新字串(zh-TW 與 en 鍵集合相等),1.1 測試全綠。驗證:npm test -w apps/desktop 全綠。
- [x] 2.2 移除逐項輸入框編輯器的殘留碼與其專用 i18n 鍵(上下移、逐項刪除、新增規則等),確認無未使用字串與元件。驗證:npm test -w apps/desktop 全綠,建置無未使用符號警告,grep 兩語系字典無殘留鍵。

## 3. 建置與真實視窗驗證

- [x] 3.1 前端與桌面殼建置成功。驗證:npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop 皆 exit 0(重建前先關閉執行中的 speclink-desktop exe)。
- [x] 3.2 真實視窗驗證(操作前先確認使用者未在使用螢幕;於臨時測試工作區啟動 release exe,備妥含 rules、context 與註解的 config.yaml):唯讀呈現 markdown 與有條目鍵;進編輯將某鍵文字區兩行對調後儲存,開檔核對行序對調且其餘鍵保留;新增以 at 符號開頭的一行後檔案仍可被引擎解析;清空專案說明與某鍵文字區後對應鍵被移除;修改後取消,開檔核對逐字元未變;手動改壞 config.yaml 後卡片浮出說明且編輯停用。驗證:CopyFromScreen 截圖與檔案內容逐項核對相符。
