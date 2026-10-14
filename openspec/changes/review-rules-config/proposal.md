## Why

審查關卡（`/speclink-review`）的 Standards 軸目前只會檢查一份寫死在技能範本裡的 12 條 code smell，再加上子代理自己去翻 CLAUDE.md、CONTRIBUTING、lint 設定時剛好讀到的 repo 慣例。透過 AI 代理跑 SDD 的開發者（以及替團隊訂標準的 PO／PM）沒有辦法明確寫下「這個專案的審查一定要檢查哪些事、違反時多嚴重」：repo 文件不保證每次都被讀到，也無法決定擋不擋蓋章，12 條清單更是只能整份接受。這個變更讓專案在 `openspec/config.yaml` 寫下自己的審查規則，作用在 workflow 的 `review?` 階段（`/speclink-review`，以及經 `/speclink-quality` 的兩站時序），並由 `/speclink-config` 技能與 desktop 設定頁協助維護。

本變更是討論 custom-review-rules 規劃的第 1 刀；第 2 刀（工單記錄首輪套用的規則、desktop 審查分頁顯示）在本刀封存後另行轉出。

## What Changes

- **新增設定區塊 `review`**（`openspec/config.yaml` 頂層鍵，隨 spec store 走，local 與 remote 皆適用）：
  - `baseline`：`extend`（預設，省略即此值）＝預設 12 條 code smell 照常檢查、再加上專案規則；`replace`＝不套用 12 條，只用專案規則。
  - `critical`、`warning`、`suggestion`：三個規則字串清單。規則放在哪個清單，違反時就以該嚴重度記入審查工單。三個清單預設皆為空。
  - 例：`review: { critical: ["所有 SQL 一律用參數化查詢，不得用字串拼接"], warning: ["新增的公開 API 要有對應的測試"], suggestion: ["函式超過 50 行時宜拆分"] }`
  - 嚴格解析：`review` 底下出現未知子鍵（例如打錯成 `critcal`）、`baseline` 不是 `extend`／`replace`、或清單項目不是字串時，讀取設定即失敗並點名該鍵（沿用 config 壞檔 fail-closed 行為，不退回預設值）。
- **新增 CLI 子指令 `speclink workflow-config review`**：
  - `speclink workflow-config review <critical|warning|suggestion> --stdin [--dry-run] [--no-color]`：以 stdin 整段取代該清單，一行一條，頭尾空白去除、空行略過；stdin 為空時移除該清單。未帶 `--stdin` 時非零 exit code 並印出用法。
  - `speclink workflow-config review baseline <extend|replace> [--dry-run] [--no-color]`：設定預設清單模式；寫 `extend` 即移除 `baseline` 鍵（省略＝預設）。值不在兩者之內時非零 exit code。
  - `baseline` 鍵不存在且三個清單皆空時，整個 `review` 鍵自檔案移除。
  - 成功 exit code 0、stdout 單行成功訊息；`--dry-run` 印 unified diff 且檔案逐位元不變；config 無法解析時非零 exit code、拒寫；remote 模式走既有讀-改-寫與版本衝突檢查（衝突時非零 exit code 並提示重跑）。
- **`speclink workflow-config show` 帶出 `review`**：人眼輸出多一行 `review`（模式與三個清單的條數）；`--json` 多一個 `review` 物件：`{ baseline, critical, warning, suggestion }`，未設定時為 `extend` 與三個空陣列。
- **審查技能（`/speclink-review`）套用專案規則**：
  - 以 `speclink workflow-config show --json` 讀取 `review`（remote 模式同語意，不直接讀檔）。
  - Standards 軸的子代理簡報帶入三個清單；`replace` 時不帶 12 條 smell。
  - 違反專案規則的發現依規則所在清單定級，描述原文引用被違反的規則；`critical`／`warning` 清單的違反列為必修（擋蓋章），`suggestion` 清單的違反列為可裁。
  - 使用者要接受的必修項含 CRITICAL 時，技能逐條列出這些 CRITICAL 並要求明確確認一次；只接受 WARNING 時流程不變。
  - 審查結果的呈現開頭多一行，寫明這次的預設清單模式與三個清單的條數；`replace` 且三個清單皆空時寫明「Standards 軸只依 repo 文件」。
- **config 技能（`/speclink-config`）管理審查規則**：
  - 預設清單模式列為第六問，每次帶現值詢問，不從 repo 推論。
  - 不從 repo 推導審查規則；只接收使用者口述的規則，依語氣建議放進哪個清單（「不得」「一律」→critical、「要」「需」→warning、「考慮」「盡量」「宜」「偏好」→suggestion），經使用者確認後以 `--dry-run` 預覽、同意才寫入。
  - 既有審查規則照判準檢查：`extend` 時不得重述 12 條 smell，`replace` 時不檢查這一點；不寫會過時的內容；引用的路徑與指令必須存在。
