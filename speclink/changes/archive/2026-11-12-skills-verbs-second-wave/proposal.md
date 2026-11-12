## Why

第一波把 discuss、propose、ingest 三支技能改成全部經動詞讀寫規格文件。其餘的技能還沒有。逐行盤點其餘技能的原稿，有約 90 處直接操作規格資料夾底下的檔案，分成四類：

- **用路徑讀**：archive 與 commit 用路徑檢查 delta spec、和正式規格比對；commit、worktree 開工前、improve 直接讀 `.evidence.json`；verify 用路徑找 delta 與搜尋正式規格；trace 從封存資料夾開提案。
- **用路徑改**：archive 與 commit「就地改寫 delta 檔」；apply 與品質關卡「直接修正 artifact」，沒有說用什麼改。
- **對規格檔做 git 操作**：worktree 開工前先把變更的 artifact commit 進去；commit 把規格資料夾底下的變動一起 stage；apply 與 drift 用 `git log` 看規格資料夾多久沒動。
- **顯示規格資料夾的路徑**：例如封存後印出的「Archived to」。

連到遠端專案的 checkout 裡沒有規格資料夾，這些步驟全部落空。既有的守門測試只認得「read、glob、open 後面接路徑」一種寫法，所以它們都沒被擋下。

另外有一個渲染的問題：給自訂工具的技能（中性渲染），把「交棒給另一支技能」寫成 `speclink apply <name>`。同一份文字的開頭又告訴 agent「`speclink <動詞>` 就是呼叫工具」。agent 會把交棒當成動詞送出去：`apply` 不是動詞所以失敗，而 `archive`、`review`、`verify` 剛好是真的動詞，會直接被執行。

上一個變更 remote-read-verbs 已經補齊這些步驟需要的動詞。這個變更改技能的文字、渲染與守門。

- **目標使用者**：連到遠端專案的開發者（apply、review、verify、commit、archive 都在他們的電腦上跑）；在宿主的 AI Agent 裡工作的 PO、PM（archive、improve、trace、analyze）。
- **使用情境**：discuss、propose、ingest 以外的每一支技能。
- **既有規格的對照**：verb-contract「store 文件讀取動詞」只對第一波的三支技能規定「改寫經動詞」。skill-authoring 規定原稿的寫法與機械守門，沒有條件區塊。workspace-tools「中性渲染目標」規定中性渲染不含斜線前綴，沒有規定交棒要和動詞分得開。archive-skill、commit-skill、worktree-apply-skill 沒有規定讀寫的方式。node-sdk「渲染 API」提到一個 `store` 選項，實作裡沒有。

Source discussion: embedded-host-needs（cut 12）。

## What Changes

**渲染多一個軸：規格在不在這個 checkout 裡**

- 技能的原稿可以把一段文字標成「規格資料夾在這個 checkout 時才產生」，或「不在時才產生」。渲染時依這個軸留下其中一種。
- 本機專案產生的技能和今天相同（軸是「在」）。遠端專案（`.speclink.yaml` 有 remote 區段）用 `speclink init --store remote` 與 `speclink update` 產生的技能，軸是「不在」。
- 原稿裡每一處規格資料夾的路徑，以及 `.evidence.json`、`.meta.yaml` 這兩個檔名，都必須在「在」的區塊裡。所以遠端專案的技能檔完全不出現規格資料夾的路徑。
- 對規格檔的 git 步驟、顯示路徑的句子都放進「在」的區塊。這是討論的裁定：git 步驟在產生技能時決定，不在技能裡做執行期的判斷。

**技能文字改經動詞**（apply、worktree 開工前與收尾、archive、commit、improve、review、verify、quality、analyze、drift、trace，以及品質關卡共用的段落）

- 讀 artifact：`speclink artifact cat`；已封存變更的文件也用它。
- 讀 evidence：`speclink evidence show <change> --json` 的 `files` 與 `entries`。
- 判斷有沒有 delta、有哪些 capability：`speclink list --json` 的 `deltaCapabilities`。
- 讀正式規格：`speclink show <capability> --item-type spec`；找相關的規格用 `speclink list --specs --json`。
- 任務進度：`speclink list --json` 的任務數欄位。
- 改既有的 artifact：第一波的「讀出全文、整份寫回」那一段共用文字。
- 變更閒置多久：`speclink list --json` 的 `created` 與 `modifiedAt`。沒有 `modifiedAt` 時只看建立時間，並說明無法判斷最近有沒有動。
- 第一波三支技能裡殘留的路徑字樣（記錄檔的位置、詞彙表的路徑、把封存記錄搬回來的步驟）改用同一套區塊。

