## Why

speclink 已改為與 OpenSpec 單向相容（只進不出），但規格仍放在 `openspec/` 資料夾、每個變更的資料檔仍叫 `.openspec.yaml`，名稱與產品對不上；OpenSpec 專案也沒有乾淨的匯入路徑——在 OpenSpec 專案執行 `speclink init` 會停在 Already initialized，加 `--force` 會把 config.yaml 蓋回範本。本變更是討論 positioning-and-process-cost 分刀計畫的第 7 刀，在 propose 時拆成兩個變更：本變更處理引擎、CLI、server、Node SDK、技能與文件；desktop-spec-dir-migration 處理桌面 app。正式規格裡其餘舊路徑的改寫由 spec-dir-path-rewrite 處理，本 repo 自己的遷移由 self-spec-dir-migration 處理。依使用者決定，本變更在第一批六刀發版之後才封存，讓改名落在下一版。

目標使用者與情境：透過 AI agent 開發的開發者（團隊情境下含 PO 與 PM）——既有 speclink 專案在升級後執行 `speclink update` 時自動遷移；OpenSpec 使用者以 `speclink init` 匯入；遠端模式的 AI agent 讀取 Context Projection。

## What Changes

- 規格資料夾預設改為 `speclink/`；引擎早有的 `.speclink.yaml` `spec_dir` 設定照常優先。專案根的判定同時認得 `speclink/`、舊的 `openspec/`、`.speclink.yaml` 與殘留的連接檔。
- 變更資料檔改名為 `.meta.yaml`：讀取先找 `.meta.yaml`、沒有才讀 `.openspec.yaml`；只寫新檔名，寫入時移除舊檔；遠端資料庫不搬，已封存變更的舊鍵靠讀取順序讀到，新封存寫 `.meta.yaml`。檔名集中一處定義。
- 舊佈局照常可用：沒寫 `spec_dir`、只有 `openspec/` 的專案，新版照舊讀 `openspec/`，每次執行在 stderr 印一行遷移警告；`speclink/` 與 `openspec/` 並存時印另一行警告（`openspec/` 可能是舊版 CLI 建的）。
- `speclink update` 自動遷移：符合偵測條件（有 `.speclink.yaml` 且沒有啟用的 `spec_dir`、有 `openspec/`、沒有 `speclink/`）時，把資料夾與所有變更（進行中與已封存）的 `.openspec.yaml` 改名，內容不改寫；有其他 worktree、`openspec/` 有未提交修改或在 linked worktree 裡時暫緩並說明原因；不寫入 `spec_dir`；不 commit；完成後提示要 commit 的檔案、團隊成員先升級再 pull、自己的腳本／CI／.gitignore 裡的 `openspec/` 要自己改。
- `speclink init` 遇到「有 `openspec/`、沒有 `.speclink.yaml`」時自動匯入：與 update 共用搬移程式，保留 config.yaml、schemas/、正式規格與變更，補上 `.speclink.yaml`、`.gitignore` 設定與技能檔；有其他 worktree 或未提交修改時直接拒絕；完成後提示 OpenSpec CLI 從此讀不到、OpenSpec 自己的技能與指令檔由使用者自行刪除。
- 遠端 Context Projection 的鏡像路徑改為 `speclink/…`；CLI 改從快照本身取得鏡像根目錄，不再寫死，讓新 CLI 也能讀舊 server 的投影。
- 技能原稿裡寫死的 `openspec/` 改用 `{{SPEC_DIR}}`、`.openspec.yaml` 改為 `.meta.yaml`；技能以專案實際使用的規格資料夾渲染（暫緩中的專案仍渲染成 `openspec/`）。
- 錯誤訊息裡的 metadata 檔路徑改為 `<實際規格資料夾>/changes/<name>/.meta.yaml`（遠端用 `speclink/`），Node SDK 的預設 `spec_dir` 改用新預設；manual 技能說明裡的資料夾也依實際規格資料夾渲染；內建 schema（spec-driven、no-spec-change）的 instruction 與 proposal 範本裡寫死的 `openspec/specs/`、`openspec/config.yaml` 改為 `speclink/`；`.speclink.yaml` 範本的「OpenSpec directory path」註解改為規格資料夾的說明。
- 文件：README（中英）寫明可從 OpenSpec 專案匯入（單向）；入門教學與設定說明（中英）寫明預設資料夾、遷移與匯入。

## Capabilities

### New Capabilities

- `spec-dir-migration`: 規格資料夾的預設位置、舊佈局相容、`update` 自動遷移與 `init` 匯入 OpenSpec 專案。最接近的 store-abstraction 只承諾預設佈局的檔案形狀、workspace-tools 只管技能同步、remote-connection 的殘留檔警告只涵蓋連接檔，都沒有涵蓋資料夾遷移與匯入。

### Modified Capabilities

- `store-abstraction`: 「預設檔案系統佈局不變」改名為「預設檔案系統佈局」，佈局改為 `speclink/` 與 `.meta.yaml`，並載明舊檔名的讀取順序。
- `context-projection`: 「投影佈局與 manifest」的鏡像佈局改為 `speclink/`，鏡像根目錄由快照路徑決定。
- `user-documentation`: 「使用者文件載明本地產物的純文字可攜性」改為載明單向匯入與新的資料夾、檔名。

