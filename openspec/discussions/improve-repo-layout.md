---
topic: 專案資料夾結構的功能分組——core 平鋪模組、crates 分層、server 模組、scripts、頂層雜項
slug: improve-repo-layout
status: promoted
created: 2026-09-09
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
hold: true
promoted_to: repo-layout-groups
---

# Discussion: 專案資料夾結構的功能分組——core 平鋪模組、crates 分層、server 模組、scripts、頂層雜項

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者於 2026-09-09 執行 `/speclink-improve`，指定方向：「整理專案資料夾結構，相關的功能分組，讓整個資料夾人看得懂、結構清楚明瞭」。方向即範圍，跳過熱區推斷。

**掃描面**：頂層（apps／crates／packages／scripts／docs／deploy／openspec＋根檔案）、workspace 14 個 crate 的目錄佈局、`crates/speclink-core/src`（30 個平鋪模組）、`crates/speclink-server/src`（21 個平鋪模組）、`scripts/`（36 個平鋪檔案）、前端三個套件的子目錄、各 crate 的 tests 佈局。

**本掃描的准入判準**：資料夾搬移本身不減少程式碼複雜度，所以刪除測試改問「一個第一次打開這個資料夾的人，能不能不靠 grep 就預測某個功能住在哪、以及哪些檔案是一組的」——分組能讓讀者把一組模組當成一個整體理解才算；只是換個位置擺的不算。前例：cli-verb-family-modules（improve，2026-08）以同一判準把 CLI 三檔重切為 13 個動詞族檔，結論明文「重切是集中複雜度（同族一處可整體理解）而非搬移」。

**Step 1 排除**：`speclink discuss search` 以資料夾／目錄結構／模組切分／重切／crate 搜尋，沒有已否決的分組方案；cli-verb-family-modules 已處理 CLI 層（本次不重提）。在途變更只有 `release-changelog-whats-new`，不重疊。**與 improve-lifecycle-layer 的關係**：該討論（同日結論、`--hold` 在途）的三刀會重寫 core 的 discuss.rs／archive.rs／station.rs／model.rs、刪除 review.rs／verify.rs、新增 KeyLines——core 模組分組（本記錄候選 1）必須排在那三刀之後，否則分組的是一個即將變動的檔案集合。

**硬限制（掃描時查證）**：(a) `scripts/install.sh`／`install.ps1` 的 raw GitHub 網址寫死在 README.md、README.en.md、docs/getting-started.md（各兩處），是對外公開的安裝入口，路徑不得移動；(b) `crates/speclink-core/src/skills.rs:79-93` 以 `include_str!("../assets/skills/…")` 相對路徑內嵌技能，模組下移一層時相對路徑要跟著改；(c) crate 間 44 條 `path = "../…"` 相依、`.github` 三個 workflow 共 11 處 `crates/speclink-*` 路徑、scripts 3 處、docs 約 40 處散文引用；release.yml 由 tag 觸發（記憶：tag 觸發的 workflow 改了要移 tag 才吃得到）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — scan (2026-09-09)

**Focus**: 專案資料夾結構哪裡讓人看不懂、怎麼依功能分組
**Position**: 這個 repo 的頂層分區（apps／crates／packages／scripts／docs／openspec）本身是清楚的，看不懂的地方集中在「分區之下全部平鋪」：core 30 個模組、12 個 crate、server 21 個模組、scripts 36 個檔案各自攤在同一層，讀者要靠檔名猜哪些是一組。候選如下：

---

### 候選 1：`crates/speclink-core/src` 的 30 個平鋪模組依功能分成四組資料夾，大檔的測試搬到子檔

- **Files**：`crates/speclink-core/src/*.rs`（30 個）、`lib.rs`（模組宣告與 re-export）、`skills.rs:79-93`（`include_str!` 相對路徑）；外部呼叫端零改動（見 Solution）。
- **Problem**：訊號 1。同一個概念的模組散在字母序裡：變更生命週期是 model／newcmd／inprogress／discard／archive／tasks／status／listing 八檔，中間夾著 init／instructions／drift／demo；品質關卡是 station／review／verify／validate／analyzer／drift 六檔，散在 a 到 v；工作區與指令交付是 init／skills／instructions／config／schema／workspace 六檔。相依圖證實這三組是真的群：archive 引用 discuss／model／station／tasks／validate 等 13 個模組，幾乎全在「生命週期＋品質」兩群內；init 只引用 config／skills／util；skills 只引用 config／init。另一個訊號：六個檔案超過 2000 行，其中測試佔一半以上（discuss.rs 3051 行、測試 1850；command/mod.rs 3578、測試 1816；init.rs 2753、測試 1655；config.rs 2366、測試 1367；archive.rs 2362、測試 1340；tasks.rs 2022、測試 1052）——讀者打開檔案要先滾過一千多行才確認「程式碼到這裡為止」。
- **Solution**：四組資料夾：`lifecycle/`（model、newcmd、inprogress、discard、archive、tasks、status、listing、capname、preflight、discuss、trace——變更與討論的生命週期，即 improve-lifecycle-layer 命名的那一層）、`quality/`（station、validate、analyzer、drift；review／verify 在該討論刀三後已刪）、`workspace/`（init、skills、instructions、config、schema、workspace）、`command/` 維持；根留 lib.rs、store.rs、util.rs 與測試支援（testkit、teststore、demo）。`lib.rs` 以 `pub use lifecycle::archive;` 等 re-export 維持 `speclink_core::archive` 這類外部路徑逐字不變——CLI／server／desktop／node 四個消費端零改動，改動只在 crate 內的 `crate::` 路徑。測試超過 1000 行的模組改為 `lifecycle/archive.rs`＋`lifecycle/archive/tests.rs`（`#[cfg(test)] mod tests;` 子模組，私有存取不變）。`skills.rs` 的 `include_str!` 改多一層 `../`。零行為變化；`cargo test -p speclink-core` 與四份 golden 不動即回歸網。
- **Wins**：第一次打開 core 的人看四個資料夾名就知道引擎分成哪幾件事；找「封存」不用在 30 個檔名裡掃；大檔一開就是程式碼、測試另檔；未來新模組有明確的落點（不會再出現「這支放哪」的問題）。刪除測試：回到平鋪，三個群的邊界只存在於讀者腦中，每次都要重新拼。
- **Recommendation**：**strongly recommended**——但**排程限制**：必須在 improve-lifecycle-layer 三刀之後（該三刀會刪 review.rs／verify.rs、新增 KeyLines、重寫 discuss／archive／station／model），否則分組的是即將變動的檔案集合，且與那三刀同檔對撞。

