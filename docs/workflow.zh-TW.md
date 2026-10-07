# 完整 SDD 工作流

**繁體中文** · [English](workflow.md)

這份文件說明 Speclink 的每一站：做什麼、用哪個技能、什麼時候可以跳過、怎樣算完成、完成後去哪裡。

第一次用，先照 [Local Repo 入門](getting-started.zh-TW.md)走一輪。想知道某項能力現在能不能用，查[專案能力狀態](product-status.zh-TW.md)。

## <a id="mental-model"></a>整體流程

```text
baseline? → discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive
                                            ↑
                                    閒置後續作：先 drift

worktree：apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive

工具：validate / analyze / audit / commit / config / manual / trace / plan
```

- 帶 `?` 的站是選用的。
- `propose → apply → archive` 是每個變更都會走的主線。實作途中需求改變，就在 `apply` 與 `ingest` 之間來回。
- 封存前有兩道選用的品質關卡：`review` 看程式碼寫得好不好，`verify` 看交付是否符合規格。兩道互不依賴，依風險決定要跑哪幾道。低風險的變更兩道都跳過也可以。
- 工具類的站在需要時才用，不是每個變更都要經過。

桌面 app 把這條路線畫成看板：討論、提案中、進行中、已就緒四欄，每張卡片就是一個變更或一份討論。已封存的變更在另一個頁面。

![Speclink 桌面 app 的看板，分成討論、提案中、進行中與已就緒四欄](assets/screenshots/desktop-board.png)

## <a id="entry"></a>從哪一站開始

依序問下面的問題，第一個答「是」的就是入口：

| 問題 | 入口 |
| --- | --- |
| 只是想了解某件事，沒有要做決定？ | 直接問 Agent，不要建討論記錄。 |
| 既有程式還沒有任何規格？ | 先 `baseline`。它只描述現在的行為，不建立變更。 |
| 已經有相關的變更？ | 繼續實作用 `apply`；新的背景會改到文件，用 `ingest`。 |
| 變更停了一陣子，規劃時的假設可能已經不對？ | 先 `drift`，再依結果回 `apply` 或 `ingest`。 |
| 想改善程式碼，但講不出要改哪裡？ | `improve`，請模型掃描並提出候選。 |
| 新需求已經清楚？ | 清楚就 `propose`；還要取捨就 `discuss`。 |

## <a id="stage-types"></a>站的種類

| 種類 | 站 | 說明 |
| --- | --- | --- |
| 主線 | `propose`、`apply`、`ingest`、`archive` | 變更從規劃、實作、更新需求，到合併進正式規格。 |
| 視情況 | `baseline`、`discuss`、`improve`、`drift`、worktree 流程 | 既有程式第一次建規格、需求要收斂、停了一陣子再繼續，或要平行推多個變更時才用。 |
| 品質關卡 | `review`、`verify`、`quality` | 封存前的兩道選用檢查，各自記工單、蓋章。 |
| 工具 | `validate`、`analyze`、`audit`、`commit`、`config`、`manual`、`trace`、`plan` | 檢查文件、安全檢查、提交、設定、手冊、溯源與執行順序。 |

## <a id="how-to-call"></a>怎麼呼叫

| 層 | 負責什麼 | 例子 |
| --- | --- | --- |
| 技能 | 寫給 Agent 讀的流程知識：什麼時候讀什麼、怎麼產生與檢查文件、什麼時候停下來問你。 | Claude `/speclink-propose`、Codex `$speclink-propose` |
| `speclink` CLI | 真正執行動作的命令列。本地與遠端都用它。 | `speclink status --change add-csv-export` |
| Host | 把引擎、Store、認證、版本、交易與事件組在一起。CLI、server 與 Node SDK 都經過它。 | 內嵌在 CLI 裡，或 `speclink-server` |

