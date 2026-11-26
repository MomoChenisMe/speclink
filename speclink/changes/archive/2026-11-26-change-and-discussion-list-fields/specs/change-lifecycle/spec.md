## ADDED Requirements

### Requirement: 建立變更時記下建立者的識別

執行環境帶有呼叫者的識別（宿主套件的身分識別）時，建立變更的動詞——`speclink new change` 與 `speclink discuss promote`——SHALL 在變更的 metadata 寫入 `created_by_id`，值為那個識別，寫出時 SHALL 經 YAML 純量跳脫。讀回的值 SHALL 與寫入的識別逐字相同：YAML 會讀成別種值的字（例如 `null`、`~`、`true`、`123`）SHALL 加上引號寫出，讀回時仍是同一個字串。執行環境沒有識別時（本機 CLI 以 git 身分執行）SHALL NOT 寫入這個鍵。`created_by` 既有的寫入 SHALL 不變。既有沒有 `created_by_id` 的變更 SHALL 照常運作；系統 SHALL NOT 事後替它們補上識別。這個鍵 SHALL 隨變更進入封存。

#### Scenario: 經宿主套件建立的變更記下識別

- **WHEN** 宿主以識別為 `u_1`、顯示名稱為 `Ann` 的呼叫者執行 `new change demo`
- **THEN** demo 的 metadata 含 `created_by: Ann` 與 `created_by_id: u_1`

#### Scenario: 本機 CLI 建立的變更沒有識別

- **WHEN** 在設有 git 身分的本機專案執行 `speclink new change demo`
- **THEN** exit code 為 0；demo 的 metadata 含 `created_by`、不含 `created_by_id`

#### Scenario: 轉為變更時同樣記下識別

- **WHEN** 宿主以識別為 `u_1` 的呼叫者執行 `discuss promote two --name add-two`
- **THEN** 新建立的變更 add-two 的 metadata 含 `created_by_id: u_1`

#### Scenario: 識別是 YAML 會讀成別種值的字

- **WHEN** 宿主以識別為 `null` 的呼叫者執行 `new change demo`，再執行 `show demo --json`
- **THEN** demo 的 metadata 含 `created_by_id: "null"`；`show --json` 的 `createdById` 是字串 `"null"`

### Requirement: 任務全部完成時記下就緒時間

一次任務完成讓變更的全部任務（至少一個）都成為已勾選時，系統 SHALL 在變更的 metadata 寫入 `ready_at`，值為帶本機時區偏移量、精確到秒的 RFC 3339 時戳；metadata 已經有 `ready_at` 時 SHALL 以新的時戳取代，SHALL NOT 留下兩行。這個寫入 SHALL 發生在 `speclink task done`（本機、遠端與宿主套件）與桌面 app 的單一勾選及「全部標完成」，各寫入點 SHALL 共用引擎的同一個函式。完成後仍有未勾選的任務時 SHALL NOT 寫入。

`speclink task undone` 與桌面 app 的取消勾選 SHALL NOT 改動 `ready_at`（取消勾選維持純狀態翻轉）。不經動詞的寫入——直接編輯 tasks.md、以 `speclink new artifact tasks --force` 整份寫回、匯入——SHALL NOT 寫入 `ready_at`；系統 SHALL NOT 事後補上。

對外輸出 SHALL 只給日期部分（`YYYY-MM-DD`），而且 SHALL 只在變更目前至少有一個任務並且全部勾完時輸出（鍵為 `readyAt`）；不符合時 SHALL 不輸出，即使 metadata 記有 `ready_at`。metadata 損壞的變更 SHALL 照「壞 metadata fail closed」的既有規則在寫入前拒絕。`ready_at` SHALL 在任務清單寫回之前寫入：寫入 `ready_at` 失敗時 SHALL NOT 勾選任何任務，指令 SHALL 以非零 exit code 結束並在 stderr 回報錯誤，所以重試同一個任務會再寫一次。`ready_at` 寫好之後任務清單寫回失敗時，記錄 SHALL 保留，而依上面的輸出規則 `readyAt` SHALL 不出現。

#### Scenario: 勾完最後一個任務時寫入

- **WHEN** 變更 demo 有 3 個任務、已勾 2 個，執行 `speclink task done 3 --change demo`
- **THEN** exit code 為 0；demo 的 metadata 含一行 `ready_at`，值符合 RFC 3339、帶時區偏移量、精確到秒；`speclink show demo --json` 的 `readyAt` 是該時戳的前 10 個字元

