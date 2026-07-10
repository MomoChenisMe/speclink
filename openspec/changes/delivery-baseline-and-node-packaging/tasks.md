## 1. Node 套件安裝確定性（design 決策一：npm ci 修復——committed package.json 不宣告未發佈的平台套件）

- [x] 1.1 記錄紅燈基線：在 crates/speclink-node（無 node_modules 的乾淨狀態）執行 npm ci，確認以 EUSAGE 失敗且錯誤訊息列出五個 @speclink/engine-* 平台套件的 Missing from lock file——這是 spec Requirement「Node 套件安裝確定性」的失敗前態，留存輸出作為對照。
- [x] 1.2 移除 crates/speclink-node/package.json 的 optionalDependencies 區塊（napi.triples、scripts、devDependencies 不動），以 Node 20 內建 npm 執行 npm install 重生 crates/speclink-node/package-lock.json；驗證 spec Requirement「Node 套件安裝確定性」轉綠：刪除 node_modules 後 npm ci 以 exit code 0 完成，1.1 的錯誤不再出現。
- [x] 1.3 單點豁免修復（design 決策五：揭露型紅燈的單點豁免）：補齊 crates/speclink-node/src/store_bridge.rs 的 ChangeMeta 初始化欄位映射（board_rank 讀 boardRank、restale_from 讀 restaleFrom），紅燈為 cargo build -p speclink-node 的 E0063 編譯失敗（已留存輸出）；驗證：cargo build -p speclink-node 通過，改動僅此一檔的兩行映射，index.d.ts 與其他 src 檔不動。
- [x] 1.4 驗證 Node 面不回歸：在 crates/speclink-node 執行 npm run build 產生 binding.js 與 .node 檔、npm test（vitest：engine／store-bridge／write-path／stress 套件）全綠；期間揭露 parity 測試的 macOS symlink 路徑差（fixture 用 /var 而 CLI 回報 /private/var），於 crates/speclink-node/__test__/helpers.ts 以 realpathSync 正規化 fixture 路徑（測試檔，屬 Non-Goals 允許範圍）；git diff --stat 確認本組僅改 crates/speclink-node/package.json、crates/speclink-node/package-lock.json、crates/speclink-node/src/store_bridge.rs（決策五豁免）與 crates/speclink-node/__test__/helpers.ts。

## 2. root 單一指令（design 決策二：root 單一指令——root package.json scripts，不引入新工具）

- [x] 2.1 確認紅燈後新增 root 指令：先執行 npm run test:all 確認以 missing script 失敗；於 root package.json 新增 scripts.test:all，以 && 依序串接四個測試面——cargo test --workspace、npm test -w packages/ui、npm test -w apps/desktop、crates/speclink-node 的 npm ci ＋ npm run build ＋ npm test；驗證 spec Requirement「root 單一指令全量驗證」的全部通過場景：npm run test:all 依序輸出四面結果並以 exit code 0 結束（workspaces 欄位與其他內容不動）。
- [x] 2.2 驗證 fail-fast 契約：暫時在任一測試面注入必失敗條件（例如臨時失敗測試），執行 npm run test:all 確認於該面以非零 exit code 中止且後續測試面未執行，隨後還原注入；對應 spec Requirement「root 單一指令全量驗證」的任一面失敗即中止場景。

## 3. 主 CI 完整測試（design 決策三：CI 結構——擴充 ci.yml，Node 測試留在 Node SDK workflow）

- [x] 3.1 本機預跑 CI 將執行的內容並全綠：cargo test --workspace、root npm ci、npm test -w packages/ui、npm test -w apps/desktop（此時允許 act 警告存在，僅要求測試通過）——推 CI 前的風險緩解，任何本機紅燈先在此處理或記錄。
- [ ] 3.2 擴充 .github/workflows/ci.yml 以滿足 spec Requirement「CI 執行完整測試」：三 OS 矩陣保留既有 build 與 smoke 步驟，新增 cargo test --workspace、setup-node、root npm ci、npm test -w packages/ui、npm test -w apps/desktop；所有測試步驟不設 continue-on-error；驗證：push 後 ci.yml 三 OS 全綠。
- [x] 3.3 驗證 spec Requirement「Node native 套件全平台交付驗證」恢復成立：node-sdk.yml 五個 build job 全數成功並上傳 binary artifact、三個可原生執行平台（win32-x64、darwin-arm64、linux-x64）vitest 通過、package job 產出主套件與平台子套件 tarballs；含 apply 揭露的 parity CLI 缺失修復——helpers 的 cliBin 改為 debug 不存在時退 release（crates/speclink-node/__test__/helpers.ts），workflow 可測平台補一步 cargo build --release -p speclink-cli（.github/workflows/node-sdk.yml，design 決策三）。
- [ ] 3.4 處置 CI 揭露的平台性紅燈（如有）：屬測試檔或設定可修者於本組修復並重推至綠；需動產品程式碼者停下記錄並回報（依 proposal Non-Goals 另開 change），不得以 continue-on-error 掩蓋；驗證：最終 ci.yml 三 OS 與 node-sdk.yml 全綠。

## 4. 測試輸出 act 警告清零（design 決策四：act(...) 清零——只修測試檔的等待缺失）

- [x] 4.1 盤點紅燈：執行 npm test -w apps/desktop 與 npm test -w packages/ui，記錄輸出中含 "not wrapped in act" 的測試檔清單與各自觸發的互動（預期 apps/desktop 有、packages/ui 待確認）——spec Requirement「測試輸出無 React act 警告」的失敗前態。
- [x] 4.2 逐檔修正測試側等待使 spec Requirement「測試輸出無 React act 警告」成立：以 Testing Library 的 await findBy*、waitFor 或明確 act 包裹補上 async 更新等待；不動元件原始碼、不新增 console 過濾或警告壓制；若追查發現源頭是元件 async 真 bug，停下記錄並回報（依 proposal Non-Goals 另開 change）；驗證：兩個 workspace 測試全數通過且輸出 grep "not wrapped in act" 零命中，滿足禁止以壓制方式清零場景。

## 5. 改動面驗收與收尾

- [x] 5.1 改動面全檢：git diff --stat 僅含允許清單——crates/speclink-node/package.json、crates/speclink-node/package-lock.json、crates/speclink-node/src/store_bridge.rs（design 決策五單點豁免）、crates/speclink-node/__test__ 的測試檔、crates/speclink-cli/tests 的測試檔（macOS symlink 正規化）、crates/speclink-core/src/config.rs（僅 #[cfg(test)] 模組內的平台條件斷言）、crates/speclink-core/tests/golden 的 snapshot 檔（乾淨樹再生）、crates/speclink-fs/tests 的測試檔（readdir 順序正規化）、.github/workflows/ci.yml、package.json、packages/ui 與 apps/desktop 的測試檔；crates/speclink-core 僅允許 config.rs 的 cfg(test) 測試模組行與 tests/golden snapshot，crates/speclink-cli 僅允許 tests 測試檔、src 零改動，crates/speclink-node/src 僅允許 store_bridge.rs 一檔（CLI src 零影響證據，parity／color／twin 對照不需重跑）。
- [ ] 5.2 對 spec 五條 Requirement 逐一終驗：「Node 套件安裝確定性」（乾淨環境 npm ci exit 0）、「root 單一指令全量驗證」（npm run test:all exit 0）、「CI 執行完整測試」（ci.yml 三 OS 綠）、「Node native 套件全平台交付驗證」（node-sdk.yml 五平台 build ＋ 可執行平台測試綠）、「測試輸出無 React act 警告」（輸出零命中）；全數成立即本 change 可進 verify。
