## 1. Design 文件更新（不寫程式碼，先把 slice 命名鎖住）

- [x] 1.1 在 `doc/speclink-design.md` §1.1「Walking skeleton slice naming」表格新增 A5 列：`add-config-rw` / 「state.db v5 migration（`config_state` singleton + `config_change` audit）+ `config.read` / `config.write` 兩個 op + `ConfigStore` trait + A3 review-flag hardcode 改讀 config + walking-skeleton fallback semantics for malformed config + external-edit detection」；驗證方式：`grep -c '^| A5 |' doc/speclink-design.md` 等於 1、`grep 'add-config-rw' doc/speclink-design.md` 至少出現於該表
- [x] 1.2 更新 §1.1 表底下「A1 已 archive、A2 已 archive、A3 已 archive；A4 為本文件當前主題」字串，改成「A1–A4 已 archive；A5 為本文件當前主題」；驗證：grep 該字串只剩 A5 版本，不再有 A4 版本

## 2. State.db v5 migration（TDD 紅燈先寫）

- [x] 2.1 [P] 在 `crates/provider-local/tests/migration_v5.rs` 寫 3 條紅燈測試：(a) v4 → v5 升版後 `_migrations` 含 version=5 row；(b) `config_state` 表存在、有 id=1 row、`content_sha256` 等於當前 config.yaml 的 sha256、`version=1`；(c) `config_change` 表存在且為空；對應 spec requirement「state.db schema MUST be upgraded to version 5」（local-storage-layout delta 與 config-rw 兩處皆 cover）；對應 design decision「Config_state singleton 表 via CHECK 約束」「Config_change audit 表設計沿 A3 state_transition 範式」；驗證方式：`cargo test -p speclink-provider-local migration_v5` 紅燈
- [x] 2.2 新增 `crates/provider-local/src/migrations/v5_config_tables.sql` 內含 `CREATE TABLE config_state (id INTEGER PRIMARY KEY CHECK (id = 1), ...)`、`CREATE TABLE config_change (change_seq INTEGER PRIMARY KEY AUTOINCREMENT, ...)`、`INSERT OR IGNORE INTO config_state (id, content_sha256, ...) VALUES (1, <runtime-computed>, ...)`、`INSERT INTO _migrations (version, applied_at) VALUES (5, <now>)`；註冊進 `crates/provider-local/src/state_db.rs` 既有的 migration runner（含 checksum、依 A1/A2/A3/A4 同 pattern）；對應 spec requirement「state.db SHALL be upgraded to version 5 with `config_state` and `config_change` tables」（config-rw 視角）以及 local-storage-layout 視角「`state.db` schema MUST be upgraded to version 5 with the `config_state` and `config_change` tables」（兩處同一 migration、雙向 cross-ref）；驗證：2.1 全綠
- [x] 2.3 [P] 加 1 條紅燈測試斷言 `config_state` 表的 CHECK 約束會拒絕 `INSERT INTO config_state (id, ...) VALUES (2, ...)`；對應 spec scenario「Schema constraint rejects second row in config_state」；驗證：測試由紅轉綠後 `cargo test -p speclink-provider-local migration_v5_singleton_check` 通過

## 3. `ConfigStore` trait 與型別（純 trait surface、無 impl 行為）

- [x] 3.1 [P] 在 `crates/provider/tests/config_store_trait.rs` 寫紅燈測試確認 `Provider` 與 `ConfigStore` 兩 trait surface 可以從外部 crate 取用、編譯通過 `let _ = provider.config_store().read_config()`；對應 spec requirement「`ConfigStore` trait SHALL be exposed via `Provider::config_store()`」；驗證：紅燈 `cargo test -p speclink-provider --test config_store_trait`
- [x] 3.2 在 `crates/provider/src/config_store.rs` 定義 `ConfigStore` trait（`read_config` / `write_config` / `read_defaults` 三 method）、`WriteConfigRequest` enum（`Set` / `Edit` 兩 variant 含 `expected_etag: Option<Etag>` 與 `actor: Option<ActorJson>`）、`Config` 結構（含 `Rules { require_artifact_review: bool, require_code_review: bool }` 與 opaque `roles: serde_yaml::Value`）；在 `crates/provider/src/lib.rs` 的 `Provider` trait 加 `fn config_store(&self) -> &dyn ConfigStore`；對應 design decision「JSONPath subset grammar 與 CLI value 解析規則」（新增 `JsonPath` newtype，承載解析後的 segment vec）；驗證：3.1 由紅轉綠

