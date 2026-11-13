## Context

- **metadata 裡的關卡資料**：蓋章寫入一組欄位，審查是 `reviewed_at`、`reviewed_by`、`reviewed_with`、`reviewed_tasks_total`、`reviewed_rounds`、`reviewed_scope`（每個檔案的路徑與內容指紋），驗證是同構的 `verified_*`。蓋章同時刪除工單文件（`review.md`、`verify.md`）。所以三種狀態可以這樣分辨：有工單 → 進行中；有完整的章 → 已蓋章；都沒有 → 未開始。
- **「蓋章後有沒有變」**：比兩個錨——任務數有沒有變、scope 裡每個檔案現在的內容指紋和記錄的是否相同。第二個要讀工作樹的檔案。沒有工作樹時引擎只比第一個。
- **線上**：`ChangeSummary`、`ChangeStatus`、`ArchivedItem` 沒有任何關卡欄位。清單的路由執行 `List` 命令後，由 Host 讀各變更的 metadata 補上幾個欄位（建立者等）；單筆的路由執行 `Status` 後同樣補欄位。
- **Speclink Desktop**：本機專案由 `speclink-desktop-core` 讀儲存與工作樹算出四態（未開始、進行中、已蓋章、已蓋章但其後有變）。遠端專案的清單項沒有這些欄位，封存時兩個「照樣帶走」的旗標固定是假。
- **事件**：示範用 server 的串流用一個分類函式把事件名稱對到資源類別。六種審查與驗證的事件沒有列在裡面，落到 unknown。Speclink Desktop 收到任何事件都整個重新整理，不看類別。
- **commit 與分支**：commit 只出現在逐任務的 evidence 裡（`task done` 帶的 `head_commit`）。蓋章的命令與請求沒有 commit。metadata 沒有分支，也沒有落地的記錄。worktree 的分支名稱是即時從 git 讀的。
- `speclink list --json` 被兩個關卡的規格釘住：不得因為 metadata 有章而多出欄位。
- 討論留下的未驗證項：舊版用戶端遇到新欄位是否照常運作。

限制：

- 引擎不讀 checkout、不呼叫 git；本機事實由呼叫端收集後當成輸入。
- 引擎只存，不附帶規則。
- 文字手術改 metadata 時，純量要經既有的跳脫函式。

## Goals / Non-Goals

**Goals:**

- 任何用戶端與宿主都能從線上讀到每個變更兩個關卡的狀態。
- 審查與驗證的事件在線上歸到變更。
- 蓋章記下 commit；變更記下分支與落地的 commit。
- Speclink Desktop 的遠端專案有和本機相同的關卡標示。

**Non-Goals:**

- 任何規則、Server 端的 git 檢查、`list --json` 的欄位、自動偵測落地。

## Decisions

### D1: 線上的關卡物件

`speclink-protocol` 新增：

```rust
pub struct StationState { pub state: StationPhase, pub stamped_at: Option<String>, pub stamped_by: Option<String>, pub stamped_with: Option<String>, pub rounds: Option<u64>, pub tasks_total: Option<usize>, pub commit: Option<String>, pub scope: Vec<ScopeEntry> }
pub enum StationPhase { None, InProgress, Stamped }   // "none" | "inProgress" | "stamped"，另有承接未知值的變體
```

- `ChangeSummary` 與 `ChangeStatus` 各新增選填的 `review` 與 `verify`（缺席時不序列化）。
- `scope` 只在單筆回應填入；清單裡是空的且不序列化。一個變更的 scope 常有幾十筆，清單不該帶。
- 判定（Host 的一個函式，清單與單筆共用）：
  1. 該關卡的工單文件存在 → `inProgress`。
  2. 否則 metadata 有完整的章（引擎既有的「已蓋章」判斷） → `stamped`，其餘欄位取自 metadata。
  3. 否則 → `none`。
