## Context

`node-sdk.yml` 已在 push main／PR 時建置五平台 release binary、產生平台子套件目錄（napi create-npm-dir → napi artifacts）、npm pack 全部 tarball 並上傳為 `npm-tarballs` artifact；`release.yml` 只在 `v*` tag 觸發，兩個 workflow 之間沒有呼叫橋，跨 run 的 artifact 也拿不到。`@speclink/server` 的 npm 發布已有完整先例：npm credential gate（NPM_TOKEN 缺席跳過且 job 綠）、`scripts/npm-server-package.mjs` 版號蓋章物化、平台子套件先發主套件後發。`crates/speclink-node/package.json` 版號 0.1.0 與 workspace 0.1.3 已漂移，證明 repo 內版號不可作為發布版號來源。

## Goals / Non-Goals

**Goals**

- release tag 推送時自動建置並發布 `@speclink/engine` 主套件與五個平台子套件至 npm，版本一律等於 tag 版
- 不複製五平台 build matrix——node-sdk.yml 的既有 build/pack 以 `workflow_call` 重用
- push／PR 情境的 CI 行為完全不變（無版號輸入時維持佔位版號打包）

**Non-Goals**

- MCP／Copilot tools adapter、獨立 `@speclink/host` 套件、crates.io 發布（見 proposal Non-Goals）
- createEngine 契約變更（平行 change node-host-actor）
- 實際發版 tag 的時點與版號決策

## Decisions

### D1：node-sdk.yml 加 `workflow_call` 觸發，同檔雙角色

`on:` 增加 `workflow_call`，帶一個選填字串輸入 `version`；既有 `push`／`pull_request` 觸發保留。同一份 build/pack 定義服務兩種角色：CI 驗證（無 version，佔位版號）與 release 建置（tag 版蓋章）。**否決**：在 release.yml 內複製五平台 matrix（重複程式碼、兩份漂移）；`workflow_dispatch` 手動觸發（跨 run 拿不到 artifact，且失去 tag 原子性）。reusable workflow 與 caller 屬同一個 workflow run，`npm-tarballs` artifact 可被 caller 的後續 job 直接下載——這是選 `workflow_call` 的決定性理由。

### D2：版號蓋章＝pack 前跑 `scripts/npm-engine-package.mjs`，只動 CI checkout

pack job 在 `napi artifacts` 之後、`npm pack` 之前，於 `version` 輸入非空時執行 `scripts/npm-engine-package.mjs --version <v> --dir crates/speclink-node`。腳本責任：把版號寫入主套件 package.json 與 npm/ 底下每個平台子套件的 package.json，並把主套件的 optionalDependencies 物化為「每個平台子套件名 → 同版」。平台子套件名從 npm/ 目錄實際內容列舉，不硬編碼 triple 清單（napi.triples 是唯一來源，腳本跟著目錄走）。改動只發生在 CI 的拋棄式 checkout，repo 內版號維持佔位符並以 package.json 內註解欄（`"//"`）記載此決策——與 `packages/server-npm` 的既有註解慣例一致。**否決**：napi CLI 的 version／prepublish 子命令（行為綁 napi-rs 版本、optionalDependencies 物化語意不透明，自寫腳本可測試且與 server 先例同構）；發布時驗證 repo 版號與 tag 一致（desktop 式）——版號已漂移證明不可信，蓋章物化才可靠。

### D3：release.yml 的接線＝一個 reusable-workflow job ＋一個 publish job

release.yml 新增 `engine-npm-build`（以 `uses: ./.github/workflows/node-sdk.yml` 呼叫並傳去掉 `v` 前綴的 tag 版）與 `engine-npm-publish` 兩個主 job，外加一個一步的 `engine-version` job 把版號算成 job output——reusable workflow 的 `with:` 只吃表達式，而 GitHub Actions 表達式沒有字串裁切函式（無 `replace`／`substring`），`${GITHUB_REF_NAME#v}` 這類 shell 展開在 `with:` 內不成立，只能由 shell 產出後以 `needs.engine-version.outputs.version` 傳入；`engine-npm-publish` needs `[engine-npm-build, release]`，形狀照抄 server 的 npm-publish job——npm credential gate、下載 `npm-tarballs` artifact、平台子套件 tarball 逐一 `npm publish --access public` 後主套件最後發（optionalDependencies 解析得到同版）。發布單位是 tarball 檔（`npm publish <tgz>`），不再重新 pack。**否決**：把 publish 步驟塞進 reusable workflow 內（push/PR 角色不得帶 secrets 與發布行為，職責混淆）。

### D4：文件翻面的表述基準

八檔中歸本 change 的發布宣稱改為「發布管線已接、自下個 release tag 起上 npm」；sdk-node（中英）的「取得方式」段補 `npm install @speclink/engine` 為主路徑、自 repo 建置降為替代路徑，並明示實際可安裝以首個帶 engine 的 release 為準。product-status 的 Node SDK 列狀態判定與措辭在實作時依當下事實寫（管線落地≠已有套件在 registry）。

## Risks / Trade-offs

- **平台子套件命名對不上**：optionalDependencies 的套件名必須與 napi create-npm-dir 產生的名稱逐字一致——腳本從目錄列舉即天然一致；測試以 fixture 目錄驗證。
- **五平台中兩個平台無法在 CI 上跑測試**（x64-apple 於 arm64 runner、aarch64-linux 交叉編譯）：發布的是既有 build 產物，此風險存在已久且不因本 change 擴大；發布閘門不新增執行期驗證。
- **首發前 npm org 權限**：`@speclink` scope 已有 server 套件發布先例，NPM_TOKEN 已在 secrets；若 token 權限不含新套件名，publish job 紅燈可單獨重跑——不需預先手動佔名。

## Open Questions

（無——npm org 權限與實際發版時點屬 proposal Non-Goals 的延後項）
