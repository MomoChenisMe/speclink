# Review — host-free-fields

## Round 1

**Phase**: discovery
**Patch**: sha256:80c1ac5538f88aa4406b90d48e26f362c7f53a4371971bcd152468bd423be640
**Rules**: extend
**Rule**: CRITICAL 呼叫端不得以 className 覆蓋 @speclink/ui 元件變體的顏色、圓角與陰影，需要的外觀改由變體提供
**Rule**: WARNING 全系統共通的行為（例如返回導覽、快捷鍵、提示框）要在共用層實作一次，各頁面只接上共用實作；新增的頁面要接上既有的共用實作，不要在頁面裡各自重寫
**Rule**: WARNING 新增功能時，repo 已有相同用途的元件、編輯器或工具函式（例如 @speclink/ui 的元件、設定頁共用的程式碼編輯器）要直接沿用，不要另做一套平行實作
**Rule**: WARNING 修正 bug 的變更：同一種寫法或同一類元件在 repo 其他位置若有同樣的問題，要在這次一併修正，或在 tasks.md 寫明不修的理由
**Rule**: WARNING 同一種操作在不同頁面要用同一個元件與同一種變體呈現（例如按鈕、選單、分頁工具、空狀態），不要同一種操作在各頁長得不一樣
**Scope**: apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/tests/it/remote_data.rs, crates/adapters/speclink-cli/src/remote_base.rs, crates/adapters/speclink-cli/src/verbs/discuss.rs, crates/adapters/speclink-cli/src/verbs/new.rs, crates/adapters/speclink-cli/src/verbs/plan.rs, crates/adapters/speclink-cli/src/verbs/query.rs, crates/adapters/speclink-cli/tests/it/free_fields.rs, crates/adapters/speclink-cli/tests/it/main.rs, crates/adapters/speclink-node/__test__/free-fields.spec.ts, crates/adapters/speclink-node/index.d.ts, crates/adapters/speclink-node/index.js, crates/adapters/speclink-node/src/host.rs, crates/adapters/speclink-node/src/lib.rs, crates/adapters/speclink-verbs/src/families/discuss.rs, crates/adapters/speclink-verbs/src/families/fields.rs, crates/adapters/speclink-verbs/src/families/mod.rs, crates/adapters/speclink-verbs/src/families/new.rs, crates/adapters/speclink-verbs/src/families/plan.rs, crates/adapters/speclink-verbs/src/inputs.rs, crates/adapters/speclink-verbs/src/render/discuss.rs, crates/adapters/speclink-verbs/src/render/fields.rs, crates/adapters/speclink-verbs/src/render/mod.rs, crates/adapters/speclink-verbs/src/render/query.rs, crates/adapters/speclink-verbs/src/tool.rs, crates/engine/speclink-core/src/command/describe.rs, crates/engine/speclink-core/src/command/fields.rs, crates/engine/speclink-core/src/command/mod.rs, crates/engine/speclink-core/src/command/tests.rs, crates/engine/speclink-core/src/command/typed.rs, crates/engine/speclink-core/src/keylines.rs, crates/engine/speclink-core/src/lib.rs, crates/engine/speclink-core/src/lifecycle/discuss.rs, crates/engine/speclink-core/src/lifecycle/discuss/tests.rs, crates/engine/speclink-core/src/lifecycle/fields.rs, crates/engine/speclink-core/src/lifecycle/listing.rs, crates/engine/speclink-core/src/lifecycle/mod.rs, crates/engine/speclink-core/src/lifecycle/model.rs, crates/engine/speclink-core/src/lifecycle/newcmd.rs, crates/engine/speclink-core/src/util.rs, crates/engine/speclink-core/tests/golden/command-describe.json, crates/engine/speclink-core/tests/it/command_describe.rs, crates/host/speclink-host/src/commit.rs, crates/host/speclink-host/src/documents.rs, crates/host/speclink-host/src/http/coverage.rs, crates/host/speclink-host/src/http/mod.rs, crates/host/speclink-host/src/http/router.rs, crates/host/speclink-host/src/http/routes.rs, crates/host/speclink-host/src/http/wire.rs, crates/host/speclink-host/src/kit.rs, crates/host/speclink-host/src/port_exec.rs, crates/host/speclink-host/tests/free_fields.rs, crates/host/speclink-host/tests/http_door.rs, crates/host/speclink-host/tests/kit.rs, crates/host/speclink-server/tests/it/admin/backup_e2e.rs, crates/host/speclink-server/tests/it/api/write_guard.rs, crates/host/speclink-server/tests/it/e2e_cli.rs, crates/host/speclink-server/tests/it/serverfs_store.rs, crates/host/speclink-server/tests/postgres_store.rs, crates/protocol/speclink-protocol/src/binding.rs, crates/protocol/speclink-protocol/src/command.rs, crates/protocol/speclink-protocol/src/query.rs, crates/protocol/speclink-remote/src/client.rs, crates/protocol/speclink-remote/tests/it/typed_client.rs
- [WARNING] crates/adapters/speclink-cli/tests/it/free_fields.rs — Standards: 違反專案規則「新增功能時，repo 已有相同用途的元件、編輯器或工具函式（例如 @speclink/ui 的元件、設定頁共用的程式碼編輯器）要直接沿用，不要另做一套平行實作」。第 16-181 行的 `NOV_12_0300_MS`、`TempProject`（`bare`/`local`/`remote`/`run`/`Drop`）、`stdout_of`、`stderr_of` 與 `Both` 幾乎逐行照抄同一個測試 crate 的 `remote_read_verbs.rs` 第 17-156 行（`Pair`）。本檔已經能 `use crate::door`、`crate::remote_plan::mock_server`，共用做得到。
- [WARNING] crates/host/speclink-host/tests/free_fields.rs — Standards: 同一條規則。第 27-178 行的 `Door`、`scope_s1`、`alice`、`door_with`、`door`、`seed`、`content`、`request`、`answer`、`body_of`、`get`、`post`、`etag_of`、`listed` 與 `station_wire.rs` 第 35-139 行逐字相同（diff 只差新增的函式與 `error` 一行），而兩檔都已 `mod common;`。
- [SUGGESTION] crates/protocol/speclink-remote/src/client.rs — Standards: possible Speculative Generality／Middle Man。`discussion_promote_with_fields`（約 923 行）沒有正式呼叫端，只有 `typed_client.rs:752` 的測試在用。另外做法不一致：core 的 `new_discussion`、`new_change` 直接加參數，`promote`、`Client::new_discussion`、`Client::discussion_promote` 卻各留一個只轉呼叫的舊名。
- [SUGGESTION] crates/host/speclink-host/src/kit.rs — Standards: possible Speculative Generality。`CallExtras` 只包一個 `fields`；`run_with_fields(…, &Fields)` 直接收欄位，`handle_with(…, &CallExtras)` 卻收包裝，兩個入口不對稱（CLAUDE.md「不為一次性使用的程式碼建立抽象層」）。
- [SUGGESTION] crates/adapters/speclink-node/src/lib.rs — Standards: possible Repeated Switches。`Command::NewChange { fields, .. } | DiscussNew | DiscussPromote` 這組比對在 `refuse_free_fields`（452-457）與 `kit.rs` 的 `attach_fields`（約 502-505）各寫一次。
- [SUGGESTION] crates/adapters/speclink-node/src/lib.rs — Standards: 不屬於 smell 清單，順帶回報：第 463 行拒絕訊息的字串在 `createEngine —` 後面有 18 個連續空白（看起來是換行接續掉了），會原樣顯示給使用者。
- [SUGGESTION] crates/adapters/speclink-verbs/src/families/fields.rs — Standards: possible Mysterious Name。`subject` 同時指種類（`FieldSubject`）與名稱字串（`FieldArgs::subject()`、`FieldsOutcome.subject`）；`FieldArgs::name(subject)` 回動詞名，`FieldEdit.name` 卻是變更名。
- [SUGGESTION] crates/adapters/speclink-cli/src/verbs/plan.rs — Standards: possible Duplicated Code。remote 的 `Field` 分支（158-163）與 `verbs/discuss.rs` 195-203 同一個形狀：`edit()` → `set_*_fields` → 手組 `FieldsOutcome` → `render_fields::edited`。
- [SUGGESTION] crates/adapters/speclink-node/src/host.rs — Standards: possible Duplicated Code。`DOOR_PREFIX = "/api/speclink/v1/"` 重複了 `speclink-host` `http/mod.rs:116` 的私有 `API_ROOT`。
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: `parse` 靠 `KeyLines::raw` 取區塊，但 `raw` 看到的比 YAML 少，`intact` 仍回 true。(a) 手寫 `fields: # 註解`（或 `fields: &a`）時 `is_block_key` 為 false，`raw` 只回那一行，YAML 讀成 Null → `none(true)`：下面的項目讀不到；寫入時 `set_block`→`remove` 只刪鍵行，`  a: "1"` 變成孤兒行。(b) 區塊中間有第 0 欄的 `# 註解` 行時 `take_while(is_continuation)` 提前停，之後的項目同樣漏讀、寫入後變孤兒。討論記錄沒有整份檢查，`read(lines) != *fields` 也照過，結果是 frontmatter 留垃圾行、手寫欄位靜默遺失；變更則被 `check_meta_text` 擋下，但回的是誤導的 MetaError（磁碟上的檔其實沒壞）。現有測試（`an_edit_never_drops_what_a_hand_edit_put_in_the_block`）沒涵蓋這兩種寫法。
- [SUGGESTION] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: `as_written` 只取鍵那一行冒號後的文字。值寫在下一行（`a:\n    123`）時回空字串且 `intact` 為 true，下一次寫入會把 123 改寫成 `a: ""`。另外 `fields: {a: 1}` 因 `raw.lines().skip(1)` 為空，整項被略過（`{a: "1"}` 卻讀得到）。
- [SUGGESTION] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: 文件有兩個第 0 欄 `fields:` 時，`raw` 只讀第一個，`write` 的 `lines.remove(FIELDS_KEY)`／`set_block` 卻把兩個都刪掉，第二個區塊的內容無聲消失；`write` 沒有對重複鍵拒絕。
- [SUGGESTION] crates/adapters/speclink-node/src/lib.rs — Correctness: `refuse_free_fields` 的訊息字面值（約第 463 行）在 `createEngine —` 後面夾了約 18 個連續空白（`createEngine —                  it does not carry them`），看起來是續行符號被攤平，使用者會直接看到。
- [SUGGESTION] crates/adapters/speclink-node/index.js — Correctness: `fieldsText` 用 `JSON.stringify`，`{ ticket: undefined }` 或值為函式的鍵會被直接略掉、不報錯；`Date` 值會經 `toJSON` 變成字串而被收下，與「非字串一律拒絕」的原意不符。