- metadata 壞掉的變更沒有這兩個物件（和既有補欄位的做法相同）。
- `ArchivedItem` 新增選填的 `reviewOutcome` 與 `verifyOutcome`，順序和上面的判定相同：封存的文件集裡有工單 → `notPassed`；否則有章 → `stamped`；否則 `none`。工單與章並存出現在「蓋章後重開工單，再用 `--carry-review`（或 `--carry-verify`）封存」：新增一輪不清掉舊章，封存的章失效守門也把這種舊章當成已被工單取代，所以結局是未通過。
- 結局的順序只有一份：引擎的 `station` 模組提供「有沒有工單、有沒有章 → 結局」的純函式（`ArchivedOutcome::of`）。「有沒有章」這個事實由兩端各自提供：Host 用引擎的「完整的章」（時間、任務數、scope 都有），本機快取沿用它既有的判斷（有蓋章時間就算），所以舊封存在本機的標示不變。Host 的已封存清單與 desktop 本機的已封存快取（`apps/desktop/core/src/cache.rs`）都呼叫它，各自只負責把結局轉成自己的字串（線上的 `StationOutcome`、卡片的 `reviewStatus`／`verifyStatus`）。本機快取原本是「有章優先」，這次一起改；快取版本加一，讓已經存下的結局重算（舊版本的快取本來就會被丟掉重建）。

讀取的成本：清單要知道每個變更有沒有工單。metadata 本來就要讀一次；兩個關卡的工單文件併進同一次讀取（一次讀取帶多個鍵），讀回來有那份文件就是有工單，不多發呼叫。代價是清單會把未結工單的內文一起讀回來——未結的工單通常很少（蓋章就刪掉了）；量大時再改成只取鍵。用成本測試釘住呼叫數不變。

為什麼三態而不是 Desktop 的四態：第四態（蓋章後有變）只有有工作樹的一端算得出來。線上給事實，用戶端自己加上第四態。

- **否決：把 scope 放進清單** — 每個已蓋章的變更多幾 KB。
- **否決：Server 用任務數算出「可能過期」** — 兩個錨拆開回報會讓各用戶端的判定不一致；交給用戶端用同一個引擎函式算。

### D2: 事件的分類

示範用 server 的分類函式把 `review-round-added`、`review-stamped`、`review-discarded`、`verify-round-added`、`verify-stamped`、`verify-discarded` 與新增的 `change-landed` 歸到「變更」，資源識別取 payload 的 `change`。其他對映不了的事件維持 unknown。

分類函式今天在示範用 server 裡。把它搬到 `speclink-host`（放在事件 payload 的函式旁邊）並公開，讓自己提供推送的宿主用同一份對照；示範用 server 改呼叫它。

### D3: 蓋章帶 commit

- `Command::ReviewStamp` 與 `Command::VerifyStamp` 新增 `head_commit: Option<String>`。
- 蓋章寫 metadata 時，有值就多寫一個鍵：`reviewed_commit` 或 `verified_commit`（經跳脫函式）。沒有值就不寫。它和其他章欄位在同一次寫入裡，不改變「一次寫入、同時刪工單」的原子性。
- 重新蓋章時（放棄後再蓋，或 schema 變更清掉驗證的章）這個鍵和其他章欄位同進退：清章的地方一起清。
- 值的檢查只有形狀：去掉前後空白後是 7 到 64 個十六進位字元，否則以 `invalid_argv` 拒絕。引擎不查它存不存在。
- 線上：`StampReviewRequest` 新增選填的 `headCommit`。
- CLI：本機與遠端的蓋章都把 checkout 的 HEAD 帶上，用引擎既有讀 HEAD 的輔助函式（在 CLI 這一邊呼叫）。讀不到（不是 git checkout）就不帶。
- `VerbInputs` 既有的 `head_commit` 欄位用於蓋章的轉換。Node 的 `dispatch`：蓋章的 stdin JSON 新增選填的 `headCommit`。

### D4: 分支

