---
topic: 工作區產物同步層（init／update／probe）的結構改善——受管技能集合與工具選集在多處重複計算
slug: improve-workspace-sync
status: promoted
promoted_to: workspace-sync-plan, workspace-sync-entrypoints
created: 2026-09-07
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
---

# Discussion: 工作區產物同步層（init／update／probe）的結構改善——受管技能集合與工具選集在多處重複計算

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

**起因**：使用者以 `/speclink-improve` 不帶方向啟動，範圍由 git 熱點推斷。

**範圍收斂**：近三個月熱點榜前段幾乎全是 CLI 層與 golden 衍生物，但 CLI 層已由三份 improve 記錄（improve-cli-command-layer 08-07、improve-wire-convert-seam 08-10、improve-cli-verb-layer 09-01）連掃三次並落地，不再重掃。扣掉 ASSET_VERSION 純 bump 的 commit 後，近兩個月實質改動最多的核心檔是 `crates/speclink-core/src/init.rs`（22 個實質 commit，來源含 remove-marker-injection、instruction-downgrade-guard、worktree-toggle-and-guards、verify-station-parity、rename-onboard-to-baseline、unify-agent-tool-bootstrap）。因此範圍定為「工作區產物同步層」：`init.rs`（init／init_remote／adopt／reconcile_builtin_tools／update／probe_assets）＋ `skills.rs`（registry 與渲染）＋ 它們的消費端（`crates/speclink-cli/src/verbs/init.rs`、`apps/desktop/core/src/project.rs`、`apps/desktop/core/src/settings.rs`、`apps/desktop/src-tauri/src/connections.rs`、`crates/speclink-core/src/config.rs` 的 tools 改寫）。

**Step 1 排除**：`speclink list --json` 為空，無進行中變更相撞。`discuss search` 以 init／skills／assets／update／workspace 搜尋：命中的都是需求討論（init-marker-openspec-alignment、desktop-instruction-staleness-prompt、remote-workspace-local-skill-bootstrap 等），沒有任何一條 Ruled out／Rejected 否決過本輪候選。唯一相近的先例是 unify-agent-tool-bootstrap（07-24）design 的「拒絕 CLI／Desktop 各自組合 config 寫入與 update、改以單一收斂入口」——候選 3 是把那條決策做完，不是重提被否決的方案。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — scan (2026-09-07)

**Focus**: 工作區產物同步層（init／update／probe）有哪些結構改善
**Position**: 這一層的核心概念只有兩個——「哪些工具要生成」（工具選集）與「每個工具要生成哪些技能檔」（受管技能集合）——但兩個概念都沒有一個擁有者，而是在四個函式裡各算一次；每次語意變動（worktree 開關、技能改名、marker 拆除）都得逐處補丁，且已有實證漏補。五個候選如下：

---

**候選 1：受管技能集合在四處各算一次**
- **Files**：`crates/speclink-core/src/init.rs:676`（generate_tool）、`:506`（generate_custom）、`:546`（expected_skill_dirs）、`:993`（differing_managed_files）；`:482`（worktree_skills_enabled）
- **Problem**：訊號 1（一個概念散在多處）。「target T 在政策 P 下應有哪些 speclink-* 目錄與內容」這條規則——for_codex 子集過濾＋worktree 門檻過濾＋目錄名格式——在四個函式各寫一遍，而且 `worktree_skills_enabled` 在每一處都重新讀一次 `openspec/config.yaml`（一次 update 對兩個工具讀約 4～6 次）。實證：worktree-toggle-and-guards（eacbb69）落地時 `differing_managed_files` 漏補門檻過濾，政策關閉的專案被永久報成「檔案缺失／過期」，事後補了那段註解；rename-onboard-to-baseline（5aa3a13）要清孤兒目錄，只能再加第四份 `expected_skill_dirs`。
- **Solution**：一支 `managed_skills(target: RenderTarget, worktree_on: bool, spec_dir) -> Vec<ManagedSkill { dir_name, content }>` 當唯一擁有者。generate ＝ 逐一寫檔；prune_orphan ＝ 目錄不在集合內就刪；differing ＝ 逐一比對；worktree 政策在一次同步開頭讀一次傳下去。四支函式縮成「集合 × 動作」。
- **Wins**：新增一種門檻（例如未來的 tdd 開關）或改名技能只動一支函式；probe 與 update 結構上不可能再漂移（讀同一個集合）；config.yaml 讀取次數降到每次同步一次。
- **Recommendation**：**strongly recommended**——刪除測試明確通過：四份規則消失後，同一行為只剩一處可讀。

