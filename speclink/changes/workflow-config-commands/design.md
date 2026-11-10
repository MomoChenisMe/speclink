## Context

流程設定是規格資料夾裡的 `config.yaml`：預設 schema、五個政策欄位（locale、spec_locale、tdd、audit、worktree）、context、各 artifact 的 rules、review 規則、子代理模型。

今天的讀寫路徑：

- **編輯的邏輯**在引擎：`speclink-core` 的 `workspace/config.rs` 有 `WorkflowConfig::from_text`、`from_store`、`update_workflow_config_text`（收 `WorkflowPolicyFields` 與 `ContentEdit`）、`validate_policy_locales`。它們做文字層級的改寫，保留使用者的排版與註解。
- **CLI**（`verbs/config.rs`）：`cmd_workflow_config` 先把 argv 與 stdin 正規化成中間型別（`WorkflowConfigWrite`、`WorkflowConfigPlan`），再分流到 `workflow_config_fs`（用 `core::util` 直接讀寫檔案）或 `remote_workflow_config`（GET 取得內容與版本，本機改寫，PUT 整份加預期版本）。共 11 個末端動詞：show、set、context、rules、review 的四個、subagent-model 的三個。
- **示範用的 server**（`api/routes.rs`）：`config` 路由直接從快照讀文件；`put_config` 路由自己解析與驗證、比對版本、開 unit of work 寫入。兩者都不經命令。
- **引擎的儲存介面**有 `read_workflow_config`，沒有寫入的方法。
- **命令層**沒有任何一種命令碰流程設定。

CLI 在寫入前後還做三件需要 checkout 的事：關閉 worktree 政策前檢查有沒有進行中的 worktree；worktree 欄位變動後同步技能檔；審查規則變動後提示技能檔過期。

這個變更排在 shared-verb-parsing 與 shared-verb-output 之後，所以動詞的解析與輸出已經有共用的落點（`speclink-verbs`）。它也排在 engine-on-store-port 之後，Host 的儲存視圖已經是 `PortView`。

## Goals / Non-Goals

**Goals:**

- 流程設定的讀與寫各有引擎命令，CLI 的本機分支、示範用的 server、Node SDK 都經它執行。
- 寫入的結果說得出改了哪些鍵，而且不寫入也能算出來。
- 寫入的唯一實作落點在引擎；各入口不再自己開檔或自己開交易。
- 朝「storage 解耦的規格驅動引擎」靠攏：流程設定的寫入也經儲存介面。

**Non-Goals:**

- 不改 `config.yaml` 的格式與任何欄位。
- 不改線上協定：讀取仍回傳內容與版本，寫入仍是整份加預期版本。
- 不改 CLI 遠端分支的流程（讀、本機改寫、整份寫回）。
- 不改桌面 app 本機模式的寫入。
- 不新增領域事件。宿主要怎麼得知設定被改，由提供掛鉤的變更決定。
- 不提供執行前檢查本身。
- 不把全域的使用者設定（`speclink config`）搬進命令層。

## Decisions

### D1: 三種命令

`Command` 新增三個變體，實作放在 `command/workflow_config.rs`：

- `WorkflowConfigShow`：結果 `WorkflowConfigShowOutcome`，帶解析後的 `WorkflowConfig` 與「檔案是否存在」。壞的 YAML 回傳 `invalid_config`，和今天 show 的行為相同。
- `WorkflowConfigEdit { edit, dry_run }`：`edit` 是 `WorkflowConfigEdit` 列舉——設定一個政策欄位、取代或移除 context、取代或移除一個 artifact 的規則、取代審查規則的一個嚴重度清單、設定 baseline、設定或移除一個子代理模型、取代一個工具的模型清單。它就是 CLI 今天的中間型別，搬進引擎並改成公開。
- `WorkflowConfigReplace { content }`：整份取代。

兩種寫入的結果都是 `WorkflowConfigWriteOutcome { before, after, changed_keys, written }`。`before` 在檔案不存在時是「沒有」。

寫入命令的步驟與失敗後的狀態：

1. 經儲存介面讀出現有內容。
2. 算出新內容：編輯用 `update_workflow_config_text`；整份取代直接用給的內容。
3. 驗證新內容：`WorkflowConfig::from_text` 與 `validate_policy_locales`。失敗回傳 `invalid_config`，儲存沒有變動。
4. 算出變動的鍵（D2）。
5. 新內容和現有內容逐位元相同，或 `dry_run` 為真 → 不寫入，`written` 是假。
6. 否則經儲存介面寫入一次，`written` 是真。

一個命令最多一次寫入，所以沒有半套狀態。經 Host 的儲存視圖時，這次寫入是該次提交唯一的一筆，預期版本取自第 1 步讀到的版本；別人在中間改過時提交回報衝突。

- **否決：只做整份取代一種命令** — 宿主的 Agent 用的是 `workflow-config set tdd true` 這種動詞文字，讀、改、寫要在同一次執行裡完成，才不會在兩次呼叫之間被別人蓋掉。
- **否決：每個末端動詞各一種命令** — 十個寫入命令只差在編輯的內容，用一個列舉表達即可。