- `Command::InProgressAdd` 新增 `branch: Option<String>`。開工標記寫入 `started_*` 的同一次文字手術裡，有值就寫 `branch`。已經有開工標記時（命令本來就不重複蓋）不改 `branch`。寫入前確認改完的文字讀得回同一個分支名稱，讀不回就拒絕、零寫入：形狀檢查擋不掉 YAML 帶不了的字元（例如 U+FFFE），那種名稱寫進去整份 metadata 就再也解析不了。
- 退回提案中（`in-progress remove`）時一併移除 `branch`。
- 形狀的檢查：1 到 255 個字元、沒有控制字元與空白；否則 `invalid_argv`。
- CLI：`in-progress add` 把 checkout 目前的分支名稱帶上（detached HEAD 或不是 git checkout 時不帶）。讀分支的小函式放在 CLI 既有收集本機事實的地方；`VerbInputs` 新增 `branch`。
- 線上：開工標記的請求新增選填的 `branch`。

### D5: 落地的命令與動詞

`speclink-core` 新增 `command/landing.rs`：

- `Command::ChangeLanded { name, commit, branch: Option<String> }`，名稱 `change-landed`。
- 效果：對 metadata 做一次文字手術，寫 `landed_commit`、`landed_at`（執行當下的時間，RFC 3339）、`landed_by`（操作者；匿名時不寫），有給 `branch` 時覆寫 `branch`。重複執行會覆寫。只有一次寫入，沒有半套狀態。
- 守門：變更必須存在且 metadata 可解析（`not_found`、`invalid_config`）；`commit` 的形狀同 D3。不看變更的階段：提案中的變更也能回報（程式碼可能先合併了）。
- 結果 `ChangeLandedOutcome { change, commit, branch }`；事件 `DomainEvent::ChangeLanded { change, commit, branch }`，種類名 `change-landed`。
- 封存時 metadata 整份搬走，落地的記錄跟著進封存。

`speclink-verbs`：`change landed <name> [--commit <sha>] [--branch <name>] [--json]`，放在既有的 `change` 家族。轉換時 `commit` 取旗標，沒有旗標時取 `VerbInputs.head_commit`；兩者都沒有 → `invalid_argv`，訊息說明要給 `--commit` 或在 git checkout 裡執行。`--branch` 沒有給時不帶分支（不自動取目前的分支：合併後的 checkout 多半在主分支上，那不是這個變更的分支）。

輸出：`✓ <name> landed at <commit 的前 12 個字元>`；`--json` 是 `{ "change", "commit", "branch"? }`，commit 是完整的值。

線上：`POST /changes/{name}/landed`，請求 `{ commit, branch? }`，回應同 `--json`。它是寫入。線上歸屬表新增這個命令。

### D6: 讀取分支與落地

- `ChangeSummary` 新增選填的 `branch`、`landedCommit`；`ChangeStatus` 新增 `branch`、`landedCommit`、`landedAt`、`landedBy`。來源是 Host 補欄位時讀的同一份 metadata。
- 引擎的 `ChangeMeta` 新增對應的欄位（都有預設值，既有的 metadata 照常可讀）。
- `speclink show <change> --json` 新增四個選填欄位：`review`、`verify`（形狀同 D1，不含 scope）、`branch`、`landed`（`{ commit, at, by? }`）。沒有資料的欄位不輸出。人眼輸出不變。本機由引擎的 show 結果帶出（引擎讀得到 metadata 與工單）；遠端由 CLI 從單筆回應組出，和 show 既有的做法相同。
  - 「沒有資料」對關卡的意思：沒有工單也沒有章。這時不輸出 `review`（或 `verify`）這個鍵，所以沒有任何記錄的變更，`show --json` 的輸出和加這四個欄位之前逐位元相同；`show --json` 裡因此只會看到 `inProgress` 與 `stamped` 兩種 state。這條規則寫在輸出函式一處，本機與遠端都經過它（遠端的單筆回應會明確給 `none`，同樣不輸出）。線上的清單與單筆回應不受影響，照 D1 給三態。
  - metadata 壞掉的變更、以及連到不回報這些欄位的舊版 Server 時，四個鍵同樣都不出現。
