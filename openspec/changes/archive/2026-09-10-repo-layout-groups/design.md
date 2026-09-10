## Context

倉庫頂層分區（apps／crates／packages／scripts／docs／openspec／deploy）清楚，但分區之下平鋪：`crates/` 12 個 crate、`scripts/` 36 檔、`docs/` 24 檔混放使用者文件與內部藍圖。路徑引用面（掃描時逐一盤點）：workspace members 一行；14 份 Cargo.toml 共 44 條 `path = "../…"`；ci.yml（:69 scripts 測試 glob、:95／:135 sidecar、:162 docker smoke）、node-sdk.yml（8 處 crates/speclink-node）、release.yml（:215-216 release-notes-render、:235 signing-gate、:273 desktop-sidecar、:394 Dockerfile、:503 release-latest-json、:518 release-notes、:550 homebrew-formula、:600 npm-server-package、:311／:470／:685 註解）；deploy/docker-compose*.yml 的 `dockerfile:`；root package.json（dev×4、cli）、apps/desktop/package.json（predev）、packages/server-npm 與 crates/speclink-node 的 package.json 註解；`.claude/skills/release/SKILL.md` 四行；scripts 內部互相引用（用法註解、desktop-install 呼叫 desktop-sidecar、四支測試以 `path.join(root, 'scripts/…')` 起子行程、docs-screenshots `import './cli.mjs'`、delivery-gate.test 與 signing-gate.test 斷言 workflow 內的指令字串、vocabulary-guard 的 SURFACE_DIRS、remote-docs.test 讀 server 的 app.rs／web.rs、desktop-install.test 讀 init.rs、delivery-gate.test 讀 Dockerfile）；程式碼註解與訊息字串七處，加上掃描時補到的五處（root package.json `test:all` 的三個 `npm --prefix`、`.gitignore` 的 napi target 忽略規則、`.dockerignore` 首行註解、`packages/ui/src/tasks.ts` 與 `taskList.test.tsx` 的同構註解、`discussionDrawer.test.tsx` 兩處 readFileSync 實讀 `discuss.rs`）；README 兩份的 `cargo install --path`；docs 散文 11 檔；`openspec/config.yaml` :31 與 :82；正典規格散文 25 處 crate 路徑（11 份技能規格的「事實來源」句、delivery-baseline 4 行、client-protocol／drift-computation／host-runtime／node-sdk-release／reference-server／store-abstraction／ui-copy-vocabulary／workflow-schemas 各 1）與 5 處 scripts 路徑（desktop-release 4、release-notes 1）；@trace 註解區塊另有 3698 處歷史路徑，不動。

硬限制：`scripts/install.sh`／`install.ps1` 的 raw GitHub 網址寫死於 README 兩份、getting-started 兩份與 release-notes 產出；`crates/speclink-core/src/skills.rs` 以 `include_str!("../assets/…")` 相對路徑內嵌（crate 整個目錄搬移，相對路徑不變）；release.yml 由 tag 觸發；Dockerfile 為 `COPY . .`（不含 crates 內部路徑）；ci.yml 的 scripts 測試步驟以 `shell: bash` 展開 glob，bash 未開 globstar 時 `**` 不遞迴。

## Goals / Non-Goals

**Goals:**

- 目錄與架構文件 §4 同形；`speclink-fs` 與 `speclink-store-fs` 由位置分辨。
- scripts 依用途一眼可讀，安裝腳本公開網址不變。
- 全部路徑引用同批更新，且有一條靜態守門讓「殘留舊路徑」在 `node --test` 就紅。
- crate 名、套件名、Cargo.lock、npm 發佈路徑、人眼與 `--json`、render golden 零變化。

**Non-Goals:**

- crate 改名；core／server 內部模組分組（刀二、刀三）；@trace 區塊的歷史路徑；README 公開安裝網址；release.yml 觸發面。

## Decisions

### D1 五組 crates 逐字對齊 §4，server 歸 host、remote 歸 protocol、fs 歸 engine

