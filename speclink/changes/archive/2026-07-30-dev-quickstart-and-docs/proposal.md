## Why

clone 專案的開發者目前只有兩種粒度：npm run dev 一次起整套（CLI build＋前端 build＋server＋desktop），或自行拼湊 cargo／npm 指令。「只想跑開箱即用的後端服務」「只想開 desktop 看看」「想直接試 CLI」都沒有一鍵入口；npm run cli 在 binary 未建置時直接報錯要人先跑別的指令。而這些入口即使存在，也沒有一份面向 clone 開發者的文件把它們講清楚。

- 目標使用者：clone 原始碼的開發者與想自架後端的使用者——含只需 server 的自架者、想試 desktop/CLI 但不想走安裝檔的人。
- 使用情境：開發環境啟動與試用，不對應特定 speclink 技能；與既有 dev-harness（npm run dev）互補。
- 源自已結論討論 one-click-install-and-run（與 desktop-installer-and-updater 同源扇出）。

## What Changes

- 新增 npm run dev:server：只驗證 dev 設定並啟動 speclink-server（預設 sqlite、零設定開箱），不建 CLI、不起 desktop；輸出直通、SIGINT 收束與 .dev 持久化沿用既有 dev harness 行為。
- 新增 npm run dev:desktop：先建置 desktop 前端（tauri dev 載入靜態 dist，前端不建會拿到過期畫面）再啟動 tauri dev，不起 server；desktop 以本地模式運作。
- npm run cli 行為變更：checkout debug binary 不存在時改為先自動於 checkout root 建置 speclink-cli 再執行（維持絕不 fallback 到 PATH 版；建置失敗仍以非零收場）。
- 新增 docs/development.md 與 docs/development.zh-TW.md 雙語對：涵蓋全部一鍵入口（dev、dev:server、dev:desktop、dev:reset、cli）的用途與預期輸出，以及下載安裝檔的未簽章放行教學（macOS 系統設定放行、Windows SmartScreen）；README.md 與 README.en.md 各加一節導流連結。
- 相容性影響：npm run dev 與 dev:reset 行為不變；npm run cli 僅在「binary 不存在」路徑從報錯改為自動建置，其餘轉送語意（args 原序、INIT_CWD、exit code、--json 位元級輸出）不變；不動任何 CLI 子指令輸出，render golden 不受影響。

## Non-Goals

- 不引入 justfile／Makefile 等新工具鏈——一切維持 npm scripts（討論已排除）。
- 不改 npm run dev 的整套編排與 dev:reset 語意。
- 不把開發路徑塞進 README 正文或 docs/getting-started（受眾混淆，討論已排除）；README 僅加導流連結。
- 不涵蓋安裝檔與自動更新的實作——屬同討論扇出的 desktop-installer-and-updater；放行教學一節以該變更 spec 定義的產物形態為準。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `dev-harness`: 新增「單獨啟動 server」與「單獨啟動 desktop」兩項需求；修改「checkout 內 CLI 測試入口」——binary 不存在時自動建置後執行（原為報錯終止）。
- `user-documentation`: 新增「開發者入口文件」需求——development 雙語對涵蓋全部一鍵入口與未簽章放行教學，README 導流，並受既有中英對等與可驗證清單約束。

## Impact

- Affected specs: `dev-harness`（修改）、`user-documentation`（修改）
- Affected code:
  - New: docs/development.md、docs/development.zh-TW.md
  - Modified: scripts/dev.mjs、scripts/cli.mjs、package.json、README.md、README.en.md，及 scripts 目錄下對應的 node --test 測試檔
  - Removed: 無
- 相依性：無新增套件；不動 CI