- `speclink list --json` 不動。

`show --json` 是唯一新增欄位的 CLI 輸出，理由是內嵌的宿主經 `host.dispatch` 要有一個讀得到關卡狀態的動詞，而 `list --json` 被釘住。

### D7: Speclink Desktop

- `speclink-desktop-core` 把「從關卡的事實算出四態」抽成一個純函式：輸入是線上的 `StationState`（或本機讀到的同樣事實）、目前的任務數、一個選填的讀檔函式；輸出是既有的四態。本機專案改呼叫它（行為不變），遠端專案也呼叫它：
  - 清單：`inProgress` → 審查中／驗證中；`stamped` 且任務數不同 → 其後有變；`stamped` 且任務數相同 → 已蓋章（有 checkout 時再做下一步）；`none` → 未開始。
  - 有 checkout 的遠端專案：對清單裡任務數相符的已蓋章變更，取單筆回應的 scope，用 checkout 的檔案比內容指紋，不符的改成其後有變。只有 checkout 握有蓋章的那一版時才比：checkout 的 HEAD 包含章記下的 commit（`git merge-base --is-ancestor`）。章沒有記下 commit、或 checkout 不包含它（章蓋在別的分支、worktree 或別台機器，checkout 還沒有那批程式碼）時，維持只比任務數的結果——否則剛蓋好的章會被誤報成其後有變。commit 是 Server 給的外部輸入，先過引擎的形狀檢查才交給 git。比對一個檔案時多看一步：內容和章的指紋不同的檔案，若還是章記下的 commit 當時的樣子（當時沒有、現在也沒有也算），不算其後有變。章記的是蓋章當下的 HEAD，被審的內容那時常常還沒提交（先蓋章、後提交）；別台機器的 checkout 有那個 commit、卻還沒有被審的內容，檔案和指紋當然不同，但它沒有在章之後被改過。「還是 commit 當時的樣子」用 git 的 blob 識別比（`git rev-parse <commit>:<path>` 對 `git hash-object`），只對指紋不符的檔案才呼叫 git。不做快取（量測與理由見 Open Questions 的結論）。那一次單筆讀取失敗時維持只比任務數的結果，清單不因此失敗。scope 的路徑是 Server 給的外部輸入：不是 repo 內相對路徑的（絕對路徑、`..`、磁碟代號）一律不讀，讀作內容有變。
  - 遠端的任務數只有總數（清單項沒有寫碼任務的分組），所以遠端的任務錨只比總數；本機另外還看寫碼任務是否全部完成。兩邊走同一個函式，差別只在餵進去的計數。
- 已封存頁用 `reviewOutcome` 與 `verifyOutcome`。
- 封存入口：遠端專案的變更是審查中或驗證中時，出現既有的三選項對話框；選「照樣帶走」時把對應的旗標帶進封存的請求（線上本來就支援）。對話框的另一個選項「放棄審查／放棄驗證」在本機是刪掉該站的工單；遠端原本沒有接這個動作，對話框一出現它就會失敗，所以一併接上（刪除工單的路由本來就有，桌面多一個 `remote_discard_station` 命令）。
- 桌面殼層把關卡的狀態算成和本機清單項同名的鍵（`reviewStatus`、`reviewedAt`、`reviewedBy` 與驗證站的同構三鍵；已封存項是 `reviewStatus`／`verifyStatus` 的結局字串），前端因此不用改元件，也不用認得線上的 `review`／`verify` 物件。checkout 的位置由前端的 remote 資料來源在清單請求裡帶上（`checkoutRoot`）。
- 元件不變：標示、對話框都是既有的；只把資料接上。

### D8: 技能

commit 與 worktree-merge 的原稿各加一步（都只在成功之後）：

