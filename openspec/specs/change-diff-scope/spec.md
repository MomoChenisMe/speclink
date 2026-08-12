# change-diff-scope Specification

## Purpose

品質站審查面的界定語意：Apply 開始前記錄的 host-local baseline、以 git 解析完整 worktree patch 的 discovery scope、把 discovery 與 validation 綁在一起的凍結快照，以及 review scope 的人眼與 --json 兩路契約。本 capability 保證審查面可重現且範圍歧義時 fail closed——只能以 hash-pinned selection 解鎖，remote workspace 與本機共用同一 resolver。

## Requirements

### Requirement: Apply 開始前記錄 host-local baseline

Apply 技能 SHALL 在執行 in-progress add 前呼叫 speclink review prepare <change>。該子指令 SHALL 不接受 stdin 或旗標，並於 `.speclink/review-scopes/<change>/baseline.json` 記錄 version、change、baseCommit、dirtyFilesAtStart、capturedAt、confidence；JSON 欄位 SHALL 為 camelCase。touched v1／v2 記錄、change metadata 與 TeamStore documents SHALL NOT 因 baseline 而增欄。

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


<!-- @trace
source: converge-review-remediation-rounds
updated: 2026-08-03
code:
  - AGENTS.md
  - CLAUDE.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/it/remote_verb_parity.rs
  - crates/speclink-cli/tests/it/review_verbs.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/it/render_golden.rs
  - crates/speclink-host/src/change_diff.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/it/typed_client.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/it/review_api.rs
-->

---
### Requirement: Git-backed discovery scope 解析完整 worktree patch

speclink review scope <change> SHALL 以 touchedFiles 聯集作自動候選路徑，使用可信 baseCommit 到目前 worktree 的 Git diff，涵蓋 staged、unstaged、rename、delete 與 touched untracked additions；SHALL NOT 使用只比較 commit graph 的 `<base>...HEAD`。openspec artifacts 與 `.speclink` work data SHALL NOT 成為 review target。

成功的文字 file delta SHALL 回報 oldPath、newPath、kind、beforeHash、afterHash 與 hunks；每個 hunk SHALL 含 id、oldStart、oldLines、newStart、newLines。addition SHALL 允許 oldStart=0、oldLines=0；deletion SHALL 允許 newStart=0、newLines=0；多段修改 SHALL 保留多筆 hunk。rename SHALL 保留 oldPath 與 newPath。binary SHALL 回報 file hashes 與 kind=binary，hunks 為空。

#### Scenario: 未提交 staged 與 unstaged 內容都在 scope

- **WHEN** touched path 同時有 staged 與 unstaged 修改，baseCommit 等於 Apply 開始時 HEAD
- **THEN** review scope 的 patch 同時含兩部分修改，即使 `git diff <base>...HEAD` 對該 fixture 為空

#### Scenario: untracked touched file 是整檔 addition

- **WHEN** touchedFiles 含一個未追蹤 UTF-8 檔案
- **THEN** resolved payload 將它列為 added，beforeHash 為 null，首個 hunk 的 oldStart=0、oldLines=0

#### Scenario: delete 與 rename 保留雙端語意

- **WHEN** touched scope 含一個刪除檔與一個 Git 可辨識 rename
- **THEN** delete 的 afterHash 為 null 且 newLines=0，rename 同時輸出 oldPath 與 newPath


<!-- @trace
source: converge-review-remediation-rounds
updated: 2026-08-03
code:
  - AGENTS.md
  - CLAUDE.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/it/remote_verb_parity.rs
  - crates/speclink-cli/tests/it/review_verbs.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/it/render_golden.rs
  - crates/speclink-host/src/change_diff.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/it/typed_client.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/it/review_api.rs
-->

---
### Requirement: review scope 的 human 與 JSON 契約

speclink review scope <change> SHALL 支援 `--json`、`--base <rev>`、`--candidate-hash <sha256>` 與可重複的 `--include-hunk <id>`，且 SHALL NOT 讀取 stdin。自動歸屬成功時 command SHALL exit 0 並建立 frozen snapshot。

`--json` resolved payload SHALL 含 change:string、phase:"discovery"|"validation"、state:"resolved"、baseCommit:string、candidateHash:string、patchHash:string、paths:string[]、files:array、patch:string、outOfScopeChanged:string[]（恆存在，無範圍外變動時為空陣列）；files 每項 SHALL 含 oldPath:string|null、newPath:string|null、kind:string、beforeHash:string|null、afterHash:string|null、hunks:array；validation phase 的 files 每項 SHALL 另含 attribution:"finding"|"adjacent"|"new"，discovery phase SHALL 缺席該欄位；hunks 每項 SHALL 含 id:string 與四個 number ranges。human 成功路徑 SHALL 將 phase、patchHash、路徑數與 hunk 數寫至 stdout；outOfScopeChanged 非空時 SHALL 另以一行列出範圍外變動路徑。兩條路徑於 `--no-color` 下 SHALL 不含 ANSI。

