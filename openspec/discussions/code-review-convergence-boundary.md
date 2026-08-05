---
topic: 審查流程如何保證收斂，並釐清與驗證流程及 Matt Pocock code-review skill 的邊界
slug: code-review-convergence-boundary
status: promoted
promoted_to: converge-review-remediation-rounds, verify-station-parity
created: 2026-08-03
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 審查流程如何保證收斂，並釐清與驗證流程及 Matt Pocock code-review skill 的邊界

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者觀察到 2026-08-02-code-review-stage 的審查迴圈在每輪修正後仍產生新的 CRITICAL 與 SUGGESTION，除非人工停止，否則沒有可預期終點；同時詢問現行審查與驗證的分工，以及和 Matt Pocock code-review skill 的差異。本討論採 assumptions 模式：已找到 .agents/skills/speclink-review/SKILL.md、crates/speclink-core/src/review.rs、crates/speclink-cli/src/commands.rs 等 3 個以上相關來源，並讀取 canonical review-skill/review-station specs、封存 change design、verify skill 與外部 Matt Pocock skill。目標是先診斷現有終止契約，再裁定續輪 discovery 與 remediation 的邊界；本輪不實作。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-03)

**Focus**: 現行審查流程是否已有可保證收斂的終止契約，以及無限續輪的真正成因
**Position**: 現行設計有收斂緩解，但沒有確定終止條件：
- Matt Pocock skill 只做一次固定 diff 的 Standards／Spec 報告後結束，不包含修正、重審、工單或蓋章；Speclink 增加迴圈後才承擔收斂問題
- Speclink 審查的正典分工是 Standards（repo 慣例與 smell）＋Correctness（bug 獵捕）；change artifacts 僅供意圖脈絡，逐 requirement 合規歸驗證站，證據見 .agents/skills/speclink-review/SKILL.md 與 openspec/specs/review-skill/spec.md
- 封存 design D7b 已記錄同批檔案續輪產生新 possible-X 的棘輪效應與三輪 38 筆 findings，現行只用必修／可裁分類、全量建置測試門、accepted 不重報三項機制緩解
- 續輪仍讓兩軸重新掃描 findings 檔案加修正觸及檔，禁止的只有已接受事項及其近似變體；因此新的既存問題仍可不斷加入，零 findings 自動蓋章不具可達性保證
- 建議把 Round 1 定義為唯一 discovery round，後續改為 remediation validation：只驗證既有必修項是否修好與修正 delta 是否引入回歸，不再新增 smell／SUGGESTION；並新增 passed clean、passed with reservations、failed or deferred 三種明確終局
**Ruled out**: 以「一直重審直到模型找不到任何 finding」作為通過條件——生成式審查不具完備性，規格自身實測已證明不可收斂；把 Matt 的單輪結束誤認成它解決了修正迴圈——其流程根本未定義修正迴圈
**Open**: 晚發現的既存 CRITICAL 應立即重開 discovery、列為下一個 change，或僅在安全／資料損失等窄例外下允許阻斷；是否需要最大 remediation 輪數作硬性保險

### Round 2 — assumptions (2026-08-03)

**Focus**: 多輪審查的續輪究竟應審整個 finding 檔案，還是只驗收上一輪修正
**Position**: 續輪應改為修正驗收，只審上一輪未解 finding 與其修正差異，而非重新 discovery：
- 使用者校正：驗證側的工單與收尾迴圈正在 change verify-station-parity 定義；其 verify-skill delta 明定三維度不變並新增修正後重驗／接受蓋章／先不蓋，因此同一收斂規則也必須覆蓋該 change
- 審查續輪的輸入應為上一輪未解必修項＋主線為修正它們產生的 delta；輸出只判定原 finding 已解／未解，以及 delta 是否直接引入回歸
- 相鄰程式、呼叫端與測試可作為因果判斷證據，但未修改區域不得因此成為新 smell 或 SUGGESTION 的 discovery 面；晚出的 blocking finding 須能直接連到修正 delta，安全／資料損失等窄例外另行裁定
- 現行 review skill 以末輪 finding 的 path 加 fix-touched path 定界，工單只存 Scope 路徑與 findings；touched record 也只提供檔案歸屬，沒有逐輪修正前內容，因此目前實際是重審整檔，無法跨 session 精準重建修正 diff
- 最小修正可先在 skill 層定義 follow-up validation 模式：同 session 使用主線剛產生的 patch，跨 session 以 finding 的 symbol／line／trigger 錨點驗收目前程式，不重新 discovery；若之後證明需要位元級 delta 再擴工單，不先增加完整程式快照
**Ruled out**: 續輪重新掃描上一輪 findings 涉及的整份檔案——同檔未改區域仍可無限產生新問題；只看變更行而不驗證原 finding——可能修了表象卻未恢復呼叫端或系統 invariant
**Open**: 是否將同一個「首輪 discovery、續輪 validation」契約同時納入 review-skill 新變更與進行中的 verify-station-parity；安全例外與無進展終止條件的精確文字

