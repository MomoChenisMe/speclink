## MODIFIED Requirements

### Requirement: schema init 產出可載入的骨架

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

schema init SHALL 產出一份自身即通過載入與 validate 的 schema 骨架：

- schema.yaml 的純量值 SHALL 經 YAML 序列化寫出，不得以字串拼接產生未跳脫的值——含 `: ` 的 description 因此不會使文件無法解析
- 每個 artifact 宣告的 template 檔 SHALL 一併於 templates 目錄產出；骨架 template 的內容為該 artifact id 的標題行

#### Scenario: 預設 description 不破壞文件
- **WHEN** 使用者執行 speclink schema init my-flow（不帶 --description）
- **THEN** 產出的 schema.yaml 可被解析，且 speclink schemas 列得出 my-flow

#### Scenario: init 產出通過自身 validate
- **WHEN** 使用者執行 speclink schema init my-flow 後對它執行 speclink schema validate my-flow
- **THEN** 指令以 exit code 0 結束並回報 schema 合格

##### Example: 骨架檔案樹
| 路徑 | 內容 |
|------|------|
| speclink/schemas/my-flow/schema.yaml | 兩個 artifact（plan、tasks）與 apply 區塊 |
| speclink/schemas/my-flow/templates/plan.md | `# plan` |
| speclink/schemas/my-flow/templates/tasks.md | `# tasks` |

### Requirement: schema 指令旗標

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

schema 管理指令的旗標 SHALL 具備下列行為，不得收下後忽略：

- schema which --all SHALL 列出全部可解析的 schema，各自標示解析到的位置與來源層級（project／user／built-in），同名被遮蔽的位置一併列出
- schema validate SHALL 檢查 schema 引用的每個 template 檔存在（內建以內嵌資產計）；--verbose SHALL 逐項印出各驗證步驟與結果
- schema init <名稱> --default SHALL 在骨架建立後把 schema: <名稱> 寫入 speclink/config.yaml；檔案其餘內容 SHALL 逐位元組保留；config.yaml 不存在時 SHALL 建立僅含 schema 鍵的檔案

#### Scenario: which --all 列出全部
- **WHEN** 專案有一個自訂 schema 且使用者執行 speclink schema which --all
- **THEN** 輸出含內建 spec-driven 與該自訂 schema，各自帶解析位置與來源層級

#### Scenario: validate 抓缺席 template 檔
- **WHEN** 自訂 schema 宣告 template: missing.md 但 templates 目錄無該檔，使用者對它執行 speclink schema validate
- **THEN** 指令以非 0 exit code 結束並指名缺席的 template 檔

#### Scenario: init --default 寫入預設
- **WHEN** 使用者執行 speclink schema init my-flow --default
- **THEN** speclink/config.yaml 的 schema 鍵值為 my-flow，檔內其餘既有內容逐位元組不變

### Requirement: no-spec-change 內建 schema 的組成

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

內建 schema `no-spec-change`（中文「不改規格」）SHALL 給不改任何正式規格承諾行為的變更使用。它的 artifact SHALL 為 proposal（無前置）、design（前置為 proposal，選用）與 tasks（前置為 proposal），applyRequires SHALL 為 tasks，SHALL NOT 含 specs artifact。proposal 的 instruction SHALL 要求 Why 寫明「為什麼這個變更不改正式規格承諾的行為」並列出查過的正式規格；proposal 的 template SHALL 含 Why、What Changes、Non-Goals（選用）與 Impact，SHALL NOT 含 Capabilities。tasks 的 instruction SHALL 要求最後一個任務以既有測試確認相關正式規格的行為未變，SHALL NOT 要求任務引用 requirement 名稱。design 的 instruction 與 template SHALL 與 spec-driven 相同。schema 沒有 specs artifact 的變更執行 `speclink validate` 時 SHALL NOT 產生「No delta specs found」警告；schema 含 specs artifact 的變更維持現行警告。封存 `no-spec-change` 變更 SHALL 不需要 `--skip-specs`，正式規格不變。內建 schema 在本機與遠端工作區 SHALL 皆可解析。

