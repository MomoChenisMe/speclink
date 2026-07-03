## Context

`.speclink.yaml`（AppConfig）目前承載 locale、spec_locale、tdd、audit、tools、spec_dir；`openspec/config.yaml`（WorkflowConfig）承載 schema、context、rules，且已有 locale/spec_locale 作為 fallback（現行解析：app 層勝、workflow 層 fallback）。十六輪討論定案設定三分：工作流政策跟 store、workspace 設定跟 repo、個人差異跟環境變數。本 change 假設 store-trait-and-fs-adapter 已落地（WorkflowConfig 經儲存介面讀取）；其後的 verb-contract-and-remote-client 依賴本 change 的 init 拆分與政策歸屬。

## Goals / Non-Goals

**Goals:**

- 政策欄位（locale、spec_locale、tdd、audit）正典歸屬 `openspec/config.yaml`，四層解析順序明確、可測試。
- 既有專案零修改可運作（舊鍵相容層＋deprecation 警告）。
- tools 開放自訂描述子，init/update 全生命週期（生成、同步、清理）支援。
- 中性（neutral）渲染目標，供描述子與後續 SDK 渲染使用。
- init 內部拆分為 workspace init 與 store init 兩階段。

**Non-Goals:**

- 不引入 `.speclink.remote.yaml` 與模式解析（屬 verb-contract-and-remote-client）。
- 不做 marker 區塊的 remote 變體（同上）。
- 不公開 SDK 渲染 API（屬 node-sdk，本 change 只建立內部渲染基底）。
- overrides 窄覆寫（repo 層級 tdd/audit 刻意分歧）暫不實作——等 verb 契約的 gate 政策一併定案。
- 被否決方案（見討論記錄）：政策欄位整組回歸 workspace 檔（遠端雙真相、SDK 不成立）；為每個 harness 內建 Tool 枚舉值（描述子開放矩陣）；硬切換移除舊鍵（相容層成本低、遷移更平順）。

## Decisions

1. **四層解析順序：環境變數 ＞ 舊 app 鍵（deprecated）＞ config.yaml 正典 ＞ 內建預設**
   - 環境變數：SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT（布林值接受 true/false，其餘值視為未設定）。
   - 舊 app 鍵維持「app 勝」的既有語意（向後相容），但讀到即警告——保留語意、加上遷移訊號。
   - 替代案：config.yaml 直接勝過舊鍵——被否決：會靜默改變既有專案的有效值，違反相容原則。
   - serde 相容：WorkflowConfig 增加 nullable 的 tdd、audit 欄位；AppConfig 欄位全數保留可解析。兩檔既有內容都能讀。

2. **deprecation 警告的形狀**
   - 觸發：`.speclink.yaml` 存在 locale、spec_locale、tdd、audit 任一鍵時。
   - 行為：每次指令執行輸出至 stderr 恰一行，列出偵測到的鍵名並指引「請搬移至 openspec/config.yaml」；stdout（含 `--json`）不受影響。
   - 替代案：只在 init/update 警告——被否決：日常指令才是使用者看得到的地方，一行成本可接受。

3. **tools 描述子：serde 雙形式與驗證**
   - `tools` 清單元素接受字串（內建名）或物件描述子：name（kebab-case，2-50 字，不得與內建名衝突）、skills_dir、instructions_file（皆為專案根相對路徑，正規化後不得逸出專案根）、invocation（cli｜tool-call，預設 cli）。
   - init/update 對描述子與內建工具同一生命週期：生成（skills_dir 下 speclink-*/SKILL.md、instructions_file 的 marker 區塊 upsert）、同步（update 重生成）、清理（自 tools 移除後 update 剝除 marker、移除 speclink-* 技能目錄，空目錄一併移除——語意對齊既有內建工具的 prune）。
   - 驗證失敗（名稱衝突、路徑逸出、invocation 未知值）：exit code 非 0、單行語義化錯誤訊息。

4. **中性渲染目標**
   - 渲染目標抽象為「內建 claude｜內建 codex｜描述子」三態；描述子走 neutral 本體：無 slash 前綴、無 plan mode 參照、`/speclink:` 依 invocation 措辭——cli 措辭為「執行 speclink <動詞>」、tool-call 措辭為「呼叫 speclink 工具（參數為 argv 陣列）」。
   - claude/codex 的既有生成內容位元級不變（回歸保護）。
   - 替代案：維護第二份技能本體——被否決（單一來源，措辭由渲染參數決定）。