### 候選 2：`crates/` 的 12 個 crate 依架構文件的分層分成四組資料夾，順帶消除 `speclink-fs` 與 `speclink-store-fs` 的撞名混淆

- **Files**：`Cargo.toml` workspace members、14 份 `Cargo.toml` 的 44 條 `path = "../…"`、`.github/workflows/{ci,node-sdk,release}.yml` 11 處、`scripts/desktop-install.mjs`／`npm-engine-package.mjs` 3 處、docs 約 40 處散文路徑。
- **Problem**：訊號 1。`docs/platform-architecture.zh-TW.md` §4 把系統分成 Engine SDK／Host／Store Adapter／Invocation Adapters／Client Protocol 五層，但 `crates/` 底下 12 個 crate 平鋪，分層只活在文件裡。最具體的證據是撞名：`speclink-fs`（引擎的預設 `openspec/` 檔案系統 Store，被 cli／host／node／server／desktop-core 六處依賴）與 `speclink-store-fs`（TeamStore 契約的檔案系統驅動）——兩個名字都叫 fs、都是「檔案系統儲存」，光看目錄無法分辨誰是誰，要各自打開 lib.rs 首行註解才知道。
- **Solution**：`crates/engine/`（speclink-core、speclink-fs、speclink-protocol）、`crates/store/`（speclink-store、speclink-store-fs、speclink-store-sqlite、speclink-store-postgres）、`crates/host/`（speclink-host、speclink-server）、`crates/clients/`（speclink-cli、speclink-node、speclink-remote）。crate **名稱不改**（不動發佈與相依名），只動目錄：members 一行、44 條 path 相依、CI 三檔、scripts 三處、docs 散文。`speclink-fs` 的撞名靠位置解決（engine/ 底下 vs store/ 底下），改名留作 Deferred。
- **Wins**：目錄與架構文件同形，新人讀完 §4 就能對到資料夾；「fs 是哪個 fs」由路徑回答。刪除測試：回到平鋪，五層分層只剩文件，撞名回來。
- **Recommendation**：**worth exploring**——收益真實但成本全在邊界：release.yml 由 tag 觸發，路徑改錯要移 tag 重跑才看得到（記憶：release-pipeline-npm-pitfalls）；docs 約 40 處散文引用要同批改（docs-links／docs-parity 測試會抓漏）；44 條 path 相依純機械但一條漏就編譯炸。要先確認 CI 三檔能在 PR 上被完整演練（ci.yml 可，release.yml 需 workflow_dispatch 或丟棄 tag）。

### 候選 3：`crates/speclink-server/src` 的 21 個平鋪模組依「身分與存取／API 面／管理面／網頁面」分四組

- **Files**：`crates/speclink-server/src/*.rs`（21 個），純內部 binary crate，外部零引用。
- **Problem**：訊號 1。最大的檔案 `identity_sqlite.rs`（2232 行）靠檔名前綴告訴讀者它與 `identity.rs`（746 行）是一對；`auth.rs`／`device.rs`／`setup.rs` 也都是身分與存取，但與 `backup.rs`／`assets.rs`／`audit.rs` 混排在字母序裡；`routes.rs`（2099 行）、`read_api.rs`、`verb.rs`、`events.rs`、`context.rs`、`error.rs` 六檔是同一條 HTTP 對外面。`tests/it/` 53 個測試檔同樣平鋪（admin_*、auth_*、backup_*、device_* 靠前綴分組）。
- **Solution**：`identity/`（identity、identity_sqlite→sqlite、auth、device、setup）、`api/`（routes、read_api、verb、events、context、error）、`admin/`（admin、audit、backup）、`web/`（web、assets）；根留 main、lib、app、config、state。`tests/it/` 依同四組分子目錄（Rust 整合測試以 `tests/it/main.rs` 宣告子模組，現況已是此形）。零行為變化，server 整合測試 53 檔即回歸網。
- **Wins**：`identity_sqlite` 這種「用檔名編碼分組」的命名消失；server 的四個面各自一個資料夾，對得上 platform-architecture 的「Client Protocol／管理面／網頁面」用語。刪除測試：回到平鋪，分組再度只靠前綴猜。
- **Recommendation**：**worth exploring**——證據清楚、無外部路徑成本，但 server 不在近期熱區（近一個月只 routes.rs 12 次），收益要等下次動 server 才回本。

### 候選 4：`scripts/` 的 36 個檔案（18 對腳本＋測試）依用途分組；安裝腳本例外留在根

- **Files**：`scripts/*.mjs`／`*.sh`／`*.ps1`；引用面 `package.json` 6 處、`.github/workflows/ci.yml` 4 處、`release.yml` 10 處、`node-sdk.yml` 1 處、`apps/desktop/package.json` 與 `packages/server-npm/package.json` 各 1 處；`.claude/skills/release`（repo 自用發版技能）引用的腳本名。
- **Problem**：訊號 1。36 個檔案一層平鋪，實際是六種用途：發版（release-notes×4、release-latest-json、signing-gate、homebrew-formula、delivery-gate.test）、安裝（install.sh、install.ps1、install.test、cli.mjs）、desktop（desktop-install、desktop-sidecar）、npm 打包（npm-engine-package、npm-server-package、npm-server-launcher.test）、文件守門（docs-links、docs-screenshots、docs-parity.test、remote-docs.test、vocabulary-guard.test）、開發（dev.mjs）。每支腳本的 `.test.mjs` 同名相鄰是好慣例，但六個用途混在一起讓「發版到底跑哪幾支」要靠讀 release.yml 才拼得出來。
- **Solution**：`scripts/release/`、`scripts/npm/`、`scripts/desktop/`、`scripts/docs/`、`scripts/dev/`；**`install.sh`／`install.ps1`／`install.test.mjs` 留在 `scripts/` 根**——raw GitHub 網址寫死在 README 兩份與 getting-started（共六處），是對外安裝入口，搬了就斷。測試檔跟腳本走。引用面同批改。
- **Wins**：「發版跑哪些」看一個資料夾就知道；release 技能與 release.yml 的敘述能對到目錄。刪除測試：回到平鋪，用途分組只在 release.yml 的步驟順序裡。
- **Recommendation**：**worth exploring**——與候選 2 同一種風險（release.yml tag 觸發），兩者宜同一刀改 CI，只驗一次。

