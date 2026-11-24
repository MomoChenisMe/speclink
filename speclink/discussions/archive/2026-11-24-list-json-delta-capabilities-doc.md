---
topic: 文件缺口：list --json 的 deltaCapabilities 沒有寫進 verb-contract
slug: list-json-delta-capabilities-doc
status: concluded
created: 2026-11-24
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 文件缺口：list --json 的 deltaCapabilities 沒有寫進 verb-contract

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

接續 embedded-host-needs-7：內嵌宿主升到 0.17.0、畫面讀取全改走動詞後，發現 docs/verb-contract.md（與 .zh-TW.md）對 `speclink list --json` 的欄位描述和實際輸出不一致，宿主會因此以為 `list --json` 讀不到每個變更的 capability，改成逐一讀 `show --json` 或走 HTTP。需求本身清楚（要補哪幾處、中英同步），不需要先釐清需求，以文件輸入的分診進行。相關正式規格為 change-lifecycle（list --json 曝 deltaCapabilities 的承諾）、verb-contract（技能以 list --json 的 deltaCapabilities 判斷 delta）、server-verb-api、client-protocol 與 user-documentation（中英文件結構對等）；目前沒有進行中的變更。
Prior discussions: embedded-host-needs-7, docs-cleanup-findings
Source doc: /Users/momochenisme/Documents/GitHub/speclink-doc-gap-list-delta-capabilities.md

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-24)

**Focus**: 來源文件的每一條說法和程式碼對不對得上
**Position**: 文件的說法全部成立，而且實際缺口比文件列的大；沒有任何一條要使用者裁定的事實。
- 確認：`list --json` 與 `--detail` 都帶 `deltaCapabilities`，空清單不印（`crates/engine/speclink-core/src/lifecycle/listing.rs` 的 `delta_capabilities` 欄位）
- 確認：依名稱遞增，本機 `crates/engine/speclink-fs/src/lib.rs` 與內嵌宿主 `crates/host/speclink-host/src/port_view.rs` 的 `capabilities_under` 都排序
- 確認：三種形態都輸出——都走同一個 `ListChangeJson`；遠端由 `crates/adapters/speclink-cli/src/verbs/query.rs` 的 `to_list_change_json` 抄 `GET /changes` 的同名欄位，舊 Server 沒送就不出現、指令不失敗
- 確認：中英文兩份的「New fields of `list --json` and `show --json`」表格與「Optional fields in lists and details」的「只印 `created`、`modifiedAt`、`fields`」那句都漏寫；`--detail` 表不用動
- 確認：「若不打算成為穩定欄位請寫明」不必處理——change-lifecycle 規格已以 SHALL 承諾 `list --json` 曝 `deltaCapabilities`，verb-contract 規格也規定技能以它判斷 delta
- 文件沒提到的落差一：「只印三個鍵」那句同樣漏了 `restaleFrom` 與 `metaError`，兩者也會印（listing.rs 的同名欄位）
- 文件沒提到的落差二：`summary` 在 `list --json` 不是必有——摘要是空字串時不印（`skip_serializing_if = "String::is_empty"`），遠端收到空字串也不印；必有的只有 `completedTasks`、`name`、`status`、`totalTasks`
- 文件沒提到的落差三：worktree 一節只說 `completedTasks`、`totalTasks`、`status`、`metaError` 取自 worktree 副本，實際上整個項目都是，含 `deltaCapabilities`（`crates/host/speclink-host/src/worktree.rs`）；只影響本機
- 決策樹：修文件的範圍（D1）→ 做法：直接 docs commit 或開變更（D2，依 D1）→ 是否加守門測試防止再脫節（D3）
**Open**: D1 範圍只補 deltaCapabilities 還是三個落差一起補；D2 做法；D3 要不要加守門測試

### Round 2 — interview (2026-11-24)

**Focus**: D1：修文件的範圍
**Position**: 三個落差和 `deltaCapabilities` 一起補，中英文兩份同步，使用者照建議定案。
- 「New fields」表格加 `deltaCapabilities`：有 delta spec 時出現、依名稱遞增、舊 Server 沒送就不出現
- 表格上方加一句：必有的鍵是 `completedTasks`、`name`、`status`、`totalTasks`，`summary` 只在摘要不是空字串時出現
- 「只印三個鍵」那句改成列出全部會印的鍵（`restaleFrom`、`metaError`、`deltaCapabilities`、`created`、`modifiedAt`、`fields`），並寫明 `summary` 和 `GET /changes` 的差別
- worktree 一節改成「這一項的值全部取自 worktree 副本」，點名 `deltaCapabilities`
**Ruled out**: 只補 `deltaCapabilities` — 「只印三個鍵」那句仍是錯的，來源文件也已在問有沒有其他漏寫的鍵，等於再來一次
**Open**: D2 做法：直接 docs commit 或開變更；D3 要不要加守門測試

### Round 3 — interview (2026-11-24)

**Focus**: D2：用什麼方式修文件
**Position**: 直接做一個 docs commit，不開 speclink 變更，使用者同意。
- 沒有規格承諾要改：change-lifecycle 已承諾 `list --json` 曝 `deltaCapabilities`，這次只讓文件追上規格與程式；只改段落內文與表格，不動 H2，user-documentation 的中英結構對等不受影響
- 先例：提交 45ad753f `docs(remote)` 直接對齊文件；docs-cleanup-findings 也排除過為文字對齊另開變更
- 收尾：沒有變更可連結，先寫結論，修完後以 `speclink discuss archive` 封存
**Ruled out**: 開一個 speclink 變更 — 兩個檔案四處文字，proposal、tasks、兩站品質關卡與封存的成本遠大於修改本身；追溯理由由這份封存的討論記錄承擔
**Open**: D3 要不要加守門測試防止文件再脫節

