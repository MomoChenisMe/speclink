## Why

worktree 第一刀（引擎聚合讀＋config 欄位＋兩技能）落地後，流程留有四個斷點：政策只能靠 CLI 開關（desktop 產出政策區沒有 worktree 開關，settings 寫入還得靠保值邏輯防止存檔吃鍵）；兩顆 worktree 技能無論政策開關永遠注入，關閉時技能清單仍掛著按了必拒的動詞；技能對「一次給多個 change」與「先 apply 過再轉 worktree」兩個實務情境沒有防護，後者會做出「任務已勾、程式碼卻留在主樹」的分家 worktree；desktop 看板完全看不到 worktree 事實（資料流未接、卡片無標示、抽屜無分支資訊）。

目標使用者是透過 AI 代理跑 SDD 的開發者。使用情境對應 apply 階段的並行實作流程（speclink-apply-with-worktree／speclink-worktree-merge 兩技能）與 desktop 的專案政策管理（產出政策設定頁、看板觀察）。

## What Changes

- **條件式技能注入**（speclink-core）：技能生成期讀 openspec/config.yaml 的 worktree 政策——true 才注入 apply-with-worktree 與 worktree-merge 兩顆技能，false 時經既有 prune 機制移除。SPECLINK_WORKTREE 環境變數不影響注入（僅為執行期逃生口）；技能內 P1 執行期政策檢查保留為第二道防線。影響 claude／codex 與自訂描述子工具的技能足跡；golden 對照新增政策開／關維度。
- **workflow-config set 的 worktree 寫入語意**（speclink-cli）：寫入成功後同步技能足跡；由 true 改 false 時，若 local workspace 存在活躍 linked worktree，以非零 exit code 拒絕並於 stderr 列出 worktree 清單（先跑 merge 收完才能關）。worktree 設定欄位本身第一刀已存在（openspec/config.yaml，bool，預設 false）——本次不新增設定欄位，僅變更寫入語意。
- **desktop 產出政策區新增 worktree 開關**（apps/desktop）：比照 tdd／audit 的 toggle，存檔走既有 settings seam 並觸發同一技能同步；關閉遇活躍 worktree 同樣擋下並浮出清單；settings 內的 carry_over_worktree 保值邏輯退役。
- **技能防護兩則**（技能資產文本）：apply-with-worktree 的前置段新增 P0「多 change 輸入拒收」——偵測到多個 change 名時請使用者挑一個並印出多 session 配方，不做靜默依序批次；新增 P3.5「進度與程式碼分家偵測」——讀 change 目錄 .evidence.json 的 touched 清單對主樹查 git 狀態，發現髒檔即停下，選項依推薦序為「先走 speclink-commit 收程式碼」「照樣繼續」「停止」。
- **desktop worktree 呈現**（apps/desktop）：看板資料流改用帶 worktree facts 的 listing 組裝；卡片加 worktree 標示；變更抽屜顯示分支與 worktree 路徑；watcher 擴充監看各 worktree 的 change 目錄與 .git/worktrees 增減。僅 local workspace；文案遵循詞彙表「worktree 直出」明文例外。
- **worktree 掛著時 desktop 動詞的防護範圍**（如對 worktree 中的 change 執行封存／退回提案中）由 design 裁定——承前討論 worktree-parallel-apply 的 Deferred 項。

## Non-Goals

- 不做 desktop 卡片上的 merge 按鈕（承原討論，後續視需求）。
- 不讓 SPECLINK_WORKTREE 環境變數影響技能注入——注入只跟 config 檔的持久值走。
- 不支援單 session 多 change 依序批次 apply——「平行＝多 session」是既定模型，技能明確拒收。
- 不動 remote 模式的 worktree 呈現與政策擋下——worktree facts 是 host-local 觀察，remote 僅程式碼隔離（維持第一刀結論）。
- 不改兩顆 worktree 技能的 apply 本體與 merge 流程內容——本次只動前置防護段。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-tools`: 技能足跡改為政策條件式——worktree 政策關閉時兩顆 worktree 技能不生成並被清理，開啟時恢復注入；內建工具 marker 的兩行 worktree 技能指引同受此閘控
- `node-sdk`: instructions.render 的選項新增 worktree 軸（預設 false），以對應 marker 的政策維度
- `workflow-config`: set 的 worktree 欄位寫入後同步技能足跡；由開改關遇活躍 worktree 時拒絕寫入
- `desktop-config`: 產出政策區新增 worktree 開關，存檔觸發技能同步，關閉遇活躍 worktree 浮出擋下訊息；carry_over 保值邏輯退役
- `worktree-apply-skill`: 前置指示新增「多 change 輸入拒收」與「進度與程式碼分家偵測」兩道防護
- `worktree-overlay`: desktop 看板成為 worktree 觀察面的第二個消費端——卡片標示、抽屜分支與路徑、watcher 監看 worktree 增減

## Impact

- Affected specs: `workspace-tools`、`workflow-config`、`desktop-config`、`worktree-apply-skill`、`worktree-overlay`、`node-sdk`（均為修改，無新增）
- Affected code:
  - Modified（engine／CLI）: `crates/speclink-core/src/skills.rs`、`crates/speclink-core/src/init.rs`、`crates/speclink-core/assets/skills/apply-worktree-pre.md`、`crates/speclink-core/tests/it/render_golden.rs`、`crates/speclink-core/tests/golden`、`crates/speclink-cli/src/commands.rs`、`crates/speclink-host/src/worktree.rs`（可否關閉的判定）、`crates/speclink-node/src/render.rs`（worktree 渲染軸）
  - Modified（desktop 後端）: `apps/desktop/core/src/settings.rs`、`apps/desktop/core/src/query.rs`、`apps/desktop/core/src/verbs.rs`（動詞防護）、`apps/desktop/src-tauri/src/watch.rs`
  - Modified（desktop 前端）: `apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/i18n/messages.ts`
  - Modified（生成物，隨技能資產再生）: `.claude/skills/speclink-apply-with-worktree/SKILL.md`
  - New: (none)
  - Removed: (none)
- 相容性影響:
  - speclink update／init 的技能輸出改為政策條件式：worktree 關閉（預設）的專案跑 update 後，兩顆 worktree 技能檔會被移除——現況是無條件生成。既有使用者開啟政策即恢復；技能檔內容不因政策而異。
  - speclink workflow-config set worktree 由純檔案寫入變為「寫入＋技能同步」；由開改關且有活躍 worktree 時以非零 exit code 拒絕（stderr 列 worktree 清單）。show 與 set 的 --json 輸出形狀不變。
  - 人眼輸出：set 成功後多技能同步結果一行；golden 對照新增政策開／關維度，回歸對照需再生。
  - 已注入但政策關閉的舊足跡（尚未跑 update 的專案）行為不變：技能 P1 執行期檢查仍拒跑，不會因本次改動壞掉。
