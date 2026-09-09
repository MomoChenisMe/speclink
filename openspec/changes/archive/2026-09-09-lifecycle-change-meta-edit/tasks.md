## 1. 基準

- [x] 1.1 建立回歸基準：改碼前跑 `cargo test -p speclink-core` 與 `cargo test -p speclink-cli --test it`，記下全綠與 golden 零 diff（`render_golden::`）的狀態；基準本身若紅，先停下回報，不在本 change 修。驗證：兩條指令 exit code 0。路徑：crates/speclink-core、crates/speclink-cli/tests/it <!-- speclink-task:tsk_01M239Z228AZH14CDPNSYEMB0W -->

## 2. KeyLines 長出 change meta 的四個能力（design D1 KeyLines 長出 whole 區域與清單／區塊三式）

- [x] 2.1 【Red】在 crates/speclink-core/src/keylines.rs 測試模組寫 D1 KeyLines 長出 whole 區域與清單／區塊三式的測試：`whole` 對「尾行缺換行」插入前先補行、空文件插入得 `key: value\n`、CRLF 文件新行沿 CRLF；`push_list` 已含回 false 不改、空清單寫單值、既有清單以「, 」累加；`remove_list` 不在清單回 false、縮清單保留其餘、清空整行移除；`set_block` 取代既有同名鍵與其縮排區塊並落在區域尾端、續行原文逐字寫入、任一行含換行回 `Err` 且 `text()` 逐位元不變；`frontmatter` 既有測試不動。驗證：`cargo test -p speclink-core keylines` 新測試失敗於方法不存在。 <!-- speclink-task:tsk_01M239Z228V65S5V35T4M8WMV2 -->
- [x] 2.2 【Green】實作 `KeyLines::whole`、`push_list`、`remove_list`、`set_block`（內部以 `start` 欄位區分 frontmatter 與 whole 區域，不引入 Region 列舉；`whole` 永遠可插、尾行補行沿多數行尾），更新模組文件的介面段。驗證：`cargo test -p speclink-core keylines` 全綠。路徑：crates/speclink-core/src/keylines.rs <!-- speclink-task:tsk_01M239Z228FE263QKBABSZYQ84 -->

## 3. edit_meta（design D2 edit_meta 四步與回傳形狀）

- [x] 3.1 【Red】在 crates/speclink-core/src/model.rs 測試模組寫 D2 edit_meta 四步與回傳形狀的四條測試，用 `TestStore::meta_writes` 計數：meta 不存在回 `Ok(None)` 且寫入 0 次；壞 YAML 回 `Err` 可 `downcast_ref::<MetaError>()` 且寫入 0 次；閉包不改任何鍵回 `Ok(Some(_))` 且寫入 0 次；閉包 `set` 一鍵回 `Ok(Some(_))`、寫入恰 1 次、其餘位元組逐字保留。驗證：`cargo test -p speclink-core model::tests::edit_meta_` 失敗於函式不存在。 <!-- speclink-task:tsk_01M239Z22866FAN8F548W27TK1 -->
- [x] 3.2 【Green】實作 `model::edit_meta<T>(store, name, edit: impl FnOnce(&mut KeyLines) -> Result<T>) -> Result<Option<T>>`：讀原文→`check_meta_text` fail-closed→`KeyLines::whole`→套閉包→`text()` 與原文不同才寫。驗證：`cargo test -p speclink-core model::tests::edit_meta_` 全綠。路徑：crates/speclink-core/src/model.rs <!-- speclink-task:tsk_01M239Z228SW71Q14QJZMM49ZH -->

## 4. 七個寫入點改走 edit_meta（design D3 七個寫入點改走 edit_meta、各自守門與訊息不動）

