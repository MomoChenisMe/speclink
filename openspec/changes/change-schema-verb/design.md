## Context

`change depends` 是最接近的前例：speclink-core 的 `Command::ChangeDepends` 由 `run_change_depends` 先做 not-found，再呼叫 lifecycle/plan.rs 的 `set_depends` 以 `KeyLines` 文字手術寫 `.openspec.yaml`；成功發出 `DomainEvent::ChangeDependsChanged`，冪等時不發事件。CLI 的 `ChangeCommands` 定義在 crates/adapters/speclink-cli/src/verbs/plan.rs，遠端走 speclink-remote 的 client 方法與 speclink-protocol 的 `SetDependsRequest`；server 在 app.rs 掛 `POST /changes/{name}/depends`，routes.rs 經 Command gateway 直通；speclink-host 的 bridge.rs 與 commit.rs 為 outcome 命名與事件提交。worktree 映射由既有的 overlay 讀寫分岔處理，`change depends` 已遵循。

`new_artifact`（lifecycle/newcmd.rs）收到變更解析後的 `Schema`，對 schema 沒定義的 artifact 現在照樣寫檔。驗證章欄位由 quality/station.rs 的 `write_stamp` 寫入，前綴為 `verified`；驗證工單是變更內的 `verify.md`。no-spec-change-schema（依賴的前一刀）提供內建登記表與 verify 技能的行為檢查。

## Goals / Non-Goals

**Goals:**

- 變更建立後可以雙向切換 schema，切換不會留下與內容矛盾的狀態。
- 本機與遠端同語意，訊息逐字相同。
- ingest 與 verify 技能在需要時走這個指令。

**Non-Goals:**

- 桌面 app 不新增切換入口。
- 不在切換時自動新增、刪除或改寫任何 artifact。
- 遠端不支援自訂 schema（沿用 desktop-config 的遠端內建限縮）。
- 不改審查章與審查工單。

## Decisions

### D1：命令與核心函式

新增 `Command::ChangeSchema { name, schema }`、`CommandOutcome::ChangeSchema(SchemaOutcome { change, schema, previous, changed, verify_stamp_cleared })` 與 `DomainEvent::ChangeSchemaChanged`（kebab 名 `change-schema-changed`）。守門與寫入放在新檔 lifecycle/change_schema.rs 的 `set_schema`，`run_change_schema` 只做 not-found 與錯誤分類，與 `run_change_depends` 同形。替代方案「放進 lifecycle/plan.rs」被否決：plan.rs 的職責是執行順序，schema 與它無關。

### D2：守門順序與 schema 解析

順序固定為：作用中變更（名稱逐字比對）→ metadata 可解析 → schema 可解析 → 切到沒有 specs artifact 的 schema 時沒有 delta（以 `store.delta_capabilities` 判定並列出）→ 沒有未結驗證工單（`verify.md` 存在即視為未結）。schema 解析由呼叫端注入解析函式：`run_change_schema` 以執行情境的 workspace 與 user 層目錄組成 `resolve_with(ws, user_dir, name)`；fs 模式兩者都有，server 的執行情境（speclink-host bridge 的 `engine_context`）兩者都沒有，等於 `resolve_with(None, None, name)`（只有內建）。替代方案「核心自己讀 user 層」被否決：會讓 server 讀到伺服器本機的目錄，違反遠端只看內建的規則。

### D3：一次寫入完成切換與清除驗證章

`set_schema` 以 `KeyLines` 對原文做一次手術：取代或新增 `schema:` 行；若帶驗證章，同一份文字移除 `verified_at`、`verified_by`、`verified_with`、`verified_tasks_total`、`verified_scope`（含其清單行），保留 `verified_rounds`；最後一次寫回。寫入失敗時檔案維持原樣（單一寫入），沒有半套狀態。與現值相同時不寫檔、不發事件。理由：驗證章是依舊 schema 的檢查標準蓋的，切換後已不代表新標準下的結果；審查章看工藝品質，與 schema 無關。

### D4：遠端端點

