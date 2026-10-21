## Context

內建 schema 目前只有 spec-driven，「它是唯一內建」寫死在多處：crates/engine/speclink-core/src/workspace/schema.rs 的 `sources`、`resolve_with`、`builtin_template` 以 `name == "spec-driven"` 判斷，`list_all` 只放 `spec_driven()` 一項；apps/desktop/core/src/settings.rs 的 `schemas_snapshot` 也只放 `spec_driven()`，遠端模式的 `builtin_schemas()` 因此只有一項。spec-driven 由 assets/schema/spec-driven/fork.schema.yaml 與四份 template 定義，`spec_driven()` 以 `parse_schema` 走與自訂 schema 相同的路徑解析。

`speclink new change --schema <name>` 已存在且不驗證名稱；`speclink validate` 在變更沒有 delta 時一律警告「No delta specs found」（quality/validate.rs 的結構檢查）；archive 對沒有 delta 的變更照常封存（delta 迴圈為空）；analyzer 在沒有 specs 時略過涵蓋與模糊度檢查。`speclink workflow-config show` 只輸出政策欄位、context、rules 與 review，沒有 schema——目前沒有任何指令能讀到專案預設 schema。propose 技能（assets/skills/propose.md）建立變更時從不帶 `--schema`；verify 技能（assets/skills/verify.md）的規格涵蓋與需求對應都以 delta spec 為準。

## Goals / Non-Goals

**Goals:**

- 內建第二個 schema `no-spec-change`，在 CLI、server 與桌面（含遠端模式）都能解析、列出與 fork。
- propose 依「會不會改到正式規格承諾的行為」選 schema，verify 對不改規格的變更檢查行為沒有被改到。
- spec-driven 的一切輸出位元級不變。

**Non-Goals:**

- 不提供切換已建立變更 schema 的指令，也不在 no-spec-change 變更寫入 delta 時拒絕——兩者由下一刀 change-schema-verb 一起處理（拒絕訊息要指向那個指令）。
- 不在看板或詳情頁顯示變更的 schema。
- 不驗證 `new change --schema` 的名稱（現行行為）。
- 不加第三個內建 schema。
- 不修改 ingest、apply、archive、review 技能。

## Decisions

### D1：內建 schema 登記表

schema.rs 新增一張內建登記表，依序為 spec-driven、no-spec-change，每筆含名稱、正典 YAML 常數與 template 查找函式；提供 `builtin(name) -> Option<Schema>`（各自以 `OnceLock` 快取、走 `parse_schema`）與 `builtin_names()`。`sources`、`resolve_with`、`builtin_template` 與 `list_all` 改查登記表；`spec_driven()` 保留為 `builtin("spec-driven")` 的薄包裝，既有呼叫端不動。解析順序維持 project → user → 內建。替代方案「在每個判斷點加 `|| name == "no-spec-change"`」被否決：第三個判斷點漏改就會出現只在某條路徑解析得到的內建。

### D2：template 單一來源

no-spec-change 只新增自己的 proposal.template.md（Why、What Changes、Non-Goals（選用）、Impact，沒有 Capabilities）；design 與 tasks 的 template 在 template 查找函式中對應到 spec-driven 既有的內嵌資產，同一份文字不複製兩次。design 的 instruction 在 no-spec-change 的 YAML 內與 spec-driven 逐字相同。`schema fork no-spec-change` 照既有 fork 流程寫出 schema.yaml 與 templates/ 下三份檔。

### D3：validate 依 schema 決定是否警告缺 delta

結構檢查在發出「No delta specs found」之前，先解析變更的 schema：解析成功且沒有 id 為 `specs` 的 artifact → 不警告；解析失敗或含 specs artifact → 維持現行警告。這樣自訂 schema 若沒有 specs 也一併受惠，且不寫死 no-spec-change 的名稱。解析由 validate.rs 的 `change_schema`（project → user → 內建，解析不到時以 spec-driven 代之）負責，命令層的 validate 與桌面的 validate 入口都改用它，不再一律傳 spec-driven；archive 的結構前置只讀 `valid`，維持現狀。

### D4：propose 技能的選擇步驟

在 propose.md 的「Scan existing specs」之後新增一步「Choose the schema」，寫明四條依序規則、判斷不了時以 AskUserQuestion 詢問兩個選項（建議者在前並附理由），並把 `--schema <chosen>` 加進建立變更的兩個指令範例。proposal 的撰寫步驟加一段：選了 no-spec-change 時依該 schema 的 template 撰寫，Why 寫明不改行為的理由與查過的規格。收尾摘要寫出選定的 schema。新增文字不加大寫強調字，維持 skill_authoring 的強調字上限（propose.md 為 6）與 500 行上限。

### D5：verify 技能的行為檢查分支

