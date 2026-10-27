## Context

政策開關（tdd、audit）的有效值解析在引擎端是對的（環境變數 ＞ .speclink.yaml 舊鍵 ＞ openspec/config.yaml ＞ 預設），instructions tasks 端也已消費；壞掉的是 apply 消費端：apply.md 資產叫 agent 自讀 `.speclink.yaml`，正典搬家後照字面執行開關靜默失效。同時使用者已裁定移除舊鍵相容層（第一個正式版即無舊鍵使用者）。本變更跨 speclink-core（解析與 payload 組裝）、speclink-protocol（wire contract）、speclink-server（route 映射）、speclink-cli（fs／remote 兩路映射與警告移除）、技能資產與文件。

## Goals / Non-Goals

**Goals**
- apply instructions payload（fs 與 remote）帶引擎解析完的 `tdd`、`audit` 有效值
- apply.md 依 payload 判斷紀律開關；tdd.md 移除過時字句與 standalone 死文字
- 政策解析縮為三層：環境變數 ＞ openspec/config.yaml ＞ 內建預設；deprecation 警告機制整組移除

**Non-Goals**
- 不渲染 standalone tdd 技能檔；不改 TDD／audit 紀律內文；不動 instructions tasks 端；不動 .speclink.yaml 的應用層鍵（spec_dir、tools、remote）；不提供舊鍵搬移工具

## Decisions

### D1: 有效值由 payload 傳遞，解析責任收回引擎

apply.md 不再指示 agent 讀任何設定檔；`speclink instructions apply` 的 payload 新增 `tdd`、`audit` 布林欄位（camelCase，經 #[serde(rename_all = "camelCase")]），值取自引擎的 resolve_policy 結果——與既有 `locale` 欄位完全同機制、同一組裝點（crates/speclink-core/src/instructions.rs 的 apply instructions 組裝函式，locale 欄位旁）。替代案「apply 技能自跑 workflow-config show」被否決：show 顯示正典值而非有效值（不含環境變數覆寫），且多一次指令、技能文字仍可能漂移。

### D2: 唯一實作落點與 local／remote 共用

有效值計算只在 speclink-core 的 resolve_policy 一處。三條消費路徑共用它：
- fs 模式 CLI：core 直接組裝 ApplyInstructions（含新欄位）
- server：crates/speclink-server/src/routes.rs 的 apply_instructions 映射函式把 engine 欄位逐一搬進 protocol ApplyInstructions（比照 locale 欄位一行映射）
- remote 模式 CLI：crates/speclink-cli/src/verbs/instructions.rs 的 to_apply_instructions 把 protocol 欄位映回 core 型別（同樣比照 locale）

不平行實作第二套解析（回歸對照：crates/speclink-cli/tests/it/remote_verb_parity.rs）。

### D3: wire contract 增欄與版本偏斜 fail closed

crates/speclink-protocol/src/query.rs 的 ApplyInstructions 新增 `pub tdd: bool` 與 `pub audit: bool`，刻意不加 serde(default)：舊 server 的回應缺欄位時，新 client 反序列化失敗即報錯。理由承 Progress 寫碼計數欄位的既有先例——缺欄位若默認 false 會靜默關掉 TDD 紀律，正是本次要修的病；fail closed 讓版本偏斜可見。JSON Schema 由型別上既有的 JsonSchema derive 自動導出，repo 無獨立匯出檔、無需再生步驟。protocol struct 變更後 speclink-desktop crate 需重新編譯驗證（該 crate 依賴 protocol；測試前需手補 sidecar 與 server-web dist）。

### D4: 相容層移除的 serde 與向後相容

crates/speclink-core/src/config.rs：
- AppConfig 移除 `locale`、`spec_locale`、`tdd`、`audit` 四個政策欄位。AppConfig 標註「unknown keys are ignored」（無 deny_unknown_fields），故仍含這些鍵的舊 .speclink.yaml 照常解析、鍵靜默不生效——與既有 worktree 鍵的行為一致，「能讀既有檔案」成立。`spec_dir`、`tools`、`remote` 欄位不動。
- resolve_policy 內四個政策欄位的解析各拿掉 app 層，落與 worktree 相同的三層形狀（env ＞ config.yaml ＞ default）；worktree 本就三層，不動。
- deprecated_policy_keys() 刪除；crates/speclink-cli/src/common.rs 的 warn_deprecated_policy_keys() 與 crates/speclink-cli/src/main.rs 的呼叫點刪除。
- 因本次改動孤兒化的 app-wins 解析輔助函式與註解一併清掉；resolve_policy 的單元測試中 legacy 層案例（tdd_old_app_key_wins_over_canonical 等）刪除。