下面的技能名都用 Claude 的寫法。Codex 把開頭的 `/` 換成 `$`，也可以輸入 `/skills` 從清單挑。`analyze` 是例外：Codex 沒有這個技能，直接用 CLI。

升級 Speclink 之後，執行 `speclink update` 重新產生技能檔。

## <a id="stages"></a>各站說明

每一站用同樣的格式：第一句是用途，下面的清單是細節。

### <a id="baseline"></a>baseline：建立規格基準

從現有的程式碼與測試，為系統現在的行為建立正式規格。舊稱 onboard。

- **什麼時候用**：既有專案剛導入 Speclink、還沒有規格，或只想補上沒涵蓋的能力。**跳過**：已經有足夠的規格，或你要寫的是新需求。
- **輸入 → 產物**：README、程式入口、原始碼、測試、`openspec/config.yaml` 的專案說明與產出規則，以及你確認過的 capability 清單 → 直接寫進 `openspec/specs/<capability>/spec.md`，不建立變更。
- **技能**：`/speclink-baseline [範圍]`
- **底層 CLI**：沒有 `speclink baseline` 指令。Agent 寫完規格後，用 `speclink validate --specs --all --strict` 檢查。
- **完成**：你確認了能力的邊界，每份規格都有程式碼證據，strict 檢查通過。
- **下一步**：新需求走 `propose`；還模糊就先 `discuss`。
- **出狀況**：發現既有規格要改，不要在 baseline 裡改，另開一個變更。

### <a id="discuss"></a>discuss：討論

把需要取捨的問題一輪一輪談清楚，留下有來源的結論。

