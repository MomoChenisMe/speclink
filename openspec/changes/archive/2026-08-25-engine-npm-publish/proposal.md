## Why

`@speclink/engine` 的引擎與 Store bridge 已可用且有 dispatch 契約測試守著，但尚未發布至 npm——使用者只能 clone 本 repo、備妥 Rust 工具鏈自行建置，對「寫個腳本把規格接進既有流程」的受眾門檻過高。五平台 tarball 其實已在 CI（node-sdk.yml）持續打包，缺的只是 workflow 邊界的一刀與 publish job；2026-08-12 的發布討論已裁定 npm 通路留待與 engine 一起規劃，本 change 即該規劃的落地。

## What Changes

- `.github/workflows/node-sdk.yml` 的 build/pack 改為可被 `workflow_call` 重用（保留既有 push/PR 觸發不變），並新增選填 `version` 輸入：有值時於打包前把該版號蓋進主套件與五個平台子套件、主套件的 optionalDependencies 以同版指向平台子套件；無值時維持 repo 內佔位版號（現行 CI 行為完全不變）
- `.github/workflows/release.yml` 新增 engine 的 npm publish 流程：以 `workflow_call` 呼叫上述 reusable workflow（帶 tag 版號）取得 tarballs，publish job 沿用 `@speclink/server` 先例語意——NPM_TOKEN 缺席時各步跳過且 job 綠、存在而失敗時紅燈可單獨重跑、平台子套件先發主套件最後發（optionalDependencies 解析得到同版）
- 新增 `scripts/npm-engine-package.mjs` 版號蓋章腳本（把 tag 版寫入主套件與 npm/ 平台子套件、物化 optionalDependencies），仿 `scripts/npm-server-package.mjs` 的介面與測試慣例，附對應測試檔
- `crates/speclink-node/package.json` 的版號明訂為佔位符（發布產物版號一律由 release tag 決定），以註解記載此決策，解除現況 0.1.0 與 workspace 0.1.3 的漂移歧義
- 文件翻面（發布宣稱歸本 change）：README.md、README.en.md、docs/product-status.zh-TW.md、docs/product-status.md、docs/roadmap.zh-TW.md、docs/roadmap.md 的「尚未發布至 npm」宣稱，與 docs/sdk-node.zh-TW.md、docs/sdk-node.md 的「取得方式」段，改為「發布管線已接、自下個 release tag 起上 npm」語意；實際可 npm install 以首個帶 engine 的 release 為準

## Non-Goals

- MCP／Copilot tools adapter（使用者裁定不動；觸發條件＝有非 Claude/Codex 的 agent 平台要接）
- 獨立 `@speclink/host` 套件（藍圖三件套是過期構想；Host 面的 actor 注入由平行 change node-host-actor 收進 createEngine）
- Rust crates 發布至 crates.io（git 依賴即可，公開契約成本不值得）
- 實際發版的 tag 時點與版號（發版時決定，2026-08-12 討論的版號策略不變）
- createEngine 契約段的文件更新（歸平行 change node-host-actor）

## Capabilities

### New Capabilities

- `node-sdk-release`: engine 的 npm 發布通路——reusable build/pack workflow、tag 版蓋章與 publish 閘門。掃描到的近鄰 spec 皆不涵蓋：`server-release` 的「npm 套件一行啟動 server」只管 server 套件家族、`cli-distribution` 只管 CLI 安裝通路、`node-sdk` 只定 SDK 的 createEngine/dispatch/render 契約而對發布沉默。

### Modified Capabilities

- `user-documentation`: 「安裝通路文件與發布狀態誠實化」原本明訂 sdk-node 文件（中英）須寫「尚未發布至 npm」並以 repo 建置示範。發布管線接上後這條成了過期正典，改為「管線已接＋生效時點」的表述：sdk-node 以 `npm install` 為主路徑，同段明示以首個帶 engine 的 release 為準，repo 建置保留為替代路徑。誠實化的意圖不變，變的是「誠實」現在指什麼。

## Impact

- Affected specs: `node-sdk-release`（新）、`user-documentation`（修改）
- Affected code:
  - New: scripts/npm-engine-package.mjs、scripts/npm-engine-package.test.mjs
  - Modified: .github/workflows/node-sdk.yml、.github/workflows/release.yml、scripts/delivery-gate.test.mjs、crates/speclink-node/package.json、docs/sdk-node.zh-TW.md、docs/sdk-node.md、docs/product-status.zh-TW.md、docs/product-status.md、docs/roadmap.zh-TW.md、docs/roadmap.md、README.md、README.en.md
  - Removed: （無）
