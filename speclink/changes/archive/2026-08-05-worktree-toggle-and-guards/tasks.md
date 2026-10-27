## 1. 引擎：政策條件式技能生成

- [x] 1.1 技能生成依 worktree 政策過濾：Skill 註冊表新增政策閘欄位（僅兩顆 worktree 技能為閘控），`crates/speclink-core/src/init.rs` 的生成迴圈讀 openspec/config.yaml 的 worktree 檔值（不走含 env 的四層解析）決定是否輸出，SPECLINK_WORKTREE 環境變數不影響生成（涵蓋 spec 需求「worktree 技能的政策條件式生成」）。先寫紅測：政策關（鍵缺席與 false 兩例）生成集合不含兩顆技能、政策開含之、env=true 且鍵缺席仍不含；再實作至綠。檔案：`crates/speclink-core/src/skills.rs`、`crates/speclink-core/src/init.rs`。驗證：cargo test -p speclink-core 新增單元測試通過。 <!-- speclink-task:tsk_01KZ6GTRZ6Z2P60SZME58P9PXP -->
- [x] 1.2 政策由開改關後再生即清理：worktree: true 下 update 生成兩顆技能，改 false 再 update 後兩顆技能目錄被 prune、其餘技能與 marker 保留（沿用既有清理生命週期）。以整合測試覆蓋 speclink update 的可觀察檔案效果。檔案：`crates/speclink-core/src/init.rs`、`crates/speclink-cli/tests/it/workflow_config.rs`。驗證：cargo test 整合案例通過。 <!-- speclink-task:tsk_01KZ6GTRZ6THNVF05611ERTKP5 -->

- [x] 1.3 marker 的 worktree 技能指引跟隨政策：內建工具 marker 內文的兩行 worktree 技能指引受同一政策閘控制（政策關即不輸出，其餘內容不變）；Node SDK 的 instructions.render 選項新增 worktree 軸（未給定＝false）；assets.lock 指紋輸入涵蓋政策開／關兩種 marker 變體（涵蓋 spec 需求「marker 技能指引跟隨 worktree 政策」與「渲染 API 的 worktree 軸」）。先寫紅測（政策關的 marker 不含兩行、政策開含之且僅差這兩行；node 兩軸內容對應），再實作。檔案：`crates/speclink-core/src/init.rs`、`crates/speclink-node/src/render.rs`、`crates/speclink-core/tests/it/render_golden.rs`。驗證：cargo test -p speclink-core 與 node 渲染測試通過。 <!-- speclink-task:tsk_01KZ6HRYWVJXNGGB3FMAM4ERK0 -->

## 2. CLI：workflow-config set worktree 的同步與擋下

- [x] 2.1 host 提供「可否關閉 worktree 政策」判定：以既有 discover 事實回傳擋下清單（change 名、分支、路徑），git 不可用時回空清單（fail-open）。先寫紅測（有映射→非空清單；無 git→空），再實作。檔案：`crates/speclink-host/src/worktree.rs`。驗證：cargo test -p speclink-host 通過。 <!-- speclink-task:tsk_01KZ6GTRZ62N16YCVR2HVYNYEH -->
- [x] 2.2 speclink workflow-config set worktree 寫入成功後同步技能足跡（重用 update 的生成與清理入口）：set true 後技能檔出現、set false（無活躍 worktree）後技能檔移除；同步失敗時 config 寫入保留、exit code 非 0 且 stderr 提示重跑 speclink update（與 2.3 合力涵蓋 spec 需求「worktree 欄位寫入的技能同步與關閉擋下」）。檔案：`crates/speclink-cli/src/commands.rs`、`crates/speclink-cli/tests/it/workflow_config.rs`。驗證：整合測試斷言檔案效果、exit code 與 stderr 內容。 <!-- speclink-task:tsk_01KZ6GTRZ6C8WRC0ABMT3K2T1A -->
- [x] 2.3 speclink workflow-config set worktree false 遇活躍 linked worktree 拒絕：exit code 非 0，stderr 逐列 change 名、分支與路徑並含收尾提示，openspec/config.yaml 位元組不變、技能足跡不動。檔案：`crates/speclink-cli/src/commands.rs`、`crates/speclink-cli/tests/it/workflow_config.rs`。驗證：整合測試以真實 git worktree fixture 斷言上述四項。 <!-- speclink-task:tsk_01KZ6GTRZ6AEHY08WW3KBFB1DM -->

## 3. 技能資產：兩道前置防護

- [x] 3.1 apply-with-worktree 前置新增 P0「多 change 輸入拒收」段：偵測多個 change 名→請使用者擇一並印多 session 配方，明文禁止靜默依序批次，位置在政策檢查（P1）之前（與 3.2 合力涵蓋 spec 需求「apply-with-worktree 技能的前置指示」）。檔案：`crates/speclink-core/assets/skills/apply-worktree-pre.md`。驗證：render_golden 斷言生成的 SKILL.md 含該段字面且順序正確。 <!-- speclink-task:tsk_01KZ6GTRZ6TTY4YP304QB44KG2 -->
- [x] 3.2 apply-with-worktree 前置新增 P3.5「進度與程式碼分家偵測」段：讀 change 目錄 .evidence.json 的 touched 清單對主樹查 git 狀態，evidence 缺席或空清單靜默續行；髒檔時停下列檔並依推薦序提供三選項（先走 speclink-commit／照樣繼續／停止），位置在 P3 之後、P4 建立 worktree 之前；內嵌資產版本戳遞增。檔案：`crates/speclink-core/assets/skills/apply-worktree-pre.md`、`crates/speclink-core/src/init.rs`（版本戳）。驗證：render_golden 斷言段落字面與位置；指令檔過期探測測試仍綠。 <!-- speclink-task:tsk_01KZ6GTRZ6Z0VRC8K8EJNFCQ9T -->
- [x] 3.3 golden 兩維度再生：預設 fixture（政策關）對照不含兩顆 worktree 技能；政策開 fixture 對照含之，且與變更前全量集合僅差 P0／P3.5 新增文本（diff 人工過目確認無意外變動）。檔案：`crates/speclink-core/tests/it/render_golden.rs`、`crates/speclink-core/tests/golden`。驗證：cargo test render_golden 全綠。 <!-- speclink-task:tsk_01KZ6GTRZ6PSFMNW3ANC0G3WZA -->