- **desktop 設定頁新增「審查規則」**：位於專案設定的 `openspec/config.yaml` 分組、產出規則與產出流程之間；唯讀顯示預設清單模式（可展開 12 條名稱）與三個清單；編輯沿用產出規則頁同一個程式碼編輯器、同一組編輯／取消／儲存控制與同一條寫入路徑，預設清單為兩個單選；整份換掉且三個清單皆空時頁內提示。12 條名稱由引擎提供，desktop 不另存一份。
- **詞彙**：`openspec/LANGUAGE.md` 新增「審查規則」詞條。
- 畫面草稿：https://claude.ai/artifact/RNqumZRVKjC7AWox64Z2pu （設定頁唯讀與編輯中兩張屬本刀；審查分頁一張屬第 2 刀）。

### 相容性影響

- `workflow-config show` 人眼輸出多一行 `review`（刻意變更）；`--json` 新增 `review` 欄位，既有欄位不變。依賴人眼輸出逐行比對的腳本需容許新行。
- 審查技能與 config 技能的內文變更：引擎的技能版號（ASSET_VERSION）提升，golden 對照同批更新；既有專案在 desktop 或 `speclink update` 會看到技能檔過期提示，執行 `speclink update` 即取得新版。
- 舊版 CLI 讀到含 `review` 的 config 會略過這個區塊（設定檔不拒絕未知頂層鍵），因此舊版技能不會套用專案規則，但也不會失敗。
- 新版 CLI 遇到寫錯的 `review` 區塊會讓所有讀取 config 的指令拒絕執行，直到修正——與現行 config 壞檔行為一致；remote 寫入由 server 以同一套引擎解析擋下（`invalid_config`）。
- 沒有寫 `review` 的專案：審查行為與現在完全相同（12 條照舊、無專案規則）。

## Non-Goals

- 工單記錄首輪套用的規則、desktop 審查分頁顯示套用規則——屬第 2 刀。
- Correctness 軸（抓 bug）與審查流程規則（首輪／複驗、三級嚴重度、必修／可裁界線、工單格式）不開放自訂。
- 驗證站（`/speclink-verify`）不開放自訂規則：它的規則就是變更自己的規格，已可用 `rules.specs`／`rules.design` 要求規格多寫什麼。
- 不改擋章與接受機制：CRITICAL 與 WARNING 照舊都擋蓋章、都能經 `--accept` 放行。
- 不在章上標示「帶保留」。
- 改審查規則不讓已蓋的章失效。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 新增 `review` 區塊的語意、嚴格解析與 `workflow-config review` 寫入子指令；`workflow-config show` 帶出 `review`。
- `review-skill`: Standards 軸的 smell baseline 改為依 `baseline` 條件套用；專案審查規則的讀取、套用與定級；裁量分類把專案規則 critical／warning 清單的違反列為必修；接受 CRITICAL 時逐條確認；呈現開頭寫明套用的規則。
- `config-skill`: 審查規則的第六問、規則接收與分格建議、既有審查規則的判準檢查。
- `desktop-config`: 專案設定新增「審查規則」頁的唯讀與編輯行為；分類欄與遠端、解析失敗的分類清單加入審查規則。
- `desktop-app`: 側欄導覽結構中專案設定分類欄的「openspec/config.yaml」群組加入審查規則。這條需求同時被進行中的 desktop-swipe-back-finger-lift 修改，本刀的區塊以該變更的版本為底，封存須排在它之後。

步驟 3 的規格掃描命中上述五個既有 capability（另有 review-station、quality-skill 相關但本刀不改其需求），沒有需要新增的 capability。

## Impact

- Affected specs: workflow-config、review-skill、config-skill、desktop-config、desktop-app
- Affected crates／apps：speclink-core（設定解析與寫入接縫、兩份技能範本、技能版號）、speclink-cli（workflow-config 子指令與 show 輸出）、speclink-server（僅新增測試：寫錯的 review 區塊被 PUT /config 擋下）、apps/desktop（desktop-core 設定快照與寫入、Tauri 指令、前端設定頁）
- 生成的技能：speclink-review 與 speclink-config，影響所有已啟用工具（claude、codex、copilot）的技能檔；本 repo 自身的技能檔由 `speclink update` 再生
- Affected code:
  - New:
    - apps/desktop/src/views/project-settings/ReviewRulesPage.tsx
  - Modified:
    - crates/engine/speclink-core/src/workspace/config.rs
    - crates/engine/speclink-core/src/workspace/config/tests.rs
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/src/workspace/skills.rs
    - crates/engine/speclink-core/assets/skills/review.md
    - crates/engine/speclink-core/assets/skills/config.md
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/adapters/speclink-cli/src/verbs/config.rs
    - crates/adapters/speclink-cli/tests/it/workflow_config.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/host/speclink-server/tests/it/api/policy_write.rs
    - apps/desktop/core/src/settings.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src/adapter/workspace.ts
    - apps/desktop/src/settingsCatalog.ts
    - apps/desktop/src/views/ProjectSettingsView.tsx
    - apps/desktop/src/views/project-settings/useProjectSettings.ts
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/projectSettingsView.test.tsx
    - apps/desktop/src/__tests__/remoteDataSource.test.ts
    - apps/desktop/src/__tests__/workspace.test.ts
    - openspec/LANGUAGE.md
    - docs/configuration.md
    - docs/configuration.zh-TW.md
  - Removed: (none)
