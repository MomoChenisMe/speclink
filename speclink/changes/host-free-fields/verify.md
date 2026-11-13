# Verify — host-free-fields

## Round 1

**Phase**: discovery
**Patch**: sha256:80c1ac5538f88aa4406b90d48e26f362c7f53a4371971bcd152468bd423be640
**Scope**: apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/tests/it/remote_data.rs, crates/adapters/speclink-cli/src/remote_base.rs, crates/adapters/speclink-cli/src/verbs/discuss.rs, crates/adapters/speclink-cli/src/verbs/new.rs, crates/adapters/speclink-cli/src/verbs/plan.rs, crates/adapters/speclink-cli/src/verbs/query.rs, crates/adapters/speclink-cli/tests/it/free_fields.rs, crates/adapters/speclink-cli/tests/it/main.rs, crates/adapters/speclink-node/__test__/free-fields.spec.ts, crates/adapters/speclink-node/index.d.ts, crates/adapters/speclink-node/index.js, crates/adapters/speclink-node/src/host.rs, crates/adapters/speclink-node/src/lib.rs, crates/adapters/speclink-verbs/src/families/discuss.rs, crates/adapters/speclink-verbs/src/families/fields.rs, crates/adapters/speclink-verbs/src/families/mod.rs, crates/adapters/speclink-verbs/src/families/new.rs, crates/adapters/speclink-verbs/src/families/plan.rs, crates/adapters/speclink-verbs/src/inputs.rs, crates/adapters/speclink-verbs/src/render/discuss.rs, crates/adapters/speclink-verbs/src/render/fields.rs, crates/adapters/speclink-verbs/src/render/mod.rs, crates/adapters/speclink-verbs/src/render/query.rs, crates/adapters/speclink-verbs/src/tool.rs, crates/engine/speclink-core/src/command/describe.rs, crates/engine/speclink-core/src/command/fields.rs, crates/engine/speclink-core/src/command/mod.rs, crates/engine/speclink-core/src/command/tests.rs, crates/engine/speclink-core/src/command/typed.rs, crates/engine/speclink-core/src/keylines.rs, crates/engine/speclink-core/src/lib.rs, crates/engine/speclink-core/src/lifecycle/discuss.rs, crates/engine/speclink-core/src/lifecycle/discuss/tests.rs, crates/engine/speclink-core/src/lifecycle/fields.rs, crates/engine/speclink-core/src/lifecycle/listing.rs, crates/engine/speclink-core/src/lifecycle/mod.rs, crates/engine/speclink-core/src/lifecycle/model.rs, crates/engine/speclink-core/src/lifecycle/newcmd.rs, crates/engine/speclink-core/src/util.rs, crates/engine/speclink-core/tests/golden/command-describe.json, crates/engine/speclink-core/tests/it/command_describe.rs, crates/host/speclink-host/src/commit.rs, crates/host/speclink-host/src/documents.rs, crates/host/speclink-host/src/http/coverage.rs, crates/host/speclink-host/src/http/mod.rs, crates/host/speclink-host/src/http/router.rs, crates/host/speclink-host/src/http/routes.rs, crates/host/speclink-host/src/http/wire.rs, crates/host/speclink-host/src/kit.rs, crates/host/speclink-host/src/port_exec.rs, crates/host/speclink-host/tests/free_fields.rs, crates/host/speclink-host/tests/http_door.rs, crates/host/speclink-host/tests/kit.rs, crates/host/speclink-server/tests/it/admin/backup_e2e.rs, crates/host/speclink-server/tests/it/api/write_guard.rs, crates/host/speclink-server/tests/it/e2e_cli.rs, crates/host/speclink-server/tests/it/serverfs_store.rs, crates/host/speclink-server/tests/postgres_store.rs, crates/protocol/speclink-protocol/src/binding.rs, crates/protocol/speclink-protocol/src/command.rs, crates/protocol/speclink-protocol/src/query.rs, crates/protocol/speclink-remote/src/client.rs, crates/protocol/speclink-remote/tests/it/typed_client.rs
- [WARNING] crates/adapters/speclink-node/index.js — Correctness: Implementation may diverge from spec: node-sdk 規格寫「`fields` 的任一值不是字串時 SHALL 以 invalid_argv（dispatch）或 400（handleRequest）結束」，但 `fieldsText`（index.js:175-182）用 `JSON.stringify` 把物件轉成文字再交給 `fields_of`（crates/adapters/speclink-node/src/host.rs:186-191）檢查。`JSON.stringify` 會丟掉值是 `undefined`、函式、Symbol 的鍵，也會把 `Date` 轉成字串，所以 `{ source: undefined }` 不報錯、欄位直接不見，`{ at: new Date() }` 被當成字串寫入，`new Map(...)` 變成 `{}`。測試（crates/adapters/speclink-node/__test__/free-fields.spec.ts:116-128、230-242）只涵蓋數字、布林、巢狀物件、陣列、字串。建議：在 `fieldsText` 轉文字之前先檢查它是純物件、每個值 `typeof === 'string'`，不符就送出會被拒絕的 `'null'`，並補 `{ n: undefined }` 與 `Date` 的測試。
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: Implementation may diverge from spec: free-fields 規格寫「寫入時 SHALL 檢查…一份文件的鍵數不超過 32，任一項不符 SHALL 拒絕整次寫入」，design D3 也寫檢查「合計的數量」。`apply`（fields.rs:116）只在鍵數超過 32「而且比原本多」時才拒絕：手改成 40 個鍵的區塊，改值或「移除一個、加一個」都會照寫，文件仍超過 32 個。這是刻意的（fields.rs:92-93 的註解說為了讓手改過大的區塊還能修），但 design 的「留著的邊界」沒記，測試 `the_33rd_key_is_refused_and_the_32_stay`（fields.rs:382-395）只測到剛好 32 個。建議二選一：把這個放寬寫進 spec 與 design 並補一個超過 32 個鍵的測試，或把條件改成 `fields.len() > MAX_KEYS`。
- [SUGGESTION] speclink/changes/host-free-fields/specs/free-fields/spec.md — Coherence: 「自由欄位的寫入動詞」寫沒有 `=` 的參數與重複的鍵 SHALL 為「用法錯誤」，design D4 與程式（crates/adapters/speclink-verbs/src/families/fields.rs:109-136）是 `invalid_argv`，CLI 測試（crates/adapters/speclink-cli/tests/it/free_fields.rs:267-286）只斷言非零 exit code。行為一致，只是用詞不同；建議 spec 改成和 design 相同的說法。
- [SUGGESTION] speclink/changes/host-free-fields/proposal.md — Coherence: 「讀取」一節只列 `list`、`show`、`discuss list`、`discuss show` 的 `--json`，spec 與 design D5 還包含 `discuss search --json`；程式有做，也有測試（crates/adapters/speclink-verbs/src/render/discuss.rs:496-515、crates/adapters/speclink-cli/tests/it/free_fields.rs:397）。建議 proposal 補上這一項。
- [SUGGESTION] crates/adapters/speclink-cli/src/verbs/plan.rs — Coherence: design D4「`change field` 和 `change depends` 一樣寫進變更所在的那一份（worktree 副本）」有實作（plan.rs:91-99 的 `home_store_for`），但 CLI 的 free_fields.rs 裡沒有 worktree 的案例，也沒有其他 CLI 測試檔用到 `field` 動詞。建議補一個變更在 worktree 時寫入 worktree 副本的測試。