---

**候選 2：工具選集解析四份、降級守門目標與寫入集是兩份平行計算**
- **Files**：`crates/speclink-core/src/init.rs:230-262`（update 的 selected／customs 拆分）、`:264-280`（守門目標的三分支重算）、`:913-935`（probe_assets 只取 builtins）、`:375-395`（reconcile_builtin_tools 再載一次 AppConfig 抽描述子）；`apps/desktop/src-tauri/src/connections.rs:202-226`（preselected_tools 第四份）
- **Problem**：訊號 1＋4。`.speclink.yaml` 的 tools 清單 → 「內建去重＋描述子驗證＋未知名警告」這段解析寫了四次。更要緊的是 `update` 的降級守門目標集（legacy 用 `.claude` 存在、否則用 selected、加 customs）是對寫入集的**手寫鏡像**——註解說「與寫入集同源」，實際是兩段獨立分支各自維護；`reconcile_builtin_tools` 為了提前守門又重載一次 config 抽描述子。`probe_assets` 因為自己解析，順手把描述子排除在外（註解「第一版不涵蓋」），於是 update 會寫、probe 不會看的檔案集合已經存在。
- **Solution**：一個 `ToolSelection::resolve(&AppConfig) -> { builtins, customs, notes, legacy_fallback }`，帶 `skill_roots(root) -> Vec<PathBuf>`。守門目標＝`selection.skill_roots()`；generate／prune 迴圈走 `selection`；probe 與 desktop 預選讀 `selection.builtins`（probe 順帶涵蓋 customs，零額外設計）。
- **Wins**：守門與寫入集變成同一個值，不可能分岔；描述子進 probe 免費；desktop 少一份自寫解析。與候選 1 合起來就是「一次同步先算一份計畫（選集 × 集合），guard／generate／prune／probe 都只消費計畫」。
- **Recommendation**：**strongly recommended**——與候選 1 天然同刀，建議一起立案。

---

**候選 3：五個公開入口、兩條生成路徑，文件宣稱的「單一入口」並未實現**
- **Files**：`crates/speclink-core/src/init.rs:72`（init）、`:91`（init_remote）、`:166`（workspace_init）、`:375`（reconcile_builtin_tools）、`:414`（adopt）、`:230`（update）；`crates/speclink-cli/src/verbs/init.rs:81,351`；`apps/desktop/src-tauri/src/connections.rs:245-278`
- **Problem**：訊號 1＋4。`init` 走 `workspace_init → generate_tool(force)`：不清孤兒、不管描述子、自己跑一次 strip；`adopt`／desktop 設定頁／checkout 綁定走 `reconcile → update`：全套。同一個「讓工作區符合 tools」有兩條生成路徑。`reconcile_builtin_tools` 的 doc comment 說它是「CLI init、remote init 與 desktop 共用的單一入口」，但 CLI 的 `cmd_init` 與 `cmd_init_remote` 呼叫的是 `init`／`init_remote`，不經 reconcile——unify-agent-tool-bootstrap（07-24）的決策只做了一半。另外 `reconcile → update` 路徑上降級守門跑兩次（註解自承）；desktop 綁定是「先寫 remote section 再 reconcile」、`init_remote` 是「先 workspace_init 再寫 remote section」，同一目標狀態兩種順序。
- **Solution**：`init` ＝ 已初始化守門 ＋ store_init ＋ 寫 `.speclink.yaml`（tools／spec_dir／remote）＋ gitignore ＋ `update()`；`init_remote` 同形只多 remote section。`workspace_init`、`generate_tool`、`skill_targets` 隨之消失，守門只在 `update` 一處。
- **Wins**：生成行為只有一種，init 出來的工作區與 update 收斂後的工作區保證同形；desktop 綁定與 CLI remote init 變成同一組步驟。
- **Recommendation**：**worth exploring**——摩擦有實證，但 `write_if`（init 不帶 --force 不覆寫既有技能檔）與 update 的「一律改寫」語意是否為契約，要先查 workspace-tools 規格與 55 個 init.rs 測試才能定刀形；且依賴候選 1／2 先落地。

