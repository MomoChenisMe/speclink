## 1. 版號蓋章腳本（先測試後實作）——落實 spec「engine npm 套件家族與版號蓋章」與 design D2

- [x] 1.1 新增 scripts/npm-engine-package.test.mjs：以 fixture 套件樹（主套件＋數個平台子套件目錄）斷言蓋章後主套件與每個子套件 version 皆為指定版、主套件 optionalDependencies 恰為子套件名各釘同版；另斷言子套件清單來自目錄列舉（fixture 增減目錄時結果跟著變）與缺 --version 參數時以非零結束。測試慣例與 scripts/npm-server-package.test.mjs 同款，先寫並確認紅燈 <!-- speclink-task:tsk_01M0PAACE9SVVNA7YDJWYBCZ87 -->
- [x] 1.2 實作 scripts/npm-engine-package.mjs（介面 --version X.Y.Z --dir <套件根>）：寫入主套件與 npm/ 底下各平台子套件的 version、物化主套件 optionalDependencies（自目錄列舉子套件名，不硬編碼 triple 清單），跑 1.1 測試至綠燈。此組落實 spec 需求「engine npm 套件家族與版號蓋章」與 design D2（版號蓋章＝pack 前跑 scripts/npm-engine-package.mjs，只動 CI checkout） <!-- speclink-task:tsk_01M0PAACE9RGDMY9FENTFDRTQ7 -->

## 2. build/pack workflow 的 workflow_call 化——落實 spec「build/pack workflow 可被 release 管線重用」與 design D1

- [x] 2.1 .github/workflows/node-sdk.yml 的 on: 增加 workflow_call（選填字串輸入 version），既有 push／pull_request 觸發保留，無 version 輸入時的版號行為不變（產物完整性斷言對所有觸發一律生效）——落實 spec 需求「build/pack workflow 可被 release 管線重用」與 design D1（同檔雙角色） <!-- speclink-task:tsk_01M0PAACE91M3449G8X8JJ1CW8 -->
- [x] 2.2 pack job 在 napi artifacts 之後、npm pack 之前，新增條件步驟：version 輸入非空時執行 node scripts/npm-engine-package.mjs --version <輸入值> --dir crates/speclink-node；無輸入時該步跳過，佔位版號打包行為與現行完全一致 <!-- speclink-task:tsk_01M0PAACE9X1XDXC0GXXSMMCQB -->

## 3. release 管線接線——落實 spec「npm 發布閘門與發布順序」與 design D3

- [x] 3.1 .github/workflows/release.yml 新增 engine-npm-build job：以 uses 呼叫 ./.github/workflows/node-sdk.yml 並傳 version（自 tag 名去除 v 前綴），使蓋章 tarball 以 npm-tarballs artifact 存在於同一 workflow run <!-- speclink-task:tsk_01M0PAACE9JRFH1V02E8Z1FE57 -->
- [x] 3.2 release.yml 新增 engine-npm-publish job（needs engine-npm-build 與 release）：npm credential gate 照 server 的 npm-publish job 同語意（NPM_TOKEN 缺席各步跳過且 job 綠）、下載 npm-tarballs artifact、以 npm publish --access public 逐一發布平台子套件 tarball 後最後發布主套件 tarball，不重新打包——落實 spec 需求「npm 發布閘門與發布順序」與 design D3（一個 reusable-workflow job ＋一個 publish job） <!-- speclink-task:tsk_01M0PAACE9QSVFDBXVJDVJV90J -->

## 4. 佔位版號決策落檔

- [x] 4.1 crates/speclink-node/package.json 以 "//" 註解欄記載：版號為佔位符，發布產物版號一律由 release tag 蓋章決定（scripts/npm-engine-package.mjs），不隨 workspace 版本 bump <!-- speclink-task:tsk_01M0PAACE9RP4M2YH0K909QJC1 -->

## 5. 文件翻面（發布宣稱）——落實 design D4

- [x] 5.1 docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 的「取得方式」段改寫：npm install @speclink/engine 為主路徑（附「以首個帶 engine 的 release 為準」的生效說明）、自 repo 建置降為替代路徑 <!-- speclink-task:tsk_01M0PAACE9T9ATYH0Q5R0CSC7Y -->
- [x] 5.2 docs/product-status.zh-TW.md 與 docs/product-status.md 的 Node N-API SDK 列與底部註記：依實作完成當下事實改寫（發布管線已接、自下個 release tag 起上 npm），刷新查核日期 <!-- speclink-task:tsk_01M0PAACE9WQK6P2HFBPK4M71Y -->
- [x] 5.3 docs/roadmap.zh-TW.md 與 docs/roadmap.md 的「SDK / SDK 發布」線：目前到哪與可觀察下一步改寫為管線已接、等首個帶 engine 的 release <!-- speclink-task:tsk_01M0PAACE99GX6EAVAQHX1PSEF -->
- [x] 5.4 README.md 與 README.en.md 的能力狀態行：Node SDK 的「尚未發布至 npm」措辭改為管線已接語意——本組四項皆遵循 design D4（文件翻面的表述基準） <!-- speclink-task:tsk_01M0PAACE9T371SCFC1TT4KFXP -->

## 6. 收尾驗證

- [x] 6.1 兩個 workflow 檔通過 YAML 語法檢查與 GitHub Actions 結構自檢（workflow_call 輸入引用、job needs 圖、artifact 名一致），scripts 測試全綠；speclink validate engine-npm-publish 通過 <!-- speclink-task:tsk_01M0PAACE9813KR45NPJRETF8E -->
- [x] [M] 6.2 在 npmjs.com 確認 @speclink org 的 NPM_TOKEN（repo secrets）為可發布新套件名的 automation token——首次發布 @speclink/engine 與五個平台子套件需要此權限；不足則調整 token 或 org 設定 <!-- speclink-task:tsk_01M0PAACE9124NQY69F4J3JJGZ -->
