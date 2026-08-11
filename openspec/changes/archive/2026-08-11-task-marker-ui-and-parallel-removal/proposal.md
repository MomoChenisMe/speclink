## Summary

移除無消費者的 `[P]` 平行標記(解析容忍舊檔、語意與 payload 欄位清除),並補上 `[M]` 手動測試標記的 desktop 呈現:任務列徽章與看板卡片的待手測標示。

## Motivation

`[P]` 在全正典的唯一出處是 tasks 起草指引的翻譯保留規則——沒有任何指引教何時加、沒有任何技能與 GUI 消費,是 OpenSpec 血統的休眠慣例;教了沒人用是對讀者的假承諾,使用者裁定清除。同時,manual-task-marker-gates 落地後 desktop 對 `[M]` 只有字面前綴照印:任務列直出工程 token(違反共用詞彙「工程詞不出現在使用者可見文案」原則),看板卡片無法區分「寫碼寫到一半的 9/10」與「寫碼收工、只等手測的 9/10」——後者正是新流程裡「輪到使用者了」的關鍵訊號。

本 change 源自已結論的討論 task-marker-ui-and-parallel-removal。

## Proposed Solution

- `[P]` 移除採「認得但不承載」:解析器保留 `[P]` 前綴剝離(封存區三個舊 change 與外部使用者 repo 的舊檔顯示容忍),Task 結構的 parallel 旗標、instructions/query 任務 payload 的 parallel 欄位、起草指引的 `[P]` 提及全數移除;起草指引的翻譯保留規則改點名 `[M]`(前一 change 的遺漏一併補上)。
- 任務列(desktop 抽屜任務分頁):UI 端剝離 `[M]`/`[P]` 行首前綴;`[M]` 任務於該列描述正上方獨立一行顯示「✋ 圖示+手動測試」小徽章、左緣與描述切齊——編號起始欄與現況同位、描述左緣不位移、長文字換行時徽章不動;配色取語意色票(2026-08-11 實測裁定:原行尾灰 chip 在灰階任務文字裡讀不出來,且視線起點在描述左緣不在行尾);勾完後文字劃線、徽章保留;`[P]` 舊標記只剝不顯。
- 看板變更卡:沿「看板卡片的審查標示」家族的行內小章樣式(不增加文字列、不觸犯「變更卡無狀態 chip」的解剖學正典);寫碼任務全完成且尚有未勾 `[M]` 時顯示「待手測」章,tooltip 載明剩餘項數;其他狀態的卡片逐位元不變。資料源:desktop 協定的變更清單項增 codeTotal/codeComplete/codeRemaining 三欄(加欄不改名,沿 apply payload 的欄位命名)。
- 共用詞彙:「手動測試」(任務徽章)與「待手測」(卡片章)入 openspec/LANGUAGE.md 新條目。

## Non-Goals

- 不啟用 `[P]`(啟用需先設計 apply 的平行執行機制,獨立大案——使用者裁定移除而非等待)
- remote 看板卡片的待手測章(沿審查/驗證標示的 local-only 先例;remote 變更摘要 payload 不動)
- CLI 的 list --json 不增列寫碼進度欄位(無消費者;desktop 協定與 CLI 輸出的相容釘住沿審查狀態欄位的既有句式)
- 勾選互動不加料:無高亮、無確認彈窗、無分組重排
- 徽章的 lucide 圖示確切選型留待實作(Hand 或同義)

## Alternatives Considered

- `[P]` 維持休眠:原建議,使用者否決——休眠慣例對讀者是假承諾
- 完全移除 `[P]` 解析:舊檔的字面前綴會滲進任務描述顯示,rejected
- 任務列保留字面 `[M]`:工程 token 直出;詞彙原則的例外線(slug/config.yaml/worktree)只給「可複製輸入的把手」,任務列是檢視面,套不上
- 純符號徽章:新使用者要 hover 才懂,rejected
- 徽章置於編號前或編號後內嵌:前者破壞編號縱向對齊(使用者明示約束),後者打斷「編號→描述」閱讀流
- 卡片獨立 chip 列:觸犯解剖學正典「變更卡 SHALL NOT 帶狀態 chip」且增加文字列,改走審查標示家族的行內小章

## Impact

- Affected specs: manual-task-marker、verb-contract、task-identity、desktop-app、client-protocol
- 封存順序約束:本 change 對 manual-task-marker 與 verb-contract 的 delta 以 manual-task-marker-gates 落地後的正典為基——manual-task-marker-gates SHALL 先封存,本 change 後封存(否則 archive merge gate 的 FROM 比對不成立)。
- Affected code:
  - Modified:
    - crates/speclink-core/src/tasks.rs(Task 移除 parallel 旗標;前綴剝除保留 `[P]` 容忍)
    - crates/speclink-core/src/instructions.rs(TaskJson 移除 parallel 欄位)
    - crates/speclink-protocol/src/query.rs(TaskEntry 移除 parallel;wire fixture 更新)
    - crates/speclink-server/src/routes.rs(task_entry 搬運移除 parallel)
    - crates/speclink-cli/src/verbs/instructions.rs(remote 回程轉接移除 parallel)
    - crates/speclink-core/assets/schema/spec-driven/tasks.instruction.md 與 crates/speclink-core/assets/schema/spec-driven/fork.schema.yaml(翻譯保留規則 `[P]` 改點名 `[M]`)
    - apps/desktop/core/src/query.rs(變更清單項增 codeTotal/codeComplete/codeRemaining 三欄)
    - packages/ui/src/tasks.ts(行首標記剝離與 manual 旗標)
    - packages/ui/src/components/TaskList.tsx(手動測試徽章)
    - packages/ui/src/components/ChangeCard.tsx(待手測行內小章)
    - packages/ui/src/stage.ts(awaitingManualCount:待手測判定單一入口)
    - packages/ui/src/adapter.ts(清單項新欄位型別;desktop 端經共用 ChangeItem 型別流通,tauriDataSource.ts 無需改動)
    - packages/ui/src/i18n.tsx(手動測試/待手測詞條,tw 與 en)
    - openspec/LANGUAGE.md(手動測試/待手測詞彙條目)
    - crates/speclink-core/src/init.rs(MARKER_VERSION 提升,若 asset 指紋涵蓋)
    - crates/speclink-core/tests/golden/assets.lock 與 golden snapshots(衍生物再生)
    - crates/speclink-cli/tests/it/manual_task_gates.rs、crates/speclink-cli/tests/it/remote_read_path.rs、crates/speclink-remote/tests/it/typed_client.rs(fixture 隨 parallel 欄位移除更新)
    - packages/ui/src/__tests__/taskList.test.tsx 與卡片相關 UI 測試(徽章與章的呈現斷言)
  - New: (none)
  - Removed: (none)
