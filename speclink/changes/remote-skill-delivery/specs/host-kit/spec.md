## ADDED Requirements

### Requirement: 入口提供這個範圍的技能

HTTP 入口 SHALL 回答技能清單的請求：用戶端給它要的工具（內建工具的名稱，或帶呼叫形式的自訂工具），入口回傳每個工具要安裝的技能。回傳的集合 SHALL 由三件事決定，依序過濾：只給特定工具的技能不給其他工具；這個範圍的流程設定沒有開啟 worktree 時不含兩支 worktree 技能（設定不存在讀作預設值；設定無法解析時保留這兩支；環境變數不參與）；宿主在建立時給了技能清單時只含清單裡的技能。每支技能的內容 SHALL 由引擎內建的範本以「規格資料夾不在 checkout」渲染；宿主套件 SHALL NOT 接受任何自訂或改寫的技能文字。同一個範圍、同一份流程設定、同一組工具，每個使用者得到的內容 SHALL 相同。這個請求是唯讀的：能讀這個範圍的人都能用，SHALL NOT 受寫入能力限制，SHALL NOT 產生提交。入口能回答這個請求時，握手 SHALL 宣告技能範本的版號。宿主在建立時給的技能清單含不認得的名稱時，建立 SHALL 失敗並列出這些名稱。

#### Scenario: worktree 政策開啟時含兩支 worktree 技能

- **WHEN** 範圍的流程設定為 worktree: true，請求 claude 的技能清單
- **THEN** 回應含 apply-with-worktree 與 worktree-merge

#### Scenario: worktree 政策關閉時不含

- **WHEN** 範圍沒有流程設定文件，請求 claude 的技能清單
- **THEN** 回應不含 apply-with-worktree 與 worktree-merge，含其餘的技能

#### Scenario: 宿主只提供部分技能

- **WHEN** 宿主以技能清單 discuss、propose 建立宿主套件，請求 claude 的技能清單
- **THEN** 回應只含 discuss 與 propose

#### Scenario: 只給 Claude 的技能不給其他工具

- **WHEN** 同時請求 claude 與 codex 的技能清單
- **THEN** claude 的清單含 analyze，codex 的清單不含

#### Scenario: 內容以規格不在 checkout 渲染

- **WHEN** 請求 claude 的技能全文
- **THEN** 每支技能的內容不含規格資料夾的路徑，frontmatter 的版本等於握手宣告的技能版號

#### Scenario: 自訂工具得到中性渲染

- **WHEN** 請求一個呼叫形式為 tool-call 的自訂工具的技能全文
- **THEN** 內容的開頭是 tool-call 的呼叫說明，對其他技能的引用是 `speclink-<名稱>`

#### Scenario: 沒有寫入能力的人也能取得

- **WHEN** 寫入能力為否的人請求技能清單
- **THEN** 回應是 200；儲存沒有收到提交

#### Scenario: 技能清單含不認得的名稱

- **WHEN** 以技能清單 discuss、no-such-skill 建立宿主套件
- **THEN** 建立失敗，訊息列出 no-such-skill

#### Scenario: 握手宣告技能版號

- **WHEN** 對有範圍函式的宿主套件送出握手
- **THEN** 回應的 capabilities 含 skills，其 version 是引擎的技能範本版號
