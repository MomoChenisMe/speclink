## Why

外部協作測試把兩個「規格如此」的行為誤讀成缺陷，暴露文件與正典的三處縫：(1) workflow 文件把工單列為品質站產物、卻沒說蓋章會在同一原子寫入內刪除工單，讀者自然預期封存裡有 review.md 與 verify.md；(2) 兩站正典承諾「SUGGESTION 紀錄留在工單的 git 歷史」，remote 模式無 git、store 只留 digest 與墓碑，該承諾不可成立；(3) trace 在 remote 模式明確拒絕是刻意行為（v1 Non-Goal），但 verb-contract 正典與文件的本質本機動詞只列 demo、拒絕行為零測試釘住，remote-getting-started 還把非成員讀取寫成 404（程式碼與 server-identity 正典為 403）。（源討論：remote-fix-plan-gaps，刀 3）

## What Changes

- workflow 中英文件補「蓋章消耗工單」敘述：同一原子寫入刪工單＋寫章、封存的已蓋章 change 不含工單檔、僅未結工單經 carry 旗標隨封存移動、remote 模式蓋章後工單文字不可回讀。
- review-station 與 verify-station 正典的「僅 SUGGESTION 的末輪乾淨蓋章」場景修訂：git 歷史保留承諾限定 fs 模式，remote 模式明定工單文字不保留。
- verb-contract 正典的本質本機動詞自「demo」改列「demo、trace」，補「trace 於 remote 明確拒絕」場景；docs/verb-contract 中英文件的 FsOnly 列同步補 trace。
- remote-getting-started 中英文件把非成員讀取的 404 敘述修正為 403（對齊 server-identity 正典的 permission_denied）。
- 補 trace 的 FsOnly 拒絕釘住測試（與 demo 既有釘住同形）；修正 CLI dispatch 表的過期動詞計數註解。

## Non-Goals

- 不改蓋章刪工單的行為本身——那是正典明定契約，兩模式一致，本刀只補揭露。
- 不做「工單內文折進封存產物」或「history 存內文」的產品路線——工單是工作文件、結論已在 stamps；稽核需求出現再另議。
- 不做 remote trace 功能本體——維持 v1 Non-Goal 的 backlog，本刀只正名拒絕行為。
- 不動 server 與引擎的任何行為碼——本刀是文件、正典與釘住測試。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `review-station`: 「僅 SUGGESTION 的末輪乾淨蓋章」場景的 git 歷史承諾限定 fs 模式並明定 remote 不保留工單文字。
- `verify-station`: 同 review-station 的對應場景修訂。
- `verb-contract`: 本質本機動詞列表補 trace，新增 remote 拒絕場景。
- `user-documentation`: 新增品質站蓋章效果與非成員錯誤碼的文件揭露要求。

## Impact

- Affected specs: review-station, verify-station, verb-contract, user-documentation
- Affected code:
  - Modified: docs/workflow.md, docs/workflow.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, docs/remote-getting-started.md, docs/remote-getting-started.zh-TW.md, crates/speclink-cli/tests/it/mode_dispatch.rs, crates/speclink-cli/src/main.rs
  - New: (none)
  - Removed: (none)