### 候選 5：頂層雜項——根目錄的 `prompt.md`、`docs/` 使用者文件與內部設計文件混放

- **Files**：`prompt.md`（根目錄；ed9068a4 的早期規劃筆記「plan engine-as-SDK」，之後未再改）、`docs/`（24 檔：getting-started／configuration／workflow／sdk-node／remote-getting-started 是使用者文件；platform-architecture／implementation-refactor-roadmap／product-status／roadmap／server-backup／server-deployment／server-store-drivers／verb-contract 是設計與營運文件；雙語成對）。
- **Problem**：訊號 1 的弱版。根目錄一份沒有標題的 `prompt.md`，新人會以為是專案級的 AI 提示檔；`docs/` 把「怎麼用」與「為什麼這樣設計」混在同一層，24 個檔名要先讀一遍才分得出。
- **Solution**：`prompt.md` 移入 `docs/design/`（或封存為討論記錄的背景，因其內容就是 remote 模式的起源需求）；`docs/design/` 收 platform-architecture、implementation-refactor-roadmap、roadmap、verb-contract、server-store-drivers；使用者文件留 `docs/` 根。docs-links 測試會抓斷鏈。
- **Wins**：根目錄只剩每個 repo 都有的檔；docs 一層分「用」與「設計」。刪除測試：不做也只是一份多餘檔與一層混排，成本低收益也低。
- **Recommendation**：**speculative**——證據薄、純整潔；可作為候選 2／4 那一刀的順手項，不值單開。

---

**我的首選**：候選 1。它是唯一有「讀者每次都要重拼」的重複成本證據（30 檔三群交錯＋六個 2000 行以上的檔案一半是測試），且外部零改動、無 CI 風險；唯一代價是要排在 improve-lifecycle-layer 三刀之後。第二順位是候選 2＋4 合一刀（同一次 CI 路徑演練吃掉兩個候選的風險），候選 5 順手帶。候選 3 獨立，等下次動 server 前做。

**Open**：使用者要先深挖哪一個候選？

### Round 2 — interview (2026-09-09)

**Focus**: 使用者裁定候選範圍與排程。
**Position**: 五個候選全做。排程裁定：全部排在 improve-lifecycle-layer 的三刀（刀一候選 1＋2、刀二候選 5、刀三候選 3＋4）封存之後才開始深挖與立案——候選 1 的排程限制（core 檔案集合要等那三刀定型）本來就要求如此，其餘四個候選一併順延，避免兩份 improve 的變更在 core／CI 路徑上交錯。本記錄保持在途、不結論，等該三刀封存後回本記錄從候選 1 開始逐一 grill。
**Ruled out**: 先做無相依的候選 2／3／4／5、候選 1 殿後——使用者裁定不交錯，兩份 improve 依序清。
**Open**: 等 improve-lifecycle-layer 三刀封存；屆時第一題＝候選 1 的分組命名與 re-export 策略。

### Round 3 — interview (2026-09-09)

**Focus**: 排程更正＋候選 1 的第一題——core 四組資料夾的命名與成員歸屬。
**Position**: 排程更正：使用者裁定本記錄的討論現在就開始，五個候選逐一 grill 到結論；只有**實作**仍排在 improve-lifecycle-layer 三刀封存之後（候選 1 的檔案集合要等那三刀定型）。候選 1 第一題提案的分組（依 `//!` 首行說明與 `crate::` 相依圖）：
- `lifecycle/`：model、newcmd、inprogress、discard、archive、tasks、status、listing、capname、preflight、discuss、trace。證據：archive 引用 discuss／model／station／tasks／validate 等 13 個模組，其中 9 個在本組；discuss 引用 command／config／model／newcmd；trace 引用 archive／capname／discuss／model／station——trace 的相依全在本組與 quality，它是「變更與討論的溯源」，歸 lifecycle。capname 只被 newcmd／validate／trace 用，是命名主閘，歸 lifecycle（新變更開名時的守門）。preflight 只被 drift／instructions 用，131 行，是 apply 前檢查——歸 lifecycle（它檢查的是變更的狀態）。
- `quality/`：station、validate、analyzer、drift（＋刀三前的 review／verify，刀三後不存在；刀一新增的 keylines 不在此組）。證據：四者都是「對變更做檢查、產出 findings 或裁決」；validate 引用 archive／capname／model／schema（規格結構）；drift 引用 archive／preflight／tasks。
- `workspace/`：init、skills、instructions、config、schema、workspace。證據：init 只引用 config／skills／util／testkit；skills 只引用 config／init；schema 引用 util／workspace；config 引用 skills／util——這六個互相引用、幾乎不引用 lifecycle（instructions 例外：引用 model／preflight／status／tasks 以組 apply 指令）。它們是「本機工作區的設定與指令交付」，即 improve-workspace-sync 處理過的那一層。
- `command/` 維持（mod.rs、typed.rs）。
- 根：lib.rs、store.rs（儲存接縫）、util.rs、keylines.rs（刀一後的文件原語，lifecycle 與刀二的 change meta 都用，屬基礎件）、testkit.rs、teststore.rs、demo.rs（測試與示範支援）。
名稱選「lifecycle／quality／workspace」而非「change／checks／init」：lifecycle 對齊 improve-lifecycle-layer 與 product-status 文件用語、quality 對齊 LANGUAGE.md 的「品質關卡」、workspace 對齊 workspace-tools／workspace-session 規格名。
**Ruled out**: 五組以上（把 discussion 單獨成組）——discuss 一檔一組是「跳好幾個小資料夾」反樣式，且 discuss 與 archive／discard 互相引用密切；依「動詞」分（一動詞一資料夾）——cli-verb-family-modules 已否決同型切法（31 檔百餘行）；不分組、只搬測試——六個 2000 行檔案的問題解了，但 30 檔交錯的主問題沒動。
**Open**: 使用者是否同意四組命名與成員歸屬（特別是 trace／capname／preflight 歸 lifecycle、keylines 留根）？同意後第二題談外部路徑策略（lib.rs re-export 維持 `speclink_core::archive` 不變 vs 讓四個消費端改走新路徑）。

### Round 4 — interview (2026-09-10)

