---
topic: apply-with-worktree 接上 plan：守門搬到前段、在主 checkout 跑，並盤點其他流程缺口
slug: worktree-apply-plan-preflight
status: promoted
created: 2026-09-16
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: worktree-apply-plan-preflight
---

# Discussion: apply-with-worktree 接上 plan：守門搬到前段、在主 checkout 跑，並盤點其他流程缺口

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 plan-handoff-after-archive 封存後追問：apply 已接上 plan 守門，apply-with-worktree 呢？需求可驗證，未經 grill 直接進假設。
掃描結果：apply-with-worktree 由 apply-worktree-pre.md＋apply 本體＋apply-worktree-post.md 拼成（skills.rs B_APPLY_WITH_WORKTREE），plan 守門只在本體第 1 步，跑在前段 P2～P4（存在性檢查、產物 commit、建 worktree）之後、且依 P6 在 worktree 內執行；linked worktree 的 .git 是檔案，observed_facts 三道閘回空 overlay，worktree 內的 plan 只讀自己快照、看不到別的 worktree 進度與主線後續封存。前段沒給名字時沒有任何選 change 的步驟；P0 多名字配方不分可並行／要等。
相關規格：worktree-apply-skill、change-plan、skill-routing。相關變更：add-change-plan-engine（已封存）、add-plan-handoff-after-archive（已封存）、add-change-plan-remote（進行中，delta 含 change-plan）。
Prior discussions: plan-handoff-after-archive, change-execution-order, worktree-parallel-apply

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-16)

**Focus**: apply-with-worktree 的 plan 守門位置與執行地點——五條假設使用者全數確認
**Position**: 守門搬到前段、在主 checkout 跑、建 worktree 之前就擋，落點為獨立小 change。
- A 前段於 P1 政策檢查後加「以 plan 選 change 並守門」：未指名取 next；指名且 blockedBy 非空即停，不 commit 產物、不建 worktree（現況 P2～P4 都需名字、守門卻在 P4 之後才跑）
- B 主 checkout 是唯一正確判定地點（worktree.rs observed_facts：.git 非目錄即空 overlay）；本體第 1 步的 plan 在 worktree 內降為複查，結果不同以主 checkout 為準，寫法沿後段「本體 Next steps 在此不適用」的先例
- C P2 存在性檢查併入 plan 步驟：名字不在 changes（已封存、打錯、skipped）即停並列可用名單
- D P0 多名字先跑 plan：blockedBy 空者列並行配方、非空者說「等 X 落地」，使用者從並行名單擇一
- E 落點：獨立小 change，只動 apply-worktree-pre.md、delta 只落 worktree-apply-skill「前置指示」MODIFIED；不動 change-plan 規格（remote 刀正在改）；ASSET_VERSION 三連動
- 續作舊 worktree 的假阻擋（主線已封存的前置在快照裡仍活著）與 next 選到別的 worktree 正在做的 change，皆由 B 消解
**Ruled out**: 改引擎讓 worktree 內的 plan 跳到主 checkout 聚合（改 list 與 plan 共用閘門、範圍大、違反「worktree 內只讀自己副本」設計）；在 apply.md 本體加 worktree 分支條件（本體須與 apply 位元同源，會污染純 apply）
**Open**: 其他流程還有哪裡沒接上 plan（使用者要求盤點）

### Round 2 — assumptions (2026-09-16)

**Focus**: 盤點其他流程哪裡還沒接上 plan——逐技能掃 speclink plan／change depends 的消費點
**Position**: 只剩兩個真缺口（ingest 收尾不重判軟依賴、docs/workflow 與 README 流程圖沒有 plan），其餘流程已接上或不需接。
- 已接上：propose 收尾（depends＋plan 列波次）、apply 第 1 步守門、archive 3b 順序提示與尾段 next、commit 先封存子流程（今天封存的 add-plan-handoff-after-archive）；grep 全部技能資產只有這四份含 speclink plan
- 缺口 1 ingest：中途改需求可能讓本 change 新建立在別的提案成果上，ingest.md 沒有任何 depends 判定；硬信號（delta 重疊）plan 每次現算不需管，只有軟依賴的宣告會漏；ingest 出邊是 apply、守門會吃到新宣告，所以補的只是「收尾重判一次 depends」
- 缺口 2 文件：docs/workflow.zh-TW.md 與 README 兩語流程圖零提及 plan／depends；屬手寫文件，不在技能資產與手冊過期提醒範圍
- 不需接 worktree-merge：依賴與重疊的配對在 apply 守門就已序列化（被擋者開不了工），合併順序問題不會出現；硬衝突由 rebase／merge 衝突路徑擋
- 不需接 drift／review／verify／quality：出邊是 apply 或 archive，兩端已守門
- 不需接的引擎面：plan 基底順序 ready→in-progress→proposed（plan.rs 測試 basis_order），進行中的 change 不會被後來提案的重疊夥伴阻擋，續作不會假擋；唯宣告依賴是使用者手動寫的，被擋屬預期
**Ruled out**: worktree-merge 加 Plan order hint（守門在 apply 已序列化，提示無事可提）；drift 重跑 plan（apply 守門即吃到）
**Open**: 缺口 1 與缺口 2 是併進本 change 還是各自獨立（ingest 屬技能資產、同批三連動較省；文件改動不需版號）

