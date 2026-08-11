---
topic: review/verify 是否要求任務全勾?手動測試任務想排在 review 之後的守門調整
slug: review-before-manual-test-tasks
status: promoted
promoted_to: manual-task-marker-gates
created: 2026-08-10
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: review/verify 是否要求任務全勾?手動測試任務想排在 review 之後的守門調整

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因:使用者的 change(keep-lists-intact-across-imported-images)有一條手動驗證任務(3.2 開檔實測匯入結果),想先跑 review 修完問題再手測,但 review 技能的 Step 2 守門與 quality 的轉述在 9/10 時直接擋下。

模式:assumptions(codebase scout 找到 crates/speclink-core/src/station.rs、review.rs、verify.rs 與 speclink-review/verify/quality 三個 SKILL.md,證據足夠先列假設)。

現況地圖(已驗證):review 的 add-round 引擎刻意不擋(review.rs:33「守門留在技能層」)、stamp 兩站引擎都擋(任務全勾才蓋章)、verify 的 add-round 引擎就擋(verify.rs:32,工單語意=成品驗證)、review 技能 Step 2 擋開跑、quality 不自帶守門只轉述。review 技能已內建「蓋章被拒→條件恢復後直接重試蓋章」路徑。

相關 changes:speclink list 目前無進行中 change;無既有開放討論。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-10)

**Focus**: 守門落點盤點,以及「先審後手測」該在哪一層解
**Position**: 引擎其實已留路——review 落工單不擋、只有蓋章擋;唯一硬擋點是 review 技能 Step 2。原提四條假設:
- 先審後手測的動機成立(review 修碼會作廢手測,手測最貴應排最後)
- 手測任務留在 tasks.md(封存守門 lifecycle.rs:175 的強制力)
- 解法動技能層即可:Step 2 放行「剩餘全為手動驗證任務」情境,蓋章照舊等全勾後重試(引擎註解明說守門刻意留技能層)
- verify 側不動(工單語意=成品驗證是封存守門的根;9/10 時本就有口頭盤點路徑)
- 使用者修正第三條:希望從引擎端解——tasks.md 建立時就給手動測試任務特殊標記,讓後續判斷有正典依據,不靠 agent 讀文字猜
**Ruled out**: 先手測再 review(手測會被 review 後的修碼作廢,白做風險高);把手測搬出 tasks.md(失去「沒手測就封存不了」的強制力)
**Open**: 標記的語法與解析落點;哪些守門消費這個標記(review 開跑/verify 落工單/蓋章);蓋章是否豁免手動任務;蓋章後補勾任務是否觸發 stamp 失效(freshness 錨的行為)

### Round 2 — assumptions (2026-08-10)

**Focus**: 引擎端任務標記路線的可行性探查——解析器落點與蓋章失效錨的行為
**Position**: 使用者裁定走引擎標記路線(tasks.md 建立時就標記手動測試任務,判斷正典化、不靠 agent 讀文字猜);探查結果支持,但劃出一條硬界線:
- 解析器有現成先例:`[P]` 平行標記(tasks.rs:111 前綴解析進 Task.parallel),手動標記照抄同模式(如 `[M]` → Task.manual),解析成本低
- 關鍵事實:失效判定 station.rs:509 要求「當前完成數＝總數」,蓋章若落在 9/10 會立刻 Stale——因此蓋章不可豁免手動任務,除非重寫 freshness 錨(深層手術,不建議)
- 導出的語意界線:標記只影響「幾時能開審/落工單」(review 技能 Step 2、verify 引擎 add-round 守門),永不影響「幾時能蓋章/封存」——stamp 照舊等全勾,freshness 與 archive 守門零改動
- 波及面:tasks 解析器、instructions payload 的進度欄位(review 技能靠它讀守門)、verify 站 add-round 守門、propose/apply/review/verify/quality 技能文字、spec 正典＋golden＋assets.lock 三連動;GUI 呈現可後延
**Open**: 使用者是否接受「手測仍須在蓋章前完成」的界線;標記語法定案(`[M]` 或其他);verify 站是否與 review 同步放行;GUI 是否需要顯示手動標記

### Round 3 — interview (2026-08-10)