speclink-protocol 新增 `SetSchemaRequest { schema }` 與 `SetSchemaResponse { change, schema, previous, changed, verify_stamp_cleared }`（`verify_stamp_cleared` 讓 CLI 遠端模式印出與 fs 模式相同的清章行，CLI `--json` 仍是四個欄位）；speclink-remote 新增對應 client 方法；server app.rs 掛 `POST /changes/{name}/schema`，routes.rs 經 Command gateway 直通（解析只看內建，見 D2）；events.rs 把 `change-schema-changed` 歸入 change 範圍的失效通知；bridge.rs 命名 outcome 為 `change-schema`，commit.rs 處理新事件。錯誤沿用既有分類：not-found 為 404，守門失敗為 409 refused、message 為引擎原文。

### D5：CLI 子指令與模式

verbs/plan.rs 的 `ChangeCommands` 新增 `Schema { name, schema, json }`，在模式分岔的單點宣告為兩種模式皆可用；fs 模式呼叫 core 命令，遠端模式呼叫 client 方法；兩者共用同一個輸出函式，產生 `✓ <name> schema: <old> → <new>`、`(unchanged)` 與驗證章清除的第二行，`--json` 為 `{ change, schema, previous, changed }`。

### D6：拒絕在沒有 specs 的 schema 寫 delta

`new_artifact` 在命名主閘之前，若 `kind` 為 spec 且變更的 schema 沒有 id 為 `specs` 的 artifact，回傳 Refusal（schema 解析不到時 `run_new_artifact` 傳入 `None`，維持現行的盡力寫入）：`change '<name>' uses schema '<schema>', which has no specs — switch it first: speclink change schema <name> spec-driven`。遠端 `new artifact` 走 server 的同一個核心函式，訊息一致。

### D7：技能與版號

ingest.md 的「Update artifacts」步驟開頭加入讀取 `schemaName` 與切換規則；verify.md 把前一刀寫的行為改變建議改為四步修正（verify discard → change schema → ingest → 重新 verify），並寫明不建議 `--accept`。不新增大寫強調字。ASSET_VERSION 由 v1.50.0 升為 v1.51.0，再生 golden、assets.lock 與 repo 自用技能檔。

## Implementation Contract

- **CLI 行為**：`speclink change schema` 依 workflow-schemas 新需求的九個情境成立（切換、JSON、有 delta 拒絕、解析不到拒絕、未結驗證工單拒絕、清除驗證章、同值不寫、不存在、worktree 副本）。驗收：speclink-core 的 change_schema 單元測試與 `cargo test -p speclink-cli --test it` 的整合測試。
- **寫 delta 拒絕**：no-spec-change 變更的 `new artifact spec` 非零結束且零寫入，spec-driven 照常。驗收：newcmd 測試與 CLI 測試。
- **遠端**：`POST /changes/{name}/schema` 依 server-verb-api 新需求成立，訊息與 fs 模式逐字相同。驗收：`cargo test -p speclink-server` 的端點測試，以及 crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs 的兩模式比對。
- **技能**：生成的 speclink-ingest 含切換規則；生成的 speclink-verify 含四步修正。驗收：render_golden 新測試與 skill_authoring。
- **範圍內**：上列 crate、兩份技能原稿、四份文件與版號再生物。**範圍外**：桌面入口、自動改寫 artifact、遠端自訂 schema。

## Risks / Trade-offs

- [回歸對照：Command／Outcome 列舉新增成員] → typed.rs 的對應表與 bridge.rs 命名需同批補齊，否則編譯失敗而非靜默；既有 golden 與 CLI 測試不應變動。
- [回歸對照：`new artifact spec` 新拒絕] → 只在 schema 沒有 specs artifact 時觸發；既有測試全用 spec-driven，不受影響。
- [清除驗證章讓已蓋的章消失] → stdout 第二行明講，並保留 `verified_rounds`，流程成本資料不遺失。
- [跨平台：CRLF 的 meta] → 沿用 `KeyLines` 的行尾保留，加一個 CRLF 測試。
- [遠端：舊版 server 沒有新端點] → 請求以錯誤結束、server 端零寫入；發版說明提醒 server 與 CLI 一起升級。

## Migration Plan

不需要遷移。新指令與新端點只在使用時生效。回滾時，已切成 no-spec-change 的變更在舊版會回報 schema 找不到（同前一刀）。

## Open Questions

(none)
