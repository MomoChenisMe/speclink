## ADDED Requirements

### Requirement: change schema 動詞切換變更的 schema
`speclink change schema <name> <schema> [--json]` SHALL 把作用中變更 `.openspec.yaml` 的 `schema:` 值改為 `<schema>`，其餘位元組逐字保留、每行沿用自身行尾；metadata 沒有 `schema:` 行時 SHALL 新增一行。守門 SHALL 依序：`<name>` 不是作用中變更（以名稱逐字比對）即錯；`<name>` 的 metadata 無法解析即拒絕；`<schema>` 在專案層、user 層與內建都解析不到即錯（沿用現行的 schema 找不到訊息）；`<schema>` 沒有 id 為 `specs` 的 artifact 而變更含 delta spec 時即錯，訊息列出這些 capability；變更有未結的驗證工單時即錯，訊息提示先執行 `speclink verify discard <name>`。任一守門失敗 SHALL 以非零 exit code 結束、stderr 一行原因且零寫入。`<schema>` 與現值相同時 SHALL 成功且不寫檔。切換寫入時，若 metadata 帶著驗證章，同一次寫入 SHALL 移除 `verified_at`、`verified_by`、`verified_with`、`verified_tasks_total` 與 `verified_scope`，`verified_rounds` 與審查章欄位 SHALL 保留。成功時 stdout SHALL 為一行 `✓ <name> schema: <舊值> → <新值>`（未寫檔時為 `✓ <name> schema: <值> (unchanged)`，移除驗證章時另一行 `  verify stamp cleared — re-run verify under <新值>`）；`--json` 時 stdout SHALL 為 `{ "change": string, "schema": string, "previous": string, "changed": boolean }`。在主 checkout 且 worktree 政策開啟時，`<name>` 已映射到 worktree 者 SHALL 寫入 worktree 副本，主副本不變。指令在本機與遠端模式 SHALL 同語意、同輸出形狀。

#### Scenario: 切到不改規格
- **WHEN** 作用中變更 tidy 的 `.openspec.yaml` 為 `schema: spec-driven`、沒有 delta spec，執行 `speclink change schema tidy no-spec-change`
- **THEN** exit code 為 0，`schema:` 行變為 `schema: no-spec-change`，其餘行逐位元不變，stdout 為 `✓ tidy schema: spec-driven → no-spec-change`

#### Scenario: 切回 spec-driven 並輸出 JSON
- **WHEN** tidy 的 schema 為 no-spec-change，執行 `speclink change schema tidy spec-driven --json`
- **THEN** exit code 為 0，stdout 是 JSON（排版同其他 `--json` 輸出），內容為 `{"change":"tidy","schema":"spec-driven","previous":"no-spec-change","changed":true}`

#### Scenario: 有 delta 時拒絕切到不改規格
- **WHEN** 變更 c 含 `specs/auth/spec.md` 的 delta，執行 `speclink change schema c no-spec-change`
- **THEN** exit code 非零，stderr 說明 c 含 delta spec（列出 auth）而 no-spec-change 沒有 specs，`.openspec.yaml` 逐位元不變

#### Scenario: 解析不到的 schema 拒絕
- **WHEN** 執行 `speclink change schema c no-such-flow`
- **THEN** exit code 非零，stderr 為現行的 schema 找不到訊息，`.openspec.yaml` 逐位元不變

#### Scenario: 未結驗證工單時拒絕
- **WHEN** 變更 c 有未結的驗證工單，執行 `speclink change schema c spec-driven`
- **THEN** exit code 非零，stderr 提示先執行 `speclink verify discard c`，`.openspec.yaml` 與驗證工單皆不變

#### Scenario: 切換時移除驗證章、保留審查章與輪數
- **WHEN** 變更 c 的 schema 為 no-spec-change，`.openspec.yaml` 帶著審查章、驗證章與 `verified_rounds: 2`，沒有未結工單，執行 `speclink change schema c spec-driven`
- **THEN** exit code 為 0，`verified_at`、`verified_by`、`verified_with`、`verified_tasks_total` 與 `verified_scope` 已移除，`verified_rounds: 2` 與所有 `reviewed_*` 欄位不變，stdout 第二行說明驗證章已清除

#### Scenario: 同一個 schema 不寫檔
- **WHEN** c 的 schema 已是 spec-driven，執行 `speclink change schema c spec-driven`
- **THEN** exit code 為 0，stdout 為 `✓ c schema: spec-driven (unchanged)`，`.openspec.yaml` 逐位元不變

#### Scenario: 變更不存在
- **WHEN** 執行 `speclink change schema ghost spec-driven`，沒有名為 ghost 的作用中變更
- **THEN** exit code 非零，stderr 說明找不到 ghost，沒有任何檔案被寫入

#### Scenario: 目標變更在 worktree 內時寫入副本
- **WHEN** 主 checkout 且 worktree 政策開啟，tidy 已映射到 worktree，執行 `speclink change schema tidy no-spec-change`
- **THEN** worktree 副本的 `.openspec.yaml` 為 `schema: no-spec-change`，主副本不變

### Requirement: schema 沒有 specs 產出時拒絕寫入 delta spec
對 schema 沒有 id 為 `specs` 的 artifact 的變更執行 `speclink new artifact spec <capability> --change <name>`（含 `--stdin` 與 `--force`）時，系統 SHALL 以非零 exit code 拒絕、零寫入，stderr SHALL 說明該變更的 schema 沒有 specs，並提示 `speclink change schema <name> spec-driven`。schema 含 specs artifact 的變更 SHALL 維持現行行為。

#### Scenario: 不改規格變更寫 delta 被拒
- **WHEN** 變更 tidy 的 schema 為 no-spec-change，執行 `speclink new artifact spec auth --change tidy --stdin` 並由 stdin 提供合法的 delta 內容
- **THEN** exit code 非零，stderr 說明 tidy 使用 no-spec-change、沒有 specs，並提示 `speclink change schema tidy spec-driven`；tidy 目錄下沒有出現 specs/

#### Scenario: spec-driven 變更照常寫入
- **WHEN** 變更 c 的 schema 為 spec-driven，執行 `speclink new artifact spec auth --change c --stdin` 並提供合法的 delta 內容（auth 為正式規格已收錄的 capability）
- **THEN** exit code 為 0，`specs/auth/spec.md` 建立
