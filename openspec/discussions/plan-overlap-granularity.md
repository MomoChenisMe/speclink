---
topic: 改進 speclink plan 的排序：capability 重疊不該把小 change 排到最後
slug: plan-overlap-granularity
status: promoted
created: 2026-09-24
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: plan-requirement-overlap, plan-requirement-overlap-desktop
---

# Discussion: 改進 speclink plan 的排序：capability 重疊不該把小 change 排到最後

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在某個有 18 個進行中 change 的專案上跑 `speclink plan --json`，兩個無依賴、無人依賴的小型 UI change 被排到第 14、15 波（共 15 波）：它們的 delta 目錄碰到四個熱門 capability，引擎以目錄級重疊推波次、再以 created 決定先後，新建的小 change 因此墊底。使用者主張：重疊粒度應降到 requirement 級、重疊只影響封存順序不影響開工順序、tie-break 不該只看建立時間、輸出要說明被擋原因，並問 agent 能否自行調整波次。需求已具體（有預期輸出形狀與規則），未經 grill 直接進假設。相關規格：change-plan（基底與拓樸修正、plan 輸出、rank 移動檢查、apply／archive 技能守門）、board-card-order（rank 為輸入且受依賴約束）、archive-merge（封存合併 fail-closed 守門）、drift-computation（規格面維度沿用合併閘的 MergeViolation，只查 requirement 名存在）；無進行中 change。
Prior discussions: change-execution-order, plan-handoff-after-archive, worktree-apply-plan-preflight

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-24)

**Focus**: 目錄級 delta 重疊推波次是否該退場，退場後波次、封存順序、tie-break、輸出與 agent 把手各長什麼樣
**Position**: 波次只由 depends_on 推後，重疊降到 requirement 級且只產生封存順序提示；八條假設待使用者確認：
- 事實：`delta_capabilities` 只看 `specs/<cap>/spec.md` 存在（speclink-fs lib.rs:227），不讀內文；波次 = 1 + max(前置波次, 已配置重疊者波次)（plan.rs:331-340）；重疊不是有向邊，先後由基底順序（階段→rank→created→名字）決定
- 事實：drift 規格面直接沿用 `merge_violations`（drift.rs:46-50），只查 MODIFIED 目標名存在；兩個 change 改同一 requirement，後封存者會靜默蓋掉前者內文，drift 不會報
- 重開先前討論 change-execution-order 的「delta capability 重疊做拓樸修正」：當時理由是硬信號零成本現算，實例證明目錄級重疊把獨立小 change 鎖死，理由不再成立；該討論否決的「封存加軟依賴硬守門」維持不動
- 假設 1：推波次的邊只剩 depends_on，requirement 級重疊不推波次
- 假設 2：requirement 級重疊 = 同 capability 同 requirement 名被兩個 change 用 MODIFIED／REMOVED／RENAMED 動到；ADDED 對 ADDED 不算重疊，但兩個 change ADDED 同名另標「衝突」（後封存者必被合併閘規則 1 擋下，屬改名問題非順序問題）；重用 archive.rs:346 merge_violations 的區段解析
- 假設 3：archiveAfter 先後照基底順序定，只提示不擋封存；archive 技能改讀 archiveAfter
- 假設 4：提示文字不能只寫「封存前跑 drift」；改為「先封存 X；X 封存後重讀本 change 對 requirement R 的 MODIFIED 區塊、對照正典重寫（ingest）再封存」
- 假設 5：缺 rank 的 change tie-break 改為「被依賴數多的先→task 數少的先→created→名字」；有 rank 者照舊排前；此順序同時是桌面三欄卡片順序，缺 rank 的卡會重排
- 假設 6：新增 CLI 動詞 `speclink change rank <name> --before|--after <other>`，沿用 model.rs:325 set_board_rank 與 plan.rs:472 check_rank_move；不新增 wave 覆寫欄位（會與 rank、depends_on 三套真相打架）
- 假設 7：JSON 保留 `blockedBy`（語意窄化為只含 depends_on 前置），新增 `requirementOverlap`（change、capability、requirement、operation）與 `archiveAfter`；不拆 blockedBy 成物件——消費者含 CLI、protocol、remote convert、server route、桌面 query.rs 與 TrayPanel、五份技能 asset、四份 golden
- 假設 8：`--strict-overlap` 只作用於 plan 動詞，桌面與 server 不加開關
**Ruled out**: 直接存 wave 號讓 agent 覆寫——與 rank、depends_on 形成三套真相；把 blockedBy 拆成物件——改動面翻倍且桌面「等 N 項」灰章要重寫
**Open**: 八條假設的逐條確認；記錄不得出現外部專案實名；LLM 評估能否納入排序與 token 成本；桌面 UI 是否需重新設計；排程分頁與其他分頁的風格對齊

