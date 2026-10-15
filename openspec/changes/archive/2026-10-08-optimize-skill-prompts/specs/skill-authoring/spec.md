## Purpose

技能原稿的寫法規範與品質量測：規定 speclink-core 內嵌的技能原稿要怎麼寫，才能讓 Claude、Codex、Copilot 與自訂描述子接上的模型都照做；哪些規範由測試機械擋住；以及如何用實跑評測比較技能的新舊版本。本 capability 只管技能文字的寫法與量測，不管單一技能的流程行為（屬各逐技能規格），也不管跨技能的入口與交棒路由（屬 skill-routing）。

## ADDED Requirements

### Requirement: 技能原稿的跨模型寫法規範

crates/engine/speclink-core/assets/skills/ 下每一份技能原稿 SHALL 依下列十條撰寫：

1. 每條規則在同一份技能中 SHALL 只完整陳述一次，寫在模型執行該動作的步驟裡；其他位置需要提及時 SHALL 只用一句話指向該處。
2. 規則之間 SHALL NOT 互相矛盾；一條規則有例外時，SHALL 寫明哪一條優先。
3. 全大寫強調字（MUST、NEVER、SHALL、ALWAYS、IMPORTANT、CRITICAL、STOP、NOT）SHALL 只用於鐵律：做錯會毀損資料、無法還原、必須停下等使用者，或引擎會靜默誤讀的固定格式。
4. 同一概念 SHALL 只用一個詞。
5. 依工具而不同的能力 SHALL 附退路：指示使用 AskUserQuestion 時，SHALL 寫明工具不可用時以純文字詢問並等待回覆；指示開子代理時，SHALL 寫明不能開子代理時由主執行緒自己執行。
6. 多步驟流程 SHALL 以編號步驟呈現；判斷點 SHALL 寫成「條件 → 動作」；固定格式的產出 SHALL 附樣板；技能結尾 SHALL 附產出的自我檢查清單。
7. 技能 SHALL NOT 含歷史資訊：design 決策編號、日期、某條規則由哪個變更加入。示範格式用的時間戳放在程式碼區塊或行內程式碼中，不算歷史資訊。
8. 單一原稿檔 SHALL NOT 超過 500 行。由多份原稿組成的技能（apply-with-worktree）以各份原稿分別計算。
9. 判斷標準的原文 SHALL NOT 為了縮短篇幅而濃縮；縮短只能來自刪除重複陳述、刪除歷史資訊與重整結構。
10. 技能之間的引用 SHALL 寫成 `/speclink:<名稱>`，指令中 `--agent` 的值 SHALL 寫成 `{{TOOL}}`，由渲染依工具換成各自的寫法；提到 plan mode 的內容 SHALL 獨立成一行，因為自訂描述子的渲染會整行刪除提到 plan mode 的行。

本規範約束技能文字的寫法，SHALL NOT 改變技能所描述的流程行為；逐技能規格要求技能寫到的內容 SHALL 全部保留。

#### Scenario: 同一規則只完整陳述一次

- **WHEN** 檢視任一份技能原稿中的任一條規則
- **THEN** 該規則的全文在原稿中只出現一次，位於模型執行該動作的步驟；其他提及處只有一句指向該步驟的話

#### Scenario: 有例外的規則寫明優先序

- **WHEN** 一份技能同時要求「持續做完全部任務」與「遇到手動任務時停下」
- **THEN** 技能寫明遇到手動任務時，停下的規則優先

#### Scenario: 工具相依的能力附退路

- **WHEN** 一份技能指示用 AskUserQuestion 詢問使用者
- **THEN** 同一份技能寫明 AskUserQuestion 不可用時，以純文字提出同樣的問題並等待使用者回覆

#### Scenario: 示範用的時間戳不算歷史

- **WHEN** 一份技能需要示範 RFC 3339 時間戳的格式
- **THEN** 時間戳放在程式碼區塊或行內程式碼中；技能的散文不含 design 決策編號，也不含「某條規則由哪個變更加入」的說明

#### Scenario: 技能引用依工具渲染

- **WHEN** 原稿寫著 `/speclink:ingest`，分別以 claude、codex、copilot 與自訂描述子渲染
- **THEN** 四份技能檔中的引用依序為 `/speclink-ingest`、`$speclink-ingest`、`/speclink-ingest`、`speclink ingest`；Codex 渲染的技能檔不含 `/speclink-` 形式的技能引用

##### Example: 同一個引用在四種渲染目標

| 渲染目標 | 原稿 `/speclink:ingest` 的渲染結果 |
| --- | --- |
| claude | `/speclink-ingest` |
| codex | `$speclink-ingest` |
| copilot | `/speclink-ingest` |
| 自訂描述子 | `speclink ingest` |

#### Scenario: 判斷標準不濃縮

- **WHEN** 改寫一份含判斷標準清單的技能（例如 improve 的摩擦訊號與刪除測試）
- **THEN** 判斷標準的原文保留；篇幅的減少只來自刪除重複陳述、刪除歷史資訊與重整結構

### Requirement: 寫法規範的機械守門

