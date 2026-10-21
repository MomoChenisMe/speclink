## MODIFIED Requirements

### Requirement: 單一正典載入
每個內建 schema SHALL 只由一份內嵌的正典 YAML 定義：spec-driven 為 crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml，no-spec-change 為 crates/engine/speclink-core/assets/schema/no-spec-change/fork.schema.yaml。內建名稱 SHALL 只有這兩個；名稱解析順序維持 project → user → 內建。載入時 SHALL 走與自訂 schema 相同的解析與驗證路徑；template 內容由內嵌資產依 template 檔名附掛。引擎 SHALL NOT 另持手寫的內建 schema 定義。schema fork <內建名稱> 產出的 schema.yaml SHALL 與該內建的正典 YAML 逐位元組相同。內建 schema 顯示的 description SHALL 為正典 YAML 的字面值。

#### Scenario: 內建與 fork 同源
- **WHEN** 使用者執行 speclink schema fork spec-driven，以及 speclink schema fork no-spec-change
- **THEN** 兩份產出的 schema.yaml 各自與對應的內嵌正典 YAML 逐位元組相同

#### Scenario: 列表 description 來自正典
- **WHEN** 使用者執行 speclink schemas
- **THEN** spec-driven 的 description 顯示正典 YAML 的字面「Default OpenSpec workflow - proposal → specs → tasks (design optional)」，no-spec-change 的 description 顯示其正典 YAML 的字面「No-spec-change workflow - proposal → tasks (design optional), for changes that alter no promised behavior」

#### Scenario: template 內容附掛
- **WHEN** 引擎載入內建 schema
- **THEN** spec-driven 的四個 artifact 與 no-spec-change 的三個 artifact 各自帶有非空的 template 內容，與對應的內嵌 template 資產逐字相同

## ADDED Requirements

### Requirement: no-spec-change 內建 schema 的組成
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
- **THEN** exit code 為 0，變更移入封存目錄，`openspec/specs/` 下的正式規格沒有任何變動

#### Scenario: 專案層同名 schema 遮蔽內建
- **WHEN** 專案的 `openspec/schemas/no-spec-change/schema.yaml` 存在且可載入，執行 `speclink schema which no-spec-change`
- **THEN** exit code 為 0，解析結果指向專案層的 schema.yaml、來源層級為 project；`speclink schema which --all` 同時列出被遮蔽的內建位置
