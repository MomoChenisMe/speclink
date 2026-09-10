## Summary

`crates/host/speclink-server/src` 的 21 個平鋪模組依「身分與存取／管理面／API 面／網頁面」分成四組，`tests/it` 的 49 個平鋪測試檔鏡射分成同四組子目錄；以 Rust 2018 的「主模組檔＋同名資料夾」形式讓 `speclink_server::<模組>` 既有路徑一字不改，`identity_sqlite.rs` 的檔名前綴改由目錄承擔；零行為變化。

## Motivation

目標使用者是第一次打開 server crate 的人（新加入的開發者、透過 AI 代理讀碼的 SDD 使用者），使用情境是「找登入與權限在哪、管理面在哪、HTTP 面在哪」。來源討論 improve-repo-layout（Round 1、8）的查證：

- 最大的檔案 `identity_sqlite.rs`（2232 行）靠檔名前綴告訴讀者它與 `identity.rs`（746 行）是一對；`auth.rs`／`device.rs`／`setup.rs` 也都是身分與存取，卻與 `backup.rs`／`assets.rs`／`audit.rs` 混排在字母序裡；`routes.rs`（2075 行）、`read_api.rs`、`verb.rs`、`events.rs`、`context.rs`、`error.rs` 六檔是同一條 HTTP 對外面。相依圖證實：auth→identity／config／error／state，device→identity／error／state，setup→identity／audit／state／web，admin→audit／auth／backup／identity／web，routes／read_api／verb／context 只向 auth／state／verb／error／events 引用。
- `tests/it` 49 個測試檔同樣平鋪，靠前綴分組（admin_* 十檔、web_* 六檔、auth_*／device_*／backup_* 等），測試檔數比 src 更多，是更需要分組的那一半。
- 這是 improve-repo-layout 三刀的最後一刀。前提已成立：刀一（repo-layout-groups）與刀二（core-module-groups）已封存，crate 位於 `crates/host/speclink-server`，正典 delivery-baseline 的「倉庫路徑守門」已含 (a)(b)(c) 三條。

相關規格掃描：`delivery-baseline`（本案修改對象——守門要延伸到 server 模組與測試檔路徑）；`reference-server`、`server-admin`、`server-identity`、`server-web-console` 規範的是 server 的可觀察行為，不涵蓋原始碼佈局，本案零行為變化不動它們。

## Proposed Solution

影響的 crate：speclink-server（模組與測試檔搬移、`lib.rs`、一個 `include_bytes!` 相對路徑）；crate 外只有三處引用 server 內部檔案路徑（apps/server-web 的一行註解、docs/product-status 兩份的測試檔連結）與守門測試。不新增或變更 CLI 指令、HTTP 端點、旗標、exit code；不涉及設定欄位。

