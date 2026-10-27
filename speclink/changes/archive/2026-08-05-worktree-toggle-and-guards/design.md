## Context

worktree 第一刀已落地：openspec/config.yaml 的 worktree 政策欄位（bool，預設 false，SPECLINK_WORKTREE 可於執行期覆蓋）、host 層 worktree discovery（分支慣例 speclink/<change> 對映 active change）、CLI list 的 worktree 觀察面輸出、兩顆技能（apply-with-worktree／worktree-merge）。現況斷點：

- 技能生成不看政策——`crates/speclink-core/src/init.rs` 對 registry 全量輸出，政策關閉時技能仍在，只靠技能內 P1 執行期檢查拒跑。
- desktop 端 worktree 只存在於 `apps/desktop/core/src/settings.rs` 的 carry_over_worktree 保值邏輯（設定頁沒有欄位，存檔時從原檔回填以免吃鍵）。
- desktop 看板資料流（`apps/desktop/core/src/query.rs`）用的是不帶 worktree facts 的 listing 組裝；watcher（`apps/desktop/src-tauri/src/watch.rs`）不監看 worktree。
- 技能對「一次多個 change」與「先 apply 再轉 worktree」無防護。

本 change 源自討論 worktree-toggle-and-guards（openspec/discussions/worktree-toggle-and-guards.md），五項假設與範疇均經使用者裁定。

## Goals / Non-Goals

**Goals:**

- 技能足跡與 worktree 政策綁定：開＝注入、關＝清理；desktop 產出政策區可直接切換。
- 政策由開改關時，活躍 worktree 保護收尾工具不被抽走。
- apply-with-worktree 技能補上兩道前置防護（多 change 拒收、進度與程式碼分家偵測）。
- desktop 看板成為 worktree 觀察面的第二個消費端（卡片標示、抽屜分支與路徑、watcher 即時更新）。

**Non-Goals:**

- desktop 卡片 merge 按鈕；remote 工作區的 worktree 呈現、注入同步與政策擋下（remote 設定頁不顯示 worktree 開關，維持現狀）；SPECLINK_WORKTREE 影響注入；單 session 多 change 批次；兩技能 apply 本體與 merge 流程內容的改動。

## Decisions

**D1 — 注入判定：生成期過濾，registry 保持全量。** `speclink_core::skills::registry()` 維持完整清單（skill_body、instructions、golden 皆需完整正典）；Skill 結構新增政策閘欄位（如 worktree_gated: bool，僅兩顆 worktree 技能為 true），`crates/speclink-core/src/init.rs` 的三個生成迴圈（內建工具、自訂描述子、指令檔過期探測）依政策過濾。政策輸入只讀 openspec/config.yaml 的 worktree 檔值（serde 既有 WorkflowConfig，向後相容：鍵不存在＝false），**不走含 env 的四層解析**——注入是專案持久狀態，env 是執行期逃生口。落點歸屬：過濾邏輯是領域規則，歸 speclink-core；不含 ANSI、不碰儲存媒介。
（替代案：registry() 收政策參數——牽動所有呼叫端且 skill_body 等唯讀面不該受政策影響，否決。）

**D1a — marker 區塊的技能清單同步跟隨政策（實作期補裁定）。** 內建工具 marker 內文（instructions_body）硬寫兩行 worktree 技能指引；技能檔被政策清掉而 marker 仍指路，等於叫代理呼叫不存在的技能，與本 change 的目的直接抵觸。故 marker 的這兩行同受政策閘控：政策關即不輸出。既有設計已有同款先例——Codex 無 verify 技能（for_codex=false），其 marker 的 done_line 就不提 verify，marker 的技能清單本就跟隨實際生成集合。
連帶介面：`instructions_body` 新增 worktree 布林參數（自訂描述子的 custom_instructions_body 本就不列 worktree 技能，不受影響）；Node SDK 的 instructions.render 選項同步新增 worktree 軸（預設 false），維持 node-sdk 規格「回傳內容與 CLI 以對等參數生成者一致」；assets.lock 的指紋輸入涵蓋政策開／關兩種 marker 變體。
（替代案：marker 維持無條件列出——政策關閉時代理仍被指引呼叫不存在的技能，否決。）

