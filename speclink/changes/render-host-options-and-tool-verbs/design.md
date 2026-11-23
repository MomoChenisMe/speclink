## Context

技能的渲染在 `speclink-core` 的 `workspace/skills.rs`：`render_skill_file_for(target, skill, spec_dir, location)` 產生 frontmatter、前言、對話語言段落與內文。前言現在有兩種：Claude 的 fork 說明，以及自訂工具（SDK 的 `neutral` target）的 `## Invocation` 段。內文裡的條件區塊只有一個條件 `SPEC_IN_CHECKOUT`。Node SDK 的 `skills.render`（`speclink-node` 的 `src/render.rs`）直接轉呼叫這個函式；`index.js` 把 options 原樣交給 napi 的物件型別，所以不認得的鍵被丟掉而沒有任何提示。

`speclink` 工具的說明在 `speclink-verbs` 的 `tool.rs`：`tool_definition()` 走訪 clap 的動詞樹，每個有引擎命令的末端動詞一行。Node 把結果凍結成常數 `agentTool`。

對照程式碼確認的事實：

- 技能原稿裡每一處 `AskUserQuestion` 都已經有「沒有這個工具就用文字問並等回答」的退路，所以不需要「有沒有提問工具」的選項。
- `neutral` target 是給五個內建 AI 工具以外的 harness 用的，假設有程式碼的 checkout。`specInCheckout: false` 只表示規格不在 checkout 裡。
- 來源文件寫的代價是模型「可能」做錯，沒有實際觀察到的錯誤。

限制：CLI 為每個工具產生的技能檔是回歸保護對象（`render_golden` 與 `skill_verbization` 測試）；改技能原稿要連動內嵌資產的版本、golden 與 lock 檔。

## Goals / Non-Goals

**Goals:**

- 宿主用兩個選項告訴模型這個環境的限制，文字由引擎產生，每個宿主得到同一段。
- 打錯選項鍵名時立刻知道。
- 宿主能產生只列自己開放的動詞的工具說明。
- 不帶新選項時所有既有輸出逐位元不變。

**Non-Goals:**

- 改技能原稿的內文、新增條件區塊（沒有程式碼時把整段步驟換掉）：討論決定延後，等實際看到模型做錯再針對那一支技能處理。
- 「有沒有提問工具」的選項：退路已經在技能裡。
- 「檔案搜尋」與「git 歷史」分成兩個選項：沒有只缺其中一樣的情境。
- 給內嵌宿主另開一個 target：`neutral` 維持原意。
- 讓 `createHost` 的 `skills`（入口告訴用戶端安裝哪些技能）影響 `skills.render`：兩者維持各自獨立，文件已寫明。
- CLI 的 `speclink init`／`speclink update` 暴露這兩個選項。

## Decisions

### 前言由引擎產生，技能內文不改

在 `speclink-core` 的 `workspace/skills.rs` 新增 `render_skill_file_with(target, skill, spec_dir, location, limits)`，`limits` 是新型別 `HarnessLimits { skills: Option<Vec<String>>, checkout: bool }`（預設是沒有清單、有 checkout）。既有的 `render_skill_file_for` 改成以預設的 `limits` 轉呼叫它，所以全部既有的呼叫端（CLI 的產生、測試、golden）不必改、輸出不變。

`limits` 不是預設值時，在 frontmatter 與既有前言（fork 說明或 `## Invocation`）之後、對話語言段落之前，多寫一段 `## Harness limits`，每個限制一個項目，段落以 `---` 結束，格式和 `## Invocation` 段相同。這是唯一的實作落點：Node SDK 只負責把 options 轉成 `HarnessLimits`。

替代方案：在技能原稿加第二個條件區塊。否決理由見 Non-Goals。替代方案：只補文件、教宿主自己加前言。否決，因為每個宿主各寫各的，用語和技能對不上。

### skills 選項：可用的技能清單

文字（`<list>` 是清單裡的技能，依技能登錄表的順序，以這個 target 引用技能的形式列出，例如 neutral 是 `speclink-discuss`，以逗號加空格分隔）：

> This harness provides only these skills: `<list>`. A skill that is not in this list is not available here: do not load it, and do not tell the user to run it. When a step of this document points to such a skill, tell the user that an engineer does that step in a checkout of the code.

驗證（任何一項不符就拋出錯誤、不渲染）：值必須是字串陣列；不能是空陣列；每個名稱都要在 `skills.list()` 裡（不帶 `speclink-` 前綴，例如 `propose`）；清單必須包含正在渲染的那一支技能。重複的名稱視為一個。

### checkout 選項：沒有程式碼與 git 歷史

`checkout` 為 false 時的文字：

> This harness has no checkout of the code and no git history. Skip every step of this document that reads code files, searches the code, or runs a git command. Check facts only with speclink verbs. In your answer, say which steps you skipped for this reason.

規則：`checkout` 必須是布林（其他值拋出錯誤，和 `specInCheckout` 的既有規則相同）；省略或 true 時不寫這一項。`checkout` 為 false 表示連程式碼都沒有，所以規格也不在 checkout 裡：`specInCheckout` 省略時視同 false；明確給 true 時拋出錯誤，訊息指出兩個選項互相矛盾。

### 渲染 API 拒絕不認得的選項鍵

在 `crates/adapters/speclink-node/index.js` 的 `skills.render` 包裝檢查 options 的鍵：合法的是 `target`、`invocation`、`specInCheckout`、`specDir`、`toolName`、`skills`、`checkout`。有其他的鍵時拋出 Error，訊息含那個鍵與合法鍵的清單。做法比照同一個檔裡 `createHost` 對 `HOST_OPTIONS` 的檢查。檢查放在 JS 這一側，因為 napi 的物件型別在進到 Rust 之前就丟掉了不認得的鍵。

