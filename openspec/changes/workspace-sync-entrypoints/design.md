## Context

刀 A（workspace-sync-plan）落地後，`crates/speclink-core/src/init.rs` 的現況：

- `ToolSelection`（`resolve`／`builtins_only`）與 `SyncPlan`（`resolve`／`guard`／`apply`／`differing_files`／`write_skills`）已是同步層的唯一擁有者；`update`、`reconcile_builtin_tools`、`adopt`、`probe_assets` 全走計畫。
- `init` 與 `init_remote` 還各自組流程：計畫守門 → store_init（僅 fs）→ 寫 `.speclink.yaml` → gitignore → 對選中工具逐一剝遺留 marker → `plan.write_skills(force)`。`write_skills` 是 init 專用寫入器：`write_if` 語意（不帶 force 不覆寫既有檔）、不清孤兒、不清下架足跡、不碰自訂足跡狀態，只額外刪政策關閉時的兩顆 worktree 技能目錄。
- `SyncPlan::differing_files` 與 `probe_assets` 只看 `SyncTargetKind::Builtin`，描述子 target 被跳過；`probe_assets` 的 doc comment 仍寫「自訂描述子第一版不涵蓋」。
- 消費端零程式碼變更：CLI `verbs/init.rs` 只印固定摘要；desktop `project.rs` 的 `init_project_at`／`adopt_project_at`／`probe_assets_at` 直接委派；desktop 的提示面 `assetPrompt.ts` 只用 `status`、`currentVersion` 與 `differingFiles.length`。

約束：`speclink init` 的 stdout／stderr／exit code 不變；`.speclink.yaml` 維持樣板重寫語意（fs init --force 不得把 remote section 帶進來）；規格 workspace-tools「受管檔再生的降級守門」要求守門拒絕時連設定檔都零寫入；技能檔 render、`ASSET_VERSION`、golden 不動。

## Goals / Non-Goals

**Goals:**

- 五個入口（init、init_remote、adopt、reconcile_builtin_tools、update）共用同一支寫入器 `SyncPlan::apply`；`write_skills` 與 init 自己的剝除迴圈刪除。
- `init --force` 之後工作區只剩本次選集應有的 `speclink-*` 目錄，自訂足跡狀態歸零。
- 探測涵蓋計畫裡每個 target：描述子的缺失／過期／較新進入五態聚合，差異檔清單含描述子路徑。
- 規格 workspace-tools 兩條 requirement 與實作對齊。

**Non-Goals:**

- 不改 `init`／`init_remote` 的簽名與 `InitOutcome` 形狀；CLI init 不新增輸出行（apply 回報的 stripped／pruned 在 init 路徑丟棄）。
- 不改 `.speclink.yaml` 的重寫語意（不改用保留其他鍵的 tools 改寫）。
- 不改探測對「無效描述子」的處置：無效描述子不在計畫內，探測不因它變無法判定。
- 不新增 CLI 旗標、不動 desktop 前端、不動 Node SDK、不動 golden 與 `ASSET_VERSION`。
- 不動 `CustomFootprint` 狀態檔格式。

## Decisions

### init 與 init_remote 改走 apply，計畫解析兩次

`init` 的新流程：已初始化守門 → `SyncPlan::resolve(root, builtins_only(tools), spec_dir).guard()` → `store_init` → 寫 `.speclink.yaml` → `ensure_gitignore` → `SyncPlan::resolve(...).apply(root)`（回傳的 `UpdateOutcome` 丟棄）。`init_remote` 同形：無 store_init，最後多 `write_remote_section`。

計畫解析兩次是刻意的：守門必須在第一個寫入（store_init）之前，而 `--force` 的 store_init 會把 `openspec/config.yaml` 寫回樣板、worktree 政策隨之歸零，寫入用的計畫必須在那之後解析才會排除兩顆 worktree 技能。兩次解析的 `targets` 的 `skills_root` 集合相同（只取決於選集），所以守門面與寫入面仍是同一組目錄；差異只在 `files`。替代方案「解析一次、guard 後手動改 worktree_on」否決——計畫是不可變值，改欄位等於再開一條規則入口。

刪除 `write_skills` 後 `write_if` 只剩 `.speclink.yaml` 與 `config.yaml` 兩個呼叫端，保留。init 自己的「對選中工具逐一剝遺留 marker」迴圈刪除：apply 的步驟 1 對兩個內建指令檔無條件剝，涵蓋它。`reconcile_builtin_tools` 的 doc comment 改為「五個入口共用的是 `SyncPlan::apply`」。

### init --force 的清理語意與設定檔重寫不變