### Round 2 — interview (2026-09-24)

**Focus**: LLM 評估放哪一層、桌面要不要重設計、排程分頁樣式，以及八條假設是否成立
**Position**: 八條假設全部成立；LLM 只在技能層透過 rank 把手介入，桌面不重設計但排程分頁卡片化：
- 假設 1–8 使用者逐條同意，記錄不出現外部專案實名
- LLM 不進 plan 引擎：plan 被 apply／archive／commit／看板重繪／系統匣頻繁呼叫，塞 LLM 等於每次重繪燒 token 且結果不可重現、與拖排打架
- LLM 的位置是 propose 收尾既有的「盤點執行順序」步驟：寫 depends_on 之外，再以 `change rank --before` 決定新 change 是否插隊；成本＝每個 change 建立時一次，讀的 Impact 現在已在讀
- ingest 收尾已有 depends_on 重判（ingest.md:239-245，只判本 change、不移除既有邊）；rank 不在 ingest 寫——ingest 時 change 可能已開工，開工後的優先序由使用者拖排決定，技能只在 change 仍為提案中且內容規模明顯變動時口頭建議 `change rank`，不代寫
- propose.md:448 與 ingest.md:244 的「Hard signal — delta capability overlap」字面要改成 requirement 級重疊＋archiveAfter 提示的說法
- 桌面不重設計：波次、前置兩段不變；重疊段改列 `capability › requirement 名`＋操作，ADDED 同名標衝突；阻擋只剩前置；新增「封存順序」段列 archiveAfter 與重寫提示；卡片波次章與系統匣不動、灰章只數前置
- 排程分頁樣式：使用者選「卡片化對齊審查／驗證分頁」——每段成 `rounded-md border border-border/60 bg-muted/20` 卡片、標題列與內容分格；阻擋併入前置卡片標題列右側的狀態徽章（「可以開工」／「等 N 項」）；requirement 名與操作用審查分頁的小邊框標籤
**Ruled out**: LLM 進引擎（token 與可重現性）；ingest 代寫 rank（開工後優先序歸使用者）；維持段落只換標題徽章（使用者選卡片化）
**Open**: 切幾刀與各刀範圍；結論字面確認

### Round 3 — interview (2026-09-24)

**Focus**: 未開工的 change 走 ingest 時，技能要不要寫 rank
**Position**: 以「有沒有 board_rank」而不是「有沒有開工」當分界——沒 rank 才代寫，有 rank 只口頭建議：
- board_rank 欄位不記作者，技能分不出是使用者拖的還是 propose 寫的；有 rank 就視為人已排過，技能不覆寫
- 提案中且無 rank 的 change：ingest 收尾與 propose 收尾同一套判斷（讀其他提案 Impact、看本 change 規模），小而急者以 `change rank --before <other>` 插隊
- 提案中但已有 rank：口頭建議可跑 `change rank`，不代寫
- 已開工（進行中／已就緒）：維持現況，只重判 depends_on，rank 不碰
- propose 收尾的新 change 永遠沒 rank，所以同一條規則在 propose 等於「一律可寫」
**Ruled out**: 以開工與否當分界——提案中的 change 也可能被使用者拖排過，覆寫會吃掉人的決定
**Open**: 兩刀切法與結論字面的最後確認

## Conclusion

