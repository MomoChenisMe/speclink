## Context

`crates/engine/speclink-core/src` 現況 30 個平鋪模組＋`command/`（mod.rs、typed.rs）。`lib.rs` 逐一 `pub mod`；`testkit` 以 feature 與 `#[cfg(any(test, feature = "testkit"))]` 門控、`teststore` 為 `#[cfg(test)] pub(crate)`。crate 外對 `speclink_core::<模組>` 的引用約 500 處（util 29、tasks 28、store 27、init 27、model 26、config 23、workspace 22、schema 19、analyzer 10、command 8、listing 7、discuss 5…）；crate 內 `crate::<模組>` 544 處（model 161、tasks 51、station 44、util 36、config 36、store 30…）。`include_str!` 相對路徑在 demo.rs（根，不動）、skills.rs、schema.rs（`../assets/…`）。`tests/it` 無 `#[path]`、不讀 src 檔案。

crate 外引用 core 內部檔案路徑的位置：scripts/desktop/desktop-install.mjs（:24 訊息、:104 讀 init.rs 取 ASSET_VERSION）與 desktop-install.test.mjs:26；packages/ui/src/__tests__/discussionDrawer.test.tsx（:143、:158 讀 discuss.rs 做同構斷言）；packages/ui/src/tasks.ts:31 與 packages/ui/src/__tests__/taskList.test.tsx:50 註解（tasks.rs）；crates/engine/speclink-core/tests/it/render_golden.rs:828 訊息（init.rs）；docs/product-status.md:44／:56 與 zh-TW :45／:57（station.rs、tasks.rs）；openspec/config.yaml:56 指 util.rs（留根，不動）。正典規格散文只有 delivery-baseline 的兩個反例行（帶 `path-guard:allow`），@trace 區塊的歷史路徑不動。

刀一落地的 delivery-baseline「倉庫路徑守門」：靜態斷言排除 .git／node_modules／target／dist／.speclink／.dev／openspec/changes／openspec/discussions／CHANGELOG.md，跳過 @trace 區塊與 `path-guard:allow` 標記行，檢查 (a) 舊形 crate 路徑、(b) 舊形 scripts 路徑。

## Goals / Non-Goals

**Goals:**

- 第一次打開 core 的人看三個資料夾名就知道引擎分成哪幾件事；大檔一開就是程式碼。
- crate 內外全部既有路徑一字不改；行為、簽名、錯誤字串零變化。
- 守門延伸到 core 模組路徑，殘留舊形在 `node --test` 就紅。

**Non-Goals:**

- 模組本體的合併、拆分、改名；消費端路徑遷移；golden、assets、`ASSET_VERSION`；server 模組（刀三）。
- `tests/it` 只改兩條架構守門的掃描面（見 D3 連帶）與 render_golden 的提示字串，不動任何測試意圖。

## Decisions

### D1 三組成員與根

- `lifecycle/`：model、newcmd、inprogress、discard、archive、tasks、status、listing、capname、preflight、discuss、trace。依據：archive→discuss／model／station／tasks／validate 等 13 個模組中 9 個在本組與 quality；trace→archive／capname／discuss／model／station；capname 只被 newcmd／validate／trace 用（開名守門）；preflight 只被 drift／instructions 用（apply 前檢查變更狀態）。
- `quality/`：station、validate、analyzer、drift（對變更做檢查、產出 findings 或裁決）。
- `workspace/`：init、skills、instructions、config、schema、workspace（init→config／skills／util；skills→config／init；schema→util／workspace；config→skills／util；instructions 例外引用 model／preflight／status／tasks 以組 apply 指令）。
- `command/` 維持；根：lib、store（儲存接縫）、util、keylines（文件原語，lifecycle 與 change meta 共用）、testkit、teststore、demo。
- 命名對齊：lifecycle＝improve-lifecycle-layer 與 product-status 用語；quality＝LANGUAGE.md「品質關卡」；workspace＝workspace-tools／workspace-session 規格名。

### D2 lib.rs 的永久 re-export

```rust
mod lifecycle;
mod quality;
#[path = "workspace/mod.rs"]
mod workspace_group;
pub mod command;
pub mod keylines;
pub mod store;
pub mod util;
pub mod demo;
pub use lifecycle::{archive, capname, discard, discuss, inprogress, listing, model, newcmd, preflight, status, tasks, trace};
pub use quality::{analyzer, drift, station, validate};
pub use workspace_group::{config, init, instructions, schema, skills, workspace};
```

三個資料夾的 `mod.rs` 只含 `pub mod <子模組>;` 各一行（`pub` 是為了讓 `lib.rs` 能 re-export，資料夾本身私有，外部拿不到 `speclink_core::lifecycle` 路徑）。`crate::archive::…` 與 `speclink_core::archive::…` 皆經 re-export 名字解析，零改動。`testkit`／`teststore` 門控行原樣保留於根。名稱衝突：`workspace/` 資料夾內有同名的 `workspace.rs` 子模組。`mod.rs` 內 `pub mod workspace;` 合法（子模組名與父模組同名不衝突），但 `lib.rs` 不能同時有 `mod workspace;`（資料夾）與 `pub use …::workspace;`（子模組）——同一層兩個同名模組是 E0255。故資料夾模組取私有別名 `workspace_group`，以單一 `#[path = "workspace/mod.rs"]` 指回資料夾；`lifecycle` 與 `quality` 兩組無此衝突，不帶屬性。re-export 後 `speclink_core::workspace` 指向子模組，與現況一致；`workspace_group` 私有，外部拿不到。

