## Context

工作區產物同步層全部落在 `speclink-core` 的 `init.rs`、`skills.rs`、`config.rs`。今天的結構：

- **工具選集**（`.speclink.yaml` 的 tools 清單 → 內建工具與自訂描述子）在 `update`、`probe_assets`、`reconcile_builtin_tools`（守門目標抽描述子）、desktop `connections.rs` 的 checkout 預選各解析一次。`update` 的降級守門目標集另外用三個分支（legacy 回退／選集／描述子）手工鏡像寫入集。
- **受管技能集合**（target 在政策下應有哪些 `speclink-*/SKILL.md` 與內容）在 `generate_tool`、`generate_custom`、`expected_skill_dirs`、`differing_managed_files` 各算一次，每一處都各自呼叫 `worktree_skills_enabled` 重讀 `openspec/config.yaml`。
- **`.speclink.yaml` 讀改寫**：`init.rs` 的 `load_app_yaml_doc` 與 `config.rs` 的 `parse_yaml_mapping` 邏輯與錯誤訊息逐字相同；remote section 兩支動詞住 `init.rs`，tools 改寫住 `config.rs`。
- **技能渲染**：`render_skill_file`／`render_skill_file_custom` 各組一份八行相同的 frontmatter，`substitute`／`substitute_neutral` 各跑一條四段相同的代換鏈；`render_skill_file_for` 是純轉發。

消費端：`speclink-cli`（`verbs/init.rs`、`verbs/connection.rs`、`verbs/config.rs`）、`apps/desktop/core`（`project.rs`、`settings.rs`）、`apps/desktop/src-tauri`（`connections.rs`、`remote.rs`、`lib.rs`）、`speclink-node`（`render.rs` 走 `render_skill_file_for`）、`speclink-host`（`ensure_gitignore`）。

約束：零使用者可觀察行為變化。技能檔輸出位元級不變（四份 golden 鎖住），`ASSET_VERSION` 不 bump；CLI 人眼輸出與 `--json` 不變；規格 `workspace-tools` 每一條 requirement 維持。來源討論 improve-workspace-sync 已把兩條行為變化（清孤兒擴及 init --force、探測涵蓋描述子）歸給刀 B。

## Goals / Non-Goals

**Goals:**

- 工具選集與受管技能集合各只有一個擁有者；守門目標、生成集合、清理集合、探測差異集合都從同一份計畫讀出。
- 刪除 `generate_tool`、`generate_custom`、`expected_skill_dirs`、`differing_managed_files`、`skill_targets`、`load_app_yaml_doc` 六支函式，工作被計畫吸收，不留轉發殼。
- `.speclink.yaml` 讀改寫三動詞同住 `config.rs`、共用一支解析器。
- `skills.rs` 一支 render、一張代換表；`RenderTarget` 成為真正的分支點。
- 一次同步只讀一次 worktree 政策。

**Non-Goals:**

- 不改 `init`／`init_remote`／`adopt`／`reconcile_builtin_tools`／`update` 的對外簽名、前置條件與可觀察結果；`init` 不改走 `update` 的完整流程（刀 B）。
- 探測不納入自訂描述子（刀 B）；`init --force` 不開始清孤兒目錄（刀 B）。
- 不動 Node SDK 公開 API、不動 golden 檔、不動 `assets.lock`、不 bump `ASSET_VERSION`。
- 不動 `CustomFootprint` 狀態檔（`.speclink/generated-tools.yaml`）的格式與語意。
- 不新增 CLI 旗標、不動任何規格條文。

## Decisions

### ToolSelection 單一解析，描述子錯誤延後裁決

`init.rs` 新增 `ToolSelection { builtins: Vec<Tool>, customs: Vec<CustomTool>, notes: Vec<String>, descriptor_error: Option<String>, legacy_fallback: bool }`，由 `ToolSelection::resolve(root, &AppConfig)` 建立：內建名去重（保留 `Tool::parse` 的 `agents` 別名）、未知內建名進 `notes`（訊息字面與今天 `update` 相同）、描述子逐一 `validate()` 與重名檢查——第一個錯誤存進 `descriptor_error` 而不是回 `Err`。tools 清單為空時 `legacy_fallback = true`，`builtins` 由「`.claude` 目錄存在 → `[Claude]`」得出（今天 `update` 的回退規則）。`ToolSelection::builtins_only(&[Tool])` 供 `init`／`init_remote`／`reconcile` 以記憶體選集直接建。