---

**候選 4：skills.rs 的兩軌渲染共用八行 frontmatter 與四段代換，中間夾一支純轉發**
- **Files**：`crates/speclink-core/src/skills.rs:182`（substitute）、`:232`（substitute_neutral）、`:222`（render_skill_file_for）、`:273`（render_skill_file_custom）、`:298`（render_skill_file）
- **Problem**：訊號 2（淺模組）。`RenderTarget` enum 已存在，但只當 dispatch key：`render_skill_file_for` 是純轉發，底下兩支 render 各自組 frontmatter（八行相同）、各自跑代換鏈（四段相同），差異只在三個點——Claude 專屬 frontmatter 行、前言（fork_context vs invocation_note）、代換表（slash 前綴、plan_dir、去 plan-mode 行）。還藏一個不對稱：custom 版結尾補 `\n`、builtin 版不補。
- **Solution**：一支 `render(target, skill, spec_dir)`，target 只決定三個差異點；`substitute`／`substitute_neutral` 合成一張代換表。
- **Wins**：frontmatter 或 NEXT_STEPS_LEAD 這類共同段落改一處；golden 已鎖三種 target 的輸出，重構安全網現成。
- **Recommendation**：**worth exploring**——集中效果明確但面積小，適合搭在候選 1 的 change 內順手做，不值得單開。

---

**候選 5：`.speclink.yaml` 讀改寫的解析器有兩份、動詞分住兩個模組**
- **Files**：`crates/speclink-core/src/init.rs:109-153`（write_remote_section／remove_remote_section／load_app_yaml_doc）、`crates/speclink-core/src/config.rs:855-905`（update_app_config_tools_text／parse_yaml_mapping）
- **Problem**：訊號 1。`load_app_yaml_doc` 與 `parse_yaml_mapping` 邏輯逐行相同（mapping／null／其他三分支、同一句錯誤訊息）；「改 `.speclink.yaml` 某一鍵、其餘鍵原樣保留」這個概念的三個動詞分住 init.rs 與 config.rs。
- **Solution**：remote section 的兩支搬到 `config.rs` 與 tools 改寫同住，共用 `parse_yaml_mapping`；刪 `load_app_yaml_doc`。
- **Wins**：讀改寫的錯誤語意（壞檔即拒、註解不保留）只剩一處；init.rs 少 45 行與 config 無關的內容。
- **Recommendation**：**strongly recommended** 但太小——建議併入候選 2 或 3 的 change，不單獨立案。

---

**我的順位**：先做候選 1＋2 合一刀（受管技能集合＋工具選集各收成一個擁有者，順帶把候選 5 帶上）。理由：兩個都有「上一次改語意時漏補」的實證，刪除測試最乾淨，而且它們落地後候選 3 的刀形才看得清楚（init 改走 update 前，update 內部得先只剩一份規則）。候選 4 面積小，等候選 1 動到 skills.rs 時順手。

**請你挑一個候選深談**（或指定合併哪幾個）。挑定後我再進 interview 輪，逐題帶證據問。

### Round 2 — interview (2026-09-07)

**Focus**: 五個候選全談——先定切刀方式（最上游問題），再逐候選過介面深度四項檢查
**Position**: 建議切成兩刀，依「有沒有可觀察的行為變化」分界：
- **刀 A（零行為變化）＝候選 1＋2＋5＋4**：在 core 內立一份「同步計畫」——`ToolSelection`（候選 2）× `managed_skills`（候選 1）——update／probe／reconcile 的守門都只消費這份計畫；`.speclink.yaml` 讀改寫收進 config.rs（候選 5）；skills.rs 兩軌渲染合一（候選 4）。安全網現成：55 個 init.rs 測試、四份 golden、CLI 整合測試。desktop 只換 `connections.rs:202` 的 preselected_tools 一處消費端。
- **刀 B（有行為變化）＝候選 3**：init／init_remote 改為「守門 → 寫設定檔 → 套用計畫」，`workspace_init`／`generate_tool`／`skill_targets` 刪除。依賴刀 A：沒有「先算計畫再套用」的拆分，init 無法在寫設定檔之前用同一份目標集守門。