### Round 3 — assumptions (2026-09-16)

**Focus**: 兩個缺口的落點——併入本 change 或獨立
**Position**: 缺口 1（ingest 重判軟依賴）併入本 change；缺口 2（docs/workflow 與 README 流程圖）使用者裁定暫不動。
- ingest 沒有自己的正典 capability（speclink list --specs 無 ingest-skill；ingest 只散見於 skill-routing 交棒表與 change-lifecycle 的 restale_from），delta 以新 capability ingest-skill ADDED 一條「收尾重判軟依賴」需求承載，規則沿 propose-skill 第 105 行同一套（讀其他提案 Impact、有則 change depends 落檔、硬信號交引擎、僅本 change 的出邊）
- 文件缺口：使用者指出 docs/workflow 兩語版與 README 流程圖已過時許久，本次不補，記為 Deferred
- 與進行中兩刀無 delta 重疊：add-change-plan-remote 動 change-plan／client-protocol／desktop-app／remote-board-order／server-verb-api／verb-contract，discuss-scout-in-flight-deltas 動 change-lifecycle／client-protocol／discuss-skill／server-verb-api；本 change 只落 worktree-apply-skill 與 ingest-skill
**Ruled out**: ingest 的 delta 塞進 skill-routing 交棒表（那張表只記出邊，判定規則放進去會失去單一職責）；文件缺口併入本 change（使用者裁定文件整體過時，零星補一句無意義）
**Open**: 無

## Conclusion

**Decision**: apply-with-worktree 接上 plan，一個獨立小 change 落地，只動技能資產。（1）apply-worktree-pre.md 於 P1 政策檢查之後新增「以 plan 選 change 並守門」步驟，在主 checkout 執行 speclink plan --json：未指名取 next（null 時列每個 change 的 blockedBy 並停）；指名者 blockedBy 非空即印前置清單並停，不 commit 產物、不建 worktree；名字不在 changes（已封存、打錯、skipped）即停並列可用名單，原 P2 的 list --json 存在性檢查併入此步。（2）P0 多名字時先跑 plan：blockedBy 空者列並行配方（各開 session 走 apply-with-worktree），非空者說明「等 X 落地再開」，使用者從並行名單擇一。（3）前段明寫：apply 本體第 1 步的 plan 在 worktree 內只是複查——linked worktree 的 .git 是檔案、overlay 為空、只讀自己快照——結果與前段不同時以主 checkout 的判定為準，寫法沿後段「本體 Next steps 在此不適用」的先例；apply.md 本體位元不動。（4）ingest.md 收尾（更新完 artifacts、給下一步建議之前）補一次軟依賴重判：沿 propose 收尾同一套規則，讀其他提案中 change 的 proposal Impact，本 change 建立在某 change 成果上即執行 speclink change depends <本 change> --on <前置>，硬信號交引擎，僅本 change、僅建議不代跑 apply。（5）canon deltas：worktree-apply-skill 規格 MODIFIED「apply-with-worktree 技能的前置指示」（插入 plan 步驟、改寫 (0) 多名字配方與 (2) 存在性檢查、加「本體複查以主 checkout 為準」）；新 capability ingest-skill ADDED「ingest 收尾重判軟依賴」。asset 異動走 ASSET_VERSION／golden／assets.lock 三連動，speclink update 再生 SKILL.md。
**Rationale**: apply-with-worktree 與 apply 同源，守門確實「自動帶到」，但帶到的位置在建 worktree 之後、執行地點在 worktree 內：被擋的 change 會先留下空 worktree 與分支，續作舊 worktree 時快照過期會假擋，next 會選到別的 worktree 正在做的 change。主 checkout 是唯一有聚合面的地方（observed_facts 三道閘），守門搬到前段在那裡跑即全部消解。ingest 是唯一會在中途改變依賴關係卻不重判的流程，補一次判定讓 apply 守門吃到新宣告。
**Rejected alternatives**: 改引擎讓 worktree 內的 plan 跳到主 checkout 聚合（改 list 與 plan 共用閘門、範圍大、違反 worktree 內只讀自己副本的設計）；在 apply.md 本體加 worktree 條件分支（本體須與 apply 位元同源）；worktree-merge 加 Plan order hint（依賴與重疊配對在 apply 守門已序列化，無事可提）；drift 重跑 plan（出邊 apply 已守門）；ingest 的 delta 塞進 skill-routing 交棒表（該表只記出邊）；文件缺口併入本 change（使用者裁定文件整體過時）。
**Deferred**: docs/workflow 兩語版與 README 流程圖補 plan（使用者裁定：文件已過時許久，另案整體處理）。
**Capture to**: proposal（新 change：worktree-apply-plan-preflight）；worktree-apply-skill MODIFIED、ingest-skill ADDED 兩份規格 delta。
**Next**: /speclink-propose --from-discussion worktree-apply-plan-preflight