- [x] 4.1 `set_board_rank` 改走 `edit_meta`＋`set("board_rank", rank)`；rank 合法性與名稱單段守衛不動，`None` 回既有字串「change not found: {name}」。可觀察行為：既有 `set_board_rank_*` 六條測試斷言不改全綠（含壞檔零寫入、缺尾換行）。驗證：`cargo test -p speclink-core model::tests::set_board_rank` 全綠。路徑：crates/speclink-core/src/model.rs <!-- speclink-task:tsk_01M239Z228RSQ1GWN6VM8SG6VB -->
- [x] 4.2 in-progress `add`／`remove` 改走 `edit_meta`：`add` 閉包內判 `started_*` 任一存在回 `Ok(false)`、否則 `set` 三鍵（by／with 過 `yaml_scalar`）；`remove` 閉包內先跑零工作痕跡守門（`RevertBlocked`）、再判無 `started_*` 回 `Ok(false)`、否則 `remove` 三鍵；名稱單段與 not-found 的訊息與靜默語意沿 D3 七個寫入點改走 edit_meta、各自守門與訊息不動的表格。可觀察行為：`add_*`／`remove_*` 既有測試斷言不改全綠，含 `*_preserves_*_verbatim` 三條與 `add_refuses_on_corrupt_meta_without_writing`。驗證：`cargo test -p speclink-core inprogress` 全綠。路徑：crates/speclink-core/src/inprogress.rs <!-- speclink-task:tsk_01M239Z228VR4CKSCKFXE9RJHS -->
- [x] 4.3 `run_claim` 改走 `edit_meta`：閉包以 `get("claimed_by")`／`get("claimed_at")` 做既有四臂 match（同 holder→`Ok(false)`、他人與半章→`crate::command::Refusal` 帶原凍結字串），再 `set` `claimed_at`／`claimed_by`（過 `yaml_scalar`）→`Ok(true)`；`None`→既有 `NotFound` 字串；`MetaError` 經 `classify` 映射不變。可觀察行為：claim 家族測試（含「already claimed by」「carries claimed_at with no claimed_by」字串）斷言不改全綠。驗證：`cargo test -p speclink-core claim` 全綠。路徑：crates/speclink-core/src/command/mod.rs <!-- speclink-task:tsk_01M239Z228JB72QVCQBH89HNJF -->
- [x] 4.4 `link`／`stamp_restale`／`clear_restale` 改走 `edit_meta`：`link` 閉包 `push_list("from_discussion", slug)`、`None` 回既有「Change '{change}' not found.」；`stamp_restale` 對 `None` 與 `downcast_ref::<MetaError>()` 命中皆 `continue`、其餘錯誤照傳、`flagged` 沿今天不論新增都推入；`clear_restale` 閉包 `remove_list("restale_from", slug)`、`None` 回 `Ok(())`。可觀察行為：`link_*`（含冪等零寫入、壞 meta 拒絕、缺尾換行）、`conclude_restale_*`、`seal_*` 既有測試斷言不改全綠。驗證：`cargo test -p speclink-core discuss::tests::link`、`cargo test -p speclink-core discuss::tests::conclude_restale`、`cargo test -p speclink-core discuss::tests::seal` 全綠。路徑：crates/speclink-core/src/discuss.rs <!-- speclink-task:tsk_01M239Z228CKAZ9Q4KTFDV2JNP -->

## 5. 站別章寫入（design D4 站別章寫入直接用 KeyLines、刪除先於寫入）

- [x] 5.1 依 D4 站別章寫入直接用 KeyLines、刪除先於寫入改寫 `write_stamp`：`KeyLines::whole(&gate.raw_meta)` → 五鍵 `remove` → 四純量 `set`（by／with／指紋過 `yaml_scalar`）→ `set_block("{prefix}_scope", 續行)` → `delete_artifact` → `write_change_meta(m.text())`；刪除 `strip_stamp_lines`。可觀察行為：重蓋章後章仍落檔尾、其餘欄位逐位元保留，review／verify／station 既有比對整份 meta 的蓋章測試斷言不改全綠。驗證：`cargo test -p speclink-core station`、`cargo test -p speclink-core review`、`cargo test -p speclink-core verify` 全綠；`grep -rn strip_stamp_lines crates/speclink-core/src` 零命中。路徑：crates/speclink-core/src/station.rs <!-- speclink-task:tsk_01M239Z228KR7SEAXA2G91ZKXX -->

## 6. 收尾（design D5 零行為變化的驗證面）

- [x] 6.1 依 D5 零行為變化的驗證面做全面驗收：`git diff` 確認八個寫入點所在的既有測試只改呼叫方式、不改斷言字串；`grep -n 'write_change_meta(' crates/speclink-core/src/*.rs crates/speclink-core/src/command/mod.rs` 非測試碼只剩 `edit_meta` 與 `write_stamp` 兩處；八個寫入點函式內不再有 `ends_with('\n')` 補行；`cargo test -p speclink-core`、`cargo test -p speclink-core --test it`（含 `render_golden::` 零 diff）、`cargo test -p speclink-cli --test it`、`cargo test -p speclink-desktop-core` 全綠。驗證：上列指令全部 exit code 0。 <!-- speclink-task:tsk_01M239Z228GF96FFEWSH2YHA3P -->
- [x] 6.2 `cargo clippy -p speclink-core --all-targets -- -D warnings` 零警告（刪除手寫前奏後不留孤兒 import，例如 inprogress.rs 的 `ChangeMeta` 若不再使用）；跑 `node --test scripts/*.test.mjs` 全綠。驗證：兩條指令 exit code 0。路徑：crates/speclink-core、scripts/ <!-- speclink-task:tsk_01M239Z228CC3X1C8654N4QPW4 -->