**查證到的兩個事實（決定刀 B 的形狀）**：
1. `write_if` 的「不覆寫既有檔」不是契約。workspace-tools 規格沒有任何 scenario 說 init 不帶 --force 要保留既有技能檔；`init` 在 `spec_root` 或 `.speclink.yaml` 存在時本來就先 bail（init.rs:74），所以 force=false 時 `write_if` 唯一能碰到「檔已存在」的情境是「沒有 .speclink.yaml、沒有 openspec/、但 .claude/skills/speclink-* 已在」——這種殘骸被靜默保留反而是漏洞，走 update 一律改寫更對（領先版本另有降級守門擋）。
2. 規格 workspace-tools:380 要求守門拒絕時「SHALL NOT 寫入任何檔案（含設定檔）」，且明列 `speclink init --force`。所以刀 B 不能是「init 寫完 .speclink.yaml 再呼叫 update() 讓它守門」——那會留下設定檔。正確形狀是 update 內部先拆成 `plan(root, selection)`（算目標集）與 `apply(plan)`（寫入），init 用記憶體中的選集先算計畫、守門、寫設定檔、再 apply。這正是 `reconcile_builtin_tools` 今天用「提前重算守門目標」手工模擬的東西（init.rs:381-393）。

**Ruled out**: 五個候選一刀合做——刀 B 會動 init 的可觀察行為（init 開始清孤兒目錄、處理描述子、降級守門改由計畫承載），需要動 workspace-tools 規格；與零行為變化的刀 A 混在一起，review 時分不出「重構造成的 diff」與「刻意的行為變化」。每候選各一刀（五刀）——候選 1 與 2 的產出是同一個值（計畫）的兩個維度，拆開會留下中間態各自半套；候選 4、5 各只有幾十行，單開 change 流程成本高於內容。

**Open**: 切刀方式是否照上述兩刀？候選 4 放刀 A 還是留待下次（它與計畫無直接關係，只是同檔順手）？

### Round 3 — interview (2026-09-07)

**Focus**: 刀 A 的計畫介面形狀——介面深度四項檢查，以及探測是否納入描述子
**Position**: 計畫型別 `SyncPlan` 立在 core `init.rs` 內，介於「讀設定與政策」和「碰磁碟」之間。四項檢查：
1. **接縫位置**：輸入＝root、`ToolSelection`（由 AppConfig 解析，或 init 時由記憶體選集直接建）、worktree 政策（一次同步讀一次）、spec_dir；輸出＝一組 target，每個 target 帶 skills_root、預期檔案表（`speclink-<name>/SKILL.md` → 內容）與遺留剝除的指令檔路徑。今天四個 target 消費端（update 生成、update 清孤兒、probe 差異、守門目標）各自從 root 重新推導這組資料，接縫放在這裡才讓推導只發生一次。
2. **adapter 數量**：一個。update ＝ resolve → guard → apply；probe ＝ resolve → diff；reconcile ＝ resolve（用新選集）→ guard → 寫設定檔 → apply。沒有薄包裝層——`generate_tool`、`generate_custom`、`expected_skill_dirs`、`differing_managed_files`、`skill_targets` 五支全部消失，不是被改寫成轉發。
3. **深度**：介面後面藏的行為——for_codex 子集、worktree 門檻（含壞 config 保留技能的安全方向）、tools 空清單時的 `.claude` 目錄偵測回退、描述子驗證與去重、未知內建名的警告、自訂足跡的 prune 目標。四支消費端今天各自重寫其中一部分，沒有任何一支完整持有全部規則。
4. **刪除測試**：現況就是「沒有計畫」的樣子——四份技能集合＋四份選集解析。立計畫後刪掉上述五支函式，它們的工作被 `apply`／`diff` 吸收，同一行為只剩一處。通過。

