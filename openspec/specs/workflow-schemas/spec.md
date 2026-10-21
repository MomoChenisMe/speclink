# workflow-schemas Specification

## Purpose

工作流 schema 引擎的正典歸屬與守門：內建 spec-driven 的單一正典載入、instruction 內容的單一來源、schema 的載入時驗證規則，以及 schema 管理指令（which／validate／fork／init）的旗標行為。本 capability 保證內建定義只有一份、fork 輸出與正典逐位元組相同、非法 schema 在載入口被拒絕而非靜默通過。

## Requirements

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


<!-- @trace
source: no-spec-change-schema
updated: 2026-10-21T23:33:58+08:00
-->

---
### Requirement: instruction 單一來源
內建 schema 的 instruction 文字 SHALL 只存在於正典 YAML 一處；獨立的 instruction 資產檔 SHALL 全數移除。正典 YAML 的 instruction SHALL 承載現行 instructions payload 的內容——含 specs instruction 的 Purpose 段規則、MODIFIED 工作流的 BEFORE 註記步驟與 REMOVED-SCENARIO 合併門檻段落。收斂後 speclink instructions 各 artifact 的 instruction 欄位輸出 SHALL 與收斂前逐字相同。

#### Scenario: payload 內容維持現行
- **WHEN** 使用者對任一 change 執行 speclink instructions specs
- **THEN** instruction 欄位含「Purpose section (new capabilities only)」段與 REMOVED-SCENARIO 合併門檻文字，與本變更前的輸出逐字相同

#### Scenario: fork 輸出收斂到現行
- **WHEN** 使用者執行 speclink schema fork spec-driven 並讀取產出 schema.yaml 內 specs 的 instruction
- **THEN** 該 instruction 含 Purpose 段規則（本變更前的 fork 輸出缺這一段）


<!-- @trace
source: schema-engine-openspec-parity
updated: 2026-08-20
-->

---
### Requirement: schema 驗證規則
schema 載入 SHALL 執行下列檢查，任一失敗即回錯誤、該 schema 不可被任何動詞使用：

- artifact id 不得重複
- requires SHALL 只指向存在的 artifact id
- requires 圖不得有循環；錯誤訊息 SHALL 印出完整環路徑
- version 鍵必填且 SHALL 為正整數
- 每個 artifact 的 description 鍵必填（值可為空字串）
- 每個 artifact 的 template 鍵必填且非空（移除以 artifact id 推導預設檔名的容錯）

schema fork 與 schema init 的目的名稱 SHALL 符合小寫 kebab-case（正則 ^[a-z][a-z0-9]*(-[a-z0-9]+)*$），不符即以非 0 exit code 拒絕。內嵌正典 YAML 本身 SHALL 通過全部檢查。

#### Scenario: 懸空 requires 被拒
- **WHEN** 自訂 schema 的某 artifact 的 requires 指向不存在的 id
- **THEN** 載入回錯誤並指名該 artifact 與不存在的 id，引用該 schema 的動詞以非 0 exit code 結束

#### Scenario: 循環相依印出環路徑
- **WHEN** 自訂 schema 的 requires 圖含循環
- **THEN** 錯誤訊息含完整環路徑

##### Example: 兩節點循環
- **GIVEN** artifacts: a（requires: b）、b（requires: a）
- **WHEN** 任一動詞解析該 schema
- **THEN** 錯誤訊息含「a → b → a」（或等價起點的同一環）

#### Scenario: 必填欄位缺席被拒
- **WHEN** 自訂 schema 缺 version、artifact 缺 description 鍵或缺 template 鍵
- **THEN** 載入回錯誤並指名缺席欄位

##### Example: 欄位邊界表
| schema 內容 | 結果 |
|-------------|------|
| version 鍵缺席 | 錯誤：version 必填 |
| version: 0 | 錯誤：version 須為正整數 |
| version: 1.5 | 錯誤：version 須為正整數 |
| description 鍵缺席 | 錯誤：description 必填 |
| description 為空字串 | 通過 |
| template 鍵缺席 | 錯誤：template 必填 |
| template 為空字串 | 錯誤：template 必填非空 |

#### Scenario: 非法名稱被拒
- **WHEN** 使用者執行 speclink schema init My_Schema
- **THEN** 指令以非 0 exit code 結束，錯誤訊息說明名稱須為小寫 kebab-case


<!-- @trace
source: schema-engine-openspec-parity
updated: 2026-08-20
-->

---
### Requirement: schema init 產出可載入的骨架
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
| openspec/schemas/my-flow/schema.yaml | 兩個 artifact（plan、tasks）與 apply 區塊 |
| openspec/schemas/my-flow/templates/plan.md | `# plan` |
| openspec/schemas/my-flow/templates/tasks.md | `# tasks` |


<!-- @trace
source: schema-engine-openspec-parity
updated: 2026-08-20
-->

---
### Requirement: schema 指令旗標
schema 管理指令的旗標 SHALL 具備下列行為，不得收下後忽略：

- schema which --all SHALL 列出全部可解析的 schema，各自標示解析到的位置與來源層級（project／user／built-in），同名被遮蔽的位置一併列出
- schema validate SHALL 檢查 schema 引用的每個 template 檔存在（內建以內嵌資產計）；--verbose SHALL 逐項印出各驗證步驟與結果
- schema init <名稱> --default SHALL 在骨架建立後把 schema: <名稱> 寫入 openspec/config.yaml；檔案其餘內容 SHALL 逐位元組保留；config.yaml 不存在時 SHALL 建立僅含 schema 鍵的檔案

#### Scenario: which --all 列出全部
- **WHEN** 專案有一個自訂 schema 且使用者執行 speclink schema which --all
- **THEN** 輸出含內建 spec-driven 與該自訂 schema，各自帶解析位置與來源層級

#### Scenario: validate 抓缺席 template 檔
- **WHEN** 自訂 schema 宣告 template: missing.md 但 templates 目錄無該檔，使用者對它執行 speclink schema validate
- **THEN** 指令以非 0 exit code 結束並指名缺席的 template 檔

#### Scenario: init --default 寫入預設
- **WHEN** 使用者執行 speclink schema init my-flow --default
- **THEN** openspec/config.yaml 的 schema 鍵值為 my-flow，檔內其餘既有內容逐位元組不變

<!-- @trace
source: schema-engine-openspec-parity
updated: 2026-08-20
-->

---
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

<!-- @trace
source: no-spec-change-schema
updated: 2026-10-21T23:33:58+08:00
-->