### D2: 變動的鍵 changed_keys

比較寫入前後解析出的 `WorkflowConfig`，回傳有差異的鍵，依固定順序排列：

- 單層的鍵：`schema`、`locale`、`spec_locale`、`tdd`、`audit`、`worktree`、`context`。
- 兩層的鍵：`rules.<artifact>`、`review.baseline`、`review.critical`、`review.warning`、`review.suggestion`、`subagent_model.review`、`subagent_model.verify`、`subagent_model.choices`。

只改排版或註解、解析結果沒有差異時，`changed_keys` 是空的，但內容不同所以仍會寫入。檔案從不存在變成存在時，回傳新內容裡有設定值的每個鍵。

這個函式是純函式，放在 `workspace/config.rs`，單獨公開。之後的執行前檢查能用 `dry_run` 的命令取得同一份結果。

### D3: 儲存介面新增寫入流程設定

`Store` trait 新增 `write_workflow_config(&self, content: &str) -> Result<()>`，預設實作回傳錯誤「這個儲存不支援寫入流程設定」。明確實作的有：

- `speclink-fs` 的 `FsStore`：寫 `<spec_dir>/config.yaml`，用既有的寫檔函式。
- `speclink-host` 的 `PortView`：暫存一筆對鍵 `config.yaml` 的寫入。
- `speclink-core` 的 `TestStore`。
- `speclink-node` 的 JS 儲存橋接：呼叫宿主的 `writeWorkflowConfig`（D6）。

worktree 的覆蓋層與只讀的 drift 儲存用預設的拒絕。CLI 今天對「在 worktree 裡改流程設定要寫到哪個 checkout」已有規則（`worktree_write_target`）：CLI 依它選好要交給命令的儲存，規則本身不動。

### D4: workflow-config 成為 speclink-verbs 的第十二個家族

- `families/workflow_config.rs`：clap 型別從 CLI 原樣搬來（含說明文字用的靜態字串），加入 `SpecVerb`；`into_command` 把 11 個末端對到 D1 的三種命令。寫入動詞的 stdin 規則是「帶 `--stdin` 才讀」，和今天的 `require_stdin_flag` 相同。
- `render/workflow_config.rs`：show 的人眼輸出與 `--json`、各寫入動詞的確認訊息、`--dry-run` 的差異（`unified_diff` 跟著搬）。
- CLI 的 `config` 家族（全域的使用者設定）留在 CLI。

搬移的方法與對照沿用前兩個變更：動手前存下這個家族的 `--help` 與各動詞四種形式的輸出，搬完逐位元比對。

### D5: CLI 的本機分支與需要 checkout 的處理

`workflow_config_fs` 改成：選儲存 → 前置檢查 → 執行命令 → 印出共用的輸出 → 後置處理。

- **前置檢查**（留在 CLI）：編輯會把 worktree 關掉時，檢查有沒有進行中的 worktree，有就拒絕。CLI 用 `dry_run` 的命令或直接看 `edit` 判斷這次是不是在關 worktree。
- **後置處理**（留在 CLI）：`changed_keys` 含 `worktree` 時同步技能檔；含任何 `review.` 開頭的鍵時印出技能檔過期的提示。判斷改用命令回傳的 `changed_keys`，不再各自比對。
- 這兩段的輸出文字與順序不變。

`remote_workflow_config` 不動：它讀回內容與版本，用引擎同一個改寫函式在本機算出新內容，再整份寫回。

### D6: Node SDK 的流程設定動詞與選填的儲存方法

dispatch 經共用的解析與輸出，自動得到 `workflow-config` 的 11 個動詞。

宿主的 JS 儲存新增選填方法 `writeWorkflowConfig(content: string): void | Promise<void>`。處理方式和蓋章的三個選填方法相同：建立引擎時記下有沒有提供；寫入動詞（非 dry-run）在執行前檢查，沒有提供時以語義化的訊息拒絕，分類是 `refused`，儲存不變。讀取與 dry-run 不需要這個方法。

`index.d.ts` 的儲存型別補上這個選填方法。fs 形式的引擎不受影響。

### D7: 示範用 server 的整份寫入改經命令

`put_config` 路由的步驟：

1. 角色檢查（既有的 `require_editor` 層，不動）。
2. 用引擎的 `WorkflowConfig::from_text` 與 `validate_policy_locales` 檢查請求的內容，失敗回傳 `invalid_config`。這一步留在路由，是為了維持「解析失敗先於版本衝突」的順序。
3. 用既有的 `run_write_with_if_match`，以流程設定這份文件與請求的預期版本，執行 `WorkflowConfigReplace`。版本不符回傳 `revision_conflict`。
4. 回應的形狀不變。

路由不再自己開 unit of work。第 2 步與命令裡的驗證呼叫的是同一組引擎函式，沒有第二份驗證邏輯。`config` 的讀取路由不動：它回傳原文與版本，不需要解析後的結果。