### Round 3 — assumptions (2026-08-03)

**Focus**: 「首輪探索、續輪只驗收修正」是否同時適用審查與驗證兩個品質站
**Position**: 使用者確認兩站採同一個續輪契約：
- 審查首輪執行 Standards＋Correctness 的完整 discovery；續輪只驗收上一輪未解 finding 與為修正它產生的 delta，並檢查該 delta 直接引入的回歸
- 驗證首輪執行完整／正確／一致三維度；續輪只重驗上一輪未解的 requirements／scenarios／design findings 與修正 delta，不從頭重跑整份規格探索新事項
- 兩站的相鄰程式、呼叫端、測試與相關 requirement 可作為因果證據，但不得擴成未修改區域的新 discovery 面
- 此決策需進兩個落點：審查側以新 change 修改 canonical review-skill；驗證側在實作前併入進行中的 verify-station-parity，其目前 0/19 任務、仍有完整 ingest 空間
**Ruled out**: 審查採續輪驗收、驗證仍每輪完整重跑——兩站將保留不同的收斂模型，驗證站同樣可能無限產生新 findings
**Open**: 續輪偶然看見與修正 delta 無關、但疑似安全／資料損失級問題時，是直接阻斷本輪，還是另立 finding／change 而不重開本輪 discovery；無進展時何時宣告未通過

### Round 4 — assumptions (2026-08-03)

**Focus**: 續輪偶然發現與修正 delta 無關、但可能屬重大問題時如何兼顧安全與收斂
**Position**: 使用者採納重大晚發問題的窄例外，處理方式是終止並重開邊界：
- 與修正 delta 無關的新問題原則上不加入目前續輪；證據不足者列為後續事項，不阻斷本輪
- 若問題具有現實觸發路徑，並有重現方式、失敗測試或明確 invariant 破壞，且影響為安全漏洞、資料損失或錯誤行為，品質站不得忽略
- 符合重大門檻時，目前品質站以未通過／範圍已改變結束，保留工單且不蓋章；另開新 discovery 或衍生 change，不把問題塞回原續輪繼續擴張
- 同一規則適用審查與驗證，讓「不得重新 discovery」保有安全逃生口，但逃生口本身不破壞終止性
**Ruled out**: 一律忽略與 delta 無關的重大問題——會以收斂為名隱藏風險；直接加入原工單繼續修——重新打開無界 discovery，回到無限迴圈
**Open**: 修正後必修集合沒有縮小、同一 finding 重現，或驗證門仍失敗時，應在第一個無進展輪立即宣告未通過，還是容許固定次數重試

### Round 5 — assumptions (2026-08-03)

**Focus**: 以未解必修集合是否縮小作為終止條件，是否違反審查的宗旨
**Position**: 不應把集合縮小當成通過標準，只能把它當成自動迴圈的續跑條件：
- 審查目的仍是對有限範圍做獨立工藝與缺陷檢查、揭露風險、驗收修正並給出明確裁決；目的不是保證再也找不到 bug，也不是讓修正器無限工作
- 若必修集合沒有縮小，流程 SHALL 停止自動重試，但結果為未通過：工單保留、不得蓋章、不得封存成已審查；使用者可回實作階段換方法後再明示重啟驗收
- 因此「審查結束」與「審查通過」必須分離；現行 review station 已有未結工單阻擋正常蓋章與封存的 fail-closed 機制，可承載未通過結局
- 嚴格集合縮小若被用成品質度量會過度機械化：同一 finding 可能因修法失敗而仍在，這表示需要停止並升級判斷，不表示風險已被接受
- 修正後新重大回歸依前輪裁定直接終止為未通過／範圍改變；只剩可裁項時才可由使用者明示 --accept 帶保留通過
**Ruled out**: 未解集合不縮小仍自動蓋章——把收斂凌駕品質；未解集合不縮小仍自動繼續修——重建無限迴圈；用 finding 數量代表程式品質——數量只是流程進展訊號，不是風險本身
**Open**: 使用者是否接受重述後的規則：集合縮小＝允許自動續跑，集合不縮小＝未通過並停下，而非通過；確認後可收斂結論並決定如何分別落入新 review change 與既有 verify-station-parity

