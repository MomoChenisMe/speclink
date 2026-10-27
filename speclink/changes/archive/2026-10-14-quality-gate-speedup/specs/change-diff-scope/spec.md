## MODIFIED Requirements

### Requirement: Apply 開始前記錄 host-local baseline
<!-- BEFORE: review prepare 不接受 stdin 或任何旗標 -->

Apply 技能 SHALL 在執行 in-progress add 前呼叫 speclink review prepare <change>（不帶旗標）。該子指令 SHALL 不讀 stdin；不帶旗標時的行為如本條所述，`--base`／`--from` 兩個旗標的行為見「以可信 commit 重設 Apply baseline 並承接快照」。不帶旗標時，prepare 於 `.speclink/review-scopes/<change>/baseline.json` 記錄 version、change、baseCommit、dirtyFilesAtStart、capturedAt、confidence；JSON 欄位 SHALL 為 camelCase。touched v1／v2 記錄、change metadata 與 TeamStore documents SHALL NOT 因 baseline 而增欄。

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

## ADDED Requirements

### Requirement: 以可信 commit 重設 Apply baseline 並承接快照

`speclink review prepare <change> --base <rev> [--from <worktree-path>]` SHALL 不讀 stdin，並依下列順序執行；前三步任一失敗 SHALL 非零結束、stderr 說明原因，且 SHALL NOT 寫入任何檔案：

1. 以 `git rev-parse --verify <rev>^{commit}` 把 `<rev>` 解析成完整 SHA，解析不到即失敗；以 `git merge-base --is-ancestor` 確認該 SHA 是目前 HEAD 的祖先，不是即失敗。
2. 目標 checkout 已有該 change 的 baseline，且審查或驗證任一站工單未結（工單存在而未蓋章）時失敗，stderr 點名哪一站——該站的快照綁著舊基準。目標沒有 baseline 時不做此檢查。
3. 帶 `--from` 時，該路徑不存在即失敗；`--from` 未搭配 `--base` 時失敗。
4. 帶 `--from` 時，把來源 checkout 中該 change 的審查快照與驗證快照（`.speclink/review-scopes/<change>/` 下兩站各自的快照目錄）逐檔複製到目前 checkout 的同一位置；目標已有同名快照檔即略過（檔名為 patchHash digest，同名即同內容）；每個檔案以暫存檔加 rename 原子寫入。來源沒有快照目錄時不複製、不視為錯誤。來源的 baseline.json SHALL NOT 被複製。
5. 最後以原子寫入產生 baseline：baseCommit 為步驟 1 的完整 SHA、dirtyFilesAtStart 為當下的 dirty paths、capturedAt 為 UTC RFC3339、confidence 為 initial；格式與不帶旗標時相同。

成功 SHALL exit 0、stdout 為空。步驟 4 中途失敗時 SHALL 非零結束，已複製的快照留著、baseline 未寫；以同樣參數重跑 SHALL 略過已存在的快照並完成剩餘步驟。remote 模式下 baseline 與快照仍是本機 checkout 的資料，行為與本機相同。不帶旗標的既有行為與輸出 SHALL 維持不變。

#### Scenario: 合併後在主 checkout 寫入基準並承接快照

- **WHEN** 主 checkout 無該 change 的 baseline，HEAD 的父 commit 為 M，來源 worktree 有一份審查快照與一份驗證快照，執行 `speclink review prepare demo --base M --from ../wt-demo`
- **THEN** exit 0、stdout 為空；主 checkout 的 baseline 為 baseCommit=M 的完整 SHA、confidence=initial；兩份快照出現在主 checkout 對應的快照目錄；之後 `speclink review scope demo --json` 不再回報 baseline 缺失

#### Scenario: 指定的 commit 不是 HEAD 的祖先

- **WHEN** `--base` 指向一個不在目前 HEAD 歷史上的 commit
- **THEN** 非零結束、stderr 說明該 commit 不是 HEAD 的祖先，baseline 與快照目錄皆不變

#### Scenario: 解析不到的 rev

- **WHEN** `--base no-such-ref`
- **THEN** 非零結束、stderr 說明解析不到 commit，零寫入

#### Scenario: 有未結工單時拒絕改寫既有基準

- **WHEN** worktree 內已有該 change 的 baseline，且審查工單存在而未蓋章，執行 `speclink review prepare demo --base T`
- **THEN** 非零結束、stderr 點名審查站工單未結，baseline 位元級不變

#### Scenario: 來源沒有快照照常成功

- **WHEN** 來源 worktree 從未跑過品質關卡，執行 `speclink review prepare demo --base M --from ../wt-demo`
- **THEN** exit 0，只寫入 baseline，快照目錄不建立

#### Scenario: --from 單獨使用被拒

- **WHEN** 執行 `speclink review prepare demo --from ../wt-demo`（未帶 --base）
- **THEN** 非零結束、stderr 說明 --from 必須搭配 --base，零寫入

#### Scenario: 複製中途失敗後重跑完成

- **WHEN** 承接快照時第二個檔案寫入失敗，之後以同樣參數重跑
- **THEN** 第一次非零結束且 baseline 未寫；重跑略過已存在的第一個檔案、複製其餘檔案並寫入 baseline，exit 0