## 4. `LocalConfigStore` read path + walking-skeleton fallback

- [x] 4.1 [P] 在 `crates/provider-local/tests/config_store.rs` 寫紅燈測試：(a) happy read — fresh init project、read_config 回 `value.rules.require_*_review = false`、`etag` 匹配 `^v[0-9]+\.[0-9a-f]{12}$`、無 warnings；(b) missing file fallback — 刪掉 `.speclink/config.yaml`、read_config 仍成功回 defaults、`etag="v0.malformed-fallback"`、warnings 含 `config.malformed_using_defaults`；(c) malformed YAML fallback — 寫一份 syntactically broken YAML、read 同樣 fallback、`config_change` 表沒有新 row；對應 spec requirement「Read path SHALL fall back to defaults when config is missing or malformed」；對應 design decision「Walking-skeleton fallback semantics for malformed config」；驗證：紅燈
- [x] 4.2 實作 `crates/provider-local/src/config_store.rs` `LocalConfigStore::read_config()` happy path + fallback path；etag 公式採 `v<version>.<sha256[:12]>`（對應 design decision「Config etag 命名格式對齊 artifact etag」）；fallback 不 raise error、走 warning；對應 spec requirement「`speclink config show` SHALL read config.yaml and return `Versioned<Config>`」；驗證：4.1 (a)(b)(c) 三條全綠
- [x] 4.3 [P] `read_defaults()` 回 `Config { rules: Rules { require_artifact_review: false, require_code_review: false }, ... }`；在 `crates/provider-local/tests/config_store.rs` 加紅燈測試斷言 `read_defaults().rules.require_*_review` 均為 `false`；驗證：紅→綠後 `cargo test -p speclink-provider-local config_store_defaults` 通過

## 5. External-edit detection（read path 寫 audit）

- [x] 5.1 [P] 在 `crates/provider-local/tests/config_store.rs` 新增紅燈測試 `read_config_detects_external_edit`：fresh init → 直接寫一份不同 bytes 的 config.yaml（繞過 engine）→ 下一次 read_config 應回新 value、warnings 含 `config.external_edit_detected`、`config_change` 表多一筆 `mode='external_edit'`、`keys_changed='["__external_edit__"]'`、`etag_before` / `etag_after` 都對得上；對應 spec requirement「Read path SHALL detect external file edits and reconcile via audit log」；對應 design decision「External-edit detection algorithm」；驗證：紅燈
- [x] 5.2 實作 reconcile 流程於 `LocalConfigStore::read_config()`：先讀檔算 sha + size → 對照 `config_state` row → 若不一致開 SQLite tx → UPDATE config_state（version+1）+ INSERT config_change(mode='external_edit', reason='config_external_edit') → commit → 回新 etag + warning；驗證：5.1 由紅轉綠
- [x] 5.3 [P] 加紅燈測試 `read_config_external_edit_then_stale_write_fails`：external edit 後、stale process 帶 `expected_etag=<舊>` 呼叫 `write_config(Set)` 應拒絕、回 `state.etag_mismatch`、檔案不變、不寫新 audit row；對應 spec scenario「Concurrent write after external edit fails CAS」；驗證：實作完 5.2 + 第 6 章 write path 後此測試可綠

## 6. `LocalConfigStore` write path（mode=set / edit）+ CAS