### Round 6 — assumptions (2026-08-03)

**Focus**: 收斂後的品質站流程是否仍保有 Matt Pocock code-review skill 的核心審查形狀
**Position**: 可以符合核心精神，但需把 Matt 式首輪審查與 Speclink 的修正編排明確分層：
- Matt 的核心不是無限修到零 findings，而是固定 change diff、Standards／Spec 兩軸隔離的平行 sub-agent、repo 標準優先且 smell 僅為判斷題、兩軸報告並列且不合併重排，報告完成即結束該次 review
- Speclink 應讓首輪成為 Matt-compatible review pass：凍結 change delta／snapshot，Standards＋Correctness 平行且報告原樣分軸；目前 touched 只給檔案集合，若 sub-agent 重審整檔仍不夠像 Matt，應以 touched 做歸屬過濾但實際判斷 change hunks
- Matt 的 Spec 軸由 Speclink 驗證站承接，因此單跑審查只等於 Matt 的工藝面加額外 bug 獵捕；需要完整 Matt 式結果時，審查與驗證應針對同一凍結 snapshot 各自輸出，不互相遮蔽
- 必修／可裁分類只能置於兩軸原報告之後，明示為 remediation routing，不得改寫 severity、合併兩軸或產生單一總分；否則違反 Matt 不 merge／rerank 的原則
- Round 2+ 應稱為 validation pass：只驗收原 finding 與修正 delta，不再冒充一次新的完整 code review；工單、蓋章、未通過與收斂規則都是 Speclink 對 Matt 單輪報告之後的生命週期擴充，不改動首輪 reviewer cognition
**Ruled out**: 原樣把 Matt Standards＋Spec 兩軸都塞回審查站——與既有驗證站重工；把每一續輪都當全新 Matt review——每輪重開 discovery；以必修／可裁清單取代兩軸原報告——重新混合並遮蔽軸別結果
**Open**: 使用者是否接受「首輪 Matt-compatible review pass＋後續 Speclink remediation／validation lifecycle」作為最後形狀；接受後結論分流為新 review change 與既有 verify-station-parity ingest

### Round 7 — assumptions (2026-08-03)