`apply` 在 init 路徑做的事，逐項對應可觀察結果：

- 步驟 2（舊足跡）：`.speclink/generated-tools.yaml` 記錄的描述子足跡，因 `builtins_only` 的選集無描述子，全部視為下架 → `prune_custom` 刪其 `speclink-*` 目錄；步驟 5 `save_custom_state` 以空清單移除狀態檔。這與 `--force` 把 `.speclink.yaml` 寫回樣板（描述子一併清掉）的語意一致：重新初始化＝重設。
- 步驟 3：每個選中工具的技能檔一律改寫（不再 `write_if`），該目錄下不在集合內的 `speclink-*` 目錄刪除（改名技能的舊目錄、政策關閉後的 worktree 技能）。
- 步驟 4：未選中的內建工具 `prune_tool`（`.claude/skills` 或 `.agents/skills` 下的 `speclink-*` 目錄，變空的目錄一併移除）。

不帶 `--force` 的 init 在「已初始化即擋下」之後只會遇到空目錄或殘留技能目錄；殘留檔改寫為現版、殘留孤兒刪除，是比靜默保留更對的結果（討論 Round 2 查證：`write_if` 的不覆寫從不是規格契約）。

設定檔重寫維持樣板：`app_config_text` 不動。替代方案「保留其他鍵」在討論已否決（fs init --force 會保留 remote section、工作區變 remote 模式）。

### 探測涵蓋描述子：differing_files 與 probe_assets 走全部 target

`SyncPlan::differing_files` 移除 `Builtin` 過濾，對每個 target 比對；路徑格式不變（`<skills_dir>/speclink-<name>/SKILL.md`，`/` 組字串），描述子的 `skills_dir` 就是 `.speclink.yaml` 裡的專案根相對字串。`probe_assets` 的迴圈改為對每個 target 取 `target.label` 當 `ToolAssetState.tool`（內建為 `claude`／`codex`，描述子為其 `name`），其餘判定（`probe_skills_dir` 三態、方向、聚合優先序 較新 > 缺失 > 過期 > 現版）不變。

無效描述子：`ToolSelection::resolve` 只把通過驗證的描述子放進 `customs`，`descriptor_error` 由 `update` 轉錯誤、探測不讀——所以無效描述子不成為 target，探測結果與今天相同。這是刻意的：探測是唯讀提示面，「設定裡有個壞描述子」不是技能檔過期的訊號，由 `update` 的錯誤負責告知。

desktop 影響面：`assetPrompt.ts` 只用狀態與差異檔數，有描述子的專案在描述子技能缺失時會開始跳「還沒安裝技能檔」提示，檔案數含描述子檔。這是本變更要的行為（描述子技能檔與內建技能檔同為受管檔）。

### init 的寫入順序與失敗後的可觀察狀態

順序：guard（零寫入）→ store_init（建目錄、寫 config.yaml 樣板）→ 寫 `.speclink.yaml` → gitignore → apply（剝除 → 舊足跡 → 逐 target 寫檔與清孤兒 → 下架內建 prune → 存足跡狀態）→ remote section（僅 remote）。任一步 `Err` 即停、已寫檔案保留；每一步冪等，重跑 `init --force` 收斂到同一終態。與今天的差異只在 apply 取代 write_skills：失敗點的可觀察狀態集合沒有新增類型。

## Implementation Contract

**行為**：

- `speclink init --force --tools <選集>` 完成後，`.claude/skills` 與 `.agents/skills` 下的 `speclink-*` 目錄集合恰等於選集在當前 worktree 政策（樣板＝關閉）下的生成集合；未選工具的 `speclink-*` 目錄不存在；`.speclink/generated-tools.yaml` 不存在；兩個內建指令檔的遺留區塊被剝除、使用者內容保留。stdout 維持「Initialized at …」與「Generated files for: …」兩行。
- `speclink init`（不帶 force）在空目錄的檔案結果與今天逐位元相同。
- `speclink init --store remote --force` 同上規則，另含 remote section、不建 `openspec/`。
- desktop `init_project_at`／`adopt_project_at` 經同一條 core 路徑，行為隨之。
- `probe_assets`：tools 清單含有效描述子時，`tools[]` 多一項 `{ tool: <描述子 name>, workspaceVersion, stale, newer, missing }`；描述子 skills_dir 無任何 `speclink-*` 技能檔 → 該項 `missing: true`，整體狀態依既有優先序聚合；`differingFiles` 含 `<skills_dir>/speclink-<name>/SKILL.md` 形式的描述子路徑。無描述子的專案輸出逐位元不變。