- commit：`git commit` 成功後執行 `speclink change landed <name>`（用 HEAD）。在 worktree 的分支上提交時不執行——那還不是落地，由 worktree-merge 在合併後回報。
- worktree-merge：合併進主分支成功後執行 `speclink change landed <name> --branch speclink/<name>`（commit 用合併後的 HEAD）。
- 兩支技能都先用 git 取得 HEAD，再以 `--commit` 明確帶上，不靠動詞自己讀 HEAD。理由：技能以工具呼叫的形式產生時，`speclink` 動詞在宿主那一端執行，那裡沒有 checkout；不帶 `--commit` 就會被「沒有 commit 可回報」拒絕，而這正是最需要落地記錄的形態。在本機 CLI 帶不帶結果相同。
- commit 技能的封存子流程（先封存再一起提交）執行過時，略過回報：變更已經封存，`change landed` 只找作用中的變更，執行只會得到找不到。落地要在提交之後才有 commit 可報，封存又在提交之前，兩者排不進同一次提交。要求「落地後才能封存」的宿主本來就會在掛鉤擋下那次封存，流程自然變成提交 → 回報 → 之後再封存。
- 規格資料夾在 checkout 裡時，回報寫進變更自己的資料檔，所以回報之後那個檔有未提交的改動（提交的 commit 不可能包含記錄它自己 SHA 的檔案）。技能只說明這件事、不代為提交：commit 技能請使用者把它留給這個變更的下一次提交；worktree-merge 既有的守則是不替使用者提交，所以在交棒時點名這個檔。連到遠端的 checkout 沒有這個問題（資料在 Server）。
- 兩處都寫明：這個動詞被拒絕時（例如連到沒有這條路由的舊版 Server），把訊息告訴使用者並繼續，不回頭取消提交或合併。

資產版號前進一版，golden 與 lock 同批更新。兩份原稿的強調字上限不增加。

## Implementation Contract

**行為**

- 線上的清單與單筆回應：有工單的關卡是 `inProgress`；蓋章後是 `stamped` 並帶時間、人、工具、輪數、任務數與（有帶時）commit；單筆回應另有 scope。
- `speclink review stamp` 在 git checkout 裡執行後，metadata 有 `reviewed_commit`，值是當時的 HEAD；線上該變更的 `review.commit` 是同一個值。
- `speclink in-progress add` 在分支 `feature/x` 上執行後，metadata 有 `branch: feature/x`。
- `speclink change landed demo --commit <sha>` 之後：`show demo --json` 的 `landed.commit` 是那個值；儲存接口上是正好一次提交，帶一筆 `change-landed` 事件；執行前掛鉤收到命令名稱 `change-landed` 與參數裡的 commit。
- 示範用 server 的事件串流：蓋章之後訂閱者收到類別為變更、資源識別為該變更名稱的事件。
- Speclink Desktop 的遠端看板：審查中的變更卡片顯示審查中；封存它時出現三選項對話框。
- `speclink list --json` 的輸出與這個變更之前逐位元相同。

**介面與資料形狀**

- 引擎：`Command::ChangeLanded`、`ChangeLandedOutcome`、`DomainEvent::ChangeLanded`；`ReviewStamp`／`VerifyStamp` 的 `head_commit`；`InProgressAdd` 的 `branch`；`ChangeMeta` 的 `reviewed_commit`、`verified_commit`、`branch`、`landed_commit`、`landed_at`、`landed_by`。
- 線上：`StationState`、`StationPhase`；`ChangeSummary`／`ChangeStatus`／`ArchivedItem` 的新欄位；`StampReviewRequest.headCommit`；開工標記請求的 `branch`；`POST /changes/{name}/landed`。
- CLI：`speclink change landed`；`show --json` 的四個欄位。
- Host：公開的事件分類函式。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| `change landed` 的變更不存在 | `not_found` |
| 沒有 `--commit` 也讀不到 HEAD | `invalid_argv`，說明怎麼給 |
| commit 或分支的形狀不合 | `invalid_argv`，儲存不變 |
| 蓋章時讀不到 HEAD | 照常蓋章，不寫 commit |
| 舊版 Server 沒有落地的路由 | 404；CLI 非 0；技能繼續 |
| 舊版用戶端收到新欄位 | 照常解析，忽略它們 |
| metadata 壞掉的變更 | 清單項沒有關卡物件，其餘照既有行為 |