**Focus**: 候選 1 第二題——外部路徑策略：lib.rs re-export 維持既有路徑，還是讓消費端改走新的分組路徑。
**Position**: re-export，而且是永久的、不是過渡期的。證據：core 之外對 `speclink_core::<模組>` 的引用約 500 處、36 個檔案、橫跨 cli／server／host／node／desktop-core／desktop-tauri 六個 crate——CLI 以 `use speclink_core as core` 別名後 `core::command::`（69）、`core::workspace::`（26）、`core::station::`（26）、`core::archive::`（12）…；其他 crate 直呼 `speclink_core::util`（29）、`tasks`（28）、`store`（27）、`init`（27）、`model`（26）。這些呼叫端關心的是「引擎有哪些動詞與型別」，不關心引擎內部怎麼分資料夾；讓它們跟著內部分組改路徑，等於把 core 的目錄結構升格成六個 crate 的耦合面，之後每次再分組都要再改 500 處。做法：`lib.rs` 內 `mod lifecycle; mod quality; mod workspace;`（資料夾模組本身不 pub）＋ `pub use lifecycle::{archive, discuss, model, …};` 一組 re-export，`speclink_core::archive::archive(...)` 這類路徑逐字不變；crate 內部的 `crate::archive::` 也因 re-export 繼續成立，所以連 core 自己的跨模組引用都不必改，改動集中在檔案搬移＋`lib.rs`＋`include_str!` 的相對路徑＋`tests/it` 若有 `#[path]`。資料夾對「讀的人」可見、對「呼叫的人」不可見——這正是候選 1 的目的。
**Ruled out**: 消費端遷移到 `speclink_core::lifecycle::archive`——500 處零收益改動，且把內部分組變成外部契約；資料夾模組設為 pub、新舊路徑並存——兩條路徑指同一物，reader 反而要問「哪條是正典」；`#[path]` 屬性保留平鋪模組名、只搬檔案——`#[path]` 是 Rust 社群公認的可讀性反樣式，lib.rs 會變成 30 行路徑表。
**Open**: 使用者是否同意「資料夾模組私有、lib.rs 一組 re-export 維持全部既有路徑」？同意後第三題談大檔測試搬子檔的門檻與命名。

### Round 5 — interview (2026-09-10)

**Focus**: 候選 1 第三題——大檔的 inline 測試搬到子檔的門檻、檔名與宣告方式。
**Position**: 門檻＝inline `mod tests` 達 500 行；檔名＝與模組同名的子目錄下 `tests.rs`；宣告＝模組檔尾端 `#[cfg(test)] mod tests;`。量測（2026-09-10，含另一 session 進行中的改動）：discuss.rs 2149、command/mod.rs 1901、init.rs 1654、archive.rs 1555、config.rs 1366、tasks.rs 1051、analyzer.rs 588、validate.rs 574、drift.rs 509 ——九個檔過門檻；model.rs 359、schema.rs 189、skills.rs 172 不過（station.rs 在刀三後會吸收門面測試，屆時再看）。500 行的理由：這條線把「打開就是程式碼」的六個 2000 行檔與三個 1000 行檔都收進來，同時不碰十幾個測試 200–400 行、一屏內看得完的模組——搬那些只增加跳檔、沒有可讀性收益。機制：Rust 2018 路徑規則下，`lifecycle/archive.rs` 的子模組檔就是 `lifecycle/archive/tests.rs`；`command/mod.rs` 的是 `command/tests.rs`。子模組保有 `use super::*` 的私有存取，測試內容逐字搬、零改寫；`cargo test -p speclink-core <名字>` 的過濾字串不變（測試路徑多一層 `tests::` 是現況就有的）。
**Ruled out**: 全部模組一律搬——十幾個小檔多出十幾個一屏內的子檔，跳檔成本大於收益；搬到 crate 的 `tests/` 整合測試目錄——失去私有存取，一千多行測試要改寫成走公開 API，且整合測試編譯成獨立 binary 拉長測試時間（記憶：test binary 已從 113 收斂到 11）；門檻用「測試比程式碼長」——drift.rs（程式碼 998、測試 509）這種讀者仍受益的檔會漏掉。
**Open**: 使用者是否同意「500 行門檻、`<模組>/tests.rs`、`#[cfg(test)] mod tests;`」？同意後候選 1 收斂，進候選 2（crates 分層）第一題：四組名稱與 speclink-remote 的歸屬。

### Round 6 — interview (2026-09-10)

**Focus**: 候選 1 收斂（三題皆定：四組＋根、永久 re-export、500 行門檻搬 `<模組>/tests.rs`）。修正 round 5 的一句：量測時工作樹並無他人改動，只是量法從「第一個 cfg(test)」改為「tests 模組起點」，數字差異來自量法，結論不變。進候選 2 第一題——crates/ 分組的名稱、數量與 speclink-remote／speclink-fs 的歸屬。
**Position**: 五組，逐字對齊 docs/platform-architecture.zh-TW.md §4 的五個小節，讓「讀完架構文件就能對到資料夾」成立：
- `crates/engine/`（§4.1 Engine SDK）：speclink-core、speclink-fs。fs 是引擎儲存介面的預設檔案系統實作（core 唯一的直接消費者之一），與 TeamStore 驅動 store-fs 分屬不同層——撞名靠這個位置差解掉，crate 名不改。
- `crates/host/`（§4.2 Speclink Host）：speclink-host、speclink-server。文件原句「HTTP server 只是 Host 的一個 adapter」、server 的 Cargo description「an axum adapter that serves the Client Protocol over the Host → Engine → TeamStore path」——server 歸 host 層，不歸 adapters。
- `crates/store/`（§4.3 Store Adapter Contract）：speclink-store、speclink-store-fs、speclink-store-sqlite、speclink-store-postgres。相依圖自成一島：三個 driver 只依賴 store（與 sqlite 共用 schema），不依賴 core。
- `crates/protocol/`（§4.5 Client Protocol）：speclink-protocol、speclink-remote。remote 是同一份 wire 契約的型別化 client（依賴 protocol；被 cli／server／desktop 消費）；文件把契約與 client 放在同一節。remote 對 server 的依賴只在 dev-dependencies（測試起一個 server），server 對 remote 亦然——資料夾分組不受 dev 循環影響。
- `crates/adapters/`（§4.4 Invocation Adapters）：speclink-cli、speclink-node。兩者都是「把引擎接到某種呼叫方式」：終端與 N-API。desktop 已在 apps/ 不動。
路徑改動面：workspace members 一行、44 條 `path = "../…"` 改為 `../../<group>/…`、.github 三個 workflow 11 處、scripts 3 處、docs 約 40 處散文。crate 名、套件名、發佈路徑（npm）全部不變。
**Ruled out**: 四組（protocol 併進 engine 或 adapters）——protocol 被 host／cli／remote／server 四方共用，塞進任一層都讓另一層跨層引用，且文件明確給它獨立一節；server 歸 adapters——與文件「server 是 Host 的 adapter」字面衝突，讀者會問 host 層為什麼只剩一個 crate；改 speclink-fs 的名字（如 speclink-engine-fs）——名字是 Cargo 相依與 docs 的引用面，位置差已足夠分辨，改名留作 Deferred；把 apps/desktop/core 搬進 crates/——它是桌面專用邏輯，文件把它歸在桌面，跨 apps／crates 搬會讓 tauri 殼與 core 分家。
**Open**: 使用者是否同意「五組逐字對齊 §4、server 歸 host、remote 歸 protocol、fs 歸 engine」？同意後第二題談 CI／release 路徑改動的驗證方式（tag 觸發的 release.yml 怎麼在合併前演練）。