**Focus**: 精確 change hunks 應由 touched、apply 或 review 哪一層負責，以及能否以檔名後加行數表示
**Position**: 使用者接受「首輪 Matt-compatible review pass＋後續 Speclink remediation／validation lifecycle」；精確差異另採分層邊界：
- 現行 touched 不是逐次 patch provenance：task done 只把尚未被先前任務認領的新髒檔整檔記入，同一檔案後續任務再改不會新增 entry；證據為 crates/speclink-core/src/tasks.rs 的 complete 先以 all_files 排除 seen paths，以及 openspec/specs/verify-evidence/spec.md 明定「未被先前任務認領的新髒檔」
- 不在 touched files 字串後附行數。行號會因插入、刪除與 formatter 漂移，無法表示刪除行、改名、binary 或同檔多段 hunk；更重要的是 hunk 必須錨定一組 before／after，單獨座標不能重建差異
- touched 的既有深介面是 change 級檔案歸屬與 archive trace／commit／drift 的共同來源，檔案清單語意已有向下相容契約；應保留作候選檔案過濾器，不把 `path:line` 或完整 patch 混入既有 files 欄位
- Round 1 若要求精確自動歸屬，來源必須早於修改：由 apply／workspace 端建立可信 before baseline，完成後產生凍結的 change delta；Round 2+ 則由 review 在修正前凍結 reviewed snapshot、修正後產生 remediation delta。review 不應事後從整個髒工作樹猜意圖
- 若要跨 session／remote 重建，另設版本化、結構化 ChangeDelta（至少 path、before hash、after hash、hunk old/new ranges 與可重建的 patch 或 snapshot reference）；座標只有在 hashes 對得上時有效。現行 remote TaskDoneRequest 雖有 touchedFiles，但 server route 未把 body 接入 Command，證明此能力必須由單一 Host／Apply adapter 端到端落地，不能只改本地 JSON
- interface depth check：seam 位於能同時看見修改前後 workspace 的 Host／Apply 邊界；local／remote 共用一個 ChangeDelta 契約；其價值是凍結可重建差異而非轉送檔名；刪掉它會使首輪退回整檔重掃，故抽象成立
- dirty worktree 中若 baseline 前已有同檔無關修改，或兩個 active changes 在 baseline 後同改一檔，單靠 snapshot 仍無法判斷意圖；沒有隔離 worktree 或逐 patch instrumentation 時必須 fail closed，要求明示 git fixed point／排除 hunks，不得靜默認領
**Ruled out**: `file:start-end` 作 touched 新格式——不穩定且破壞既有 consumer；只在 task done 才擷取 `git diff`——取得的是對 HEAD 的累積髒差異，混入前任務與既存修改；把完整 unified diff 直接塞進 touched 既有 entries——放大本地檔、remote 傳輸與相容成本，且仍未解決 before baseline
**Open**: 建議分兩刀：本次 review 收斂 change 先要求可信 fixed point、首輪只審該 diff hunks，續輪由 review 凍結修正前後 delta；另開 apply-provenance change 才建立跨 session／remote 的結構化 ChangeDelta。待使用者裁定是否採此分刀，或要求本次 review change 一併承擔 apply 端 provenance

### Round 8 — assumptions (2026-08-03)

**Focus**: 是否接受分刀收斂，以及 apply provenance 的 hunk 座標契約
**Position**: 使用者確認討論可收斂，採 review 收斂、verify 既有變更更新、apply provenance 獨立變更三路落點：
- 審查首輪維持 Matt-compatible discovery pass，針對可信 fixed point 的 change hunks；續輪是 remediation validation，只驗收上一輪未解 finding 與修正 delta 直接造成的回歸
- 驗證站採同一首輪 discovery／續輪 validation／無進展即未通過停止的收斂契約，決策併入既有 verify-station-parity
- apply provenance 另立 change，從修改前捕捉 change baseline，提供首輪審查與驗證共用的結構化 ChangeDelta；touchedFiles 繼續只承擔檔案歸屬
- 每個文字 hunk SHALL 明確包含 `oldStart`、`oldLines`、`newStart`、`newLines`，且綁定該檔同一組 before／after content hashes；建立內容可有 oldLines=0，刪除內容可有 newLines=0，多段修改保留多筆 hunk
- old/new ranges 是必備可觀察契約但不是單獨的真相來源；hash 不符時座標 SHALL 判 stale，不得套用到漂移後檔案。可重建 patch 或 snapshot reference 仍須存在
- 差異演算法、context 合併規則，以及 ChangeDelta 實體放在 touched v3 獨立欄位或另一個可遠端傳輸的 store attachment，交由 apply-provenance change 的 design 決定；local／remote SHALL 共用同一契約
- 第一個無進展輪立即停止為未通過，不另設固定最大輪數；與修正 delta 無關的新問題不加入續輪，只有具現實觸發與重現證據的安全、資料損失或錯誤行為可使本站以範圍改變／未通過結束並另開 discovery
**Ruled out**: 本次 review change 一併實作 apply provenance——會把收斂修正膨脹成跨 local／remote 的 workspace 來源追蹤；old/new 行數沒有 hashes 或 snapshot／patch anchor——形成會漂移的假精確；固定重試 N 輪——無法反映實際進展且延後已知失敗
**Open**: 無產品決策未解；apply provenance 的差異演算法、hunk context 合併與實體儲存布局明示留給該 change 的 design，不阻擋本討論結論

## Conclusion

