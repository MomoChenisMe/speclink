## Why

npm 上的 `@speclink/engine` 0.13.0，型別定義檔編譯不過：任何匯入這個套件的 TypeScript 專案，tsc 都回報 91 個語法錯誤，第一個是 `index.d.ts(524,59): error TS1131`。原因是 `Host.handleRequest` 的 JSDoc 範例寫了 `express.raw({ type: '*/*', limit: '4mb' })`，`'*/*'` 裡的 `*/` 讓註解在那裡結束，後面的範例文字被當成程式碼。`skipLibCheck` 只跳過型別檢查、不跳過語法錯誤，所以使用端關不掉，只能用套件管理工具的修補功能改掉那一行，每次升級都要重新確認。

受影響的人是用 TypeScript 內嵌引擎的宿主開發者（在自己的 Node 後端呼叫 createEngine 或 createHost）。執行期不受影響：index.js 與原生模組正常。

這個錯能發佈出去，是因為打包流程沒有任何一步會編譯型別定義檔：`crates/adapters/speclink-node` 的測試用 vitest，vitest 不解析 index.d.ts；打包後的檢查只看 tarball 裡有沒有平台載入器與平台二進位。另外，測試跑的是建置產物，沒有任何一步把打包好的套件當成使用端來載入。

這個變更來自討論 embedded-host-needs-4 的 cut 1。它刻意只收這一項，封存後單獨發 0.13.1，讓宿主儘早拿掉修補檔；同一份討論的其他項目在 cut 2。

查過的正式規格：node-sdk-release（打包與發布的保證，這次要加一條）、node-sdk（SDK 的行為契約，這次不動：修的是註解，沒有任何型別或行為改變）。

## What Changes

- 修掉型別定義檔的那一行：`crates/adapters/speclink-node/index.d.ts` 裡 `Host.handleRequest` 的 Express 範例，`'*/*'` 改寫成 `'*\/*'`。在 JavaScript 裡兩種寫法是同一個字串，範例照樣可以複製使用，註解不再提早結束。
- 新增一支使用端檢查腳本 `scripts/npm/npm-engine-consumer-check.mjs`：收主套件的 tarball（必填）與一個平台子套件的 tarball（選填），在暫存資料夾把它們解開成使用端的 node_modules，然後做兩件事——
  1. 用 Node 載入主套件，取得執行期實際匯出的名稱；
  2. 產生一支具名匯入這些名稱的 TypeScript 檔，以 tsc 編譯（不輸出、不開 skipLibCheck）。
  載入失敗、型別定義檔有語法或型別錯誤、或某個執行期匯出在型別定義檔裡沒有宣告，腳本都以非零結束並印出原因。
- 品質關卡第 1 輪之後補上的兩條規則（同一支腳本，沒有新檔）：
  1. 主套件載入成功但沒有任何執行期匯出時，檢查以非零結束。沒有名稱可以比對時，原本的寫法會直接通過，等於什麼都沒檢查。
  2. 匯出的名稱不是合法識別字時（例如 `default` 或含連字號）照樣比對。原本的寫法會讓這種名稱以語法錯誤失敗，而且訊息把錯怪到型別定義檔上。
- `.github/workflows/node-sdk.yml` 的 package 工作，在打包步驟之後、上傳 tarball 之前，對主套件 tarball 與 Linux x64 的子套件 tarball 執行這支腳本。push、PR 與 release 三種觸發都執行；失敗時不產出可發佈的 tarball。
- `crates/adapters/speclink-node` 的開發相依加上 typescript（只給這個檢查用，不進發佈的套件）。
- 既有的 workflow 守門測試（`scripts/release/delivery-gate.test.mjs`）加上斷言：package 工作有這一步，而且排在打包之後、上傳之前。

相容性影響：
- 發佈的套件只有 index.d.ts 的一行註解不同。型別、執行期行為、`--json` 輸出、人眼輸出都不變，沒有回歸對照要更新。
- 已經用修補檔繞過的宿主，升到 0.13.1 之後拿掉修補檔即可；沒拿掉的話，套件管理工具會因為那一行已經不同而回報修補套不上，不會默默留著。
- 這個變更沒有新增或改動任何引擎命令、CLI 動詞、文件種類、設定欄位或技能步驟，所以本機 repo、連到遠端的 checkout、內嵌宿主三種形態的行為都不變；內嵌宿主是唯一讀 index.d.ts 的形態，得到的是可以編譯的型別定義檔。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `node-sdk-release`: 新增一條需求——打包好的主套件在產出可發佈的 tarball 之前，要通過使用端檢查（能被載入而且至少有一個執行期匯出、型別定義檔能被編譯、每個執行期匯出都有型別宣告，名稱不是合法識別字的也算在內）。既有的三條需求不動。

## Impact

- Affected specs: node-sdk-release
- Affected crates: speclink-node（只動型別定義檔的註解與開發相依；Rust 程式碼不改）。speclink-core、speclink-cli、speclink-host、speclink-server 與 apps/desktop 都不改。
- 發版：封存後以既有的發版流程發 0.13.1；發版本身不是這個變更的任務。
- Affected code:
  - New:
    - scripts/npm/npm-engine-consumer-check.mjs
    - scripts/npm/npm-engine-consumer-check.test.mjs
  - Modified:
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/package.json
    - crates/adapters/speclink-node/package-lock.json
    - .github/workflows/node-sdk.yml
    - scripts/release/delivery-gate.test.mjs
  - Removed: (none)
