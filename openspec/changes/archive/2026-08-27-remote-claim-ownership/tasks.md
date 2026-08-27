## 1. 引擎認領語意（speclink-core）

- [x] 1.1 先寫紅測試：引擎單元測試斷言「認領標記欄位」requirement——（a）團隊模式 store 首次 Claim 寫 claimed_by／claimed_at、既有 meta 欄位逐字元保留、發布 change-claimed 且 revision 前進，（b）同人重複 Claim 冪等成功且零寫入零事件，（c）他人 Claim 被 ownership 衝突拒絕且 message 含持有人、meta 零改動，（d）fs store 拒絕文案與既有測試一致，（e）壞 meta fail-closed 拒絕且零寫入 <!-- speclink-task:tsk_01M0XTKF6S24E4ZDKNPNW9HYRX -->
- [x] 1.2 實作：change meta model 增 claimed_by／claimed_at 選填欄位（缺席即未認領、序列化不破壞既有欄位順序語意）；Command::Claim 依 store 能力分流——fs 維持既有拒絕、團隊模式執行 D3 的寫章／冪等／衝突語意，1.1 測試轉綠 <!-- speclink-task:tsk_01M0XTKF6S9WGK781H2RDYVRF4 -->

## 2. Server 端點與讀取組裝（speclink-server）

- [x] 2.1 先寫紅測試：server 整合測試斷言「claim 端點持久化與 ownership 衝突語意」requirement——（a）editor 認領成功後 GET /changes 與 GET /changes/{name} 的 claimedBy 皆為認領者且 server 重啟後仍在，（b）他人認領收 409、reason refused、message 含持有人與建議動作，（c）同人重複認領成功且零寫入，（d）reader 收 403、scope 零改動，（e）不存在的 change 回 404 <!-- speclink-task:tsk_01M0XTKF6S3SEKG992FJMW7K2X -->
- [x] 2.2 實作：claim 路由移除回聲 stub、比照 in-progress 端點經 Command gateway 直通引擎；引擎 ownership 衝突以既有 Refused 碼經 error 映射層對應 409 refused（reason registry 零擴充）；清單與單 change 讀取組裝自 meta 填 claimedBy（未認領省略），2.1 測試轉綠 <!-- speclink-task:tsk_01M0XTKF6SBEMMN9GW23N92ZFV -->

## 3. CLI 衝突訊息（speclink-cli）

- [x] 3.1 補整合測試釘住 verb-contract 既有承諾「認領被搶佔」scenario：speclink claim 撞 409 refused 時 exit code 非 0、stderr 含目前持有人與建議動作（此路徑因 server 過去從不回 409 而測不到，本刀首次可驗證） <!-- speclink-task:tsk_01M0XTKF6SM2HN3RFAJ6X0X5E6 -->

## 4. 桌面認領面（src-tauri＋apps/desktop/src＋packages/ui）

- [x] 4.1 先寫紅測試：src-tauri 整合測試斷言 handshake 後 RemoteCapabilities 的 claim 位依 role——editor true、reader false；前端測試斷言「認領操作與認領人呈現」requirement——remoteDataSource 的 claim 方法打 remote command、ChangeItem 攜帶 claimedBy、詳情抽屜對未認領 change 呈現認領操作（reader 停用附繁中說明）、看板卡片與抽屜呈現認領人、409 refused 呈現持有人與建議動作、本地分頁無認領面 <!-- speclink-task:tsk_01M0XTKF6S9WT673QXTPR5TCXN -->
- [x] 4.2 實作 Rust 側：Tauri command 曝露既有 RemoteWorkspace::claim、RemoteCapabilities 增 claim 位依 role 決定，4.1 的 src-tauri 測試轉綠 <!-- speclink-task:tsk_01M0XTKF6SNZ1BMJ8Y6GSJBKP5 -->
- [x] 4.3 實作 TS／UI 側：adapter 介面與 remoteDataSource 增 claim 方法與 ChangeItem.claimedBy 映射；詳情抽屜認領按鈕與認領人列、看板卡片認領人標記（沿建立者頭像呈現慣例）；409 沿既有錯誤呈現路徑顯示持有人；本地 dataSource 不提供認領面，4.1 前端測試轉綠 <!-- speclink-task:tsk_01M0XTKF6S4RJ9VADRM6G406MK -->

## 5. 全面驗證與人工驗收

- [x] 5.1 逐 crate 跑受影響測試（speclink-core、speclink-server、speclink-cli 以 --test it、desktop src-tauri 以 --test it、apps/desktop 前端與 packages/ui）確認全綠，並確認 TeamStore 三 driver 的 conformance suite 不因 meta 新欄位破壞 <!-- speclink-task:tsk_01M0XTKF6SHHG989298ZY2FWT2 -->
- [x] [M] 5.2 以兩個不同帳號實測：帳號一於桌面遠端分頁認領一個 change 後重開 app 認領仍在；帳號二認領同 change 見「已由持有人認領」呈現；帳號二認領另一未認領 change 成功；CLI speclink claim 對已被認領的 change 輸出含持有人的錯誤訊息 <!-- speclink-task:tsk_01M0XTKF6SN9JYQGHYNFFJJWV6 -->