### D3 測試搬子檔（門檻 500 行）

十個模組：lifecycle/discuss、lifecycle/archive、lifecycle/tasks、quality/station、quality/analyzer、quality/validate、quality/drift、workspace/init、workspace/config、command。做法：把 `#[cfg(test)] mod tests { … }` 的大括號內容逐字搬到 `<模組>/tests.rs`（command 為 `command/tests.rs`），原處改為 `#[cfg(test)] mod tests;`；`use super::*` 與其他 `use` 逐字保留。測試路徑 `speclink_core::lifecycle::archive::tests::x` 的過濾字串（`cargo test -p speclink-core archive`）維持可用。drift.rs 第 11 行的 `#[cfg(test)] use` 是測試專用 import，留在模組檔；其 `mod tests` 在第 999 行。

連帶：`tests/it` 的兩條架構守門靠「切在第一個 `#[cfg(test)]` 之前」判定正式碼，`tests.rs` 子檔裡沒有這個標記，會被整檔當成正式碼掃——兩條都要多跳一個檔名 `tests.rs`。其中 `no_direct_fs` 另有一個只掃 `src/` 頂層的 `read_dir`，搬完會讓 22 個模組整批掉出掃描面（`init.rs`／`schema.rs` 的白名單過期檢查也跟著失效），改為遞迴走訪；遞迴後的命中集合恰是白名單那四個檔（util、init、schema、testkit），零新增違規。

### D4 相對路徑與 crate 外引用

`workspace/skills.rs` 與 `workspace/schema.rs` 的 `include_str!("../assets/…")` 改 `"../../assets/…"`；demo.rs 留根不動。crate 外七處（Context 列出）改為新路徑：init.rs→`workspace/init.rs`、discuss.rs→`lifecycle/discuss.rs`、tasks.rs→`lifecycle/tasks.rs`、station.rs→`quality/station.rs`。

### D5 守門延伸

`scripts/release/delivery-gate.test.mjs` 的路徑守門增 (c)：對 22 個已搬模組名，字串 `crates/engine/speclink-core/src/<模組>.rs` 為舊形（根層七個名字與 `command/` 不在清單）；排除、@trace 跳過與 `path-guard:allow` 豁免規則沿用。對應 spec delivery-baseline MODIFIED「倉庫路徑守門」。搬移前 RED（至少命中 scripts/desktop/desktop-install.mjs 與 docs/product-status），搬完 GREEN。

### D6 驗證

`cargo build -p speclink-core`＋`cargo test -p speclink-core`（全 crate：單元＋`--test it`，含 render_golden 逐位元綠）；`cargo build --workspace --locked`（四個消費 crate 零改動仍編譯）；`cargo test -p speclink-desktop-core`；`npm test -w packages/ui`（discussionDrawer 與 taskList 測試讀新路徑）；`node --test scripts/*.test.mjs scripts/*/*.test.mjs`（守門與 desktop-install.test）；`git status` 顯示 22 個模組為 rename、十個 tests.rs 為新檔且模組檔的 diff 只有尾端一行。

## Implementation Contract

**行為**：`speclink` 全部指令的人眼輸出、`--json`、render golden 逐位元不變；`cargo test -p speclink-core` 的測試數量與改動前相同（搬家不增減）；四個消費 crate 與 desktop 不改一行仍編譯與通過測試。

**介面／資料形狀**：`speclink_core` 公開模組路徑集合不變（22 個 re-export 名＋根層 pub mod）；新增的 `lifecycle`／`quality`／`workspace` 為私有，不出現在公開 API。

**失敗模式**：漏 re-export 一個名字 → 消費 crate 編譯錯誤指名路徑；include_str 路徑錯 → core 編譯錯誤；殘留舊形路徑 → 守門斷言紅並列檔案行號；測試搬家漏 `use` → 該 tests.rs 編譯錯誤。

**驗收**：D6 全綠；`git diff --stat` 不含 tests/golden、assets、Cargo.lock；`grep -rn 'speclink_core::lifecycle\|speclink_core::quality\|speclink_core::workspace_group' crates apps` 零命中（資料夾路徑未外洩；`speclink_core::workspace::` 不入樣式——它是 `workspace.rs` 子模組的既有正當路徑，約 22 處）。

**範圍**：In scope＝提案 Impact 全部檔案。Out of scope＝模組行為、消費端、tests/it、golden、assets、server 模組。

## Risks / Trade-offs

- **回歸對照**：render golden 與 CLI 凍結測試不受影響；`cargo test -p speclink-core` 測試數量前後相等是搬家完整的判準。
- **跨平台**：純檔案搬移；Windows 的 git 大小寫無影響（全小寫）。
- **rustdoc 雙路徑**：`cargo doc` 對 re-export 名字顯示於根，資料夾私有不出現；無外部文件連結需改。
- **git blame**：`--follow` 跨目錄仍可追；接受（cli-verb-family-modules 同一取捨）。
- **與刀三的順序**：刀三動 `crates/host/speclink-server`，與本案零檔案重疊，可緊接立案。
