## Why

刀 A（workspace-sync-plan，已封存）把工作區產物同步收成一份 `SyncPlan`，但 `init`／`init_remote` 還保留一條專用寫入路徑 `write_skills`：只寫檔、不清孤兒目錄、不清下架工具的足跡、不重設自訂足跡狀態，並自己再跑一次遺留剝除。結果是「`init --force` 切換工具」會留下前一個工具的 `speclink-*` 技能目錄可被載入——這正是 unify-agent-tool-bootstrap 設計否決「只追加不清理」的理由（設定檔與可載入技能分歧），init 卻是唯一還這樣做的入口。另一個未收尾的缺口：技能檔過期探測明文「第一版不涵蓋描述子」，描述子的技能檔缺失或過期時 desktop 不會提示、`differingFiles` 不列，但 `update` 會寫它們。本變更把五個入口全部收到同一份計畫的 `apply`，並讓探測涵蓋計畫裡的每個 target。來源討論 improve-workspace-sync 的刀 B。

目標使用者：透過 Claude／Codex 或自訂 harness 跑 SDD 的開發者。情境：`speclink init`／`init --force`／`init --store remote`、desktop 開專案時的技能檔提示、以及自訂描述子專案的技能維護。影響 crate：`speclink-core`（主體）；`speclink-cli` 與 `apps/desktop` 零程式碼變更（行為經 core 改變）。

## What Changes

- **init 與 init_remote 改走 `SyncPlan::apply`**：兩個入口的流程變為「已初始化守門 → 計畫守門 → store_init（僅 fs）→ 寫 `.speclink.yaml`（維持樣板重寫語意）→ gitignore → `apply`（→ remote section，僅 remote）」。`SyncPlan::write_skills` 刪除，init 自己的遺留剝除迴圈刪除（apply 內含）。`reconcile_builtin_tools` 的 doc comment「CLI init、remote init 與 desktop 共用的單一入口」改寫為事實：五個入口共用的是計畫的 `apply`。
- **`init --force` 開始清理**（使用者可觀察的行為變化）：切換工具時移除被下架內建工具的 `speclink-*` 技能目錄；同一工具下不屬於本次生成集合的 `speclink-*` 目錄（改名技能的舊目錄、政策關閉後的 worktree 技能）一併移除；`.speclink/generated-tools.yaml` 的自訂足跡狀態隨樣板重寫歸零。兩個內建指令檔的遺留 `SPECLINK:START..END` 區塊無論選取與否都剝（今天 init 只剝選中的工具）。不帶 `--force` 的 init 只對「無 `.speclink.yaml`、無 spec 目錄、但有殘留技能目錄」的目錄產生差異：殘留檔一律改寫為現版（今天靜默保留）。
- **探測涵蓋描述子**：`probe_assets` 對計畫中每個 target（內建與描述子）讀版號、判方向、聚合五態；`differingFiles` 含描述子 skills_dir 下的差異檔（路徑為描述子的 `skills_dir` 加 `speclink-<name>/SKILL.md`）。`tools[].tool` 對描述子回描述子的 `name`。無法通過驗證的描述子不在計畫內，探測結果不受其影響（與今天相同）。
- **規格更新**：`workspace-tools` 兩條 requirement 改寫——「update 清除孤兒技能目錄」主詞放寬為所有再生入口（含 `speclink init --force` 與 desktop 的 init／adopt）；「技能檔過期探測」的判定面從內建工具放寬為 tools 清單內的內建工具與有效描述子。

描述子驗證收緊是新增的拒絕面：既有 `.speclink.yaml` 若把描述子的 skills_dir 寫成 `/`、`./` 或內建工具的 skills 目錄，升級後 `speclink update` 會以單行錯誤拒絕並零寫入，改成一個專屬目錄即可。結尾斜線的寫法照常接受，只是統一削去。

**相容性影響**：

- `speclink init` 的 stdout／stderr、exit code 與 `.speclink.yaml`／`openspec/` 內容不變；差異只在檔案系統的技能目錄集合：`init --force` 之後只剩本次選集應有的 `speclink-*` 目錄。既有使用者若靠 `init --force` 保留另一個工具的技能檔，改用 `speclink update` 或 `--tools claude,codex` 同時選兩者。
- `probe_assets` 的 `--json`／IPC 形狀不變（`status`、`currentVersion`、`tools[]`、`differingFiles`）；有描述子的專案可能多出 `tools[]` 項目與 `differingFiles` 路徑，狀態可能從「現版」變「缺失」或「過期」——desktop 隨即依既有提示面顯示（提示文案只用狀態與檔案數，不顯示工具名）。無描述子的專案輸出逐位元不變。
- 技能檔 render 輸出、`ASSET_VERSION`、golden、`assets.lock` 不動。`.speclink.yaml`／`openspec/config.yaml` 無欄位變更。CLI 無新旗標。
- 回歸對照：`crates/speclink-core/src/init.rs` 既有測試中 `init_force_over_a_legacy_workspace_strips_the_marker`、`init_force_refuses_a_workspace_that_leads_the_engine`、`reconcile_matches_init_output_for_the_same_selection` 維持；`crates/speclink-cli/tests/it/init_tools.rs` 全部維持（它們都在空目錄或單次 init 上斷言）。

## Non-Goals

Non-Goals 寫在 design.md 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `workspace-tools`：「update 清除孤兒技能目錄」requirement 的主詞從 `speclink update` 放寬為所有再生入口，補 `init --force` 切換工具與改名技能兩個 scenario；「技能檔過期探測」requirement 的判定面納入有效描述子，補描述子缺失與描述子過期兩個 scenario。

## Impact

- Affected specs: workspace-tools（三條 MODIFIED requirement：update 清除孤兒技能目錄、技能檔過期探測、tools 自訂描述子的接受與驗證；步驟 3 掃描的其他鄰接規格 skill-routing、workflow-config、desktop-config 不受影響）
- Affected code:
  - Modified:
    - `crates/speclink-core/src/init.rs`（init／init_remote 改走 apply、刪 write_skills 與 init 的剝除迴圈、probe_assets 與 SyncPlan::differing_files 涵蓋描述子、連帶移除孤兒化的 SyncTargetKind 與 SyncPlan.worktree_on、doc comment 更新、測試補齊）
    - `crates/speclink-core/src/config.rs`（`ToolDescriptor::validate` 於邊界削去 skills_dir 結尾斜線，並拒絕正規化後等同專案根或等同內建工具 skills 目錄者；比對走路徑正規化，等價拼法一併擋下）
    - `crates/speclink-cli/tests/it/init_tools.rs`（補「先 init claude 再 --force --tools codex」的 stdout 兩行與足跡清理斷言，以及兩條描述子目錄拒絕的零寫入斷言）
    - `openspec/specs/workspace-tools/spec.md`（經 delta 於封存時合併）
  - New: 無
  - Removed: 無
- 零變更但受行為影響的消費端（測試要跑）：`crates/speclink-cli/src/verbs/init.rs`（輸出不變）、`apps/desktop/core/src/project.rs`（init_project_at／adopt_project_at／probe_assets_at）、`apps/desktop/src-tauri/src/connections.rs`（bind_checkout 走 reconcile，不變）
- 測試回歸面：`cargo test -p speclink-core`（init.rs 單元測試）、`cargo test -p speclink-core --test it render_golden::`、`cargo test -p speclink-cli --test it init_tools`、`cargo test -p speclink-desktop-core project::`