環境變數層、fail-closed（壞 config.yaml 拒跑）、布林值僅收 true/false 的語意全部不變。

### D5: 技能資產改動與三連動

資產改動範圍（事實來源 crates/speclink-core/assets/skills/）：
- apply.md 步驟 5：改為「讀 apply instructions payload 的 tdd／audit 欄位」，刪「Read `.speclink.yaml`」；TDD／audit 紀律觸發後的內文（fetch --skill tdd／audit、Red→Green→Refactor、bug fix 先重現）逐字保留。
- tdd.md：刪 Usage Modes 段與開頭 Input 行的 `/speclink:tdd` standalone 描述，改寫為單一定位（由 apply 在 TDD 開啟時取用）；「set in `.speclink.yaml`」改為 payload 語意。
- ingest.md、propose.md 的 spec_locale 說明句與 onboard.md 的讀取指示：移除 .speclink.yaml 選項，僅留 openspec/config.yaml。

資產內文變更觸發三連動：crates/speclink-core/src/init.rs 的 MARKER_VERSION bump（patch 位）→ golden 四快照（claude／codex／neutral-cli／neutral-tool-call）與 assets.lock 再生（cargo test render_golden 的刻意更新流程）→ 實作收尾時跑 speclink update 讓本 repo 兩工具的渲染技能檔更新；再生的 SKILL.md 不進 evidence，收尾 commit 以 git status 盤點帶上。

### D6: workflow-config spec 的 delta 形狀

- Requirement「工作流政策的正典歸屬與四層解析順序」名稱含層數，改名即引擎眼中的未宣告刪除——delta 以 REMOVED（原名）＋ ADDED（「工作流政策的正典歸屬與三層解析順序」）成對宣告；新 requirement 承接原 scenarios，刪「舊鍵相容層勝過正典值」，並把「.speclink.yaml 的 worktree 鍵不生效」一般化為五鍵一律不生效。
- Requirement「舊政策鍵的 deprecation 警告」整條 REMOVED。
- Requirement「init 範本的政策寫入位置」MODIFIED：scenario「既有專案不受範本變更影響」的 THEN 拿掉警告子句。
- Requirement「workflow-config show 動詞」MODIFIED：內文「有效值的四層解析屬 instructions payload 職責」改三層。
- ADDED Requirement：instructions apply payload 的有效政策欄位（tdd、audit，fs 與 remote 形狀一致，版本偏斜 fail closed）。

## Risks / Trade-offs

- **回歸對照**：golden 四快照與 assets.lock 屬刻意更新，須經 render_golden 流程再生而非手改；CLI 測試中 deprecation_warning.rs 整檔移除、instructions_policy.rs 的 legacy 案例改寫，變更面大，依 TDD 先改測試再動實作。與進行中變更 desktop-schema-panel 無共檔，但若期間出現其他動資產的平行變更，版號行對撞的解法是重生衍生物、不是挑邊。
- **跨平台**：無 git 互動、無路徑分隔敏感點；環境變數解析與檔案讀取沿用既有程式路徑，Windows／macOS／Linux 行為不變，CI 全綠基準沿用。
- **版本偏斜**：新 CLI 對舊 server 會因缺欄位報錯而非降級運作——刻意選擇，錯誤訊息由 serde 反序列化錯誤承載，不另做版本協商。
- **storage 解耦方向**：政策解析與 payload 組裝都在 core、經儲存介面讀 config，本變更把技能端的「直讀檔案」改為「經 payload」，即朝 storage 解耦的規格驅動引擎靠攏（技能不再假設本地檔案系統佈局）。

## Migration Plan

無資料遷移。含舊政策鍵的 .speclink.yaml（理論上不存在）行為由「生效＋警告」變「不生效、無警告」；需要時手動把鍵搬入 openspec/config.yaml。回滾即 revert commit，無狀態殘留。

## Open Questions

（無）
