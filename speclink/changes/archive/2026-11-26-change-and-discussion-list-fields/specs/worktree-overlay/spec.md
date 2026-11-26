## ADDED Requirements

### Requirement: show 的 worktree 觀察面

於 local workspace 的主 checkout 執行 `speclink show <change>`，有效 worktree 政策為 true，而且這個變更依「worktree discovery 與映射慣例」映射成立時，`show` SHALL 從該 worktree 副本內的 change 目錄讀這個變更：proposal、design、tasks 的全文、`deltaSpecs`、metadata 與由 metadata 衍生的鍵、兩個品質關卡的狀態，人眼輸出與 `--json`（含 `--detail`）都一樣。輸出的鍵名、型別與人眼格式 SHALL 維持既有契約，SHALL NOT 增加 `worktree` 欄位。對同一份資料，`show --json` 的 `stage`、`startedAt`、`readyAt`、`createdBy`、`createdById` SHALL 與同一次 `speclink list --json --detail` 對該變更的同名鍵相同。

沒有映射、政策為 false、在 linked worktree 內執行，或 worktree discovery 回傳空映射（git 不可用或執行失敗）時，`show` SHALL 讀主副本，行為與這條需求加入之前相同，SHALL NOT 因 discovery 失敗而以非零 exit code 結束。`speclink artifact cat` SHALL 維持讀主副本。此為刻意變更：只有映射成立的情境下 `show` 的值改變；其他情境的輸出維持位元級不變。

#### Scenario: 映射成立時 show 與 list 讀同一份副本

- **WHEN** 主 checkout 啟用 worktree 政策，變更 add-dark-mode 映射成立；主副本 0／5 個任務已勾、沒有開工標記，worktree 副本 3／5 個任務已勾並有開工標記，於主 checkout 執行 `speclink show add-dark-mode --json` 與 `speclink list --json --detail`
- **THEN** 兩個指令的 exit code 都是 0；`show` 的 `tasks` 是 worktree 副本的 tasks.md 全文，`stage` 是 `in-progress`、`startedAt` 是 worktree 副本的開工日期，兩者與 list 該條目的同名鍵相同；`show` 的輸出沒有 `worktree` 鍵

#### Scenario: 人眼 show 同樣讀 worktree 副本

- **WHEN** 同上的映射成立，worktree 副本的 proposal.md 與主副本不同，於主 checkout 執行 `speclink show add-dark-mode` 與 `speclink show add-dark-mode --no-color`
- **THEN** 兩者的 Proposal 區段都是 worktree 副本的 proposal.md 內容，兩者的文字相同

#### Scenario: 沒有映射時讀主副本

- **WHEN** worktree 政策為 false（或變更沒有對應的 worktree），於主 checkout 執行 `speclink show add-dark-mode --json`
- **THEN** exit code 為 0；輸出取自主副本，與這條需求加入之前逐位元相同

#### Scenario: artifact cat 照舊讀主副本

- **WHEN** 映射成立，於主 checkout 執行 `speclink artifact cat tasks --change add-dark-mode`
- **THEN** 輸出是主副本的 tasks.md
