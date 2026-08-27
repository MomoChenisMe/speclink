---
topic: remote 協作四缺口修正計畫的未討論縫盤點與分刀
slug: remote-fix-plan-gaps
status: promoted
promoted_to: fix-discuss-section-anchor, remote-evidence-scope-wiring, stamp-contract-trace-docs
created: 2026-08-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: remote 協作四缺口修正計畫的未討論縫盤點與分刀

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

外部 AI 協作測試報告指出 remote 協作四缺口（discuss 輪錯序、task evidence 未接 quality scope、蓋章後工單讀不回、remote trace 拒絕）與 onboarding 摩擦。深度查證（4 subagent）定性：缺口 1 為共用引擎 bug 非 remote 專屬（discuss.rs 章節掃描把內容裡的「## 」行誤認為結構）、缺口 2 屬實且無任何立案覆蓋（station.rs:648-661 remote 分支讀本地 FsStore，remote-task-evidence 的 Non-Goals 未列此項）、缺口 3 為正典明定契約非 bug（蓋章原子刪工單，review-station spec:150）、缺口 4 屬實且為明訂 v1 Non-Goal；onboarding 文件縫已由 remote-docs-refresh task 2.3 修畢。本討論盤點修正順序未覆蓋的節點並定分刀。目標可驗證，無需 grill，直接假設清單。相關 specs：discussion-docs、change-diff-scope、review-station、verify-station、trace-verb、server-identity、user-documentation。相關封存 change：2026-08-25-remote-task-evidence、2026-08-27-remote-docs-refresh。前議 remote-remaining-gaps（已封存）涵蓋另一批縫，零重疊。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-27)

**Focus**: 四缺口修正順序沒覆蓋的節點清單與處置假設
**Position**: 七項假設鋪出缺口地圖（A 修法形狀、B 壞資料、C 多人語意、D stamp 決策選項、E 測試面、F 分刀歸屬、G 版號尾巴）：
- A：錯序修法定為結構錨點白名單，四個共病點同刀修——add_round 插入（discuss.rs:303-327）、conclude／set_context 的 replace_section（:83-92，偽結論會餵進 promote）、count_rounds 計數（:55-60）、UI splitDiscussionSections（DiscussionDrawer.tsx:28-47）
- B：不做壞資料修復工具（remote-task-evidence Non-Goals「不回填、不追溯」先例），前提＝損壞僅存在測試 server
- C：evidence→scope 接線前先裁三個語意——other_claims 併行認領守門、多 actor touched 是否聯集、head_commit 在審查端未 fetch 時的行為；正典衝突：change-diff-scope spec:226-236 明文要求讀 local checkout，修即改 SHALL
- D：stamp 後工單文字遺失押「接受＋文件明講」；review-station:170 與 verify-station:155 的「SUGGESTION 留在 git 歷史」承諾在 remote 不可成立，須一併修訂
- E：測試面只補個案回歸不立通案紀律；手塞 touched 的測試（remote_verb_parity.rs:1176-1185）改走真實鏈，discussion-docs:503 的 pure-append 要求補釘住 scenario
- F：缺口 1、2 各一刀平行；缺口 3、4 的文件面塞進行中的 remote-docs-refresh task 3.1
- G：缺口 2 若動 assets/skills/review.md:47 與 verify.md:58 的手動逃生敘述，MARKER_VERSION／golden／assets.lock 三連動與 32 份 SKILL.md 再生要計入工作量
**Ruled out**: 續用 remote-remaining-gaps 記錄——它涵蓋另一批縫（capability／promotedTo／claim／docs 總整理），與本題零重疊
**Open**: 七項假設待使用者逐項裁定；F 的塞刀方案需與進行中 session 協調

### Round 2 — interview (2026-08-27)

