---
topic: 兩站蓋章後互相打黃：review 與 verify 的修正時序與凍結失效互動
slug: cross-station-staleness
status: promoted
promoted_to: verify-station-parity
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 兩站蓋章後互相打黃：review 與 verify 的修正時序與凍結失效互動

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：change evidence-home-and-trace-slim 實測「review 蓋章 → verify → 修 3 件 findings」後，審查章在看板轉為「已審查·其後有變動」（琥珀）。使用者確認核心問題：兩站serial 執行時，任一站的修正都會打黃另一站已蓋的章——verify-station-parity 落地後此問題變成雙向對稱。

模式：assumptions（相關程式碼充足——review.rs 失效純函式、cache.rs 封存定格、query.rs 即時重算、verify-station-parity 全套規格、兩份定案討論 code-review-stage 與 code-review-convergence-boundary）。

相關 changes／specs：verify-station-parity（未開工 0/19，規格已含收尾迴圈與 archive 雙工單守門）、已封存 converge-review-remediation-rounds（review 正典技能的收尾迴圈與「先不蓋章」選項）、已封存 code-review-stage（雙錨凍結語意）。另一獨立議題「驗收 patch 漏掉未被點名的候選檔」（review-validation-scope-gap）明確不混入本討論。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: 兩站互相打黃該在哪一層解——引擎、規格、還是流程慣例？
**Position**: 全程可在既有規格條文內用「discovery 後先不蓋章、修正統一落、兩站各自驗收」的時序解掉，零引擎、零規格變更：
- 封存定格是前提：封存側只看「有章沒章」，不重算凍結度（cache.rs:83「封存即定格」）——琥珀章是封存前的暫態顯示，不是永久污點
- 收尾迴圈規定「Bn 空 → SHALL 蓋章」（verify-skill spec 收尾迴圈；review 正典同構），技能不會自己「等」——壓後蓋章只能走規格已有的「先不蓋章結束」三選項出口（review 正典 145 行 Stop without stamping、verify-skill spec 同構）
- 慣例定形：兩站 discovery 都選「先不蓋章」→ 修正統一落 → 兩站各自 validation → 迴圈自動蓋章 → archive；兩站 discovery 凍在同一棵樹、驗收 patch 也相同，兩顆章最終涵蓋雙站都驗過的內容，比 serial 更誠實
- 「吸收他站 hunks」不需新條文：validation patch 的定義就是「從上輪 frozen afterText 到現值的差異」（verify-station spec「validation 只凍結修正 patch」情境），不分修正出自誰，他站 hunks 機械式入 patch 且回歸檢查已被「只回報 remediation patch 直接引入的 regression」強制——再寫即重述正典（config-station-canon-guard 正在立的紅線）
**Ruled out**: 輕量重蓋動詞——蓋章刪工單，重蓋＝新工單＝完整 discovery（convergence-boundary 定案首輪唯一 discovery），為小修正重跑探索不成比例；stamp-last 靠自由心證——收尾迴圈 SHALL 自動蓋章，技能層沒有裁量空間；在 verify-station-parity spec 加吸收條文——機器已涵蓋，屬重述
**Open**: 乾淨 discovery（零 findings）規格強制立即蓋章、無「先不蓋章」出口，他站後續修正仍會打黃這顆章——接受為已知暫態，還是連 review 正典一起改（正典稅：新 delta＋MARKER_VERSION＋golden 再生）？慣例文字落點（README 兩站分工表 vs 僅記於討論結論）

### Round 2 — assumptions (2026-08-04)

**Focus**: 兩站都跑時，要不要建一個專屬的編排技能？
**Position**: 不建正典技能——編排層過淺，一句 instruction 慣例可完整替代；若要一鍵入口，先以非正典的本機技能實驗：
- 刪除測試不過：技能內容只是「叫 review（選先不蓋）→ 叫 verify（選先不蓋）→ 統一修 → 各自複驗蓋章」的排程——零自有引擎狀態、零動詞、零裁決軸；刪掉它，兩站技能＋一句慣例即完整重現
- 買不到唯一想買的：乾淨 discovery 自動蓋章是站內正典的 SHALL，外層技能攔不住；要堵仍得改正典，稅照付
- 正典技能固定成本高（本專案特有）：skills.rs 逐 skill 正典化＋golden 再生＋三處同步；且兩站語意日後每動一次，多一處永久同步義務
- 既有 per-situation 技能先例不轉移：review／verify／audit 各自擁有引擎狀態（工單、章、動詞）與獨立裁決軸，是能力；編排技能一樣都沒有，是排程——不同類
- 便宜落點已在路上：verify-station-parity 本就更新 workflow 行與 README 兩站分工表，慣例一句進 instructions，代理每 session 讀到——使用者說「兩站都跑」即自動編排；instruction 層本就是此架構的編排層
- 中間路線：若想要具名入口，先做 .claude/skills 本地檔（不進引擎、不進 golden、隨時可刪）dogfood；用得順再談產品化
**Ruled out**: 引擎正典化的編排技能——深度不足且成本不成比例；合併兩站為單一檢查站早經 code-review-stage 否決（混合裁決互相遮蔽），編排技能雖非合併裁決，仍不足以自立
**Open**: 若日後「兩站都跑」成為預設流程，重議產品化（屆時傾向引擎層支援而非技能包裝）；乾淨 discovery 縫隙的 Deferred 裁決仍待定；慣例文字落點待確認

