## ADDED Requirements

### Requirement: remote 專案的技能取自 Server

`.speclink.yaml` 含 remote 區段的專案執行 speclink update 時，引擎 SHALL 先握手：Server 宣告技能時，SHALL 以 `.speclink.yaml` 的工具清單向 Server 請求技能全文，並把回應寫成各工具技能目錄下的受管技能檔——檔案集合與內容 SHALL 等於 Server 的回應，checkout 多出來的 speclink- 技能目錄 SHALL 依既有的清理生命週期移除。Server 對某個工具的回應不含任何技能時，SHALL NOT 寫入或移除該工具技能目錄下的任何檔案，並 SHALL 於 stdout 以一行英文列出這些工具、說明它們的技能檔維持原狀；其餘工具照常安裝。Server 沒有宣告技能、沒有可用的憑證、無法連線、或握手沒有成功時，SHALL 退回本機的範本（以「規格資料夾不在 checkout」渲染），並於 stderr 以單行英文說明原因，exit code 為 0。握手宣告了技能而技能請求失敗時 SHALL 以非零 exit code 結束，SHALL NOT 寫入任何檔案，SHALL NOT 退回本機的範本。Server 的回應與 checkout 對不上時——技能名稱不是由小寫英數與連字號組成、至多 64 字元的名稱，同一個工具的技能名稱重複，回應缺少全文，或回應的工具不等於請求的工具——SHALL 同樣以非零 exit code 結束且 SHALL NOT 寫入任何檔案：技能名稱會成為資料夾名稱，不合形式的名稱 SHALL NOT 成為路徑。stdout 的摘要 SHALL 指出技能的來源（server 或 local）與版本。speclink init --store remote 的流程不變，結尾 SHALL 多一行提示：登入後執行 speclink update 以取得 Server 提供的技能。本機專案 SHALL NOT 向任何 Server 請求技能。寫入的技能檔 SHALL 與本機產生的技能檔同形（同樣的路徑與 frontmatter），既有的探測與清理 SHALL 能照常處理它們。

#### Scenario: 自 Server 安裝

- **WHEN** remote 專案的 .speclink.yaml 的 tools 為 claude 與 codex，Server 宣告技能，執行 speclink update
- **THEN** .claude/skills 與 .agents/skills 下每個 speclink- 技能檔的內容等於 Server 對該工具回應的內容；stdout 指出來源為 server 與其版本；exit code 為 0

#### Scenario: 舊 Server 退回本機範本

- **WHEN** Server 的握手沒有宣告技能，執行 speclink update
- **THEN** 技能檔由本機的範本產生，stderr 有一行說明 Server 沒有提供技能，exit code 為 0

#### Scenario: 無法連線退回本機範本

- **WHEN** Server 不可達，執行 speclink update
- **THEN** 技能檔由本機的範本產生，stderr 有一行說明無法連線，exit code 為 0

#### Scenario: 宣告了卻取不到

- **WHEN** Server 的握手宣告技能，但技能請求回應 500，執行 speclink update
- **THEN** exit code 非零，stderr 說明取得技能失敗，checkout 的技能檔逐位元不變

#### Scenario: 回應的技能名稱是路徑

- **WHEN** Server 回應的技能名稱為 `../../escape`，執行 speclink update
- **THEN** exit code 非零，checkout 的技能檔逐位元不變，checkout 之外沒有新檔案

#### Scenario: Server 不再提供的技能被清掉

- **WHEN** checkout 有 speclink-improve 技能目錄，Server 的回應不含 improve，執行 speclink update
- **THEN** speclink-improve 目錄被移除

#### Scenario: Server 對某個工具不提供任何技能

- **WHEN** .speclink.yaml 的 tools 為 claude 與 codex，兩者都已有技能檔；Server 對 claude 回應 apply、對 codex 回應空的清單，執行 speclink update
- **THEN** exit code 為 0；.claude/skills 的技能檔等於 Server 的回應；.agents/skills 下的每個檔案逐位元不變；stdout 有一行指出 codex 沒有拿到技能、技能檔維持原狀

#### Scenario: 自訂工具的技能自 Server 取得

- **WHEN** .speclink.yaml 含一個 invocation 為 tool-call 的自訂描述子，Server 宣告技能，執行 speclink update
- **THEN** 該描述子的技能目錄下的技能檔內容等於 Server 對它回應的中性渲染

#### Scenario: 本機專案不連線

- **WHEN** 在沒有 remote 區段的專案執行 speclink update
- **THEN** 沒有發出任何網路請求，行為與輸出和既有行為相同

### Requirement: CLI 的技能版本提醒

