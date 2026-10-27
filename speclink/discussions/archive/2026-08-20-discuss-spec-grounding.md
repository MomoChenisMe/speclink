---
topic: discuss 階段是否先讀既有規格再形成假設（規格接地）
slug: discuss-spec-grounding
status: promoted
promoted_to: discuss-grounding-and-flow
created: 2026-08-20
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: discuss 階段是否先讀既有規格再形成假設（規格接地）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者查核「propose 是否參考現有規格」時，發現 discuss 的偵察步驟明文排除規格——crates/speclink-core/assets/skills/discuss.md:166 寫 "find related source files (not docs, not tests)"，但同檔第 63 行的 Context 模板卻期待記下 "related changes/specs found by the codebase scout"。期待存在，產出它的步驟不存在。本討論定案 discuss 階段的規格接地方式。

模式：assumptions——codebase scout 找到多個直接相關檔案（crates/speclink-core/assets/skills/discuss.md、.claude/skills/speclink-propose/SKILL.md、crates/speclink-core/assets/schema/spec-driven/schema.yaml、crates/speclink-core/src/instructions.rs）。

相關規格／討論：propose skill 第 3 步已有規格掃描的現成樣式（SKILL.md:98-108：list --specs → 候選 ≤5 → 讀 Purpose ≤3，只顯示不擋）。開啟中討論 capability-naming-dedup 題目相鄰（命名撞車的引擎防護點），其 Round 1 已確立 propose 掃描「只顯示、不擋、不留痕」；兩題互相引用、分開結論。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-20)

**Focus**: discuss 是否存在「規格接地」缺口？修法的形狀是什麼？
**Position**: 缺口確認，五條假設全數獲使用者確認。
- 偵察步驟明文排除規格（crates/speclink-core/assets/skills/discuss.md:166 "not docs, not tests"）
- skill 內部矛盾：Context 模板期待記 related specs（discuss.md:63），但沒有任何步驟產出它——補掃描是補實既有期待，不是加新功能
- 修法沿用 propose 第 3 步樣式（speclink list --specs --json → 候選 ≤5 → 讀 Purpose ≤3），只改 skill 資產文字，不動引擎
- 與 capability-naming-dedup 相鄰不同題：那邊管命名撞車的引擎防護點，這邊管討論階段的接地品質；開新記錄、互相引用
- 改 asset 內文觸發 MARKER_VERSION／golden／assets.lock 三連動，列入日後提案的 Impact
**Open**: 「先看規格再形成假設」這個順序本身適不適合（錨定風險如何處理）？規格命中要不要計入 assumptions 模式的 3+ 門檻？要不要同時掃 in-flight change 的 delta？讀取深度停在 Purpose 還是對直接目標規格讀全文？

### Round 2 — assumptions (2026-08-20)

**Focus**: 紀律定位確認後，規格接地是否真的「更快抓到問題」？機制是什麼？
**Position**: 是。紀律定為「使用者需求是目標，正典是證據、不是裁決」，接地把提早抓問題機制化。
- 三分對照（正典已涵蓋／與正典衝突／正典沒講）把模糊需求變成可查核的主張，衝突在討論第 1 輪現形
- 對照組：不接地時，衝突最快在 propose 第 3 步現形（只顯示不擋、靠 AI 自覺），最糟到 archive 合併閘門才硬炸——代價差一整條 pipeline（提案＋delta＋可能已實作的任務）
- 雙軌偵察掛法與成本上界獲確認：候選 ≤5、讀 Purpose ≤3、主題直接動到的 capability 才讀全文、沒命中靜默略過
**Ruled out**: 「正典當裁決」——偏離正典必須允許，但偏離要寫進記錄成為有意識的決定；不知道 base 的大膽只是盲目
**Open**: 規格命中要不要計入 assumptions 模式的 3+ 門檻（我方建議：計入）；in-flight change 的 delta 要不要一併掃（我方建議：讓給題目相鄰的 capability-naming-dedup 統一決定，本題停在正典層）

### Round 3 — assumptions (2026-08-20)