#### Scenario: JSON resolved payload 可供 reviewer 使用

- **WHEN** 對可自動歸屬的兩檔三 hunk change 執行 review scope --json
- **THEN** exit 0、stdout 為合法 JSON、state 為 resolved、paths 有兩項、hunks 合計三項且 patchHash 以 `sha256:` 開頭

#### Scenario: validation payload 帶出身標記與範圍外註記

- **WHEN** 驗證輪的修復動到一個 findings 檔與一個未點名候選檔，另有一個 discovery 時被排除的檔也變動
- **THEN** resolved payload 的 files 分別帶 attribution "finding" 與 "adjacent"，outOfScopeChanged 含被排除檔的路徑且該檔不在 files 中

#### Scenario: 找不到 change

- **WHEN** 對不存在的 change 執行 review scope
- **THEN** command 非零結束、stderr 說明找不到 change，stdout 為空且不得建立 baseline 或 snapshot


<!-- @trace
source: review-validation-scope-movement
updated: 2026-08-05
-->

---
### Requirement: 歧義 scope 必須 fail closed 並以 hash-pinned selection 解鎖

baseline 缺失／late／unavailable、base commit 不可解析、touched path 已在 dirtyFilesAtStart、另一 active change 的 touched record 重疊時，review scope SHALL 視為 needsInput，不得建立 frozen snapshot。needsInput SHALL 僅發生於 discovery phase——validation phase 的範圍歸因依「frozen snapshot 綁定 discovery 與 validation patch」以內容移動解析，SHALL NOT 產生 needsInput。

human 路徑 SHALL 非零結束並於 stderr 列出 ambiguous paths，以及可信 `--base`、hash-pinned `--include-hunk`、隔離 worktree 三種處置。`--json` 路徑 SHALL 將 state:"needsInput"、candidateHash:string|null、ambiguousPaths:string[]、files:array 寫至 stdout後非零結束；candidate 可計算時 files SHALL 帶可選 hunk IDs。

人工 selection SHALL 同時提供前次 candidateHash 與至少一個 include-hunk。resolver SHALL 重算完整 candidate；hash 不符、hunk ID 不存在、重複 ID、選取 binary delta 或空選擇 SHALL 非零拒絕且零 snapshot effects。成功 selection SHALL 只把選定文字 hunks 納入 frozen patch，仍以實際檔案 before／after hashes 作漂移錨。

touchedFiles 缺失或為空時 SHALL NOT 自動審查全 worktree；使用者明示 `--base` 後，resolver SHALL 只把整個 diff 當 needsInput candidate，仍須 hash-pinned hunk selection 才能凍結。

#### Scenario: 開始前已髒的 touched file 不被靜默認領

- **WHEN** baseline 的 dirtyFilesAtStart 與 touchedFiles 都含 `src/lib.rs`
- **THEN** review scope 非零結束、state 為 needsInput、ambiguousPaths 含 `src/lib.rs`，snapshots 目錄不新增檔案

#### Scenario: candidate 漂移拒絕舊選擇

- **WHEN** 第一次 needsInput 回報 candidateHash A，使用者選 hunk 前 worktree 又改變並以 A 重試
- **THEN** command 非零結束、stderr 說明 candidate 已漂移，不建立 snapshot

#### Scenario: hash-pinned hunk selection 成功

- **WHEN** candidateHash 未變且使用者選取同一文字檔的兩個有效 hunk IDs
- **THEN** command exit 0，resolved patch 僅含該兩個 hunks，files 的 beforeHash／afterHash 仍錨定實際整檔內容


<!-- @trace
source: review-validation-scope-movement
updated: 2026-08-05
-->

---
### Requirement: frozen snapshot 綁定 discovery 與 validation patch

resolved scope SHALL 在 `.speclink/review-scopes/<change>/snapshots/` 原子建立 version 1 snapshot；檔名 SHALL 使用 patchHash 的十六進位 digest，不含 `sha256:` 的冒號。snapshot SHALL 記錄 change、phase、candidateHash、patchHash、baseCommit、createdAt、dirtyFilesAtCapture path/hash、canonical patch、file deltas、hunk ranges，以及 UTF-8 scope 的 beforeText／afterText；binary SHALL 只記 hashes。

