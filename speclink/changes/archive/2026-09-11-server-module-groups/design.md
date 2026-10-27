## Context

`crates/host/speclink-server/src` 現況 21 個平鋪模組，`lib.rs` 逐一 `pub mod`，另含 `build_store`（lib.rs 內的函式，外部 5 處引用）。crate 外對 `speclink_server::<模組>` 的引用 34 處（identity 8、state 6、events 6、config 5、app 5、build_store 2、audit 2；來源為 apps/desktop/src-tauri、crates/protocol/speclink-remote 的測試與 main.rs），tests/it 內 114 處（identity 38、state 35、backup 14、audit 12、config 6、events 4、build_store 3、setup 1、app 1）；crate 內 `crate::<模組>` 71 處。`identity_sqlite` 只在 crate 內被引用（lib.rs 宣告、identity.rs 的 `pub use crate::identity_sqlite::IdentitySqlite`、backup.rs 一處 import、config.rs 一個測試函式名）。唯一相對路徑內嵌：assets.rs 的 `include_bytes!("../../../../apps/server-web/dist/index.html")`（release build 的 fail-closed 內嵌）。

`tests/it`：48 個測試檔＋`main.rs`＋`common/`（mod.rs、subscriber.rs），`main.rs` 平鋪宣告 48 個測試 mod 與 common；測試檔以 `use crate::common` 取鷹架，測試檔之間零互引。ci.yml 以 `-p speclink-server --test postgres_store` 點名的是 tests/pg 目標，不受影響。crate 外引用 server 內部檔案路徑：apps/server-web/src/pages/admin/AuditPage.tsx:15 註解（audit.rs）、docs/product-status 兩份連結 src/main.rs 與五個測試檔、scripts/docs/remote-docs.test.mjs 讀 src/app.rs 與 src/web.rs。正典規格散文零引用；@trace 區塊的歷史路徑不動。

刀二落地的 delivery-baseline「倉庫路徑守門」含 (a) crate 舊形、(b) scripts 舊形、(c) core 模組舊形三條與六個 scenario；`path-guard:allow` 豁免、@trace 跳過、排除目錄清單皆為現行正典。

## Goals / Non-Goals

**Goals:**

- 打開 server crate 看四個資料夾名就知道四個面；`identity_sqlite` 這種前綴編碼消失。
- crate 內外全部既有路徑一字不改（唯一例外：無人使用的 `identity_sqlite` 根層名）。
- tests/it 鏡射 src 分組，測試過濾字串繼續可用，測試數量前後相等。
- 守門延伸到 server 模組與測試檔路徑。

**Non-Goals:**

- 模組行為、路由、錯誤字串；inline 測試搬家（未達門檻）；tests/pg；Dockerfile；其他 crate。

## Decisions

### D1 四組成員與根

- `identity/`：sqlite（原 identity_sqlite）、auth、device、setup；主模組 identity.rs。依據：auth→identity／config／error／state，device→identity／error／state，setup→identity／audit／state／web，identity↔sqlite 互引；共同核心是「誰是誰、能不能進來」。
- `admin/`：audit、backup；主模組 admin.rs。admin→audit／auth／backup／identity／web；backup→identity／sqlite／config；audit 的 `//!` 首行即「管理面稽核日誌」。audit 被 identity／setup 引用屬向下引用，可接受。
- `api/`：routes、read_api、verb、events、context、error；宣告檔 api.rs（無同名主模組）。六檔只向 auth／state／verb／error／events 引用，是同一條 HTTP 對外面；error 是 wire 錯誤信封的單一對映點。
- `web/`：assets；主模組 web.rs（assets→web；web→identity／setup／state）。
- 根：main、lib、app（router 組裝，引用全部四組）、config、state。

### D2 主模組檔＋同名資料夾（Rust 2018 形式），而非資料夾私有＋re-export

`src/identity.rs` 尾端加 `pub mod auth; pub mod device; pub mod setup; pub mod sqlite;`，檔案放 `src/identity/`；`admin.rs` 加 `pub mod audit; pub mod backup;`；`web.rs` 加 `pub mod assets;`；新增 `src/api.rs` 只含六行 `pub mod`。`lib.rs`：

```rust
pub mod admin;
pub mod app;
mod api;
pub mod config;
pub mod identity;
pub mod state;
pub mod web;
pub use admin::{audit, backup};
pub use api::{context, error, events, read_api, routes, verb};
pub use identity::{auth, device, setup};
pub use web::assets;
```

理由：identity／admin／web 三組各有同名主模組，若資料夾私有再 `pub use identity::identity`，根層會出現兩個 `identity`（E0255），刀二為此把 workspace 資料夾改名 `workspace_group`——在 server 這會變成 `identity_group`，正是本案要消除的名字編碼。主模組檔宣告子模組是 2018 edition 的慣用形，`speclink_server::identity` 天然不變，子模組以根層 `pub use` 掛回舊名，crate 內 `crate::auth` 等 71 處與 crate 外 34 處、tests 114 處零改動。`api` 無同名主模組，資料夾私有＋re-export 與刀二同形。

### D3 `identity_sqlite` 收進 `identity::sqlite`

`identity.rs` 的 `pub use crate::identity_sqlite::IdentitySqlite` 改 `pub use sqlite::IdentitySqlite`；`backup.rs` 的 `use crate::identity_sqlite::IdentitySqlite` 改 `use crate::identity::IdentitySqlite`（identity.rs 已 re-export 該型別，main.rs 現在就是這樣用）；根層不保留 `identity_sqlite` 別名（零 crate 外引用，留著是死名字）；config.rs 的測試函式名 `identity_sqlite_without_a_path_fails_closed` 是測試案例名，不動。

