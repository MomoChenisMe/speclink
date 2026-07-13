## Context

任務定址現況：crates/speclink-core/src/tasks.rs 的 parse 以「檔內第 N 個 checkbox」給 1-based 序數，mark_done／uncomplete 以序數翻轉；CLI 的 task done／undone 保留原始 argv 字串但只接受數字；領域事件 task-completed／task-uncompleted 載荷為序數（事件契約 experimental，允許不相容調整）；桌面 packages/ui/src/tasks.ts 以 ordinal 樂觀翻轉。task done 時 TouchedRecord 把 git 髒檔清單寫入 .speclink/touched/（change 為鍵，逐 task 序數存檔案清單），archive 以它建 trace、speclink-commit 技能以它做檔案歸屬。上一刀交付的 ExecutionContext 供 actor 與 repo binding，EffectiveWorkflowPolicy 攜政策 digest。正典 verb-contract「任務取消勾選動詞」明文「task id 非數字即錯」與「純狀態翻轉、不寫 touched 記錄」。

## Goals / Non-Goals

**Goals:**

- 任務獲得不可變 stable ID（markdown 內嵌註解），重排與編輯後身分不變；ordinal 降級為顯示與相容用途。
- task done 產生逐任務 evidence（ID、actor、repo、head commit、touched files、spec／tasks／policy basis digest），舊 touched 格式向下可讀。
- host 提供 VerifyBundle（固定 basis）與 stale 判定；archive trace 改由 evidence 建立且輸出格式凍結。
- 桌面與 UI 以 stable ID 呈現與操作，ID 註解對使用者不可見。
- 數字 ordinal 的一切現行輸出逐位元凍結。

**Non-Goals:**

- 不做 drift 拆分、Protocol、Server 端 evidence 儲存與上行傳輸、approval/review gate 強制、task reorder 功能、Node dispatch 動詞擴充。

## Decisions

### 決策一：ID 形制為 tsk_ 前綴 ULID，內嵌於任務行尾註解

ID 格式 tsk_ 加 26 字元 ULID（時間排序、無碰撞協調需求），以 HTML 註解內嵌於任務行尾（speclink-task 冒號標記）。解析時剝離註解得顯示文字；序列化保留註解原位。採用輕量 ulid 依賴（無傳遞重依賴）。替代方案：內容 hash 作 ID——內容編輯即身分斷裂，路線圖 §3.5 明文禁止；獨立 sidecar 檔存 ID 對映——任務與身分分離兩檔，手動編輯 tasks.md 後對映即刻失效，違反「本地可讀寫體驗」，被拒。

### 決策二：蓋章時機——Engine 產出全檔蓋章、task done 單行補章、不做全檔強制遷移

new artifact tasks（Engine 產出 tasks.md）時全檔任務指派 ID。task done 遇目標行無 ID 時，於同一次寫入對該行補 ID（其餘行不動——保住 verb-contract「其餘內容不變」精神與 twin 對照的可預期性）；task undone 維持純狀態翻轉、不補章不寫記錄。既有無 ID 的 tasks.md 不做任何背景遷移，ordinal 操作照常。替代方案：task 動詞執行前全檔 normalize——一次 task done 造成全檔 diff，破壞「僅該任務行變更」的既有契約、污染 speclink-commit 的檔案歸屬與使用者 diff 審視，被拒；新增顯式 normalize 動詞——本刀無消費者需要它，屬臆測性介面，被拒。

### 決策三：定址雙值域，stable ID 為第一級身分

task done／undone 的 task-id 參數：純數字走現行 ordinal 路徑（輸出逐位元不變）；tsk_ 前綴走 ID 查找（找不到回「task 不存在」對稱錯誤）；其餘值沿現行「非法 task id」錯誤形狀。parse 偵測重複 ID 時 task 動詞拒絕並點名重複值（tasks.md 檔面損壞類，不靜默取第一個）。領域事件 task-completed／task-uncompleted 載荷改攜 stable ID（無 ID 的 ordinal 操作以該次補章後的 ID 入載荷；undone 對無 ID 任務以序數字串入載荷——undone 不補章）。替代方案：事件同時攜 ordinal 與 ID 雙欄位——事件是 experimental 契約，保留雙欄位徒增下游相容負擔，被拒。

### 決策四：evidence 演進 TouchedRecord 為 v2 schema，同檔向下相容

.speclink/touched/ 下每 change 一檔的記錄演進：頂層加 version 與逐任務 entries——taskId、actor（顯示身分字串）、repo（binding key）、headCommit、touchedFiles、basisDigests（spec／tasks／policy 三 digest）、recordedAt（UTC）。讀取端接受舊格式（無 version 視為 v1，檔案清單語意不變），寫入端一律 v2；speclink-commit 技能消費的檔案清單欄位語意保留。actor 與 repo 取自 ExecutionContext（上一刀），basis digest 沿 EffectiveWorkflowPolicy digest 機制擴充 spec 與 tasks 內容 digest。替代方案：另立 .speclink/evidence/ 目錄——同一 task done 寫兩處、discard 與 archive 的清理路徑加倍，且 touched 舊檔仍須相容，徒增檔面，被拒。

### 決策五：VerifyBundle 與 stale 判定落在 speclink-host

