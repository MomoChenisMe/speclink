## 1. 工具解析與 CLI 選擇

- [x] 1.1 Red：先釘住「tools 自訂描述子的接受與驗證」與「init 內建 Agent 工具選擇」（D1 工具解析與相容性、D4 桌面與 CLI 完整入口）：在 `crates/engine/speclink-core/src/workspace/config/tests.rs`、`crates/adapters/speclink-cli/tests/it/init_tools.rs`、`crates/adapters/speclink-cli/tests/it/tools_descriptor.rs` 新增 copilot／agents 別名、三工具互動、pipe缺參數零寫入、custom copilot 遷移拒絕與 --no-color 測試；驗證 cargo test -p speclink-core copilot 及 cargo test -p speclink-cli --test it copilot，先記錄預期失敗。 <!-- speclink-task:tsk_01M4BD2YXNZQ11Q233XKZJ3V4C -->
- [x] 1.2 Green／Refactor：在 `crates/engine/speclink-core/src/workspace/skills.rs`、`crates/engine/speclink-core/src/workspace/config.rs`、`crates/engine/speclink-core/src/workspace/init.rs`、`crates/adapters/speclink-cli/src/verbs/init.rs` 接入 D1 工具解析與相容性、D4 桌面與 CLI 完整入口；copilot 是內建、agents仍對應codex，CLI選擇及成功／失敗stdout／stderr契約符合規格，custom copilot 在寫入前拒絕且附兩種人工遷移；取得 speclink instructions --skill audit 並檢查保留名／路徑邊界；驗證1.1測試轉綠及 cargo test -p speclink-cli --test it init_tools，刪除本次孤兒程式碼。 <!-- speclink-task:tsk_01M4BD2YXNYS21T6BT39D0TT27 -->

## 2. 共用技能內容

- [x] 2.1 Red：在 `crates/engine/speclink-core/src/workspace/skills.rs` 的測試與 `crates/engine/speclink-core/tests/it/render_golden.rs` 新增「中性渲染目標」與「共享技能的實際執行工具標記」回歸（D2 共享渲染與執行者標記）：三種共享選集逐檔相同、技能引用不是CLI子指令、無固定$／slash前綴或未解析模板、new change與stamp的工具標記前言、worktree／語言政策、Claude及custom本文不漂；驗證 cargo test -p speclink-core copilot，先記錄預期失敗。 <!-- speclink-task:tsk_01M4BD2YXNY94ZG9VN14BVBXBC -->
- [x] 2.2 Green／Refactor：在 `crates/engine/speclink-core/src/workspace/skills.rs` 的現有renderer完成 D2 共享渲染與執行者標記；Codex／Copilot共用替換與前言，移除共享命令範例的靜態--agent並指示依實際代理追加值、未知省略，保持有效CLI動詞及既有非Claude技能集合；驗證2.1轉綠及 CLI fixture 的有效命令／缺席工單錯誤，本文golden差異須限共享target並於6.1同批更新；不複製資產或改既有custom渲染。 <!-- speclink-task:tsk_01M4BD2YXN0ZBT38E2KGSS4XJS -->

## 3. 共享目錄生命週期

- [x] 3.1 Red：在 `crates/engine/speclink-core/src/workspace/init/tests.rs`，新增 `crates/adapters/speclink-cli/tests/it/copilot_tools.rs` 並在 `crates/adapters/speclink-cli/tests/it/main.rs` 註冊「built-in tools 權威收斂」、「技能檔過期探測」與「共享受管目錄的更新守門」測試（D3 共享目錄同步與清理）：共選只寫一套、選集切換本文不變、最後停用才清理／保留my-skill、CRLF不誤報、共享probe逐工具結果一致及differingFiles去重、較新guard／顯性降級、remote init不落可寫openspec；驗證 cargo test -p speclink-core copilot、cargo test -p speclink-cli --test it copilot_tools，記錄預期失敗。 <!-- speclink-task:tsk_01M4BD2YXNDREADP305SC14W7N -->
- [x] 3.2 Green／Refactor：在 `crates/engine/speclink-core/src/workspace/init.rs` 完成 D3 共享目錄同步與清理：SyncPlan依共享目錄去重、清理依任一啟用者保留、updated保留真實工具名、probe只讀共享目錄一次再回報各工具、guard不繞過；驗證3.1轉綠、stdout成功工具摘要及 JSON的status:string、tools:array、workspaceVersion:string/null、stale／newer／missing:boolean、differingFiles:string[]，配置／I/O失敗符合零寫入或既有可重試收斂；CLI測試包含 --no-color 及自訂使用者檔保全。 <!-- speclink-task:tsk_01M4BD2YXN60BF4M98JB7MTX5M -->