### Round 4 — interview (2026-11-24)

**Focus**: D3：怎麼防止文件再和 `--json` 的鍵脫節
**Position**: 不加守門測試，改在 `speclink/config.yaml` 的 `rules.tasks` 加一條產出規則，由使用者提議、照建議寫法定案。
- 規則文字：「新增、改名或移除 `--json` 輸出或 HTTP 回應的欄位時，必須有一項 task 同步 docs/verb-contract.md 與 docs/verb-contract.zh-TW.md：搜尋同一個欄位與同一個輸出在文件裡的每一處（例如 `list --json` 會印的鍵、`GET /changes` 的欄位表），逐一補齊」
- 位置：緊接在既有的 sdk-node 同步規則之後，寫法沿用該條
- 「每一處」是重點：這次 `deltaCapabilities` 已寫進 `GET /changes` 的表格，漏的是「`list --json` 會印哪些鍵」那句
- 限制：只是寫 tasks 時的提醒，不擋任何東西；不開變更的直接 commit 看不到它
**Ruled out**: 以 `ListChangeJson` 的鍵名做子字串比對的守門測試 — `deltaCapabilities`、`restaleFrom`、`metaError` 原本就出現在文件的 `GET /changes` 表格，這次的缺口它會直接通過；解析「只印哪些鍵」那句的守門 — 中英句型不同，改寫句子就會壞，太脆弱
**Open**: none

## Conclusion

**Decision**: 不開變更，以一批直接 commit 補齊 docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 對 `list --json` 的欄位描述，並在 `speclink/config.yaml` 的 `rules.tasks` 加一條同步規則；不加守門測試。
- 事實基準：`list --json`（含 `--detail`）在本機、遠端與內嵌宿主三種形態都帶 `deltaCapabilities`，依名稱遞增，沒有 delta spec 時不出現；遠端取自 `GET /changes` 的同名欄位，舊 Server 沒送就不出現、指令不失敗；這是 change-lifecycle 規格承諾的穩定欄位，文件不必寫「不穩定」
- 「New fields of `list --json` and `show --json`」表格加 `deltaCapabilities`：有 delta spec 時出現、依名稱遞增、舊 Server 沒送就不出現
- 同一表格上方加一句：必有的鍵是 `completedTasks`、`name`、`status`、`totalTasks`；`summary` 只在摘要不是空字串時出現
- 「Optional fields in lists and details」裡「`list --json` 只印 `created`、`modifiedAt`、`fields`」那句改成列出全部會印的鍵：`restaleFrom`、`metaError`、`deltaCapabilities`、`created`、`modifiedAt`、`fields`，並寫明 `summary` 在 `GET /changes` 必有、在 `list --json` 為空字串時不印
- 「The local `worktree` field of `speclink list --json`」一節改成「這一項的值全部取自 worktree 副本」，點名 `deltaCapabilities`
- `--detail` 的欄位表不改；只改段落內文與表格，不加減 H2，中英兩份同步
- 例：一個有 `auth`、`billing` 兩份 delta spec、proposal 沒有摘要文字的變更，`list --json` 的這一項是 `{"completedTasks":0,"name":"add-x","status":"in-progress","totalTasks":2,"deltaCapabilities":["auth","billing"],"created":"2026-11-24"}`，沒有 `summary`
- `rules.tasks` 緊接在 sdk-node 同步規則之後加：「新增、改名或移除 `--json` 輸出或 HTTP 回應的欄位時，必須有一項 task 同步 docs/verb-contract.md 與 docs/verb-contract.zh-TW.md：搜尋同一個欄位與同一個輸出在文件裡的每一處（例如 `list --json` 會印的鍵、`GET /changes` 的欄位表），逐一補齊」
- 修完跑 `node --test scripts/*.test.mjs scripts/*/*.test.mjs`（中英對照與詞彙守門掃整個 docs）
**Rationale**: 這是文件追不上程式與規格的缺口，不改任何規格承諾，所以直接修比開變更便宜；只補 `deltaCapabilities` 會讓「只印哪些鍵」那句仍然錯，因此同批補齊程式碼查出的其他落差；防止再發生的關鍵是加新鍵的變更在規劃時就排進同步文件的任務，產出規則在那個時間點提醒，比事後的守門測試更對準原因。
**Rejected alternatives**:
- 只補 `deltaCapabilities` — `restaleFrom`、`metaError` 與 `summary` 的落差還在，宿主要再問一次
- 開一個 speclink 變更 — 兩個檔案四處文字加一行設定，proposal、tasks、兩站品質關卡與封存的成本遠大於修改本身
- 以鍵名子字串比對的守門測試 — 這三個鍵原本就寫在 `GET /changes` 表格，這次的缺口它會直接通過
- 解析「只印哪些鍵」那句的守門測試 — 中英句型不同，改寫句子就會壞
**Deferred**: none
**Capture to**: none — 直接修改 docs/verb-contract.md、docs/verb-contract.zh-TW.md 與 speclink/config.yaml
**Next**: 離開討論模式後直接修改並 commit，再執行 `speclink discuss archive list-json-delta-capabilities-doc`
