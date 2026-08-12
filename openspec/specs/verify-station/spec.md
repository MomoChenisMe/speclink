# verify-station Specification

## Purpose

驗證工單這個資料站的引擎語意：工單的建立與逐輪追加、讀取、frozen scope 與續輪 snapshot 的界定、蓋章守門與效果、以指紋錨判定章失效，以及放棄驗證與封存時的工單守門。本 capability 保證驗證章與審查章各自獨立、兩份工單可並存於同一 change，CLI 清單輸出的驗證欄位形狀被釘住，動詞在 remote 模式與本機同語意。

## Requirements

### Requirement: 驗證工單的建立與追加

系統 SHALL 提供 `speclink verify add-round <change> --stdin`:自 stdin 讀入一輪驗證內容,於 `openspec/changes/<change>/verify.md` 追加 `## Round N` 區段(工單不存在時建立)。每輪內容 SHALL 含 `**Scope**:` 檔案清單與零或多行分級 findings(CRITICAL/WARNING/SUGGESTION),骨架與審查工單同構、append-only。寫碼任務未全完成時 SHALL 拒絕(引擎守門,採 manual-task-marker 的寫碼任務全完成預測子)——工單語意限定為成品驗證(代碼成品),盤點輪不落工單;僅餘 `[M]` 手動測試任務未勾時 SHALL 放行。

新技能產生的 structured round SHALL 於 Scope 前同時包含 `**Phase**: discovery|validation` 與 `**Patch**: sha256:<hex>`。Phase 與 Patch 必須成對且格式合法;structured Round 1 只能是 discovery,後續 structured round 只能是 validation。只含既有 Scope/findings 的 legacy stdin SHALL 維持可讀,該輪 phase/patchHash 解析為 null。任何格式或輪次序列錯誤 SHALL 非零拒絕且工單零寫入。

#### Scenario: 寫碼任務未完成即拒絕落工單

- **WHEN** 對寫碼任務 4/5 的 change 執行 `verify add-round`
- **THEN** exit code 非零,stderr 說明驗證工單要求寫碼任務全數完成,無檔案建立

#### Scenario: 僅餘手動任務可落工單

- **WHEN** 對寫碼任務 4/4 全勾、一個 `[M]` 任務未勾且無工單的 change 執行 `verify add-round` 且 stdin 合法
- **THEN** exit code 0,`verify.md` 建立且含 `## Round 1`

#### Scenario: 首輪建立工單

- **WHEN** 對任務全數完成且無工單的 change 執行 `verify add-round` 且 stdin 合法
- **THEN** exit code 0,`verify.md` 建立且含 `## Round 1`

#### Scenario: 追加輪次不改寫既有輪

- **WHEN** 對已有 Round 1 的驗證工單再次執行 `verify add-round`
- **THEN** exit code 0,新增 `## Round 2` 且 Round 1 位元級不變

#### Scenario: 追加 structured validation

- **WHEN** 對已有 structured discovery Round 1 的工單追加 Phase=validation 且 Patch 合法的 Round 2
- **THEN** exit code 0,新增 `## Round 2`,stdout 確認 phase/patch 且 Round 1 位元級不變

#### Scenario: 第二個 discovery 被拒絕

- **WHEN** structured Round 1 已是 discovery,又追加 Phase=discovery
- **THEN** exit code 非零、stderr 說明後續輪只能是 validation,工單位元級不變

#### Scenario: phase 與 patch 必須成對

- **WHEN** stdin 只有 Phase 沒有 Patch
- **THEN** exit code 非零、stderr 說明兩欄必須同時存在,工單零寫入

#### Scenario: legacy round 保持相容

- **WHEN** stdin 只含既有 Scope 與 findings,不含 Phase/Patch
- **THEN** add-round 維持成功行為,該輪 phase 與 patchHash 解析為 null

#### Scenario: 內容缺少 Scope

- **WHEN** stdin 內容不含 `**Scope**:` 行
- **THEN** exit code 非零,stderr 說明格式要求,工單不變


<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: 驗證工單的讀取

系統 SHALL 提供 `speclink verify show <change> [--json]`：人眼路徑將工單原文印至 stdout（`--no-color` 無 ANSI）；`--json` payload SHALL 含 `change:string`、`rounds:array`、`lastRound:object`。rounds 每項 SHALL 含 `index:number`、`phase:string|null`、`patchHash:string|null`、`scope:string[]`、`findings:array`；findings 每項 SHALL 含 `severity:string`、`path:string`、`text:string`，lastRound 與 rounds 末項同形。Local fs、remote CLI、typed client 與 server response SHALL 輸出相同 camelCase 欄位與 null 語意。

#### Scenario: 讀取 JSON

- **WHEN** 對有兩輪的驗證工單執行 `verify show <change> --json`
- **THEN** exit code 0，`rounds` 長度 2，`lastRound.index` 為 2，structured 工單的 `lastRound.phase` 為 validation 且 `lastRound.patchHash` 為 `sha256:` digest

#### Scenario: legacy JSON 使用 null