**Decision**: plan 的重疊降到 requirement 級且只產生封存順序，波次只由 depends_on 推後；缺 rank 的 change 改用「被依賴數→task 數→created」排；給 agent 一個 rank 把手；桌面排程分頁改內容並卡片化；分兩刀落地。
- 波次 = 1 + max(作用中宣告前置的波次)；delta 重疊不再推波次。`--strict-overlap` 旗標只作用於 plan 動詞，退回目錄級重疊推波次；桌面與 server 不加開關。
- requirement 級重疊：同 capability 同 requirement 名被兩個 change 用 MODIFIED／REMOVED／RENAMED 動到。先後照基底順序，後者標 `archiveAfter: [前者]`。ADDED 對 ADDED 不算重疊；兩個 change ADDED 同名另標「衝突」（後封存者必被合併閘擋下，屬改名問題）。區段解析重用封存合併閘的解析。
- archiveAfter 只提示不擋封存，不新增引擎守門。提示文字：先封存 X；X 封存後重讀本 change 對 requirement R 的區塊、對照正典重寫（ingest）再封存。不寫「跑 drift 即可」，因為 drift 規格面只查 requirement 名存在，抓不到內文被蓋。
- 基底順序：階段 → 有 board_rank 者照 rank → 缺 rank 者依「被依賴數多的先 → task 數少的先 → created 早的先 → 名字」。桌面三欄的缺 rank 卡片會跟著重排。rank 移動檢查不變（只有 depends_on 構成拒絕理由）。
- JSON：保留 `blockedBy`（語意窄化為只含 depends_on 前置），新增 `requirementOverlap`（每項 change、capability、requirement、operation、conflict 布林）與 `archiveAfter`（change 名陣列）。人眼輸出對應加行。不拆 blockedBy 成物件。
- 新動詞 `speclink change rank <name> --before|--after <other>`，沿用 set_board_rank 與 check_rank_move，違反 depends_on 即拒絕、零寫入。不新增 wave 覆寫欄位。
- LLM 不進 plan 引擎。技能層以「有沒有 board_rank」當分界：提案中且無 rank 的 change，propose 收尾與 ingest 收尾同一套判斷（讀其他提案 Impact、看本 change 規模），小而急者以 `change rank --before` 代寫插隊；提案中但已有 rank 只口頭建議、不代寫；進行中／已就緒維持現況只重判 depends_on、rank 不碰。propose 與 ingest 的「硬信號＝delta capability 重疊交引擎」字面改成 requirement 級重疊與封存順序提示。
- apply 守門改讀新語意的 blockedBy（字面不變）；archive 技能與 commit 的先封存子流程的順序提示改讀 archiveAfter 並附重寫提示；apply-worktree-pre 同步。
- 桌面排程分頁：四段卡片化（rounded-md border border-border/60 bg-muted/20，標題列與內容分格，對齊審查／驗證分頁）；阻擋段併入前置卡片標題列右側的狀態徽章（「可以開工」／「等 N 項」）；重疊段列 `capability › requirement`＋操作小邊框標籤、衝突另標；新增「封存順序」卡片列 archiveAfter 與重寫提示。卡片波次圓章與系統匣不動，灰章只數前置。
- 記錄與衍生 artifacts 不出現外部專案實名，一律寫「某個專案」。
- **cut 1 `plan-requirement-overlap`**: 引擎、CLI、技能資產。
  - plan.rs 的重疊粒度、波次公式、tie-break、--strict-overlap
  - change rank 動詞
  - JSON 與人眼輸出新欄位、golden 更新
  - propose／ingest／apply／archive／commit／apply-worktree-pre 六份技能字面，ASSET_VERSION／golden／assets.lock 三連動
- **cut 2 `plan-requirement-overlap-desktop`**: protocol、server、remote、桌面。
  - GET /plan 與 protocol query 帶新欄位、remote convert 往返
  - 排程分頁內容改動與卡片化、灰章改數前置
  - 系統匣沿用，只驗數字
**Rationale**: 目錄級重疊零成本但把獨立小 change 鎖死；requirement 級重疊才是封存時真正互相覆蓋的邊界，而且它只影響封存順序。把它從波次拿掉、改成提示，開工順序回到 depends_on 與人排的 rank，引擎維持零 token 且可重現，LLM 的判斷留在每個 change 只跑一次的技能收尾。
**Rejected alternatives**:
- LLM 進 plan 引擎 — plan 被 apply／archive／commit／看板重繪／系統匣高頻呼叫，燒 token 且不可重現、與拖排打架
- 直接存 wave 號給 agent 覆寫 — 與 rank、depends_on 形成三套真相
- 把 blockedBy 拆成物件 — 消費者跨 CLI、protocol、server、桌面、技能、golden，改動面翻倍
- archiveAfter 硬擋封存 — 先前討論 change-execution-order 已否決封存硬守門
- 以開工與否決定 ingest 是否代寫 rank — 提案中的 change 也可能被使用者拖排過，覆寫吃掉人的決定
- 排程分頁維持段落只換標題徽章 — 使用者選卡片化
- 桌面與 server 也加保守模式開關 — 沒有需求
- 合成一刀 — 桌面測試耗時，兩刀隔離
**Deferred**: none
**Capture to**: proposal（兩刀，本記錄 --hold，最後一刀帶 --last）；change-plan 規格四條 requirement MODIFIED（基底與拓樸修正、plan 輸出、change depends 旁新增 change rank、archive 技能建議）；desktop-app 排程分頁 requirement MODIFIED；propose-skill／ingest-skill／archive-skill／commit-skill 對應 requirement MODIFIED
**Next**: /speclink-propose --from-discussion plan-overlap-granularity（cut 1）