`crates/engine/{speclink-core,speclink-fs}`、`crates/host/{speclink-host,speclink-server}`、`crates/store/{speclink-store,speclink-store-fs,speclink-store-sqlite,speclink-store-postgres}`、`crates/protocol/{speclink-protocol,speclink-remote}`、`crates/adapters/{speclink-cli,speclink-node}`。以 git mv 搬整個 crate 目錄（含各自的 tests/、assets/、Dockerfile、.cargo/），crate 內部相對路徑（`include_str!`、`.cargo/config.toml` 的 target-dir、napi 建置）不受影響。Cargo.toml：members 五組路徑；同組相依維持 `../<name>`、跨組改 `../../<group>/<name>`；apps/desktop 兩份改 `../../../crates/<group>/<name>`。`cargo build --workspace --locked` 綠且 `git diff --exit-code Cargo.lock` 為零差異即證明 44 條相依全對（path 相依不寫進 lock）。remote↔server 的循環只在 dev-dependencies，不影響分組。

替代案見提案 Alternatives Considered。

### D2 五組 scripts，安裝腳本留根，測試跟腳本走

分組如提案；`.test.mjs` 與腳本同目錄。唯一跨組 import：`scripts/docs/docs-screenshots.mjs` 的 `import { checkoutCliPath } from '../dev/cli.mjs'`。以 `path.join(root, 'scripts/…')` 起子行程的四支測試（homebrew-formula、release-latest-json、signing-gate、install）改為新路徑（install.test 留根不變）；讀 repo 其他檔案的測試（vocabulary-guard 的 SURFACE_DIRS 指向 `crates/engine/speclink-core/assets/skills`、remote-docs.test 讀 `crates/host/speclink-server/src/app.rs`／`web.rs`、desktop-install.test 與 desktop-install.mjs 讀 `crates/engine/speclink-core/src/init.rs`、delivery-gate.test 讀 `crates/host/speclink-server/Dockerfile`）同批改。`root` 常數各檔沿 `path.resolve(dirname, '..')` 推導者改為 `'../..'`（多一層）。

### D3 scripts 測試 glob 改兩個明確樣式

ci.yml 的步驟改為 `node --test scripts/*.test.mjs scripts/*/*.test.mjs`（維持 `shell: bash`）；delivery-gate.test 的「ci.yml 跑 scripts 測試面」斷言改認這個字面；`openspec/config.yaml` tasks 規則的必跑指令同步改為此形；root package.json 的 `test:all` 已是 `"scripts/**/*.test.mjs"`（Node 自行展開）不動。理由：bash 未開 globstar 時 `**` 等同單層，會漏掉根層的 install.test.mjs 或子目錄其一；兩樣式在 bash／zsh／pwsh 皆逐字成立。

### D4 docs/design 與 prompt.md

`git mv docs/platform-architecture.zh-TW.md docs/design/`、`git mv docs/implementation-refactor-roadmap.zh-TW.md docs/design/`；兩檔互相的相對連結同目錄不變；`openspec/config.yaml:31` 的路徑改 `docs/design/platform-architecture.zh-TW.md`；docs-links.mjs 對 docs/ 遞迴掃描（`readdirSync … recursive: true`），斷鏈即紅；docs-parity 的 PAIRS／ZH_DOCS 名單不含這兩檔。`git rm prompt.md`（7 行、零引用、內容已被藍圖 §1／§5／§6 吸收，git 歷史保留）。

### D5 正典規格散文的路徑字串直接改寫（使用者裁定的明文例外）

30 處純定位字串（提案 Impact 列出的 20 份規格）以直接編輯改為新路徑；SHALL NOT 改任何需求名、scenario 名、條文語意；@trace 註解區塊（`<!-- @trace` 到 `-->`）內的歷史路徑 SHALL NOT 動。前例：manual-marker-scope-beyond-tests 的「Purpose 直編」；本案延伸到 Requirements 內的定位字串，proposal 明文記載為例外。守門：`speclink validate --specs` 綠、D6 的斷言綠。

### D6 「無殘留舊路徑」靜態守門

`scripts/release/delivery-gate.test.mjs` 新增一條 test：遞迴走訪 repo（排除 `.git`、`node_modules`、`target`、`dist`、`.speclink`、`.dev`、`openspec/changes`、`openspec/discussions`、`CHANGELOG.md`），對每個文字檔逐行檢查——跳過 `<!-- @trace` 到 `-->` 之間的行、以及帶 `path-guard:allow` 標記的行——不得出現 (a) `crates/speclink-` 開頭且下一段不是五組名之一的路徑（即舊形 `crates/speclink-<name>`），(b) `scripts/<已搬檔名>`（清單＝五組內全部檔名，不含 install.sh／install.ps1／install.test.mjs）。失敗訊息列出檔案與行號。

