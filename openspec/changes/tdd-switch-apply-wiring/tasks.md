## 1. 移除 .speclink.yaml 舊鍵相容層（解析縮為三層）

- [x] 1.1 先寫紅測試（落實 requirement「工作流政策的正典歸屬與三層解析順序」；design D4: 相容層移除的 serde 與向後相容）：改寫 crates/speclink-core/src/config.rs 的 resolve_policy 單元測試——刪除 legacy 層案例（tdd_old_app_key_wins_over_canonical、audit_old_app_key_wins_over_canonical 與 locale／spec_locale 的同型案例），新增「.speclink.yaml 政策鍵一律不生效」案例（app 設 locale 與 tdd 而 openspec/config.yaml 未設 → 有效值為內建預設）；跑 cargo test -p speclink-core 確認新案例失敗 <!-- speclink-task:tsk_01M0KYNVCWEWJJ8YMZBZ4G4SQK -->
- [x] 1.2 實作三層解析（承接 REMOVED requirement「工作流政策的正典歸屬與四層解析順序」；design D4）：crates/speclink-core/src/config.rs 移除 AppConfig 的 locale、spec_locale、tdd、audit 四欄與 deprecated_policy_keys()，resolve_policy 對四鍵各拿掉 app 層（形狀比照既有 worktree 鍵：env ＞ openspec/config.yaml ＞ 預設），並清掉因此孤兒化的 app-wins 輔助函式與註解（含檔頭說明、resolve_policy 的四層敘述與 locale／spec_locale 解析函式的 app-wins 註解）；驗證 cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M0KYNVCWC4Y8ED4QH88TKDCD -->
- [x] 1.3 移除 CLI 警告機制（承接 REMOVED requirement「舊政策鍵的 deprecation 警告」，並覆蓋 MODIFIED requirement「init 範本的政策寫入位置」的 update 無警告 scenario）：刪除 crates/speclink-cli/src/common.rs 的 warn_deprecated_policy_keys 與 crates/speclink-cli/src/main.rs 的呼叫點；刪除 crates/speclink-cli/tests/it/deprecation_warning.rs；改寫 crates/speclink-cli/tests/it/instructions_policy.rs 的 legacy 案例為「.speclink.yaml 政策鍵不生效且 stderr 無警告」（以 speclink instructions tasks --change --json 斷言 payload 與 stderr）；驗證 cargo test -p speclink-cli --test it 全綠，且 workflow_config 案例確認 MODIFIED requirement「workflow-config show 動詞」的人眼與 --json 輸出維持位元級不變（該條僅四層改三層的字句校正） <!-- speclink-task:tsk_01M0KYNVCWR0AZVHAHK5VSEN2W -->

## 2. instructions apply payload 帶 tdd／audit 有效值

- [x] 2.1 先寫紅測試（落實 requirement「instructions apply payload 的有效政策欄位」；design D1: 有效值由 payload 傳遞，解析責任收回引擎）：crates/speclink-cli/tests/it/instructions_policy.rs 新增案例，斷言 speclink instructions apply --change 某change --json 的 payload 含 "tdd" 與 "audit" 布林欄位（camelCase），值依 delta spec 的 Example 表四列（config.yaml 設值、未設、SPECLINK_TDD 覆寫兩向）；跑 cargo test -p speclink-cli --test it 確認失敗 <!-- speclink-task:tsk_01M0KYNVCW76BRY7GYVRSJQ571 -->
- [x] 2.2 core 組裝：crates/speclink-core/src/instructions.rs 的 ApplyInstructions 結構與 apply instructions 組裝函式新增 tdd、audit 欄位，值取 resolve_policy 結果（與既有 locale 欄位同一組裝點）；驗證 cargo test -p speclink-core 全綠且 2.1 案例轉綠 <!-- speclink-task:tsk_01M0KYNVCWVJ3816VVNW1PF9X2 -->
- [x] 2.3 wire contract（design D3: wire contract 增欄與版本偏斜 fail closed）：crates/speclink-protocol/src/query.rs 的 ApplyInstructions 新增 tdd 與 audit 布林欄位（刻意不加 serde(default)——舊 server 缺欄位時反序列化失敗即 fail closed，沿 Progress 寫碼計數先例）；JSON Schema 由既有 JsonSchema derive 自動導出、repo 無獨立匯出檔，無需另行再生；驗證 cargo test -p speclink-protocol 全綠 <!-- speclink-task:tsk_01M0KYNVCWB5RM6GYEWAJ6VT15 -->
- [x] 2.4 server 與 remote 映射（design D2: 唯一實作落點與 local／remote 共用）：crates/speclink-server/src/routes.rs 的 apply_instructions 映射函式與 crates/speclink-cli/src/verbs/instructions.rs 的 to_apply_instructions 各補 tdd、audit 兩欄（比照 locale 一行映射）；驗證 cargo test -p speclink-server 與 cargo test -p speclink-cli --test it（含 remote_verb_parity 與 remote_read_path 案例）全綠 <!-- speclink-task:tsk_01M0KYNVCWCV0Y29H1DDWAMNED -->
- [x] 2.5 desktop 重編驗證：protocol 結構變更後跑 apps/desktop/src-tauri 所屬 crate 的 cargo test（依慣例先補建 sidecar 與 apps/server-web 的 dist：npm run build -w apps/server-web 與 desktop sidecar 建置腳本）；驗證該 crate 測試全綠 <!-- speclink-task:tsk_01M0KYNVCWZ9EW8JEKH4VBJ69Q -->