speclink-core 的整合測試（`cargo test -p speclink-core --test it`）SHALL 列出 crates/engine/speclink-core/assets/skills/ 下全部 `.md` 原稿，逐份檢查下列五項；任一項不符時測試 SHALL 失敗（非零 exit code），失敗訊息 SHALL 指名原稿檔名、違反的項目，以及規範出處「skill-authoring 規格『技能原稿的跨模型寫法規範』」：

1. **行數**：原稿行數 SHALL NOT 超過 500。行數以行為單位計算，LF 與 CRLF 結尾 SHALL 得到相同結果。
2. **強調字上限**：以大小寫敏感、字詞邊界比對 MUST、NEVER、SHALL、ALWAYS、IMPORTANT、CRITICAL、STOP、NOT 的出現次數，次數 SHALL NOT 超過測試內登記表為該原稿登記的上限。登記表 SHALL 涵蓋 assets/skills/ 下每一份原稿；出現未登記的原稿時測試 SHALL 失敗，並要求登記上限。上限只在修改登記表時改變。
3. **不寫歷史**：移除 fenced code block 與行內程式碼之後，原稿 SHALL NOT 含 `YYYY-MM-DD` 形式的日期（前後不接數字即算，緊貼中文或後接時間的日期也算）；原稿全文 SHALL NOT 含 `design D` 加數字的決策編號，也 SHALL NOT 含「design 決策」字樣。
4. **工具退路**：原稿含 `AskUserQuestion` 時，SHALL 含 `plain text`（不分大小寫）；原稿含 `sub-agent`、`subagent` 或 `Agent tool` 時，SHALL 含 `cannot spawn` 或 `can't spawn`（不分大小寫）。單獨的 `inline` 不算退路：無關的句子也常含這個字。
5. **渲染記號**：原稿 SHALL NOT 含寫死的技能引用——前一個字元不是英數字、底線、`.`、`/` 或 `:` 的 `/speclink-` 加小寫字母（`.claude/skills/speclink-apply` 這類路徑不算）；原稿 SHALL NOT 含 `$speclink-`，也 SHALL NOT 含 `--agent claude`、`--agent codex` 或 `--agent copilot`。

#### Scenario: 強調字超過登記上限

- **WHEN** 在 review.md 多加一個 MUST，使 review.md 的強調字次數比登記上限多 1，再執行 `cargo test -p speclink-core --test it`
- **THEN** 測試失敗，訊息含 review.md、「強調字上限」、登記上限與實際次數，以及規範出處

#### Scenario: 強調字減少時通過

- **WHEN** 從 review.md 刪掉一個 NEVER，登記表不變
- **THEN** 測試通過

##### Example: 強調字的計次

| 原稿片段 | 計入次數 | 理由 |
| --- | --- | --- |
| `Do NOT stop here` | 1 | NOT 為全大寫的獨立字詞 |
| `This is not optional` | 0 | 小寫不計 |
| `See the NOTE below` | 0 | NOTE 不是 NOT（字詞邊界） |
| `STOP and ask. NEVER guess.` | 2 | STOP 與 NEVER 各 1 |

#### Scenario: 原稿超過 500 行

- **WHEN** 某份原稿有 501 行
- **THEN** 測試失敗，訊息含該原稿檔名、「行數」與實際行數 501

#### Scenario: 程式碼內的時間戳不誤判

- **WHEN** manual.md 的 fenced code block 中有 `generated: 2026-09-05T23:31:00+08:00`，散文中沒有日期
- **THEN** 不寫歷史的檢查通過

#### Scenario: 散文中的日期失敗

- **WHEN** 某份原稿的散文寫著「此規則於 2026-08-10 加入」
- **THEN** 測試失敗，訊息含該原稿檔名、「不寫歷史」與命中的日期

#### Scenario: 缺少工具退路

- **WHEN** 某份原稿含 `AskUserQuestion` 但全文不含 `plain text`
- **THEN** 測試失敗，訊息含該原稿檔名與「工具退路」

#### Scenario: 寫死的技能引用

- **WHEN** 某份原稿寫著「run `/speclink-apply <name>`」
- **THEN** 測試失敗，訊息含該原稿檔名、「渲染記號」與命中的 `/speclink-apply`

#### Scenario: 技能目錄路徑不誤判

- **WHEN** 某份原稿寫著 `.claude/skills/speclink-propose/SKILL.md`，沒有其他寫死的引用
- **THEN** 渲染記號的檢查通過

#### Scenario: 新原稿未登記上限

- **WHEN** assets/skills/ 新增一份 `example.md`，登記表沒有它
- **THEN** 測試失敗，訊息含 `example.md` 並要求在登記表登記強調字上限

### Requirement: 技能評測套件