**驗收**

- `cargo test -p speclink-core`：蓋章寫入與清除 commit、開工的分支、落地的命令與守門；命令名稱與參數的 golden 同批更新；`render_golden` 中 `list --json` 不變。
- `cargo test -p speclink-host --test station_wire`：三態的判定、清單不帶 scope、已封存的結局（含工單與章並存 → `notPassed`）、落地的路由、事件的分類；成本測試的呼叫數不變。
- `cargo test -p speclink-desktop-core`：本機已封存清單對工單與章並存的封存項回 `reviewedNotPassed`／`verifiedNotPassed`，只有章的項目仍是 `reviewed`／`verified`。
- `cargo test -p speclink-protocol`：新欄位的形狀；新型別解析沒有這些欄位的舊回應、不認得的 state 讀作未知。「舊版用戶端解析新回應」（回答未驗證項）放在 `cargo test -p speclink-host --test station_wire`：它拿真的路由回應，用一組只含舊欄位的型別解析清單、單筆、已封存三種回應。
- `cargo test -p speclink-cli --test it change_landing::`：動詞在兩種模式的輸出、`show --json` 的新欄位、`list --json` 不變、蓋章與開工帶出 commit 與分支。
- `cargo test -p speclink-server`：串流的事件類別。
- `cargo test -p speclink-desktop-core`、`cargo test -p speclink-desktop`、`npm test -w apps/desktop`：四態的函式、遠端的標示、封存的對話框。
- Node：`host.dispatch` 執行 `change landed` 與 `show --json`。

**範圍**

- 在範圍內：上面列的介面、兩支技能各一步。
- 不在範圍內：規則、`list --json`、Server 端的 git、使用者文件。

## Risks / Trade-offs

- [回歸對照] `list --json` 不能變。→ 兩個關卡規格既有的釘住測試不改；CLI 遠端的清單轉換明確丟掉新欄位。
- [回歸對照] `show --json` 多欄位。→ 刻意的變更，golden 同批更新；人眼輸出的 golden 不變。
- [舊用戶端] 新欄位都是選填。→ 用舊型別解析新回應的測試；`speclink-remote` 與桌面既有的整條鏈測試不改。
- [跨平台] 讀 HEAD 與分支都是 CLI 既有呼叫 git 的方式（三個平台相同）；分支名稱可能含非 ASCII 字元，寫入時經跳脫函式。→ 測試放一個含 `/` 與中文的分支名稱。
- [Desktop 的成本] 遠端有 checkout 時要為已蓋章的變更各取一次單筆回應。→ 只對任務數相符的已蓋章變更做；數量通常是個位數，所以不做快取（量測見 Open Questions 的結論）。
- [技能多一步可能失敗] 落地的回報被拒絕。→ 技能寫明繼續，不回頭。
- [資料的可信度] commit 與分支是用戶端說的，Server 不查。→ 這是討論的裁定；宿主要驗證時在自己的掛鉤裡對自己的 Git 服務查。寫進文件影響。
- [儲存解耦] 新資料都在既有的 metadata 文件裡，儲存不用認得新的文件種類。

## Migration Plan

沒有資料搬移。既有的 metadata 沒有新鍵，讀作沒有資料。使用者執行 `speclink update` 取得兩支技能的新步驟。要退回時還原 commit；metadata 裡多出來的鍵會被舊版忽略並原樣保留。

## Open Questions

