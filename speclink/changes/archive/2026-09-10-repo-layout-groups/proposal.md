## Summary

倉庫層的目錄整理：`crates/` 的 12 個 crate 依架構文件 §4 分成五組資料夾、`scripts/` 的 36 個檔案依用途分成五組（安裝腳本留根）、兩份內部藍圖移入 `docs/design/`、刪除根目錄的 `prompt.md`，並同批更新所有路徑引用與補一條「無殘留舊路徑」的靜態守門；crate 名、套件名、npm 發佈路徑、人眼與 `--json` 輸出全部不變。

## Motivation

目標使用者是第一次打開這個 repo 的人（新加入的開發者、透過 AI 代理讀碼的 SDD 使用者），使用情境是「不靠搜尋就預測功能住哪、哪些檔案是一組」。來源討論 improve-repo-layout（Round 1、6–11）的查證：

- `docs/platform-architecture.zh-TW.md` §4 把系統分成 Engine SDK／Host／Store Adapter／Invocation Adapters／Client Protocol 五層，但 `crates/` 底下 12 個 crate 平鋪，分層只活在文件裡。最具體的證據是撞名：`speclink-fs`（引擎儲存介面的預設檔案系統實作，被六個 crate 依賴）與 `speclink-store-fs`（TeamStore 契約的檔案系統驅動）兩個名字都叫 fs，光看目錄分不出誰是誰。
- `scripts/` 36 個檔案一層平鋪，實際是六種用途（發版、安裝、desktop、npm 打包、文件守門、開發）；「發版到底跑哪幾支」要讀 release.yml 才拼得出來。
- 根目錄一份 7 行、無標題、零引用的 `prompt.md`（早期規劃筆記，內容已被架構藍圖吸收）；`docs/` 把使用者文件與內部藍圖混在同一層。
- 這是 improve-repo-layout 三刀中的第一刀：它決定後兩刀（core 模組分組、server 模組分組）的最終路徑，先落地後兩刀的 tasks 才能寫最終路徑。

## Proposed Solution

影響的 crate 與 app：全部 14 個 workspace member 的目錄位置（crate 名不變）、`apps/desktop/core` 與 `apps/desktop/src-tauri` 的 path 相依；`.github/workflows` 三檔；`deploy/` 兩檔；root、desktop 與 server-npm 的 package.json；`scripts/` 全部；docs 與 README；`openspec/config.yaml` 兩處字面；正典規格散文 30 處路徑字串。不新增或變更 CLI 指令、旗標、stdin 與 exit code；不涉及設定欄位語意。

1. **crates 五組**（逐字對齊架構文件 §4）：`crates/engine/`（speclink-core、speclink-fs）、`crates/host/`（speclink-host、speclink-server）、`crates/store/`（speclink-store、speclink-store-fs、speclink-store-sqlite、speclink-store-postgres）、`crates/protocol/`（speclink-protocol、speclink-remote）、`crates/adapters/`（speclink-cli、speclink-node）。改 workspace members 一行與 14 份 Cargo.toml 的 44 條 path 相依；Cargo.lock 逐位元不變（crate 名與版本不變）。
2. **scripts 五組**：`scripts/release/`（release-notes、release-notes-render、release-notes-scratch、release-latest-json、signing-gate、homebrew-formula 與各自測試，加 delivery-gate.test）、`scripts/npm/`（npm-engine-package、npm-server-package 與測試、npm-server-launcher.test）、`scripts/desktop/`（desktop-install、desktop-sidecar 與測試）、`scripts/docs/`（docs-links、docs-screenshots 與測試、docs-parity.test、remote-docs.test、vocabulary-guard.test）、`scripts/dev/`（dev、cli 與測試）。`install.sh`、`install.ps1`、`install.test.mjs` 留在 `scripts/` 根，因為 README 兩份、getting-started 兩份與 release-notes 產出的安裝指令共寫死 raw GitHub 網址 `scripts/install.sh`。
3. **docs 與根目錄**：`docs/design/` 收 platform-architecture.zh-TW.md 與 implementation-refactor-roadmap.zh-TW.md（README 未連結、只有中文、互相引用）；其餘 README 連結的 12 對文件留 `docs/` 根；刪除 `prompt.md`。
4. **路徑引用同批更新**：ci.yml、node-sdk.yml、release.yml、deploy/docker-compose*.yml、Dockerfile 註解、三份 package.json、`.claude/skills/release/SKILL.md`、scripts 內部互相引用（含 docs-screenshots 對 cli.mjs 的跨組 import）、程式碼註解與訊息字串七處、README 的 cargo install 路徑、docs 散文、`openspec/config.yaml`（專案說明的藍圖路徑；tasks 規則的 scripts 測試指令）、正典規格散文 30 處（25 處 crate 路徑、5 處 scripts 路徑；純定位字串直接改寫，@trace 註解區塊不動——使用者裁定，延伸「Purpose 直編」前例）。
5. **scripts 測試 glob**：ci.yml 與 config 規則的 `scripts/*.test.mjs` 改為 `scripts/*.test.mjs scripts/*/*.test.mjs` 兩個明確樣式（bash 未開 globstar 時 `**` 不遞迴；兩樣式在任何 shell 都成立），root package.json 的 test:all 已是遞迴樣式不動。
6. **靜態守門**：delivery-gate 測試新增一條斷言——repo 內不得殘留舊形 `crates/speclink-<name>` 與 `scripts/<已搬檔名>` 字串（排除 `.speclink`、`.dev`、openspec/changes、openspec/discussions、CHANGELOG.md、@trace 註解區塊、帶 `path-guard:allow` 標記的行與 `scripts/install.*` 三檔）；docs 約 40 處散文因此不靠人眼盤點。反例文件（正典規格逐字引用舊形路徑）以行標記豁免。

