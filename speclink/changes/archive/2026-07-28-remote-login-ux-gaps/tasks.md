## 1. server：account summary 增含專案隸屬

- [x] 1.1 撰寫失敗測試（crates/speclink-server/tests/web_account.rs 增節）：依規格「帳號 browser API 保持憑證祕密邊界」——隸屬兩專案（editor 與 viewer）的成員讀 account summary 得 memberships 兩項（各含 projectKey、projectName、role，camelCase）、無隸屬者得空陣列、admin 得到的是自己的隸屬而非全部專案、payload 仍不含任何 secret。驗證：cargo test -p speclink-server --test web_account 新增案例失敗 <!-- speclink-task:tsk_01KYGXEWPJ86WYQ0PT8V7PMQRK -->
- [x] 1.2 實作：crates/speclink-server/src/web.rs 的 account summary handler 依 design「決策一：memberships 進 account summary，UI 共用一個元件」以既有隸屬查詢組出 memberships 欄位，並依 design「決策五：向後相容與序列化」確認既有欄位序列化不動（純新增欄位）。驗證：cargo test -p speclink-server --test web_account 全綠（含既有案例不變） <!-- speclink-task:tsk_01KYGXEWPJA8R6C2VVAJETHD8C -->
- [x] 1.3 重構：確認與管理頁的隸屬組裝無重複實作（共用查詢路徑）。驗證：cargo test -p speclink-server 全綠 <!-- speclink-task:tsk_01KYGXEWPJR847PPTEW6BZ9EJ8 -->

## 2. server-web：我的專案區塊與核准頁收尾

- [x] 2.1 撰寫失敗測試（apps/server-web/src/__tests__/account.test.tsx 增節）：依規格「帳號頁呈現我的專案」——成員看到兩專案顯示名與角色、顯示名缺席以 key 呈現、無隸屬時空狀態文字可見且區塊不隱藏、區塊無編輯操作。驗證：npm test -w apps/server-web 新增案例失敗 <!-- speclink-task:tsk_01KYGXEWPJTXKJEWYHC072MG5Q -->
- [x] 2.2 實作：apps/server-web/src/api/client.ts 的 AccountSummary 型別增 memberships、apps/server-web/src/pages/AccountPage.tsx 新增我的專案區塊、apps/server-web/src/i18n/messages.ts 補中英文案。驗證：account.test.tsx 全綠 <!-- speclink-task:tsk_01KYGXEWPJ5F182G8W5GGJJHK6 -->
- [x] 2.3 撰寫失敗測試（apps/server-web/src/__tests__/activate.test.tsx 增節）：依規格「核准頁 session 保護且明確確認」的結果頁指引——核准與拒絕兩種結果頁均含可返回 Speclink app 繼續的指引文字。驗證：新增案例失敗 <!-- speclink-task:tsk_01KYGXEWPJY0TVNNKZPG5SF5GD -->
- [x] 2.4 實作：apps/server-web/src/pages/ActivatePage.tsx 的 done 狀態補指引文案（i18n 中英兩版）。驗證：activate.test.tsx 全綠；npm test -w apps/server-web 全綠 <!-- speclink-task:tsk_01KYGXEWPJWFVSK8NAEM8HZV4C -->

## 3. desktop：device login 分段編排（Rust）

- [x] 3.1 撰寫失敗測試（apps/desktop/src-tauri/tests/login_orchestration.rs 增節）：依 design「決策二：desktop device login 由單一阻塞呼叫改為分段編排」——啟動段對有有效 refresh credential 的 origin 直接回已登入（不開瀏覽器）；無 credential 時回等待授權資訊（裝置碼、驗證網址、有效期限、輪詢間隔）且瀏覽器開啟 URL 帶 user_code 預填；單次觀測段對 pending 回 pending、對 approved 完成 credential 存入與身分寫回後回已登入、對 denied／expired 回對應終態且不留 credential。驗證：cargo test -p speclink-desktop 新增案例失敗 <!-- speclink-task:tsk_01KYGXEWPJPXFJ4F6NAS4WCT2A -->
- [x] 3.2 實作：apps/desktop/src-tauri/src/connections.rs 將 device_login 拆為啟動與單次觀測兩段、apps/desktop/src-tauri/src/lib.rs 註冊對應 IPC command（原單段 command 移除，前後端同版出貨）。驗證：cargo test -p speclink-desktop 全綠 <!-- speclink-task:tsk_01KYGXEWPJ00QM9S9KVQKP2V0P -->
- [x] 3.3 重構：確認兩段皆為純請求-回應（無執行緒睡眠、無跨呼叫共享狀態），靜默 refresh 快路徑語意與拆分前一致。驗證：cargo test -p speclink-desktop 全綠 <!-- speclink-task:tsk_01KYGXEWPJQA5BM6MP1K6VEEK1 -->