1. **src 四組（主模組檔＋同名資料夾）**：`identity.rs`＋`identity/`（sqlite.rs＝原 identity_sqlite.rs、auth.rs、device.rs、setup.rs）；`admin.rs`＋`admin/`（audit.rs、backup.rs）；`web.rs`＋`web/`（assets.rs）；`api.rs`（只宣告子模組的私有資料夾模組）＋`api/`（routes.rs、read_api.rs、verb.rs、events.rs、context.rs、error.rs）。根留 main.rs、lib.rs、app.rs、config.rs、state.rs。選這個形式而非「資料夾私有＋re-export」的理由：identity／admin／web 三組都有同名主模組，私有資料夾與 re-export 的名字會在 crate 根撞名（刀二為此把 workspace 資料夾改名 `workspace_group`）；主模組檔宣告子模組是 Rust 2018 的慣用形，`speclink_server::identity`／`admin`／`web` 路徑天然不變。
2. **既有路徑一字不改**：`lib.rs` 對搬入子目錄的 13 個模組以 `pub use identity::{auth, device, setup}; pub use admin::{audit, backup}; pub use api::{routes, read_api, verb, events, context, error}; pub use web::assets;` 掛回根——crate 外 34 處 `speclink_server::<模組>`（desktop-tauri、remote 測試、main.rs）與 tests/it 114 處引用不改，crate 內 71 處 `crate::<模組>` 亦不改。唯一消失的根層名字是 `identity_sqlite`：它沒有 crate 外引用，`identity.rs` 已 `pub use` 其 `IdentitySqlite`，`backup.rs` 的一處 `crate::identity_sqlite::IdentitySqlite` 改走 `crate::identity::IdentitySqlite`。
3. **相對路徑**：`web/assets.rs` 的 `include_bytes!("../../../../apps/server-web/dist/index.html")` 多一層 `../`。
4. **tests/it 鏡射四組**：`identity/`（auth_device、auth_pat、auth_whoami、binding、device_e2e、device_flow、identity、invite、refresh_rotation）、`admin/`（admin_* 十檔去掉 `admin_` 前綴、audit、backup_e2e、backup_restore、cli_admin）、`api/`（command_routes、query_routes、discussion_routes、read_api、review_api、verify_api、verb_api、drift_api、context_api、import_api、policy_write、board_order、sse_events、sync_state、health）、`web/`（web_* 六檔去掉 `web_` 前綴）；跨面的 e2e_cli、phase2_chain、startup、serverfs_store 與 `common/` 留根。各子目錄 `mod.rs` 列子模組，`main.rs` 只列四組＋common＋四個跨面檔；測試內 `use crate::common` 不變；`cargo test -p speclink-server --test it admin` 這類過濾字串因模組路徑含目錄名而繼續可用。`tests/pg` 不動。
5. **crate 外引用同批改**：apps/server-web/src/pages/admin/AuditPage.tsx 的註解指 `admin/audit.rs`；docs/product-status 兩份連到的五個測試檔中三個改新路徑（admin_e2e→admin/e2e.rs、backup_e2e→admin/backup_e2e.rs、device_e2e→identity/device_e2e.rs；e2e_cli 與 phase2_chain 留根不變）。scripts/docs/remote-docs.test.mjs 讀的 app.rs 與 web.rs 都留根，不受影響。
6. **守門延伸**：delivery-baseline「倉庫路徑守門」增 (d) 條——已搬入子目錄的 13 個 server 模組與 45 個測試檔不得再以舊形 `crates/host/speclink-server/src/<模組>.rs`／`tests/it/<檔名>.rs` 引用。

**相容性影響**：HTTP 端點、人眼輸出、`--json`、SSE、管理面 HTML 全部不變；server 整合測試 53 檔即回歸網，測試數量前後相等；`speclink_server` 公開路徑集合只少一個無人使用的 `identity_sqlite`。

## Non-Goals

- 不改任何模組的行為、簽名、路由、錯誤字串；不合併或拆分模組本體。
- 不動 `tests/pg`（PostgreSQL 專用鷹架，與 driver crate 刻意不共用）、`Dockerfile`、`config.rs`／`state.rs`／`app.rs`。
- 不搬 inline 測試（server 各模組的 inline 測試最多 407 行，未達刀二的 500 行門檻）。
- 不動 core（刀二已完成）與其他 crate。

## Alternatives Considered

- 資料夾私有＋re-export（刀二形式）——identity／admin／web 撞名，得替資料夾取 `identity_group` 這種名字，正是本案要消除的「用名字編碼分組」。
- 三組（web 併進 api）——web 是伺服器渲染 HTML 表單的最小網頁面，與 JSON API 是不同對外面，規格（server-web-console）也分開命名。
- audit 歸 identity——它是管理面稽核，identity 只是寫入者之一。
- 只分 src 不分 tests/it——測試檔數（49）比 src（21）更多。
- 保留 `identity_sqlite` 根層別名——零使用者，留著是死名字。

## Impact

- Affected specs：delivery-baseline（MODIFIED「倉庫路徑守門」：增 (d) server 模組與測試檔舊形路徑條款與一條 scenario，既有六條 scenario 逐字保留）。
- Affected code:
  - New: crates/host/speclink-server/src/api.rs、crates/host/speclink-server/tests/it/identity/mod.rs、crates/host/speclink-server/tests/it/admin/mod.rs、crates/host/speclink-server/tests/it/api/mod.rs、crates/host/speclink-server/tests/it/web/mod.rs
  - Modified: crates/host/speclink-server/src/lib.rs、crates/host/speclink-server/src/identity.rs、crates/host/speclink-server/src/admin.rs、crates/host/speclink-server/src/web.rs、crates/host/speclink-server/src/backup.rs（搬入 admin/ 並改一處 import）、crates/host/speclink-server/src/assets.rs（搬入 web/ 並改 include_bytes! 路徑）、其餘 11 個搬移的模組檔（git mv）、crates/host/speclink-server/tests/it/main.rs、45 個搬移的測試檔（git mv）、apps/server-web/src/pages/admin/AuditPage.tsx、docs/product-status.md、docs/product-status.zh-TW.md、scripts/release/delivery-gate.test.mjs
  - Removed: （無；搬移不刪檔）
