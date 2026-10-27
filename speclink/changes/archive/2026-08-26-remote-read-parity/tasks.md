## 1. Wire 欄位（speclink-protocol）

- [x] 1.1 先寫紅測試：crates/speclink-protocol 單元測試斷言（a）ChangeSummary 的 createdBy/created/fromDiscussions、（b）ChangeStatus 的 createdBy/createdWith/startedAt/startedBy、（c）DiscussionInfo 的 promotedTo——三組欄位各驗序列化含鍵、缺席／空清單省略鍵、反序列化無鍵舊 payload 不失敗且得預設值 <!-- speclink-task:tsk_01M0WN7417DFEWHX0ZKW4TRHYQ -->
- [x] 1.2 實作三個 struct 的選填欄位（camelCase、serde default、Option 以 skip_serializing_if 省略、Vec 以 is_empty 省略，比照 kind 欄位既有慣例），落實 client-protocol 的「變更清單的建立者與來源討論欄位」「單 change 讀取回應的 meta 歸屬欄位」「討論資訊 payload 增選填 promotedTo 欄位」三條 requirement，1.1 測試轉綠 <!-- speclink-task:tsk_01M0WN741749KZ1FAWBKHTC83K -->

## 2. Server 讀取面組裝（speclink-server）

- [x] 2.1 先寫紅測試：server 整合測試斷言（a）GET /changes/{name} 對 meta 含 created_by/created_with/started_at/started_by 的 change 回四鍵、四欄皆缺回無鍵，（b）GET /changes 清單項對有無 meta 欄位的兩個 change 各回三鍵／無鍵，（c）GET /discussions 對已轉出討論回 promotedTo 且順序沿 frontmatter 累加、未轉出討論無鍵，（d）本地 CLI discuss list --json 輸出與改動前逐位元相同 <!-- speclink-task:tsk_01M0WN7417ESTDS4PHC7NMQ0T1 -->
- [x] 2.2 實作：單 change 讀取路由自既有 parsed meta 補四欄位（落實「單 change 讀取回應攜帶 show 組合欄位」的擴充）；清單路由沿 startedAt 的逐筆 meta 組裝路徑補三欄位（metaError 容錯路徑欄位缺席、清單不失敗，落實「變更清單回應攜帶建立者與來源討論欄位」）；討論列表路由以引擎 promoted_to 查詢函式邊緣組裝（單筆查詢失敗欄位缺席、列表不失敗，落實「討論列表回應攜帶 promotedTo」），2.1 測試轉綠 <!-- speclink-task:tsk_01M0WN7417JC0BFGQA9SGECF5E -->

## 3. 桌面 Rust 橋（src-tauri）

- [x] 3.1 先寫紅測試：apps/desktop/src-tauri/tests/it/remote_data.rs 斷言 handshake 後 RemoteCapabilities 的 change_capabilities 與 change_meta 為 true <!-- speclink-task:tsk_01M0WN7417X7QX30XK7GRRNCTW -->
- [x] 3.2 實作：兩個 capability 位翻真；同檔「ChangeStatus/ChangeSummary 皆不帶 metadata 與 capability 名清單」過期註解更正為現況描述，3.1 測試轉綠 <!-- speclink-task:tsk_01M0WN7417DY4BSN7W52E1ZYPS -->

## 4. 桌面 TS adapter 與 UI（apps/desktop/src）

- [x] 4.1 先寫紅測試：remoteDataSource.test.ts 斷言 changeCapabilities 回傳 status payload 的 deltaCapabilities、changeMeta 以 status payload 組出含七欄位的 ChangeMetaInfo（缺席欄位為 null／缺席）、promotedTo 非空時討論項如實攜帶；remoteCapabilities.test.tsx 斷言 remote 詳情抽屜不再呈現「server 尚未提供」的 capability 清單與詮釋資料停用說明、舊 server（無新欄位）下對應列缺席且無錯誤 <!-- speclink-task:tsk_01M0WN7417YRDX25Z4BB5RNZXN -->
- [x] 4.2 實作：changeCapabilities/changeMeta 改以既有 remote_status 路徑映射（不開新 Tauri command、不另發 HTTP 請求）、刪除兩方法的 unsupported 拒絕；toDiscussionItem 改映射 wire promotedTo、刪除空清單補丁與其註解；App.tsx 的「capability 缺口的讀取（無 server 來源）」過期註解更正——落實 remote-workspace-data「capability 驅動停用且不偽造缺口」修訂後的直達與誠實降級語意，4.1 測試轉綠 <!-- speclink-task:tsk_01M0WN7417QTJEGMMP4W7A2EQ0 -->

## 5. 全面驗證與人工驗收

- [x] 5.1 逐 crate 跑受影響測試（speclink-protocol、speclink-server、desktop src-tauri 以 --test it、apps/desktop 前端測試與 packages/ui 既有測試）確認全綠且無既有測試被新欄位破壞；一併確認 remote 刪除守門的既有測試與 remote-resilience「remote 破壞性操作確認一致」更正後的 deleteChange 語意相符（純 spec 過期句更正，無新行為） <!-- speclink-task:tsk_01M0WN7417SWNBBEW15ZSFD709 -->
- [x] [M] 5.2 以本刀建置的 server 與桌面 app 實開一個 remote 分頁，目視確認：詳情抽屜的建立者／開工歸屬列與 capability 清單與本地分頁同形、看板卡片有建立者頭像與來源討論標記、已轉出討論落入「已轉出變更的討論」群組 <!-- speclink-task:tsk_01M0WN7417E4YFSBN6EZ530Q9M -->
