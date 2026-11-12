## ADDED Requirements

### Requirement: 技能依規格是否在 checkout 渲染

技能的渲染 SHALL 有一個「規格資料夾是否位於這個 checkout」的輸入，並依它處理原稿的條件區塊。speclink init 於本機模式產生的技能 SHALL 以「在」渲染；speclink init --store remote 產生的技能 SHALL 以「不在」渲染；speclink update 與其他同步技能檔的入口 SHALL 依專案設定決定：`.speclink.yaml` 含 remote 區段時以「不在」渲染，否則以「在」渲染，SHALL NOT 依規格資料夾是否存在於磁碟判斷。技能檔過期的探測 SHALL 用相同的判斷產生比對基準。本機專案以「在」渲染的技能中，對規格檔的 git 步驟與規格路徑的顯示 SHALL 保留；以「不在」渲染的技能 SHALL NOT 含這些內容。render golden 基線 SHALL 增列 Claude 與中性 tool-call 兩種目標以「不在」渲染的快照。

#### Scenario: 本機專案的 worktree 技能保留規格檔的 commit 步驟

- **WHEN** 在開啟 worktree 政策的本機專案執行 speclink update，檢視產生的 apply-with-worktree 技能
- **THEN** 它含「變更的 artifact 尚未 commit 時先 commit」的步驟與對規格資料夾的 git 指令

#### Scenario: remote 專案的技能沒有規格檔的 git 步驟

- **WHEN** 在 `.speclink.yaml` 含 remote 區段的 checkout 執行 speclink update，檢視產生的 commit 技能
- **THEN** 它不含規格資料夾的路徑與對規格檔的 git 步驟，並含一句說明這次 commit 只含程式碼的檔案

#### Scenario: 殘留的規格資料夾不影響判斷

- **WHEN** remote 專案的 checkout 裡留有一個舊的 speclink 資料夾，執行 speclink update
- **THEN** 產生的技能仍然以「不在」渲染

#### Scenario: 探測用相同的基準

- **WHEN** remote 專案剛以 speclink update 產生技能檔，隨即探測技能檔是否過期
- **THEN** 探測結果是目前的版本，沒有任何檔案被列為不同

## MODIFIED Requirements

### Requirement: 中性渲染目標
<!-- BEFORE: 中性渲染只規定不含 /speclink- 前綴與 plan mode 參照；對其他技能的引用渲染成 speclink 加空白加名稱 -->

描述子生成的技能檔 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；對其他技能的引用 SHALL 渲染成 `speclink-<名稱>`（技能的名稱），SHALL NOT 渲染成與動詞相同的 `speclink <名稱>` 形式；開頭的呼叫說明 SHALL 寫明 `speclink-<名稱>` 是另一支技能、不是動詞，不得當成指令或工具的參數送出；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude、codex、copilot、pi 與 cursor 的生成內容 SHALL 與 render golden 基線位元級一致；基線 SHALL 僅隨提案記載的刻意變更同批更新。

#### Scenario: tool-call 措辭

- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 內建工具輸出鎖定於 golden 基線

- **WHEN** tools 僅含 claude、codex、copilot、pi 與 cursor，執行 speclink update
- **THEN** 生成的 .claude/skills/、.agents/skills/、.github/skills/、.pi/skills/、.cursor/skills/ 技能內容與 render golden 基線完全一致

#### Scenario: 交棒寫成技能名稱

- **WHEN** 原稿的收尾段引用 apply、archive 與 review 三支技能，以中性的 tool-call 形式渲染
- **THEN** 結果以 `speclink-apply`、`speclink-archive`、`speclink-review` 表述這三處；不含反引號內以 `speclink apply`、`speclink archive <` 開頭的交棒寫法

#### Scenario: 呼叫說明分辨技能與動詞

- **WHEN** 檢視中性 cli 與 tool-call 兩種形式產生的任一技能檔的開頭說明
- **THEN** 說明含一句：寫成 `speclink-<name>` 的是另一支技能，不是動詞

#### Scenario: 真的動詞仍是動詞寫法

- **WHEN** 原稿的步驟寫 `speclink archive <name>`（執行封存動詞），以中性的 tool-call 形式渲染
- **THEN** 該步驟維持 `speclink archive <name>`