- commit 技能在「不是 worktree、但在功能分支上」提交時要不要回報落地。目前定為回報（引擎不知道哪個分支算正式分支；宿主可以用分支的記錄自己分辨）。實作時對照 commit 技能既有的流程確認這一步放的位置。
  - **結論（2026-11-12）：回報。** 技能只在一種分支上略過：目前的分支正好是 `speclink/<change-name>`（worktree 的變更分支，由 worktree-merge 在合併後回報）。其他分支一律回報；技能不猜哪個分支是正式分支。宿主要分辨時，用開工時記下的 `branch`，或拿落地的 commit 去問自己的 Git 服務。這一步放在 `git commit`（步驟 9）之後、顯示結果（步驟 10）之前，編號 9a；封存子流程執行過時略過（理由見 D8）。
- Speclink Desktop 的遠端快取鍵用「範圍版本號加檔案修改時間」是否足夠，或直接每次重算。實作時量一次有 10 個已蓋章變更的清單重新整理要多久，再決定要不要快取。
  - **結論（2026-11-12，實作時量測）：不做快取，每次重算。** 量測環境是整合測試的行程內 Server（loopback、debug 建置）、10 個已蓋章且任務數相符的變更、每個 scope 一個檔：清單重新整理沒有 checkout 時平均 17 ms，有 checkout 時 77 ms，也就是每個已蓋章的變更多約 6 ms（一次單筆讀取加讀檔算指紋）。真實網路上這筆成本是「已蓋章而尚未封存的變更數 × 一次往返」；這種變更通常是個位數（蓋章之後接著就是封存），而快取要在殼層多留一份跨命令的狀態並處理失效。等到實際遇到「很多已蓋章的變更同時掛著、連線又慢」再加快取；到時候快取鍵用單筆回應的版本加檔案的修改時間。

## 稽核結論

依 sharp-edges 清單（壞蛋、懶惰的開發者、搞混的開發者三個視角）檢查這個變更新增的介面，逐項結論如下。

**值會不會在 metadata 注入新的鍵**

- commit（蓋章與落地）：寫入前先過形狀檢查，只收 7 到 64 個十六進位字元，換行、冒號、井號都進不來。全數字的短 commit 會被 YAML 讀成數字（整份 metadata 因此解析失敗），所以寫入時經 `yaml_string`：讀回來不是同一個字串的值一律加引號。
- 分支：形狀檢查擋掉空白與控制字元（含換行、Tab、NEL、U+2028、全形空白），冒號、井號、引號、方括號這類 YAML 記號則照收並加引號。`null`、`true`、`123`、`1e5`、`~`、`.inf`、`0x1F` 這類會被讀成別種值的名稱同樣加引號。測試逐一寫入後讀回，確認值相同、文件其餘欄位不變、沒有多出鍵。
- 回報的人（`landed_by`）走既有的純量跳脫，測試以帶換行與冒號的名字確認沒有多出 `branch` 鍵。
- **找到並修正的問題**：手改過的 metadata 若把鍵寫成 `landed_commit : x`（冒號前有空白），文字手術看不到那個鍵，會再補一行同名的鍵，整份 metadata 從此解析失敗。落地是會重複覆寫的寫入，所以 `change landed` 在寫入前多一道把關：改完的文字解析後讀不回這次的 commit（與分支）就拒絕，零寫入。開工標記寫分支時有同一道把關（見下一條）。蓋章沿用既有的寫入路徑，這個既有的弱點（`started_at :`、`reviewed_at :` 等既有的鍵都一樣）不在這個變更擴大處理。
- **品質關卡找到並修正的問題**：分支的形狀檢查擋不掉 YAML 帶不了的字元（U+FFFE、U+FFFF）；這種名稱加了引號照樣寫不進去，寫入後整份 metadata 解析失敗，線上的開工請求就能觸發。開工標記寫分支前因此確認讀得回來，讀不回就以參數錯誤拒絕。分支檢查（`checked_branch`）本身也確認名稱存得進 metadata，CLI 自己帶分支之前用的就是這同一個檢查：過不了（形狀不合，或 metadata 存不了；git 允許的比引擎多）就當作沒有分支，不讓使用者沒給的參數把動詞擋下來；專案資料夾位於別的 repo 底下時也不帶外層 repo 的分支。