## Round 2

**Phase**: validation
**Patch**: sha256:ac9deb6eb58bc0106c278d2b95f567c4eb53dc6e6a8ff241698da7f351b770cb
**Scope**: crates/adapters/speclink-cli/src/remote_base.rs, crates/adapters/speclink-cli/src/verbs/discuss.rs, crates/adapters/speclink-cli/src/verbs/plan.rs, crates/adapters/speclink-cli/tests/it/free_fields.rs, crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs, crates/adapters/speclink-cli/tests/it/worktree_overlay.rs, crates/adapters/speclink-node/__test__/free-fields.spec.ts, crates/adapters/speclink-node/index.js, crates/adapters/speclink-node/src/host.rs, crates/adapters/speclink-node/src/lib.rs, crates/adapters/speclink-verbs/src/families/discuss.rs, crates/adapters/speclink-verbs/src/families/fields.rs, crates/adapters/speclink-verbs/src/families/plan.rs, crates/adapters/speclink-verbs/src/render/fields.rs, crates/adapters/speclink-verbs/src/render/mod.rs, crates/engine/speclink-core/src/command/fields.rs, crates/engine/speclink-core/src/keylines.rs, crates/engine/speclink-core/src/lifecycle/fields.rs, crates/host/speclink-host/src/http/mod.rs, crates/host/speclink-host/src/http/routes.rs, crates/host/speclink-host/src/kit.rs, crates/host/speclink-host/tests/common/door.rs, crates/host/speclink-host/tests/common/mod.rs, crates/host/speclink-host/tests/free_fields.rs, crates/host/speclink-host/tests/station_wire.rs, crates/protocol/speclink-remote/src/client.rs, crates/protocol/speclink-remote/tests/it/typed_client.rs
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: F2 的退路（`as_written` 回 `None` 就用 `number.to_string()`／`flag.to_string()`／空字串）也套到「找不到鍵那一行」的情況，而且 `intact` 照樣是 true。手寫 `fields:\n  "a": 1.10\n`（鍵加引號）修正前是「略過該項＋拒絕寫入」，現在讀成 `1.1`，下一次 `field set` 任何鍵就把它寫回 `a: "1.1"`，原文靜默改掉（同類：`True`→`true`、`~`→空字串、`0x1F`→`31`）。flow 寫法 `{a: 1.10}` 與下一行寫法 `a:\n    1.10` 也一樣。這違反「數字照原文」（`1.10` 保持 `1.10`）。方向：退路取到的文字不保證等於原文時，把 `intact` 設為 false，或從原文取下一行／flow 內的字面。