5. **init 拆分：workspace init 與 store init**
   - workspace init＝指令檔 marker、技能生成、settings、gitignore（永遠本地、不需網路）；store init＝建立 openspec/ 樹與 config.yaml 範本（僅 fs 儲存執行）。
   - 對外指令與旗標不變；範本內容改變：`.speclink.yaml` 範本瘦身（tools、spec_dir 註解示例），`openspec/config.yaml` 範本增加 locale/spec_locale/tdd/audit 的註解示例區。
   - 命名慣例：內部函式 snake_case（workspace init 與 store init 各自成函式），不新增 CLI 子指令。

6. **instructions payload 的政策來源改為解析結果**：tdd/audit/locale 注入值改取自四層解析（原直讀 AppConfig）；`--json` 欄位名不變。

## Implementation Contract

- **行為**：
  - 只在 config.yaml 設 tdd: true 的專案，技能可觀察到的 tdd 開關為 true（instructions payload 與技能條件內容）。
  - `.speclink.yaml` 與 config.yaml 同鍵衝突時，舊鍵值生效且 stderr 出現一行警告；設定 SPECLINK_TDD=false 時覆寫前兩者。
  - `speclink init` 後：config.yaml 範本含政策示例區、.speclink.yaml 範本不含政策鍵。
  - 描述子 {name: wad-harness, skills_dir: .wad/skills, instructions_file: WAD.md, invocation: tool-call} 經 update 生成對應技能與 marker；自 tools 移除後 update 清理之。
- **介面／資料形狀**：tools 元素的 YAML 雙形式（字串｜物件）；警告訊息單行、含鍵名清單與目標檔名；環境變數四鍵如上。`--json` 無新欄位、無變更欄位。
- **失敗模式**：描述子驗證失敗＝exit code 非 0＋單行語義化訊息（含錯在哪個欄位）；環境變數非法布林值＝視為未設定（不報錯，落到下一層）。
- **驗收條件**：
  - 解析順序矩陣單元測試：4 鍵 × 4 層來源組合，斷言有效值與警告有無。
  - init 範本快照測試：兩檔內容與預期一致。
  - 描述子生命週期測試：生成→同步→移除清理，斷言檔案系統效果；驗證失敗案例斷言 exit code 與訊息。
  - neutral 渲染 golden 測試：cli 與 tool-call 兩種措辭；claude/codex 渲染輸出與既有 golden 完全一致。
  - parity/color/twin 對照：除「含舊政策鍵時多一行 stderr 警告」與 init 範本內容屬刻意更新外，全數通過；對照 fixture 同步更新並記錄。
- **範圍邊界**：in scope＝解析順序、警告、範本、描述子、neutral 渲染、init 拆分、設定篇文件；out of scope＝remote 連接檔、模式解析、marker remote 變體、SDK 公開 API、overrides 窄覆寫。

## Risks / Trade-offs

- [警告行干擾以 stderr 解析輸出的既有腳本] → 警告固定單行、固定前綴，且僅在含舊鍵時出現；文件明載。
- [描述子路徑逸出專案根造成任意寫檔] → 路徑正規化後強制以專案根為前綴，逸出即驗證錯誤（安全紅線）。
- [neutral 渲染與 claude/codex 渲染共用本體導致既有輸出漂移] → claude/codex golden 測試鎖住位元級輸出。
- [twin/parity fixture 含舊政策鍵導致對照大面積失敗] → 刻意更新對照 fixture 為新版設定佈局，並於任務中獨立一步記錄。

## Migration Plan

1. WorkflowConfig 增欄與四層解析函式（含環境變數），矩陣測試先行。
2. deprecation 警告輸出（CLI 層，stderr）。
3. init 拆分與範本更新，快照測試。
4. tools 描述子解析、驗證與 init/update 生命週期。
5. neutral 渲染目標與 golden 測試。
6. instructions payload 來源切換。
7. 對照 fixture 刻意更新＋全量回歸。
8. 設定篇雙語文件與 README 連結。

## Open Questions

（無——歸屬、順序、描述子欄位皆由討論記錄第 6、9、10、11、12 輪定案。）