- **WHEN** legacy 工單 round 不含 Phase／Patch，執行 `verify show --json`
- **THEN** phase 與 patchHash 明確輸出 null，既有 index、scope、findings 內容不變

#### Scenario: local 與 remote payload 同構

- **WHEN** 同一 structured 驗證工單分別經 local fs 與 remote server 讀取
- **THEN** 兩份 rounds／lastRound 的欄位集合、camelCase 名稱、null 與值逐項相同

#### Scenario: 無工單

- **WHEN** 對無驗證工單的 change 執行 `verify show`
- **THEN** exit code 非零，stderr 說明該 change 無驗證工單


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證 frozen scope 與續輪 snapshot

系統 SHALL 提供 `speclink verify scope <change>`，支援 `--json`、`--base <rev>`、`--candidate-hash <sha256>` 與可重複的 `--include-hunk <id>`，且 SHALL NOT 讀取 stdin。該動詞 SHALL 復用 change-diff-scope capability 的 Apply baseline、Host resolver、old/new hunk ranges、before／after hashes、human／JSON payload、hash-pinned selection 與 needsInput 規則，不得另寫 Git diff 演算法或以 touched 整檔替代。工單不存在時 phase 為 discovery；structured 工單存在時 phase 為 validation。

resolved scope SHALL 將 version 1 snapshot 原子寫入 `.speclink/review-scopes/<change>/verify-snapshots/<digest-hex>.json`。驗證站與審查站 SHALL 使用不同 snapshot namespace；`verify stamp`／`verify discard` 成功時只清除 verify snapshots，保留 Apply baseline 與 review snapshots。清除失敗只寫 stderr warning，不回滾工單／metadata mutation。

baseline 缺失／late／unavailable、dirty-at-start、active change path overlap、candidate hash 漂移、referenced snapshot 缺失，或 legacy 工單無法對應 snapshot時 SHALL fail closed：human 路徑非零並於 stderr 列可信 base、hash-pinned hunk selection、隔離 worktree或 discard 後重新 discovery 等適用處置；JSON 路徑 SHALL 以與 `review scope` 同構的 `state:"needsInput"` payload 寫至 stdout 後非零結束。失敗 SHALL 不新增 snapshot，也不得退回整檔 discovery。

#### Scenario: discovery scope 復用 Host resolver

- **WHEN** change 無 verify 工單、Apply baseline 可信且 touched candidate 可自動歸屬，執行 `verify scope --json`
- **THEN** exit code 0，payload 的 phase 為 discovery、state 為 resolved，file hunks 含 oldStart／oldLines／newStart／newLines，verify-snapshots 新增 patchHash 對應檔案

#### Scenario: validation 只凍結修正 patch

- **WHEN** structured Round 1 有兩筆 findings，主線只修正其中一檔後執行 `verify scope --json`
- **THEN** phase 為 validation，patch 只含從上輪 frozen afterText 起的 remediation 差異與其直接新增路徑，不含未修改的另一檔

#### Scenario: snapshot 缺失不退回 discovery

- **WHEN** verify ticket 的 lastRound.patchHash 存在但 referenced verify snapshot 已被移除
- **THEN** command 非零結束並說明無法精確驗收，工單不變且不得用 touched 整檔建立新 discovery snapshot

#### Scenario: 兩站 snapshot 清理互不影響

- **WHEN** review 與 verify snapshots 同時存在且 `verify stamp` 成功
- **THEN** verify snapshots 被清除，Apply baseline 與 review snapshots 位元級不變


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證蓋章守門與蓋章效果

系統 SHALL 提供 `speclink verify stamp <change> [--accept]`,守門與審查站同一條:寫碼任務全數完成(manual-task-marker 預測子)＋工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免必修條件。通過時 SHALL 於同一原子寫入內:將 `verified_at`/`verified_by`/`verified_with`/`verified_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`verified_scope` 寫入 `.openspec.yaml` 並刪除 `verify.md`,不得出現「章已寫而工單仍在」的中間狀態;canonical mutation 成功後 SHALL 依「驗證 frozen scope 與續輪 snapshot」清理 verify snapshots。

#### Scenario: 寫碼任務未完成即拒絕蓋章

- **WHEN** 對寫碼任務 4/5 的 change 執行 `verify stamp`
- **THEN** exit code 非零,stderr 說明寫碼任務未全數完成,metadata 與工單皆不變

#### Scenario: 僅餘手動任務可蓋章

- **WHEN** 寫碼任務全勾、一個 `[M]` 任務未勾且驗證工單末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 驗證工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 時執行 `verify stamp`
- **THEN** exit code 非零,stderr 點名未解必修數並提示 `--accept` 或先修正重驗

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 寫碼任務全數完成且驗證工單末輪僅含 SUGGESTION 級 findings 時執行 `verify stamp`(無 `--accept`)
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除,SUGGESTION 紀錄留在工單的 git 歷史

#### Scenario: 乾淨蓋章

- **WHEN** 寫碼任務全數完成且末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,`.openspec.yaml` 含五個 verified 欄位且 `verify.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 8 個任務,其中 7 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,驗證工單 Round 1 的 findings 為空
- **WHEN** `verify stamp` 成功
- **THEN** `.openspec.yaml` 內 `verified_tasks_total` 為 8(全任務總數)


