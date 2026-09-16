---
topic: changes 的有限度排行：propose 後排好順序與可並行資訊，apply 照順序跑，desktop／tray 看得到順序
slug: change-execution-order
status: promoted
created: 2026-09-15
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: add-change-plan-engine, add-change-plan-desktop, add-change-plan-remote
---

# Discussion: changes 的有限度排行：propose 後排好順序與可並行資訊，apply 照順序跑，desktop／tray 看得到順序

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出：看板與系統匣的變更卡只照時間排，看不出誰該先做；propose 收尾的順序判定只在 AI 腦中、不落檔，apply 也不看，每次都要重讀全部 change 浪費 token。目標：propose 完成後把順序與可並行資訊算好存好，apply 不指名時照順序挑、指名時守門，desktop／tray 依同一份順序呈現並保留手動拖排。需求已可驗證，未經 grill 直接進假設。
掃描結果：propose-skill 規格「收尾盤點提案中變更的執行順序」已規定 AI 判定但不落檔；Store::delta_capabilities 可機械算硬信號；board-card-order 的 board_rank 已承載手動拖排（local 存卡片 meta、remote 存 server board resource，桌面與系統匣共用 board_sorted_changes）；ChangeMeta 無依賴欄位；list --json 無 delta 與依賴欄位。
相關規格：propose-skill、board-card-order、remote-board-order、tray-status-menu、archive-merge、worktree-apply-skill。
Prior discussions: task-marker-ui-and-parallel-removal, worktree-parallel-apply, worktree-archive-merge-order

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-15)

**Focus**: 執行順序從哪來、存哪、誰用——七條假設與規格對照
**Position**: 引擎算順序、propose 寫依賴、apply 讀 plan、桌面與系統匣共用同一份順序；使用者確認大多正確，並補四點修正。
- 決策樹：A 順序基底（建立先後 vs 拖排）／B 依賴存哪（每卡 meta vs 中央檔 vs 現算；B1 硬信號、B2 軟信號）／C 輸出動詞形狀／D apply 消費（D1 不指名、D2 指名守門）／E 桌面與系統匣呈現／F remote
- 假設 2 成立：軟依賴存每張卡 .openspec.yaml 新欄位 depends_on（同 board_rank 的每卡自帶模式，避免中央檔在平行 session／worktree 對撞）
- 假設 3 成立：硬信號（delta capability 重疊）不存、每次由 Store::delta_capabilities 現算，delta 目錄即真相
- 假設 4 成立：新動詞 speclink plan --json 輸出波次與 blocked_by，只算未封存 change（已就緒仍算未封存，因合併閘看封存順序）；不塞進凍結的 list --json
- 假設 5 成立：apply 指名與否都先呼叫 plan（引擎算、零 token）；不指名取第一個可開工者，指名時 blocked_by 非空即阻擋
- 假設 6 成立：桌面與系統匣的順序來自同一引擎函式
- 使用者修正一：plan --json 要有讓 AI 一眼讀懂依賴與順序的乾淨結構（下一輪定形狀）
- 使用者修正二：不只提案中，進入進行中之後的欄內顯示也要照排定順序
- 使用者修正三：要定 UI 設計
- 使用者修正四：remote 一起納入，不延後（推翻假設 7）
- 規格對照：手動拖排已由 board-card-order 涵蓋；propose-skill 的「收尾盤點」與本題衝突，須改為判定後寫入；depends_on／plan／apply 守門／卡片徽章為規格空白
- 介面深度：新模組落 speclink-core lifecycle 層，CLI 與桌面 core 各一薄轉接；刪掉即兩端失去順序，非空殼
**Ruled out**: 中央順序檔（平行 session／worktree 對撞）；把硬信號存檔（delta 一改即過期）；塞進 list --json（動到凍結輸出）；只在不指名時查 plan（指名會撞依賴，與使用者補充相反）；remote 延後（使用者裁定納入）
**Open**: A 順序基底——拖排是否等於調整執行優先序（與修正二連動）；plan --json 的欄位形狀；UI 設計（卡片、系統匣、詳情面板、拖排違反依賴時的行為）；remote 的 plan 計算落點與 depends_on 寫入路徑；阻擋的硬度（硬擋或警告後可續）；先前討論 task-marker-ui-and-parallel-removal（任務層 [P]）與 worktree-parallel-apply（執行期偵測）不構成翻案，已記入對照

### Round 2 — interview (2026-09-15)

**Focus**: 拖排與引擎依賴順序的關係——拖排是否成為排程基底、有依賴時能不能亂拖
**Position**: 拖排成為排程基底（board_rank 取代建立日期），但拖排本身受依賴約束：無關的卡隨意排，有依賴或重疊的卡不得拖到前置之前或後繼之後。
- 三個生命週期欄（提案中／進行中／已就緒）一律照 plan 順序顯示；已就緒也照排，因封存順序才是合併閘在乎的
- 排程規則一句話：先照使用者拖的順序（rank），缺 rank 者以建立日期補位，再以 depends_on 與 delta 重疊做拓樸修正
- 拖排約束落兩層：引擎的 reorder 寫入路徑拒絕違反依賴的 rank（唯一真相）；桌面拖動時依 plan 的 depends_on／overlaps 把不合法落點變灰、不可放下
- plan --json 形狀定案：waves（波次陣列）＋changes（每卡 wave／stage／depends_on／overlaps／blocked_by／ready）＋next（第一個可開工者）；成環回錯誤並點名環
- UI：卡片進度列右側加波次圓圈數字章（同波同號＝可並行）、被擋顯示灰章「等 N 項」附 tooltip；詳情面板新增「排程」分頁（波次／前置可編輯／重疊唯讀）；系統匣列首加同一波次數字、被擋列變淡
- remote 納入：server 加唯讀端點 GET /plan（與 validate／analyze 同類），計算函式在 speclink-core 兩端共用；rank 輸入 local 讀 meta、remote 由 server 讀 board resource；depends_on 寫入走 meta 寫入端點模式（editor 限定）；CLI remote 模式轉呼叫 server
- apply 守門硬度：指名時 blocked_by 非空即硬擋並列出前置（使用者第一輪已接受阻擋）
**Ruled out**: 放下後彈回並提示（使用者裁定：有關聯就不該讓人亂調，應在落點就擋）；拖排只管顯示（三欄照 plan 排會讓拖排成廢功能）；建立日期當唯一基底（使用者要能手動排優先序）
**Open**: propose 寫入 depends_on 的動詞形狀；封存是否也依 plan 守門（軟依賴）；切刀方式

