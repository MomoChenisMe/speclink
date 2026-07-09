---
topic: link 封印時機與『已連結未 ingest』的主動偵測
slug: link-seal-timing
status: promoted
promoted_to: discussion-reflection-seal
created: 2026-07-09
---

# Discussion: link 封印時機與『已連結未 ingest』的主動偵測

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

承接 manual-spec-edit-integrity 收尾時暴露的問題：`discuss link` 把討論翻成 promoted（看板進「已轉出變更的討論」組），但併既有 change 的內容折疊要靠後續 ingest——中途停手（本 session 實際發生過）會留下「假完成」狀態：change 帶 from_discussion、proposal 卻是舊的、討論卻顯示已轉出。

偵察定案的關鍵事實：(1) link/promote/mark_promoted 翻狀態時零內容檢查（discuss.rs:291），看板靠 promoted 分組；(2) promote 開新 change 會寫 TBD 骨架（discuss.rs:419），故「promote 沒 propose 填」可從內容偵測；(3) link 併既有 change 不碰 proposal，舊內容看起來完整，「link 沒 re-ingest」從檔案偵測不到；(4) ingest 折內容靠對話脈絡、不靠 from_discussion（ingest 技能），故 link 與內容折疊互不相依；(5) unlink_discarded（discuss.rs:333，discard-change-verb 機制、另一 session 進行中）已是「狀態追隨現實」的先例；(6) 檔案跟隨 git，mtime 在 clone/checkout 後不可靠，時間戳比對不可行。

模式：assumptions（命中 discuss.rs、model.rs、ingest/discuss 兩支 skill）。相關：manual-spec-edit-integrity 的 Deferred「Layer 1 不變量檢查」；in-flight change discard-change-verb（同動 analyze/listing 與 promoted 狀態，須留意併發）；rediscuss-promoted-change（re-conclude 已轉出討論的既有能力，使「re-ingest 過期」成真實情境）。

使用者裁定：要主動偵測，propose 與 ingest 兩條路都要防——推翻「靠 link-當封印從根本避免」的較便宜方案。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: 「已連結未 ingest」的假完成該靠 link-當封印從根本避免，還是靠主動偵測？
**Position**: 使用者裁定要主動偵測且兩條路都要防，故機制升級為「conclusion 戳記＋比對偵測」，link 時機退為次要：
- 缺陷本質＝狀態不誠實：promoted 由 link 這個 metadata 事實觸發（mark_promoted 零內容檢查），卻被看板當成「決策已折入」在讀。
- 原提「link-當封印」（併既有 change 時把 link 挪到 ingest 後）可讓偵測不到的懸空格不發生，但只是「避免」不是「偵測」，且 promote 開新 change 那條路天生 link-先、無法統一——被使用者的「要主動偵測」要求取代。
- 升級方案：propose 與 ingest 完成時，在 change meta 蓋上「已折入的討論結論指紋」（conclusion_text 的內容 hash，git-safe，非 mtime）；偵測器＝change 的每個 from_discussion slug，若其當前 conclusion hash 不等於 meta 記下的 hash（或根本沒記）⇒「待 (re-)ingest」⇒ 看板徽章＋CLI/analyze finding（agent 也看得到）。
- 此單一機制統一覆蓋兩條路：promote 沒填→無戳記→亮；link 沒 ingest→無戳記→亮；已 ingest 後 re-conclude（rediscuss）→hash 不符→亮「需 re-ingest」。TBD-marker 偵測因此被含括、無須另做。
- 布林「ingested: slug」不夠：抓得到「從沒 ingest」，抓不到「re-conclude 後過期」——而 rediscuss-promoted-change 使後者為真實情境，故需 hash 而非布林。
- 連帶決定：有了主動偵測，link 時機變得無關緊要，故保留現行 link-先（維持中斷仍自動封存的安全網，discuss.rs 註解），不動 skill 順序。
**Ruled out**: mtime 時間戳比對（git clone/checkout 破壞 mtime，檔案模式不可靠）；布林 ingested 旗標（漏抓 re-conclude 過期）；link-當封印重排 skill（偵測到位後多餘，且無法涵蓋 promote 路徑）；純 TBD-marker 偵測（漏抓 link-既有的舊內容過期，正是使用者踩到的格）。
**Open**: 戳記落點與格式（change meta 新欄位 ingested_conclusions: slug@hash8 的逗號累加，對齊 from_discussion？）；偵測器落點（listing 計算旗標＋看板徽章 vs analyze 規則 vs 兩者）；hash 對象（conclusion 段全文 strip 後）與空結論（promote 於 open 討論）的處理；本刀 vs 併入 manual-spec-edit-integrity 的 Deferred Layer 1；與 discard-change-verb 併發於 analyze/listing 的協調。

