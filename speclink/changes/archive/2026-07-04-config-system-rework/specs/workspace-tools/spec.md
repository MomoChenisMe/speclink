## ADDED Requirements

### Requirement: tools 自訂描述子的接受與驗證
.speclink.yaml 的 tools 清單 SHALL 接受兩種元素形式：內建工具名字串（claude、codex），或自訂描述子物件（欄位：name 必填、skills_dir 必填、instructions_file 必填、invocation 選填且值域為 cli 或 tool-call、預設 cli）。描述子驗證規則：name SHALL 為 kebab-case（2 至 50 字）且 SHALL NOT 與內建工具名衝突；skills_dir 與 instructions_file SHALL 為專案根相對路徑，正規化後 SHALL NOT 逸出專案根。驗證失敗時指令 SHALL 以非 0 exit code 結束並輸出單行語義化錯誤訊息（指明錯誤欄位與原因）。

#### Scenario: 合法描述子生成對應工具檔
- **WHEN** .speclink.yaml 的 tools 含描述子 name: wad-harness、skills_dir: .wad/skills、instructions_file: WAD.md，執行 speclink update
- **THEN** 生成 .wad/skills/speclink-*/SKILL.md 技能檔與 WAD.md 內的 SPECLINK marker 區塊，exit code 為 0

#### Scenario: 名稱與內建工具衝突被拒
- **WHEN** tools 含描述子 name: claude，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 name 與內建工具名衝突

#### Scenario: 路徑逸出專案根被拒
- **WHEN** tools 含描述子 skills_dir: ../outside/skills，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 skills_dir 逸出專案根

### Requirement: 描述子的同步與清理生命週期
speclink update SHALL 對描述子與內建工具一視同仁：在 tools 清單上的描述子重新生成其技能與 marker 區塊；自清單移除的描述子，其生成物 SHALL 被清理——skills_dir 下的 speclink- 前綴技能目錄移除（因而變空的目錄一併移除）、instructions_file 的 SPECLINK marker 區塊剝除（使用者自有內容保留，剝除後全空的檔案刪除）。

#### Scenario: 移除描述子後生成物被清理
- **WHEN** 先以含 wad-harness 描述子的 tools 執行 speclink update，再將該描述子自 tools 移除並重新執行 speclink update
- **THEN** .wad/skills/ 下的 speclink- 前綴目錄被移除，WAD.md 的 marker 區塊被剝除；若 WAD.md 除區塊外無其他內容則整檔刪除

### Requirement: 中性渲染目標
描述子生成的技能與指令區塊 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude 與 codex 的生成內容 SHALL 與本變更前位元級一致。

#### Scenario: tool-call 措辭
- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 既有內建工具輸出不變
- **WHEN** tools 僅含 claude 與 codex，執行 speclink update
- **THEN** 生成的 CLAUDE.md、AGENTS.md marker 區塊與 .claude/skills/、.agents/skills/ 技能內容與本變更前的 golden 基線完全一致