**D2 — 同步觸發：重用既有 update 入口，單一實作落點。** 政策寫入成功後的技能同步＝呼叫既有的 init/update 再生流程（冪等：依 .speclink.yaml 記錄的 tools 重生成技能與 marker，生成集合經 D1 過濾，prune 以生成集合為準清掉除名者）。CLI 的 workflow-config set（`crates/speclink-cli/src/commands.rs`）與 desktop 存檔（`apps/desktop/core/src/settings.rs`）呼叫同一個 core 入口，不平行實作。
寫入順序與失敗狀態（多筆寫入紀律）：①開→關方向先跑活躍 worktree 檢查（拒絕即整體不動）；②寫 config（yaml 純量走 `crates/speclink-core/src/util.rs` 的 yaml_scalar 跳脫，沿用既有政策寫入 seam）；③技能同步。③失敗時接受半套：config 已寫為正典、技能足跡過期，回報錯誤並提示重跑 speclink update 重建；不一致期間由技能內 P1 執行期檢查兜底，無安全風險。
（替代案：只做「技能生成＋prune」子集入口——多切一個入口、測試面加倍，冪等全量 update 已涵蓋，否決。）

**D3 — 關閉擋下：檢查落 host，CLI 與 desktop 共用。** 活躍 worktree 事實＝`crates/speclink-host/src/worktree.rs` 的 discover（git worktree list --porcelain 對映 active change；git 失敗＝空集合，fail-open 沿用第一刀）。speclink-host 提供「worktree 政策可否關閉」判定（回傳擋下清單：change 名、分支、路徑），CLI set 與 desktop-core 存檔各自呼叫。擋下時：CLI 以非零 exit code、stderr 列清單；desktop 走既有「設定寫入失敗浮出」機制顯示清單與「先跑 /speclink-worktree-merge 收完」提示。僅 local workspace 檢查；remote 不適用（Non-Goal）。

**D4 — desktop 設定頁：worktree 開關進產出政策區，carry_over 退役。** `apps/desktop/src/views/ProjectSettingsView.tsx` 產出政策區新增 worktree toggle（僅 local workspace 顯示），文案遵循 LANGUAGE.md 的「worktree 直出」明文例外。WorkflowPolicyFields 已含 worktree 欄位，desktop 端補上 UI→fields 的實值傳遞後，`apps/desktop/core/src/settings.rs` 的 carry_over_worktree 及其「設定頁沒有 worktree 開關」前提的測試一併退役，改為「UI 送什麼寫什麼」並經既有寫後驗證。

**D5 — 技能防護：assets 文本是唯一正典。** `crates/speclink-core/assets/skills/apply-worktree-pre.md` 新增兩段：
- P0（置於 P1 政策檢查之前）：輸入解析出多個 change 名→以 AskUserQuestion 請使用者挑一個，並印多 session 配方（每個 change 開一個新 session 各跑一次本技能）；明文禁止靜默依序批次。
- P3.5（置於 P3 之後、P4 建 worktree 之前）：讀 openspec/changes/<change-name>/.evidence.json 的 touched 檔清單，對主樹查 git status --porcelain；evidence 檔不存在或清單為空＝乾淨，靜默續行；有髒檔→停下列出檔案，AskUserQuestion 三選項依推薦序：「先走 /speclink-commit 把本 change 的程式碼收進 HEAD 再回來」「照樣繼續（明知 worktree 缺這些實作）」「停止」。
生成物（.claude/skills 等三處工具足跡）隨資產再生更新；內嵌資產版本戳遞增（workspace-tools 的「產物層版本戳同源」與「內嵌資產版本鎖定紀律」）。

**D6 — desktop 呈現：查詢層接 worktree facts，watcher 擴充。** `apps/desktop/core/src/query.rs` 的看板組裝改用 listing 的帶 worktree facts 組裝點（`crates/speclink-core/src/listing.rs` 既有 assembly，CLI list 同款——同一契約唯一實作），facts 由 desktop-core 呼叫 host discover 取得。監看路徑的推導（從 facts 導出各 worktree 的 openspec/changes/<change>/ 與主 repo 的 .git/worktrees/）歸 speclink-desktop-core 並可獨立測試；`apps/desktop/src-tauri/src/watch.rs` 只接上路徑清單與事件轉發（Tauri 殼單行委派紀律）。前端：`apps/desktop/src/App.tsx` 卡片加 worktree 標示、變更抽屜顯示分支（speclink/<change>）與 worktree 路徑；文案進 `apps/desktop/src/i18n/messages.ts`（zh-TW 與 en，worktree 直出）。

**D7 — worktree 掛著時的 desktop 動詞防護（承前討論 Deferred，本 design 裁定）：最小紅線。** 對有 worktree 映射的 change，desktop 的「封存」與「退回提案中」動詞在 desktop-core 動詞層擋下，提示先跑 /speclink-worktree-merge；引擎與 CLI 動詞不動（CLI 使用者屬進階情境，保留手動彈性）。其餘動詞（開抽屜、看 diff 等唯讀面）不擋。