## Round 2

**Phase**: validation
**Patch**: sha256:ac9deb6eb58bc0106c278d2b95f567c4eb53dc6e6a8ff241698da7f351b770cb
**Scope**: crates/adapters/speclink-cli/src/remote_base.rs, crates/adapters/speclink-cli/src/verbs/discuss.rs, crates/adapters/speclink-cli/src/verbs/plan.rs, crates/adapters/speclink-cli/tests/it/free_fields.rs, crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs, crates/adapters/speclink-cli/tests/it/worktree_overlay.rs, crates/adapters/speclink-node/__test__/free-fields.spec.ts, crates/adapters/speclink-node/index.js, crates/adapters/speclink-node/src/host.rs, crates/adapters/speclink-node/src/lib.rs, crates/adapters/speclink-verbs/src/families/discuss.rs, crates/adapters/speclink-verbs/src/families/fields.rs, crates/adapters/speclink-verbs/src/families/plan.rs, crates/adapters/speclink-verbs/src/render/fields.rs, crates/adapters/speclink-verbs/src/render/mod.rs, crates/engine/speclink-core/src/command/fields.rs, crates/engine/speclink-core/src/keylines.rs, crates/engine/speclink-core/src/lifecycle/fields.rs, crates/host/speclink-host/src/http/mod.rs, crates/host/speclink-host/src/http/routes.rs, crates/host/speclink-host/src/kit.rs, crates/host/speclink-host/tests/common/door.rs, crates/host/speclink-host/tests/common/mod.rs, crates/host/speclink-host/tests/free_fields.rs, crates/host/speclink-host/tests/station_wire.rs, crates/protocol/speclink-remote/src/client.rs, crates/protocol/speclink-remote/tests/it/typed_client.rs
- [WARNING] crates/adapters/speclink-cli/tests/it/free_fields.rs — Correctness: 測試輔助改成共用 `remote_read_verbs::TempProject` 之後，暫存目錄的前綴從 `speclink-cli-free-fields-` 變成共用的 `speclink-cli-remote-read-`。`a_server_that_does_not_declare_fields_gets_the_create_and_the_user_gets_one_warning` 用 `TempProject::remote("old-server", …)`，而 remote_read_verbs.rs 的 `a_server_without_the_trace_route_is_a_not_found_failure` 也用 `TempProject::remote("old-server", …)`，兩者在同一個測試 binary、同一個 pid，目錄都是 `speclink-cli-remote-read-old-server-remote-<pid>`。平行執行時一方會刪掉或覆寫另一方的 `.speclink.yaml`（連到別人的假 server），client-protocol Scenario「舊 Server 不保存欄位時警告」唯一的 CLI 測試會變成時好時壞。我在這幾個檔沒找到序列化的鎖。修法：把 free_fields.rs 那個 tag 改成不重複的名字（例如 `fields-old-server`）。

