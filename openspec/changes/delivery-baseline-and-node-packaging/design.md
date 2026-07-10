## Context

implementation-refactor-roadmap 的 G0 交付 Gate 要求先修復驗證基礎，才允許 `engine-typed-core` 進入 Node dispatch 遷移與全量回歸。現況（逐項實測）：

- `crates/speclink-node/package.json` 的 optionalDependencies 宣告五個 `@speclink/engine-*` 平台套件（0.1.0），但這些套件從未發佈到 npm registry，因此 package-lock.json 無法記錄對應條目，npm ci 以 EUSAGE 失敗（Missing from lock file ×5）。Node SDK workflow 的安裝步驟使用 npm ci，任何碰 `crates/**` 的 push 該 workflow 必紅。
- 主 CI（ci.yml）只做 cargo build --release --locked 加 CLI smoke；cargo test --workspace、`packages/ui`（201 tests）與 `apps/desktop`（113 tests）的 vitest 全部不在 CI。
- root package.json 有 workspaces（`packages/ui`、`apps/desktop`）與 package-lock.json，但 scripts 為空；`crates/speclink-node` 是獨立 npm 專案（自帶 lock），不在 root workspaces。
- Desktop 測試通過但輸出 React act(...) warnings，表示 async 更新未被測試等待，存在假綠風險。
- release workflow 只發佈 CLI binary，無任何 npm 發佈步驟。

本變更不落在 `speclink-core` 或 `speclink-cli` 的流程邏輯——兩個 crate 的原始碼零改動；改動面全部在 packaging 檔、CI 設定、root scripts 與測試檔。

## Goals / Non-Goals

**Goals:**

- 乾淨環境下 `crates/speclink-node` 的 npm ci 確定性成功。
- root 單一指令依序執行四個測試面：Rust workspace、`packages/ui`、`apps/desktop`、`crates/speclink-node`。
- 主 CI 執行完整測試（cargo test --workspace ＋ npm workspace 測試），Node SDK workflow 恢復綠燈。
- Desktop 測試輸出的 act(...) warnings 清零。

**Non-Goals:**

- 不動 `crates/speclink-core`、`crates/speclink-cli`、`crates/speclink-node/src`、`packages/ui/src`、`apps/desktop/src` 的產品程式碼（測試檔除外）。
- 不發佈 npm 套件、不新增發佈管線；optionalDependencies 的發佈時注入屬未來 release 變更。
- 不改 CLI 人眼／--json 輸出；parity／color／twin 回歸對照不需更新。
- 不處理逾出 packaging／設定／測試檔範圍的平台性既有紅燈與元件 async 真 bug（各自另開 change）。

## Decisions

### 決策一：npm ci 修復——committed package.json 不宣告未發佈的平台套件

移除 `crates/speclink-node/package.json` 的 optionalDependencies 區塊，以 npm install 重生 package-lock.json。`napi.triples` 設定保留——Node SDK workflow 的 create-npm-dir 與 artifacts 步驟由 triples 驅動，不依賴 optionalDependencies；開發與 CI 的 native module 一律由本機 napi build 產生（binding.js ＋ .node），本來就不經 registry 安裝。未來實際發佈 npm 套件時，再由該 release 變更把 optionalDependencies 注入發佈用的 package.json（napi 生態的標準做法）。

替代方案與取捨：

- 發佈 placeholder 套件佔住 registry 名稱：把發佈行為帶進本變更（違反 Non-Goal），且 0.1.0 空殼套件會被真實使用者安裝到，拒絕。
- 以 file: 協定把 npm/ 平台子套件目錄 vendor 進版本控制：五個空殼目錄常駐 repo，pack 時還要剔除，維護成本高於價值，拒絕。
- 只重生 lock、不動 package.json：不可行——registry 查不到套件 metadata，npm install 自身就以 E404 失敗。

向後相容：既有開發流程（npm run build、npm test）不變；移除的宣告對任何現行消費者無影響（套件未發佈，無人依賴）。

### 決策二：root 單一指令——root package.json scripts，不引入新工具

在 root package.json 新增 test:all script，依序執行：cargo test --workspace → npm test -w packages/ui → npm test -w apps/desktop → `crates/speclink-node` 的依賴安裝＋napi build＋vitest。以 && 串接（npm scripts 在 Windows 走 cmd.exe，&& 語意一致，跨平台成立）。

- 不把 `crates/speclink-node` 加入 root npm workspaces：加入會共用 root lock 與 hoisting，改變 napi CLI 與 vitest 的解析位置，並牽動 Node SDK workflow 以 `crates/speclink-node` 為 working-directory 的假設。取捨：保留一次獨立的依賴安裝，換取兩個 lock 各自穩定、CI 矩陣行為不變。
- 替代方案：Makefile／justfile——Windows 原生無 make，引入新工具依賴，違反「禁止過度設計」，拒絕。

### 決策三：CI 結構——擴充 ci.yml，Node 測試留在 Node SDK workflow