#### Scenario: 建立不改規格變更
- **WHEN** 執行 `speclink new change tidy --schema no-spec-change`，接著執行 `speclink status --change tidy --json`
- **THEN** 兩者 exit code 為 0；`schemaName` 為 `no-spec-change`，`artifacts` 依序為 proposal、design、tasks，`applyRequires` 為 `["tasks"]`，沒有 specs

#### Scenario: proposal 範本不含 Capabilities
- **WHEN** 對 schema 為 no-spec-change 的變更執行 `speclink instructions proposal --change tidy --json`
- **THEN** `template` 含 `## Why`、`## What Changes`、`## Impact`，不含 `## Capabilities`；`instruction` 要求 Why 說明為什麼不改正式規格承諾的行為並列出查過的正式規格

#### Scenario: validate 不警告缺少 delta
- **WHEN** 對沒有 delta spec 的 no-spec-change 變更執行 `speclink validate tidy`，再對沒有 delta spec 的 spec-driven 變更執行 `speclink validate other`
- **THEN** `tidy` 的結果為有效且沒有「No delta specs found」警告；`other` 照現行輸出該警告

#### Scenario: 封存不需要 skip-specs
- **WHEN** no-spec-change 變更的任務全數完成，執行 `speclink archive tidy`（不帶 `--skip-specs`）
- **THEN** exit code 為 0，變更移入封存目錄，`speclink/specs/` 下的正式規格沒有任何變動

#### Scenario: 專案層同名 schema 遮蔽內建
- **WHEN** 專案的 `speclink/schemas/no-spec-change/schema.yaml` 存在且可載入，執行 `speclink schema which no-spec-change`
- **THEN** exit code 為 0，解析結果指向專案層的 schema.yaml、來源層級為 project；`speclink schema which --all` 同時列出被遮蔽的內建位置