- [x] 6.1 [P] 在 `crates/provider-local/tests/config_store.rs` 寫 6 條紅燈測試：(a) write set happy（沒帶 expected_etag）— 套用 patch、檔案被覆寫、`config_change(mode='set', keys_changed=["rules.require_code_review"], reason='config_write')` row 寫入、回新 etag；(b) write set CAS 通過（expected_etag 對得上）；(c) write set CAS 失敗（expected_etag 對不上）→ exit 7、`state.etag_mismatch`、無檔案變動、無 audit row；(d) write set unknown key → `config.key_not_found`；(e) write edit happy（mode=edit、content=合法 YAML）— 整檔覆寫、`config_change(mode='edit', keys_changed=["__edit__"])` 寫入；(f) write edit malformed → exit 3、`config.malformed`、無檔案變動；對應 spec requirement「`speclink config set <key> <value>` SHALL patch config.yaml with optimistic concurrency」「`speclink config edit` SHALL replace config.yaml contents via interactive editor or stdin」；對應 design decision「`keys_changed` 對 mode=edit 採簡化標記」；驗證：紅燈
- [x] 6.2 實作 `LocalConfigStore::write_config()` Set / Edit 兩 path；單 SQLite tx 內完成「讀現 etag → 比對 expected_etag（若有） → 算 patch / 覆寫 → atomic 寫檔（tempfile + rename） → UPDATE config_state（version+1、新 sha） → INSERT config_change row」；對應 design decision「Lock 沿用 stub no-op」（取 `global-short` lock 走 stub no-op、單 tx + CAS 保證原子性）；驗證：6.1 全綠
- [x] 6.3 [P] 加紅燈測試 `write_config_value_parsing_precedence`：drive `WriteConfigRequest::Set { value: ConfigValue::String("true"), ... }` 等多種 input、檢驗 `ConfigValue` 解析器照 `true/false/null → int → float → string` 順序；對應 spec scenario「Value parsing precedence」與 design decision「JSONPath subset grammar 與 CLI value 解析規則」；驗證：實作 `ConfigValue::parse(&str)` 函式後測試由紅轉綠

## 7. CLI subcommand（`config show` / `set` / `edit`）

- [x] 7.1 [P] 在 `crates/cli/tests/config_cli.rs` 寫紅燈 CLI 整合測試：(a) `speclink config show --json` 回 envelope 含 `data.value` 與 `data.etag`；(b) `config show --key rules.require_code_review --json` 回 `{ key, value, etag }`；(c) `config show --key 'rules.*' --json` exit 2、`config.key_not_found`、hint 提到 wildcard 不支援；對應 spec requirement「`speclink config show` SHALL read config.yaml and return `Versioned<Config>`」；驗證：紅燈
- [x] 7.2 實作 `crates/cli/src/commands/config.rs`（`show` / `set` / `edit` 三 subcommand）、`crates/cli/src/commands/mod.rs` 註冊 subcommand group；`show --key` JSONPath 解析共用 `crates/provider/src/jsonpath.rs`（grammar 限制 alphanumeric + `_` + `-`、加 `[index]`、明文拒絕 wildcard）；對應 design decision「JSONPath subset grammar 與 CLI value 解析規則」；驗證：7.1 全綠
- [x] 7.3 [P] CLI 整合測試 set path：`speclink config set rules.require_code_review true --json` exit 0、回 `keys_changed=["rules.require_code_review"]`、re-read 看到新值；`--expected-etag <錯>` exit 7；`rules.unknown_key true` exit 2 `config.key_not_found`；對應 spec requirement「`speclink config set <key> <value>` SHALL patch config.yaml with optimistic concurrency」；驗證：實作 CLI set + ConfigValue parser 後綠
- [x] 7.4 [P] CLI 整合測試 edit --stdin path：`echo '<new yaml>' | speclink config edit --stdin --json` exit 0、回新 etag；malformed yaml → exit 3 `config.malformed`；對應 spec requirement「`speclink config edit` SHALL replace config.yaml contents via interactive editor or stdin」；驗證：實作 CLI edit + stdin reader 後綠
- [x] 7.5 CLI 整合測試 edit `$EDITOR` path（無 stdin 旗標、走 `$EDITOR=cat` mock）：驗證命令會 spawn child process、把當前 config 寫進臨時檔、子程序結束後讀回 buffer；非 TTY 環境下若 user 漏給 `--stdin` 也漏給 `--editor` → exit 2、hint 提示用 `--stdin`；對應 design decision「JSONPath subset grammar 與 CLI value 解析規則」（CLI value 解析規則包含 stdin / editor mode 分流）；驗證：cargo test -p speclink-cli config_cli_edit_editor_mode 通過

