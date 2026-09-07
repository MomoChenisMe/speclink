# manual-skill Specification

## Purpose

/speclink-manual 技能的內容契約：技能檔渲染到 claude／codex／neutral 三目標、以引數選擇生成或導覽動線、生成模式的讀取策略與只重生受影響頁的輸出與報告、導覽模式的對話內行為，以及 remote 模式下的限制敘述。本 capability 保證技能文字與 manual-pages 契約及引擎實際行為一致；不涵蓋頁格式本身（見 manual-pages）與任何讀取端。

## Requirements

### Requirement: 技能檔的渲染

內嵌 speclink-manual 技能（事實來源 crates/speclink-core/assets/skills/manual.md）SHALL 經 init 與 update 渲染至 claude（`.claude/skills/speclink-manual/SKILL.md`）、codex 與 neutral 三種目標。其 description SHALL 以觸發情境句開場（需要一份人類操作手冊、或想被導覽如何操作系統時），再以一句說明產出。渲染產物由 speclink-core 的 render_golden 測試保護，golden 快照更新屬刻意變更。

#### Scenario: 三目標皆渲染

- **WHEN** 於 tools 含 claude 與 codex 的工作區執行 speclink update
- **THEN** 兩種工具的技能目錄各出現 speclink-manual 技能檔，內容與 golden 快照一致

#### Scenario: description 以觸發情境開場

- **WHEN** 檢視渲染產出的 speclink-manual 技能檔 frontmatter 的 description
- **THEN** 句子先陳述觸發情境（需要人類操作手冊、或想被導覽），再說明產出（生成 openspec/manual/ 的 Markdown 手冊，或對話內導覽）


<!-- @trace
source: manual-skill
updated: 2026-09-02
-->

---
### Requirement: 動線由引數選擇

技能 SHALL 依呼叫引數分流：無引數 SHALL 走生成模式；引數含「導覽」或「tour」SHALL 走導覽模式；其他引數 SHALL 視為生成模式的範圍提示（例如只重生指定分區或指定頁）並於摘要中覆述所理解的範圍。

#### Scenario: 無引數走生成

- **WHEN** 使用者呼叫 /speclink-manual 不帶引數
- **THEN** 技能進入生成模式，結束時輸出生成摘要

#### Scenario: 導覽引數走導覽

- **WHEN** 使用者呼叫 /speclink-manual 導覽（或 tour）
- **THEN** 技能進入導覽模式，不寫任何檔案


<!-- @trace
source: manual-skill
updated: 2026-09-02
-->

---
### Requirement: 生成模式的讀取策略

生成模式 SHALL 只以正式規格為內容來源：先以 speclink list --specs --json 取得全部 capability，再以 speclink show 讀各 capability 的 Purpose，將其分流為使用者面向與引擎內部；Purpose 為空或為 TBD 佔位時 SHALL 改以該 capability 的 Requirement 標題判斷。旅程骨幹 SHALL 依序優先取自劇本型規格（驗收劇本）、路由交棒型規格（技能交棒表）、使用者文件型規格；三者皆無時 SHALL 按功能域從使用者面向的能力規格重建旅程，並在 about 頁載明屬重建。技能 SHALL NOT 以 README、docs 目錄或程式碼作為手冊內容來源。已有手冊時，每個 capability 的 Purpose 與其全部 @trace updated 時戳仍 SHALL 讀取（分流與過期判定所需；時戳得為 RFC 3339 或純日期，依 manual-pages 契約「過期判定基準」分段比較），Requirement 內文 SHALL 只讀過期頁與未入冊能力所涉的規格。

#### Scenario: 有劇本型規格的專案

- **WHEN** 規格中存在驗收劇本與交棒表型的 capability
- **THEN** 手冊的旅程章節順序與站別取自該等規格，about 頁載明旅程「轉寫自」哪些規格

#### Scenario: 無劇本型規格且 Purpose 為 TBD 的專案

- **WHEN** 規格皆無劇本型內容且多數 Purpose 為 TBD 佔位
- **THEN** 技能以 Requirement 標題分流、按功能域重建旅程，about 頁載明屬重建並列出規格內部的新舊矛盾

#### Scenario: 不讀規格以外的來源