## Round 3

**Phase**: validation
**Patch**: sha256:603c865d2ef9734b97d9a1e9f1c9639e6ffdb811c757975dc9f6fe5852b4738f
**Scope**: crates/adapters/speclink-cli/tests/it/free_fields.rs, crates/engine/speclink-core/src/lifecycle/fields.rs, crates/host/speclink-host/tests/free_fields.rs
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Correctness: design.md 稽核結論第 5 點新寫的句子說「數字、布林與 null 的原文不在『鍵: 值』那一行時（值寫在鍵的下一行…）寫入則拒絕」，但 null 寫在下一行（`a:\n    ~` 或 `a:\n    null`）沒有被拒絕。原因：`text_on_key_line` 對 `a:` 那一行回 `Some("")`，`Value::Null` 那一支直接收下、不經 `shown`，`as_written_all` 仍是 true，寫入照做，`~` 被改寫成 `""`。新測試只涵蓋 null 配引號鍵（`"a": ~`），沒有涵蓋 null 在下一行。這個行為本身在修正前就有；新的是「design 的新措辭和程式不一致」。這是讀程式推得的，沒有實跑。出路二選一：`Value::Null` 那一支在鍵那行沒字、而 YAML 區塊還有續行時也走 `shown`；或把 design 那句的 null 限縮成 flow 寫法與引號鍵。

## Round 4

**Phase**: validation
**Patch**: sha256:693b643e94774e2df7bcef8065fd857b28c813d49fc5bf287c69143784dfbcd6
**Scope**: crates/engine/speclink-core/src/lifecycle/fields.rs

## Round 5

**Phase**: validation
**Patch**: sha256:89dd35f0135a0f0297ef033f5f572acc9c267f9a35c50d841fc9187926b11968
**Scope**: crates/engine/speclink-core/src/lifecycle/fields.rs
- [WARNING] crates/engine/speclink-core/src/lifecycle/fields.rs — Coherence: `one_line_each` 只比「每一行縮排是否等於第一行」，沒有確認每一行真的是一個 `鍵: 值` 項目。flow 寫法放在 `fields:` 下面、又跨行且縮排相同時會通過：`fields:\n  {a: "x",\n  b: 2.10}\n`。此時 `entry_lines` 兩行同縮排，`text_on_key_line` 在第二行找到 `b:`，取到的文字是 `2.10}`（非空），`as_written_all` 仍為 true；`fields:` 是區塊鍵，`KeyLines::raw` 會收進兩行續行，`all_removed` 也是 true，所以 `intact` 為 true。讀取得到 `b = "2.10}"`，下一次 `field set` 會把原文寫成 `b: "2.10}"`，寫後回讀也相等，不會被擋。這牴觸 design.md D2 現在的字面（「只改寫…每個項目都是一行一個 `鍵: 值` 的區塊」、第 (3) 點「整個區塊寫成 `{a: 1.10}`」要拒絕）與稽核結論第 5 點「一條結構規則」。同類：`{b: "x",\n  a: 1.10,\n  c: "y"}` 讀成 `1.10,`。

## Round 6

**Phase**: validation
**Patch**: sha256:9baa32111a2a0bac2130f304ca8bcd0dc45631206b0100d03017225f9fa264cf
**Scope**: crates/engine/speclink-core/src/lifecycle/fields.rs
- [SUGGESTION] crates/engine/speclink-core/src/lifecycle/fields.rs — Coherence: `one_entry_each` 的 `alone.len() == 1` 讓「寫在 `fields:` 下一行、同一行有兩個以上項目的全字串 flow」（`fields:\n  {a: "x", b: "y"}\n`）從可改變成拒絕。同樣內容寫在鍵行上（`fields: {a: "x", b: "y"}`）或只有一個項目（`fields:\n  {a: "x"}`）仍可改。這符合 D2 第 (2) 點的字面，文件不變、沒有東西丟失；但稽核結論第 5 點的「全是字串的 `{a: "x"}` 寫法照常可改」沒有限定位置，字面上涵蓋不到這個差別，也沒有測試釘住它。此條是讀程式碼推得，沒有實跑。

## Round 7

**Phase**: validation
**Patch**: sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
**Scope**: speclink/changes/host-free-fields/design.md