## 8. State machine evaluator 接通 config（A3 hardcode 替換）

- [x] 8.1 [P] 在 `crates/runtime/tests/state_machine.rs` 寫 4 條紅燈測試對應 spec scenarios：(a) default config 仍 skip reviewing — fresh init project + DAG 齊全 → proposing → ready；(b) `config set rules.require_artifact_review true` 後 DAG 齊全 → proposing → reviewing；(c) `config set rules.require_code_review true` 後 last task done → in_progress → code_reviewing；(d) mid-cycle flip — set 完旗標立刻 artifact.write 觸發 evaluator → 看到新 config；對應 spec requirement「Walking-skeleton mode SHALL hard-code both review flags to `false`」（A5 已 MODIFIED 為 config-driven）；對應 design decision「Walking-skeleton fallback semantics for malformed config」；驗證：紅燈
- [x] 8.2 在 `crates/runtime/src/state_machine.rs` 把 `ReviewPolicy::walking_skeleton()` 移除、改為 `ReviewPolicy::from_config(&Config)`；evaluator 每次 firing 都呼叫 `Provider::config_store().read_config()`、不快取；read_config 回 warning 時把 warning pass-through 進 op JSON envelope；驗證：8.1 全綠
- [x] 8.3 [P] 紅燈測試 `state_machine_passes_through_malformed_warning`：把 config.yaml 寫壞、artifact.write 觸發 evaluator → response envelope `warnings` 含 `config.malformed_using_defaults`；驗證：實作完 evaluator warning pass-through 後綠

## 9. Project bootstrap 插入 `config_state` row

- [x] 9.1 [P] 在 `crates/cli/tests/cli.rs`（或新檔 `crates/cli/tests/init_config_state.rs`）寫紅燈測試：fresh `speclink init` 後查 `state.db.config_state` 應有 `id=1`、`version=1`、`content_sha256` 對得上、`written_by=NULL`；對應 spec requirement「`speclink init` MUST insert the `config_state` singleton row in the same transaction as the project row」；對應 design decision「A1 init prepare/commit phase 插入 config_state row」；驗證：紅燈
- [x] 9.2 修改 `crates/provider-local/src/store.rs` 的 init prepare/commit phase（既有 LocalStore 與 init 邏輯住處）：在同 SQLite tx 內 insert `project` row 與 `config_state(id=1, ...)` row；prepare 階段算 config.yaml sha256 + size + mtime_ns；rollback path SQLite tx 自動清掉兩 row；驗證：9.1 由紅轉綠
- [x] 9.3 [P] 紅燈測試 `init_failure_leaves_no_config_state_row`：用 read-only 目錄或 mock 一個會 fail 的 prepare step、init 失敗、檢查 state.db（若有開啟）沒留下任何 row；對應 spec scenario「Failed init leaves no config_state row」；驗證：實作完 9.2 後綠
- [x] 9.4 [P] 紅燈測試 `init_force_updates_config_state_only_if_bytes_changed`：對既 init 過的 project 跑 `init --force`、檢查若 config.yaml bytes 沒變 → config_state row 不動；若 init 過程改寫了 config.yaml → version+1；對應 spec scenario「Re-init with --force preserves config_state row alignment」；驗證：實作完 9.2 + 對 force path 加 sha 比對後綠

## 10. Error 註冊與 envelope warnings