**Focus**: remote-docs-refresh 封存（2026-08-27）後，假設 F 是否仍成立
**Position**: F 改判——塞刀選項消滅，改立獨立收尾小刀；其餘六項假設不動：
- 事實：看板現在零進行中 change，平行 session 顧慮消失，缺口 1、2 可即開刀
- 查證：docs 刀的 21 檔盤點漏了三處文件縫，封存後仍在——remote-getting-started.md:85 仍寫 404（程式碼與 server-identity spec:328 為 403）、verb-contract.md:23 的 FsOnly 仍只列 demo、workflow 雙語仍未講蓋章消耗工單
- 新分刀：刀 1 discuss 結構錨點（撞 discussion-docs）、刀 2 evidence→scope 接線（撞 change-diff-scope）、刀 3 stamp 契約與 trace 正名收尾（workflow 補寫＋review／verify-station git 歷史條款修訂＋verb-contract 補 trace＋403/404 修正＋trace 拒絕釘住測試與 main.rs:168 過期註解）；三刀 spec 組互斥，可平行
**Ruled out**: 「缺口 3、4 文件面併入 remote-docs-refresh」——該刀已封存，且其逐檔盤點未抓到這三縫
**Open**: 三刀分法待裁定；刀 3 是否含 review／verify-station 條款修訂、或縮為純文件刀；其餘六項假設（A–E、G）待逐項裁定

### Round 3 — interview (2026-08-27)

**Focus**: 三刀分法與其餘六項假設的使用者裁定
**Position**: 全數確認，開放問題清空：
- 刀 3 裁定全包——workflow 雙語補寫、review-station:170 與 verify-station:155 條款修訂、verb-contract 補 trace、403/404 修正、trace 拒絕釘住測試與 main.rs:168 註解，同刀一次對齊
- A–E、G 六項假設照案通過，無修改
**Ruled out**: 刀 3 縮為純文件刀（條款修訂併刀 2）——使用者裁定文件與條款同刀對齊
**Open**: 無——進入結論

## Conclusion

**Decision**: 四缺口修正計畫的未討論縫以三刀處置，三刀 spec 組互斥、可平行開：
1. 刀 1（discuss 結構錨點）：章節掃描改結構錨點白名單，四個共病點同刀修——add_round 插入（discuss.rs:303-327）、conclude／set_context 的 replace_section（:83-92）、count_rounds（:55-60）、UI splitDiscussionSections（DiscussionDrawer.tsx:28-47）；discussion-docs:503 的 pure-append 要求補釘住 scenario；不做壞資料修復工具，已損壞紀錄人工重建
2. 刀 2（evidence→scope 接線）：remote 分支改讀 server 端 change_evidence（照 drift 既有模式，斷點 station.rs:648-661）；change-diff-scope spec:226-236 的 local-checkout SHALL 同刀修訂；手塞 touched 的測試（remote_verb_parity.rs:1176-1185）改走真實鏈；三個多人語意（other_claims 守門、多 actor 聯集、head_commit 未 fetch）於本刀 propose／design 階段定案；若動 assets/skills 敘述，MARKER_VERSION／golden／assets.lock 三連動計入工作量
3. 刀 3（stamp 契約與 trace 正名，全包）：workflow 雙語補「蓋章原子刪工單」、review-station:170 與 verify-station:155 的 git 歷史條款修訂（remote 不可成立）、verb-contract 補 trace 於 FsOnly、remote-getting-started 403/404 修正、trace 拒絕釘住測試與 main.rs:168 過期註解
另：外部報告的 onboarding 項已由 remote-docs-refresh task 2.3 修畢，不立案；remote trace 功能本體維持 v1 Non-Goal 的 backlog，本波只做正名
**Rationale**: 修正順序定了「修什麼」但沒定「怎麼修、誰家的刀」；remote-docs-refresh 封存後塞刀選項消滅，且其盤點漏了三處文件縫，證明這些縫需要自己的刀。分刀依「同組 spec 的 delta 併刀、互斥組平行」排列，避免版號對撞
**Rejected alternatives**: 文件面併入 remote-docs-refresh（已封存且盤點漏縫）；壞資料修復工具（remote-task-evidence「不回填、不追溯」先例）；stamp 工單內文折進封存產物或 history 存內文（正典定位工單為工作文件，結論已在 stamps；稽核需求出現再另議）；測試通案紀律入正典（只補個案回歸）；刀 3 縮為純文件刀（使用者裁定條款與文件同刀對齊）；續用 remote-remaining-gaps 記錄（另一批縫，零重疊）
**Deferred**: 刀 1 錨點機制細節（白名單 vs 結構註解）歸刀 1 design；刀 2 三個多人語意的具體裁定歸刀 2 design；「損壞僅存在測試 server」前提於刀 1 開刀前確認一次，若正式環境有損壞再議修復手段
**Capture to**: proposal（三刀依此立案）
**Next**: /speclink-propose --from-discussion remote-fix-plan-gaps（一份討論扇出三刀）