## Round 3

**Phase**: validation
**Patch**: sha256:603c865d2ef9734b97d9a1e9f1c9639e6ffdb811c757975dc9f6fe5852b4738f
**Scope**: crates/adapters/speclink-cli/tests/it/free_fields.rs, crates/engine/speclink-core/src/lifecycle/fields.rs, crates/host/speclink-host/tests/free_fields.rs
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: F2 的退路（`as_written` 回 `None` 就用 `number.to_string()`／`flag.to_string()`／空字串）也套到「找不到鍵那一行」的情況，而且 `intact` 照樣是 true。手寫 `fields:\n  "a": 1.10\n`（鍵加引號）修正前是「略過該項＋拒絕寫入」，現在讀成 `1.1`，下一次 `field set` 任何鍵就把它寫回 `a: "1.1"`，原文靜默改掉（同類：`True`→`true`、`~`→空字串、`0x1F`→`31`）。flow 寫法 `{a: 1.10}` 與下一行寫法 `a:\n    1.10` 也一樣。這違反「數字照原文」（`1.10` 保持 `1.10`）。方向：退路取到的文字不保證等於原文時，把 `intact` 設為 false，或從原文取下一行／flow 內的字面。

## Round 4

**Phase**: validation
**Patch**: sha256:693b643e94774e2df7bcef8065fd857b28c813d49fc5bf287c69143784dfbcd6
**Scope**: crates/engine/speclink-core/src/lifecycle/fields.rs
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: F2 的退路（`as_written` 回 `None` 就用 `number.to_string()`／`flag.to_string()`／空字串）也套到「找不到鍵那一行」的情況，而且 `intact` 照樣是 true。手寫 `fields:\n  "a": 1.10\n`（鍵加引號）修正前是「略過該項＋拒絕寫入」，現在讀成 `1.1`，下一次 `field set` 任何鍵就把它寫回 `a: "1.1"`，原文靜默改掉（同類：`True`→`true`、`~`→空字串、`0x1F`→`31`）。flow 寫法 `{a: 1.10}` 與下一行寫法 `a:\n    1.10` 也一樣。這違反「數字照原文」（`1.10` 保持 `1.10`）。方向：退路取到的文字不保證等於原文時，把 `intact` 設為 false，或從原文取下一行／flow 內的字面。
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: `value_is_below` 只看下一個非空行是否更深。手寫 `fields:\n  a:\n  # note\n    1.10\n`（註解縮排不深於鍵）時，下一行是註解，判成「值不在下面」，`text_on_key_line` 回 `Some("")`。這次拿掉了 Number／Bool 的 `.filter(|text| !text.is_empty())`，所以數字 `1.10` 被讀成空字串、`intact` 仍為 true，下一次 `field set` 把它寫成 `a: ""`。修正前同一形狀讀成 `1.1` 並拒絕寫入。註解行以空白開頭，`is_continuation` 收進 `KeyLines::raw`，`all_removed` 擋不到；寫入後的回讀檢查也相等。null 的同形狀（`a:\n  # note\n    ~`）修正前後都沒被拒絕，屬於 F1 的殘留。方向：找「下一行」時跳過註解行，或空文字只對 Null 成立。

## Round 5

**Phase**: validation
**Patch**: sha256:89dd35f0135a0f0297ef033f5f572acc9c267f9a35c50d841fc9187926b11968
**Scope**: crates/engine/speclink-core/src/lifecycle/fields.rs

## Round 6

**Phase**: validation
**Patch**: sha256:9baa32111a2a0bac2130f304ca8bcd0dc45631206b0100d03017225f9fa264cf
**Scope**: crates/engine/speclink-core/src/lifecycle/fields.rs