掃描前先量過命中面，三項排除是硬需求而非潔癖：`.speclink` 是本機執行記錄與凍結快照（400 餘行舊路徑，內容是歷史事實、改寫即說謊），`.dev/config.yaml` 由 `npm run dev` 整檔重寫（改了也留不住），兩者不排除則守門在每台開發機永遠紅、只有 CI 綠；`openspec/changes` 底下進行中的提案必然逐字引用搬移前路徑（本案自己就有 5 行），不排除則 5.1 在封存前綠不了；正典規格把舊形路徑當反例逐字寫出（本案 ADDED 的需求與兩條 scenario），封存後落進未排除的 `openspec/specs/delivery-baseline/spec.md`，因此以 `path-guard:allow` 行標記豁免（使用者裁定；`speclink validate` 實測吃得下行尾 HTML 註解）。

兩個實作細節讓守門不會命中自己：(a) 的 needle 以正則 `crates\/speclink-` 寫出（源碼字面帶跳脫斜線，不構成舊形字串）；(b) 的檔名清單自 `scripts/<group>/` 逐一讀出，不寫死 32 個字面，新增腳本自動納管。這條斷言在搬移前 RED（現況全是舊形）、搬移完成後 GREEN，是本刀的完成判準。對應 spec delivery-baseline ADDED「倉庫路徑守門」。

### D7 驗證與 CI

本地：`cargo build --workspace --locked`＋Cargo.lock 零差異、`cargo test -p speclink-core --test it render_golden`（asset 未動，應逐位元綠）、`node --test scripts/*.test.mjs scripts/*/*.test.mjs`、`speclink validate --specs`、`npm --prefix crates/adapters/speclink-node run build && npm --prefix crates/adapters/speclink-node test`（node crate 目錄搬移後 napi 建置與 binding 路徑）。CI：ci.yml 在 PR 上驗 workspace 建置、docker smoke（同一 Dockerfile 新路徑）、scripts 測試面；node-sdk.yml 於合併 commit 驗 8 處路徑；release.yml 唯一實質路徑（Dockerfile）已被 ci.yml 同指令覆蓋，不演練 tag。

## Implementation Contract

**行為**：`speclink` 全部指令的人眼輸出、`--json`、render golden 逐位元不變；`npm run dev`／`dev:server`／`dev:desktop`／`dev:reset`／`cli` 五條 root script 與 desktop 的 `predev` 行為不變（只換路徑）；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 綠且含新的路徑守門斷言；`docker build -f crates/host/speclink-server/Dockerfile .` 可建；`cargo install --path crates/adapters/speclink-cli` 可裝；README 的 curl／irm 安裝指令逐字不變且可用。

**介面／資料形狀**：無程式介面變動。目錄形狀如 D1／D2／D4；Cargo.toml path 相依如 D1；ci.yml 測試步驟字面如 D3。

**失敗模式**：殘留舊路徑 → 守門斷言紅並指名檔案行號；path 相依漏改 → `cargo build --workspace` 編譯錯誤指名 crate；docs 斷鏈 → docs-links 測試紅；規格改壞 → `speclink validate --specs` 紅。

**驗收**：D7 的本地清單全綠；`git status` 顯示搬移為 rename（相似度 100%）而非刪除＋新增；`git diff --stat` 不含 Cargo.lock 與 crates/…/tests/golden。

**範圍**：In scope＝提案 Impact 全部檔案。Out of scope＝crate 名、core／server 內部模組、@trace 區塊、README 安裝網址、release.yml 觸發方式、記憶檔內的舊路徑（repo 外）。

## Risks / Trade-offs

- **回歸對照**：render golden 與 CLI 凍結測試不受影響（內容零變）；守門斷言是新增的回歸網。跨平台：git mv 在 Windows 上保留大小寫；glob 兩樣式在 pwsh 也成立（Windows CI 的 scripts 步驟以 bash 展開，維持不變）。
- **release.yml 未在 PR 上跑**：其路徑改動只有 Dockerfile 一處實質，ci.yml docker smoke 同路徑覆蓋；其餘為註解。
- **node-sdk.yml 只在合併後跑**：失敗只是 job 紅（不發佈），修了再推。
- **git blame 跨目錄追溯**：`git log --follow` 對搬移後檔案仍可用；接受。
- **與 improve-layout-signal 的順序**：該案 tasks 寫舊路徑，先 apply 該案再 apply 本案；若順序反轉，該案 tasks 的路徑需改為 `crates/engine/…`。