### Round 7 — interview (2026-09-10)

**Focus**: 候選 2 第二題——路徑搬移後 CI／release／docker／scripts 的驗證方式；tag 觸發的 release.yml 要不要用丟棄 tag 演練。
**Position**: 不需要丟棄 tag。理由來自逐處盤點：
- release.yml 只有兩處 crate 路徑：`file: crates/speclink-server/Dockerfile`（docker/build-push-action）與一行註解。Dockerfile 本身是 `COPY . .`（不含任何 crates/ 內部路徑），而 ci.yml:162 在每個 PR 上用同一個 `-f crates/speclink-server/Dockerfile` 路徑做 smoke build——release.yml 唯一實質的路徑改動在 PR 階段就被同一條指令驗過。cargo 步驟用 `-p speclink-cli -p speclink-server` 套件名，不吃路徑。
- node-sdk.yml 8 處（working-directory、artifact path、npm-engine-package 的 --dir）：觸發條件是 push main 且 `paths: crates/**`，巢狀後仍匹配；它在合併 commit 上跑，失敗只是 job 紅，不涉及 tag，修了再推即可。
- ci.yml：`cargo build/test --workspace --locked` 直接驗 44 條 path 相依（Cargo.lock 不含 path 相依的路徑，crate 名與版本不變，lock 逐位元不變）；docker smoke 驗 Dockerfile 路徑；`node --test scripts/*.test.mjs` 驗 scripts 面。
- compose 兩檔（deploy/docker-compose*.yml 的 `dockerfile:`）與 scripts 兩處（desktop-install.mjs 讀 init.rs 取 ASSET_VERSION；npm-engine-package.mjs 註解）、script 測試 6 行（vocabulary-guard 的 SURFACE_DIRS、remote-docs 讀 app.rs／web.rs、desktop-install.test 讀 init.rs、delivery-gate 讀 Dockerfile）：同批改。
- 機械化守門：在 scripts/delivery-gate.test.mjs 補一條斷言——repo 內（排除 openspec/changes/archive 與 openspec/discussions 的歷史記錄、CHANGELOG）不得殘留舊形 `crates/speclink-` 字串；docs 約 40 處散文因此不靠人眼盤點，漏一處 `node --test` 就紅，而它本來就是每個 change 最後一個 task 的必跑項。
**Ruled out**: 丟棄 tag 演練 release.yml——它實質的路徑步驟已由 ci.yml 同指令覆蓋，演練會真的產出 release 資產與 GitHub Release 草稿（記憶：tag 觸發 workflow 的清理成本）；替 release.yml 加 workflow_dispatch 入口——為一次搬移改發版流程的觸發面，超出本候選；只改路徑不加守門斷言——docs 40 處靠人眼，下次再搬一定漏。
**Open**: 使用者是否同意「不演練 tag、ci.yml 為主驗證、node-sdk.yml 於合併 commit 驗、補『無殘留舊路徑』斷言進 delivery-gate 測試」？同意後候選 2 收斂，進候選 3（server 模組分組）：四組成員與 tests/it 是否同步分組。

### Round 8 — interview (2026-09-10)

**Focus**: 候選 2 收斂（五組對齊 §4；ci.yml 主驗證、不演練 tag、補無殘留舊路徑斷言）。進候選 3 第一題——server 四組成員、外部路徑策略、tests/it 是否同步分組。
**Position**: 四組依 `crate::` 相依圖切，外部路徑沿候選 1 的永久 re-export，tests/it 鏡射 src 分四個子目錄。證據：
- `identity/`：identity、identity_sqlite（改名 sqlite.rs，前綴由目錄承擔）、auth、device、setup。相依：auth→identity／config／error／state；device→identity／error／state；setup→identity／audit／state／web；identity↔identity_sqlite 互引。五者的共同核心是「誰是誰、能不能進來」。
- `admin/`：admin、audit、backup。admin→audit／auth／backup／identity／web；backup→identity／identity_sqlite／config；audit 是「管理面稽核日誌」（`//!` 首行），三者是營運與管理面。audit 被 identity／setup 引用屬跨組向下引用（識別層寫稽核），與 admin 引用 identity 對稱，可接受。
- `api/`：routes、read_api、verb、events、context、error。全部只向 auth／state／verb／error／events 引用，是同一條 HTTP 對外面；error 是 wire 錯誤信封的單一對映點（→identity 只為錯誤分類）。
- `web/`：web、assets（assets→web；web→identity／setup／state）。
- 根：main、lib、app（router 組裝，引用全部四組）、config、state（config→events、state→config／events／identity 是共用狀態）。
- 外部路徑：`speclink_server::identity`（8）／`state`（6）／`events`（6）／`config`（5）／`app`（5）／`audit`（2）被 desktop-tauri 與 remote 測試引用，tests/it 另有 114 處（identity 38、state 35、backup 14、audit 12…）——lib.rs 資料夾模組私有＋一組 `pub use` 維持全部路徑不變，與候選 1 同法。
- tests/it：49 個平鋪測試檔以前綴分組（admin_* 10、web_* 6、auth_*／device_*／identity／invite／refresh_rotation 等），改為 tests/it/{identity,admin,api,web}/ 四個子目錄，各自 `mod.rs` 列子模組、main.rs 只列四組＋common 與 startup／health 等跨面檔；檔名去掉前綴（admin_api.rs→admin/api.rs）。`cargo test -p speclink-server --test it admin` 這類過濾字串因模組路徑含目錄名而繼續可用。tests/pg 不動（PostgreSQL 專用鷹架、與 driver crate 刻意不共用）。
**Ruled out**: 三組（web 併進 api）——web 是伺服器渲染 HTML 表單的最小網頁面（`//!` 首行「決策 4」），與 JSON API 是不同對外面，文件與規格（server-web-console）也分開命名；audit 歸 identity——它是管理面稽核，identity 只是寫入者之一；tests/it 不分組只分 src——測試檔數（49）比 src（21）更多，是更需要分組的那一半；改 identity_sqlite 以外的檔名——其餘檔名沒有前綴編碼，不動。
**Open**: 使用者是否同意「四組成員如上、identity_sqlite→identity/sqlite.rs、永久 re-export、tests/it 鏡射分四目錄」？同意後候選 3 收斂（只此一題），進候選 4（scripts 分組）第一題：六組名稱與 install.* 例外。

