## 1. 契約測試先行（TDD 紅燈）——落實 spec 需求「createEngine 的建構期 actor 注入」

- [ ] 1.1 新增 crates/speclink-node/__test__/actor.spec.ts，覆蓋 delta spec 五個 scenario：fs 形式明給 actor 優先於 git identity（created_by 為給值）、fs 形式未給回退 git identity（與 CLI 蓋章逐位元一致）、宿主 Store 形式帶 actor 落章（created_by 與 review／verify 的 _by 皆為給值）、宿主 Store 形式未給維持無章（metadata 不含 created_by）、trim 後空字串視同未給；先寫並確認紅燈——此測試檔即 spec 需求「createEngine 的建構期 actor 注入」的驗證面 <!-- speclink-task:tsk_01M0PAFGYPAM6NB7Y9CXTBX0C2 -->

## 2. Rust 綁定實作（轉綠）

- [ ] 2.1 crates/speclink-node/src/lib.rs：engineFromFs 與 engineFromStore 增選填 actor 參數並存於 Engine 實例；run_engine 的 ExecutionContext actor 解析改為「建構期 actor（trim 後非空）優先 → fs 形式回退 git_identity → 宿主 Store 形式維持 None」；dispatch 簽名不增任何身分參數 <!-- speclink-task:tsk_01M0PAFGYPYH27C9J663DHEGGP -->
- [ ] 2.2 crates/speclink-node/index.js 的 createEngine 把 options.actor 傳入兩種建構路徑；index.d.ts 的 CreateEngineOptions 增 actor?: string 欄位與 JSDoc 說明（"Name <email>" 格式、一實例一身分、多身分宿主開多實例） <!-- speclink-task:tsk_01M0PAFGYPD5KJ387GY9ZER25N -->
- [ ] 2.3 npm run build 後跑 1.1 測試至全綠，並確認既有 __test__ 套件（engine、store-bridge、write-path、dispatch-contract、render）無回歸 <!-- speclink-task:tsk_01M0PAFGYPRPDJGS0MJADA1T0F -->

## 3. 文件對齊（createEngine 契約段）

- [ ] 3.1 docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 的 createEngine 段補 actor 選項：語意（建構期綁定、優先序與兩形式回退）、格式（"Name <email>"）、多人宿主用法（每請求或每身分一個 engine 實例）與「認證是宿主職責、SDK 只收結果」的邊界說明 <!-- speclink-task:tsk_01M0PAFGYPBK3Y5BXMZ2R1YNVQ -->

## 4. 收尾驗證

- [ ] 4.1 speclink validate node-host-actor 通過；cargo test -p speclink-node 與 crates/speclink-node 的 vitest 全綠；git status 盤點確認改動只落在 proposal Impact 列出的檔案 <!-- speclink-task:tsk_01M0PAFGYP69CWNV1M4M9J8HGN -->