## 3. 技能資產修正與 MARKER_VERSION／golden／assets.lock 三連動

- [x] 3.1 crates/speclink-core/assets/skills/apply.md 步驟 5「Check project preferences」改為讀取 apply instructions payload 的 tdd／audit 欄位並刪除「Read `.speclink.yaml`」字句；TDD 與 audit 紀律觸發後的內文（fetch `speclink instructions --skill tdd`／`--skill audit`、Red-Green-Refactor、bug fix 先寫重現測試）逐字保留 <!-- speclink-task:tsk_01M0KYNVCW41NDPMXPNDY5H06Z -->
- [x] 3.2 crates/speclink-core/assets/skills/tdd.md 刪除 Usage Modes 段與 Input 行的 standalone 呼叫描述，定位改寫為「apply 於 TDD 開啟時經 speclink instructions --skill tdd 取用」；「set in `.speclink.yaml`」字句改為 payload 語意 <!-- speclink-task:tsk_01M0KYNVCWDRS0N1AFQ0JRQXZT -->
- [x] 3.3 連帶字句修正：crates/speclink-core/assets/skills/ingest.md 與 propose.md 的 spec_locale 說明句移除 .speclink.yaml 選項（僅留 openspec/config.yaml）；onboard.md 的讀取指示改為自 {{SPEC_DIR}}config.yaml 取 spec_locale <!-- speclink-task:tsk_01M0KYNVCW5WE619V7SKRDGQ31 -->
- [x] 3.4 三連動落地（design D5: 技能資產改動與三連動）：crates/speclink-core/src/init.rs 的 MARKER_VERSION bump（patch 位）；依 render_golden 刻意更新流程再生 crates/speclink-core/tests/golden/ 的 claude.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md 與 assets.lock；驗證 cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M0KYNVCWY9CVYZCZYMQCC9TS -->
- [x] 3.5 以本 repo 新建置的 speclink binary（非安裝版）跑 speclink update，再生 claude 與 codex 兩工具的渲染技能檔；驗證 .claude/skills/speclink-apply/SKILL.md 含 payload 判斷語意且全檔無「Read `.speclink.yaml`」字句 <!-- speclink-task:tsk_01M0KYNVCW5K85R2SBJ2G2RWTF -->

## 4. 文件同步

- [x] 4.1 docs/configuration.md 與 docs/configuration.zh-TW.md 移除 .speclink.yaml 舊政策鍵相容層與 deprecation 警告的說明，政策解析順序改為三層（環境變數 ＞ openspec/config.yaml ＞ 內建預設）；驗證兩檔對政策鍵的敘述與 delta spec 一致且無殘留 deprecation 字句（grep -i deprecat docs/configuration.md docs/configuration.zh-TW.md 無政策鍵相關命中） <!-- speclink-task:tsk_01M0KYNVCWKRKXV9XQPGMFR04H -->

## 5. 收尾驗證

- [x] 5.1 逐 crate 全量驗證並盤點提交面：cargo test -p speclink-core、cargo test -p speclink-protocol、cargo test -p speclink-remote、cargo test -p speclink-server、cargo test -p speclink-cli --test it、desktop crate cargo test 全綠；跑 speclink validate tdd-switch-apply-wiring 確認 delta 依 design D6: workflow-config spec 的 delta 形狀（REMOVED＋ADDED 成對宣告）可乾淨套用；git status 盤點——golden 四快照、assets.lock、speclink update 再生的全部渲染 SKILL.md 均納入提交清單，工作樹無未認領檔案 <!-- speclink-task:tsk_01M0KYNVCWA8D82X7ZH8PP05E5 -->