### 工具說明的動詞篩選

在 `speclink-verbs` 的 `tool.rs` 新增 `tool_description(exclude)`：和 `tool_definition()` 走訪同一棵動詞樹，略過被排除的動詞後組出說明；`tool_definition()` 的 `description` 改成呼叫它（不排除任何動詞），兩者只有一份組字串的程式。Node 新增匯出 `agentToolDescription(options)`。

排除的規則：`exclude` 的每一項是動詞的文字，字與字之間一個空格（`archive`、`review`、`discuss archive`）。一項比對到「字序列以它開頭」的全部末端動詞：`review` 排除 `review show`、`review add-round`、`review stamp`、`review discard`；`archive` 只排除頂層的 `archive`，不排除 `discuss archive`。驗證：`exclude` 必須是字串陣列；每一項至少要比對到清單裡的一個動詞，否則拋出錯誤並指出那一項（需要 checkout 的動詞本來就不在清單裡，排除它們也是錯誤）；options 有 `exclude` 以外的鍵時拋出錯誤。全部動詞都被排除時仍然回傳說明，只是動詞清單是空的。

替代方案：給 `agentTool` 物件加方法。否決，因為 node-sdk 的規格把 `agentTool` 定義成只有三個欄位的常數，而且它是凍結的資料。替代方案：接受要保留的動詞清單。否決，因為討論與來源文件要的是排除，而且宿主的權限表已經有 `commandNames` 的測試提醒新命令。

## Implementation Contract

**行為**

- `skills.render(name, { …, skills: [...] })` 的回傳字串含 `## Harness limits` 段與可用技能的那一項；`checkout: false` 時含沒有 checkout 的那一項；兩個都給時兩項都有，技能那一項在前。
- 兩個選項都沒有給時，回傳字串和這個變更之前逐位元相同，沒有 `## Harness limits`。
- 技能內文（對話語言段落之後的部分）在有沒有這兩個選項時都相同，除了 `checkout: false` 連帶讓 `specInCheckout` 變成 false 時，內文是 `specInCheckout: false` 的那一份。
- `agentToolDescription()` 等於 `agentTool.description`；帶 `exclude` 時，被排除的動詞沒有自己的那一行，其餘各行與說明開頭的用法段落不變。

**介面與資料形狀**

- `RenderOptions` 多 `skills?: string[]` 與 `checkout?: boolean`。
- 新匯出 `agentToolDescription(options?: { exclude?: string[] }): string`。
- Rust：`HarnessLimits`、`render_skill_file_with`、`tool_description`。

**失敗模式**（都是同步拋出 Error，不回傳部分結果）

- `skills` 不是字串陣列、是空陣列、含不認得的名稱、不含正在渲染的技能。
- `checkout` 不是布林；`checkout: false` 加上 `specInCheckout: true`。
- options 含不認得的鍵。
- `exclude` 不是字串陣列、某一項比對不到任何動詞；`agentToolDescription` 的 options 含不認得的鍵。

**驗收條件**

- `cargo test -p speclink-core --test it render_golden::` 與 `cargo test -p speclink-core --test it skill_verbization` 不改任何 golden 就通過。
- `cargo test -p speclink-core skills` 的新測試：預設的 `limits` 與 `render_skill_file_for` 逐位元相同；兩種限制各自與同時出現時的段落文字與位置。
- `cargo test -p speclink-verbs tool` 的新測試：不排除時與 `tool_definition().description` 相同；前綴比對；比對不到時的錯誤。
- `crates/adapters/speclink-node/__test__/render.spec.ts` 與 `agent-tool.spec.ts` 涵蓋上面每一種行為與失敗模式。

**範圍界線**

- 在範圍內：上述 Rust 與 Node 的程式、型別定義、`docs/sdk-node` 的中英文說明。
- 不在範圍內：`crates/engine/speclink-core/assets/skills/` 底下的任何檔案、內嵌資產的版本、CLI 的指令與輸出、`createHost` 的選項、入口的路由。

## Risks / Trade-offs

- [前言說「跳過讀程式碼的步驟」，內文仍寫著用 Grep 找程式碼，模型可能照內文做] → 前言放在內文之前並用明確的祈使句；討論已接受這個取捨，實際看到錯誤時再改那一支技能的內文。
- [不認得的鍵報錯會讓原本「多給也沒事」的呼叫端壞掉] → 提案標成 BREAKING；錯誤訊息列出合法的鍵；`docs/sdk-node` 寫明。
- [`checkout: false` 悄悄把 `specInCheckout` 當成 false] → 規格與文件寫明；明確給 true 時報錯而不是默默覆蓋。
- [回歸對照] → 既有的 golden 與 CLI 測試不改就要通過；新行為只在帶新選項時出現，由新的測試涵蓋。
- [跨平台] → 沒有路徑處理、檔案系統與 git 呼叫；前言的文字以 `\n` 換行，和既有的前言相同。Windows、macOS、Linux 的輸出相同。
- [前言的英文文字沒有經過技能的寫法守門] → 任務裡把新文字對照 skill-authoring 的寫法規範審閱一次，並確認既有的守門測試不因新函式而失敗。
- [改到 napi 的匯出後忘記重新產生 binding 檔] → 任務的驗證步驟先跑 Node 的建置再跑測試。