<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: 驗證指紋錨與失效判定

蓋章時系統 SHALL 以驗證工單各輪 Scope 聯集記錄 `{ path, hash }` 至 `verified_scope`,路徑正規化與行尾 CRLF→LF 後 SHA-256 規則 SHALL 與審查站位元級同構(共用同一實作)。失效判定同構:當前全任務總數不再等於蓋章時的 `verified_tasks_total`、或任一寫碼任務未完成、或任一 scope 檔內容不符(含缺檔)→ stale;補勾或取消勾 `[M]` 任務 SHALL NOT 影響判定。判定結果 SHALL NOT 以 CLI 專屬查詢欄位曝光;封存守門 SHALL 消費此判定(見 change-lifecycle 的封存的章失效守門);desktop 協定曝光維持既有紅線、不在本 change 接線。

#### Scenario: 蓋章後修改範圍檔

- **WHEN** 驗證蓋章成功後修改任一 scope 檔內容
- **THEN** 失效判定為 stale

#### Scenario: 蓋章後補勾手動任務不失效

- **WHEN** 寫碼任務全完成、一個 `[M]` 任務未勾時驗證蓋章成功,之後將該 `[M]` 任務勾選
- **THEN** 失效判定仍為 fresh

#### Scenario: 行尾差異不觸發失效

- **WHEN** scope 檔內容僅行尾由 LF 變為 CRLF
- **THEN** 失效判定仍為 fresh


<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: 放棄驗證

系統 SHALL 提供 `speclink verify discard <change>`：刪除驗證工單、不寫任何 metadata，並依「驗證 frozen scope 與續輪 snapshot」清理 verify snapshots；無工單時非零 exit code。

#### Scenario: 放棄既有工單

- **WHEN** 對有驗證工單的 change 執行 `verify discard`
- **THEN** exit code 0，`verify.md` 不存在，`.openspec.yaml` 不變


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 封存的驗證工單守門與雙工單並存

`speclink archive` 偵測到 `verify.md` 時 SHALL 預設拒絕，stderr 列三處置（verify stamp／verify discard／`--carry-verify` 明示帶走）；`--carry-verify` 時工單隨目錄移入封存區。review 與 verify 工單並存時 SHALL 並列兩組處置，`--carry-review` 與 `--carry-verify` 可同時帶。皆無工單時 archive 行為 SHALL 維持不變。

#### Scenario: 僅驗證工單時拒絕

- **WHEN** 對僅有 `verify.md` 的 change 執行 `speclink archive`
- **THEN** exit code 非零，stderr 含 stamp、discard 與 `--carry-verify` 三處置，change 未搬移

#### Scenario: 雙工單並存

- **WHEN** 對同時有 `review.md` 與 `verify.md` 的 change 執行 `speclink archive`
- **THEN** exit code 非零，stderr 並列審查與驗證兩組處置

#### Scenario: 明示帶走驗證工單

- **WHEN** 帶 `--carry-verify` 封存僅有驗證工單的 change
- **THEN** 封存成功，封存目錄內含 `verify.md`


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: CLI 清單輸出的驗證欄位釘住

`speclink list --json` 的輸出 SHALL 不因 metadata 含任何 verified 欄位而改變形狀，與審查欄位的釘住規則同構。

#### Scenario: 帶驗證章的 change 不外洩新欄位

- **WHEN** 某 change 的 `.openspec.yaml` 含全套 verified 欄位時執行 `speclink list --json`
- **THEN** 該 change 的 JSON 項目與無 verified 欄位的 change 具有相同的欄位集合


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證動詞的 remote 模式行為

verify 工單動詞於 remote workspace SHALL 經 store 文件管道讀寫，revision 衝突、離線或認證失效時以非零 exit code 與 stderr 訊息回報，行為與審查動詞家族一致。`verify scope` SHALL 在執行 agent 持有的 local checkout 操作共用 Apply baseline、touched 與 verify snapshots，只透過 typed remote client 取得 active changes／verify ticket；Server SHALL NOT 新增 Git diff endpoint，也 SHALL NOT 保存 host-local baseline／snapshot。remote read 失敗時 SHALL 零 sidecar effects；本地 Git 錯誤不得偽裝成 revision conflict。

#### Scenario: 離線時追加驗證輪

- **WHEN** remote workspace 離線狀態下執行 `verify add-round`
- **THEN** exit code 非零，stderr 回報連線錯誤，遠端與本地投影皆不變

#### Scenario: remote scope 仍使用 local checkout

- **WHEN** workspace 連線 Remote Store且 local checkout 有可信 baseline 與 touched records
- **THEN** `verify scope` 使用 local Host resolver 產生與 fs mode 同欄位的 payload，server 不收到 patch 或 snapshot

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->