# discuss-skill Specification

## Purpose

/speclink-discuss 技能的訪談行為：以決策樹遍歷提問、每題附上建議答案與支持它的 Evidence、把輸入分診為事實或決策並逐節點查證，以及外部文件作為預填樹來源時的逐條分診與 Source doc 記錄慣例。本 capability 保證討論記錄維持既有的樹狀慣例與格式，使用者面對的是有依據的具體選項而不是空白提問。

## Requirements

### Requirement: interview 模式以決策樹遍歷提問

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 將 interview 重新定位為兩種提問位置：需求磨利（grill）階段與 assumptions 內的逐節點退路，SHALL NOT 保留以檔案數門檻分流的全域雙模式。分流軸 SHALL 為需求清晰度：需求鈍（無可驗證目標、無門檻、improve 類措辭）時先走 grill 階段一次一題磨需求，需求已利時 grill SHALL 塌縮為零題並直接進 assumptions（唯一預設姿態）。提問紀律 SHALL 維持決策樹遍歷：開場先攤開決策空間（根節點為「這題到底在決定什麼」，展開子決策與依賴邊），提問 SHALL 依依賴順序進行、一次一題，上游決策先解。停止條件 SHALL 維持使用者主導：one nudge maximum 與結論的 Deferred 欄位續留，技能檔 SHALL NOT 要求「所有分支解完才可收斂」。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含決策樹提問紀律

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 將提問策略描述為決策樹遍歷（grill 階段與節點退路皆適用）：開場攤開決策空間、依依賴順序一次一題、上游決策先解

#### Scenario: 停止條件維持使用者主導

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的收斂（Convergence）相關段落
- **THEN** one nudge maximum 與 Deferred 機制 SHALL 保留，技能檔 SHALL NOT 含「未走完所有分支即不得收斂」的規定，且 SHALL 規定使用者喊停時未走分支記入結論的 Deferred

#### Scenario: 需求清晰度分流取代檔案數門檻

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的分流規則
- **THEN** 技能檔 SHALL 以需求清晰度分流（需求鈍先 grill 磨需求、需求利直接 assumptions），SHALL NOT 含「相關檔案 3 個以上走 assumptions、不足走 interview」的檔案數門檻，且 SHALL 規定需求已利時 grill 塌縮為零題


<!-- @trace
source: discuss-grounding-and-flow
updated: 2026-08-21
-->

---
### Requirement: interview 每題附建議答案與 Evidence

技能檔 SHALL 對兩類提問分別規定證據義務，SHALL NOT 允許任一類空白提問：grill 意圖題（問目標、範圍、門檻、成功判準）SHALL 附框題脈絡——現況或正典證據——或最佳猜測建議；節點退路題（assumptions 中證據撐不起立場的節點）SHALL 附代理人的建議答案並附 Evidence（檔案路徑或查證結果）。兩類皆 SHALL 讓使用者僅需同意或修正。

#### Scenario: 渲染產物將 Evidence 列為 interview 硬規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的提問規則
- **THEN** 技能檔 SHALL 規定 grill 題附框題脈絡或最佳猜測建議、節點退路題附建議答案與 Evidence，兩類皆不得空白提問，使用者僅需同意或修正


<!-- @trace
source: discuss-grounding-and-flow
updated: 2026-08-21
-->

---
### Requirement: 事實與決策分診及逐節點查證

技能檔 SHALL 規定每個決策節點解決前先分診：環境（程式碼、檔案系統、工具）查得到的事實 SHALL 由代理人沿樹逐節點自行查證，SHALL NOT 拿去問使用者、SHALL NOT 憑印象作答；僅真正的決策（使用者裁定事項）交由使用者。開場偵察 SHALL 為漏斗式：先跑 speclink list --specs --json（候選 ≤5、讀 Purpose ≤3、主題直接動到的 capability 才讀全文、零命中靜默略過），以命中的 capability 名與正典詞彙轉譯搜尋詞後再掃原始碼（至多讀 5 檔）；主題已含具體檔名或符號時，程式碼軌 SHALL 直接開跑不等正典。偵察用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 沿樹逐節點進行（確定會走到的分支才深讀）。

#### Scenario: 渲染產物含事實決策分診規則

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔
- **THEN** 技能檔 SHALL 規定：查得到的事實由代理人自行查證後附為 Evidence，不得提問使用者、不得憑印象作答；決策類問題才交由使用者裁定