## 4. 桌面設定與初始化

- [x] 4.1 Red：在 `apps/desktop/core/src/settings.rs`、`apps/desktop/core/src/project.rs` 的測試，以及 `apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`、`apps/desktop/src/__tests__/workspaceChooser.test.tsx`、`apps/desktop/src/__tests__/assetUpdatePrompt.test.tsx`、`apps/desktop/src/__tests__/workspace.test.ts`、`apps/desktop/src/__tests__/messages.test.ts` 先涵蓋「未初始化目錄經確認後自動初始化」、「設定頁圖形化讀寫兩層設定」、「未啟用資料夾經確認後補齊啟用」、「checkout 工具選擇與共享技能驗收」（D4 桌面與 CLI 完整入口）：copilot預選／保存、只勾copilot、共享停用、custom衝突輸入保留、remote Workflow邊界、zh-TW／en文案；驗證 cargo test -p speclink-desktop-core copilot 與 npm test -w apps/desktop，先記錄新增案例失敗。 <!-- speclink-task:tsk_01M4BD2YXN5PZGHKYM1K164CBM -->
- [x] 4.2 Green／Refactor：在 `apps/desktop/core/src/settings.rs`、`apps/desktop/core/src/project.rs`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/components/WorkspaceChooser.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/components/AssetUpdatePrompt.tsx`、`apps/desktop/src/adapter/workspace.ts`、`apps/desktop/src/i18n/messages.ts` 完成 D4 桌面與 CLI 完整入口；工具選取／新建／啟用／checkout綁定共用core同步、預設與遠端policy行為維持、提示勾選只管理生成物；驗證4.1轉綠、app.tools／ProjectProbe.tools為真實string array與camelCase欄位，無新IPC／桌面writer；取得audit指示檢查工具保存的邊界及錯誤。 <!-- speclink-task:tsk_01M4BD2YXNECFHFBWRJM2X81EK -->

## 5. Node SDK

- [x] 5.1 Red：在 `crates/adapters/speclink-node/__test__/render.spec.ts` 先增加「渲染 API」案例（D5 SDK 渲染一致性）：copilot與codex逐位元相同、與CLI共享檔一致、unknown target／skill拋錯、claude／neutral本文回歸；先 npm --prefix crates/adapters/speclink-node run build，再 npm --prefix crates/adapters/speclink-node test，記錄copilot target的預期失敗。 <!-- speclink-task:tsk_01M4BD2YXNG436BDGHBNG0WQGQ -->
- [x] 5.2 Green／Refactor：在 `crates/adapters/speclink-node/src/render.rs`、`crates/adapters/speclink-node/index.d.ts` 完成 D5 SDK 渲染一致性；新增copilot target、委派相同core renderer、有效名錯誤清單與TS union一致、保留neutral invocation與toolName，不新建工具層；驗證重新建置binding及5.1轉綠，不以Rust cargo harness代替Node測試。 <!-- speclink-task:tsk_01M4BD2YXNXE6KTTMKZCY1XMMQ -->

## 6. 資產與文件

- [x] 6.1 完成 D6 資產鎖與操作文件的資產部分：在 `crates/engine/speclink-core/src/workspace/init.rs` 遞增ASSET_VERSION，經 UPDATE_GOLDEN=1 與 UPDATE_ASSETS_LOCK=1 的既有core render_golden測試再生 `crates/engine/speclink-core/tests/golden/`；審閱Codex本文的刻意變更，確認Claude／neutral只變版本戳，三種共享選集及worktree矩陣一致，再在關閉再生開關下跑 cargo test -p speclink-core --test it render_golden:: 與 cargo test -p speclink-cli --test it。按本checkout原選集再生 `.agents/skills/`，不補回使用者刪除的`.claude`檔或擅改原tools；對生成物作相同內容核對。 <!-- speclink-task:tsk_01M4BD2YXNVPH1PZBYDW564RK8 -->
- [x] 6.2 完成 D6 資產鎖與操作文件的文件部分：更新 `docs/configuration.zh-TW.md`、`docs/configuration.md`、`docs/getting-started.zh-TW.md`、`docs/getting-started.md`、`docs/sdk-node.zh-TW.md`、`docs/sdk-node.md`，用有效CLI示例說明copilot內建名、兩工具共用與停用、copilot描述子遷移、兩入口技能呼叫與實際--agent；修正 `scripts/claude-code/skill-groups.test.mjs` 以核心registry比對群組表，保留新增未分組／已移除技能的失敗斷言，不依賴本checkout勾選Claude；驗證中英逐節對等、示例不存在虛構apply／propose子指令，以及 node --test scripts/*.test.mjs scripts/*/*.test.mjs 的文件／詞彙／連結守門。 <!-- speclink-task:tsk_01M4BD2YXN2PPMWDAJZVPAXXZG -->

## 7. 實機確認與收尾

- [x] [M] 7.1 使用者在有有效Copilot權限的Copilot CLI驗收「共享技能的實際執行工具標記」及「checkout 工具選擇與共享技能驗收」：在暫存fixture選codex,copilot，核對技能來源及共享本文、使用speclink-propose建驗收變更，從該fixture的`.openspec.yaml`確認created_with=copilot；再選claude,codex,copilot確認同名技能載入仍符合共享契約。將工具版本、實際技能路徑、操作與結果記入 `openspec/changes/copilot-project-tool/` 的驗收記錄；無帳號／環境時保持未勾選，不用檔案存在代替驗收。 <!-- speclink-task:tsk_01M4BD2YXN866YZZXD2VXKN664 -->
- [x] [M] 7.2 使用者在VS Code的Copilot agent chat重做7.1的技能來源／共享本文／建立變更歸屬與Claude共存確認，另驗證桌面工具勾選從codex,copilot切成copilot時保留`.agents/skills`，改成claude才清理受管技能而保留my-skill；將工具版本與結果記入 `openspec/changes/copilot-project-tool/` 的驗收記錄，未實測不勾選。 <!-- speclink-task:tsk_01M4BD2YXNHXJNXM3A1BFKTH88 -->
- [x] 7.3 對 `openspec/changes/copilot-project-tool/` 與本次觸及面完成收尾：確認三份delta需求及D1至D6皆有實作／測試或明示未完成的[M]驗收，跑受影響面的 cargo test -p speclink-core、cargo test -p speclink-cli、cargo test -p speclink-desktop-core、npm test -w apps/desktop，以及Node binding build後的Node測試；確認golden及CLI契約、JSON欄位與型別，最後必跑 node --test scripts/*.test.mjs scripts/*/*.test.mjs 與 speclink validate copilot-project-tool --strict，不跑全量test:all；記錄未完成驗收，不宣稱支援已完成。 <!-- speclink-task:tsk_01M4BD2YXNJJMA4DNKMWSB6AF7 -->

## 驗證記錄

已通過：speclink-core 1,040 項、speclink-cli 517 項、speclink-desktop-core 276 項、桌面 Vitest 725 項、Node SDK 50 項、scripts 287 項，及 strict 規格驗證。18 份本 checkout 共享技能與 Codex／Copilot SDK 渲染一致；Claude／custom golden 除版本戳外不變。CLI 測試仍有原有 trace.rs 未使用 Path import 警告，本次未修改。

[M] 7.1 與 7.2 的 Copilot CLI／VS Code 技能來源、共享本文、提案建立者，以及桌面只留 Copilot 時保留技能，已由使用者實測確認（詳見以下記錄）。Claude 共存及只勾 Claude 的清理實機測試依使用者決定略過，不宣稱通過；手動任務勾選仍由使用者完成。

## 實機驗收範圍調整（2026-10-09）

- 使用者將 VS Code 測試目錄改為 `/Users/mosu/project/speclink-test`；後續 7.2 操作以此目錄為準。
- 依使用者決定，7.1／7.2 的 Claude 共存實機測試，以及 7.2 只勾 Claude 時的共享技能清理實機測試，記為略過，不宣稱已通過。
- 剩餘實機驗收為 Copilot CLI／VS Code 的實際技能來源、Shared execution 本文、提案建立者標記，以及桌面從 codex,copilot 改為只勾 copilot 時保留共享技能。7.1／7.2 目前仍保持未勾選，等待剩餘結果。

## 實機驗收結果（2026-10-09，使用者截圖）

- 在 `/Users/mosu/project/speclink-test`，CLI 的 `cli-check-demo` 與 VS Code 的 `vscode-check-demo` 均有 proposal.md、specs/、tasks.md；兩份 metadata 的 created_with 均為 copilot。
- Copilot CLI 截圖已顯示技能從此專案 `.agents/skills/speclink-propose` 載入，且本文含 Shared execution；VS Code 的實際技能來源／本文仍待聊天記錄佐證。
- 使用者提供的工具保存結果是 tools 只剩 codex，共享 speclink-propose 與自訂 my-skill 都存在；這確認取消 Copilot 後保留技能的方向。7.2 原定只剩 copilot 的方向仍待確認，不以此反向案例代替。
- Claude 相關實機驗收依前述決定略過；不代替使用者勾選手動任務。

## 共享技能保留補測（2026-10-09）

- 使用者補測 App 工具勾選，`/Users/mosu/project/speclink-test/.speclink.yaml` 的 tools 僅含 copilot；`.agents/skills/speclink-propose/SKILL.md` 與 `.agents/skills/my-skill/SKILL.md` 均存在。使用者截圖與本機讀取結果一致，取消 Codex 後保留共享技能的驗收通過。
- Copilot CLI 技能來源、Shared execution 本文、提案與 created_with 已確認。VS Code 提案與 created_with 已確認；僅剩 VS Code 實際技能載入來源及 Shared execution 本文待聊天記錄或使用者確認。Claude 相關實機測試仍依使用者決定略過。

## 本輪驗收結論（2026-10-09）

- 使用者提供 VS Code Copilot 截圖，回報實際載入 `/Users/mosu/project/speclink-test/.agents/skills/speclink-propose/SKILL.md`，並完整呈現 Shared execution 本文，包含實際代理的 --agent 規則；來源與共享本文驗收通過。
- Copilot 回報載入內容未顯示版本；直接讀取該來源檔頭確認 metadata.version 為 v1.43.0。此差異不影響已確認的來源／本文。Speclink 引擎 v1.43.0、Copilot CLI 1.0.94；本機安裝的 VS Code 版本 1.141.0，Copilot 擴充套件版本未取得。
- 本輪使用者選定範圍均已通過：CLI／VS Code 載入共享技能並完成提案、兩份 created_with=copilot、App 只留 Copilot 仍保留受管技能與 my-skill。Claude 相關實機測試略過；未以自動測試或檔案存在冒充這些略過項目的實機結果。
- 7.1／7.2 的 [M] checkbox 保持原值，由使用者自行確認與勾選；品質 Review／Verify 尚未完成，未進行封存。


## 8. 已實作追加範圍的回填與核對

- [x] 8.1 回填 D7 系統語系預設與明確設定優先，涵蓋「工作流政策的正典歸屬與三層解析順序」：核對 `crates/engine/speclink-core/src/workspace/config.rs`、`config/tests.rs`、`instructions.rs`、`crates/host/speclink-host/{Cargo.toml,src/policy.rs,src/bridge.rs}` 與 `Cargo.lock` 的已實作行為，沿用 unset_languages_use_the_injected_system_locale、unset_spec_language_uses_system_independently_of_artifact_language、explicit_languages_override_system_defaults、CLI instructions_policy／config_fail_closed 及 Host 通過紀錄，確認明確值優先、兩個未設定語言各自取 OS、unknown／缺值落 en、不寫回 config；TDD 的預期失敗與轉綠沿用本次對話中已執行的紀錄，不虛構新增程式工作。 <!-- speclink-task:tsk_01M4FGW4RFQBBS045EC50FA7QK -->
- [x] 8.2 回填 D8 App 移除 auto 選項與舊值保留，涵蓋「設定頁圖形化讀寫兩層設定」、「設定頁政策下拉的未知值顯性呈現」：核對 `apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts` 與 `apps/desktop/src/__tests__/projectSettingsView.test.tsx`；沿用「舊 auto 設定可讀；選單僅有系統語系與明確語言，改選未設定後送 null」的互動測試及 messages 測試，確認只有四個選項、舊值只讀保留、明確改選才送 specLocale:null，沿用桌面兩個受影響測試檔 69 項通過的紀錄。 <!-- speclink-task:tsk_01M4FGW4RFD01PDAZAD07MY1C8 -->
- [x] 8.3 回填 D9 語言說明、資產與既有證據回填，涵蓋「init 範本的政策寫入位置」、「workflow-config show 動詞」、「語言預設的技能說明」、「盤點前取得 workflow config 並套用 specs 產出規則」：核對 `crates/engine/speclink-core/assets/skills/{baseline,config,ingest,propose}.md`、`assets/schema/spec-driven/fork.schema.yaml`、`src/workspace/init.rs`、`tests/golden/`、`crates/adapters/speclink-cli/src/verbs/config.rs`、`docs/configuration{,.zh-TW}.md` 與 `.agents/skills/`；沿用 core render_golden 69 項整合、CLI 518 項、Node binding build 與 SDK 50 項通過紀錄，確認所有工具的語言本文刻意更新、canonical JSON 仍保留 null；沿用使用者 CLI／VS Code／App 截圖，不把 Claude 略過項目記成實測通過。 <!-- speclink-task:tsk_01M4FGW4RF4SH9R347CF13EX6Z -->
- [x] 8.4 完成「本輪回填的驗收與遷移補充」及 `openspec/changes/copilot-project-tool/` 文件同步：proposal／design／五份 delta 對齊 D7 至 D9，原 15 個 [x] 任務及驗收文字保留；核對新增 requirement 名稱及所有原有 scenarios 均被帶入，執行 speclink analyze copilot-project-tool --json 與 speclink validate copilot-project-tool --strict，最後執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs，確認沒有 Critical／Warning、原完成進度不倒退，以及討論 link／seal 仍連到本變更。 <!-- speclink-task:tsk_01M4FGW4RFP6F24B0P3G3R2MMK -->

## 已實作追加範圍的驗證證據

本輪回填不改應用程式碼。沿用 `/private/tmp/speclink-system-locale-core.log`（974 項 unit）、`speclink-system-locale-core-it.log`（69 項 integration）、`speclink-system-locale-cli-final.log`（16 項 unit、502 項 integration）、`speclink-system-locale-host-final.log`（115 項 unit、5 項 integration）、`speclink-remove-auto-desktop.log`（69 項）、`speclink-system-locale-node-build.log` 與 `speclink-system-locale-node.log`（SDK 50 項）、`speclink-system-locale-scripts.log`（287 項）以及上述使用者實機驗收。暫存 log 是本次執行證據，永久驗證目標為 task 所列的測試名稱及 package 測試入口。

CLI 的兩個新增場景確認：未設定欄位的 instructions 反映 OS 語言且 config 未被改寫；缺 config 檔時仍成功採 OS 預設。已在本機暫存 fixture 觀察到 canonical locale／specLocale:null、artifact 與 spec 指引均為繁中。此結果驗證本機 OS 判定；跨平台 mapping 使用可注入測試，未宣稱已於三種 OS 各自實機驗收。

使用者已自行勾選原 7.1／7.2 手動任務；前面「仍待確認／保持未勾選」段落為測試過程記錄，以本輪驗收結論與當前 checkbox 為準。Claude 相關實機測試略過的限制保留。追加核對任務已完成，沒有新增待實作程式任務；品質站的 Review／Verify 與封存仍由後續流程處理。

## 本輪 ingest 文件驗證

本輪分析已通過：0 Critical、0 Warning；Suggestion 僅為補充 example 的建議，未列為阻擋。strict validate 通過，scripts 287 項通過（`/private/tmp/speclink-ingest-scripts.log`）。核對原 15 個完成 task 逐字不變；workflow-config／baseline-skill／desktop-config 的 MODIFIED 區塊保留全部 canonical scenario 名稱，workspace-tools／node-sdk delta 逐位元不變。
追加 8.1 至 8.3 為已有實作與測試的回填核對，8.4 完成本輪文件驗證後由 CLI 記錄完成；原兩個 [M] checkbox 是使用者在 App 勾選，未由 ingest 代理代勾。linked copilot-project-tool 討論已透過 link／seal 再確認，未修改原討論內容。

## 9. 品質補救

- [x] 9.1 修正本輪 Review／Verify 的語言資訊缺口與測試缺項：Baseline 透過只讀 languages 查詢取得 Server 有效語言；補 Remote null／auto、Client 覆寫隔離與零寫入，補 ja writer 例值並保留 auto；同步語言 DTO／既有 config API／技能／文件／資產，完成完整建置測試及 strict validate，原 19 個完成任務與 Claude 實測略過記錄保留。 <!-- speclink-task:tsk_01M4GS99DZASVJ41QMXPSEEJT0 -->

## 品質補救驗證（2026-10-10）

依使用者選擇修正 Review 1 項、Verify 3 項 WARNING（Baseline 資訊缺口重複，因此實際 3 件事）。Baseline 新增只讀 workflow-config languages --json 來源；Host／GET /config 回傳 Server 的具體語言，canonical show JSON 保留 null。新增 CLI null／auto／明確值與不寫入測試、Remote 回應優先於 client 環境測試、舊 Server 缺欄停止並提示 upgrade 的測試；Bridge 新測試以 Server ja-JP、Client zh-TW 與繁中覆寫確認 proposal／specs 日文、零事件／零 staged writes／config 及 revision 不變。

新增 CLI 查詢及 Baseline 文字測試先失敗（缺 languages 子指令與技能步驟），完成實作後轉綠。政策欄位寫入 Example 的第三列在 Core 及 Desktop writer 都加入 ja，仍保留 auto 相容案例。完整流程首次發現 phase2_chain 的兩個舊測試寫死 English 預設；改以同機 Server OS 預設比對後，四項 phase2_chain 測試通過。

最終 npm run test:all exit 0：scripts 287、UI 611、Desktop 726、Server Web 146、Node SDK 50，Rust workspace 2,763 項通過；Server Web／Node binding 建置成功。另 npm run build -w apps/desktop 通過。PostgreSQL 26 項（Store 23、Server 3）依既有 harness 因無 SPECLINK_TEST_POSTGRES_URL 略過，Server 1 項既有 ignored 測試；未宣稱這些有實測。完整紀錄為 /private/tmp/speclink-quality-remediation-test-all-final.log，首次失敗紀錄為 speclink-quality-remediation-test-all.log，目標重跑為 speclink-quality-remediation-phase2.log。

ASSET_VERSION 已遞增 v1.44.0，golden／assets.lock 再生後完整 suite 在未開再生旗標時通過；本 checkout 按 copilot 選集同步 .agents/skills，未改 .speclink.yaml 或恢復 Claude 生成檔。原 19 個完成任務與手動略過範圍保留。strict validate 與 git diff --check 通過；品質票仍待第二輪驗證，尚未蓋章。

- [x] 9.2 修正 Verify Round 2 的未知語言值 WARNING：Host 的 languages 檢查正典語言值，CLI Remote 檢查正典值及具體回應碼；GET /config 的未知值仍可讀取修復而不輸出不合法 metadata。補 Local／Remote／實際 Server API 的未知值、case mismatch、auto 相依、零輸出／零寫入與 show 可讀測試，完成完整建置測試與 strict validate，保留原 20 個完成任務。 <!-- speclink-task:tsk_01M4HHG6H9MNFN5GXWY72S871N -->

## 品質第三輪驗證（2026-10-10）

剩餘的未知語言值問題已在新 languages 查詢邊界修正；canonical show 與設定頁維持可讀，GET /config 遇未知語言只省略可選 metadata，保留 content／schema／revision。Local／Remote 檢查正典語言值，Client 另檢查 Server 的具體 specLocale 回應。Host 未知值測試先失敗（舊 helper 成功回 zh-Hant）後轉綠。六項 CLI 語言回歸與實際 Server unknown-language API 測試通過。

完整 npm run test:all exit 0（/private/tmp/speclink-quality-round3-test-all.log）：Rust 2767、UI 611、Desktop 726、Server Web 146、Node SDK 50、scripts 287 通過，Server Web 與 Node binding 建置成功。PostgreSQL 26 項（Store 23、Server 3）因無資料庫略過；Server 的原 ignored 項目為由父測試啟動的 helper process。沒有改動 UI 或技能本文，ASSET_VERSION 保持 v1.44.0；桌面 build 沿用上一輪未變的 UI 產物，sidecar 同步本輪 CLI。strict validate 與 git diff --check 通過。原 20 個完成任務保留，9.2 完成後由 CLI 記錄；Review／Verify 尚待第三輪驗證，未蓋章。