**Focus**: 調整落地後的影響面盤點——成本、行為改變、風險、與不變的邊界
**Position**: 影響可控，但盤點浮出兩條邊界，要補進結論才收案。
- 每次 discuss 的成本上界固定：多 1 次 speclink list --specs ＋ ≤3 次 Purpose 讀取；無規格專案（剛 onboard）或無命中主題靜默略過，零噪音
- 行為改變：規格命中計入 3+ 門檻後，部分原走 interview 的需求形主題會改走 assumptions——收斂更快，但實作層證據可能偏薄；使用者仍可用「Ask me questions instead」手動切回訪談，既有開關已涵蓋
- 維護成本一次性：改 discuss.md asset 觸發 MARKER_VERSION／golden／assets.lock 三連動＋speclink update 再生 32 份 SKILL.md；期間有平行 change 動 asset 會版號對撞，解法是重生衍生物
- 下游影響：asset 發給所有採用 speclink 的專案；規格數百的專案 list 輸出變大但讀取上界不變
- 失效模式誠實面：候選仍靠 AI 關鍵字比對，命名家族共用 token 時會漏——漏抓率降低、不歸零
- 邊界一：propose 第 3 步不因此省略——直接 propose 與 --from-doc 路徑沒有經過討論，第二道防線必須留
- 邊界二：引擎側確定性防護（capability-naming-dedup 的題目)不因此免做——指令側接地只是降險，不是保證
**Ruled out**: 「討論已掃過規格，propose 可省略第 3 步」——propose 有不經討論的入口，防線不能只剩一道
**Open**: 結論草案補上兩條邊界後收案？

### Round 4 — assumptions (2026-08-20)

**Focus**: 對照 spectra 原始目的，接地是否造成 discuss 偏離？assumptions／interview 雙模式的實效比較與優化。
**Position**: 不偏離——正典是 codebase 形狀的「承諾層」，接地補完「搭配現有形狀研究可行性」；該優化的是模式分流本身。
- 原始目的五要素逐一比對（需求為目標／讀 codebase 形狀／研究可行性提假設／使用者決定／留下演進過程）：接地強化其中三項、無一違反；反偏離護欄＝「證據不裁決」紀律
- 使用者實測觀察成立：3 檔門檻在成熟 codebase 幾乎永遠滿足，interview 近乎死路徑；接地計入命中後更甚（本專案 71 個 capability）
- 關鍵發現：兩模式的實質差異只在呈現節奏（批次列假設 vs 逐題問），證據要求完全相同——interview 規則本就要求每題附建議答案與 Evidence
- 優化案：assumptions 為預設姿態；interview 降為「逐節點退路」——證據撐不起立場的節點才化為單一問題（仍附最佳猜測），全域模式分流與 3 檔門檻廢除
- 附帶效果：前輪 open「規格命中要不要計入門檻」隨門檻廢除而消解
- spectra 靠對話上下文餵 propose、speclink 硬化成討論記錄——「留下演進過程」這條目的由記錄機制承載，接地讓 Evidence 多了「討論當下正典長什麼樣」的快照價值
**Ruled out**: 維持雙模式現狀——檔案數是「有沒有脈絡」的粗代理指標，量錯了東西；該量的是每個節點證據撐不撐得起立場
**Open**: 模式重構與規格接地是否一併收進結論（一份討論可扇出兩個變更）？

### Round 5 — assumptions (2026-08-20)

**Focus**: 偵察雙軌要平行跑，還是先查正典再依內容查程式碼（漏斗式）？
**Position**: 改漏斗式——先查正典，用命中的 capability 名與正典詞彙轉譯搜尋詞，再進程式碼，搜尋確實更快。
- 正典是策展過的索引：speclink list --specs 一條指令回全圖（capability 名＋Purpose），比盲 Grep 便宜且無噪音
- 加速機制是「語言轉譯」：使用者措辭貼近領域語言，與規格命名對得上、與程式碼識別符常對不上——正典層先把使用者語言轉成系統語言，再進大草堆
- 規格不含檔案路徑，所以加速靠詞彙轉譯與範圍圈定，不是直接跳檔
- 主題已含具體檔名／符號時，程式碼軌直接開跑，不等正典
- 正典零命中（工具鏈、選型類主題）→ 退回現行關鍵字 Grep，總成本只多一條便宜指令
**Ruled out**: 平行雙軌（Round 2 原案）——平行時程式碼軌仍用使用者原始關鍵字，浪費正典的轉譯價值