為什麼描述子錯誤不直接 `Err`：`update`／`reconcile` 要在任何寫入前把它轉成錯誤（維持「壞描述子零寫入」），但 `probe_assets` 今天完全忽略描述子——同一支解析若在探測時因描述子壞掉回 Unknown，就是行為變化。錯誤放在值上，由消費端裁決，解析只做一次。替代方案「兩支解析（含描述子／只內建）」被否決：那是把四份縮成兩份，擁有者還是兩個。

### SyncPlan 為唯一 adapter：resolve、guard、apply、diff

`SyncPlan::resolve(root, selection: ToolSelection, spec_dir) -> SyncPlan` 讀一次 worktree 政策，對每個內建（固定順序 Claude、Codex，只取 `selection.builtins` 內的）與每個描述子建一個 `SyncTarget { kind: SyncTargetKind::Builtin(Tool) | Custom(String), label, skills_dir: String, skills_root: PathBuf, files: Vec<(String, String)> }`——`skills_dir` 是專案根相對、`/` 組成的目錄字串（探測回報路徑用），`skills_root` 是同一目錄的絕對路徑（守門與寫檔用），`files` 是「`speclink-<name>` 目錄名 → SKILL.md 內容」的預期檔案表。計畫另帶 `deselected_builtins: Vec<Tool>`（非 legacy 回退時，`[Claude, Codex]` 減選集）、`custom_strip_targets`（描述子仍宣告 `instructions_file` 者的路徑與棄用訊息）、`selection` 本身，以及 resolve 當下讀到的 `worktree_on`。`SyncPlan`／`SyncTarget`／`SyncTargetKind`／`managed_skills` 全部 `pub(crate)`——模組外只有 desktop 用到 `ToolSelection`，計畫沒有 crate 外消費端。

- `guard(&self) -> Result<()>`：對每個 target 的 `skills_root` 跑既有 `probe_skills_dir` 方向判定，任一領先即以今天的英文單行訊息拒絕。這就是規格「判定目標取自實際寫入集」的結構保證——目標集與寫入集是同一個 `targets`。
- `apply(&self, root) -> Result<UpdateOutcome>`：見「apply 寫入順序」決策。
- `differing_files(&self) -> Vec<String>`：只看 Builtin target（唯一消費端是探測，第一版不涵蓋描述子），逐檔比對（換行正規化維持），路徑由 `skills_dir` 組成 `<skills_dir>/speclink-<name>/SKILL.md`（與今天 `differing_managed_files` 相同格式，不做 `strip_prefix`）。
- `write_skills(&self, force: bool) -> Result<()>`：`init` 專用——逐 target 逐檔 `write_if`，不動描述子狀態、不清孤兒（見 init 決策）。唯一的刪除：`worktree_on` 為 false 時移除兩顆 worktree 技能的目錄（今天 `skip_gated_skill` 在 init 路徑上的行為——`init --force` 會把政策重置為關閉，工具不得繼續載入被政策關掉的技能）。`skills_root` 已是絕對路徑，不需要 `root` 參數。

消費端組合：`update` ＝ load AppConfig → resolve selection → 描述子錯誤即 `Err` → resolve plan → guard（`allow_downgrade` 為 true 時跳過）→ apply。`probe_assets` ＝ load AppConfig（失敗回 Unknown）→ resolve selection → `legacy_fallback` 時直接回 Current／空清單（今天的探測從不做 `.claude` 目錄回退；沒有清單就沒有受管工具可查）→ resolve plan → 對 Builtin target 逐一 `probe_skills_dir` 組 `ToolAssetState` → 狀態聚合 → 非 Current 時 `differing_files()`。`reconcile_builtin_tools` ＝ 讀原文 → `update_app_config_tools_text` 改寫 → 把改寫後文字解析成 AppConfig 建 selection（描述子由改寫後文字承接，取代今天「再載一次原檔抽描述子」）→ 描述子錯誤即 `Err`（今天是寫完設定檔才在 `update` 裡失敗；現在連設定檔都不寫，與「壞描述子零寫入」的失敗模式對齊）→ resolve plan → guard → 寫設定檔 → apply；`update` 內的第二次 guard 因走同一份 plan 而免除。替代方案「計畫做成 trait 給 desktop 自建」否決——desktop 三個消費端全走 core 公開函式，多一層是空轉接。

### managed_skills 為受管技能集合唯一擁有者