工單不存在時 phase SHALL 為 discovery。工單存在時 phase SHALL 為 validation，resolver SHALL 比對上一輪快照 dirtyFilesAtCapture 全部條目的現況雜湊，內容移動者 SHALL 進驗證 patch：在保存 scope（patch deltas 與 carried texts）內的路徑比較 frozen afterText 與目前內容；不在保存 scope 但更早某輪快照收錄過的路徑 SHALL 沿工單各輪 patchHash 鏈（新→舊）回走，以最近收錄該路徑的快照重建凍結後狀態並輸出其差異（attribution "adjacent"）；任何輪皆未收錄的路徑 SHALL NOT 進 patch，SHALL 列入 outOfScopeChanged 註記且不阻擋凍結。再加入上輪 capture 後才變髒的新路徑（attribution "new"）。內容未移動的路徑 SHALL NOT 進驗證 patch。snapshot 缺失（含回走鏈中任一輪）或工單 patchHash 與 snapshot 不符 SHALL 非零結束，不得退回整檔 discovery。

review stamp 或 review discard 成功後 SHALL 清除該 change 的 review snapshots但保留 Apply baseline；清除失敗 SHALL 以 stderr warning 回報，且 SHALL NOT 回滾已完成的工單／metadata mutation。scope 已寫 snapshot但 add-round 失敗形成的 orphan SHALL 在下一次無對應工單的 scope 前清除。

#### Scenario: follow-up 只輸出 remediation patch

- **WHEN** Round 1 snapshot 保存 A 與 B，修正只改 A 並新增一個先前乾淨的 C
- **THEN** validation patch 只含 A 自 frozen afterText 起的差異與 C 的新差異，不重新輸出未修改的 B

#### Scenario: 未點名候選檔的修復以 adjacent 段進驗證 patch

- **WHEN** Round 1 candidate 收錄 A 與 B，findings 只點名 A，修復同時改動 A 與 B
- **THEN** Round 2 validation patch 含 A（attribution "finding"）與 B 自其最近收錄輪凍結後狀態起的差異（attribution "adjacent"），凍結 resolved 而非 needsInput

#### Scenario: 連續多輪修復未點名檔沿雜湊鏈回走

- **WHEN** B 於 Round 2 以 adjacent 段收錄後，Round 2 的修復再次改動 B
- **THEN** Round 3 凍結沿 patchHash 鏈取 Round 2 快照重建 B 的凍結後狀態，正常 resolved 輸出新差異

#### Scenario: 範圍外變動註記不擋凍結

- **WHEN** discovery 時經 hash-pinned selection 排除的檔案於驗證期間變動
- **THEN** 驗證輪凍結照常 resolved，該檔列於 outOfScopeChanged 且不在 patch 中，human 輸出含一行範圍外變動路徑

#### Scenario: snapshot 缺失不退回 discovery

- **WHEN** ticket 的 lastRound.patchHash 存在但 host-local snapshot 已被移除
- **THEN** review scope 非零結束並說明無法精確驗收，不能用 touched 整檔或目前 worktree 重新 discovery

#### Scenario: 成功蓋章後清除 snapshots

- **WHEN** review stamp 成功且 snapshots 存在
- **THEN** canonical review 工單與章依 review-station 契約更新，snapshots 被清除、baseline 保留


<!-- @trace
source: review-validation-scope-movement
updated: 2026-08-05
-->

---
### Requirement: remote workspace 使用同一 host resolver

Remote Store workspace 的 review prepare 與 review scope SHALL 在執行 agent 持有的 local checkout 操作 baseline、touched 與 snapshots；active changes 與 ticket SHALL 透過 typed remote client 讀取。Server SHALL NOT 新增 Git diff endpoint，也 SHALL NOT 保存 host-local baseline／snapshot。

離線、認證失效或 remote read 錯誤時 command SHALL 非零結束且不寫 baseline／snapshot。成功 scope 後的 review add-round／stamp／discard SHALL 繼續走既有 TeamStore document 與 revision 契約；scope 本地 Git 錯誤 SHALL NOT 被包裝成 revision conflict。

#### Scenario: remote scope 仍使用 local checkout

- **WHEN** workspace 連線 Remote Store且 local checkout 有可信 baseline 與 touched records
- **THEN** review scope 使用 local Git 產生與 fs mode 同欄位的 resolved payload，server 不收到 patch 或 snapshot

#### Scenario: remote 離線時零 sidecar effects

- **WHEN** review scope 取得 remote ticket 前發現連線離線
- **THEN** command 非零結束、stderr 回報連線錯誤，baseline 與 snapshots 內容不變

<!-- @trace
source: converge-review-remediation-rounds
updated: 2026-08-03
code:
  - AGENTS.md
  - CLAUDE.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/it/remote_verb_parity.rs
  - crates/speclink-cli/tests/it/review_verbs.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/it/render_golden.rs
  - crates/speclink-host/src/change_diff.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/it/typed_client.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/it/review_api.rs
-->