remote 動詞握手之後，Server 宣告的技能版號在數值上新於本機 CLI 的技能範本版號時，CLI SHALL 於 stderr 印出一行英文提醒，內容含兩邊的版號與更新 Speclink 的方式。speclink update 自己的握手 SHALL NOT 印出這一行，也 SHALL NOT 用掉當天的那一次：它的摘要已經指出技能的來源與版本。同一個 Server 位址同一個日曆日 SHALL 最多提醒一次；記錄 SHALL 存於使用者層級設定目錄，SHALL NOT 寫入專案內的任何檔案。提醒 SHALL NOT 改變 stdout、`--json` 的內容與 exit code，SHALL NOT 阻擋動詞的執行。任一邊的版號無法解析為數字段、Server 沒有宣告、或本機不舊於 Server 時 SHALL NOT 提醒。記錄檔無法讀寫時 SHALL NOT 提醒也 SHALL NOT 報錯；記錄檔不存在、或內容不是記錄時視為沒有記錄。CLI SHALL NOT 自行下載或安裝更新。

#### Scenario: 本機較舊時提醒一次

- **WHEN** 本機 CLI 的技能版號為 v1.57.0，Server 宣告 v1.58.0，當天第一次執行 speclink list --json
- **THEN** stdout 是清單的 JSON，exit code 為 0，stderr 有一行含 v1.57.0、v1.58.0 與更新方式

#### Scenario: 同一天不重複

- **WHEN** 同一天對同一台 Server 再執行一次 speclink list
- **THEN** stderr 沒有提醒

#### Scenario: 隔天再提醒

- **WHEN** 隔一個日曆日再執行 speclink list
- **THEN** stderr 再次出現提醒

#### Scenario: 本機較新或相同時不提醒

- **WHEN** 本機 CLI 的技能版號等於或新於 Server 宣告的版號，執行 speclink list
- **THEN** stderr 沒有提醒

#### Scenario: update 不提醒也不用掉當天的那一次

- **WHEN** 本機 CLI 的技能版號舊於 Server 宣告的版號，當天先執行 speclink update，再執行 speclink list
- **THEN** speclink update 的 stderr 沒有提醒；之後的 speclink list 出現提醒

#### Scenario: 另一台 Server 各自計算

- **WHEN** 當天已對 Server 甲提醒過，改連到同樣較新的 Server 乙執行 speclink list
- **THEN** stderr 出現對 Server 乙的提醒

## MODIFIED Requirements

### Requirement: worktree 技能的政策條件式生成
<!-- BEFORE: 一律依本機 speclink/config.yaml 的 worktree 檔值過濾 -->

技能足跡生成（speclink init 與 speclink update）SHALL 依流程設定的 worktree 檔值過濾兩顆 worktree 技能——本機專案取自 speclink/config.yaml；remote 專案自 Server 取得技能時由 Server 依該範圍的流程設定過濾，remote 專案退回本機範本時（Server 沒有宣告技能或無法連線）取本機沒有流程設定的預設值——（speclink-apply-with-worktree 與 speclink-worktree-merge）：值為 true 時生成，false 或未設時不生成，且既有生成物依既有清理生命週期移除（技能目錄移除，因而變空的目錄一併移除）。生成判定 SHALL NOT 納入 SPECLINK_WORKTREE 環境變數（環境層僅影響執行期政策，不影響足跡）。其餘技能的生成集合不受 worktree 政策影響。此過濾對 claude、codex、copilot、pi、cursor 與自訂描述子工具一視同仁。

#### Scenario: 政策關閉時生成集合不含 worktree 技能

- **WHEN** speclink/config.yaml 無 worktree 鍵（或值為 false）的專案執行 speclink update
- **THEN** 各工具 skills 目錄不含 speclink-apply-with-worktree 與 speclink-worktree-merge 目錄，其餘技能照常生成

#### Scenario: 政策開啟時注入兩顆技能

- **WHEN** speclink/config.yaml 含 worktree: true 的專案執行 speclink update
- **THEN** 各工具 skills 目錄含兩顆 worktree 技能，內容與 golden 對照一致

#### Scenario: 政策由開改關後再生即清理

- **WHEN** 先於 worktree: true 下執行 speclink update，再將該鍵改為 false 並重新執行 speclink update
- **THEN** 兩顆 worktree 技能目錄被移除，其餘技能保留

#### Scenario: 環境變數不影響生成

- **WHEN** speclink/config.yaml 無 worktree 鍵，但於 SPECLINK_WORKTREE=true 環境下執行 speclink update
- **THEN** 兩顆 worktree 技能不生成——環境層僅於技能執行期由 P1 政策檢查讀取