- **什麼時候用**：需求模糊、有好幾個合理做法、需要做決定。**跳過**：只是想了解某件事；或需求已經清楚，直接 `propose`。
- **輸入 → 產物**：一個主題，或一份文件的路徑（自己寫的計畫、plan mode 的產出）→ `openspec/discussions/<slug>.md`，含背景、每一輪與結論。
- **技能**：`/speclink-discuss <主題或文件路徑>`。開場會先搜尋舊討論，不重提已否決的方向。
- **底層 CLI**：`speclink discuss new`、`context`、`add-round`、`conclude`、`search`。
- **完成**：結論寫明決定、理由、否決的方案、暫緩的事與下一步。
- **下一步**：見[討論結論怎麼走](#discussion-outcomes)。
- **出狀況**：已經談出內容的討論，寫結論後封存。什麼都還沒談出來，才用 `speclink discuss discard` 刪掉。

### <a id="improve"></a>improve：找改進點

請模型掃描程式碼，提出可以改進的候選，寫成一份討論。

- **什麼時候用**：想改善程式碼，但講不出要改哪裡。**跳過**：你已經知道要改什麼，那是 `discuss` 或 `propose` 的題目。
- **限制**：只有你可以發起，模型不會自己跑。它只產生討論記錄，不改程式碼。
- **輸入 → 產物**：你指定的方向（最好有），或依 Git 歷史的熱點推測範圍 → 一份標記為改進討論的討論記錄，每個候選附檔案、問題、解法、好處與建議強度。
- **技能**：`/speclink-improve [範圍]`
- **底層 CLI**：`speclink discuss new <主題> --kind improve`，之後和 `discuss` 相同。
- **完成**：候選都列出來，你挑一個談完，結論也寫下了。候選全部不採用也要寫結論並封存，下次掃描才不會重提。
- **下一步**：採用的候選走 `propose`；全部不採用就封存討論。
- **出狀況**：模型重提已否決的候選時，會指出是哪一份討論否決的。

### <a id="propose"></a>propose：提案

建立一個可以交給實作的變更。

- **什麼時候用**：需求已經清楚，或一份已寫結論的討論要變成完整提案。**跳過**：只是問問題、只要整理現有規格（用 `baseline`），或變更已經存在只需要更新（用 `ingest`）。
- **輸入 → 產物**：需求描述、討論（`--from-discussion <slug>`）或一份文件（`--from-doc <path>`）→ `proposal.md`、delta 規格、`tasks.md`，需要時加上 `design.md`。實際要哪幾份，由產出流程決定。
- **技能**：`/speclink-propose <變更名稱>`。寫完文件後，它會跑 `analyze` 修正問題（最多兩輪），再跑 `validate`，最後用 `speclink change depends` 記下這個變更要等哪些變更先做完。
- **底層 CLI**：`speclink new change`、`instructions`、`new artifact`、`analyze`、`validate`、`change depends`。
- **完成**：`speclink status --change <名稱>` 顯示開工需要的文件都完成，`validate` 通過。
- **下一步**：由你決定何時 `apply`。
- **出狀況**：需求還不清楚，回 `discuss`。

### <a id="apply"></a>apply：實作

照任務清單實作，每完成一項就勾掉。

- **什麼時候用**：開工需要的文件都齊了。**跳過**：文件還沒齊、需求正在變，或變更停了一陣子還沒跑 `drift`。
- **輸入 → 產物**：提案、規格、設計（如果有）與任務 → 程式碼與測試的修改、勾好的任務，以及每項任務動到哪些檔案的證據。證據寫在 `openspec/changes/<名稱>/.evidence.json`，跟著變更一起提交。
- **技能**：`/speclink-apply <變更名稱>`
- **底層 CLI**：開工前先跑 `speclink plan`，被其他變更擋住就停下來。接著 `speclink review prepare`（記下品質關卡要用的起點）與 `speclink in-progress add`。實作時用 `speclink instructions apply --change <名稱> --json` 取得背景，每完成一項跑 `speclink task done --change <名稱> <編號>`。
- **完成**：每項任務的行為與該過的測試都通過，`instructions apply` 回報 `state: all_done`。標著 `[M]` 的手動任務由你確認，Agent 不會代勾。
- **下一步**：依風險決定要不要跑品質關卡，然後 `archive`。需求改變就先 `ingest`。
- **出狀況**：
  - 勾錯或實作回滾：`speclink task undone --change <名稱> <編號>`。
  - 誤開工而且還沒動任何東西：`speclink in-progress remove <名稱>`，退回提案中。
  - 不做了：`speclink discard <名稱>`，開工後要加 `--force`。

實作期間，變更的詳情面板最好用：提案、設計、任務與規格分頁對應同一組文件，任務分頁的進度就是 `speclink task done` 的結果。

![變更的詳情面板，顯示提案內容以及任務、規格分頁](assets/screenshots/desktop-change-drawer.png)

### <a id="ingest"></a>ingest：更新需求

把新的需求、計畫或討論結論，併進已經存在的變更。

- **什麼時候用**：實作途中需求或背景改變，或一份討論的結論要併進既有變更。**跳過**：只是繼續實作、文件不用改（用 `apply`）；還沒有變更（用 `propose`）。
- **輸入 → 產物**：既有的變更，加上對話內容或一份計畫檔 → 更新後的提案、設計、規格與任務。已完成的任務不會被改寫。
- **技能**：`/speclink-ingest`（用目前對話的內容），或 `/speclink-ingest <計畫檔>`。參數是計畫檔，不是變更名稱；要更新哪個變更，技能會從對話判斷或問你。
- **底層 CLI**：`speclink instructions <artifact> --json`、`analyze`、`validate`。討論併入時，內容寫進去之後再跑 `speclink discuss seal <slug> <變更名稱>`。
- **完成**：新背景都反映到受影響的文件，已完成的任務沒被改寫，`analyze` 與 `validate` 通過。有 link 的話也已經 seal。
- **下一步**：回 `apply`。
- **出狀況**：不要先 seal 再補內容。seal 代表內容已經寫進去了。

### <a id="drift"></a>drift：檢查漂移

檢查一個停了一陣子的變更，和現在的程式碼差多遠。

- **什麼時候用**：變更暫停後要繼續，或懷疑別人的 commit 改到同一塊。**跳過**：一路連續在做，基準沒變。變更建立超過 5 天，而且最近 3 天沒有人動它，`apply` 會建議先跑 drift。
- **輸入 → 產物**：變更的文件、Git 歷史、目前的程式碼與證據 → 一份報告：漂移程度（light、medium、heavy）、失效的參照、任務衝突，以及一個建議的下一步。
- **技能**：`/speclink-drift <變更名稱>`
- **底層 CLI**：`speclink drift <變更名稱> --json`
- **完成**：報告列出時間差、失效參照、任務衝突與建議，你選好了下一步。
- **下一步**：light 回 `apply`；medium 用 `ingest` 更新計畫；heavy 建議用 `speclink archive <名稱> --skip-specs` 收掉重來，也可以先試 `ingest`。
- **出狀況**：看不懂的外部修改先保留，不要用重置或覆寫解決。

### <a id="worktree"></a>worktree：平行實作

同時推多個互不相干的變更，每個變更在自己的 git worktree 裡做，彼此不干擾。

- **什麼時候用**：手上有兩個以上互不衝突的變更。**跳過**：只有一個變更，或幾個變更會改到同一批檔案（排隊做比較快）。
- **前置**：先開啟 worktree 設定：`speclink workflow-config set worktree true`。兩個 worktree 技能只在設定開啟時才會產生。
- **輸入 → 產物**：一個可以開工的變更 → 一個 worktree 與分支。實作、品質關卡與提交都在裡面完成。
- **技能**：`/speclink-apply-with-worktree <變更名稱>`，收尾用 `/speclink-worktree-merge <變更名稱>`。一次只處理一個變更；要平行就另開視窗，每個 session 各跑一個。
- **底層 CLI**：`speclink plan` 判斷哪些變更可以並行；`speclink list` 用 `[worktree]` 標出在 worktree 裡的變更。
- **完成**：worktree 裡的任務做完、選擇要跑的品質關卡已蓋章、變更已提交。`worktree-merge` 把分支併回主分支，並清掉 worktree。
- **下一步**：回主 checkout 執行 `archive`。封存只能在主 checkout 跑，在 worktree 裡會被拒絕。
- **出狀況**：品質關卡要在 worktree 裡跑，因為起點記在那裡。worktree 裡的 `tasks.md` 和主 checkout 的是兩份，只改 worktree 那份。

### <a id="quality"></a>quality：兩道關卡一起跑

同一個變更要跑兩道品質關卡時，用它一次安排好。

- **什麼時候用**：改動大，程式碼品質與是否符合規格都在意。**跳過**：只想跑一道，直接用 `/speclink-review` 或 `/speclink-verify`。
- **輸入 → 產物**：程式任務都完成的變更 → 兩張工單（`review.md`、`verify.md`），最後是兩枚章。
- **技能**：`/speclink-quality <變更名稱>`
- **底層 CLI**：`speclink review` 與 `speclink verify` 的 `scope`、`add-round`、`show`、`stamp`。
- **完成**：兩道都先檢查、先不蓋章，每一輪停下來等你決定：全修、挑著修，或不修。你說可以了，才依序蓋章：先審查，再驗證。
- **下一步**：`archive`（worktree 流程先 `worktree-merge`）。
- **出狀況**：沒有問題的一輪也會停下來，不會自己蓋章或封存。檢查途中不要提交，提交會讓修改離開檢查的範圍。

### <a id="review"></a>review：審查

檢查程式碼寫得好不好，發現的問題分級記在工單裡。

- **什麼時候用**：改動大、跨模組，或是會長期維護的程式碼。**跳過**：低風險的小改動。跳過是正當選擇，不是欠帳。
- **輸入 → 產物**：從 apply 開工時記下的起點到現在的修改 → `review.md` 工單。問題分 CRITICAL、WARNING、SUGGESTION 三級。判準是 repo 自己的慣例文件，加上常見的程式碼壞味道與找 bug。
- **技能**：`/speclink-review <變更名稱>`
- **底層 CLI**：`speclink review prepare`、`scope`、`add-round`、`show`、`stamp`、`discard`。
- **完成**：程式任務都完成，最後一輪沒有必修問題（CRITICAL、WARNING），就可以蓋章。SUGGESTION 不擋章。
- **下一步**：也要跑 `verify` 就接著跑，否則 `archive`。
- **出狀況**：有必修問題但你決定接受，用 `speclink review stamp --accept` 帶著問題蓋章。

### <a id="verify"></a>verify：驗證

逐條對照這個變更的規格，判斷交付是否符合。

- **什麼時候用**：規格條款多，或符合規格本身就是交付重點。**跳過**：低風險的小改動。
- **輸入 → 產物**：變更的全部文件，以及凍結下來的修改內容 → `verify.md` 工單。
- **技能**：`/speclink-verify <變更名稱>`。任務還沒做完時也能跑，當作進度盤點。
- **底層 CLI**：`speclink verify scope`、`add-round`、`show`、`stamp`、`discard`。
- **完成**：程式任務都完成，最後一輪沒有必修問題。SUGGESTION 不擋章。
- **下一步**：`archive`。
- **出狀況**：第一輪做完整檢查。之後每一輪只看上一輪沒解決的問題，以及修正造成的新問題。必修問題每一輪都要變少；沒有變少就以「未通過」停下，保留工單、不蓋章。

### <a id="quality-rules"></a>品質關卡的共同規則

| | `review` 審查 | `verify` 驗證 |
| --- | --- | --- |
| 回答的問題 | 程式碼寫得好不好 | 交付是否符合規格 |
| 看什麼 | 修改的程式碼，對照 repo 慣例 | 變更的規格，逐條對照修改 |
| 產物 | `review.md` 工單 | `verify.md` 工單 |
| 蓋章順序 | 先 | 後 |

- **前提**：只看程式任務。標著 `[M]` 的手動任務沒勾，也可以檢查與蓋章；但封存前要完成。
- **兩道都跑時，先修完再一起蓋**：章記錄的是範圍內檔案的內容。先蓋的章會被另一道關卡的修正弄成「其後有變動」。
- **蓋章會消耗工單**：蓋章時，同一次寫入會寫上章，並刪掉工單（`review.md`、`verify.md`）。所以已蓋章的變更封存後沒有工單檔，蓋章後 `show` 回報「沒有工單」是正常的。本地模式的工單文字只留在 Git 歷史；遠端模式蓋章後讀不回工單文字。
- **只有沒結的工單會跟著封存**：要用 `--carry-review` 或 `--carry-verify` 明確帶走。
- **蓋章後檔案又被改**：卡片顯示「已審查·其後有變動」或「已驗證·其後有變動」，封存也會被擋。回該關卡再跑一輪、重新蓋章。
- **封存遇到沒結的工單會被擋**：回去蓋章、用 `discard` 放棄這道關卡，或用 `--carry-*` 帶著走。

卡片與系統匣面板上，審查章與驗證章並排顯示，審查在前。

### <a id="archive"></a>archive：封存

把 delta 規格合併進正式規格，並把完成的變更移進封存區。

- **什麼時候用**：任務全部完成（含 `[M]` 手動任務）、文件檢查通過、假設沒有過期，而且選擇要跑的品質關卡都結束了。**跳過**：還有任務沒做完、`validate` 沒過，或需求還在變。
- **輸入 → 產物**：可以封存的變更 → 更新後的正式規格，以及 `openspec/changes/archive/<日期>-<名稱>/`。
- **技能**：`/speclink-archive <變更名稱>`。封存前會依 `plan` 提醒封存順序，封存後點名下一個可以開工的變更。
- **底層 CLI**：`speclink archive <名稱>`。可以一次列多個名稱，或用 `--all` 封存所有可以封存的變更。遠端一次只能封存一個。
- **完成**：指令成功，印出合併進正式規格的統計，變更已移進封存區。如果變更來自一份已寫結論的討論，而且它是那份討論最後一個還沒封存的變更，討論也會一起封存（討論標了 `hold` 的除外）。
- **下一步**：用 `/speclink-commit` 或你自己的方式提交。
- **出狀況**：
  - 不要用 `--no-validate` 或 `--mark-tasks-complete` 跳過沒做完的事。
  - 規劃時的假設已經過期：回 `drift` 或 `ingest`，不要硬封存。
  - 被章失效或沒結的工單擋下：見[品質關卡的共同規則](#quality-rules)。

### <a id="validate"></a>validate：格式檢查

檢查變更或規格的格式、必要段落與規則。封存時會被拒收的 delta（目標需求不存在、撞名、少宣告移除的 scenario），也會在這裡提早抓出來。

- **什麼時候用**：提案完成、文件更新後、封存前。交付前不要跳過。
- **技能**：沒有獨立技能，`propose`、`ingest` 與 `archive` 會呼叫它。
- **CLI**：`speclink validate <名稱>`；檢查全部正式規格用 `speclink validate --specs --all --strict`。
- **完成**：exit code 為 0，顯示 valid。
- **下一步**：`analyze`、實作，或 `archive`。
- **出狀況**：照錯誤訊息修文件，再跑一次。

### <a id="analyze"></a>analyze：交叉比對

交叉比對提案、設計、規格與任務，找出涵蓋、一致、模糊與缺漏四類問題。

- **什麼時候用**：提案或 ingest 完成後（技能會自動跑）。它不是程式測試。
- **技能**：`/speclink-analyze <變更名稱>`（只有 Claude 有；Codex 直接用 CLI）。
- **CLI**：`speclink analyze <名稱> [--json]`
- **完成**：沒有 CRITICAL；WARNING 與 SUGGESTION 看過並決定是否處理。
- **下一步**：修文件，或 `apply`。
- **出狀況**：有 CRITICAL 就先修文件，不要開始實作。

### <a id="audit"></a>audit：安全檢查

從安全的角度檢查還沒提交的程式修改：危險的預設值、型別混淆、靜默失敗。

- **什麼時候用**：碰到認證、權限、設定、外部輸入或對外介面時。`openspec/config.yaml` 設了 `audit: true` 時，`apply` 實作時也會套用精簡版的檢查。**跳過**：只改文件。
- **輸入 → 產物**：`git diff HEAD`（目前還沒提交的修改）→ 依嚴重度排序的問題清單。它不改變變更的狀態，也不是每個變更都要跑。
- **技能**：`/speclink-audit`（不帶變更名稱）
- **CLI**：沒有 `speclink audit` 指令。
- **完成**：每個問題都附位置、可能被怎麼誤用、怎麼修；或明確回報沒有問題。
- **下一步**：修正後回 `apply` 或跑測試。

### <a id="commit"></a>commit：限定範圍的提交

只挑出屬於某一個變更的檔案來提交。

- **什麼時候用**：想要一個只含單一變更的 commit。**跳過**：你有自己的提交方式。它不是每個變更都要經過的一站。
- **輸入 → 產物**：變更名稱、Git 狀態與任務證據 → 經你確認的檔案清單，以及一個 commit。
- **技能**：`/speclink-commit <變更名稱>`。也支援「先封存再一起提交」。
- **CLI**：技能用 `speclink list --json`、`.evidence.json`、`speclink artifact cat`、`speclink plan` 與 Git 判斷檔案，不用 `git add .`。
- **完成**：commit 只含你確認過的檔案，並回報 hash 與訊息。
- **下一步**：繼續 `apply`，或 `archive`。提交不能取代封存。
- **出狀況**：清單裡有無關的檔案就排除，不要覆寫或刪掉它們。

### <a id="config"></a>config：工作流設定

從程式碼整理出專案說明與產出規則，寫進 `openspec/config.yaml`，讓 Agent 寫出的文件貼合這個 repo。

- **什麼時候用**：剛導入時，或專案慣例改變後。**跳過**：預設已經夠用。
- **輸入 → 產物**：程式碼、套件設定、README 與測試 → 經你核可的 diff。
- **技能**：`/speclink-config`
- **CLI**：`speclink workflow-config show`、`set`、`context`、`rules`。欄位見[設定說明](configuration.zh-TW.md)。
- **完成**：diff 經你核可並寫入。
- **下一步**：回任何一站；之後產生的文件都會套用新設定。
- **出狀況**：寫錯就再跑一次改回來，不影響既有變更。

### <a id="manual"></a>manual：手冊

從正式規格產生給人讀的操作手冊（`openspec/manual/`），或在對話中帶你導覽系統。

- **什麼時候用**：需要一份手冊、要帶新人認識系統，或封存後想知道哪些手冊頁可能過期。**跳過**：沒有使用者面的規格，或沒人要讀手冊。
- **輸入 → 產物**：只讀正式規格，不讀 README、docs 或程式碼 → 手冊頁（含首頁與來源頁）。導覽模式不寫檔。
- **技能**：`/speclink-manual`（產生）、`/speclink-manual 導覽`（導覽）
- **CLI**：沒有 `speclink manual` 指令。桌面 app 的「手冊」頁可以瀏覽手冊。
- **完成**：摘要列出新增、重寫、沒動的頁數，以及可能過期的頁。
- **下一步**：提交手冊的修改。遠端專案目前不能產生手冊，導覽可以。
- **出狀況**：手冊頁是一般檔案，刪掉或還原即可。

### <a id="trace"></a>trace：溯源

回答「這個功能怎麼來的、為什麼這樣設計」。它沿著正式規格、封存的變更、討論與程式碼往回找。

- **什麼時候用**：接手不熟的功能，或要改一個設計之前。只限本地。
- **輸入 → 產物**：一個 capability 名稱或一個問題 → 溯源鏈：哪些變更、哪些討論、動過哪些檔案，每一段都附來源。
- **技能**：`/speclink-trace <capability 或問題>`
- **CLI**：`speclink trace <capability> [--json]`
- **下一步**：要改就 `discuss` 或 `propose`。

### <a id="plan"></a>plan：執行順序

排出所有未封存變更的執行順序。

- **什麼時候用**：手上有多個變更，不知道先做哪個。技能也會自動用它：propose 收尾記下依賴、apply 開工前檢查、archive 前後提醒順序。
- **輸入 → 產物**：所有未封存的變更與宣告的依賴 → 一份順序，內容有：
  - 一波一波的順序。同一波的變更可以並行。
  - 每個變更被誰擋住。
  - 下一個可以開工的變更。
  - 兩個變更改到同一條需求時的封存順序。
- **技能**：沒有獨立技能。
- **CLI**：
  - `speclink plan [--json]`
  - 宣告依賴：`speclink change depends <變更> --on <前置變更>`（加 `--remove` 取消）
  - 調整同一欄裡的順序：`speclink change rank <變更> --before <另一個變更>`（或 `--after`；只限本地）
- **下一步**：從 `next` 指出的變更開工。桌面詳情面板的「排程」分頁顯示同一份資訊。

## <a id="discussion-outcomes"></a>討論結論怎麼走

| 結論 | 什麼時候用 | 怎麼做 | 結果 | 下一步 |
| --- | --- | --- | --- | --- |
| 建立新變更，一次寫完 | 結論明確，想直接拿到全部文件 | `/speclink-propose --from-discussion <slug>` | 建立並連結變更，寫完開工需要的全部文件 | 文件檢查通過後，由你決定何時 `apply` |
| 建立新變更，先佔位 | 只想先建立變更，稍後再寫完整提案 | `speclink discuss promote <slug> [--name <變更名稱>]` | 建立變更，用結論（沒有結論時用主題）預填 proposal 的 Why，討論標為已轉出變更。**還不能開工** | 對這個變更再跑一次 `propose`，補齊文件 |
| 併入既有變更 | 結論要修正一個進行中的變更 | `speclink discuss link <slug> <變更>` → `/speclink-ingest` → `speclink discuss seal <slug> <變更>` | `link` 只建立來源連結；`ingest` 把內容寫進文件；`seal` 才把討論標為已轉出變更 | 回 `apply` |
| 決定不做 | 談出了內容，但結論是不做 | `speclink discuss archive <slug>` | 保留結論與理由，不建立空的變更 | 無 |

- 轉為變更不一定要等討論結束：討論還在進行時，就可以先轉出一部分。
- 一份討論可以轉出好幾個變更。要分批轉出時，用 `speclink discuss conclude --hold` 保留討論；轉出最後一批時加 `--last`（`discuss promote`、`new change --from-discussion` 與 `discuss seal` 都接受）。
- 討論已寫結論、沒有標 `hold`，而且最後一個關聯的變更封存時，討論會自動一起封存。
- `link` 之後不能先 `seal` 再補內容。seal 代表決定已經寫進文件了。
- 想找以前的決定：`speclink discuss search <關鍵字>` 會搜尋進行中與已封存的討論。

## <a id="recovery"></a>出狀況時

| 狀況 | 怎麼辦 |
| --- | --- |
| 轉為變更後只有 proposal 的骨架 | 對同一個變更跑 `propose`，不要直接 `apply`。 |
| 討論結論要併進既有變更 | `link` → `ingest` → `seal`，三步都要。 |
| 變更停了一陣子 | 先 `drift`；light 回 `apply`，假設過期走 `ingest`。 |
| 實作途中需求改變 | `ingest` 更新文件，再跑 `analyze` 與 `validate`，然後回 `apply`。 |
| `apply` 說缺文件 | 回 `propose` 補齊。 |
| `apply` 說被其他變更擋住 | 先做擋住它的變更，或用 `speclink change depends --remove` 取消不需要的依賴。 |
| 任務勾錯或實作回滾 | `speclink task undone --change <名稱> <編號>`。 |
| 誤開工、想退回提案中 | `speclink in-progress remove <名稱>`，只有還沒動任何東西時可以。 |
| 遠端的唯讀投影標成可能過期或被改動 | 不要直接改投影；重新取得 instructions 讓它更新。 |
| `analyze` 有 CRITICAL | 先修文件的涵蓋、一致與缺漏問題，再實作。 |
| 蓋章後檔案又被改 | 回該關卡再跑一輪、重新蓋章。 |
| 封存被沒結的工單擋下 | 回去蓋章、放棄這道關卡，或用 `--carry-*` 帶著走。 |
| 在 worktree 裡封存被拒 | 先 `worktree-merge` 回主分支，在主 checkout 封存。 |
| 封存說 delta 過期或不完整 | 回 `drift` 或 `ingest` 修正 delta，再跑 `validate`。 |

## <a id="limits"></a>目前限制

- `validate` 與 `analyze` 只檢查文件，不是程式測試，也不代表實作符合規格。實作面由品質關卡負責。
- 桌面 app 的遠端看板勾任務時，不回報動到的檔案（CLI 會）。其他限制見[專案能力狀態](product-status.zh-TW.md)。

## <a id="related"></a>相關文件

- [Local Repo 入門](getting-started.zh-TW.md)
- [Remote 入門](remote-getting-started.zh-TW.md)
- [設定說明](configuration.zh-TW.md)
- [專案能力狀態](product-status.zh-TW.md)
- [動詞與旗標契約](verb-contract.zh-TW.md)
