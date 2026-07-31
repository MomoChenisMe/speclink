## Why

透過 AI 代理跑 SDD 的開發者(desktop 看板+CLI 雙介面)一旦誤開工——agent 對錯的變更執行 apply,或在 desktop 誤勾任務——「進行中」就沒有回頭路:started_at 戳記永久停留,卡片無法回到提案中,看板長期失真。本變更為 apply 階段的誤啟動補一條受守門保護的修正路徑。

## What Changes

- 引擎新增 in-progress 標記的反向動詞:自 change meta 移除 started_at/started_by/started_with 三欄位,僅在零工作痕跡(已勾任務數為 0 且 touched 記錄為空)時成功;有痕跡時拒絕並列出證據(N 個已勾任務、touched 記錄的檔案清單),不提供強制旗標或機械清理。守門在引擎 command 層裁決一次,CLI、desktop、remote 三個入口共用。
- CLI 新增子指令 speclink in-progress remove(接 change 名稱參數):無旗標、不讀 stdin;成功與冪等情形(未開工的 change)exit 0;守門拒絕與未知 change exit 非 0 並於 stderr 說明。相容性影響:無——既有 speclink in-progress add 的輸出、行為與 parity 凍結完全不動,新子指令不影響任何既有指令的人眼或 --json 輸出。
- desktop 看板:「進行中」欄的變更卡與其詳情抽屜新增「退回提案中」動作(樣式沿討論卡的封存按鈕);點擊直接呼叫引擎動詞,UI 不預判守門條件;被擋下時以對話框呈現引擎回傳的證據與出路(已勾任務可取消後重試;touched 需請 agent 判斷)。退回成功後卡片經派生自然回到「提案中」欄。
- remote:server 新增與 POST /changes/{name}/in-progress 成鏡像的移除端點,同一引擎守門裁決,本地與遠端行為一致;desktop remote 模式走同一按鈕與對話框。
- 生成技能:apply 技能(claude 與 codex 兩生成檔)補「開錯工怎麼退」指引,正典來源與生成物同步更新。
- 順帶修復既有不對稱:討論抽屜補「封存」動詞(concluded 討論、經確認後封存),與討論卡對稱。
- 詞彙:openspec/LANGUAGE.md 立「退回提案中」詞條。

## Non-Goals

- 不開放跨欄拖曳改變階段——spec pin「跨欄拖曳不改變變更階段」原樣保留,拖曳仍只管排序。
- 不提供 --force 或任何機械清理 touched/已勾任務的路徑:touched 只記檔案不記行,檔案可能混有其他變更的內容,機械清理有誤刪風險;需要判斷的清理留給 agent 或人。
- 不動生命週期 gate 的六站轉換表——退回定位為 gate 外的修正動詞(與 discard 同類)。
- 不處理 remote 模式 touched/evidence 被污染時的修復路徑(本期只保證守門行為一致)。
- 不改變任務勾選蓋戳與 touched 記錄的既有行為。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-lifecycle`: in-progress 標記新增反向動詞(移除 started_* 三欄位)與零工作痕跡守門
- `desktop-app`: 變更卡與詳情抽屜的「退回提案中」動作與守門對話框;討論抽屜補「封存」動詞
- `server-verb-api`: in-progress 移除端點(與既有加入端點成鏡像)

## Impact

- Affected specs: change-lifecycle、desktop-app、server-verb-api(皆為修改,無新增 capability)
- Affected code:
  - New:
    - packages/ui/src/components/RevertBlockedDialog.tsx(守門對話框;實作時若併入既有對話框元件則不另立新檔)
  - Modified:
    - crates/speclink-core/src/inprogress.rs(反向動詞與零痕跡守門)
    - crates/speclink-core/src/command/mod.rs(新 Command、Outcome 與 DomainEvent)
    - crates/speclink-cli/src/main.rs(in-progress 子指令群補 remove)
    - crates/speclink-cli/src/commands.rs(本地 dispatch)
    - crates/speclink-cli/src/remote_commands.rs(remote dispatch)
    - crates/speclink-server/src/routes.rs(鏡像端點)
    - crates/speclink-remote/src/client.rs(typed client 補對應方法)
    - crates/speclink-protocol/src/command.rs(守門拒絕的證據載荷 wire 形狀,serde camelCase rename)
    - apps/desktop/core/src/manage.rs(本地 command bridge)
    - apps/desktop/src-tauri/src/remote.rs(remote command bridge)
    - apps/desktop/src-tauri/src/lib.rs(command 註冊)
    - apps/desktop/src/adapter/tauriDataSource.ts(本地 adapter 補動詞)
    - apps/desktop/src/adapter/remoteDataSource.ts(remote adapter 補動詞)
    - apps/desktop/src/App.tsx(按鈕與對話框接線)
    - packages/ui/src/adapter.ts(SpeclinkDataSource 介面補動詞)
    - packages/ui/src/components/ChangeCard.tsx(進行中卡的退回動作)
    - packages/ui/src/components/RichDetailDrawer.tsx(詳情抽屜的退回動作)
    - packages/ui/src/components/DiscussionDrawer.tsx(補封存動詞)
    - packages/ui/src/i18n.tsx(按鈕與對話框文案)
    - crates/speclink-core/assets/skills/apply.md(apply 技能正典補退回指引)
    - .claude/skills/speclink-apply/SKILL.md(生成物同步)
    - openspec/LANGUAGE.md(「退回提案中」詞條)
  - Removed: 無