host 的 evidence 模組提供：produce_verify_bundle(change)——回 change 名、任務 stable ID 清單、spec／tasks／policy basis digest 與產生時間；judge_staleness(evidence, bundle)——任一 basis digest 不符即回帶不符項的 stale 拒絕。錯誤分類屬 host 層型別（沿 binding 錯誤先例），不動命令層封閉五碼；本刀不接線任何 CLI 動詞（verify 技能與 Phase 2 Server 是消費者）。替代方案：stale 判定放 core——staleness 是應用層裁決（比對「誰的證據對誰的基準」），放 core 會迫使 core 認識 evidence 儲存，違反分層，被拒。

### 決策六：archive trace 由 evidence 建立、輸出格式凍結，gate 檢查不強制

archive 注入 trace 的檔案清單改自 v2 evidence entries 聚合（v1 舊檔沿現行路徑），注入格式與現行逐位元一致。host 提供 archive gate 的 evidence 檢查函式（任務全勾、evidence 存在且未 stale），本地 archive 不強制呼叫（順位 4 gate 不強制的延續）；強制點屬 Phase 2 Server。替代方案：本地 archive 立即強制 evidence 檢查——舊 workspace 與手動任務流程立刻被擋，破壞現行可用性，被拒。

### 決策七：桌面與 UI 以 stable ID 呈現與操作

packages/ui/src/tasks.ts 解析剝離 ID 註解入獨立欄位（顯示文字不含標記）、清單項以 stable ID 作 React key（無 ID 舊檔退回 ordinal key）、勾選請求以 stable ID 定址（無 ID 任務以 ordinal 相容路徑）；樂觀翻轉的就地改寫保留註解原文。desktop core 的任務動詞路徑跟進雙值域。替代方案：UI 顯示原始行（含註解）——把實作細節暴露給使用者且截斷排版，被拒。

### 決策八：輸出凍結與刻意變更清單

凍結面：數字 ordinal 的 task done／undone 人眼與 --json 輸出、exit code、錯誤訊息逐位元不變；parity 31／color 16／twin 8 全綠。刻意變更面（新測試固定）：tasks.md 檔面的 ID 註解（產出全檔、task done 單行補章）、tsk_ 值域的新輸出、事件載荷、touched v2 schema。桌面顯示文字因剝離而與現行一致（原本就無註解），vitest 斷言剝離行為。

## Implementation Contract

- **行為**：Engine 產出的 tasks.md 每個任務行尾帶 speclink-task ID 註解；speclink task done tsk_01J… 與 speclink task done 3 皆可勾選同一任務，重排 tasks.md 後 tsk_ 定址仍命中原任務而 ordinal 可能位移；task done 後 .speclink/touched/ 記錄含該任務的 taskId、actor、repo、headCommit、touchedFiles、basisDigests、recordedAt；task undone 不寫任何記錄；archive 注入的 trace 輸出格式與現行一致；桌面任務清單看不到 ID 註解且勾選正常。
- **介面／資料形狀**：任務行格式「- [ ] 1.1 描述 <!-- speclink-task:tsk_ULID -->」；TouchedRecord v2 JSON（version、entries 陣列、camelCase 欄位）；VerifyBundle { change, taskIds, specDigest, tasksDigest, policyDigest, producedAt }；stale 判定回帶不符 basis 項的拒絕；事件載荷 task_id 為 stable ID 字串。
- **失敗模式**：重複 ID 使 task 動詞拒絕並點名重複值；tsk_ 查無此 ID 回與 ordinal 超界對稱的「task 不存在」錯誤；stale 判定列出全部不符的 basis；舊 v1 touched 檔正常讀取不報錯。
- **驗收**：cargo test -p speclink-core、-p speclink-host、-p speclink-cli 全綠含上述新測試；npm test -w packages/ui 與 -w apps/desktop 全綠；parity／color／twin 全綠；baseline exe 對照數字 ordinal 情境逐位元一致；npm run test:all 全綠。
- **範圍邊界**：in scope——tasks.rs 解析與蓋章、雙值域定址、事件載荷、TouchedRecord v2、host evidence 模組、archive trace 來源、CLI 值域、UI/desktop 剝離與 stable key；out of scope——drift 拆分、Protocol、Server 儲存、approval gate 強制、reorder 功能、dispatch 動詞擴充。

## Risks / Trade-offs

- [ID 註解破壞第三方或使用者的 tasks.md 工具鏈] → 採 HTML 註解（markdown 渲染不可見）；剝離規則單一實作於 parse；刻意變更記錄於 spec 場景。
- [單行補章與「其餘內容不變」契約的張力] → verb-contract 的既有場景只約束 undone（不補章路徑，完全不變）；done 的單行補章以新場景明定「僅目標行變更」。
- [touched v2 與 speclink-commit 技能的相容] → 檔案清單欄位語意保留、技能讀取路徑不變；以 commit-skill 既有測試與手動場景驗證。
- [ULID 依賴引入] → 選無傳遞重依賴的實作並鎖版本；只在 core 產 ID 一處使用。
- [事件載荷改 ID 影響未知消費者] → 事件契約明文 experimental；工作區內消費者（無持久化）全在本刀同步更新。
- [雙值域解析誤把未來格式當數字] → 解析規則封閉：純數字、tsk_ 前綴、其餘拒絕；以錯誤形狀測試鎖住。

## Migration Plan

漸進遷移、無強制轉換：舊 tasks.md 無 ID 照常以 ordinal 操作；Engine 新產出的 tasks.md 帶 ID；task done 逐行補章使活躍 change 自然收斂到有 ID 狀態。touched 記錄寫入即 v2、讀取相容 v1。回滾還原 commit 即可；已寫入的 ID 註解與 v2 記錄對舊版程式是可忽略的行尾註解與多餘欄位，不阻斷。

## Open Questions

（無）