ci.yml 在既有三 OS 矩陣中保留 build 與 smoke，新增：cargo test --workspace、root npm ci、npm test -w packages/ui、npm test -w apps/desktop。Node 側不搬進 ci.yml——Node SDK workflow 已涵蓋五平台 build、三個可原生執行平台的 vitest 與 tarball 打包，npm ci 修復後恢復綠燈，矩陣結構不動。

apply 期間揭露的一個潛在 CI 紅燈一併處理：engine.spec 的 CLI parity 測試（helpers 的 cliJson）寫死使用 target/debug/speclink，但 Node SDK workflow 只執行 napi release build、從不產生 debug CLI——測試在 CI 必以 ENOENT 失敗（本機不可見，因 cargo test 順手建了 debug binary）。修法：helpers 改為 debug 不存在時退用 release 路徑（測試檔），workflow 對三個可測平台補一步 cargo build --release -p speclink-cli（與 napi release build 共用編譯產物，增量成本低）。這是對「node-sdk.yml 零改動」的單步修正，矩陣與打包結構不變。

首次 push 後 CI 揭露：main 的 ci.yml 自 2026-07-05 desktop（Tauri）crates 加入 Cargo workspace 起即連續紅燈（最後綠燈 74264f2），成因兩個、皆為 CI 環境缺口而非程式碼：（a）ubuntu runner 缺 GTK/WebKit 系統庫，glib-sys 的 build script 失敗；（b）tauri::generate_context! 在編譯期要求 frontendDist（apps/desktop/dist）存在，CI 從不建前端——本機因 dist 殘留而不可見。修法（僅動 ci.yml）：Linux 補裝 Tauri 系統依賴、cargo test 前先以 vite 建 desktop 前端；既有 Build (release) 步驟縮為 -p speclink-cli（smoke 只需 CLI，全 workspace 由 cargo test 的 dev profile 編譯一次，避免 Tauri 雙 profile 重複編譯）。

替代方案與取捨：

- 另開第三個 workflow 專跑測試：多一處觸發條件與快取設定要同步，拒絕。
- 把 Node build/test 矩陣搬進 ci.yml：與 Node SDK workflow 重複五平台編譯，CI 時間翻倍，拒絕。

跨平台風險：三 OS 首次全量跑 cargo test --workspace 可能揭露平台性既有紅燈（路徑分隔、換行、git 行為），處置見「風險」。

### 決策四：act(...) 清零——只修測試檔的等待缺失

以 Testing Library 的 async 工具（await findBy*、waitFor）或明確的 act 包裹，讓觸發 async state 更新的測試操作被正確等待；vitest 設定與元件原始碼不動。驗收以測試輸出不含 "not wrapped in act" 字樣為準。

替代方案：全域壓制 warning（console filter 或環境旗標）——只是把假綠風險藏起來，與目標相反，拒絕。若追查發現 warning 源頭是元件本身的 async bug，依 proposal Non-Goal 另開小刀，本變更不修元件。

### 決策五：揭露型紅燈的單點豁免——store_bridge 的 ChangeMeta 欄位補齊

apply 期間揭露：ChangeMeta 在先前變更中新增了 board_rank 與 restale_from 欄位，crates/speclink-node/src/store_bridge.rs 的初始化未跟上，cargo build -p speclink-node 以 E0063 編譯失敗——speclink-node 是 Cargo workspace 成員，等同 main 上 cargo test --workspace 已紅，先前不可見只因 npm 依賴裝不起來且 CI 不跑測試。此斷裂直接阻擋本變更的 Node 面驗證與 workspace 測試綠燈，經使用者裁定納入本變更、以單點豁免處理：僅補兩行欄位映射（board_rank 讀 boardRank、restale_from 讀 restaleFrom），index.d.ts 與其他 src 檔不動（型別宣告本就落後於 bridge，其補齊屬 engine-typed-core 之後的契約工作）。

替代方案與取捨：

- 另開小刀 change：嚴守原 Non-Goals，但為兩行機械修正跑完整 propose／apply 週期，儀式成本不成比例，且 G0 全程被阻塞，拒絕。
- 留紅並記錄：G0 的核心驗收（workspace 測試綠）不可能成立，等同 G0 失敗，拒絕。

同一豁免原則延伸到測試側的平台性紅燈修復（proposal Non-Goal 4 本就允許測試檔內按 bug 修）；實際揭露並修復三筆，全部是「期望值未對齊宿主平台語意」的測試缺陷，產品行為零改動：