repo SHALL 備有技能評測套件 integrations/claude-code/skill-evals/：一個含 `.claude-plugin/plugin.json` 的 Claude Code plugin，評測題目放在 `evals/<題目名>/`，以 `claude plugin eval` 執行。每題 SHALL 只驗單輪；每題的 prompt.md frontmatter SHALL 以 `tags` 標示 `skill:<技能短名>`。repo 內 `.claude/skills/` 下的每個 `speclink-<短名>` 技能 SHALL 至少有一題標示 `skill:<短名>`；`node --test scripts/claude-code/skill-evals.test.mjs` SHALL 檢查這項涵蓋度，缺題時失敗並列出缺題的技能短名。

評測執行腳本 `scripts/claude-code/skill-evals.mjs` SHALL 提供兩個子指令，輸出皆為不帶顏色的純文字，正常結果寫到 stdout、錯誤寫到 stderr，不讀 stdin：

- `run --speclink <執行檔路徑> --model <模型 ID> --label <名稱> --max-cost-usd <金額>`：SHALL 讓評測工作區內執行的 `speclink` 都是指定的執行檔（同時決定受測的技能版本與 CLI 版本），SHALL 把該執行檔 `--version` 的輸出寫進 `target/skill-evals/<名稱>/speclink-version.txt`，SHALL 以每題 3 次、不跑無技能對照組、指定模型與花費上限執行評測，結果寫進 `target/skill-evals/<名稱>/`。評測工具因花費上限或認證失敗而以 2 結束時，`run` SHALL 以 2 結束。四個參數缺任何一個時，`run` SHALL 以非零 exit code 結束並在 stderr 說明缺少的參數。
- `compare <舊結果目錄> <新結果目錄>`：SHALL 讀兩個目錄的 `aggregate-result.json`，逐題在 stdout 印出題目、舊分數、新分數與差值。任一題新分數低於舊分數時 SHALL 以 1 結束並列出退步的題目；兩邊題目集合不同時 SHALL 以 2 結束並列出缺少的題目；否則 SHALL 以 0 結束。無法比較時 SHALL 以 3 結束、在 stderr 指名該檔案，且 stdout 不印比較結果；無法比較指任一個 `aggregate-result.json` 不存在、不是 JSON，或因花費上限而不完整（頂層 `partial` 為 true，或任一次執行的 `skippedPaidGraders` 為 true：該次略過了付費評分，分數不能和完整結果比）。

執行腳本需要 claude CLI 與使用者的額度，SHALL NOT 在 CI 執行；只有 skill-evals.test.mjs 隨 `node --test` 在 CI 執行。

#### Scenario: 技能缺評測題

- **WHEN** `.claude/skills/` 下有 `speclink-trace`，而 evals 下沒有任何題目的 tags 含 `skill:trace`
- **THEN** `node --test scripts/claude-code/skill-evals.test.mjs` 失敗，訊息列出 `trace`

#### Scenario: 比較結果有退步

- **WHEN** 執行 `node scripts/claude-code/skill-evals.mjs compare target/skill-evals/old-haiku target/skill-evals/new-haiku`，兩邊題目相同且至少一題新分數較低
- **THEN** stdout 逐題列出新舊分數與差值，並列出退步的題目；exit code 為 1

##### Example: 比較的判定

| 題目 | 舊分數 | 新分數 | 判定 |
| --- | --- | --- | --- |
| propose-from-discussion | 0.67 | 1.00 | 進步 |
| apply-leaves-manual-task | 1.00 | 1.00 | 持平 |
| commit-gate-shows-plan | 1.00 | 0.67 | 退步，exit 1 |

#### Scenario: 比較結果沒有退步

- **WHEN** 兩邊題目相同，每題新分數都大於或等於舊分數
- **THEN** stdout 逐題列出新舊分數與差值；exit code 為 0

#### Scenario: 題目集合不同

- **WHEN** 舊結果有 trace-cites-archive 這一題，新結果沒有
- **THEN** stdout 列出缺少的題目 trace-cites-archive；exit code 為 2

#### Scenario: 結果檔不存在

- **WHEN** 執行 compare 時，新結果目錄沒有 aggregate-result.json
- **THEN** stderr 指名缺少的檔案路徑；exit code 為 3

#### Scenario: 結果因花費上限而不完整

- **WHEN** 執行 compare 時，新結果的 aggregate-result.json 頂層 `partial` 為 true
- **THEN** stderr 指名該檔案；stdout 不印比較結果；exit code 為 3

#### Scenario: run 以指定執行檔決定受測版本

- **WHEN** 執行 `node scripts/claude-code/skill-evals.mjs run --speclink target/skill-evals/speclink-old --model claude-haiku-5-5 --label old-haiku --max-cost-usd 20`
- **THEN** `target/skill-evals/old-haiku/speclink-version.txt` 記錄 target/skill-evals/speclink-old 的 `--version` 輸出；評測工作區內 scaffold 與受測模型執行的 `speclink` 都是這個執行檔；評測結果寫進 `target/skill-evals/old-haiku/`

#### Scenario: run 缺少參數

- **WHEN** 執行 `node scripts/claude-code/skill-evals.mjs run --speclink target/debug/speclink --label new-opus`，沒有 --model 與 --max-cost-usd
- **THEN** stderr 說明缺少 --model 與 --max-cost-usd；exit code 非零；不啟動評測
