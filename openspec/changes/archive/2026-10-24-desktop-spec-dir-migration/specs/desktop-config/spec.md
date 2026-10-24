## MODIFIED Requirements

### Requirement: 未初始化目錄經確認後自動初始化

<!-- BEFORE: 初始化建立 openspec/ 骨架 -->

所選目錄向上探索未命中任何 speclink 專案時，app SHALL NOT 逕行寫入，而 SHALL 顯示初始化確認對話框（含 AI 工具多選 claude／codex／copilot，預設只勾選 claude；說明文字寫明會建立 `speclink/`）。使用者確認後 app SHALL 執行與 speclink init 等效的初始化（speclink/ 骨架含 specs/、changes/archive/ 與 config.yaml、專案根的 .speclink.yaml 記錄所選 tools、為每個所選工具生成 skills 檔），隨即切換至該專案；使用者取消時 app SHALL 維持原專案，且目標目錄 SHALL NOT 產生任何寫入。初始化失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。

#### Scenario: 確認後初始化並切入新專案

- **WHEN** 使用者選定不含任何 speclink 標記的空目錄，於確認對話框保持預設（claude）並確認
- **THEN** 該目錄產生 speclink/（含 specs/、changes/archive/、config.yaml）、.speclink.yaml（tools 含 claude）與 .claude/skills/ 技能檔，不產生 openspec/ 與 CLAUDE.md，且 app 切換至該專案並於看板顯示空清單

#### Scenario: 勾選 codex 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 codex 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 .agents/skills/ 技能檔而無 AGENTS.md，.speclink.yaml 的 tools 同時記錄 claude 與 codex

#### Scenario: 取消初始化則零寫入

- **WHEN** 使用者於確認對話框取消
- **THEN** app 維持原專案，所選目錄內容與選擇前完全相同（無任何新檔案或目錄）

#### Scenario: 勾選 copilot 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 copilot 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 .github/skills/ 技能檔，不產生 .github/copilot-instructions.md、AGENTS.md，.speclink.yaml 的 tools 同時記錄 claude 與 copilot

### Requirement: 未啟用資料夾經確認後補齊啟用

<!-- BEFORE: 啟用時就地補齊，既有 openspec/ 內容零觸碰 -->
<!-- REMOVED-SCENARIO: 遷移資料夾確認啟用後補齊並切入 -->

所選目錄向上探索命中 workspace、store mode 為本地檔案、且該 workspace root 不存在 `.speclink.yaml` 時，app SHALL 判定為未啟用 speclink，SHALL NOT 逕行寫入亦 SHALL NOT 直接以既有專案開啟，而 SHALL 顯示啟用確認對話框（含 AI 工具多選 claude／codex／copilot，預設只勾選 claude；文案為啟用語意，遵循 LANGUAGE.md、不出現工程詞）。規格資料夾為舊佈局的 `openspec/` 時，對話框 SHALL 另寫明確認後資料夾會改名為 `speclink/`、之後 OpenSpec CLI 讀不到這個專案。判定與寫入 SHALL 錨定向上命中的 workspace root，而非使用者所選的子目錄。

使用者確認後 app SHALL 經引擎的工作區補齊入口執行啟用（必要時先把 `openspec/` 匯入為 `speclink/`、補骨架缺件、專案根 `.speclink.yaml` 記錄所選 tools、為每個所選工具生成 skills 檔，不生成指令檔），既有規格文件的內容 SHALL 不被改寫，隨即切換至該專案；使用者取消時 app SHALL 維持原專案，目標目錄 SHALL NOT 產生任何寫入。啟用失敗（包含因有其他 worktree 或未提交修改而拒絕匯入）時 app SHALL 顯示單行錯誤訊息說明原因且 SHALL NOT 切換 root。`.speclink.yaml` 存在的專案 SHALL 照舊直接開啟，SHALL NOT 出現啟用對話框；向上探索完全未命中的目錄 SHALL 照舊走初始化確認流程。

#### Scenario: OpenSpec 專案確認啟用後匯入並切入