### Requirement: change schema 動詞切換變更的 schema

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink change schema <name> <schema> [--json]` SHALL 把作用中變更 `.meta.yaml` 的 `schema:` 值改為 `<schema>`，其餘位元組逐字保留、每行沿用自身行尾；metadata 沒有 `schema:` 行時 SHALL 新增一行；`schema:` 不是第 0 欄的單行 `schema: <值>`（例如 `schema : x`，或值寫在縮排續行）時 SHALL 拒絕且零寫入，訊息提示先改成單行的 `schema: <name>`。守門 SHALL 依序：`<name>` 不是作用中變更（以名稱逐字比對）即錯；`<name>` 的 metadata 無法解析即拒絕；`<schema>` 不是小寫 kebab-case 名稱（含路徑分隔符、`..`、空白、換行、大寫字母，或為空字串）即拒絕，在解析之前判定，訊息沿用 `schema init`／`schema fork` 的名稱不合法訊息；`<schema>` 在專案層、user 層與內建都解析不到即錯（沿用現行的 schema 找不到訊息）；`<schema>` 沒有 id 為 `specs` 的 artifact 而變更含 delta spec 時即錯，訊息列出這些 capability；變更有未結的驗證工單時即錯，訊息提示先執行 `speclink verify discard <name>`。任一守門失敗 SHALL 以非零 exit code 結束、stderr 一行原因且零寫入。`<schema>` 與現值相同時 SHALL 成功且不寫檔。切換寫入時，若 metadata 帶著驗證章，同一次寫入 SHALL 移除 `verified_at`、`verified_by`、`verified_with`、`verified_tasks_total` 與 `verified_scope`，`verified_rounds` 與審查章欄位 SHALL 保留。成功時 stdout SHALL 為一行 `✓ <name> schema: <舊值> → <新值>`（未寫檔時為 `✓ <name> schema: <值> (unchanged)`，移除驗證章時另一行 `  verify stamp cleared — re-run verify under <新值>`）；`--json` 時 stdout SHALL 為 `{ "change": string, "schema": string, "previous": string, "changed": boolean }`。在主 checkout 且 worktree 政策開啟時，`<name>` 已映射到 worktree 者 SHALL 寫入 worktree 副本，主副本不變。指令在本機與遠端模式 SHALL 同語意、同輸出形狀。

#### Scenario: 切到不改規格
- **WHEN** 作用中變更 tidy 的 `.meta.yaml` 為 `schema: spec-driven`、沒有 delta spec，執行 `speclink change schema tidy no-spec-change`
- **THEN** exit code 為 0，`schema:` 行變為 `schema: no-spec-change`，其餘行逐位元不變，stdout 為 `✓ tidy schema: spec-driven → no-spec-change`

#### Scenario: 切回 spec-driven 並輸出 JSON
- **WHEN** tidy 的 schema 為 no-spec-change，執行 `speclink change schema tidy spec-driven --json`
- **THEN** exit code 為 0，stdout 是 JSON（排版同其他 `--json` 輸出），內容為 `{"change":"tidy","schema":"spec-driven","previous":"no-spec-change","changed":true}`

#### Scenario: 有 delta 時拒絕切到不改規格
- **WHEN** 變更 c 含 `specs/auth/spec.md` 的 delta，執行 `speclink change schema c no-spec-change`
- **THEN** exit code 非零，stderr 說明 c 含 delta spec（列出 auth）而 no-spec-change 沒有 specs，`.meta.yaml` 逐位元不變

#### Scenario: 解析不到的 schema 拒絕
- **WHEN** 執行 `speclink change schema c no-such-flow`
- **THEN** exit code 非零，stderr 為現行的 schema 找不到訊息，`.meta.yaml` 逐位元不變

#### Scenario: 不合法的 schema 名稱在解析前拒絕
- **WHEN** 執行 `speclink change schema c ../evil` 或 `speclink change schema c Spec-Driven`
- **THEN** exit code 非零，stderr 為 schema 名稱不合法的訊息（`must be lowercase kebab-case`），`.meta.yaml` 逐位元不變

#### Scenario: 未結驗證工單時拒絕
- **WHEN** 變更 c 有未結的驗證工單，執行 `speclink change schema c spec-driven`
- **THEN** exit code 非零，stderr 提示先執行 `speclink verify discard c`，`.meta.yaml` 與驗證工單皆不變

#### Scenario: 切換時移除驗證章、保留審查章與輪數
- **WHEN** 變更 c 的 schema 為 no-spec-change，`.meta.yaml` 帶著審查章、驗證章與 `verified_rounds: 2`，沒有未結工單，執行 `speclink change schema c spec-driven`
- **THEN** exit code 為 0，`verified_at`、`verified_by`、`verified_with`、`verified_tasks_total` 與 `verified_scope` 已移除，`verified_rounds: 2` 與所有 `reviewed_*` 欄位不變，stdout 第二行說明驗證章已清除

#### Scenario: 同一個 schema 不寫檔
- **WHEN** c 的 schema 已是 spec-driven，執行 `speclink change schema c spec-driven`
- **THEN** exit code 為 0，stdout 為 `✓ c schema: spec-driven (unchanged)`，`.meta.yaml` 逐位元不變

#### Scenario: 變更不存在
- **WHEN** 執行 `speclink change schema ghost spec-driven`，沒有名為 ghost 的作用中變更
- **THEN** exit code 非零，stderr 說明找不到 ghost，沒有任何檔案被寫入

#### Scenario: 目標變更在 worktree 內時寫入副本
- **WHEN** 主 checkout 且 worktree 政策開啟，tidy 已映射到 worktree，執行 `speclink change schema tidy no-spec-change`
- **THEN** worktree 副本的 `.meta.yaml` 為 `schema: no-spec-change`，主副本不變