**交棒和動詞分得開**

- 中性渲染（自訂工具的兩種呼叫形式）把「另一支技能」寫成 `speclink-<名稱>`（技能的名稱），不再寫成 `speclink <名稱>`。
- 開頭的呼叫說明多一句：`speclink-<名稱>` 是另一支技能，不是動詞，不要當成工具的參數送出。tool-call 說明裡的 argv 範例原本是 `["apply", "add-auth"]`（`apply` 不是動詞），改成真的動詞 `["list", "--json"]`。
- 內建工具（claude、codex、copilot、pi、cursor）的寫法不變。

**守門**

- 原稿層：規格資料夾的路徑與兩個檔名不得出現在「在」的區塊之外。
- 渲染層：以「不在」渲染全部技能，結果不得含規格資料夾的路徑；中性渲染的結果不得把任何技能名稱寫成 `speclink <名稱>` 的形式。
- 兩支技能明文豁免，理由寫在測試裡：baseline（直接寫正式規格）與 manual（寫手冊頁）。實作時的稽核發現「兩者在遠端專案本來就先拒絕」不成立：baseline 的原稿沒有拒絕的步驟；manual 的拒絕靠執行時讀 `.speclink.yaml`，內嵌的宿主沒有這個檔。所以這個變更替兩支各加一個「不在」的區塊：以「不在」渲染時，開頭就說明不支援並結束，不寫任何檔案。本機專案的內容不變。
- 原稿層另外擋兩種寫法：把規格資料夾的名稱寫死（不經佔位符），以及拼錯的區塊記號。渲染層的「在」規則以段落為單位（到下一個空白行為止），並涵蓋字尾變化，路徑換到下一行或把 read 寫成 reads 都逃不掉。

**各入口**

- `speclink-core`：渲染的軸、區塊的語法、產生技能時依專案是不是遠端決定軸；原稿；守門；資產版號前進一版；golden 多兩份「不在」的快照。
- `speclink-node`：`skills.render` 的選項新增 `specInCheckout`（預設「在」）。內嵌的宿主渲染給自己 Agent 的技能時傳「不在」。
- **CLI 指令**：沒有新增或變更旗標。`speclink init --store remote` 與 `speclink update` 在遠端專案產生的技能內容改變。
- **相容性影響**：
  - 本機專案：技能檔的內容改變（改經動詞），要執行 `speclink update`。步驟的效果相同。
  - 遠端專案：技能檔的內容改變更多（沒有規格路徑與規格檔的 git 步驟），要執行 `speclink update`。
  - 自訂工具：交棒的寫法從 `speclink <名稱>` 變成 `speclink-<名稱>`。
  - 內建工具的 golden 快照整批更新（刻意的變更）。
  - `skills.render` 沒有傳新選項時輸出等於本機專案的內容。
- **設定欄位**：沒有新增或變更。
- **影響的技能與工具**：discuss、propose 小幅調整（ingest 不用改）；其餘全部調整。全部工具都要重新產生技能檔。

## Non-Goals

- **遠端專案產生 worktree 的兩支技能**：產生哪些技能要跟著 Server 上的流程設定，在變更 remote-skill-delivery 做。這個變更只改這兩支技能的文字。
- **baseline 與 manual 在遠端可用**：不做。這個變更只補上「不在」時的拒絕；讓它們經動詞寫入正式規格與手冊頁是之後的事，寫成已知缺口。
- **內部技能（sync、clarify、tdd）**：它們不產生技能檔，只經 `instructions --skill` 取用。這次不改。
- **新的動詞**：都在上一個變更。
- **技能在沒有程式碼 checkout 的地方做 apply、review、verify、commit**：這幾支本來就需要程式碼。
- **把封存的討論記錄搬回來、寫入詞彙表的動詞**：維持已知缺口。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `skill-authoring`: 新增「原稿的條件區塊」；修改「寫法規範的機械守門」（新增第六項：規格路徑只能在區塊裡）；修改「技能原稿的跨模型寫法規範」（情境與範例表裡自訂描述子的引用改成 `speclink-ingest`）。
- `verb-contract`: 修改「store 文件讀取動詞」（規則從三支技能擴大到全部產生的技能，並釘住「不在」的渲染沒有規格路徑）。
- `workspace-tools`: 新增「技能依規格是否在 checkout 渲染」；修改「中性渲染目標」（交棒寫成技能名稱）。
- `node-sdk`: 修改「渲染 API」（新增 `specInCheckout`，拿掉沒有實作的 `store` 與 `instructions.render`）。
- `archive-skill`: 新增「封存前的讀取與 delta 改寫經動詞」。
- `commit-skill`: 新增「commit 的檔案清單取自 evidence 動詞」。
- `worktree-apply-skill`: 新增「worktree 技能的規格檔步驟依 checkout 產生」。
- `baseline-skill`: 新增「規格不在 checkout 時先拒絕」。
- `manual-skill`: 修改「remote 模式的限制敘述」（以「不在」渲染時在技能檔裡直接拒絕，不靠執行時讀 `.speclink.yaml`）。

