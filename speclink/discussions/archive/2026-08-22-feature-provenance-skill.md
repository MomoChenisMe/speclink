---
topic: 功能溯源技能：從規格沿討論鏈回答「這功能怎麼來的」
slug: feature-provenance-skill
status: promoted
promoted_to: feature-provenance-skill
created: 2026-08-22
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 功能溯源技能：從規格沿討論鏈回答「這功能怎麼來的」

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者要做一個新技能：回答「某功能怎麼來的／怎麼設計的」——先查規格，再沿相依討論鏈找到當初的決策與一起演進的其它規格。討論起點是可行性確認，隨即轉入設計。要求明確（可驗證的查詢流程），無需 grill 階段。

Scout 實測的鏈路把手：spec 的 @trace source（change 名）→ archive 目錄 .openspec.yaml 的 from_discussion → 討論 frontmatter 的 promoted_to（扇出清單）→ 各 change archive 目錄的 specs/<capability>/ 子目錄。程式碼連結自 2026-08 起住在封存 change 的 .evidence.json（逐 task 的 files 清單）；七月舊 @trace 的 code: 清單存在但受平行 session 污染、不可信。

相關規格：discussion-docs、discuss-skill、archive-skill、change-lifecycle。相關討論先例：sdd-engine-as-sdk...（一場討論扇出 4 個 change 的實例）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-22)

**Focus**: 這條「規格 → 討論 → 關聯規格」的溯源鏈可行嗎？每一環的 metadata 是否已存在？
**Position**: 可行，鏈路每一環都有現成把手，不需新增資料。
- spec Requirement 的 @trace source 指向最後一次動它的 change；完整演進史靠 glob 172 個封存 change 目錄的 specs/<capability>/ 子目錄
- change → 討論走 .openspec.yaml 的 from_discussion；討論 → 其它 change 走 frontmatter promoted_to（sdd-engine-as-sdk 一場扇出 4 個 change 為實證）
- 使用者確認三點：(1) 想探索 skill 搭配引擎篩選查資料的空間 (2) 中途導入 SDD、覆蓋率非 100%，查無規格時回歸 codebase 找線索 (3) 查完規格後應能接到程式碼、最後回頭看實際 codebase
**Ruled out**: 「需要新增 metadata 或 migration 才可行」——實測現有欄位已足夠
**Open**: 引擎要不要加溯源／篩選動詞（vs 純 skill）；查無規格時的 codebase 考古步驟長怎樣；程式碼線索的正確來源（@trace code: 已不可信、新制在 .evidence.json）；skill 入口形態與名稱

### Round 2 — assumptions (2026-08-22)

**Focus**: 程式碼線索從哪來（@trace code 清單已不可信），以及引擎要不要參與篩選？
**Position**: 程式碼連結分時期取用，引擎參與方向獲使用者首肯（「篩選更完善的話也可以」）。
- 新封存（2026-08-04 evidence-home-and-trace-slim 起，44/172 個）：.evidence.json 逐 task files 清單，實測乾淨可信
- 舊封存（~128 個）：@trace code 清單受平行 session 污染不可用，改走 git log --grep <change名> 反查（舊 commit 慣例 scope 即 change 名，實測一發命中），commit diff 即觸及檔案
- 最終步固定讀 live code：evidence 與 commit 都是封存快照，後續 change 可能已改動或搬移檔案
- 分工制提案：引擎動詞（speclink trace）組裝機械鏈＋篩選＋標記無 evidence 的舊 change；skill 負責兩種考古（無規格→codebase、舊 change→git）與 live code 驗證
**Ruled out**: 引擎全包含 git 考古——agent 本來就擅長 git 考古與讀碼，引擎呼叫 git log 是多餘複雜度；沿用舊 @trace code 清單——污染已證實
**Open**: 一個 change 還是拆兩個（skill 先行 vs 同步做動詞）；skill 入口形態與名稱

### Round 3 — assumptions (2026-08-22)