verify.md 在讀取 status 的步驟記下 `schemaName`；在 Verify Completeness 與 Verify Correctness 兩步加入 no-spec-change 分支：不做以 delta 為準的 Spec Coverage、Requirement Implementation Mapping、Scenario Coverage 與 Example Traceability，改做行為檢查——從 proposal Why 列出的正式規格出發，以凍結 patch 的具體名稱找涵蓋這些區域的需求並比對；改到承諾行為即回報 WARNING `Behavior change in a no-spec-change change: <capability> / <requirement>`。WARNING 為必修，沿用既有擋章規則，不改引擎。

### D6：版號與再生

技能原稿改變，ASSET_VERSION 由 v1.49.0 升為 v1.50.0，再生 golden snapshot 與 assets.lock，並以新建置的 CLI 執行 `speclink update` 再生 repo 自用的技能檔。同批其他變更（readme-positioning、process-cost-metrics）不改技能原稿，不會撞版號；之後的 change-schema-verb、stamp-order-rules 在本變更之後依序升版。

### D7：workflow-config show 輸出 schema

propose 規則 2 要判斷「專案預設是不是自訂 schema」，需要一個讀得到專案預設的指令。`workflow-config show` 本來就是 config.yaml 正典內容的唯讀出口，因此由它多輸出 schema：人眼輸出在 locale 行之前印 `schema` 行（未設定為 `unset (spec-driven)`），`--json` 多 `schema` 欄位（字串，未設定為 null）。fs 與 remote 共用同一個輸出函式，兩條路徑一起成立。替代方案「技能用 `speclink schemas --json` 看有沒有自訂 schema、有就詢問」被否決：預設是自訂 schema 的專案每次 propose 都多問一題；「判定為 spec-driven 時不帶 `--schema`」也被否決：判定為 no-spec-change 時仍會蓋掉自訂的預設。

## Implementation Contract

- **解析**：`speclink schemas`、`speclink schema which --all`、`speclink schema fork no-spec-change`、`speclink new change <n> --schema no-spec-change` 與 `speclink status --change <n> --json` 的行為依 workflow-schemas 的 MODIFIED 與 ADDED 需求成立；spec-driven 的 `schemas` 列、instructions 輸出與 fork 結果位元級不變。驗收：`cargo test -p speclink-core schema`、`cargo test -p speclink-cli --test it`。
- **validate**：no-spec-change 變更沒有 delta 時不警告，spec-driven 變更照舊警告。驗收：speclink-core 的 validate 測試。
- **workflow-config show**：人眼輸出在 locale 行之前多一行 `schema`、`--json` 多 `schema` 欄位（未設定為 null），其餘各行與欄位位元級不變。驗收：`cargo test -p speclink-cli --test it workflow_config`。
- **桌面**：本機與遠端的產出流程快照都列出兩個內建，遠端以內嵌定義解析 no-spec-change。驗收：`cargo test -p speclink-desktop-core settings`。
- **技能**：生成的 speclink-propose 含四條選擇規則、詢問規則與 `--schema`；生成的 speclink-verify 含行為檢查分支與 WARNING 字面。驗收：render_golden 新測試與 skill_authoring。
- **範圍內**：schema.rs、validate.rs 與它在命令層和桌面的呼叫端、CLI 的 workflow-config show、兩份技能原稿、兩份新資產、desktop core 的快照、LANGUAGE.md、版號與再生物。**範圍外**：切換指令、寫 delta 的拒絕、看板 schema 標示、其他技能。

## Risks / Trade-offs

- [回歸對照：`schemas` 與 `schema which --all` 的輸出多一項] → 屬刻意變更，同批更新對應的 CLI 測試期望值；spec-driven 相關 golden 不得變動，若變動代表 D1 改壞了既有路徑。
- [回歸對照：`workflow-config show` 的人眼輸出多一行] → 屬刻意變更，同批更新 CLI 測試的整段期望值；其餘各行逐位元不變。
- [回歸對照：desktop 快照測試斷言只有一個內建] → 同批更新 settings.rs 的測試期望值為兩個內建、spec-driven 在前。
- [遠端 `new artifact` 不帶 `--stdin` 時以專案預設 schema 取 template] → 現行 crates/adapters/speclink-cli/src/verbs/new.rs 的遠端路徑取 config 的 schema 而非變更的 schema；技能一律用 `--stdin`，不受影響，記為已知限制，不在本變更修。
- [AI 選錯 schema] → verify 的行為檢查會以必修 WARNING 擋章；下一刀的切換指令讓修正成本變小。
- [跨平台] → 內嵌資產與 fork 寫檔沿用既有路徑處理；fork 測試在 Windows／macOS／Linux 的 CI 都會跑。

## Migration Plan

不需要遷移：既有變更的 schema 欄位不變，專案預設不變。發版後新變更由 propose 技能依規則選擇 schema。回滾時，舊版遇到 `schema: no-spec-change` 的變更會回報 schema 找不到，需要先封存或改回 spec-driven。

## Open Questions

(none)