`managed_skills(target: RenderTarget, worktree_on: bool, spec_dir) -> Vec<(String, String)>`：走 `skills::registry()`，非 Claude target 只取 `for_codex`，`worktree_gated` 只在 `worktree_on` 時保留，內容用 `render_skill_file_for`。這是 `SyncTarget::files` 唯一的產生點。`worktree_skills_enabled` 保留（含「config 存在但解析失敗 → 保留技能」的安全方向），但只在 `SyncPlan::resolve` 呼叫一次。`skip_gated_skill` 的「政策關閉時刪掉上次留下的目錄」行為，在 update 路徑改由 apply 的孤兒清理承接（不在預期檔案表的 `speclink-*` 目錄本來就會被清）——`update_prunes_renamed_skill_directory`、`generation_omits_worktree_skills_when_the_policy_is_false` 兩個既有測試釘住結果相同；在 init 路徑由 `write_skills` 只刪那兩顆 gated 目錄承接（`init_force_removes_gated_skill_directories_the_reset_policy_no_longer_allows` 釘住）。

### apply 寫入順序與失敗後的可觀察狀態

`apply` 維持今天 `update` 的順序，任一步 `Err` 即停、已寫檔案保留（與今天相同：每一步冪等，重跑收斂）：

1. 遺留剝除：兩個內建指令檔（`CLAUDE.md`、`AGENTS.md`）無條件剝，描述子的 `instructions_file` 剝並記棄用提示（`deprecations`）。
2. 舊足跡刪除：載入 `.speclink/generated-tools.yaml`，不再屬於現行描述子（name＋skills_dir 判定不變）的足跡走 `prune_custom`，被刪的名字先暫存、不立刻進 `pruned`。
3. 逐 target（Claude、Codex、描述子依序）：寫 `files` 每一檔（一律覆寫）、清該 `skills_root` 下不在 `files` 內的 `speclink-*` 目錄；`updated` 依序推入 label。
4. `deselected_builtins` 逐一 `prune_tool`，有移除即進 `pruned`。
5. 步驟 2 暫存的名字追加進 `pruned`，然後 `save_custom_state`。

舊足跡的刪除放在寫入之前（今天 `update` 的相對順序）而不是最後：描述子只改名、`skills_dir` 沿用同一目錄時，先寫後刪會把剛生成的檔案一併帶走。報告延後到步驟 5 是為了讓 `pruned` 維持「內建在前、描述子在後」。描述子的生成因此提前到內建之後、下架之前——`updated` 清單的順序仍是「claude、codex、描述子」，CLI 的「Updated skill files for:」一行位元級不變。唯一終態會變的輸入是「描述子 `skills_dir` 指向內建目錄（如 `.claude/skills`）且該足跡剛下架」：今天先寫 claude 再被 `prune_custom` 整批刪掉（破壞性），現在先刪後寫、claude 檔留存；`validate()` 不擋這種設定，屬誤用，新終態較佳。

### init 在本刀只換寫入來源，不改流程

`init`／`init_remote` 改為：已初始化守門 → `SyncPlan::resolve(root, ToolSelection::builtins_only(tools), spec_dir).guard()`（取代 `skill_targets`＋`refuse_downgrade`）→ `store_init`（僅 fs）→ 寫 `.speclink.yaml`（樣板＋tools，語意不變）→ gitignore → 逐工具剝遺留 marker（只剝選中的，語意不變）→ `plan.write_skills(force)`。`init`（fs 模式）的計畫 resolve 兩次：守門那次在 `store_init` 之前（拒絕＝零寫入），寫檔那次在之後——`--force` 會把 `openspec/config.yaml` 寫回範本，worktree 政策以重置後的檔為準，這正是今天 `generate_tool` 讀政策的時點；`init_remote` 沒有 `store_init`，一份計畫從頭用到尾。`workspace_init` 的內容併入兩支本體（`.speclink.yaml` 的文字組合抽成兩參數的純函式 `app_config_text`，其餘四行各自直寫）。刀 B 再把 `write_skills` 換成 `apply`。

### `.speclink.yaml` 讀改寫收進 config.rs

`write_remote_section`／`remove_remote_section` 搬到 `config.rs`，緊鄰 `update_app_config_tools_text`，三支共用 `parse_yaml_mapping`（`load_app_yaml_doc` 刪除；remote 兩支經一行私有的 `read_app_yaml_doc` 把 `read_opt(...).unwrap_or_default()` 交給 `parse_yaml_mapping` 的空文字分支承接「檔案不存在 → 空 mapping」，錯誤訊息 `invalid .speclink.yaml: …` 字面不變）。`init.rs` 內的呼叫改 `crate::config::` 路徑；CLI `verbs/connection.rs`、desktop `connections.rs`／`remote.rs` 改 use 路徑。純量寫出走 serde_yaml 序列化整份 mapping（今天即如此），不手拼 YAML。