#### Scenario: 開場 scout 維持淺掃

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察段落
- **THEN** 偵察 SHALL 維持時間盒（正典讀取至多 3 份 Purpose、原始碼至多 5 檔），用途 SHALL 為接地與需求清晰度判定；深入查證 SHALL 規定於決策樹逐節點進行

#### Scenario: 正典先行的漏斗偵察

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的偵察順序規定
- **THEN** 技能檔 SHALL 規定先查正典（speclink list --specs --json）、以命中詞彙轉譯搜尋詞後再掃原始碼；主題含具體檔名或符號時程式碼軌直接開跑；正典零命中時靜默退回關鍵字掃描


<!-- @trace
source: discuss-grounding-and-flow
updated: 2026-08-21
-->

---
### Requirement: 討論記錄的樹慣例與格式不變

技能檔 SHALL 規定記錄內容慣例：首輪 Position 攤開初始決策空間（得含 ASCII 樹），之後每輪聚焦解掉一個節點，中途發現的新分支記入該輪 Open。討論文件的骨架（Context／Rounds／Conclusion）、輪模板欄位（Focus／Position／Ruled out／Open）與 append-only 規則 SHALL 維持不變，既有討論記錄 SHALL NOT 需要遷移。

#### Scenario: 首輪攤樹且每輪一節點

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的記錄規則
- **THEN** 技能檔 SHALL 規定首輪 Position 含初始決策空間、後續每輪解一個節點、新分支記入該輪 Open

#### Scenario: 既有記錄格式沿用

- **WHEN** 以更新後的技能進行討論並經 speclink discuss 動詞寫入記錄
- **THEN** 產出的討論文件 SHALL 維持 Context／Rounds／Conclusion 骨架與 Focus／Position／Ruled out／Open 欄位，與既有記錄格式一致，無需任何遷移

<!-- @trace
source: discuss-decision-tree-interview
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
-->

---
### Requirement: 文件作為預填樹來源逐條分診

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 規定：topic 指定文件路徑（自寫 markdown、plan mode 產出、repo 內 docs 或任意可讀路徑）時，代理人 SHALL 讀取該文件並萃取其主張作為決策樹節點，逐條對 codebase 分診為三類——證實（附程式碼證據）、牴觸（指出文件內容與程式碼實況的差異並附證據）、真決策（送使用者裁定）。文件 SHALL NOT 僅作背景素材一次性閱讀。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含文件分診紀律

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 規定：文件主張萃取為決策樹節點，逐條分診為證實／牴觸／真決策三類，且每類附對應證據或裁定去向

#### Scenario: 文件主張與程式碼牴觸時逐條呈現

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔對牴觸類主張的處理規定
- **THEN** 技能檔 SHALL 規定牴觸須逐條指出文件內容與程式碼實況的差異並附程式碼證據，SHALL NOT 允許籠統帶過或僅摘要文件

<!-- @trace
source: discuss-propose-from-docs
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
-->

---
### Requirement: Source doc 記錄慣例

技能檔 SHALL 規定：以文件為輸入的討論，其記錄的 Context 段 SHALL 含一行 Source doc: <路徑>；輪的 Evidence 引用文件時 SHALL 以段落標題或短句為之；討論記錄 SHALL 只存討論結果，SHALL NOT 內嵌整份規劃文件；代理人 SHALL NOT 修改使用者的原始規劃文件。

#### Scenario: Context 記錄文件來源

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的記錄規則
- **THEN** 技能檔 SHALL 規定 Context 含 Source doc: <路徑> 一行、輪 Evidence 以段落標題或短句引用文件、記錄不內嵌整份文件、不修改原始文件

#### Scenario: 未給文件時記錄照舊

- **WHEN** 討論的 topic 未指定任何文件
- **THEN** 技能檔 SHALL 規定記錄流程與現行相同，Context 無 Source doc 行，無額外文件讀取步驟

<!-- @trace
source: discuss-propose-from-docs
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
-->

---
### Requirement: 正典接地與三分對照

