## Why

工作區產物同步層（`speclink-core` 的 init／update／probe）只有兩個核心概念——「哪些工具要生成」（工具選集）與「每個工具要生成哪些技能檔」（受管技能集合）——但兩個概念都沒有單一擁有者，各在四個函式裡重算一次。已有兩次漏補實證：worktree 政策開關落地時，探測那一份漏補門檻過濾，政策關閉的專案被永久報成「檔案缺失」；技能改名時為了清孤兒目錄只能再加第四份集合計算。這次把兩個概念各收成一份「同步計畫」，讓守門、生成、清理、探測都只消費同一份計畫，結構上不可能再漂移。

來源討論：improve-workspace-sync（刀 A）。目標使用者是透過 Claude／Codex 跑 SDD 的開發者，對應 `speclink init`／`speclink update`、desktop 開專案時的技能檔過期提示、設定頁的工具切換與 remote checkout 綁定——全部經過這一層。

## What Changes

零使用者可觀察行為變化的重構，影響 crate：`speclink-core`（主體）、`speclink-cli`（改 use 路徑）、`apps/desktop/src-tauri`（一處改讀選集、兩處改 use 路徑）。

- **立同步計畫（候選 1＋2）**：`crates/speclink-core/src/init.rs` 新增 `ToolSelection`（由 `.speclink.yaml` 解析：內建去重、描述子驗證與重名拒絕、未知內建名警告、tools 空清單時的 `.claude` 目錄回退；init 時可由記憶體選集直接建）與 `managed_skills`（依 target 算出 `speclink-<name>/SKILL.md` → 內容的預期檔案表；for_codex 子集、worktree 門檻、壞 config 保留技能的安全方向都在這一支）。兩者組成 `SyncPlan`：一組 target，各帶 skills_root、預期檔案表、遺留剝除的指令檔路徑。update ＝ resolve → guard → apply；probe ＝ resolve → diff（只讀內建子集，與今天相同）；reconcile ＝ resolve（新選集）→ guard → 寫設定檔 → apply。worktree 政策一次同步只讀一次 `openspec/config.yaml`（今天每個消費端各讀一次）。
- **刪五支重複函式**：`generate_tool`、`generate_custom`、`expected_skill_dirs`、`differing_managed_files`、`skill_targets`——工作被計畫的 apply／diff／guard 吸收，不是改寫成轉發。`reconcile_builtin_tools` 裡「提前重算守門目標」的手工段落消失，守門目標＝計畫的 skills_root 集合。
- **desktop 改讀選集**：`apps/desktop/src-tauri/src/connections.rs` 的 checkout 預選函式改用 core 的 `ToolSelection` 取內建選集，刪掉自寫的 tools 解析迴圈；footprint 回退行為不變。
- **`.speclink.yaml` 讀改寫收攏（候選 5）**：`write_remote_section`／`remove_remote_section` 從 `init.rs` 搬到 `crates/speclink-core/src/config.rs`，與 `update_app_config_tools_text` 同住、共用既有的 `parse_yaml_mapping`；刪 `init.rs` 的 `load_app_yaml_doc`（兩者邏輯與錯誤訊息逐字相同）。三個消費端只改 use 路徑。
- **skills.rs 兩軌渲染合一（候選 4）**：`render_skill_file`／`render_skill_file_custom` 合成一支由 `RenderTarget` 分支的 render；`substitute`／`substitute_neutral` 合成一張代換表。target 只決定三個差異點：Claude 專屬 frontmatter 行、前言（fork 規則 vs invocation 說明）、代換表內容。輸出位元級不變。

**相容性影響**：無。CLI 人眼輸出與 `--json` shape 不變；技能檔 render 輸出位元級不變，`ASSET_VERSION` 不 bump、四份 golden 與 `assets.lock` 不再生，golden 全綠就是本次重構的驗收；`.speclink.yaml`／`openspec/config.yaml` 無欄位新增或變更；影響的技能與工具（claude／codex／自訂描述子）之生成集合與內容零變化。Node SDK 的 `skills.render()` 走 `render_skill_file_for`，公開 API 不動。

## Non-Goals

Non-Goals 寫在 design.md 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無——規格層行為不變，本次只改實作結構。討論已把兩條規格變更——「清孤兒擴及所有再生入口」與「探測涵蓋描述子」——歸給刀 B，不在本變更。）

## Impact

- Affected specs: 無。步驟 3 掃描結果：相關規格為 workspace-tools（工具檔生成、描述子生命週期、中性渲染、update 清孤兒、降級守門、技能檔過期探測——全部是本層的既有契約，本次逐條維持，以其既有測試與 golden 為回歸對照）；skill-routing 與 workflow-config 只是鄰接，不受影響。
- Affected code:
  - Modified:
    - `crates/speclink-core/src/init.rs`（新增 ToolSelection／managed_skills／SyncPlan；update／probe_assets／reconcile_builtin_tools／adopt／init 改消費計畫；刪五支函式與 load_app_yaml_doc；remote section 兩支搬走）
    - `crates/speclink-core/src/config.rs`（承接 write_remote_section／remove_remote_section）
    - `crates/speclink-core/src/skills.rs`（render 與 substitute 合一）
    - `crates/speclink-core/tests/it/render_golden.rs`（一處 substitute 呼叫改走合一後的介面；golden 檔本身不動）
    - `apps/desktop/src-tauri/src/connections.rs`（預選改讀 ToolSelection；write_remote_section use 路徑）
    - `apps/desktop/src-tauri/src/remote.rs`（write_remote_section use 路徑）
    - `crates/speclink-cli/src/verbs/connection.rs`（write_remote_section／remove_remote_section use 路徑）
  - New: 無
  - Removed: 無（只刪函式，不刪檔）
- 測試回歸面：`cargo test -p speclink-core`（init.rs 55 個單元測試＋`--test it render_golden`）、`cargo test -p speclink-cli --test it`（init／update／connection 相關）、`cargo test -p speclink-desktop`（connections／project 相關）
