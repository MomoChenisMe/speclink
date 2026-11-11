## Why

技能是給 AI agent 讀的步驟說明。規格文件不在 checkout 裡時（連到遠端的專案，或內嵌引擎的宿主），技能裡任何「去讀某個路徑」的步驟都會落空。動詞契約已經規定技能要用動詞讀文件，守門測試也在，但它只認得少數幾種寫法，漏掉的還在。

這個變更處理最先被用到的三支技能：discuss、propose、ingest。非工程角色在宿主的 Agent 裡用的就是這三支。逐行盤點的結果：

- **discuss 讀正式規格用的是路徑。** 它叫 agent 從規格清單的 `path` 欄位找到資料夾，再讀裡面的 `spec.md`。在遠端，`path` 是儲存的鍵，不是檔案。
- **有幾個步驟沒說用什麼讀或寫。** 例如「讀這個變更的 artifacts」「讀其他變更提案的 Impact」「直接修正 artifact」「把新內容併進既有的提案」。agent 在本機會直接開檔與改檔；沒有檔案時就做不下去。
- **spec 的語言設定叫 agent 去看設定檔。** 兩支技能寫的是「專案在 `config.yaml` 設了 `spec_locale` 時」，沒有指出用哪個動詞看。
- **一個復原步驟是搬檔案。** 最後一刀的旗標下錯時，技能叫 agent 把討論記錄從封存資料夾搬回來。
- **`speclink change rank` 在遠端被拒絕。** propose 與 ingest 的收尾步驟會用它調整執行順序，而它今天只能在本機執行；宿主的 Agent 工具也用不了它。

- **目標使用者**：在宿主的 AI Agent 裡做討論與提案的 PO、PM、顧問；連到遠端專案的開發者。
- **使用情境**：`/speclink-discuss`、`/speclink-propose`、`/speclink-ingest` 三支技能，以及它們收尾時用到的 `speclink change rank`。
- **既有規格的對照**：verb-contract 的「store 文件讀取動詞」規定技能用動詞讀文件；「模式分岔的單點宣告」把 `change rank` 列為只限本機。change-plan 的「change rank 動詞寫入看板順序鍵」規定它在遠端拒絕。node-sdk 的 dispatch 把 `change rank` 列為需要 checkout 的動詞。discuss-skill、propose-skill、ingest-skill 沒有規定「讀寫一律經動詞」。remote-board-order 規定遠端的順序存在一份看板順序文件。

Source discussion: embedded-host-needs（cut 9）。

## What Changes

**三支技能的文字**

- discuss 讀正式規格的 Purpose 改用 `speclink show <capability>`，並寫明清單回傳的 `path` 不是可以開啟的檔案。
- 沒說方法的步驟補上動詞：
  - 讀 artifact：`speclink artifact cat <artifact> --change <name>`。
  - 讀討論記錄：`speclink discuss show <slug> --json` 的 `content`。
  - 改 artifact：先用 `artifact cat` 讀出全文，改好後用 `speclink new artifact <id> --change <name> --stdin --force` 整份寫回。ingest 另外寫明：既有的任務行（勾選狀態與行尾的識別註解）原樣保留。
  - 讀其他變更的提案：`speclink artifact cat proposal --change <other>`；清單裡那個變更帶 `worktree` 時，從它的 `worktree.path` 執行。
- spec 的語言設定改成指向 `speclink workflow-config show --json` 的 `specLocale`。
- 指示 JSON 的 `outputPath` 改成說明用途：它是引擎存放的位置，僅供顯示；寫入一律用 `speclink new artifact`。
- 「把討論記錄搬回來」的復原步驟加上條件：規格資料夾在這個 checkout 裡才照做；不在時告訴使用者這一步要在持有規格的 Server 上處理。
- 影響的技能：discuss、propose、ingest 三支，全部的工具（claude、codex、copilot、cursor、pi 與自訂工具的兩種呼叫形式）都要重新產生技能檔。技能資產的版號前進一版。

**`speclink change rank` 在三種形態都可用**

- `speclink-core` 新增引擎命令 `ChangeRank`（名稱、錨點、前或後、是否強制），以及領域事件 `change-ranked`。欄序、守門、整欄補上順序鍵、取中點、依賴檢查沿用既有的同一份實作；只有「順序鍵從哪裡讀、寫到哪裡」由儲存決定。
- 引擎內層的儲存介面新增兩個方法：讀出全部變更的順序鍵、寫入一批順序鍵。預設實作是今天的做法（各變更的 metadata）。
- `speclink-host` 的儲存視圖覆寫這兩個方法：讀寫範圍的看板順序文件（鍵 `board/order`）。一次命令的全部順序鍵收在同一次提交。
- 線上新增一條路由：對某個變更送出排序請求（錨點、前或後、是否強制）。`speclink-protocol` 新增請求與回應的型別，`speclink-remote` 新增對應的呼叫，示範用 server 新增處理它的路由。
- CLI：`speclink change rank` 在遠端專案改成送出這個請求。子指令、旗標、stdout、`--json` 的形狀與 exit code 和本機相同。本機的行為不變（含 worktree 副本的寫入路由）。
- Node SDK 的 `dispatch` 不再把 `change rank` 當成需要 checkout 的動詞。
- **相容性影響**：
  - 本機專案：`change rank` 的輸出與寫入逐位元不變。
  - 遠端專案：`change rank` 從「非 0 並說明只限本機」變成「執行並成功」。訊息 `change rank is not available in remote mode` 不再出現。
  - 連到舊版 Server 的新版 CLI：Server 沒有這條路由，回 404；CLI 顯示既有的「找不到」訊息並以非 0 結束。
  - 技能檔：三支技能的內容改變，使用者要執行 `speclink update`。
