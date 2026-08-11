## Why

67 份正典規格中 66 份的 Purpose 仍是 archive 產生的佔位文字——能力邊界說明從未被寫過。根因鏈（討論 archived-parity-and-spec-purpose 議題 4 查證）：代理寫 delta 時實際讀到的規格指引 asset 從未要求新 capability 帶 `## Purpose`（283 份封存 delta 僅 1 份寫過）、validate 對正典規格零檢查（CLI 的 `--specs` 旗標宣告後從未接線）、archive 對缺 Purpose 靜默寫入佔位。Purpose 的唯一機器消費者是 propose 的 capability 歸屬判斷——全數佔位使歸屬只能靠名稱猜，是 capability 邊界漂移的直接來源。

## What Changes

- **規格指引 asset 補 Purpose 規則段**：`specs.instruction.md` 於 Format requirements 與 MODIFIED workflow 之間補上（對齊上游 OpenSpec schema.yaml 的同段落）：新 capability 的 delta 以 `## Purpose` 開頭（一兩句、50 字元以上）；既有 capability 的 delta SHALL NOT 加 `## Purpose`（會被忽略）；要改既有 Purpose（含殘留佔位）直接編輯正典檔。此為規則的單一正典——技能層不複製規則內文（validate-only，與上游同形）。
- **change 驗證加 Purpose 早期檢查（主力機制）**：delta 對應的 capability 不存在於正典（＝新開）且缺 `## Purpose`／內容為空／不足 50 字元時，validate 報 error；錯誤訊息自帶修復指引與範例骨架（照上游 GUIDE_* 模式）——propose 技能收尾本就跑 validate 並帶修復迴圈，錯誤訊息即教材。
- **archive 對新 capability 硬擋**：同三種不合格情形拒絕放行（fail-closed，與封存合併守門既有風格一致），取代現行「靜默寫佔位」。既有 capability 的 delta 帶 Purpose 仍是忽略不報錯（向後相容）。
- **接上空轉的 `validate --specs`**：正典規格驗證——缺 `## Purpose` 區段或內容為空＝error；內容不足 50 字元＝warning（strict 下報）；內容仍為 archive 佔位句＝warning。`--specs` 於 remote 模式由 client 以既有正典規格讀取端點取內容、本地跑同一驗證器，輸出兩模式同形。
- 門檻常數單一定義（50 字元），change 驗證、archive 守門、正典規格驗證三處共用。

## Capabilities

### New Capabilities

- `spec-validation`：正典規格與 delta 規格的 Purpose 品質驗證——change 側早期檢查、archive 守門的合格判準、`validate --specs` 的正典規格驗證面與分級。

### Modified Capabilities

- `archive-merge`：「新 capability 的 Purpose 自 delta 帶入」由「缺席寫佔位」改為「缺席／不合格拒絕放行」。

## Impact

- Affected specs: `spec-validation`（新）、`archive-merge`（修改）
- Affected code:
  - Modified: crates/speclink-core/assets/schema/spec-driven/specs.instruction.md（規則段）、crates/speclink-core/src/validate.rs（早期檢查＋正典規格驗證）、crates/speclink-core/src/archive.rs（硬擋）、crates/speclink-core/src/command/mod.rs（Validate command 增 specs 分支）、crates/speclink-cli/src/verbs/checks.rs（--specs 接線）、crates/speclink-core/tests/golden/assets.lock 與 golden 快照（asset 內文變更的三連動：MARKER_VERSION、golden、assets.lock）
  - New: （無）
  - Removed: （無）