## Conclusion

**Decision**: 引入 change 層的執行順序（有限度排行）。順序規則：先照使用者拖排的 board_rank，缺 rank 者以建立日期補位，再以宣告依賴（depends_on）與 delta capability 重疊做拓樸修正。軟依賴存每張卡 .openspec.yaml 的 depends_on 欄位，propose 收尾由 AI 判定後以 `speclink change depends <name> --on <other>`（可重複、--remove 移除）寫入；硬信號（delta 重疊）不存、每次由引擎現算。新增唯讀動詞 `speclink plan --json`：waves（波次，同波可並行）＋changes（每卡 wave／stage／depends_on／overlaps／blocked_by／ready）＋next（第一個未開工且無阻擋者）；成環回錯誤並點名環。apply 指名與否都先查 plan：不指名取 next，指名且 blocked_by 非空即硬擋並列出前置。桌面三個生命週期欄（提案中／進行中／已就緒）與系統匣一律照 plan 順序排；拖排受依賴約束——無關的卡隨意排，有依賴或重疊的卡不得拖到前置之前或後繼之後（引擎 reorder 拒絕違反依賴的 rank 為唯一真相，桌面拖動時不合法落點變灰不可放下）。UI：卡片進度列右側加波次圓圈數字章（同波同號＝可並行）、被擋顯示灰章「等 N 項」附 tooltip 列前置；詳情面板新增「排程」分頁（波次／前置可編輯／重疊唯讀）；系統匣列首加同一波次數字、被擋列變淡。remote 一併納入：server 加唯讀端點 GET /plan（與 validate／analyze 同類），計算函式在 speclink-core 兩端共用，rank 輸入 local 讀卡片 meta、remote 由 server 讀 board resource；depends_on 寫入走 meta 寫入端點模式（editor 限定）；CLI remote 模式轉呼叫 server。封存不加新守門：硬信號已有合併閘，軟依賴只在封存技能收尾建議「plan 建議先封存 X」。分三刀：刀一 引擎與 CLI（depends_on 欄位、plan 與 depends 動詞、reorder 依賴拒絕、propose／apply／archive 三個技能改寫）；刀二 桌面與系統匣 local（三欄照 plan 排、波次章、排程分頁、拖動灰化落點）；刀三 remote（GET /plan、depends_on 寫入端點、桌面 remote 資料源、CLI remote 轉呼叫）。
**Rationale**: 順序判定今天已存在於 propose-skill 規格的「收尾盤點」，但只在 AI 腦中、不落檔，apply 也不看，每次重讀全部 change 燒 token。把硬信號交給引擎現算、軟信號落檔一次，之後 apply、看板、系統匣、remote 都零 token 讀同一份。拖排成為排程基底並受依賴約束，讓「人手排優先序」與「引擎守依賴」在同一個順序上並存，三欄顯示順序＝執行順序，不需第二套排序。
**Rejected alternatives**: 中央順序檔（平行 session／worktree 對撞）；把硬信號存檔（delta 一改即過期，還要失效機制）；塞進 list --json（動到逐位元凍結的輸出）；只在不指名時查 plan（指名會撞依賴，與使用者補充相反）；建立日期當唯一基底（使用者要能手動排優先序）；拖排只管顯示（三欄照 plan 排會讓拖排成廢功能）；放下後彈回並提示（使用者裁定有關聯就該在落點擋）；remote 延後（使用者裁定納入）；封存加軟依賴硬守門（程式碼已寫完，硬擋只逼人刪依賴）。先前討論 task-marker-ui-and-parallel-removal 否決的是任務層 [P]，worktree-parallel-apply 否決的是執行期偵測；本題是 change 層靜態依賴，「人決定是否並行」原則不變，plan 只建議。
**Deferred**: 波次章與灰化落點的確切視覺選型（實作時定）；封存的軟依賴守門（先不做）；plan 在 worktree 聚合讀下的 stage 判定細節（design 階段定）。
**Capture to**: proposal（三刀，本記錄 --hold，最後一刀帶 --last）；propose-skill 規格「收尾盤點提案中變更的執行順序」MODIFIED 為判定後寫入 depends_on 再跑 plan；board-card-order 規格欄內順序 MODIFIED 為 plan 順序、rank 為輸入且受依賴約束；新 capability：change-plan（引擎與動詞）、tray-status-menu／desktop-app 延伸（刀二）、server-verb-api 延伸（刀三）。
**Next**: /speclink-propose --from-discussion change-execution-order（刀一）