**Decision**: 審查與驗證採有限的「首輪探索、續輪修正驗收」生命週期，並分成審查收斂、既有驗證更新、apply provenance 三個落點：
- 審查 Round 1 凍結可信 change snapshot／delta，執行 Standards＋Correctness 兩軸完整 discovery；兩軸報告原樣並列，不合併、不跨軸重排，triage 僅在報告後作修正路由
- 驗證 Round 1 對同一凍結 snapshot 執行完整／正確／一致三維度；審查不重做 spec 合規，驗證不取代工藝與 bug 獵捕
- 兩站 Round 2+ 都是 validation pass：只驗收上一輪未解 finding 與修正 delta，並檢查該 delta 直接引入的回歸；相鄰呼叫端、測試、規格可作因果證據，但不得擴成未修改區域的新 discovery
- 只要未解必修集合縮小即可自動續跑；第一個無進展輪立即停止為未通過、保留工單且不蓋章，不設固定最大輪數。只剩可裁事項時可由使用者明示接受並帶保留蓋章
- 與修正 delta 無關的新問題原則上不加入續輪；若有現實觸發路徑與重現／失敗測試／明確 invariant 證據，且影響安全、資料損失或錯誤行為，本站以未通過／範圍改變結束，另開 discovery 或衍生 change
- touchedFiles 保持 change 級檔案歸屬與既有 consumer 相容；精確差異由獨立、版本化 ChangeDelta 承擔。Round 1 的 before baseline 由 apply／workspace 在修改前捕捉，Round 2+ 的 remediation delta 由品質站在修正前後凍結
- apply provenance 的每個文字 hunk SHALL 包含 `oldStart`、`oldLines`、`newStart`、`newLines`，綁定同一組 before／after content hashes，並保有可重建 patch 或 snapshot reference；新增可用 oldLines=0，刪除可用 newLines=0，多段修改保留多筆 hunk。hash 不符即 stale，禁止把座標套到漂移內容
- dirty worktree 若有同檔既存修改或多個 active changes 重疊，無法可靠歸屬時 SHALL fail closed，要求可信 git fixed point、明示排除 hunks、隔離 worktree 或逐 patch provenance，不得靜默認領
**Rationale**: Matt Pocock 式審查的核心是對固定 diff 做一次獨立分軸 discovery 並完整呈現報告；Speclink 可在其後增加工單、修正與蓋章，但必須把後續明確降為有限範圍的 validation。ChangeDelta 以 before／after anchors 與 old/new ranges 提供精確範圍，避免 touched 檔案清單造成整檔重掃，同時不破壞 commit、archive、drift 等既有消費者。
**Rejected alternatives**:
- 每輪重新掃描 finding 所在整檔直到零 findings：生成式 discovery 無完備終點，會重建無限迴圈
- 以 blocking set 縮小作通過標準：數量只能決定是否續跑，不能取代風險裁決
- 固定最多 N 輪：會讓已知無進展多跑，也可能截斷仍在實質改善的修正
- 將 Matt 的 Spec 軸塞回審查站：與 Speclink 驗證站重工並模糊責任
- 把行數接在 touched path 後或只存無 anchor 的 hunk ranges：座標會漂移且破壞既有檔案清單契約
- 本次審查收斂 change 同時承擔 apply provenance：會把流程修正膨脹成跨 local／remote 的 workspace 來源追蹤
**Deferred**: 無產品決策未解。apply provenance 的差異演算法與版本、hunk context 合併規則、patch 或 snapshot 保留方式，以及 ChangeDelta 實體位於 touched v3 獨立欄位或可遠端傳輸的 store attachment，交由該 change 的 design 裁定；不得弱化已定 old/new ranges、hash anchors、可重建性與 local／remote 同契約。
**Capture to**:
- 新 change `converge-review-remediation-rounds` 的 proposal／design／review-skill spec／tasks
- 既有 change `verify-station-parity` 的 design／verify-skill spec／tasks，經 discussion link 後 ingest
- 新 change `capture-apply-change-provenance` 的 proposal／design／spec／tasks
**Next**: 先將討論分別轉為 `converge-review-remediation-rounds` 與 `capture-apply-change-provenance`，再 link 至 `verify-station-parity` 並以 `$speclink-ingest` 納入既有 artifacts；三者完成 formalization 後才進入 apply。