內容沒有變動的寫入：命令不寫入、不提交，版本不變。今天的路由在這種情況會提交並讓版本前進。這是可觀察的差異，見風險。

## Implementation Contract

**行為**

- CLI：`workflow-config` 的 11 個動詞，接受與拒絕、`--help`、人眼輸出、`--json`、`--dry-run`、exit code 與變更前逐位元相同；本機模式下寫出的 `config.yaml` 內容與變更前逐位元相同。
- 引擎：三種命令能在任何實作了寫入方法的儲存上執行。寫入的結果列出變動的鍵。
- SDK：`dispatch(['workflow-config', …])` 能用；沒有 `writeWorkflowConfig` 的宿主儲存，寫入動詞被拒絕，讀取照常。
- server：整份寫入的回應、錯誤分類與順序不變。

**介面**

- `speclink_core::command::Command::{WorkflowConfigShow, WorkflowConfigEdit, WorkflowConfigReplace}` 與對應的結果型別。
- `speclink_core::config::{WorkflowConfigEdit, changed_keys}`。
- `speclink_core::store::Store::write_workflow_config`。
- JS：宿主儲存的選填方法 `writeWorkflowConfig(content)`。

**失敗的樣子**

- 現有內容或新內容無法解析、locale 不合法：`invalid_config`，不寫入。
- 儲存不支援寫入：命令回傳錯誤，分類 `refused`（SDK）或一般錯誤（引擎直接呼叫時）。
- 經 Host 的儲存視圖時別人先改：提交回報衝突；server 回 409。
- `--dry-run`：不寫入，結果仍帶 `before`、`after`、`changed_keys`。

**驗收**

- `cargo test -p speclink-core`：三種命令的單元測試（每種編輯、驗證失敗、沒有變動、dry-run、檔案不存在）；`changed_keys` 的表格測試；列舉全部命令變體的既有測試更新後通過。
- `cargo test -p speclink-fs`、`cargo test -p speclink-host`：新方法的測試。
- `cargo test -p speclink-verbs`、`cargo test -p speclink-cli`（含 `workflow_config` 的說明文字測試與 `remote_verb_parity` 的 workflow-config 案例）。
- 搬移前後這個家族的 `--help` 與輸出逐位元相同。
- `cargo test -p speclink-server`：流程設定寫入的既有測試不改斷言就通過，除了風險裡列出的那一項。
- Node：`workflow-config-verbs.spec.ts` 通過。
- 在 `speclink-cli` 的 `verbs/config.rs` 與 `speclink-server` 的 `put_config` 裡搜尋不到直接寫流程設定檔或直接開 unit of work 的程式。

**範圍**

- 範圍內：引擎的三種命令與儲存方法；`speclink-verbs` 的第十二個家族；CLI 本機分支；server 的寫入路由；SDK 的選填方法。
- 範圍外：線上協定、CLI 遠端分支、桌面 app、領域事件、掛鉤、使用者文件。

## Risks / Trade-offs

- [回歸對照] CLI 的輸出與寫出的檔案內容是契約 → 搬移前後逐位元比對這個家族的說明文字與四種形式的輸出；另外對每種編輯比對寫出的 `config.yaml`。`workflow_config.rs` 的既有 CLI 測試與 `remote_verb_parity` 的案例照跑。引擎的 golden 不受影響。
- [跨平台] 寫入經儲存介面，換行沿用改寫函式既有的行為（保留原檔的換行）；Windows 的 CI 跑同一批測試。
- [沒有變動的整份寫入不再讓版本前進] 今天示範用的 server 對內容相同的寫入也提交 → 改成不提交後，回應裡的版本等於寫入前的版本。實作時先查 server 與用戶端的測試有沒有依賴「每次寫入版本都變」；有的話回報使用者決定維持哪一種，不自行改斷言。
- [驗證在路由與命令各呼叫一次] 是同一組函式，成本是多解析一次 YAML → 接受，換來錯誤順序不變。
- [儲存介面多一個方法] 沒有明確實作的儲存會在執行時才被發現不支援 → 預設實作的錯誤訊息指出儲存的種類；四個明確的實作各有測試。
- [SDK 新開放寫入政策的動詞] 宿主在提供執行前檢查之前，任何能呼叫 dispatch 的人都能改政策 → 列在相容性影響；宿主沒有提供 `writeWorkflowConfig` 時寫入一律被拒絕，這是預設的保護。
- [changed_keys 的粒度] `subagent_model` 只到第二層，分不出是哪個工具 → 目前的用途是政策鍵的權限，第二層足夠；需要更細時再加，不破壞既有的值。

## Open Questions

- 整份寫入在內容相同時要不要讓版本前進（見風險），等查完既有測試。
- 宿主的執行前檢查要拿 `changed_keys` 還是整個 `edit`，由提供掛鉤的變更決定；這個變更兩者都給得出來。