- [x] 10.1 [P] 在 `crates/runtime/src/error.rs` 與 `crates/provider/src/error.rs` 註冊 `config.not_found` (exit 2) / `config.malformed` (exit 3) / `config.key_not_found` (exit 2) 三條 error code；確保 `output::error_code_to_exit` 對照表同步更新；對應 spec requirement「New error codes SHALL be registered with stable exit codes」；驗證：在 `crates/provider/tests/error_codes.rs` 加 3 條紅燈斷言新 code 的 exit 值對得上、執行 `cargo test -p speclink-provider error_codes` 由紅轉綠
- [x] 10.2 [P] 在 `crates/runtime/src/ops.rs`（或對應 envelope 組裝點）加入 `config.external_edit_detected` 與 `config.malformed_using_defaults` 兩個 warning code 的 emit 路徑；JSON envelope `warnings` 欄位必定為陣列、出現順序穩定（external_edit 在前、malformed 在後若同時觸發）；驗證：在 8.3 / 5.1 既有測試斷言中加入 `warnings` shape 比對、CLI human renderer 印出對應 hint

## 11. Operations.md 標記與 design.md cross-reference

- [x] 11.1 在 `doc/protocol/operations.md` `config.read`（L1482 附近）與 `config.write`（L1550 附近）兩 op 的「| **MVP** | ✓ |」下方 Status 欄改為「implemented (A5)」；底部 Errors 表的 `config.not_found` / `config.malformed` / `config.key_not_found` / `state.etag_mismatch` 四條改為 `implemented (A5)`；驗證：grep `implemented (A5)` 在 operations.md 至少 6 次（兩 op × 三 error）
- [x] 11.2 在 `doc/speclink-design.md` §11 章節（Config 結構）加 cross-reference subsection 指向 A5 落地內容：etag 公式 `v<version>.<sha256[:12]>`、external_edit reconcile 流程、state.db v5 兩表 schema、walking-skeleton fallback semantics；對應 design decision「State.db v5 cache vs YAML SOT 取捨」；驗證：grep `state.db v5` 與 `external_edit` 在 §11 章節下各出現至少 1 次
- [x] 11.3 在 `doc/speclink-design.md` §18.1 MVP scope checklist 對 `config.read` / `config.write` 兩條既有 item 補上「✅ Implemented (A5)」標記；驗證：grep `Implemented (A5)` 在 §18.1 至少 2 次

## 12. End-to-end demo path（walking skeleton showcase）

- [x] 12.1 [P] 在 `crates/cli/tests/walking_skeleton_e2e.rs`（既有檔案、A3 / A4 已建）加一條 end-to-end 測試：fresh init → create change → write all artifacts → DAG 自動 proposing → ready → apply start → in_progress → 改 `speclink config set rules.require_code_review true` → 最後一條 task done → 看到 in_progress → code_reviewing（不再走捷徑進 archived candidate）；驗證 `archive` op 此時被拒、回 hint「code review pending」（A5 不接 `add-review` slice、預期 archive 仍會 reject、error 為 `state.transition_invalid` 而非 `change.code_review_pending`，後者由 review slice 接）；對應 spec scenario「Setting require_code_review=true holds in_progress through code_reviewing」；驗證：紅燈 → 綠
- [x] 12.2 在 `crates/cli/tests/walking_skeleton_e2e.rs` 加 happy-path 測試：fresh init → end-to-end 跑完 walking-skeleton 4-state path（require_*_review 都是 default false）→ archive 成功；證明 A5 沒破 A4 既有路徑；驗證：和 A4 既有 archive happy path 對齊、`cargo test -p speclink-cli walking_skeleton_e2e_happy_path` 綠

## 13. 全域驗證

- [x] 13.1 全 workspace 跑 `cargo fmt --check`、`cargo clippy --workspace -- -D warnings`、`cargo test --workspace`；驗證：CI matrix（Linux / macOS / Windows × stable Rust）皆綠
- [x] 13.2 跑 `spectra analyze add-config-rw --json`，遞補/修正所有 Critical / Warning finding；驗證：第二次跑 analyze 後 critical/warning 為 0、或剩餘為「Suggestion」級別
- [x] 13.3 跑 `spectra validate add-config-rw`、確認 strict 模式通過；驗證：exit 0
- [x] 13.4 跑 `spectra verify add-config-rw`（AI 三維度 QA review，可選 skill），對 Completeness / Correctness / Coherence 三維度過關；驗證：手動 QA 確認 / 文件記錄