#### Scenario: 過期探測不把被政策排除的技能報成過期

- **WHEN** worktree 政策關閉的專案於技能檔已同步的狀態下執行技能檔過期探測
- **THEN** 探測結果不列出兩顆 worktree 技能——被政策排除者不屬於預期生成集合

#### Scenario: remote 專案的 worktree 技能跟著 Server 的流程設定

- **WHEN** Server 上該範圍的流程設定為 worktree: true，在連到它的 checkout 執行 speclink update
- **THEN** checkout 出現 speclink-apply-with-worktree 與 speclink-worktree-merge 兩個技能目錄；之後 Server 的設定改成 false 並再次執行 speclink update，這兩個目錄被移除

### Requirement: 受管檔再生的降級守門
<!-- BEFORE: 守門一體適用於所有再生入口，沒有 remote 專案的例外 -->

引擎的受管檔再生 SHALL 於任何寫入之前判定方向：即將被新建或改寫的技能檔中，任一檔的 frontmatter 版號數值領先當前產物層版號時 SHALL 拒絕執行——輸出單行英文說明（含工作區領先的版號與引擎現版版號）、exit code 非零、SHALL NOT 寫入任何檔案（含設定檔）。判定目標 SHALL 取自該次再生的實際寫入集（tools 清單選集、無清單時的目錄偵測、自訂描述子的技能檔），SHALL NOT 僅取內建工具。守門 SHALL 一體適用於所有經再生入口的路徑：`speclink update`、`speclink init --force` 的重建、工具選集收斂、`workflow-config` 寫入後的技能足跡同步（CLI 與桌面設定頁）、桌面的技能檔更新動作。remote 專案自 Server 取得技能時 SHALL NOT 套用本守門：技能版本跟著 Server，Server 提供的版本舊於 checkout 既有的版本時 SHALL 照常寫入，並於輸出說明技能已換成 Server 提供的版本；remote 專案退回本機範本時本守門照常適用。旗標 `--allow-downgrade`（`speclink update`）SHALL 為本機範本路徑上唯一的明示越過入口；`--force` SHALL NOT 被視為同意降級。缺失、過期、現版與無法判定情境下的既有行為 SHALL 不變。守門訊息 SHALL 為英文單行（與 update 既有輸出語言一致），不隨 locale 設定變化。

#### Scenario: 較新工作區拒絕 update

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink update
- **THEN** stderr 單行說明含兩個版號、exit code 非零、工作區零檔案變動

#### Scenario: --allow-downgrade 越過守門

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink update --allow-downgrade
- **THEN** 受管檔照常再生為引擎現版內容、exit code 0

#### Scenario: 過期工作區照常更新

- **WHEN** 工作區技能檔版號舊於引擎現版，執行 speclink update（不帶旗標）
- **THEN** 受管檔照常再生、exit code 0，行為與守門引入前相同

#### Scenario: --force 重建不等於同意降級

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink init --force
- **THEN** 單行說明含兩個版號、exit code 非零、工作區零檔案變動；`--force` SHALL NOT 越過守門

#### Scenario: 無 tools 清單的工作區同受守門

- **WHEN** `.speclink.yaml` 未記錄 tools 清單（再生走目錄偵測），而既有技能檔的版號新於引擎現版，執行 speclink update
- **THEN** 拒絕執行、零檔案變動；SHALL NOT 因判定面只看內建清單而放行

#### Scenario: 自訂描述子的技能檔同受守門

- **WHEN** tools 清單只含自訂描述子，其 skills_dir 下技能檔的版號新於引擎現版，執行 speclink update
- **THEN** 拒絕執行、零檔案變動

#### Scenario: 技能足跡同步被拒時的失敗形狀

- **WHEN** 工作區技能檔版號新於引擎現版，執行 speclink workflow-config set worktree true
- **THEN** 設定檔已寫入新值，技能足跡同步被守門拒絕——單行說明含兩個版號、exit code 非零、受管檔零變動（沿用同步失敗的既有形狀）

#### Scenario: remote 專案換成 Server 較舊的版本

- **WHEN** remote 專案 checkout 的技能檔版號為 v1.60.0，Server 宣告並提供 v1.58.0，執行 speclink update（不帶 --allow-downgrade）
- **THEN** exit code 為 0，技能檔的版號變成 v1.58.0，stdout 說明技能已換成 Server 提供的版本

#### Scenario: remote 專案退回本機範本時守門仍然有效

- **WHEN** remote 專案 checkout 的技能檔版號領先本機引擎，Server 沒有宣告技能，執行 speclink update
- **THEN** exit code 非零，輸出既有的降級拒絕訊息，沒有任何檔案被寫入