## Implementation Contract

**行為（使用者可觀察）：**

1. worktree 政策關閉（預設）的專案跑 speclink update：.claude/skills（與其他工具足跡）不含 speclink-apply-with-worktree 與 speclink-worktree-merge，且 CLAUDE.md／AGENTS.md 的 marker 區塊不含那兩行 worktree 技能指引；開啟政策後 update（或直接 set worktree true）技能與指引同時出現；關回去（無活躍 worktree）同時消失。其餘技能集合與 marker 其他內容不受政策影響。
2. speclink workflow-config set worktree false 在存在活躍 linked worktree 時：exit code 非 0，stderr 列出每個 worktree 的 change 名、分支、路徑，config 檔位元組不變。
3. desktop（local workspace）設定頁產出政策區出現 worktree 開關；存檔＝config 寫入＋技能同步；關閉遇活躍 worktree 時存檔失敗浮出，訊息列 worktree 清單與收尾指引，config 不變。remote 工作區設定頁不顯示此開關。
4. 生成後的 apply-with-worktree SKILL.md 含 P0 段與 P3.5 段（文案如 D5）；worktree-merge SKILL.md 內容不變（僅注入條件變化）。
5. desktop 看板（local）：有 worktree 映射的 change 卡片帶 worktree 標示；抽屜顯示分支與路徑；worktree 增減或其內 change 目錄變動後看板自動更新，無需手動重整。
6. 對有 worktree 映射的 change，desktop 的封存與退回動詞被擋下並提示先收尾。

**介面／資料形狀：** Skill 結構新增政策閘欄位；host 新增「可否關閉 worktree 政策」判定（輸入 workspace，輸出擋下清單——change 名、branch、path）；list 的 worktree JSON 形狀沿用第一刀（camelCase，欄位不變）；workflow-config show／set 的 --json 形狀不變。desktop 前端經既有查詢命令取得卡片資料，新增欄位沿用 listing 的 worktree 物件。

**失敗模式：** git 不可用→discover 空集合→關閉不擋、看板無標示（fail-open，沿用第一刀）；技能同步失敗→config 為正典、足跡過期，錯誤浮出並提示重跑 update，P1 執行期檢查兜底；.evidence.json 不存在或無 touched→P3.5 靜默通過。

**驗收：**

- render_golden 兩維度：政策關（預設 fixture）生成集合不含兩顆 worktree 技能；政策開含之且內容含 P0／P3.5 段。
- CLI 整合測試（crates/speclink-cli/tests/it/workflow_config.rs 擴充）：set worktree true→技能檔出現；set false 無 worktree→技能檔消失；set false 有活躍 worktree→非零 exit、stderr 清單、config 不變。
- desktop-core 測試（不依賴 Tauri）：設定頁欄位寫入含 worktree 實值、carry_over 測試汰換為實值寫入測試；查詢層 worktree facts 進卡片 JSON；監看路徑推導；動詞擋下（D7）。
- 手動驗收：desktop 開兩個 worktree，看板標示與抽屜資訊即時出現；merge 收掉後標示退場。

**範圍邊界：** in scope＝上述六項行為與其測試；out of scope＝Non-Goals 全部、引擎/CLI 動詞的 worktree 防護、worktree-merge 技能文本、server 端政策 API。

## Risks / Trade-offs

- **golden 大面積再生**：預設 fixture 的技能集合改變（少兩顆），政策開維度新增——一次性再生並人工過目 diff；政策開集合與現行全量集合逐位元組一致（僅 P0／P3.5 文本新增），以此驗證無意外變動。CLI 回歸對照（workflow_config.rs 等）同步更新。
- **跨平台**：git worktree list --porcelain 與分支慣例沿用第一刀（已跨平台驗證）；watcher 監看 .git/worktrees/ 在 Windows 的路徑分隔與大小寫由監看路徑推導層統一處理；抽屜顯示路徑用 OS 原生形式。
- **切開關弄髒 git 樹**：寫／刪技能檔屬預期行為（生成物本受版控），經使用者點頭接受；文件與擋下訊息不再額外警示。
- **半套風險**：config 寫入成功、技能同步失敗的視窗期——P1 執行期檢查兜底，update 可重建，接受並記載（D2）。
- **儲存解耦方向**：注入過濾與同步全走 core 的生成 seam、facts 走 host 觀察面，不新增任何持久儲存；規格驅動引擎與儲存層的邊界不受本 change 影響。