## Impact

- Affected specs: skill-authoring（新增一條、修改兩條）、verb-contract（修改一條）、workspace-tools（新增一條、修改一條）、node-sdk（修改一條）、archive-skill、commit-skill、worktree-apply-skill、baseline-skill（各新增一條）、manual-skill（修改一條）
- Affected crates: `speclink-core`、`speclink-node`。`speclink-cli` 不用改（它的測試沒有釘住被改寫的句子）。其他 crate 與桌面 app 不改。
- Affected code:
  - New:
    - crates/engine/speclink-core/tests/golden/claude-remote.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call-remote.snapshot.md
  - Modified:
    - crates/engine/speclink-core/src/workspace/skills.rs
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/assets/skills/apply.md
    - crates/engine/speclink-core/assets/skills/apply-worktree-pre.md
    - crates/engine/speclink-core/assets/skills/apply-worktree-post.md
    - crates/engine/speclink-core/assets/skills/archive.md
    - crates/engine/speclink-core/assets/skills/commit.md
    - crates/engine/speclink-core/assets/skills/improve.md
    - crates/engine/speclink-core/assets/skills/review.md
    - crates/engine/speclink-core/assets/skills/verify.md
    - crates/engine/speclink-core/assets/skills/quality.md
    - crates/engine/speclink-core/assets/skills/station-shared.md
    - crates/engine/speclink-core/assets/skills/analyze.md
    - crates/engine/speclink-core/assets/skills/drift.md
    - crates/engine/speclink-core/assets/skills/trace.md
    - crates/engine/speclink-core/assets/skills/discuss.md
    - crates/engine/speclink-core/assets/skills/propose.md
    - crates/engine/speclink-core/assets/skills/baseline.md
    - crates/engine/speclink-core/assets/skills/manual.md
    - crates/engine/speclink-core/tests/it/skill_verbization.rs
    - crates/engine/speclink-core/tests/it/main.rs
    - crates/engine/speclink-core/tests/it/skill_authoring.rs
    - crates/engine/speclink-core/tests/it/render_golden.rs
    - crates/engine/speclink-core/src/workspace/init/tests.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/adapters/speclink-node/src/render.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/render.spec.ts
  - Removed: (none)
  - 另外會重新產生：crates/engine/speclink-core/tests/golden 底下既有的技能快照，以及本 repo 由 `speclink update` 產生的技能檔。
  - 實作後的更正：`config.md` 與 `ingest.md` 的原稿不用改（它們沒有規格資料夾的佔位符；`config.md` 的結論在 design 的 Open Questions）。`baseline.md` 與 `manual.md` 是稽核後加入的。
- 相依：排在變更 remote-read-verbs 之後（技能要用它新增的動詞與欄位）。
- 三種形態下的結果：
  - 本機 repo：全部技能可用，步驟的效果相同。
  - 連到遠端的 checkout：apply、review、verify、quality、commit、archive、improve、analyze、drift、trace 與 worktree 收尾可用。已知缺口：worktree 的兩支技能在遠端專案還不會被產生（等 remote-skill-delivery）；baseline 與 manual 維持拒絕；連到示範用 server 時沒有 `modifiedAt`，閒置的判斷只看建立時間；用 CLI 執行 `speclink link` 或 `unlink` 之後技能檔不會自動重新產生，探測也不提示，要自己執行 `speclink update`（交給 remote-skill-delivery）。
  - 沒有本機規格檔的內嵌宿主：archive、improve、analyze、trace 可用（tool-call 形式，以「不在」渲染）。apply、review、verify、quality、commit 與 worktree 的技能不適用：它們需要程式碼的 checkout。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - Node SDK 文件：`skills.render` 的 `specInCheckout`。
  - 工作流程文件：遠端專案的 commit 只含程式碼的檔案；規格的變動不在 checkout 裡。
  - 專案能力狀態：baseline 與 manual 在遠端的缺口。
