---
topic: 討論結論分頁的排版機制——Decision 欄位整段長句不可讀
slug: conclusion-tab-readability
status: promoted
created: 2026-09-22
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: discuss-conclusion-bullets
---

# Discussion: 討論結論分頁的排版機制——Decision 欄位整段長句不可讀

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者貼出 wadpilot 專案 team-server-and-admin 討論的結論分頁截圖：Decision 欄位是數千字的單行長句，讀起來很累，問能否有更好的排版機制。需求已附截圖與可驗證目標（結論分頁可讀），不需 grill 階段。掃描發現：渲染端（DiscussionDrawer 的 ConclusionView）已把六欄位拆成標籤區塊、欄位內文交 Markdown 渲染，來源有條列即可渲染成清單；癥結在寫法——技能文件規則第 5 條只對輪的 Position 規定「超過一句就條列」，Conclusion 無對應規則，本專案 Decision 行最長 3947 字、wadpilot 3355 字。相關規格：desktop-app「討論結論以欄位標籤呈現」「markdown 文件內容行寬有上限」、discuss-skill「討論記錄的樹慣例與格式不變」；無進行中變更。
Prior discussions: drawer-document-readability, specs-archive-pagination

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-22)

**Focus**: 結論分頁不可讀的根因在渲染器還是在記錄寫法，以及 Decision 該裝多少細節
**Position**: 根因在寫法：Conclusion 沒有輪的 Position 那條「超過一句就條列」規則，修法主軸是技能規則，前端不改。
- 證據：ConclusionView（DiscussionDrawer.tsx）已拆六欄位標籤、欄位內文交 Markdown 渲染；輪的 Position 規則（discuss.md 第 38 行）落地後 12 份記錄有 11 份零超長行，同招對 Conclusion 沒做
- Decision 保留全部定案細節、不縮水，只拆行：一句總結起頭；多刀時每刀一個 bullet「**cut N `change-name`**：一句範圍」，細節縮排子項、一子項一件事（使用者確認）
- 分工不變：定案事實在 Decision，理由／否決／中途問題在各輪；propose 讀 Decision 拿事實、讀輪拿否決清單；細節的第二讀取面就是既有的討論過程分頁，不加新檢視機制
- 不在子項尾標「（第 N 輪）」回指——每行多一個括號、多一道工，輪的 Focus 已可找到
**Ruled out**: 渲染器用「；」或「cut N」啟發式自動拆行——會切壞正常句子，且前次討論已裁定舊記錄不回改；Decision 只放一句定論、細節留在輪——輪是 append-only、含被推翻的立場，propose 要逐輪拼最終狀態，12 輪的討論拼錯風險真實
**Open**: 引擎 promote 把整段 Conclusion 預填進提案 Why 要不要一起處理；Rejected alternatives／Deferred 的條列格式；三處技能同步與 ASSET_VERSION 成本；wadpilot 那份是否重跑 conclude

### Round 2 — assumptions (2026-09-22)

**Focus**: 引擎把整段 Conclusion 預填進提案 Why 要不要一起處理，以及 Rejected alternatives／Deferred 的條列格式
**Position**: Why 預填不動；結論欄位的條列規則與輪的第 5 條對稱（使用者確認）。
- Why 預填只是鷹架：propose 技能第 5 步整份重寫 proposal.md，預填活不到提案完成；只在直接敲 `discuss promote` 的原始動詞路徑留下，此時整段結論當 Why 比只放 topic 合理；Decision 改條列樹後佔位本身也可讀
- 規則字面：Decision、Rejected alternatives、Deferred 超過一句就條列；Rationale、Capture to、Next 維持單段
- Rejected alternatives 每項一行「方案——落敗理由」；Deferred 每項一行「問題——為何現在不解」或單寫 none
- 三處技能同步（core asset、.claude／.agents 實例、golden 再生）加 ASSET_VERSION 是既定成本；wadpilot 那份於規則落地後重跑一次 conclude 覆寫
**Ruled out**: 預填改成只塞 Rationale——要動引擎、規格、測試三處，換來沒人細讀的佔位文字
**Open**: none

## Conclusion

**Decision**: 結論分頁的可讀性靠寫法規則修，不動渲染器：discuss 技能的文件規則加第 8 條「Conclusion bullets over prose」，與輪的第 5 條對稱。
- 規則字面：Decision、Rejected alternatives、Deferred 超過一句 SHALL 條列；Rationale、Capture to、Next 維持單段
- Decision：一句總結起頭，之後 `- ` 一點一行；保留全部定案細節、不縮水
- 多刀結論：每刀一個 bullet「**cut N `change-name`**：一句範圍」，細節縮排子項、一子項一件事
- Rejected alternatives：每項一行「方案——落敗理由」；Deferred：每項一行「問題——為何現在不解」或單寫 none
- 分工不變：定案事實在 Decision，理由／否決／中途問題在各輪；細節的第二讀取面是既有的討論過程分頁，不加新檢視機制；子項不回指輪次
- conclude 模板範例同步改成條列形；三處技能同步（core asset、.claude／.agents 實例、golden 於乾淨樹再生）並 bump ASSET_VERSION；舊記錄不回改，wadpilot 的 team-server-and-admin 於規則落地後重跑一次 conclude 覆寫
**Rationale**: 渲染端（ConclusionView）已拆六欄位標籤、欄位內文交 Markdown 渲染，來源有條列就能渲染成清單；輪的 Position 在 2026-09-10 加同型規則後 12 份記錄 11 份零超長行，證明規則有效；Decision 是唯一不含被推翻立場的地方，propose 靠它拿最終狀態與刀清單，所以細節留在 Decision、只拆行不刪。
**Rejected alternatives**:
- 渲染器用「；」或「cut N」啟發式自動拆行——會切壞正常句子，且 drawer-document-readability 已裁定舊記錄不回改
- Decision 只放一句定論、細節留在輪——輪 append-only 含被推翻立場，propose 要逐輪拼最終狀態，12 輪討論拼錯風險真實
- 子項尾標「（第 N 輪）」回指——每行多一個括號、多一道工，輪的 Focus 已可找到
- promote 的 Why 預填改成只塞 Rationale——預填只是鷹架，propose 技能整份重寫；改法要動引擎、規格、測試三處換沒人細讀的佔位
**Deferred**: none
**Capture to**: proposal | specs/discuss-skill（「討論記錄的樹慣例與格式不變」MODIFIED 補結論條列規則）
**Next**: /speclink-propose --from-discussion conclusion-tab-readability