**Focus**: 分時期（2026-08-04 分界）的設計被使用者否決——時期概念能不能出現在設計裡？
**Position**: 不能。分期是本 repo 的歷史敘事，技能是 speclink 產品技能、要面對各種封存狀態的專案，設計改為逐 change 存在性偵測。
- 規則：有 .evidence.json 就用；沒有就靜默走 git 反查——presence check，不寫死任何日期
- 降級是內部管線：兩條路對終端使用者輸出同一種敘事答案，只自然引用來源（討論結論、提案 Why、commit），不出現「舊時期／降級」字眼
- 引擎動詞的 --json 可帶 evidence: null 等機器可讀欄位供 skill 判斷補考古，屬 skill 與引擎之間的契約，不進使用者可見文案
**Ruled out**: 以日期分界描述取線索邏輯——把本 repo 的時間軸寫死進產品技能，對新導入專案毫無意義
**Open**: 一個 change 還是拆兩個（skill 先行 vs 同步做引擎動詞）；skill 入口形態與名稱

### Round 4 — assumptions (2026-08-22)

**Focus**: 拆兩個 change（skill 先行）還是一個 change 一起做？
**Position**: 使用者裁定一個 change 一起做——skill 與引擎動詞同一個 change 內交付。
- 引擎動詞與 skill 的分工照第 2、3 輪定案：動詞組裝機械鏈＋篩選＋機器可讀的 evidence 缺口欄位；skill 負責考古降級與 live code 驗證
**Ruled out**: 拆兩個 change、skill 先行磨介面——使用者選擇一次交付
**Open**: skill 入口形態與名稱；引擎動詞名稱與 JSON 介面形狀

### Round 5 — assumptions (2026-08-22)

**Focus**: skill 入口形態與名稱、引擎動詞名稱與介面形狀。
**Position**: 使用者確認：skill 與引擎動詞同名 trace，入口收自然語言問題。
- 引擎：speclink trace <capability> [--json]——輸出各 Requirement 的 source change、from_discussion、討論 promoted_to 兄弟 change 與各自觸及 capability、evidence 檔案清單（無則 null）
- skill：/speclink-trace <自然語言問題>；canon pass 對應 capability → 有規格走引擎鏈＋討論／proposal 決策＋evidence 或 git 反查＋live code 確認；查無規格直接 codebase 考古（git log／blame）
- 命名理由：規格錨點既有 @trace，動詞、skill、錨點三者同詞同概念
- 詞彙缺口：中文使用者可見詞「溯源」LANGUAGE.md 無詞條，結論標 capture
**Open**: 無——全數收斂

## Conclusion

**Decision**: 新增功能溯源能力，一個 change 交付兩件事：引擎動詞 `speclink trace <capability> [--json]`（組裝規格→change→討論→兄弟 change→evidence 的機械鏈，evidence 缺失以 null 標記）＋產品技能 `/speclink-trace <自然語言問題>`（canon pass 對應 capability；有規格走引擎鏈、讀討論結論與 proposal Why、evidence 或 git 反查拿檔案、最終讀 live code 確認現況；查無規格靜默改走 codebase 考古 git log／blame），輸出統一的敘事答案附來源路徑。
**Rationale**: 溯源鏈每一環的 metadata 已存在（@trace source、.openspec.yaml from_discussion、討論 promoted_to、.evidence.json），不需新資料；分工制讓引擎做機械組裝與篩選（desktop 未來直接受益）、agent 做它擅長的 git 考古與讀碼；逐 change 的存在性偵測（presence check）取代任何日期分界，降級是內部管線、對使用者輸出永遠是同一種敘事。
**Rejected alternatives**: 以日期分期描述取線索邏輯——把本 repo 時間軸寫死進產品技能，對新導入專案無意義；沿用舊 @trace 的 code 清單——受平行 session 污染已證實；引擎全包含 git 考古——agent 擅長此事，引擎呼叫 git log 是多餘複雜度；拆兩個 change skill 先行——使用者裁定一次交付。
**Deferred**: desktop 溯源面板（引擎動詞的 --json 已為其鋪路，但面板本身不在此 change）。
**Capture to**: proposal（經 promote 種子）；LANGUAGE.md（「溯源」詞條——中文使用者可見詞目前無正典）
**Next**: /speclink-propose --from-discussion feature-provenance-skill
