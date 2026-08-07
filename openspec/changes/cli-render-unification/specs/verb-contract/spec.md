## ADDED Requirements

### Requirement: 動詞人眼輸出的兩模式同形

CLI 動詞的人眼輸出（stdout 文本，含 --no-color 模式）在本機與 remote 兩模式 SHALL 逐位元一致，僅下列明文分歧清單除外，清單外的任何輸出差異 SHALL 視為缺陷：

1. new change 的 Path 行——本機印、remote 不印（server 端路徑對本機使用者無意義）
2. list 的 worktree 標示——remote 恆缺席（worktree 是本機主 checkout 的觀察面）
3. status 的 schema 覆寫旗標——remote 以固定訊息明確拒絕（server 的 workflow config 決定 schema）
4. workflow-config 的文件標籤——remote 以 config.yaml 為標籤（server 端無本機路徑可印）

同形範圍涵蓋 list、discuss 全部子指令、task done 與 task undone、in-progress remove、discard、archive、review 與 verify 的 add-round／stamp／discard／show。模式差異 SHALL 只存在於資料取得與守門拒絕，SHALL NOT 存在於輸出文本的組版。

#### Scenario: list 的 invalid 標記兩模式同形

- **WHEN** 專案含一筆 metadata 損壞的變更，分別於本機與 remote 模式執行 list
- **THEN** 兩模式 stdout 均在該變更行尾渲染 invalid 標記，整段文本逐位元一致，exit code 均為 0

#### Scenario: discuss 動詞成功訊息兩模式同形

- **WHEN** 分別於本機與 remote 模式執行 discuss 子指令（例如 new、add-round、conclude、archive）
- **THEN** 兩模式的成功訊息文本逐位元一致，exit code 一致

#### Scenario: 封存與工單閱讀對新 server 同形

- **WHEN** remote server 為新版，於 remote 模式執行 archive 與 review show
- **THEN** archive 的 stdout 含封存目的地（dated 名稱）、規格計數行、封存討論行，與本機同文本；review show 的 stdout 印出工單文件原文全文，與本機同文本

#### Scenario: 對舊 server 整體退化

- **WHEN** remote server 為舊版（回應缺新欄位），於 remote 模式執行 archive 與 review show
- **THEN** 兩指令輸出整體退回既有 remote 輸出（簡短封存行、結構化工單摘要），exit code 0，SHALL NOT 出現新舊欄位混合的部分渲染

### Requirement: 動詞 --json 輸出形狀凍結

動詞的 --json 輸出欄位集合與 camelCase 命名 SHALL 維持既有契約不變；工單原文 SHALL NOT 出現在任何 --json 輸出。

#### Scenario: 工單 --json 兩模式同形且無原文欄位

- **WHEN** 分別於本機與 remote 模式執行 review show --json
- **THEN** 兩模式 payload 欄位集合一致——change、rounds、lastRound，rounds 各項含 index、phase、patchHash、scope、findings——且不存在攜帶工單原文的欄位
