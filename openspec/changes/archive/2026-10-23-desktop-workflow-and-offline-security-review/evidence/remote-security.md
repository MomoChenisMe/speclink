# Protocol、Remote、Server 與 Node 串接驗證

- Protocol 76 tests 通過；舊 binding 的 securityReviewV2 / checkHistory 預設 false，新操作在寫入前回 unsupported_version。
- Host 159 unit + 5 bridge tests 通過；Remote 20 unit + 104 integration tests 通過。
- Server 35 unit tests 通過；整包 integration 原為 350 passed、1 ignored、1 failed（寫入路由檢查清單少兩項）。補上安全 rounds / accept 後，api::write_guard 的 5 tests 全過，包含新路由 reader 403 與零文件效果；安全 API 的 5 個定向測試先前亦已通過。
- 新增 Host security_remote 整合測試通過：本機與 server 使用相同 canonical projectKey / KB 內容指紋、成對發佈原子性、失敗不殘留 basic、風險接受的 index revision CAS、接受理由遮罩、當次 client-attested 來源、過期核對拒絕、無核對不通過、一般 --accept 無法替代來源核對，使用同一 proof 才能 verify stamp。
- Node 先 build 再 test：8 files / 55 tests 通過。新增蓋章後完整紀錄、新一輪不覆寫、已封存 dated 名稱查詢、缺 commitChecks 前置拒絕、無授權 checkout 的 JS Store 安全寫入拒絕。
- JS 宿主的 commitChecks 必須自行提供資料庫交易／原子儲存；Rust bridge 不以分開的舊 write / delete 回呼模擬。SDK 的 index.d.ts 與雙語操作文件已同步。
- CLI / Server cargo check 通過；新增接線之後仍需最終回歸（task 12.6）。未執行實際雲端模型，未於審查下載安全標準。