技能檔 SHALL 規定：偵察命中相關正典時，假設清單 SHALL 對使用者需求逐項做三分對照——正典已涵蓋（附 spec 證據）、與正典衝突（指出衝突內容並附證據）、正典沒講（新地盤，順帶檢查 capability 命名鄰近既有規格）。紀律 SHALL 明文為「使用者需求是目標，正典是證據、不是裁決」：偏離正典 SHALL 為允許的結論方向，但 SHALL 記入討論記錄成為有意識的決定。

#### Scenario: 需求與正典衝突時列為假設

- **WHEN** 偵察發現使用者需求與正典既有承諾衝突
- **THEN** 技能檔 SHALL 規定該衝突以假設形式呈現（附 spec 證據），由使用者裁定改正典或改需求，SHALL NOT 逕行擋下或否決討論方向

#### Scenario: 正典零命中時流程照舊

- **WHEN** 主題與任何正典 capability 無關（工具鏈、依賴選型類主題）
- **THEN** 技能檔 SHALL 規定不提及規格掃描，後續流程與純程式碼偵察相同

<!-- @trace
source: discuss-grounding-and-flow
updated: 2026-08-21
-->

---
### Requirement: 多需求 backlog 與恢復摘要

技能檔 SHALL 規定：單一討論載有多個需求時，首輪 Open SHALL 攤開全部需求清單，之後每輪 Open SHALL 復述剩餘未談項；已定案項的去向由該輪 Position 首句承載。續用既有 open 討論時，代理人 SHALL 先呈現恢復摘要——逐輪 Focus 與 Position 首句、最後一輪的 Open 邊界——再接續討論。摘要 SHALL 自既有欄位機械推導，SHALL NOT 引入新記錄格式，既有討論記錄 SHALL NOT 需要遷移。

#### Scenario: 恢復時先呈現摘要

- **WHEN** 開場檢查發現同主題的 open 討論並續用其 slug
- **THEN** 技能檔 SHALL 規定先輸出逐輪「Focus→Position 首句定論」清單與最後一輪 Open 邊界，再接續討論

#### Scenario: 多需求首輪攤清單

- **WHEN** 討論主題含多個需求（一次 5-10 個）
- **THEN** 技能檔 SHALL 規定首輪 Open 列出全部需求清單，後續每輪 Open 復述剩餘項，已定案項去向由該輪 Position 首句承載

<!-- @trace
source: discuss-grounding-and-flow
updated: 2026-08-21
-->

---
### Requirement: 記錄建檔以使用者首次回覆為觸發

技能檔 SHALL 規定：討論記錄的建立時機為第一個實質回合——由使用者的回覆（對假設清單的確認或修正、對提問的回答）使主題實際前進的那一刻；代理人自身的研究產出或首份假設清單 SHALL NOT 單獨構成建檔條件。使用者回覆前 SHALL NOT 在磁碟留下記錄檔，使誤觸發或一輪即答的主題零檔案離場。

#### Scenario: 研究再深也不先建檔

- **WHEN** 代理人完成偵察與線上查證並呈現首份假設清單，使用者尚未回覆
- **THEN** 技能檔 SHALL 規定此時不得執行 speclink discuss new，磁碟上不存在記錄檔

#### Scenario: 使用者首次回覆後建檔

- **WHEN** 使用者對假設清單給出確認或修正，或回答了第一個問題
- **THEN** 技能檔 SHALL 規定於記錄該首輪前執行 speclink discuss new 並補寫 Context，之後每輪照常 add-round

<!-- @trace
source: discuss-grounding-and-flow
updated: 2026-08-21
-->

---
### Requirement: 中途轉出教學

技能檔 SHALL 規定：多需求討論中單項談定、使用者要先立案時，代理人 SHALL 教執行 speclink discuss promote 即刻轉出（引擎於無結論時以 topic 預填提案），討論 SHALL 繼續加輪談剩餘項，SHALL NOT 要求先寫結論。最終 conclude SHALL 照常執行——引擎保留 promoted 狀態、寫入結論、並將已轉出變更標為待重新反映；技能檔 SHALL 註明該標記與最終結論無關時僅需一次確認。技能檔 SHALL 另規定分期轉出：結論規劃「之後回同一份記錄再轉出一個變更」（例：先立一刀、封存後再立下一刀）時，conclude SHALL 帶 --hold 讓記錄留在途直到下一次轉出清掉旗標；未帶 --hold 的記錄會在最後一個轉出變更封存時隨行封存，之後的刀 SHALL 走新討論。技能檔 SHALL 另註明：discard 剛清掉旗標的那個變更不會還原旗標，仍規劃下一刀時 SHALL 重跑 conclude --hold。conclude 指令範例 SHALL 標示 --hold 的用途。