**相容性影響**：人眼輸出與 `--json` 皆不變；render golden 不變（asset 內容未動）。可觀察差異只有三類文件化的指令路徑：README 的 `cargo install --path crates/adapters/speclink-cli`、repo 自用發版技能與 desktop-release 規格中的 `node scripts/desktop/desktop-install.mjs`、release-notes 規格中的 `scripts/release/release-notes-render.mjs`；公開安裝網址不變。既有 checkout 的使用者 pull 後重跑 `cargo build` 即可，無資料遷移。

**驗證方式**（討論 Round 7 裁定）：ci.yml 為主——`cargo build --workspace --locked` 驗 path 相依、docker smoke 用同一 Dockerfile 路徑覆蓋 release.yml 唯一實質的路徑改動、scripts 測試面驗守門斷言；node-sdk.yml 於合併 commit 驗；不以丟棄 tag 演練 release.yml。

## Non-Goals

- 不改 crate 名（`speclink-fs` 改名留 Deferred）、不改 npm 套件名與發佈路徑、不改 Cargo.lock。
- 不動 `apps/desktop/core`（桌面專用邏輯留在 apps/）。
- 不動 core 與 server 內部的模組分組（刀二、刀三）。
- 不改 README 的公開安裝網址；不改 @trace 註解區塊的歷史路徑。
- 不為 release.yml 加 workflow_dispatch，不演練 tag。

## Alternatives Considered

- 四組（protocol 併進 engine 或 adapters）——protocol 被 host／cli／remote／server 四方共用，塞進任一層都造成跨層引用，且文件給它獨立一節。
- server 歸 adapters——與文件「server 是 Host 的 adapter」字面衝突。
- 安裝腳本自成 `scripts/install/`——公開網址釘死在根，搬了就斷。
- 正典規格路徑走 MODIFIED delta——約 25 條需求逐字複製只換路徑，delta 約 2000 行；使用者裁定直接改字串。
- 只改路徑不加守門斷言——docs 40 處靠人眼，下次再搬一定漏。

## Impact

- Affected specs：delivery-baseline（ADDED：倉庫路徑守門要求與 scenario）。正典散文的 30 處路徑字串以直接編輯處理，不列 delta：archive-skill、baseline-skill、client-protocol、commit-skill、config-skill、delivery-baseline、discuss-skill、drift-computation、host-runtime、improve-skill、manual-skill、node-sdk-release、propose-skill、reference-server、store-abstraction、trace-skill、ui-copy-vocabulary、workflow-schemas、desktop-release、release-notes。
- Affected code:
  - New: crates/engine/、crates/host/、crates/store/、crates/protocol/、crates/adapters/（目錄；內容為搬移）、scripts/release/、scripts/npm/、scripts/desktop/、scripts/docs/、scripts/dev/（目錄；內容為搬移）、docs/design/（目錄；內容為搬移）
  - Modified: Cargo.toml、crates/speclink-core/Cargo.toml 等 12 份 crate Cargo.toml、apps/desktop/core/Cargo.toml、apps/desktop/src-tauri/Cargo.toml、.github/workflows/ci.yml、.github/workflows/node-sdk.yml、.github/workflows/release.yml、deploy/docker-compose.yml、deploy/docker-compose.postgres.yml、crates/speclink-server/Dockerfile、package.json、apps/desktop/package.json、packages/server-npm/package.json、crates/speclink-node/package.json、.claude/skills/release/SKILL.md、openspec/config.yaml、README.md、README.en.md、docs/development.md、docs/development.zh-TW.md、docs/remote-getting-started.md、docs/remote-getting-started.zh-TW.md、docs/product-status.md、docs/product-status.zh-TW.md、docs/sdk-node.md、docs/sdk-node.zh-TW.md、docs/roadmap.md、docs/roadmap.zh-TW.md、docs/implementation-refactor-roadmap.zh-TW.md、scripts/delivery-gate.test.mjs、scripts/desktop-install.mjs、scripts/desktop-sidecar.mjs、scripts/docs-links.mjs、scripts/docs-screenshots.mjs、scripts/dev.mjs、scripts/dev.test.mjs、scripts/homebrew-formula.mjs、scripts/homebrew-formula.test.mjs、scripts/release-latest-json.mjs、scripts/release-latest-json.test.mjs、scripts/release-notes-render.mjs、scripts/signing-gate.mjs、scripts/signing-gate.test.mjs、scripts/vocabulary-guard.test.mjs、scripts/remote-docs.test.mjs、scripts/desktop-install.test.mjs、apps/server-web/src/pages/admin/AuditPage.tsx、crates/speclink-cli/tests/it/init_tools.rs、crates/speclink-core/src/drift.rs、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-node/.cargo/config.toml、crates/speclink-server/src/assets.rs、.gitignore、.dockerignore、packages/ui/src/tasks.ts、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/discussionDrawer.test.tsx、20 份正典規格的散文路徑
  - Removed: prompt.md