## 4. desktop：等待授權面與行動呼籲（前端）

- [x] 4.1 撰寫失敗測試（apps/desktop/src/__tests__/serversPanel.test.tsx 與 store 測試增節）：依規格「device login 預設與 PAT fallback」的等待授權面——等待狀態顯示裝置碼、驗證網址、各附複製、倒數與取消；取消後回未登入且輪詢排程停止；依規格「伺服器管理最小面」——登入成功後開啟工作區入口取得鍵盤焦點、工作區選擇器未自動開啟。驗證：npm test -w apps/desktop 新增案例失敗（以 Node 20 執行） <!-- speclink-task:tsk_01KYGXEWPJ7WMDYFGKPQ0PDZN3 -->
- [x] 4.2 實作 store 分段輪詢：apps/desktop/src/store.ts 新增等待授權 phase 變體與輪詢排程（依啟動段回傳間隔、slow_down 加大、以截止時刻判逾時、取消即停），apps/desktop/src/adapter/connections.ts 對接兩段 IPC。驗證：store 測試全綠 <!-- speclink-task:tsk_01KYGXEWPJ6TQ4HC365A753NFX -->
- [x] 4.3 實作 UI：apps/desktop/src/components/ServersPanel.tsx 依 design「決策三：等待授權面是連線互動狀態的新變體」渲染等待授權面（等寬裝置碼、兩個複製鈕沿用既有複製語彙與回饋、倒數、取消），依 design「決策四：登入成功的行動呼籲沿用 focus 管理既有模式」實作成功後開啟工作區聚焦；apps/desktop/src/i18n/messages.ts 補文案。驗證：npm test -w apps/desktop 全綠 <!-- speclink-task:tsk_01KYGXEWPJQDPG8K7MPF0EE2HV -->
- [x] 4.4 真實視窗驗證（jsdom 測不出 pointer 與剪貼簿體感）：以真實視窗走一次 device login——等待授權面可見碼與網址、複製鈕實際寫入剪貼簿、取消即停、核准後開啟工作區鈕取得焦點。驗證：人工檢核上述四點並截圖留檔 <!-- speclink-task:tsk_01KYGXEWPJD0EPFHMHW2R4BTBC -->
- [x] 4.5 撰寫失敗測試（apps/desktop/src/__tests__/workspaceChooser.test.tsx 增節）：依規格「device login 預設與 PAT fallback」的等待授權面——工作區選擇器的新增並登入後，選擇器內就地顯示等待授權面（裝置碼、驗證網址、倒數與取消）；取消即停止觀測且停留在 server 步驟；明確不支援時就地現 PAT 輸入並可完成登入。驗證：npm test -w apps/desktop 新增案例失敗 <!-- speclink-task:tsk_01KYMF01WYWAWFMZWXK9VA598G -->
- [x] 4.6 實作：apps/desktop/src/store.ts 的 addConnection 回傳正規化 origin；ServersPanel 的等待授權面與 PAT 輸入抽成共用元件；apps/desktop/src/components/WorkspaceChooser.tsx 接 connectionPhases 就地渲染等待授權面／PAT 輸入／錯誤（取消接 cancelLogin）；apps/desktop/src/App.tsx 接線新 props。驗證：npm test -w apps/desktop 全綠 <!-- speclink-task:tsk_01KYMF7HWY5QSTH95WVC40K0H9 -->

## 5. 收尾驗證

- [x] 5.1 全套測試：cargo test --workspace 全綠；npm test -w packages/ui、npm test -w apps/server-web、npm test -w apps/desktop（Node 20）全綠。驗證：全部通過 <!-- speclink-task:tsk_01KYGXEWPJ51DYMM7565V356PP -->
- [x] 5.2 執行 speclink validate remote-login-ux-gaps 與 speclink analyze remote-login-ux-gaps。驗證：validate 通過、analyze 無 Critical 或 Warning <!-- speclink-task:tsk_01KYGXEWPJAK1ANBHPD6QC59C2 -->
