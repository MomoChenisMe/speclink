## 1. env→config 生成邏輯（TDD）

- [x] 1.1 紅（規格「env 到設定的生成邏輯可測」；design「決策 2：env 鍵與映射規則」）：新增 scripts/dev.test.mjs（node --test），對尚未存在的生成純函式寫齊案例——sqlite 全預設的 YAML 形狀（store.path=.dev/store.db、identity.path=.dev/identity.db、public_url=http://localhost:8080）、serverfs（path=.dev/store）、postgres（store.url 取自 SPECLINK_POSTGRES_URL）、memory（無 path/url）、process env 蓋過 .env 檔值、postgres 缺 SPECLINK_POSTGRES_URL 時錯誤點名該鍵、SPECLINK_STORE_DRIVER=mysql 時錯誤列出 sqlite/serverfs/postgres/memory 四合法值。執行 node --test scripts/ 確認全紅。 <!-- speclink-task:tsk_01KXMS129CEZRBAPEGAZS2MR9H -->
- [x] 1.2 綠（design「決策 1：env 插值在編排層，生成 .dev/config.yaml」與「決策 3：零依賴 Node script」）：實作 scripts/dev.mjs 的純函式層——.env 逐行解析（KEY=VALUE、跳過註解與空行、不支援多行與變數展開）與 env→{configYaml, addr} 生成（零依賴、欄位固定字串模板），落實規格「env 驅動的 dev 設定與 .env.example 對照」的解析與 fail-closed 錯誤面。node --test scripts/ 全綠。 <!-- speclink-task:tsk_01KXMS129CQS1R2YQ19ABFJ816 -->

## 2. 編排與生命週期

- [x] 2.1 dev.mjs 主流程（規格「一鍵啟動 remote 開發環境」；design「決策 1：env 插值在編排層，生成 .dev/config.yaml」的主流程面）：確保 .dev/ 存在、每次啟動整檔重寫 .dev/config.yaml（檔頭註明由 npm run dev 生成、手改無效），spawn 兩個 child——cargo run -p speclink-server -- --config .dev/config.yaml --addr 127.0.0.1:{port} 與 desktop 的 tauri dev（經 apps/desktop 的 npm workspace script 呼叫；Windows 上 spawn npm 需 shell 相容處理）——stdio 直通終端。驗證：npm run dev 後終端可見含 /setup?token= 的連結行且 desktop dev 視窗開啟。 <!-- speclink-task:tsk_01KXMS129C5E7Z11HVCG62PTX1 -->
- [x] 2.2 收束語意（design「決策 4：process 生命週期」；規格「一鍵啟動 remote 開發環境」的同殺情境）：SIGINT/SIGTERM 時同殺兩個 child；任一 child 先退出時終止另一個並以其 exit code 退出。驗證：npm run dev 執行中 Ctrl+C 後，程序列表無殘留的 speclink-server 與 tauri/vite dev process。 <!-- speclink-task:tsk_01KXMS129CW46XBV5BAG3WFA6M -->
- [x] 2.3 --reset 模式（規格「dev 資料持久化與顯式重置」；design「決策 5：dev:reset 的邊界」）：npm run dev:reset 遞迴刪除 .dev/ 且僅刪該目錄，對不存在的 .dev/ 冪等成功。驗證：連續執行兩次 dev:reset 皆 exit 0，.env 檔內容不變。 <!-- speclink-task:tsk_01KXMS129CWQPKXWHV35SRQ0QY -->

## 3. 入口與範例檔

- [x] 3.1 root package.json 新增 scripts：dev（node scripts/dev.mjs）、dev:reset（node scripts/dev.mjs --reset）、並把 node --test scripts/ 併入 test:all 鏈；.gitignore 新增 .dev/ 與 .env。驗證：npm run test:all 含 scripts 測試且全綠。 <!-- speclink-task:tsk_01KXMS129C1D63XB78B2AP4NHR -->
- [x] 3.2 新增 .env.example：逐鍵列出 SPECLINK_STORE_DRIVER／SPECLINK_STORE_PATH／SPECLINK_POSTGRES_URL／SPECLINK_IDENTITY_PATH／SPECLINK_PORT／SPECLINK_PUBLIC_URL 與預設值、適用 driver 註記，並註明 SPECLINK_POSTGRES_PASSWORD 為 server 原生機制（URL 可不含密碼）與「postgres 資料重置請自行 drop/recreate database」——落實規格「env 驅動的 dev 設定與 .env.example 對照」的範例檔面。驗證：內容與 design 決策 2 的表逐鍵一致。 <!-- speclink-task:tsk_01KXMS129CYMBN5H35ASMR46B1 -->

## 4. 正典文件

- [x] 4.1（design「決策 6：正典文件落點」）docs/platform-architecture.zh-TW.md §13.4 於開箱流程段後補「本地開發啟動」一段：native 直跑＋設定檔、同一條 /setup 流程、與 docker 部署形態的關係一句話；措辭使用「本地開發啟動」、不得出現「dev server」一詞（避開同節 example/dev server 定位條款）。 <!-- speclink-task:tsk_01KXMS129CC6BJ81M18G0Q4WQD -->
- [x] 4.2（design「決策 6：正典文件落點」）docs/implementation-refactor-roadmap.zh-TW.md §4.2 刀組記入 remote-dev-harness，定位 Phase 3 前置基建（排 phase2-e2e-chain 之後、desktop-workspace-session 之前）。 <!-- speclink-task:tsk_01KXMS129C016KJGBEW09EEA6B -->

## 5. 驗收動線

- [x] 5.1 手動全鏈（覆蓋規格「一鍵啟動 remote 開發環境」與「dev 資料持久化與顯式重置」的持久化/重置情境）：npm run dev:reset → npm run dev（全新，無 .env）→ 瀏覽器走 /setup 建 Admin 與 Project/Repo → 結束後再 npm run dev 確認不再印 setup 連結 → CLI 以 PAT 對 http://localhost:8080 走一條 remote 讀路徑（如 speclink list）確認可用 → npm run dev:reset → npm run dev 回到全新 setup token。另以 SPECLINK_STORE_DRIVER=postgres＋SPECLINK_POSTGRES_URL 指向本地資料庫啟動一次，確認生成的 config 被 server 接受（spot-check 生成形狀與 server 解析的接縫；資料庫內容不需驗證）。 <!-- speclink-task:tsk_01KXMS129CW4MW74TGJ89EB205 -->
- [x] 5.2 回歸：cargo test --workspace 與 npm run test:all 全綠，確認 scripts 測試已接入且未破壞既有測試鏈。 <!-- speclink-task:tsk_01KXMS129CACTRS6JYN47WEFB8 -->