## 4. desktop 後端：設定寫入、看板資料流與動詞防護

- [x] 4.1 settings 寫入含 worktree 實值且 carry_over 退役：存檔以 UI 實值寫入 worktree 鍵（yaml 純量走既有跳脫 seam）、寫入成功後觸發與 CLI 同一技能同步、由開改關遇活躍 worktree 回拒絕錯誤（含擋下清單）；carry_over_worktree 函式與其「UI 無此欄位」前提的測試汰換為實值寫入測試。檔案：`apps/desktop/core/src/settings.rs`。驗證：cargo test -p speclink-desktop-core 通過。 <!-- speclink-task:tsk_01KZ6GTRZ6TXXNAW4B267NFMS7 -->
- [x] 4.2 看板查詢接 worktree facts：卡片 JSON 對有映射的 change 帶 worktree 物件（camelCase 欄位 path 與 branch，型別均為字串），無映射時欄位缺席；組裝走與 CLI list 同一 listing 落點，facts 由 host discover 取得（與 4.3、5.2 合力涵蓋 spec 需求「desktop 看板的 worktree 呈現」）。檔案：`apps/desktop/core/src/query.rs`。驗證：desktop-core 測試斷言 payload 欄位存在、命名與型別。 <!-- speclink-task:tsk_01KZ6GTRZ6Y5VPE8PFS9BF0B75 -->
- [x] 4.3 監看路徑推導與 watcher 擴充：自 worktree facts 導出監看清單（各 worktree 的 openspec/changes/<change名>/ 與主 repo 的 .git/worktrees/），推導函式歸 desktop-core 並含跨平台路徑單元測試；`apps/desktop/src-tauri/src/watch.rs` 只接上清單與事件轉發（單行委派）。檔案：`apps/desktop/core/src/query.rs`、`apps/desktop/src-tauri/src/watch.rs`。驗證：desktop-core 單元測試通過；手動確認 worktree 增減觸發看板更新。 <!-- speclink-task:tsk_01KZ6GTRZ6QSS47J521Q205Z0X -->
- [x] 4.4 desktop 動詞防護：對有 worktree 映射的 change，封存與退回提案中動詞於 desktop-core 動詞層拒絕並回「先執行 worktree-merge 收尾」訊息；唯讀動詞不受影響（涵蓋 spec 需求「worktree 掛著時的 desktop 動詞防護」）。先寫紅測（有映射→拒絕；移除映射→放行），再實作。檔案：`apps/desktop/core/src/verbs.rs`。驗證：cargo test -p speclink-desktop-core 通過。 <!-- speclink-task:tsk_01KZ6GTRZ627VZDYN81HMAW7QB -->

## 5. desktop 前端：開關與 worktree 呈現

- [x] 5.1 設定頁產出政策區新增 worktree 開關：僅 local workspace 顯示（remote 隱藏）、載入反映 config 現值、存檔送畫面實值、關閉遇活躍 worktree 時浮出擋下訊息（列 change 名、分支、路徑與收尾指引）；zh-TW 與 en 文案均直出 worktree（涵蓋 spec 需求「產出政策的 worktree 開關」）。檔案：`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`。驗證：前端測試覆蓋顯示、存檔與 remote 隱藏三例。 <!-- speclink-task:tsk_01KZ6GTRZ692SP5QMTSBWYWGV2 -->
- [x] 5.2 卡片 worktree 標示與抽屜分支路徑：有映射的 change 卡片帶 worktree 標示，變更抽屜顯示分支名與 worktree 路徑（OS 原生形式），worktree 移除後標示與資訊退場；zh-TW 與 en 文案直出 worktree。檔案：`apps/desktop/src/App.tsx`、`apps/desktop/src/i18n/messages.ts`。驗證：前端測試斷言標示與抽屜欄位隨 worktree 物件出現與消失。 <!-- speclink-task:tsk_01KZ6GTRZ628802TF6RXPPZNQV -->

## 6. 收尾驗證

- [x] 6.1 全量回歸：cargo test 跑過 speclink-core、speclink-cli、speclink-host、speclink-desktop-core 與前端測試，golden 與 CLI 回歸對照全綠（含本批刻意更新者）。驗證：測試指令輸出零失敗。 <!-- speclink-task:tsk_01KZ6GTRZ68XQX41NWY6TS7XPR -->
- [x] 6.2 手動驗收：desktop 切開關（開→技能檔出現；關→移除；掛 worktree 時關→擋下訊息）；開兩個 worktree 於看板同時觀察標示與任務計數即時更新，merge 收尾後標示退場；生成的 SKILL.md 之 P0／P3.5 文本人工過目。驗證：上述六個觀察點逐項確認。 <!-- speclink-task:tsk_01KZ6GTRZ636QEJAA34B0C63M8 -->