**查證事實**：
- 探測納入描述子會改 desktop 可觀察行為：描述子指到的 skills_dir 不在時，探測會從「現版」變「缺失」，desktop 隨即跳更新提示。規格「技能檔過期探測」寫「依 tools 清單、各工具」，字面上不排除描述子，但今天的實作明文「第一版不涵蓋」，而刀 A 的承諾是零行為變化。
- 候選 4 的檔尾不對稱（custom 補 `\n`、builtin 不補）被 neutral 與 claude／codex 四份 golden 位元級鎖住。合一後兩軌都做「收斂到恰一個結尾換行」，結果只要 golden 全綠即等價，不需要決策。

**Ruled out**: 計畫做成 trait 或獨立模組供 desktop 直接建構——desktop 三個消費端（project.rs、settings.rs、connections.rs）全走 core 公開函式，沒有自建計畫的需求，多一層是空轉接。用 closure 參數化 generate／prune／diff 的過濾規則——規則仍是被各處傳入，擁有者還是不存在。

**Open**: 「探測涵蓋描述子」放哪裡？我建議留給刀 B（刀 B 本來就要動 workspace-tools 規格，描述子探測在那裡補一個 scenario 就能收）；刀 A 的 probe 只讀計畫的 builtin 子集，行為零變化。

### Round 4 — interview (2026-09-07)

**Focus**: 刀 B（候選 3）的介面深度四項檢查，以及它帶來的行為變化要不要全收
**Position**: 刀 B 的形狀＝五個公開入口都變成「前置條件 ＋ 同一份計畫」的組合：
- `init`：已初始化守門 → 用記憶體選集建計畫 → 降級守門 → store_init → 寫 `.speclink.yaml`（維持今天的樣板重寫語意）→ gitignore → apply
- `init_remote`：同上，少 store_init、多 remote section
- `adopt`：store_init → gitignore → reconcile
- `reconcile_builtin_tools`：改寫 tools → 建計畫 → 守門 → 寫設定檔 → apply（今天的「提前重算守門目標」手工段落消失）
- `update`：載入設定 → 建計畫 → 守門 → apply
四項檢查：(1) **接縫**：入口與計畫之間；每個入口只剩它獨有的前置條件，生成邏輯零份。(2) **adapter 數量**：五個入口不是疊起來的薄包裝——每個都加一條真實前置條件（已初始化拒絕、store 骨架、remote section、tools 改寫），沒有一個是純轉發；`workspace_init`、`generate_tool`、`skill_targets` 三支純轉發刪除。(3) **深度**：入口藏住「守門必須在設定檔寫入之前」這條時序（規格 workspace-tools:380 的「含設定檔零寫入」），今天 `reconcile` 用註解與手工重算來守，`init` 用另一支 `skill_targets` 來守。(4) **刪除測試**：刪掉 `workspace_init`／`generate_tool`／`skill_targets`，工作被 apply 吸收；`reconcile_builtin_tools` 的 doc comment「CLI init、remote init 與 desktop 共用的單一入口」終於為真。通過。

**查證事實**：
- 規格「update 清除孤兒技能目錄」（workspace-tools:502）只寫 `speclink update`；「init 不清理」只是程式註解（init.rs:553），不是規格條文。
- `init --force` 整份重寫 `.speclink.yaml` 為樣板＋tools（描述子、remote section 一併清掉）——沒有規格釘住，但它是「重新初始化＝重設」的合理語意，且 fs init --force 若保留 remote section 會把工作區變成 remote 模式，明顯錯。刀 B 維持此語意不動。
- 因此刀 B 真正的行為變化只有一條：**`init --force` 走計畫後會清掉被下架工具的技能足跡與孤兒 speclink-* 目錄**（今天留著）。例：原本 tools 為 claude，`init --force --tools codex` 今天會留下 `.claude/skills/speclink-*` 可載入；刀 B 後移除。這與 unify-agent-tool-bootstrap design 否決「只追加不 prune 會讓 config 與可載入 Skills 分歧」的理由同向。
- 另一條微小差異：init 對 CLAUDE.md／AGENTS.md 的遺留剝除從「只剝選中的工具」變「兩個都剝」（update 今天的做法）。剝除只動舊版 marker 區塊，對新專案是 no-op。