### Round 9 — interview (2026-09-10)

**Focus**: 候選 3 收斂（四組、identity_sqlite→identity/sqlite.rs、永久 re-export、tests/it 鏡射四目錄）。進候選 4 第一題——scripts 的分組名稱、install 例外、測試 glob 與呼叫端改動面。
**Position**: 五個子目錄＋根留安裝腳本；`.test.mjs` 跟腳本走；兩處非遞迴 glob 同批改為遞迴。盤點（36 檔）：
- `scripts/release/`：release-notes.mjs、release-notes-render.mjs、release-notes-scratch.mjs、release-latest-json.mjs、signing-gate.mjs、homebrew-formula.mjs 與各自 .test.mjs，加 delivery-gate.test.mjs（它守的是 workflow 步驟順序與發版契約，release.yml 是主要受檢物）。呼叫端：release.yml 6 處（:215／:235／:503／:518／:550 與註解）、repo 自用 release 技能 .claude/skills/release/SKILL.md 3 行（:111／:114／:115）。
- `scripts/npm/`：npm-engine-package.mjs、npm-server-package.mjs、npm-server-launcher.test.mjs 與測試。呼叫端：node-sdk.yml:155、release.yml:600、packages/server-npm/package.json 的註解。
- `scripts/desktop/`：desktop-install.mjs、desktop-sidecar.mjs 與測試。呼叫端：apps/desktop/package.json predev、ci.yml:95／:135、release.yml:273、release 技能 :150、記憶檔（desktop-updater-signing 的單一指令）。
- `scripts/docs/`：docs-links.mjs、docs-screenshots.mjs、docs-parity.test.mjs、remote-docs.test.mjs、vocabulary-guard.test.mjs 與測試。docs-screenshots 引用 `./cli.mjs` 改為 `../dev/cli.mjs`（唯一跨組 import；release-notes→release-notes-render 同組不變）。
- `scripts/dev/`：dev.mjs、cli.mjs 與測試。呼叫端：package.json 的 dev／dev:server／dev:desktop／dev:reset／cli 五條。
- 根：install.sh、install.ps1、install.test.mjs——README.md／README.en.md／docs/getting-started.md 共六處 raw GitHub 網址寫死 `scripts/install.sh`，是對外安裝入口，不得移動。
- glob：root package.json 的 test:all 已是 `scripts/**/*.test.mjs`（遞迴，不用改）；ci.yml:69 是 `scripts/*.test.mjs`（非遞迴，要改成 `**`）；openspec/config.yaml 的 tasks 規則字面「node --test scripts/*.test.mjs」與每個 change 最後一個 task 的必跑項同步改為 `scripts/**/*.test.mjs`（規則文字屬 workflow config，隨本刀改）。
- 守門：候選 2 的「無殘留舊路徑」斷言同時涵蓋 `scripts/<檔名>` 舊形（排除 install.* 三檔與歷史記錄）。
**Ruled out**: 六組（安裝腳本自成 `install/`）——公開網址釘死在根，搬了就斷；改 README 網址——已發出去的文件與使用者的 shell 歷史都指舊網址，不可逆；測試集中到 `scripts/tests/`——腳本與測試同名相鄰是既有慣例、也是讀者找測試的方式；delivery-gate.test 放根——它是 release 契約守門，放 release/ 讓「發版跑哪些」一眼可見。
**Open**: 使用者是否同意「五組＋根留 install.*、跨組 import 一處、ci.yml 與 config 規則 glob 改遞迴」？同意後候選 4 收斂，進候選 5（頂層雜項）第一題：prompt.md 去向與 docs/design/ 成員。

### Round 10 — interview (2026-09-10)

**Focus**: 候選 4 收斂（五組＋根留 install.*、跨組 import 一處、ci.yml 與 config 規則 glob 改遞迴、舊路徑斷言涵蓋 scripts）。進候選 5 第一題——prompt.md 去向、docs/design/ 成員。
**Position**: prompt.md 刪除；docs/design/ 只收兩份未對外連結的內部藍圖。查證：
- prompt.md：7 行、無標題、repo 內零引用（只有本討論提到它）；內容是 remote 模式的起源需求陳述（PO／PM／RD 在同一 AI Agent 系統跑 SDD），ed9068a4「plan engine-as-SDK」那次 commit 帶入後未再改。它的內容已被 docs/platform-architecture.zh-TW.md §1 核心結論與 §5／§6 兩種模式完整吸收；git 歷史保留原文。刪除通過刪除測試：沒有讀者需要它、沒有工具引用它。
- docs/ 的 24 檔分三類，用「README 有沒有連結」當客觀判準：(a) 兩份 README 連結的 12 對——getting-started／configuration／workflow／development／sdk-node／remote-getting-started／product-status／roadmap／verb-contract 與 server-backup／server-deployment／server-store-drivers（後三份中文版被兩份 README 直接連結，是營運者文件，不是內部設計）——全部留 docs/ 根；(b) README 未連結、只有中文版、互相引用的兩份——platform-architecture.zh-TW.md 與 implementation-refactor-roadmap.zh-TW.md——移入 docs/design/；(c) assets/ 不動。
- 引用面：openspec/config.yaml:31 的專案說明字面（`docs/platform-architecture.zh-TW.md` 僅是架構藍圖）改路徑；兩份藍圖互相的相對連結不變（同目錄）；docs-links.mjs 對 docs/ 遞迴掃描，斷鏈即紅；docs-parity 的 PAIRS 與 ZH_DOCS 名單不含這兩份，不受影響。
- 本候選不單開，併入候選 2＋4 那一刀順手做（同一批動 config.yaml 與 docs 路徑）。
**Ruled out**: prompt.md 移入 docs/design/——一份 7 行無標題筆記在設計目錄裡仍是「這是什麼」的問題，內容又已被藍圖吸收；把 server-* 三份營運文件也移入 design/——它們是 README 直接連結的使用者文件（部署／備份／驅動選擇），移了要改 README 六處連結且語意錯；docs/ 依語言分目錄（zh-TW/、en/）——docs-parity 與 README 的雙語連結模式建立在同目錄成對，改動面大且與本候選「分用途」的目的無關。
**Open**: 使用者是否同意「刪 prompt.md、docs/design/ 只收兩份藍圖、併入候選 2＋4 的刀」？同意後最後一題：五個候選的切刀與順序（實作仍排在 improve-lifecycle-layer 三刀之後）。

### Round 11 — interview (2026-09-10)

**Focus**: 候選 5 收斂（刪 prompt.md、docs/design/ 兩份藍圖、併入倉庫層那一刀）。最後一題——五個候選的切刀與順序；實作全部排在 improve-lifecycle-layer 三刀封存之後（使用者 round 2 裁定）。
**Position**: 三刀依序、全走 main、`conclude --hold`：
- **刀一（候選 2＋4＋5，倉庫層搬移）**：crates/ 五組、scripts/ 五組＋根留 install.*、docs/design/ 兩份、刪 prompt.md；同批改 workspace members、44 條 path 相依、三個 workflow、compose 兩檔、package.json 三處、release 技能、config.yaml（glob 規則字面＋專案說明路徑）、docs 散文；補 delivery-gate 的「無殘留舊路徑」斷言（涵蓋 crates/speclink-* 與 scripts/<檔名> 兩種舊形，排除 install.*、openspec/changes/archive、openspec/discussions、CHANGELOG）。零行為變化。先做的理由：它決定其他兩刀的檔案路徑（core 與 server 的資料夾在 crates/engine/、crates/host/ 底下），先落地後兩刀的 tasks 才能寫最終路徑；三個候選共用同一批 CI／config／docs 改動與同一條守門斷言，合一刀只驗一次。
- **刀二（候選 1，core 模組分組）**：lifecycle／quality／workspace 三個資料夾＋根、lib.rs 永久 re-export、九個檔的 tests 搬 `<模組>/tests.rs`、skills.rs 的 include_str! 補一層。零行為變化；`cargo test -p speclink-core` 全綠＋四份 golden 不動即回歸網。排第二：strongly recommended 的候選，且此時 improve-lifecycle-layer 三刀已封存、檔案集合定型（review.rs／verify.rs 已刪、keylines.rs 已在）。
- **刀三（候選 3，server 模組分組）**：identity／admin／api／web 四組＋根、lib.rs 永久 re-export、tests/it 鏡射四目錄。零行為變化；`cargo test -p speclink-server --test it` 全綠即回歸網。排最後：收益要等下次動 server 才回本，且與刀二互不相依、機制相同（先做刀二把 re-export＋tests 搬家的做法走順）。
- 不用 worktree 平行：刀一動的是目錄本身，任何與它平行的 change 都會在目錄搬移上撞。
- 記錄存活：`conclude --hold`；刀一在 improve-lifecycle-layer 三刀封存後才轉出（promote 清 hold），刀一封存前再 `conclude --hold` 一次留給刀二，刀二封存前再一次留給刀三。
**Ruled out**: 五候選五刀——候選 5 只有三個檔案動作、候選 4 與 2 共用整批 CI 改動，單開是空刀；一刀全做——目錄搬移＋兩個 crate 的模組分組＋九個檔測試搬家，review 面是三種不同機制混在一起；刀二先於刀一——刀二的 tasks 得寫 crates/speclink-core/… 舊路徑，刀一落地後 drift；刀二與刀三合一——各自 30／21 個 src 檔＋49 個測試檔＋9 個測試搬家，合併後 review 一次要盯 100 個以上的檔案搬移；在 improve-lifecycle-layer 三刀之前先做刀一——目錄搬移與在途 change 的檔案編輯必撞（使用者 round 2 裁定順序）。
**Open**: 使用者是否同意「三刀依序（2＋4＋5 → 1 → 3）、全走 main、`conclude --hold`、全部排在 improve-lifecycle-layer 三刀之後」？同意即寫結論。

## Conclusion

**Decision**: 五個候選全部落地，分三刀依序、全走 main，且全部排在 improve-lifecycle-layer 三刀封存之後才開始立案（round 2 裁定）。刀一（候選 2＋4＋5，倉庫層搬移，零行為變化）：crates/ 依 docs/platform-architecture.zh-TW.md §4 分五組——crates/engine/（speclink-core、speclink-fs）、crates/host/（speclink-host、speclink-server）、crates/store/（speclink-store 與 store-fs／store-sqlite／store-postgres）、crates/protocol/（speclink-protocol、speclink-remote）、crates/adapters/（speclink-cli、speclink-node），crate 名、套件名、npm 發佈路徑不變，speclink-fs 撞名靠位置解、改名留 Deferred；scripts/ 分五組——release/（release-notes 三支、release-latest-json、signing-gate、homebrew-formula、delivery-gate.test）、npm/（npm-engine-package、npm-server-package、npm-server-launcher.test）、desktop/（desktop-install、desktop-sidecar）、docs/（docs-links、docs-screenshots、docs-parity.test、remote-docs.test、vocabulary-guard.test）、dev/（dev、cli），.test.mjs 跟腳本走，install.sh／install.ps1／install.test.mjs 留根（README 兩份與 getting-started 共六處 raw GitHub 網址寫死），docs-screenshots 的 `./cli.mjs` 改 `../dev/cli.mjs`；docs/design/ 收 platform-architecture.zh-TW.md 與 implementation-refactor-roadmap.zh-TW.md 兩份（README 未連結、只有中文、互相引用），其餘 12 對 README 連結文件留 docs/ 根；刪 prompt.md（7 行、零引用、內容已被藍圖吸收）。同批改動：workspace members、44 條 `path = "../…"`、ci.yml／node-sdk.yml／release.yml、deploy/docker-compose*.yml 的 dockerfile 路徑、root 與 desktop 與 server-npm 三份 package.json、.claude/skills/release/SKILL.md、openspec/config.yaml（tasks 規則字面 `node --test scripts/*.test.mjs` 改 `scripts/**/*.test.mjs`；專案說明的藍圖路徑）、ci.yml:69 的 glob 改遞迴、docs 約 40 處散文、scripts 兩處與 script 測試六行；補 scripts/delivery-gate.test 一條斷言——repo 內不得殘留 `crates/speclink-` 與 `scripts/<舊檔名>` 舊形（排除 install.* 三檔、openspec/changes/archive、openspec/discussions、CHANGELOG）。驗證：ci.yml 為主（cargo --workspace --locked 驗 path 相依、docker smoke 用同一 Dockerfile 路徑覆蓋 release.yml 唯一實質改動、`node --test scripts/**/*.test.mjs`），node-sdk.yml 於合併 commit 驗，不演練 tag。刀二（候選 1，core 模組分組，零行為變化）：crates/speclink-core/src 分 lifecycle/（model、newcmd、inprogress、discard、archive、tasks、status、listing、capname、preflight、discuss、trace）、quality/（station、validate、analyzer、drift）、workspace/（init、skills、instructions、config、schema、workspace）、command/ 維持，根留 lib、store、util、keylines、testkit、teststore、demo；資料夾模組私有、lib.rs 一組 `pub use` 永久維持 `speclink_core::<模組>` 全部既有路徑（外部約 500 處、crate 內 `crate::` 亦不改）；inline tests 達 500 行的九個模組（discuss、command/mod、init、archive、config、tasks、analyzer、validate、drift）搬成 `<模組>/tests.rs` 並以 `#[cfg(test)] mod tests;` 宣告；skills.rs 的 include_str! 相對路徑補一層。刀三（候選 3，server 模組分組，零行為變化）：crates/speclink-server/src 分 identity/（identity、identity_sqlite→sqlite.rs、auth、device、setup）、admin/（admin、audit、backup）、api/（routes、read_api、verb、events、context、error）、web/（web、assets），根留 main、lib、app、config、state；lib.rs 永久 re-export（外部 34 處、tests 114 處不改）；tests/it 的 49 檔鏡射分 identity／admin／api／web 四目錄、檔名去前綴、各目錄 mod.rs 列子模組、main.rs 只列四組＋common 與跨面檔；tests/pg 不動。
**Rationale**: 頂層分區（apps／crates／packages／scripts／docs／openspec）本身清楚，看不懂的是分區之下全部平鋪：core 30 個模組三群交錯、12 個 crate 讓文件的五層分層只活在文件裡且 speclink-fs 與 speclink-store-fs 撞名、server 21 個模組靠檔名前綴編碼分組、scripts 36 檔六種用途混排。判準是「第一次打開的人能否不靠 grep 預測功能住哪、哪些檔是一組」，前例 cli-verb-family-modules 以同一判準確立「重切是集中複雜度而非搬移」。三刀順序由路徑相依決定：刀一改目錄本身，決定後兩刀 tasks 的最終路徑；候選 2／4／5 共用同一批 CI／config／docs 改動與同一條守門斷言，合一刀只驗一次；候選 1 與 3 機制相同（私有資料夾模組＋永久 re-export＋tests 搬家）、互不相依，分兩刀讓每刀 review 面不超過一個 crate。永久 re-export 是全案的核心取捨：資料夾對讀者可見、對呼叫者不可見，內部分組不升格為外部契約。全部排在 improve-lifecycle-layer 之後：候選 1 的檔案集合要等那三刀定型（review.rs／verify.rs 刪、keylines.rs 加），且目錄搬移與在途 change 的檔案編輯必撞。介面深度四項在每個候選都以「讀者能否把一組當整體理解」為深度判準（Round 3–11）。
**Rejected alternatives**: 核心：五組以上（discuss 單獨成組）——一檔一組反樣式；依動詞一資料夾——cli-verb-family-modules 已否決；只搬測試不分組——主問題沒動；消費端遷移到新路徑——500 處零收益改動且內部分組變外部契約；新舊路徑並存——讀者要問哪條正典；`#[path]` 保留平鋪名——社群公認反樣式；全部模組一律搬測試——小檔多出一屏內子檔；測試搬到 tests/ 整合目錄——失去私有存取、增加 test binary；門檻用「測試比程式碼長」——drift.rs 會漏。crates：四組（protocol 併 engine 或 adapters）——protocol 被四方共用；server 歸 adapters——與文件「server 是 Host 的 adapter」字面衝突；改 speclink-fs 名字——位置差已足夠，Deferred；apps/desktop/core 搬進 crates/——tauri 殼與 core 分家；丟棄 tag 演練 release.yml——實質步驟已由 ci.yml 同指令覆蓋，演練會產出真實 release 資產；加 workflow_dispatch——為一次搬移改發版觸發面；只改路徑不加守門斷言——docs 40 處靠人眼。server：三組（web 併 api）——不同對外面、規格分開命名；audit 歸 identity——它是管理面稽核；只分 src 不分 tests/it——測試檔更多；改其餘檔名——沒有前綴編碼不動。scripts：六組（install/ 自成一組）——公開網址釘死在根；改 README 網址——已發出的文件不可逆；測試集中 scripts/tests/——同名相鄰是既有慣例；delivery-gate.test 放根——它是發版契約守門。頂層：prompt.md 移入 docs/design/——無標題筆記在哪都是「這是什麼」；server-* 營運文件移入 design/——README 直接連結的使用者文件；docs 依語言分目錄——與分用途無關、改動面大。切刀：五刀——候選 5 三個檔案動作、候選 4 與 2 共用 CI 改動；一刀全做——三種機制混一個 review 面；刀二先於刀一——tasks 寫舊路徑必 drift；刀二與刀三合一——百檔以上搬移一次 review；先於 improve-lifecycle-layer 做——目錄搬移撞在途編輯；worktree 平行——刀一動目錄本身必撞。
**Deferred**: speclink-fs 改名（如 speclink-engine-fs）——刀一落地後若位置差仍不夠分辨再議；station.rs 在 improve-lifecycle-layer 刀三吸收門面測試後是否過 500 行門檻——刀二 propose 期量測；memory 檔內引用的舊路徑（desktop-updater-signing 的 scripts/desktop-install.mjs、stale-installed-cli 等）——刀一封存後由使用者側更新，不在 repo 範圍；apps/desktop/src 根目錄 11 個鬆散檔（store／tabs／tray／session／recents）與 packages/ui/src 根 15 檔的分組——本次未掃描前端，下次 improve 指定前端方向時處理。
**Capture to**: proposal
**Next**: 等 improve-lifecycle-layer 三刀封存後，/speclink-propose --from-discussion improve-repo-layout（先立刀一 2＋4＋5；刀一封存前 conclude --hold 留給刀二，刀二封存前再留給刀三）
