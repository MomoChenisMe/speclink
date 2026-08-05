## MODIFIED Requirements

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