#### Scenario: 寫入就緒時間失敗時不勾選

- **WHEN** 變更 demo 有 3 個任務、已勾 2 個，demo 的 `.meta.yaml` 無法寫入，執行 `speclink task done 3 --change demo`
- **THEN** exit code 不是 0，stderr 說明寫入失敗；tasks.md 的第 3 個任務仍未勾選，metadata 不含 `ready_at`；排除原因後再執行同一個指令，exit code 為 0，第 3 個任務已勾選，metadata 含一行 `ready_at`

#### Scenario: 還有未勾選的任務時不寫入

- **WHEN** 變更 demo 有 3 個任務、已勾 1 個，執行 `speclink task done 2 --change demo`
- **THEN** exit code 為 0；demo 的 metadata 不含 `ready_at`

#### Scenario: 取消勾選不改動記錄

- **WHEN** 變更 demo 的 3 個任務全部勾完、metadata 記有 `ready_at`，執行 `speclink task undone 3 --change demo`
- **THEN** exit code 為 0；metadata 的 `ready_at` 那一行逐字不變；`speclink show demo --json` 沒有 `readyAt`

#### Scenario: 再次全部完成時覆寫

- **WHEN** 接續上一個情境，之後再執行 `speclink task done 3 --change demo`
- **THEN** metadata 仍然只有一行 `ready_at`，值是這一次完成的時戳；`speclink show demo --json` 有 `readyAt`

#### Scenario: 桌面 app 的全部標完成同樣寫入

- **WHEN** 使用者在桌面 app 對有 3 個未勾選任務的變更 demo 按下「全部已完成」
- **THEN** tasks.md 的 3 個任務都成為已勾選，demo 的 metadata 含一行 `ready_at`

#### Scenario: 直接編輯任務清單不寫入

- **WHEN** 使用者直接編輯 demo 的 tasks.md，把全部任務改成已勾選，之後執行 `speclink list --json --detail`
- **THEN** demo 的 metadata 不含 `ready_at`；清單裡 demo 的 `stage` 是 `ready`，沒有 `readyAt`

## MODIFIED Requirements

### Requirement: list --json 曝變更的 delta capability

speclink list --json SHALL 於帶 delta 規格的變更 payload 曝 deltaCapabilities 欄位：該變更 delta 規格的 capability 名稱字串陣列，升冪，取自 Store 的 delta capability 清單；與 speclink show <change> --json 的 deltaSpecs 同源。主 checkout 的 worktree 觀察面生效時，映射成立的變更與 list 其他欄位一樣取自該 worktree 副本，speclink show 也讀同一份副本（見 worktree-overlay「show 的 worktree 觀察面」）。變更沒有 delta 規格時 SHALL 省略該欄位，以維持 list --json 對無 delta 變更的既有輸出逐位元不變。人眼 speclink list 輸出 SHALL 不變。remote 模式 SHALL 輸出同形欄位，值取自 GET /changes 清單項的 deltaCapabilities；server 未送該欄位時 SHALL 省略，SHALL NOT 偽造值。

#### Scenario: list 曝 deltaCapabilities 且對無 delta 變更輸出不變

- **WHEN** 對含一個帶 client-protocol 與 server-verb-api 兩份 delta 規格的變更、與一個沒有 delta 規格的變更的專案執行 speclink list --json 與 speclink list
- **THEN** 前者的 --json payload 含 "deltaCapabilities": ["client-protocol", "server-verb-api"]；後者的 --json payload 無 deltaCapabilities 鍵；人眼 speclink list 輸出不含 deltaCapabilities 字樣

#### Scenario: remote 模式同形

- **WHEN** remote 模式下 GET /changes 的清單項帶 deltaCapabilities: ["cap-a"]，執行 speclink list --json
- **THEN** 該變更 payload 含 "deltaCapabilities": ["cap-a"]，欄位名集合與 fs 模式下帶 cap-a delta 規格的同名變更一致

#### Scenario: worktree 觀察面下 list 與 show 同源

- **WHEN** 主 checkout 啟用 worktree 政策，變更 add-dark-mode 映射成立，它的 worktree 副本多了一份 auth 的 delta 規格、主副本沒有，於主 checkout 執行 speclink list --json 與 speclink show add-dark-mode --json
- **THEN** list 該條目的 deltaCapabilities 含 auth；show 的 deltaSpecs 含 auth/spec.md