### skills.rs 單一 render 與代換表

`render_skill_file_for(target, skill, spec_dir)` 成為唯一 render：frontmatter 八行組一次；`RenderTarget::Builtin(Claude)` 且 `fork`／`disallow_edit` 時插入 Claude 專屬三行；前言＝Builtin(Claude) 且 fork 時 `fork_context`、Custom 時 `invocation_note`；body 代換走一張表 `Substitutions { spec_dir_slash, plan_dir, tool_name, slash_replacement, drop_plan_mode_lines, neutralize_skill_refs }` 由 target 填值。`substitute(body, tool, spec_dir)` 保留為 Builtin 的公開便利入口（golden 測試呼叫它）；`substitute_neutral`、`render_skill_file`、`render_skill_file_custom` 刪除。檔尾統一「收斂到恰一個結尾換行」——builtin 今天是「去掉多餘空行」、custom 是「去多餘再補一個」，兩者對結尾已有換行的 asset 結果相同；四份 golden 是唯一裁判，任何一份變紅即代表合一寫錯，不得再生 golden。

## Implementation Contract

**行為**：`speclink init`、`speclink init --store remote`、`speclink update`（含 `--allow-downgrade`）、`speclink workflow-config` 寫入後的技能同步、desktop 的開專案探測／技能更新／設定頁工具切換／checkout 綁定，在本變更前後對同一輸入產生位元級相同的檔案系統結果、相同的 stdout／stderr 文字、相同的 `--json` payload、相同的 IPC 回傳（`probeAssets` 的 `status`／`currentVersion`／`tools[]`／`differingFiles` 欄位與值）。

**介面**（`speclink-core`，`pub` 只為 crate 內與既有消費端所需）：
- `init::ToolSelection`（欄位如決策所列）、`ToolSelection::resolve(root: &Path, app: &AppConfig) -> ToolSelection`、`ToolSelection::builtins_only(tools: &[Tool]) -> ToolSelection`。
- `init::SyncPlan`（`pub(crate)`）、`SyncPlan::resolve(root, selection, spec_dir) -> SyncPlan`、`guard(&self) -> anyhow::Result<()>`、`apply(&self, root) -> anyhow::Result<UpdateOutcome>`、`differing_files(&self) -> Vec<String>`、`write_skills(&self, force) -> anyhow::Result<()>`。
- `init::managed_skills(target: skills::RenderTarget, worktree_on: bool, spec_dir: &str) -> Vec<(String, String)>`（`pub(crate)`）。
- `config::write_remote_section(root, url, repo) -> anyhow::Result<()>`、`config::remove_remote_section(root) -> anyhow::Result<bool>`（簽名與今天 `init::` 版相同）。
- `skills::render_skill_file_for` 簽名不變；`skills::substitute` 簽名不變。
- 既有公開函式 `init`、`init_remote`、`adopt`、`reconcile_builtin_tools`、`update`、`probe_assets`、`detect_footprint_tools`、`detect_tools`、`parse_tools`、`parse_tool_names`、`ensure_gitignore` 簽名與回傳型別不變；`UpdateOutcome`、`AssetProbe`、`ToolAssetState`、`AssetStatus` 的 serde 形狀不變。
- desktop `connections.rs` 的 checkout 預選改為：`AppConfig::load` 成功 → `ToolSelection::resolve(root, &app)`；`legacy_fallback` 為真、或解析後 `builtins` 為空（清單只有描述子或未知名）時改用 `detect_footprint_tools`——與今天「picked 為空就回退」逐字等價，維持「缺清單只依 footprint、不補 Claude」。

**失敗模式**：描述子無效／重名 → `update`／`reconcile` 以今天字面的單行錯誤失敗、零寫入；`probe_assets` 對描述子錯誤不反應（只讀內建）。降級守門拒絕 → 今天字面的單行英文訊息、零寫入（`reconcile` 連設定檔都不寫）。`.speclink.yaml` 壞檔 → `update` 錯誤、`probe` 回 `unknown`（不變）。