**Ruled out**: 刀 B 讓 `init --force` 改用「保留其他鍵」的 tools 改寫——會把 remote section 帶進 fs init，語意錯；刀 B 同時把「探測涵蓋描述子」以外的新功能塞進來——刀 B 的規格改動限定為「清孤兒與下架足跡擴及 init --force」＋「探測涵蓋描述子」兩條 scenario。

**Open**: `init --force` 開始清下架足跡與孤兒目錄——收為刀 B 的預期行為（規格 502 的主詞從 `speclink update` 放寬為「所有再生入口」）？我建議收。

### Round 5 — interview (2026-09-07)

**Focus**: 候選 4、5 的介面深度四項檢查，以及兩刀的立案順序
**Position**:
**候選 4（skills.rs 兩軌渲染合一）**：(1) **接縫**：`RenderTarget` 已是對外唯一入口——生產端消費者只有 `init.rs` 與 `speclink-node/src/render.rs`，兩者都走 `render_skill_file_for`；`render_skill_file`／`render_skill_file_custom`／`substitute_neutral` 沒有任何生產端外部消費者，`substitute` 只有 golden 測試一處呼叫（render_golden.rs:888）。接縫位置正確，問題只是接縫後面沒有合併。(2) **adapter 數量**：今天是 1 支轉發＋2 支 render＋2 支 substitute＝5；合一後 1 支 render＋1 張代換表。(3) **深度**：藏住三個 target 差異點——Claude 專屬 frontmatter 行、前言（fork 規則 vs invocation 說明）、代換表（slash 前綴、plan_dir、去 plan-mode 行、`/speclink-` 中性化）。(4) **刪除測試**：刪掉 custom 軌，其工作被單一 render 的 target 分支吸收，frontmatter 八行與代換鏈四段各只剩一份；四份 golden 位元級鎖住三種 target 的輸出，重構的驗收就是 golden 全綠。通過。

**候選 5（`.speclink.yaml` 讀改寫收進 config.rs）**：(1) **接縫**：「改一個鍵、其餘鍵原樣保留」的三個動詞（tools 改寫、remote section 寫入／移除）全部搬到 `config.rs` 與 `parse_yaml_mapping` 同住。(2) **adapter 數量**：解析器從 2 支變 1 支。(3) **深度**：藏住「壞檔即拒（不得靜默改寫）、null 視為空 mapping、註解不保留」三條讀改寫語意——今天 `load_app_yaml_doc` 與 `parse_yaml_mapping` 各寫一遍、錯誤訊息逐字相同。(4) **刪除測試**：刪 `load_app_yaml_doc`，兩支 remote 函式改呼叫 `parse_yaml_mapping`，行為零變化。通過。消費端（CLI connection.rs 三處、desktop connections.rs／remote.rs 兩處）只改 use 路徑。

**立案順序**：刀 B 的任務會寫在刀 A 重寫過的程式碼上（`init` 改組合計畫），先立刀 B 等於對還不存在的介面寫 tasks，apply 時必然 drift。建議：現在只轉出刀 A；刀 A 封存後，回到本討論「再轉出一個變更」立刀 B。與 improve-cli-verb-layer「先立刀 1，刀 2 隨後」同一做法。

**Ruled out**: 兩刀同時立案並用 worktree 平行做——刀 B 的檔案面是刀 A 的子集（init.rs 同一區段），平行必撞；候選 4 單獨立案——面積數十行，流程成本大於內容，隨刀 A 同檔順手。

**Open**: 立案順序照「先 A、A 封存後再轉出 B」？

## Conclusion