#### Scenario: 單項談定即中途轉出

- **WHEN** 討論尚未結論，使用者要求先把已談定的需求轉為變更
- **THEN** 技能檔 SHALL 規定直接執行 promote 並繼續討論剩餘項，SHALL NOT 要求先 conclude 整份討論

#### Scenario: 中途轉出後補結論

- **WHEN** 中途轉出過的討論最終執行 conclude
- **THEN** 技能檔 SHALL 說明：狀態保持已轉出、結論照常寫入、先轉出的變更被標為待重新反映，與結論無關時一次確認即可

#### Scenario: 分期轉出帶 --hold

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的中途轉出段與 conclude 指令範例
- **THEN** 內容 SHALL 規定結論規劃之後回同一記錄再轉出時 conclude 帶 --hold、旗標由下一次轉出清除、未帶旗標時後續刀走新討論、discard 不還原旗標；conclude 範例 SHALL 標示 --hold 的用途；claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映


<!-- @trace
source: discussion-spinout-hold
updated: 2026-09-07T17:58:41+08:00
-->

---
### Requirement: 結論後交棒單推 propose 入口

技能檔的下一步建議段 SHALL 規定：討論已寫入結論且結論值得自己開變更時，僅建議 propose 技能的 --from-discussion 入口，SHALL NOT 於該邊並列 promote。promote 的教學 SHALL 僅保留於中途轉出段（多需求討論中單項談定、討論未完時先立案）。其餘既有出邊（結論併入既有變更走 link 與 ingest、結論為不做仍照常結案後走 archive、無實質內容走 discard）SHALL 維持不變。

#### Scenario: 結論邊僅推 propose 入口

- **WHEN** 檢視 discuss 技能資產的下一步建議段
- **THEN** 「結論值得自己開變更」的邊僅含 propose 的 --from-discussion 入口；promote 僅出現於中途轉出教學段

<!-- @trace
source: propose-apply-handoff-updates
updated: 2026-08-27
-->

---
### Requirement: 開場舊討論查核與第四類對照

內嵌 speclink-discuss 技能（事實來源 crates/speclink-core/assets/skills/discuss.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 將偵察漏斗規定為「正典 → 舊討論查核 → 程式碼」三段。舊討論查核 SHALL 規定：以使用者題目的關鍵字加正典掃描轉譯出的英文詞執行 speclink discuss search（--json），命中的決定行 SHALL 全數列出；整份 Conclusion 以 speclink discuss show 讀取 SHALL 最多 3 份、topic 命中者優先；此查核 SHALL NOT 依討論的 kind 過濾。假設清單 SHALL 在既有三分對照之外加入第四類「舊討論已定案」，細分曾否決（附當時理由；重開該方向 SHALL 說明當時理由已失效）、曾延後（可接手）、已落地（正典會照出，不重列）；技能檔 SHALL 明文不得以此擋下討論方向。Context 段 SHALL 規定加一行 `Prior discussions: <slug 清單>`，零命中時寫 none。既有討論記錄格式與 Context／Rounds／Conclusion 骨架 SHALL 不變。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含舊討論查核

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-discuss 技能檔 SHALL 將偵察描述為「正典 → 舊討論查核 → 程式碼」，舊討論查核 SHALL 指示執行 speclink discuss search、命中決定行全列、整份 Conclusion 最多讀 3 份且 topic 命中優先，並 SHALL NOT 含依 kind 過濾的指示

#### Scenario: 曾否決方向重開須說明理由失效

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的假設清單對照規則
- **THEN** 技能檔 SHALL 含第四類「舊討論已定案」及曾否決、曾延後、已落地三種細分，規定重開曾否決方向須說明當時理由已失效，且 SHALL 明文不得以此擋下討論方向

#### Scenario: Context 記錄舊討論來源

- **WHEN** 檢視渲染產出的 speclink-discuss 技能檔的 Context 撰寫規定
- **THEN** 技能檔 SHALL 規定 Context 加一行 `Prior discussions: <slug 清單>`，零命中時寫 none

<!-- @trace
source: discuss-search-recall
updated: 2026-09-05
-->