- **設定欄位**：沒有新增或變更。

## Non-Goals

- **其餘技能**（archive、commit、worktree 開工前與收尾、apply、improve）與守門的全面加嚴：在變更 skills-verbs-second-wave 做。這個變更只對這三支技能加上對應的檢查。
- **`plan --strict-overlap` 與 `trace` 在遠端可用**：同上。
- **把封存的討論記錄搬回來的動詞**：這次不做，寫成已知缺口。它只在最後一刀的旗標下錯時才需要。
- **寫入詞彙表的動詞**：discuss 把詞彙的結論指向詞彙表，但沒有寫入詞彙表的動詞。這次不做，寫成已知缺口。
- **Speclink Desktop 遠端拖排改用新的命令**：Desktop 今天自己組出整份看板順序文件再寫回。兩條路寫的是同一份文件、都帶版本比對，可以並存；改成共用留給之後。
- **看板上討論卡的順序**：`change rank` 只排變更。
- **依「規格在不在 checkout」產生不同的技能文字**：技能的渲染沒有條件語法；這次需要分情況的只有一處，用一句文字條件表達。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 修改「store 文件讀取動詞」（技能不得用清單回傳的路徑讀檔、改寫也經動詞）與「模式分岔的單點宣告」（`change rank` 不再只限本機）。
- `change-plan`: 修改「change rank 動詞寫入看板順序鍵」（遠端不再拒絕，寫入看板順序文件）。
- `node-sdk`: 修改「dispatch 的輸入輸出契約」（`change rank` 移出需要 checkout 的清單）。
- `client-protocol`: 新增「排序請求與回應」。
- `discuss-skill`: 新增「正式規格與變更文件一律經動詞讀取」。
- `propose-skill`: 新增「artifact 的讀取與改寫一律經動詞」。
- `ingest-skill`: 新增「既有 artifact 的更新經動詞整份寫回」。

## Impact

- Affected specs: verb-contract（修改兩條）、change-plan（修改一條）、node-sdk（修改一條）、client-protocol（新增一條）、discuss-skill、propose-skill、ingest-skill（各新增一條）
- Affected crates: `speclink-core`、`speclink-fs`（不改，沿用預設實作）、`speclink-host`、`speclink-protocol`、`speclink-remote`、`speclink-server`、`speclink-verbs`、`speclink-cli`、`speclink-node`。桌面 app 不改。
- Affected code:
  - New:
    - crates/engine/speclink-core/src/command/change_rank.rs
    - crates/adapters/speclink-cli/tests/it/remote_change_rank.rs
    - crates/host/speclink-host/tests/port_rank.rs
  - Modified:
    - crates/engine/speclink-core/assets/skills/discuss.md
    - crates/engine/speclink-core/assets/skills/propose.md
    - crates/engine/speclink-core/assets/skills/ingest.md
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/tests/it/skill_verbization.rs
    - crates/engine/speclink-core/tests/it/skill_authoring.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/store.rs
    - crates/engine/speclink-core/src/lifecycle/plan.rs
    - crates/host/speclink-host/src/port_view.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/protocol/speclink-protocol/src/command.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/host/speclink-server/src/app.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/src/api/events.rs
    - crates/adapters/speclink-verbs/src/families/plan.rs
    - crates/adapters/speclink-verbs/src/render/plan.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/verbs/plan.rs
    - crates/adapters/speclink-cli/tests/it/plan_verbs.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - crates/adapters/speclink-node/src/lib.rs
  - Removed: (none)
  - 另外會重新產生：crates/engine/speclink-core/tests/golden 底下的技能快照，以及本 repo 由 `speclink update` 產生的技能檔。
- 相依：沒有硬性的先後。可以和 host-kit-agent-door 並行。排序的線上路由這次加在示範用 server；變更 host-kit-client-door 把路由搬進宿主套件時一起搬。
- 三種形態下的結果：
  - 本機 repo：三支技能可用，行為相同（讀寫改經動詞，動詞在本機讀寫同一批檔案）。`change rank` 可用，行為不變。
  - 連到遠端的 checkout：三支技能可用。`change rank` 可用，順序寫進 Server 的看板順序文件。已知缺口：把封存的討論記錄搬回來要在 Server 上處理；詞彙表沒有寫入的動詞。
  - 沒有本機規格檔的內嵌宿主：三支技能可用（tool-call 形式）。`change rank` 經 Node SDK 的 dispatch 可用。已知缺口同上；另外「讀使用者給的文件路徑」這種輸入只在 agent 有檔案系統時可用。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - 動詞與旗標的說明：`change rank` 在遠端專案可用。
  - 專案能力狀態：把兩個已知缺口列進去（搬回封存的討論記錄、寫入詞彙表）。