**Decision**: 五個候選全部落地，分兩刀依序立案。刀 A（零行為變化，候選 1＋2＋5＋4）：core `init.rs` 立一份同步計畫——`ToolSelection`（由 AppConfig 解析或 init 時由記憶體選集直接建；含內建去重、描述子驗證、未知名警告、tools 空清單時的 `.claude` 目錄回退）× `managed_skills`（for_codex 子集＋worktree 門檻＋目錄名，worktree 政策一次同步只讀一次）→ 一組 target（skills_root、預期檔案表、遺留剝除的指令檔路徑）。update ＝ resolve → guard → apply；probe ＝ resolve → diff（只讀 builtin 子集，行為零變化）；reconcile ＝ resolve（新選集）→ guard → 寫設定檔 → apply。`generate_tool`／`generate_custom`／`expected_skill_dirs`／`differing_managed_files`／`skill_targets` 五支刪除；desktop `connections.rs` 的 preselected_tools 改讀選集。`.speclink.yaml` 讀改寫三動詞（tools 改寫、remote section 寫入／移除）收進 `config.rs` 共用 `parse_yaml_mapping`，刪 `load_app_yaml_doc`。skills.rs 兩軌 render 與兩支 substitute 合成一支 render＋一張代換表，target 只決定三個差異點（Claude 專屬 frontmatter 行、前言、代換表），驗收＝四份 golden 位元級全綠。刀 B（有行為變化，候選 3）：五個入口改為「自己的前置條件＋同一份計畫」——`init`＝已初始化守門→記憶體選集建計畫→降級守門→store_init→寫設定檔（維持樣板重寫語意）→gitignore→apply；`init_remote` 同形少 store_init 多 remote section；`workspace_init`／`generate_tool`／`skill_targets` 刪除，`reconcile_builtin_tools` 的手工提前守門段落消失。刀 B 收兩條規格變更：workspace-tools「update 清除孤兒技能目錄」主詞放寬為所有再生入口（`init --force` 開始清下架足跡與孤兒目錄）；「技能檔過期探測」補描述子 scenario。
**Rationale**: 這一層只有兩個概念（工具選集、受管技能集合），卻各在四處重算，且已有兩次漏補實證：worktree-toggle-and-guards 落地時 probe 那份漏補門檻，政策關閉的專案被永久報過期；rename-onboard-to-baseline 為清孤兒只能再加第四份。計畫成為唯一擁有者後，守門與寫入集是同一個值、probe 與 update 結構上不可能漂移。兩刀以「有無可觀察行為變化」分界：刀 A 純重構、安全網現成（55 個 init.rs 測試＋四份 golden＋CLI 整合測試），review 時不會混入刻意的行為變化；刀 B 依賴刀 A 的「先算計畫再套用」拆分——規格 workspace-tools:380 要求守門拒絕時連設定檔都零寫入，沒有這個拆分，init 無法在寫設定檔之前用同一份目標集守門。介面深度四項檢查五個候選皆過站（Round 3、4、5）。
**Rejected alternatives**: 五個候選一刀合做——刀 B 動 init 可觀察行為需改規格，與零行為變化的刀 A 混做，review 分不出重構 diff 與刻意變化；每候選各一刀——候選 1 與 2 是同一個值的兩個維度，拆開留中間態，候選 4、5 各數十行不值單開；「探測涵蓋描述子」放刀 A——描述子 skills_dir 不在時探測從現版變缺失、desktop 跳提示，是行為變化，改歸刀 B；`init --force` 改用保留其他鍵的 tools 改寫——會把 remote section 帶進 fs init 使工作區變 remote 模式，語意錯，維持樣板重寫；計畫做成 trait 或獨立模組供 desktop 自建——desktop 三個消費端全走 core 公開函式，多一層是空轉接；用 closure 參數化過濾規則——規則仍被各處傳入，擁有者依舊不存在；兩刀同時立案用 worktree 平行——刀 B 檔案面是刀 A 子集（init.rs 同區段）必撞；先立刀 B——其 tasks 會寫在刀 A 尚未存在的介面上，apply 必 drift。
**Deferred**: 刀 B 立案時機——刀 A 封存後回本討論「再轉出一個變更」；刀 B 的 init 遺留剝除從「只剝選中工具」變「兩個指令檔都剝」屬微小差異，於刀 B propose 期決定是否寫進規格。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion improve-workspace-sync（先立刀 A；刀 A 封存後再轉出刀 B）
