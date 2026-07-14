## 1. registry 遷庫

- [x] 1.1 【紅】針對「registry 持久化且 binding 讀庫」的儲存層寫測試：projects/repos 表的建立與讀取、重複 project key 與同 project 重複 repo key 拒絕、schema 演進——前一版資料庫（含既有 user 與 PAT）migrate 升級後資料完整且 registry 可用、較新版本拒開。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXG4MAA5AX7KZD2BJB5G1K82 -->
- [x] 1.2 【綠】identity 資料庫 schema 遞增一版新增 projects/repos 表；identity 層新增 registry 介面（list/get project、list repos、create project、create repo）。1.1 全綠。 <!-- speclink-task:tsk_01KXG4MAA6P66D41QVK1J3J8JA -->
- [x] 1.3 【紅→綠】binding 裁決改讀 registry：auth.rs 的 project 查核與 repo 裁決改查 registry 介面，未註冊 404、repo 未註冊 not_found、多義拒絕、恰一綁定的錯誤分類與訊息維持現值。測試播種 helper 自組態 projects 遷移為 registry 介面呼叫，既有 binding/query/command/discussion/sse 測試改播種後期望不變。 <!-- speclink-task:tsk_01KXG4MAA6NYXB53B1HMKAGFG8 -->
- [x] 1.4 【紅→綠】組態 projects 段退場：config.rs 移除該段，殘留即啟動失敗且原因指出已由 registry 取代（沿用 tokens 段汰換的報告模式）；startup 測試涵蓋「殘留 projects 段拒絕啟動」情境；repo 內全部測試組態同步刪除該段。 <!-- speclink-task:tsk_01KXG4MAA6V1V1045RGXKJKPNT -->

## 2. bootstrap token 與 /setup

- [x] 2.1 【紅】針對「bootstrap token 一次性且以無 admin 為條件」寫測試：全新資料庫啟動 stdout 含 token 與指引且 hash 落庫帶 24 小時到期；已有 admin 啟動不生成且 /setup 回 404；token 過期後重啟生成新 token 舊值作廢；無效/過期/已耗用 token 訪問 /setup 回同一無效回應。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXG4MAA6GNB81N1Y441FR7CF -->
- [x] 2.2 【綠】實作 token 生成與門禁（crates/speclink-server/src/setup.rs：啟動檢查、hash 比對、耗用與關門），2.1 全綠。 <!-- speclink-task:tsk_01KXG4MAA67CQPCQAV37YAN7Q7 -->
- [x] 2.3 【紅→綠】/setup 流程頁（server-rendered，POST 同源驗證）：建立第一位 Admin（active＋admin 旗標、不經邀請）、顯示 store manifest/health 與 identity schema version、建立第一組 Project/Repo（寫 registry、重複 key 表單錯誤）、顯示初始連線資訊（組態 public url 與所建 keys）；冪等續作——已完成的節重入不重建；完成即耗用 token。涵蓋「中斷後憑同一 token 續作」情境。 <!-- speclink-task:tsk_01KXG4MAA68JCEC4ZQ01RTAN3C -->

## 3. invite 查核與端到端

- [x] 3.1 【紅→綠】invite 子命令 --project 對 registry 查核：未註冊 key 非零 exit code 且 stderr 列出既有 project keys；註冊 key 照常建邀請（URL 基底仍為組態 public url）。 <!-- speclink-task:tsk_01KXG4MAA6GZ6EY22895X3C60R -->
- [x] 3.2 【紅→綠】開箱 e2e（涵蓋「setup 流程完成開箱四要素」與「完成 setup 即可邀請與連線」）：全新資料庫啟動真 server → 取 stdout token 完成 /setup（Admin＋第一組 Project/Repo）→ invite 子命令邀請成員 → HTTP 走接受頁與登入建 PAT → 真實 CLI 以該 PAT 對新 project 執行 remote 動詞 → 重啟 server 確認 /setup 關門且資料完整。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXG4MAA6M3MW2FQ0RZPWT5EB -->

## 4. 回歸

- [x] 4.1 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff。驗收：全數通過。 <!-- speclink-task:tsk_01KXG4MAA6AERWQ93DABZSGYDN -->