**驗收**：
- `cargo test -p speclink-core` 全綠，且 `init.rs` 既有 55 個測試一個不刪、名稱不改（含 `the_guard_covers_the_legacy_fallback_without_a_tools_list`、`the_guard_covers_custom_descriptor_skill_files`、`reconcile_refuses_a_leading_workspace_before_touching_the_config`、`reconcile_matches_init_output_for_the_same_selection`、`generation_omits_worktree_skills_when_the_policy_is_false`、`update_prunes_renamed_skill_directory`、`probe_with_an_empty_tools_list_reports_current_and_writes_nothing`、`dropping_the_deprecated_instructions_file_field_does_not_prune_the_descriptor`）。
- `cargo test -p speclink-core --test it render_golden::` 全綠、`crates/speclink-core/tests/golden/` 與 `assets.lock` 零 diff（git status 為證）。
- `cargo test -p speclink-cli --test it` 中 init／update／connection 相關測試全綠。
- `cargo test -p speclink-desktop` 中 `connections.rs` 的 inspect／bind 測試全綠（`inspect_without_a_tools_list_preselects_only_actual_footprints`、`inspect_with_a_descriptor_only_tools_list_falls_back_to_footprints`、`inspect_reports_only_builtins_from_a_mixed_tools_list`、`bind_switches_claude_to_codex_preserving_user_text_and_descriptors`、`desktop_bind_and_cli_remote_init_produce_isomorphic_artifacts`）。
- 新增測試：`ToolSelection::resolve` 的四種輸入（純內建、混描述子、壞描述子、空清單＋`.claude` 目錄）；`SyncPlan::resolve` 對 worktree 政策開／關與 codex 子集的預期檔案表；`worktree_skills_enabled` 在 `crates/speclink-core/src/init.rs` 只剩 `SyncPlan::resolve` 一個呼叫點（grep 為證，不另寫計數測試）；`guard` 目標集等於 `targets` 的 `skills_root` 集合。
- 六支函式（`generate_tool`、`generate_custom`、`expected_skill_dirs`、`differing_managed_files`、`skill_targets`、`load_app_yaml_doc`）與 `substitute_neutral`、`render_skill_file`、`render_skill_file_custom` 在 `crates/` 與 `apps/` 下 grep 為零命中。

**範圍邊界**：in scope＝上列三個 core 檔、一個 core 整合測試檔、desktop 兩檔與 CLI 一檔的 use 路徑或消費端改寫。out of scope＝`init` 走 `apply`、探測涵蓋描述子、`init --force` 清孤兒、任何規格條文、golden 再生、`ASSET_VERSION`、Node SDK API、`CustomFootprint` 格式。

## Risks / Trade-offs

- [golden 位元級鎖定：render 合一若讓任一 target 的檔尾或前言差一個字元] → 四份 golden 是唯一裁判；任務順序把 render 合一放在最前、先跑 golden 再動 init.rs；golden 變紅一律修 render，不再生 golden。
- [CLI 人眼輸出：`updated`／`pruned`／`stripped` 清單順序] → apply 固定「claude、codex、描述子」順序；`cargo test -p speclink-cli --test it` 的 update 輸出測試守門。
- [Windows：`differing_files` 的路徑格式與 CRLF] → 相對路徑維持 `/` 組字串、讀檔時維持 `split('/')` 轉 PathBuf；比對前換行正規化維持（`probe_ignores_line_ending_differences`）。
- [desktop `speclink-desktop` crate 的測試需先備妥 sidecar 與 server-web dist] → 任務驗證步驟明列 `cargo test -p speclink-desktop` 前的準備條件；純邏輯測試不受影響。
- [apply 順序調整（舊足跡刪除提前、描述子生成提前到步驟 3）] → 終態分析見決策；`update_strips_a_descriptors_legacy_marker`、`the_gate_applies_to_codex_and_custom_descriptors_alike`、`dropping_the_deprecated_instructions_file_field_does_not_prune_the_descriptor` 三個測試守住描述子路徑。
- [`reconcile` 改用改寫後文字解析描述子] → 改寫只動 `tools` 內的內建字串項，描述子物件逐字保留（`update_app_config_tools_text` 既有測試），解析結果與原檔相同；`reconcile_refuses_a_leading_workspace_before_touching_the_config` 守住「拒絕即零寫入」。

## Migration Plan

純內部重構，無部署步驟、無資料遷移。回滾＝revert 該 commit；`CustomFootprint` 狀態檔格式不變，新舊 binary 可互換讀寫。

## Open Questions

無。刀 B（`init` 走 `apply`、探測涵蓋描述子、清孤兒擴及 `init --force`）於本變更封存後由討論 improve-workspace-sync 再轉出。