### Round 2 — assumptions (2026-07-09)

**Focus**: 能否靠調整 link 時機讓機制「設計上就正確」，而不靠增加流程時間的主動檢查？
**Position**: 能——但精準的修法是拆開 link 目前混做的兩件事，而非重排 link；並收回上一輪的 hash-on-read：
- 診斷：link 同時做了(a)鍛造封存鏈（change 的 from_discussion）與(b)宣告討論「已被反映」（討論翻 promoted）。假完成來自(b)在 link 當下就觸發。
- 修法：(a)留在 promote/link（維持 link-先、中斷仍自動封存的安全網）；把(b)——promoted 翻轉——移到「內容落地的動詞」（propose/ingest）完成時才蓋。如此「已轉出/promoted」＝「已有 change 反映此結論」為建構性事實，兩條路皆然，零重複成本。
- 這反轉 round 1（保留 link 翻狀態＋hash-on-read）。hash-on-read 判出局：每次看板載入逐 change 讀結論+雜湊=使用者反對的「增加流程時間」；且它唯一多抓的是「手改結論繞過 conclude 動詞」，而檔案模式已裁定該類為 best-effort、不追（manual-spec-edit-integrity）。
- 殘餘唯一情境：已反映後又 re-conclude（rediscuss）——staleness 在 link 之後才於討論側產生，任何 link 時機都擋不了。以事件驅動處理：對已 promoted 的討論執行 conclude ⇒ 把其連結的 changes 標記/回退為 stale（沿用既有 unlink_discarded 的同款機制）。看板讀已存旗標，無掃描。
- 結論：link 時機本身不必動；要動的是「狀態翻轉住在哪個動詞裡」。狀態成為實際工作的副產物，而非另設檢查器。
**Ruled out**: hash-compare-on-read（每次載入的重複執行成本；與事件驅動翻轉重複；唯一優勢是抓手改結論，已判出範圍外）；把 link 重排成最後一步（狀態翻轉一旦外移即無必要，且會失去中斷自動封存的安全網）；布林/純 TBD 偵測（被取代）。
**Open**: 封印機制（新動詞由 propose/ingest 完成時呼叫，vs 把 link 拆成 forge＋seal 兩動詞）；看板如何呈現「concluded 但已有待反映的連結 change」；re-conclude 用回退狀態或加旗標；本刀獨立 vs 併入 Layer 1；與 discard-change-verb 同動 promoted 狀態機的併發協調。

### Round 3 — assumptions (2026-07-09)

**Focus**: ingest 目前不讀討論紀錄，這是否也要修，且與 seal 設計的關係？
**Position**: 要修，且它是 seal 設計的另一半、非旁支：
- 現況確認：ingest 只從對話脈絡／plan file 取內容，從不讀被連結討論的 conclusion（ingest 技能第 1、5 步；提及 discussion 僅為呼叫 link，不讀其內容）。同 session 剛結論→ingest 可行（結論在脈絡內）；跨 session 續作則 ingest 無從得知要折什麼。
- 為何必須修：我們定的「ingest 完成即 seal（翻 promoted）」只有在 ingest 真的消化了討論當前結論時才誠實；若 ingest 從不讀討論，跨 session 的 seal 會是空頭封印。
- 對稱性：propose 已有 --from-discussion 由討論播種；ingest 應在 change 帶 from_discussion 時，自動把該 slug 的 conclusion_text（discuss.rs:281，含 rounds）當一等來源讀入並「合併」（不取代既有脈絡／plan）。
- 閉環：ingest 讀了討論 → 天生知道自己反映哪個 slug → 完成時精準 seal 那個 slug。「discussion-aware ingest」與「seal on completion」是同一故事的兩半。
**Ruled out**: 維持 ingest 只靠對話脈絡（跨 session 續作失效、使持久討論紀錄對 ingest 路徑半殘、與 propose 不對稱、令 seal 不誠實）。
**Open**: 讀 conclusion 或含 rounds；多來源討論（from_discussion 逗號列）逐一讀入的合併策略；三項（狀態外移、discussion-aware ingest、re-conclude 事件旗標）成一張新 change 的邊界。

### Round 4 — assumptions (2026-07-09)