**介面**：`init`、`init_remote`、`probe_assets` 簽名不變；`SyncPlan::write_skills` 刪除；`SyncPlan::differing_files` 簽名不變、語意改為全部 target；`AssetProbe`／`ToolAssetState` serde 形狀不變。連帶移除本變更孤兒化的 crate 私有項目：`SyncTargetKind` enum、`SyncTarget.kind`（探測與差異比對不再依 target 種類分流後無讀取者）與 `SyncPlan.worktree_on`（只有 `write_skills` 讀過）——無外部形狀影響。描述子 target 的 `skills_dir` 在 `SyncPlan::resolve` 正規化一次（去結尾分隔符、以 `/` 回報），`ToolSelection::resolve` 在第一個壞描述子之後仍收下後續合法描述子（第一個錯誤保留給 update）。

**失敗模式**：守門拒絕 → 今天字面的單行英文訊息、零寫入（含 `openspec/` 與 `.speclink.yaml`）。apply 中途失敗 → 已寫檔案保留、錯誤上拋，CLI exit code 非零。探測：`.speclink.yaml` 壞檔 → `unknown`；描述子技能檔存在但版號讀不出 → `unknown`（與內建同規則）；無效描述子 → 不影響。

**驗收**：
- 新增測試（`crates/speclink-core/src/init.rs`）：`init_force_switching_tools_prunes_the_deselected_footprint`（claude → codex）、`init_force_prunes_a_renamed_skill_directory`（`speclink-onboard` 殘留）、`init_force_resets_custom_footprint_state`（先 update 生成描述子足跡，`init --force --tools claude` 後 `.speclink/generated-tools.yaml` 不存在且描述子目錄被清）、`init_strips_the_unselected_tools_legacy_marker`（tools=[codex] 的 `init --force`，`CLAUDE.md` 遺留區塊被剝）、`init_over_residual_skill_files_rewrites_them`（無 `.speclink.yaml` 但 `.claude/skills/speclink-apply/SKILL.md` 為舊內容，不帶 force 的 init 後為現版內容）、`probe_reports_missing_for_a_descriptor_without_skills`、`probe_reports_stale_for_a_descriptor_behind_the_engine`、`probe_lists_descriptor_paths_in_differing_files`、`probe_ignores_an_invalid_descriptor`。
- 既有測試一個不刪：`init_force_over_a_legacy_workspace_strips_the_marker`、`init_force_refuses_a_workspace_that_leads_the_engine`、`reconcile_matches_init_output_for_the_same_selection`、`generation_omits_worktree_skills_when_the_policy_is_false`、`init_on_a_fresh_project_writes_no_instruction_file`、`init_does_not_create_the_user_settings_file`、全部 `probe_*`／`skill_probe_*`。
- `cargo test -p speclink-cli --test it init_tools` 全綠（空目錄 init 的輸出與檔案效果不變）；`cargo test -p speclink-desktop-core project::` 全綠；`cargo test -p speclink-core --test it render_golden::` 全綠且 golden 零 diff。
- `grep -n "write_skills\|第一版不涵蓋" crates/speclink-core/src/init.rs` 零命中。

**範圍邊界**：in scope＝`crates/speclink-core/src/init.rs` 的 init／init_remote／differing_files／probe_assets 與測試、workspace-tools 的兩條 delta。out of scope＝CLI 輸出、desktop 前端、`.speclink.yaml` 重寫語意、`CustomFootprint` 格式、golden、`ASSET_VERSION`。

## Risks / Trade-offs

- [使用者靠 `init --force` 保留另一工具的技能檔] → 這是本變更刻意移除的行為；proposal 相容性段給出替代（`--tools claude,codex` 或 `speclink update`），規格 scenario 明寫。
- [`init --force` 在守門通過後、apply 中途失敗] → 與 `update` 相同的半套語意（已寫檔案保留、重跑收斂），無新增失敗類型；`reconcile_matches_init_output_for_the_same_selection` 守住成功路徑終態與 reconcile 一致。
- [描述子專案的 desktop 提示變多] → 只在描述子技能檔確實缺失或過期時出現，且是規格要的行為；無描述子專案零變化。
- [跨平台：描述子 `skills_dir` 含 `/`，Windows 上的比對與刪除] → `differing_files` 讀檔用 `skills_root.join(dir)`（PathBuf），回報字串維持 `/`；`prune_custom` 既有路徑檢查不動；既有 `probe_ignores_line_ending_differences` 守住 CRLF。
- [golden] → 本變更不動 render，`render_golden::` 全綠即證。

## Migration Plan

無資料遷移。回滾＝revert commit。舊 binary 對新工作區無相容問題（檔案集合是子集）。

## Open Questions

無。
