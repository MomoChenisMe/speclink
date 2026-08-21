## Summary

重寫 speclink-discuss 技能的討論流程：偵察改為「正典先行」的漏斗式接地，全域雙模式分流改為需求清晰度分流（grill 磨需求階段＋assumptions 預設姿態＋逐節點退路），並補上多需求討論的 backlog 慣例、恢復摘要儀式與中途轉出教學。零引擎改動，只改技能資產文字。

## Motivation

兩份已結論討論（discuss-spec-grounding、discussion-backlog-spinoff）定案了五項改進，動機各有實據：

- 現行偵察明文排除規格（"not docs, not tests"），但 Context 模板卻期待記下 related specs——期待存在、產出步驟不存在。討論結論與正典衝突時，最快要到 propose 第 3 步（只顯示不擋）、最糟到 archive 合併閘門才現形，代價差一整條 pipeline。
- 檔案數模式門檻（3 檔以上走 assumptions）在成熟 codebase 幾乎永遠成立，interview 近乎死路徑；且 interview 的原始意圖（grillme：先磨利需求再列假設）被誤植成「程式碼不足時的退路」，觸發軸放錯。
- 一次討論 5-10 個需求時，Open 欄位散文跨輪分散，回來續談要重讀全部輪次；引擎早已支援中途轉出（promote 無結論以 topic 預填、add_round 無已結論閘門、re-conclude 保留 promoted 並標 stale），但技能檔沒教。
- 實測回饋（2026-08-21，另一專案首場新流程討論）：代理把「自身研究有實質結論」誤讀為 substantive，在使用者回覆前就建了記錄——「first substantive round」的兩個舉例都隱含使用者回應，但條文沒有明文排除代理自身產出，延後建檔（誤觸發零檔案離場）的原意可被繞過。

## Proposed Solution

改動集中在 crates/speclink-core/assets/skills/discuss.md（事實來源），五項：

1. **漏斗式接地**：偵察先跑 speclink list --specs --json（候選 ≤5、讀 Purpose ≤3、主題直接動到的 capability 才讀全文、零命中靜默略過），用命中的 capability 名與正典詞彙把搜尋詞從使用者語言轉譯成系統語言後再進程式碼；主題已含具體檔名或符號時程式碼軌直接開跑。三分對照（正典已涵蓋／與正典衝突／正典沒講）寫進假設清單。紀律：使用者需求是目標，正典是證據、不是裁決；偏離正典允許，但寫進記錄。
2. **需求清晰度分流**：廢除檔案數模式門檻。需求鈍（無可驗證目標、無門檻、improve/better 類措辭）先走 grill 階段——一次一題磨需求（目標、範圍、門檻、成功判準），題目附現況或正典證據框題；需求已利時 grill 塌縮為零題。磨利後進 assumptions（唯一預設姿態）；假設中證據撐不起的節點就地化為單一問題並附最佳猜測。決策樹遍歷紀律（依依賴順序、一次一題、使用者主導停止）不變。
3. **backlog 慣例**：多需求討論第一輪把全清單攤進 Open，之後每輪 Open 復述剩餘項；已定案項去向由該輪 Position 首句承載。
4. **恢復摘要儀式**：續用 open 討論時，先呈現「逐輪 Focus→Position 首句定論」清單＋最後一輪 Open 邊界，再繼續討論。摘要自既有欄位機械推導，零新格式、既有記錄零遷移。
5. **中途轉出教學**：談定一項即可 promote（引擎以 topic 預填提案），討論繼續加輪，最後補 conclude 保留 promoted 狀態並將已轉出變更標為待重新反映；技能檔註明結論與先轉出變更無關時該標記多一次確認即可。
6. **建檔時機釘死在使用者回覆**（實測回饋補項）：「first substantive round」明文定義為使用者的回覆使主題前進的那一刻（確認、修正、回答）；代理人自身的研究產出或首份假設清單不算 substance，使用者回覆前不得建檔。

## Non-Goals

- 不動引擎：discuss 動詞（new/add-round/conclude/promote/link/seal）行為零改動；stamp_restale 不做按項目選擇性標記（複雜度不成比例）。
- 不動記錄格式：Context／Rounds／Conclusion 骨架與 Focus／Position／Ruled out／Open 欄位不變，不新增結構化 backlog 區段，既有記錄不遷移。
- 不動 propose skill：propose 第 3 步規格掃描保留（直接 propose 與 --from-doc 不經討論，防線不能只剩一道）。
- in-flight change 的 delta 掃描範圍與引擎側命名防護——讓給討論 capability-naming-dedup 決定。

## Alternatives Considered

- 平行雙軌偵察：程式碼軌仍用使用者原始關鍵字，浪費正典的轉譯價值——改為漏斗。
- 完全移除 interview：丟失磨需求功能，對鈍需求列假設是猜心不是研究——改為還原 grillme 原意。
- 正典當裁決（掃到衝突即擋）：扼殺新方向——降為證據。
- 結構化 backlog 區段（動引擎動骨架）：恢復摘要可自既有欄位推導——多餘結構。
- 兩討論分拆兩個變更：同檔同性質資產，互相等待——合流一案。

## Impact

- Affected specs: discuss-skill（3 條 MODIFIED：interview 模式以決策樹遍歷提問、interview 每題附建議答案與 Evidence、事實與決策分診及逐節點查證；4 條 ADDED：正典接地與三分對照、多需求 backlog 與恢復摘要、中途轉出教學、記錄建檔以使用者首次回覆為觸發）
- Affected code:
  - Modified: crates/speclink-core/assets/skills/discuss.md
  - Modified（版號 bump）: crates/speclink-core/src/init.rs
  - Modified（衍生物，隨資產再生）: crates/speclink-core/tests/golden/claude.snapshot.md, crates/speclink-core/tests/golden/claude-worktree.snapshot.md, crates/speclink-core/tests/golden/codex.snapshot.md, crates/speclink-core/tests/golden/neutral-cli.snapshot.md, crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md, crates/speclink-core/tests/golden/assets.lock, crates/speclink-core/tests/golden/remote-claude.marker.md（純版號）, AGENTS.md（純版號）, CLAUDE.md（純版號）
  - Modified（review 全修回合追加）: crates/speclink-core/assets/skills/improve.md（interview 引用改為 question discipline）, crates/speclink-core/src/discuss.rs（記錄骨架 Context 註解與技能同步——經使用者裁定踩「零引擎改動」Non-Goal，僅註解字串、無行為改動）
  - New: (none)
  - Removed: (none)
- 收尾註記：speclink update 再生的工具技能目錄 SKILL.md（約 32 份）不進 evidence，收尾以 git status 盤點帶上。