### Round 3 — assumptions (2026-08-04)

**Focus**: 時序慣例在四種站別組合（都不跑／只跑一站／兩站都跑／事後變卦加跑）下怎麼取捨？（本輪補記，實際發生於編排技能一輪之前）
**Position**: 慣例是條件式的，只在「事前已知兩站都跑」時啟動；判準＝「蓋完這顆章之後，還會不會有另一站的修正進來」：
- 都不跑：無章可打黃，照舊 apply → archive
- 只跑一站：蓋完後沒有他站修正會來打黃，技能預設（修完即蓋）即正確；使用者自行改碼打黃屬正確警示，非誤報
- 兩站都跑：先都不蓋 → 統一修 → 各自複驗 → 接連蓋 → 封存
- 事後變卦加跑第二站：照跑、接受前章暫態變黃（封存定格回綠）、不回頭重做——evidence-home-and-trace-slim 即此情況，代價已實測為零實質損失
**Open**: 乾淨 discovery 強制蓋章縫隙（建議 Deferred）待裁；慣例文字落點待確認

## Conclusion

**Decision**: 兩站互相打黃以「蓋章時序」解，零引擎、零規格變更。慣例＝兩站 discovery 都選「先不蓋章結束」→ 兩站 findings 統一修正 → 各自 validation（機器凍結的 patch 自動涵蓋他站修正）→ 兩章接連蓋 → 封存。僅「事前已知兩站都跑」時啟動；單站與都不跑照技能預設；事後變卦加跑第二站則照跑、接受前章暫態變黃（封存定格回綠）、不回頭重做。入口做成本機非正典技能（.claude/skills/speclink-quality，內容即此編排，不進引擎正典、不進 golden），既有 review／verify 技能保留給單站情況直接呼叫；慣例文句另隨 verify-station-parity 既有的 README 兩站分工表／workflow 行任務帶入。
**Rationale**: 凍結語意誠實、該保留；封存側定格（有章即綠、不重算凍結度）使黃章僅為封存前暫態；蓋章刪工單、重蓋＝新工單＝重新完整 discovery，為小修正重跑不成比例——問題純屬蓋章時點，流程層即可解。編排層零引擎狀態、零裁決軸，不值正典稅；本機技能零成本承載一鍵入口。
**Rejected alternatives**: 輕量重蓋動詞（重蓋成本結構性過高）；在 verify-station-parity 規格加「收尾輪吸收他站 hunks」條文（validation patch 定義即「上輪 frozen afterText 至現值全部差異」，機器已涵蓋，再寫屬重述正典）；引擎正典化的編排技能（刪除測試不過、golden＋三處同步永久債、且攔不住站內自動蓋章）；強制站序（任一序仍有一站被打黃，未解根因）。
**Deferred**: 乾淨 discovery（零 findings）規格強制立即蓋章、無「先不蓋章」出口——他站後續修正仍會暫時打黃該章；刻意不堵（暫態＋封存定格＋改 review 正典的稅不成比例），實務被煩到再議。本機技能若日後「兩站都跑」成為預設流程，重議產品化（屆時傾向引擎層支援而非技能包裝）。
**Capture to**: 本機技能 .claude/skills/speclink-quality/SKILL.md（編排本體）；verify-station-parity 的 README／workflow docs 任務補一句慣例（經 discuss link ＋ /speclink-ingest 帶入，非新規格）
**Next**: speclink discuss link cross-station-staleness verify-station-parity 後，開工 verify-station-parity 前先 /speclink-ingest verify-station-parity 把慣例文句納入其 docs 任務；verify 側工單／章落地後，本機技能的 verify 段才有章可蓋（先期 verify 段為對話報告，技能仍可跑）