### D4 tests/it 鏡射四組

目錄與檔名：
- `identity/`：auth_device、auth_pat、auth_whoami、binding、device_e2e、device_flow、identity、invite、refresh_rotation（檔名不變；`identity/identity.rs` 合法）。
- `admin/`：api（原 admin_api）、audit_filter、data、e2e、overview_view、system、system_view、three_entry、users_view、web_api（去 `admin_` 前綴）、audit、backup_e2e、backup_restore、cli_admin。
- `api/`：command_routes、query_routes、discussion_routes、read_api、review_api、verify_api、verb_api、drift_api、context_api、import_api、policy_write、board_order、sse_events、sync_state、health。
- `web/`：account、activate、assets、invite、session、setup（去 `web_` 前綴）。
- 根：common/、e2e_cli、phase2_chain、startup、serverfs_store（跨面劇本與啟動）。

只去掉與目錄名相同的前綴，其他檔名不動，避免 `auth_device` 與 `device_flow` 之類撞名。各目錄 `mod.rs` 逐行 `mod <檔>;`，`main.rs` 改為 `mod common; mod identity; mod admin; mod api; mod web; mod e2e_cli; mod phase2_chain; mod startup; mod serverfs_store;`。測試路徑變為 `admin::api::…`，`cargo test -p speclink-server --test it admin` 仍匹配整組；`--test it web_setup` 這類舊過濾字串改為 `web::setup`。tests/pg 不動。

### D5 相對路徑與 crate 外引用

`web/assets.rs` 的 `include_bytes!` 改 `"../../../../../apps/server-web/dist/index.html"`，同檔第 24 行的說明「路徑相對於本源碼檔」同步指 `src/web/`。AuditPage.tsx 註解改 `crates/host/speclink-server/src/admin/audit.rs`；docs/product-status 兩份的三個測試檔連結改新路徑；remote-docs.test.mjs 讀的 app.rs／web.rs 留根不動。

### D6 守門延伸

`scripts/release/delivery-gate.test.mjs` 的路徑守門增 (d)：13 個已搬 src 模組名（sqlite 以舊名 identity_sqlite 列入）與 44 個已搬測試檔名（含去前綴前的舊名）——字串 `crates/host/speclink-server/src/<模組>.rs` 與 `crates/host/speclink-server/tests/it/<檔名>.rs` 為舊形；根層 src 五檔＋identity／admin／web／api 四個主模組檔，與 tests/it 根層四檔＋common 不在清單。排除、@trace 跳過與 `path-guard:allow` 豁免沿用。搬移前 RED（命中 AuditPage.tsx 與 docs/product-status），搬完 GREEN。對應 spec delivery-baseline MODIFIED「倉庫路徑守門」。

### D7 驗證

`cargo build -p speclink-server`；`cargo test -p speclink-server`（含 `--test it` 48 檔）測試數量前後相等；`cargo build --workspace --locked`（desktop-tauri 與 remote 零改動仍編譯）；`cargo test -p speclink-remote --test it`（remote 測試以 `speclink_server::…` 起 server）；`cargo build --release -p speclink-server` 需先有 `apps/server-web/dist`（`npm run build -w apps/server-web`）以驗 `include_bytes!` 新相對路徑（release build 才編譯該常數）；`node --test scripts/*.test.mjs scripts/*/*.test.mjs`；`npm test -w apps/server-web`（AuditPage 測試面）；`git status` 顯示 13 個模組與 44 個測試檔為 rename。

## Implementation Contract

**行為**：server 的 HTTP 端點、SSE、管理面 HTML、CLI 子指令（main.rs）、人眼輸出與 `--json` 逐位元不變；desktop-tauri 內嵌 server 與 remote 測試不改一行仍通過；`cargo test -p speclink-server --test it` 測試數量與改動前相同。

**介面／資料形狀**：`speclink_server` 公開路徑集合＝改動前減去 `identity_sqlite`；新增 `speclink_server::identity::sqlite`（`IdentitySqlite` 仍可經 `identity::IdentitySqlite` 取得，與現況一致）。目錄形狀如 D1／D4。

**失敗模式**：漏 re-export → 消費 crate 或 tests/it 編譯錯誤指名路徑；include_bytes 路徑錯 → release build 編譯錯誤（debug build 不驗，故 D7 明列 release build）；殘留舊形路徑 → 守門斷言紅並列檔案行號；product-status 舊連結 → docs-links 測試紅。

**驗收**：D7 全綠；`grep -rn 'identity_sqlite' crates apps` 只剩 config.rs 的測試函式名；`git diff --stat` 不含 Cargo.lock、tests/pg、Dockerfile。

**範圍**：In scope＝提案 Impact 全部檔案。Out of scope＝模組行為、tests/pg、Dockerfile、其他 crate、inline 測試搬家。

## Risks / Trade-offs

- **回歸對照**：server 整合測試 48 檔與 remote 測試是回歸網；測試數量相等是搬家完整的判準；render golden 不涉及。
- **release build 的 include_bytes**：debug 路徑不編譯該常數，漏改只在 release 才炸——D7 明列 release build 驗證，ci.yml 的 docker smoke（release build）亦覆蓋。
- **跨平台**：純檔案搬移，全小寫檔名。
- **測試過濾字串**：去前綴後 `--test it web_setup` 改 `web::setup`；記憶檔與個人習慣需更新，repo 內無此類字串（ci.yml 只點名 postgres_store）。
- **git blame**：`--follow` 仍可追；接受。