- **WHEN** 工作區同時存在 README 與 docs 目錄
- **THEN** 生成的任何頁的內容與出處只引用 openspec/specs/ 的 capability，不引用 README 或 docs

#### Scenario: 同日先封存後生成的頁不列為過期

- **WHEN** 手冊已存在，一頁 generated 為 2026-09-05T23:31:00+08:00，其 sources 規格最新 @trace updated 為 2026-09-05T23:17:28+08:00
- **THEN** 技能的過期報告不列該頁，也不重讀該規格的 Requirement 內文


<!-- @trace
source: manual-stale-time-granularity
updated: 2026-09-07T12:40:29+08:00
-->

---
### Requirement: 生成模式的輸出與報告

生成模式 SHALL 依 manual-pages 契約寫頁，每頁 frontmatter 的 generated SHALL 為生成當下帶時區偏移量的 RFC 3339 時戳（秒級，例 2026-09-05T23:31:00+08:00），技能檔 SHALL 給 agent 一條取得該時戳的建議指令；已有手冊時預設 SHALL 只重生過期頁並為未入冊能力新增頁，使用者明示要求時方全量重生。結束時 SHALL 於對話輸出摘要：新增、重生、未動的頁數；可能過期的頁清單；未入冊能力清單；about 頁記錄的矛盾數。無過期頁且無未入冊能力時 SHALL 零檔案寫入並如實回報。摘要末尾 SHALL 建議以一般提交收尾手冊異動——僅建議、SHALL NOT 代跑。

#### Scenario: 首次全量生成

- **WHEN** 工作區無 openspec/manual/ 而執行生成模式
- **THEN** 產出含 index.md 與 about.md 的完整手冊，每頁 generated 為 RFC 3339 時戳，摘要列出新增頁數且重生與未動為零

#### Scenario: 二次只重生過期頁

- **WHEN** 手冊已存在且兩頁過期、一個能力未入冊
- **THEN** 僅該兩頁被重寫（generated 換成本次時戳）、新增一頁，其餘頁逐位元不變，摘要列出可能過期的頁與新入冊能力

#### Scenario: 無異動時零寫入

- **WHEN** 手冊已存在且無任何過期頁或未入冊能力
- **THEN** 無檔案被寫入，摘要明示手冊已是最新


<!-- @trace
source: manual-stale-time-granularity
updated: 2026-09-07T12:40:29+08:00
-->

---
### Requirement: 導覽模式的行為

導覽模式 SHALL NOT 寫入任何檔案。有手冊時 SHALL 讀取各頁 frontmatter 為索引，先以一個問題確認使用者角色，再依 section 與 order 帶領旅程，每個回答 SHALL 附出處（capability 名或手冊頁名）。無手冊時 SHALL 明示「尚無手冊，改以規格直接導覽」並以 speclink list --specs 與 speclink show 讀規格作答。導覽結束時的後續建議（例如跑生成模式產出手冊）SHALL 明文為僅建議、SHALL NOT 自動呼叫其他技能。

#### Scenario: 有手冊的導覽

- **WHEN** openspec/manual/ 存在且使用者呼叫導覽模式
- **THEN** 技能先問角色一題，依手冊順序導覽，回答附出處，過程中零檔案寫入

#### Scenario: 無手冊的導覽

- **WHEN** openspec/manual/ 不存在且使用者呼叫導覽模式
- **THEN** 技能明示尚無手冊並改以規格直接導覽，過程中零檔案寫入


<!-- @trace
source: manual-skill
updated: 2026-09-02
-->

---
### Requirement: remote 模式的限制敘述

技能 SHALL 於生成模式開始前判斷專案是否綁定 remote store（.speclink.yaml 含 remote 區段）：綁定時 SHALL 明示 remote 模式尚不支援手冊生成並停止，零檔案寫入；導覽模式不受此限。技能檔 SHALL 敘明此限制。

#### Scenario: remote 綁定專案的生成被擋

- **WHEN** 於 remote 綁定的專案呼叫生成模式
- **THEN** 技能輸出不支援的說明並停止，openspec/manual/ 未被建立或改動

#### Scenario: remote 綁定專案的導覽照常

- **WHEN** 於 remote 綁定的專案呼叫導覽模式
- **THEN** 導覽照常進行（以規格或既有手冊為索引），零檔案寫入

<!-- @trace
source: manual-skill
updated: 2026-09-02
-->