- **WHEN** 使用者選定含 openspec/（內有既有規格文件）但無 .speclink.yaml、已 commit 的資料夾，啟用確認對話框寫明資料夾會改名為 speclink/，使用者保持預設（claude）並確認
- **THEN** openspec/ 改名為 speclink/、內容位元級不變，專案根產生 .speclink.yaml（tools 含 claude）與 .claude/skills/ 技能檔、不產生 CLAUDE.md，app 切換至該專案並於看板呈現既有內容

#### Scenario: 取消啟用則零寫入

- **WHEN** 使用者於啟用確認對話框取消
- **THEN** app 維持原專案，所選資料夾內容與選擇前完全相同

#### Scenario: 已啟用專案不出現啟用對話框

- **WHEN** 使用者選定專案根含 .speclink.yaml 的資料夾開啟
- **THEN** app 直接開啟該專案進看板，無啟用對話框

#### Scenario: 子目錄開啟錨定專案根

- **WHEN** 使用者選定未啟用專案的子目錄開啟並確認啟用
- **THEN** .speclink.yaml 與工具檔產生於向上命中的專案根，app 切入該根

#### Scenario: 既有工作流設定不被覆蓋

- **WHEN** 未啟用資料夾的 openspec/config.yaml 已存在且含使用者自訂政策，使用者確認啟用
- **THEN** 匯入後 speclink/config.yaml 與原檔位元級相同，僅補齊其餘缺件

#### Scenario: 啟用時勾選 copilot

- **WHEN** 使用者選定含 openspec/ 但無 .speclink.yaml、已 commit 的資料夾，於啟用確認對話框改為只勾 copilot 並確認
- **THEN** 專案根產生 .speclink.yaml（tools 僅含 copilot）與 .github/skills/ 技能檔，不產生 .claude/、.agents/、CLAUDE.md、AGENTS.md，speclink/ 內既有文件與原 openspec/ 位元級相同

#### Scenario: 有未提交修改時啟用失敗

- **WHEN** 使用者選定的 OpenSpec 資料夾中 openspec/ 有未提交修改，於啟用確認對話框確認
- **THEN** app 顯示單行錯誤說明 openspec/ 有未提交修改，維持原專案，所選資料夾內容不變

## ADDED Requirements

### Requirement: 畫面依實際規格資料夾與資料檔名顯示

桌面 app 畫面上提到規格資料夾或變更資料檔的地方 SHALL 依專案實際使用的資料夾與檔名顯示，SHALL NOT 寫死 `openspec/` 或 `.openspec.yaml`：專案設定各頁標題下的檔案註記（例如 `speclink/config.yaml · context`；舊佈局專案為 `openspec/config.yaml · context`；明寫 `spec_dir` 時為該路徑）與分類欄的組名、討論詳情顯示的記錄路徑。遷移到遠端與來源選擇對話框 SHALL 以「本機規格資料夾」稱呼本機的規格資料夾。變更資料檔解析失敗的卡片提示 SHALL 以「變更資料檔」稱呼，不帶寫死的檔名。新專案的說明文字（初始化對話框、空狀態與來源選擇）SHALL 寫 `speclink/`。文案 SHALL 跟隨 UI 語言。

#### Scenario: 新佈局專案的檔案註記

- **WHEN** 使用者在規格資料夾為 speclink/ 的專案打開專案設定的專案說明頁
- **THEN** 頁標題下的註記為 `speclink/config.yaml · context`，分類欄組名為 `speclink/config.yaml`

#### Scenario: 暫緩遷移的舊佈局專案

- **WHEN** 使用者在仍是 openspec/ 的專案打開專案設定的產出規則頁
- **THEN** 頁標題下的註記為 `openspec/config.yaml · rules`

#### Scenario: 討論詳情的記錄路徑

- **WHEN** 使用者在 speclink/ 專案打開討論 board-search-bar 的詳情
- **THEN** 顯示的記錄路徑為 `speclink/discussions/board-search-bar.md`

#### Scenario: 資料檔解析失敗的提示

- **WHEN** 某變更的資料檔無法解析，使用者在看板看到該卡片
- **THEN** 卡片提示以「變更資料檔」稱呼並附解析原因，不出現 `.openspec.yaml` 字樣
