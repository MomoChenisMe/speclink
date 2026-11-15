## MODIFIED Requirements

### Requirement: Apply 開始前記錄 host-local baseline
<!-- BEFORE: 不因 baseline 增欄的清單寫的是「TeamStore documents」 -->

Apply 技能 SHALL 在執行 in-progress add 前呼叫 speclink review prepare <change>（不帶旗標）。該子指令 SHALL 不讀 stdin；不帶旗標時的行為如本條所述，`--base`／`--from` 兩個旗標的行為見「以可信 commit 重設 Apply baseline 並承接快照」。不帶旗標時，prepare 於 `.speclink/review-scopes/<change>/baseline.json` 記錄 version、change、baseCommit、dirtyFilesAtStart、capturedAt、confidence；JSON 欄位 SHALL 為 camelCase。touched v1／v2 記錄、change metadata 與遠端儲存的文件 SHALL NOT 因 baseline 而增欄。

change 尚未帶 started_* 時，prepare SHALL 以當下 Git HEAD、開始前 dirty paths 與 UTC RFC3339 時間原子取代舊 baseline，confidence 為 initial。change 已開始且 baseline 存在時 SHALL 保留首次 baseline；已開始但 baseline 缺失時 SHALL 記 confidence=late；無 Git checkout 時 SHALL 記 confidence=unavailable 且 baseCommit=null。initial 成功 SHALL exit 0 且 stdout 為空；late／unavailable SHALL exit 0、stdout 為空並在 stderr 顯示後續審查需明示 fixed point。寫入失敗或 change 不存在 SHALL 非零結束，且 Apply 技能 SHALL NOT 接著執行 in-progress add。

#### Scenario: 首次 Apply 記錄乾淨 baseline

- **WHEN** 任務尚未開始、HEAD 為 40 字元 commit SHA，worktree 有 `notes/local.txt` 一個既存髒檔，Apply 技能開始執行
- **THEN** prepare exit 0、stdout 為空，baseline 的 baseCommit 為該 SHA、dirtyFilesAtStart 為 ["notes/local.txt"]、confidence 為 initial，touched 記錄不存在

#### Scenario: 已開始但 baseline 缺失

- **WHEN** change 已帶 started_at 且 host-local baseline 不存在時執行 review prepare
- **THEN** command exit 0、stderr 說明 baseline 為 late，記錄不得被 resolver 當成可信自動歸屬

#### Scenario: baseline 寫入失敗停止 Apply 起點

- **WHEN** `.speclink/review-scopes/<change>` 無法寫入
- **THEN** review prepare 非零結束，in-progress add 不執行，change metadata 維持原狀

### Requirement: remote workspace 使用同一 host resolver
<!-- BEFORE: review 的寫入「繼續走既有 TeamStore document 與 revision 契約」 -->

Remote Store workspace 的 review prepare 與 review scope SHALL 在執行 agent 持有的 local checkout 操作 baseline 與 snapshots；touched 認領 SHALL 自 Store 的 evidence 記錄讀取——fs 模式讀本地 evidence 檔，remote 模式 SHALL 經 typed remote client 的 change evidence 讀取端點取得，SHALL NOT 以本地檔案替代。多 actor 的 evidence entries 其 touched SHALL 取聯集；evidence 內的 head commit SHALL NOT 參與 scope 解析。併行認領守門所需的其他 active change evidence SHALL 同樣自 Store 讀取，無 evidence 的 change 視為零認領。active changes 與 ticket SHALL 透過 typed remote client 讀取。Server SHALL NOT 新增 Git diff endpoint，也 SHALL NOT 保存 host-local baseline／snapshot。

evidence 缺席或 touched 為空時 SHALL 維持 EmptyTouched fail-closed 與 needsInput 手動路徑。離線、認證失效或 remote read 錯誤（含 evidence 讀取失敗）時 command SHALL 非零結束且不寫 baseline／snapshot，SHALL NOT 把讀取失敗靜默降級為空認領。成功 scope 後的 review add-round／stamp／discard SHALL 繼續走既有的遠端文件與 revision 契約；scope 本地 Git 錯誤 SHALL NOT 被包裝成 revision conflict。

#### Scenario: remote scope 仍使用 local checkout

- **WHEN** workspace 連線 Remote Store，server 的 change evidence 帶 touched files，local checkout 有可信 baseline
- **THEN** review scope 自 server evidence 取得 touched 認領，使用 local Git 產生與 fs mode 同欄位的 resolved payload，server 不收到 patch 或 snapshot

#### Scenario: remote 離線時零 sidecar effects

- **WHEN** review scope 取得 remote ticket 前發現連線離線
- **THEN** command 非零結束、stderr 回報連線錯誤，baseline 與 snapshots 內容不變

#### Scenario: remote task done 後 scope 自動解析

- **WHEN** remote workspace 的 change 以 task done 回報 touched files，隨後同一 checkout 執行 review scope 且未帶任何手動旗標
- **THEN** scope 回 resolved payload，touched 認領即 task done 回報的檔案集合，不出現 needsInput

#### Scenario: 多 actor evidence 取聯集

- **WHEN** 同一 change 的 evidence 含兩位 actor 的 entries，touched files 分別為不同檔案集合
- **THEN** scope 的 touched 認領為兩集合的聯集，與 fs 模式 TouchedRecord 語意一致

#### Scenario: remote evidence 缺席維持 fail-closed

- **WHEN** remote workspace 的 change 無任何 evidence 記錄，執行 review scope 且未帶手動旗標
- **THEN** command 以 needsInput 回報 EmptyTouched 理由，手動旗標路徑（--base 或 --candidate-hash 加 --include-hunk）維持可用

#### Scenario: 併行認領守門於 remote 生效

- **WHEN** 同 workspace 另一 active change 的 server evidence 認領了與本 change 重疊的檔案
- **THEN** scope 回報的守門結果與 fs 模式同形，重疊認領不被靜默忽略
