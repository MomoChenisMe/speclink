## Why

no-spec-change-schema 讓 AI 在 propose 時為每個變更選 spec-driven 或 no-spec-change，但變更建立後沒有指令可以改 schema（OpenSpec 也沒有）。AI 選錯、或做到一半發現其實改到了正式規格承諾的行為時，只能手改 `.openspec.yaml`，或廢棄變更重開——已勾的任務與 evidence 要重來。沒有切換路徑，「選錯 schema」的代價就很高，變更上的 schema 也會和內容對不上（no-spec-change 變更裡出現 delta）。本變更是討論 positioning-and-process-cost 分刀計畫的第 4 刀，依賴第 3 刀 no-spec-change-schema。

目標使用者與情境：透過 AI agent 開發的開發者（團隊情境下含 PO 與 PM）。ingest 階段帶入的新需求改到承諾的行為時，由 ingest 技能切回 spec-driven 再補 delta；verify 階段發現 no-spec-change 變更改到行為時，依建議的步驟切換；使用者也可以自己執行指令。

## What Changes

- 新增 `speclink change schema <name> <schema> [--json]`，雙向切換作用中變更的 schema，只改 `.openspec.yaml` 的 `schema:` 一行，與 `change depends`、`change rank` 同一族；本機與遠端兩種模式都可用，遠端走新的 `POST /changes/{name}/schema`。
- 守門依序：變更須為作用中；metadata 須可解析；`<schema>` 須解析得到（專案層、user 層或內建）；切到沒有 specs 產出的 schema（如 no-spec-change）時，變更裡不得有 delta spec；變更有未結的驗證工單時拒絕，提示先放棄驗證工單。任一守門失敗為非零 exit code 且零寫入。切到同一個 schema 時成功但不寫檔。
- 切換成功時若變更帶著驗證章，同一次寫入移除驗證章欄位（`verified_rounds` 保留），因為驗證章是依舊 schema 的標準判定的；審查章不動。
- 對 schema 沒有 specs 產出的變更執行 `speclink new artifact spec ...` 時拒絕並零寫入，訊息提示先用 `speclink change schema <name> spec-driven` 切換。
- ingest 技能：新需求會改到正式規格承諾的行為、而變更是 no-spec-change 時，先切回 spec-driven，再在 proposal 補 Capabilities、寫 delta spec。
- verify 技能：no-spec-change 變更的行為改變 WARNING 改為具體的修正步驟——放棄驗證工單、切回 spec-driven、以 ingest 補 delta、重新驗證。
- 文件：verb-contract 與 product-status（中英）登記新指令與新端點。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-schemas`: 新增 `change schema` 動詞切換變更 schema 的規則，以及 schema 沒有 specs 產出時拒絕寫入 delta spec。
- `server-verb-api`: 新增 `POST /changes/{name}/schema` 寫入端點。
- `ingest-skill`: 新增「新需求改到承諾行為時切回 spec-driven」的步驟。
- `verify-skill`: 新增不改規格變更改到行為時的修正步驟。

## Impact

- Affected specs: `workflow-schemas`, `server-verb-api`, `ingest-skill`, `verify-skill`
- Affected code:
  - New: crates/engine/speclink-core/src/lifecycle/change_schema.rs
  - Modified: crates/engine/speclink-core/src/command/mod.rs, crates/engine/speclink-core/src/command/typed.rs, crates/engine/speclink-core/src/lifecycle/mod.rs, crates/engine/speclink-core/src/lifecycle/newcmd.rs, crates/engine/speclink-core/src/workspace/schema.rs, crates/engine/speclink-core/src/quality/station.rs, crates/engine/speclink-core/src/quality/validate.rs, crates/engine/speclink-core/src/teststore.rs, crates/adapters/speclink-cli/src/verbs/plan.rs, crates/protocol/speclink-protocol/src/command.rs, crates/protocol/speclink-remote/src/client.rs, crates/host/speclink-server/src/app.rs, crates/host/speclink-server/src/api/routes.rs, crates/host/speclink-server/src/api/events.rs, crates/host/speclink-host/src/bridge.rs, crates/host/speclink-host/src/commit.rs, crates/engine/speclink-core/assets/skills/ingest.md, crates/engine/speclink-core/assets/skills/verify.md, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/tests/golden/assets.lock, docs/verb-contract.md, docs/verb-contract.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md
- 影響的 crate／app：speclink-core（命令、守門與寫入、artifact 寫入拒絕）、speclink-cli（`change schema` 子指令與模式分岔）、speclink-protocol 與 speclink-remote（請求／回應型別與 client 方法）、speclink-server 與 speclink-host（端點、事件）；apps/desktop 沒有新入口，切換後的 schema 經既有讀取路徑呈現。
- CLI：子指令 `speclink change schema <name> <schema>`；旗標 `--json`；不讀 stdin。成功 exit code 0，stdout 人眼為一行 `✓ <name> schema: <old> → <new>`（同 schema 時為 `✓ <name> schema: <schema> (unchanged)`），`--json` 為 `{ "change": string, "schema": string, "previous": string, "changed": boolean }`；守門失敗 exit code 非零、stderr 一行原因、零寫入。
- 設定欄位：只改既有的 `.openspec.yaml` `schema:` 值；沒有新增欄位。
- 生成的技能：speclink-ingest 與 speclink-verify 的技能檔（claude、codex、copilot）內容改變；ASSET_VERSION 由 v1.50.0 升為 v1.51.0，golden snapshot 與 assets.lock 再生。
- 相容性影響：新增指令與端點，既有指令輸出不變；`new artifact spec` 對 schema 沒有 specs 產出的變更改為拒絕（之前會寫入檔案）。