- crates/speclink-node/__test__/helpers.ts：fixture 路徑以 realpathSync 正規化（macOS tmpdir 是 /var → /private/var symlink，CLI 由 getcwd 回報實體路徑）。
- crates/speclink-cli/tests/discuss_promote_snapshot.rs：TempProject 目錄在非 Windows 平台 canonicalize（同上；Windows 的 canonicalize 會加 UNC 前綴反而失真，故排除）。
- crates/speclink-core/src/config.rs：僅動 #[cfg(test)] 測試模組內一個斷言——驅動字母路徑（C:\）只在 Windows 是絕對路徑，unix 上是合法相對目錄名，該拒絕斷言以 cfg!(windows) 限定。此檔屬 core src，但 cfg(test) 程式碼不編入出貨 binary，核心紅線（CLI 回歸保護）的目的不受影響。
- crates/speclink-core/tests/golden/（claude.snapshot.md、codex.snapshot.md）：main 既有的 golden 失同步（乾淨 HEAD worktree 驗證同紅），依 CLAUDE.md 既定程序於乾淨樹 UPDATE_GOLDEN=1 再生後審視——漂移為純空白行（10 行空行刪除，無實質內容變化），把再生結果帶回工作樹。
- crates/speclink-fs/tests/store_fs.rs：canonical capability 列表斷言假設 readdir 回排序結果（NTFS 排序、APFS 不排序）；trait 明文 unsorted (callers sort)，測試改為排序後比對。

## Implementation Contract

**行為（完成後可觀察）：**

- 乾淨 checkout 在 `crates/speclink-node` 執行 npm ci → exit 0；接著 npm run build 與 npm test 全綠。
- repo root 執行 npm run test:all → 依序完成 Rust／UI／Desktop／Node 四個測試面，任一面失敗即非零 exit。
- push 後 ci.yml 三 OS 全綠（含 cargo test --workspace 與 UI／Desktop vitest）；Node SDK workflow 五個 build job 與 package job 全綠。
- `packages/ui` 與 `apps/desktop` 的測試輸出 grep "not wrapped in act" 零命中。

**介面／資料形狀：**

- root package.json 新增 scripts.test:all（既有 workspaces 欄位不動）。
- crates/speclink-node/package.json 移除 optionalDependencies 區塊；napi.triples、scripts、devDependencies 不動；package-lock.json 與之同步重生。
- .github/workflows/ci.yml 新增測試步驟；node-sdk.yml 僅在可測平台補一步 release CLI build（決策三）；release.yml 零改動。

**失敗模式：**

- test:all 為序列執行、fail-fast：前面測試面失敗即中止並回非零 exit，不吞錯。
- CI 測試步驟失敗使 workflow 紅燈，不設 continue-on-error、不設 allow-failure 平台。

**驗收方式：**

- 每項行為對應上述可執行指令；改動面以 git diff --stat 檢查，僅允許：crates/speclink-node/package.json、crates/speclink-node/package-lock.json、crates/speclink-node/src/store_bridge.rs（決策五單點豁免）、crates/speclink-node/__test__ 的測試檔、crates/speclink-cli/tests 的測試檔（macOS symlink 正規化）、crates/speclink-core/src/config.rs（僅 #[cfg(test)] 模組內的平台條件斷言）、crates/speclink-core/tests/golden 的 snapshot 檔（乾淨樹再生，空白行同步）、crates/speclink-fs/tests 的測試檔（readdir 順序正規化）、.github/workflows/ci.yml、package.json、`packages/ui` 與 `apps/desktop` 的測試檔。
- CLI 零影響的證據：crates/speclink-core 僅 config.rs 的 cfg(test) 測試模組行（不編入出貨 binary）；crates/speclink-cli 僅允許 tests 測試檔、src 零改動；crates/speclink-node/src 僅允許 store_bridge.rs 一檔的兩行欄位映射。

**範圍邊界：**

- In scope：packaging 宣告、lock 重生、root scripts、ci.yml 測試步驟、測試檔的 async 等待修正、store_bridge.rs 的 ChangeMeta 欄位補齊（決策五單點豁免）。
- Out of scope：npm 發佈、release workflow、Engine／CLI／元件原始碼（store_bridge.rs 豁免除外）、平台性既有紅燈的深度修復、node-sdk.yml 結構調整、index.d.ts 型別宣告補齊。

## Risks / Trade-offs

- **三 OS 全量測試揭露平台性既有紅燈**：緩解——先在本機完成四面全綠再推 CI；CI 揭露的失敗若屬測試檔或設定可修範圍，於本變更內按 bug 修；逾出範圍（需動產品碼）則記錄並另開 change，本變更的 CI 步驟不因此加 continue-on-error 掩蓋。
- **回歸對照**：本變更不動 CLI 原始碼，parity／color／twin 不需重跑；以 git diff 改動面檢查作為零影響證據，成本最低。
- **npm 版本差異造成 lock 重生 churn**：重生 package-lock.json 時以單一 npm 主版本（CI 用的 Node 20 內建 npm）執行，避免 peer 旗標等 metadata 反覆翻動。
- **act 修正的工時不確定性**：warnings 數量未逐一盤點，實際分佈於執行時依測試輸出定位；若單一測試檔的修正牽出元件 bug，依 Non-Goal 切出，不讓本變更膨脹。