**commit 與分支會不會被當成判定的依據**

- 全部的讀取點只有三處：`show --json`、線上的清單與單筆回應、事件 payload。章的凍結判定、封存守門、階段推導都不讀它們；測試確認蓋章記下 commit 之後凍結判定仍只看任務數與內容指紋，提案中的變更回報落地後階段不變。

**清章時會不會留下 commit**

- 章的欄位清單只有一份（`Station::stamp_keys`），commit 已加進去。重新蓋章先移除整組再寫，沒有帶 commit 的重蓋會把舊的 commit 清掉；`change schema` 清驗證章用的是同一份清單。兩條路徑各有測試。

**線上的欄位會不會洩漏別的範圍的資料、清單會不會帶 scope**

- 回填只讀請求所在範圍的文件；測試在兩個範圍放同名的變更，確認另一個範圍讀不到章、工單、分支與落地。
- 清單項不帶 scope（測試斷言沒有這個鍵）；scope 只在單筆回應。
- 回填的儲存呼叫數與加欄位之前相同（清單 2 次列出 3 次讀取、單筆 3 與 3、已封存 1 與 1），工單與資料檔併在同一次讀取。

**舊版用戶端**

- 用加欄位之前的型別解析新的清單、單筆、已封存三種回應都成功（測試用真的回應，三種各一）；新版用戶端解析沒有這些欄位的回應讀作沒有資料；不認得的 state 與結局讀作未知。

**Speclink Desktop 讀 Server 給的 scope 路徑**

- **找到並修正的問題**：綁有 checkout 時，桌面會照單筆回應的 scope 路徑去讀本機檔案。路徑是 Server 給的外部輸入，絕對路徑或 `..` 會讓它讀 checkout 以外的檔案（例如一個永遠讀不完的裝置檔，重新整理就卡住）。現在路徑先過引擎既有的「repo 內相對路徑」檢查，不合的不讀、讀作內容有變；測試以絕對路徑與 `..` 確認。
- **品質關卡找到並修正的問題**：已封存的結局原本是「有章優先」，蓋章後重開工單再帶著工單封存的變更會被回報成通過。現在線上與本機都改成「有工單優先」，判定收成引擎的一個函式（見 D1）。這改了 client-protocol 兩條既有需求的意思，所以經 ingest 補進 delta 與任務 8.1。
- **品質關卡找到並修正的問題**：章若蓋在 checkout 還沒有的 commit 上，checkout 的檔案不是那一版，比了會把剛蓋好的章誤報成其後有變。現在只在 checkout 的 HEAD 包含章記下的 commit 時才比檔案（見 D7）。第二輪再補一步：被審的內容在蓋章時還沒提交的話，章記下的 commit 是它的前一版，checkout 有那個 commit 卻沒有被審的內容，仍會被誤報；所以和指紋不同、但還是那個 commit 當時樣子的檔案，不算其後有變。

**參數會不會被寫反而不報錯**

- 蓋章的呼叫有相鄰的兩個選填字串（工具名稱與 commit）。寫反時 commit 的位置拿到工具名稱，形狀檢查會拒絕（除非工具名稱剛好是十六進位字串），所以會大聲失敗。
- 落地回報的變更名稱與 commit 寫反時，是找不到變更。

**技能新增的一步失敗時**

- 兩支技能都寫明：動詞被拒絕時把訊息告訴使用者並繼續，不還原提交、不還原合併；合併衝突而中止的分支沒有回報的步驟。技能內容的測試逐句確認。

**接受的風險（不修）**

- commit 與分支是用戶端說的，Server 不查（討論的裁定）。有寫入權限的人可以覆寫落地記錄。要驗證的宿主在自己的掛鉤裡查。
- 線上的 `stampedAt` 只有日期（沿用既有的 `startedAt`），`landedAt` 是完整的 RFC 3339 時間。
