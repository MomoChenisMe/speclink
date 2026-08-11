## MODIFIED Requirements

### Requirement: 動詞 --json 輸出形狀凍結

<!-- REMOVED-SCENARIO: instructions apply 兩模式同形含新欄位 -->

動詞的 --json 輸出欄位集合與 camelCase 命名 SHALL 維持既有契約不變;工單原文 SHALL NOT 出現在任何 --json 輸出。manual-task-marker 系列的兩次刻意契約更新:instructions apply 的任務項增列 `manual` 欄位、progress 增列 `codeTotal`/`codeComplete`/`codeRemaining`;隨 `[P]` 語意移除,任務項的 `parallel` 欄位 SHALL 自基線移除——任務項欄位集合為 `id`/`description`/`done`/`manual`。除上述刻意更新外 SHALL NOT 改名或移除既有欄位;更新後的欄位集合即凍結契約的新基線。

#### Scenario: 工單 --json 兩模式同形且無原文欄位

- **WHEN** 分別於本機與 remote 模式執行 review show --json
- **THEN** 兩模式 payload 欄位集合一致——change、rounds、lastRound,rounds 各項含 index、phase、patchHash、scope、findings——且不存在攜帶工單原文的欄位

#### Scenario: instructions apply 兩模式同形且無 parallel 欄位

- **WHEN** 分別於本機與 remote 模式對同一 change 執行 instructions apply --json
- **THEN** 兩模式任務項欄位集合皆為 id/description/done/manual(無 parallel),progress 皆含 total/complete/remaining 與 codeTotal/codeComplete/codeRemaining,欄位集合一致