**Focus**: seal 與 discussion-aware ingest 的 HOW——落在 skill 還是 engine？（結論 deferred 項的細化）
**Position**: 都要，按支柱切；查證後確認引擎缺口：
- 前提：沒有 `speclink ingest` 引擎動詞，ingest 本身就是 skill，故「改 ingest」主體是 SKILL.md。
- ingest 讀討論：主體在 skill（新增一步：取 change 的 from_discussion → `discuss show <slug>` 讀結論 → 併入現有脈絡／plan 來源）；引擎只需小補丁——`show <change> --json` 目前無 from_discussion（實測 keys：created/deltaSpecs/design/name/proposal/schema/tasks），需暴露，否則 skill 只能硬讀 .openspec.yaml。`discuss show <slug> --json` 已回 {content, info}，結論拿得到、讀取幾乎免費。
- seal：引擎為主體——`mark_promoted` 停止於 link/promote 翻 promoted；新增 `discuss seal <slug> <change>` 動詞（實測現有 discuss 動詞：new/list/show/context/add-round/conclude/archive/discard/promote/link，無 seal）；propose/ingest skill 於結尾呼叫之。
- re-conclude 標記 stale：引擎——`conclude` 動詞順手標記連結 changes。
- 架構呼應：core 管狀態語意與資料供給、skill 管讀取與合併編排。
**Ruled out**: 把「讀討論＋合併」整包做進引擎（合併是 agent 判斷，且無 ingest 引擎動詞可掛）；讓 skill 硬讀 .openspec.yaml 取 from_discussion（脆、繞過引擎契約）——改由引擎暴露 from_discussion。
**Open**: discard-change-verb 已完成，動 mark_promoted/unlink_discarded 的併發顧慮解除；本刀範圍是否含 re-conclude stale 支柱，或先只做 link＋ingest 兩支柱、stale 分刀——待 propose 時定。

## Conclusion

**Decision**: 把「討論→變更反映」的機制做正，三根支柱：
(1) 狀態外移——promoted（已轉出／已反映）不再由 link/promote 觸發，改由內容動詞 propose/ingest 完成時蓋「seal」；link/promote 只鍛造封存鏈 from_discussion（維持 link-先、中斷仍自動封存的安全網）。link 時機不動，動的是「狀態翻轉住在哪個動詞裡」。
(2) discussion-aware ingest——ingest 於 change 帶 from_discussion 時，自動讀該討論 conclusion_text（discuss.rs:281）為一等來源並合併既有脈絡／plan，與 propose 的 --from-discussion 對稱；讀了討論才 seal，seal 才誠實，且補上跨 session 續作能力。
(3) re-conclude 事件旗標——對已 promoted 的討論執行 conclude 時，順手把其連結 changes 標記 stale／待 re-ingest（沿用 discard-change-verb 的 unlink_discarded 同款事件驅動機制，discuss.rs:333）；看板讀已存旗標、無 per-load 掃描。
淨效果：看板「已轉出」為建構性真相，零重複執行成本——不靠主動檢查增加流程時間。
**Rationale**: 假完成的根因是 link 混做「鍛造封存鏈」與「宣告已反映」兩件事；拆開、把「宣告」綁到實際內容落地即根治，優於任何 link 時機重排或 hash-on-read 掃描（後者每次載入付成本，且只多抓檔案模式已放棄追的手改結論）。ingest 讀討論是 seal 誠實性的前提，也補上與 propose 的對稱與跨 session 續作。
**Rejected alternatives**: hash-compare-on-read（recurring 載入成本、與事件驅動翻轉重複、唯一多抓的手改結論已判範圍外）；link 重排成最後一步（狀態外移後多餘、且失去中斷自動封存安全網）；布林 ingested 旗標（漏抓 re-conclude 後過期，而 rediscuss 使其為真實情境）；純 TBD-marker 偵測（漏抓 link-既有的舊內容過期，正是使用者踩到的格）；ingest 維持只讀對話脈絡（跨 session 失效、與 propose 不對稱、令 seal 不誠實）；併入 discard-change-verb（該 change 另一 session 進行中 7/12，注入新範圍會衝突）。
**Deferred**: seal 的具體動詞形態（新增 seal 動詞由 propose/ingest 完成時呼叫，vs 把 link 拆成 forge＋seal 兩動詞）；讀 conclusion 是否含 rounds；多來源討論（from_discussion 逗號列）的合併策略；看板對「concluded 但有待反映連結 change」與「stale 待 re-ingest」的視覺呈現；與 manual-spec-edit-integrity 的 Layer 1（懸空引用、壞 meta 浮現）合刀或分刀——傾向分（不同訊號、不同觸發）。
**Capture to**: proposal（新 change，暫名 discussion-reflection-seal；實作須待另一 session 的 discard-change-verb 封存後再動 mark_promoted/unlink_discarded 同檔，避免併發衝突）
**Next**: 待 web-server-postgres ingest／desktop-acp-agent 修正告一段落後，discuss promote link-seal-timing 開實作刀