### Round 6 — assumptions (2026-08-20)

**Focus**: interview 是移除，還是還原成原始意圖（grillme：先磨需求，磨完才進 assumptions）？
**Position**: 不移除——還原成「磨需求階段」，需求鈍時先 grill、磨利後進 assumptions；廢除的只有檔案數模式分流。
- 使用者揭露原始意圖：interview 源自 Matt Pocock 的 grillme——AI 看完現況後逼問使用者、磨利需求，磨完才列假設；現行實作把它誤植成「程式碼不足時的退路」，觸發軸放錯了
- 真正的分流軸是「需求清晰度」，不是檔案數：需求利→直接 assumptions；需求鈍（沒有可驗證的目標、沒有門檻、improve/better 類措辭）→先 grill
- 需求已利時 grill 階段自然塌縮成零題，不是強制儀式
- 兩種問題分工明確：grill 問「你要什麼」（只有使用者能答的意圖題）；Round 4 的逐節點退路問「這樣設計對不對」（證據不足時），兩者並存不衝突
- 與 Round 5 漏斗互補：正典先行讓 grill 題更利——「正典已承諾 X，你要的與 X 差在哪？」
- 現行 skill 的 Push for specifics 範例（逼問 threshold）本來就是 grill 題型，重構後歸入 grill 階段
**Ruled out**: 完全移除 interview（Round 4 傾向）——丟失磨需求功能，對著鈍需求列假設是在猜心、不是在研究；強制每題必 grill——利需求直接跳過
**Open**: 結論收「接地（漏斗版）＋模式重構（grill 版）」兩案並列？

## Conclusion

**Decision**: 兩案並列，共同紀律為「使用者需求是目標，正典是證據、不是裁決；偏離正典允許，但要寫進記錄」。
1. 規格接地（漏斗版）：discuss 偵察改漏斗式——先 speclink list --specs --json（候選 ≤5、讀 Purpose ≤3、主題直接動到的 capability 才讀全文、零命中靜默略過），用命中的 capability 名與正典詞彙把搜尋詞從「使用者語言」轉譯成「系統語言」後再進程式碼；主題已含具體檔名／符號時程式碼軌直接開跑。三分對照（正典已涵蓋／與正典衝突／正典沒講）寫進假設清單。
2. 模式重構（grillme 還原）：廢除檔案數模式分流；分流軸改為需求清晰度——需求鈍時先 grill（一次一題磨需求：目標／範圍／門檻／成功判準，題目可附現況與正典證據；需求已利時塌縮為零題），磨利後進 assumptions（唯一預設姿態）；假設中證據撐不起的節點就地化為單一問題並附最佳猜測。三種問題分工：grill 問意圖（只有使用者能答）、節點退路問設計裁決、事實題永遠由環境查證。
**Rationale**: 問題現形點從 propose／archive 提前到討論第 1 輪，代價差一整條 pipeline；正典層先做語言轉譯，搜尋更快更準；三分對照讓結論自帶 delta 形狀，--from-discussion 種子品質提升；grillme 還原讓假設對著磨利的需求列——對著鈍需求列假設是猜心，不是研究。
**Rejected alternatives**: 正典當裁決（扼殺新方向）；平行雙軌偵察（浪費正典的轉譯價值）；完全移除 interview（丟失磨需求功能）；強制每題必 grill（利需求應跳過）；維持檔案數雙模式分流（檔案數是脈絡的粗代理，量錯東西）；條件式掃描與引擎級相關度比對（靜默略過已足，YAGNI）；「討論已掃規格故 propose 省略第 3 步」（直接 propose 與 --from-doc 不經討論，防線不能只剩一道）。
**Deferred**: in-flight change 的 delta 掃描範圍——讓給題目相鄰的 capability-naming-dedup 統一決定；引擎側確定性命名防護亦屬該討論。
**Capture to**: proposal——改 discuss skill 資產（crates/speclink-core/assets/skills/discuss.md）；Impact 需列 MARKER_VERSION／golden／assets.lock 三連動與 speclink update 再生的 32 份 SKILL.md。
**Next**: /speclink-propose --from-discussion discuss-spec-grounding（兩案可扇出為一或兩個變更，propose 時再定拆法）