**Focus**: [P] 標記的實際消費現況、章失效判定(freshness)的接線現況——蓋章語意改「驗證過」是否成立
**Position**: 使用者提議蓋章也只看寫碼任務(章=驗證過,不是成品定案),理由是手測後改碼會被偵測到、要求重驗;探查結果翻轉了前一輪的硬界線:
- [P] 平行標記:有解析、有上 wire(instructions.rs:215、query.rs:292 逐任務 parallel 欄位),但技能文字與 GUI 都沒消費——半接線的休眠慣例
- freshness 失效判定(station.rs:497)在正式碼零消費者,只有測試在呼叫;章錨欄位(reviewed_scope/reviewed_tasks_total)有寫無讀——前一輪「9/10 蓋章立刻 Stale=白蓋」實際上無強制力
- 錨語意趁未接線時改最便宜:任務錨改成「寫碼任務全完成+總數不變」,勾手測任務不作廢章、改碼(內容錨)才作廢
- 修正後方案:三道守門(開審/落工單/蓋章)統一預測子「寫碼任務全完成」;封存維持「全任務完成」(手測強制力保留)
- 誠實警告:「改碼會被偵測到」今天只在重進 review/verify 站時由 scope 機制發生;引擎沒有「封存前檢查章失效」的守門,使用者模型的閉環需把 freshness 接上封存守門才真正閉起來(這個洞今天對一般蓋章後改碼也存在)
**Ruled out**: 前一輪「蓋章必等全任務勾完」的硬界線——依據(freshness 會判 Stale)實況無強制力,且錨語意可趁未接線改掉
**Open**: freshness 是否這次就接上封存守門(閉環 vs 範圍膨脹);標記語法定案(建議 [M]);apply 技能對 [M] 任務的處理(建議留給使用者勾);GUI 顯示(建議後延)

### Round 4 — interview (2026-08-10)

**Focus**: freshness 接線的裁定,與剩餘小項的預設收攏
**Position**: 使用者裁定 freshness 這次就接上封存守門——有章且已失效(蓋章後改過碼)即拒絕封存,「章=驗證過」模型完整閉環:
- 守門只在「章存在」時生效:無章照舊可封存(review/verify 站本就是選跑),Unknown(章欄位不全)視同無章
- 章上記錄的欄位形狀不變(reviewed/verified_tasks_total 仍記全任務總數),只有判定式改 manual-aware——舊資料相容
- 小項預設:標記語法 [M](循 [P] 前綴模式);apply 技能不代勾 [M] 任務、寫碼任務全勾即回報完成;GUI 顯示後延;[P] 休眠慣例不在此次處理
**Ruled out**: freshness 接成警告不擋(閉環靠自律,「驗證過」的章可能過期);先不接留下個 change(重驗理由暫時只剩技能層 scope 機制,強制力缺角)
**Open**: 無——進入結論

## Conclusion

**Decision**: tasks.md 引入手動測試任務標記 `[M]`(循 `[P]` 前綴模式,解析為 Task.manual 並隨 instructions/query payload 上 wire);三道守門——review 技能開跑、verify 引擎落工單、兩站引擎蓋章——統一改用同一預測子「寫碼(非 [M])任務全完成」;章的語意定為「驗證過(可驗的部分)」,封存語意承接「成品定案」:封存守門維持全任務完成(含 [M],手測強制力保留),並新接 freshness 失效判定——有章且已失效(蓋章後改過碼)即拒絕封存;freshness 任務錨改 manual-aware(勾 [M] 任務不作廢章;任務總數變動或 scope 檔內容變動作廢)。
**Rationale**: 手測最貴且最怕被 review 後的修碼作廢,應排在所有改碼活動之後;現行守門用「任務全勾」當「實作完成」的代理指標,[M] 任務戳破這個代理——標記讓引擎能區分「代碼完成」與「驗收完成」。放寬蓋章的正當性(改碼會被偵測、要求重驗)靠 freshness 接上封存守門取得引擎強制力;此洞今天對一般蓋章後改碼也存在,一併補上。改動時機好:freshness 與 [P] 同為「已實作、零正式消費者」的休眠縫,錨語意趁未接線改最便宜。
**Rejected alternatives**: 先手測再 review(修碼作廢手測,白做);把手測搬出 tasks.md(失去「沒手測就封不了」的強制力);只動技能層由 agent 讀任務文字判斷(不正典、判斷不可靠——使用者點名要引擎判斷);蓋章必等全任務勾完的硬界線(其依據 freshness 判 Stale 實際未接線、無強制力);freshness 接成警告不擋(「驗證過」的章可能悄悄過期)。
**Deferred**: GUI 顯示 [M] 標記與手動/寫碼任務進度拆分;apply 技能對 [M] 的處理細節僅定原則(agent 不代勾、寫碼任務全勾即回報完成);[P] 休眠慣例是否啟用不在此次。
**Capture to**: proposal(新 change:解析器、station 守門、freshness 錨、封存守門、instructions/query payload、propose/apply/review/verify/quality 技能文字、spec 正典+golden+assets.lock 三連動)
**Next**: /speclink-propose --from-discussion review-before-manual-test-tasks