## Impact

- Affected specs: `spec-dir-migration`, `store-abstraction`, `context-projection`, `user-documentation`
- Affected code:
  - New: crates/engine/speclink-core/src/workspace/migrate.rs
  - Modified: crates/engine/speclink-core/src/workspace/workspace.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/mod.rs, crates/engine/speclink-core/src/workspace/config.rs, crates/engine/speclink-core/src/workspace/instructions.rs, crates/engine/speclink-core/src/workspace/skills.rs, crates/engine/speclink-fs/src/layout.rs, crates/engine/speclink-fs/src/lib.rs, crates/engine/speclink-core/src/lifecycle/model.rs, crates/engine/speclink-core/src/lifecycle/plan.rs, crates/engine/speclink-core/src/lifecycle/change_schema.rs, crates/engine/speclink-core/src/command/mod.rs, crates/engine/speclink-core/src/quality/analyzer.rs, crates/engine/speclink-core/src/quality/station.rs, crates/engine/speclink-core/src/lifecycle/archive.rs, crates/engine/speclink-core/src/lifecycle/discard.rs, crates/engine/speclink-core/src/lifecycle/discuss.rs, crates/engine/speclink-core/src/lifecycle/newcmd.rs, crates/engine/speclink-core/src/lifecycle/tasks.rs, crates/engine/speclink-core/src/store.rs, crates/host/speclink-host/src/worktree.rs, crates/host/speclink-host/src/projection.rs, crates/adapters/speclink-node/index.d.ts, crates/adapters/speclink-cli/src/common.rs, crates/adapters/speclink-cli/src/main.rs, crates/adapters/speclink-cli/src/verbs/init.rs, crates/adapters/speclink-cli/src/verbs/instructions.rs, crates/adapters/speclink-cli/src/verbs/query.rs, crates/adapters/speclink-cli/src/verbs/plan.rs, crates/adapters/speclink-node/src/render.rs, crates/adapters/speclink-node/src/lib.rs, crates/host/speclink-host/src/bridge.rs, crates/host/speclink-host/src/drift.rs, crates/host/speclink-server/src/api/context.rs, crates/host/speclink-server/src/api/read_api.rs, apps/desktop/src-tauri/src/remote.rs, apps/desktop/core/src/manage.rs, apps/desktop/core/src/discussions.rs, apps/desktop/core/src/settings.rs, apps/desktop/core/src/testfixture.rs, crates/engine/speclink-core/assets/skills/commit.md, crates/engine/speclink-core/assets/skills/apply.md, crates/engine/speclink-core/assets/skills/discuss.md, crates/engine/speclink-core/assets/skills/propose.md, crates/engine/speclink-core/assets/skills/ingest.md, crates/engine/speclink-core/assets/skills/archive.md, crates/engine/speclink-core/assets/skills/drift.md, crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml, crates/engine/speclink-core/assets/schema/spec-driven/proposal.template.md, crates/engine/speclink-core/assets/schema/no-spec-change/fork.schema.yaml, crates/engine/speclink-core/assets/schema/no-spec-change/proposal.template.md, crates/engine/speclink-core/tests/golden/assets.lock, README.md, README.zh-TW.md, docs/getting-started.md, docs/getting-started.zh-TW.md, docs/configuration.md, docs/configuration.zh-TW.md
- 影響的 crate／app：speclink-core（偵測、遷移、匯入、預設值、訊息）、speclink-fs（檔名與讀取順序）、speclink-cli（警告、init／update 輸出、投影根目錄）、speclink-host 與 speclink-server（遠端路徑與舊鍵讀取）、speclink-node（預設值）；apps/desktop 由 desktop-spec-dir-migration 處理，本變更只連動 remote.rs 建構 `MetaError` 的一處，以及桌面測試裡讀寫變更資料檔名的夾具與期望值（apps/desktop/core 的 manage.rs、discussions.rs、settings.rs、testfixture.rs 只改測試）。
- CLI：沒有新指令或旗標。`speclink update` 與 `speclink init` 多出遷移與匯入的行為和輸出；所有指令在舊佈局下於 stderr 多一行警告（stdout 與 exit code 不變）。
- 設定欄位：`.speclink.yaml` 的 `spec_dir` 預設值由 `openspec` 改為 `speclink`，遷移與匯入都不寫入此欄位；`.openspec.yaml` 改名為 `.meta.yaml`，欄位不變。
- 生成的技能：commit、apply、discuss、propose、ingest、archive、drift 七份原稿改變，所有用到 `{{SPEC_DIR}}` 的技能在新預設下渲染成 `speclink/`；ASSET_VERSION 由 v1.52.0 升為 v1.53.0，golden snapshot 與 assets.lock 再生。
- 相容性影響：**BREAKING**——新專案的規格資料夾與變更資料檔名改變；舊版 CLI 遇到已遷移的專案會看到空專案（刻意讓錯誤明顯），團隊須一起升級；新 CLI 可讀舊 server 的投影，舊 CLI 讀不到新 server 的投影路徑，server 與 CLI 須一起升級；golden 依新預設再生，差異只有 `{{SPEC_DIR}}` 渲染結果與七份原稿的改動。
