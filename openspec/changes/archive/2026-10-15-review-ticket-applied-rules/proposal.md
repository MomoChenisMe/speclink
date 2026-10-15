## Why

審查規則自訂的第一刀（review-rules-config）讓 config.yaml 的 `review` 區塊決定 Standards 軸套用哪些規則，但這份資訊只出現在執行審查那個對話的呈現開頭行，沒有寫進工單。隊友、其他電腦與其他 session 打開 desktop 的變更詳情頁時，看不到這次審查依哪些規則判定；之後若改了 config，也沒有辦法知道首輪當時套用的是哪一版。

目標使用者是透過 AI 代理跑 SDD 的開發者與 PO／PM：前者在 `/speclink-review`（以及 `/speclink-quality` 轉交的審查）階段產生工單，後者在 desktop 變更詳情頁的「審查」分頁閱讀結果。

## What Changes

- **工單格式**：審查工單的首輪（discovery）可帶一行 `**Rules**: extend|replace`，以及每條專案規則一行 `**Rule**: <CRITICAL|WARNING|SUGGESTION> <規則原文>`。複驗輪不寫。引擎解析器比照 `**Phase**:` 檢查格式：值不合法、`**Rule**:` 沒有 `**Rules**:`、出現在非 discovery 輪、同輪多個 `**Rules**:` 時，`speclink review add-round` 以非零 exit code 拒絕且工單零寫入。兩行皆為選填，舊工單與不寫這兩行的工單照舊可建立、追加與解析。
- **`speclink review show --json`**：`rounds[]` 與 `lastRound` 每輪新增 `rules` 欄位，值為 `{ baseline, critical[], warning[], suggestion[] }`，該輪沒有 `**Rules**:` 時為 `null`。local fs、remote CLI、typed client 與 server response 同形。子指令、旗標與 stdin 不變。
- **審查技能**：discovery 輪記錄時，依開審前讀到的 `review` 物件寫入上述行；未設定 review 區塊時只寫 `**Rules**: extend`。規則原文含換行時改以單一空白連接成一行。
- **desktop 審查分頁**：帶有 `rules` 的那一輪，展開後、範圍路徑之前多一列「套用規則」，可展開與收合：
  - 沒有專案規則且為 extend → 只寫「預設 12 條 code smell」，不顯示計數籤（帶入討論 config-skill-rule-routing 的顯示決定：審查規則是選用的，大多數專案沒有專案規則）。
  - 有專案規則 → 寫明模式，並以各嚴重度的條數籤呈現（為零的不出）。
  - replace 且沒有專案規則 → 琥珀底提示「只依 repo 文件審查」。
  - 展開後：extend 時列出 12 條 smell 名稱（取自引擎），replace 時寫明沒有套用；再依嚴重度列出專案規則原文；末尾註明之後改規則這裡不會跟著變。
  - `rules` 為 `null`（舊工單、驗證站工單）時不顯示這一列。
- **技能版號**：`ASSET_VERSION` 由 v1.46.0 升為 v1.47.0。

### 相容性影響

- 人眼路徑：`speclink review show` 照舊印工單原文，新寫入的工單會多出 `**Rules**:`／`**Rule**:` 行。
- `--json`：`review show --json` 與 `verify show --json` 的每輪都多出 `rules` 欄位（驗證站共用同一份解析與序列化，驗證技能不寫這些行，所以驗證站恆為 `null`）。這是 additive 變更：既有欄位名稱、型別與 findings 順序不變，回歸對照只需補上新欄位。
- 舊 server 回傳的輪次沒有 `rules` 時，新 CLI 與 desktop 視為 `null`；server 回傳不認得的 baseline token 時，CLI 比照 phase 與 severity 報錯（「這支 CLI 比 server 舊」），不靜默略過。
- 技能檔：升版號後既有專案的技能檔會被標為過期，執行 `speclink update` 即可；舊版審查技能不寫這兩行，desktop 對其工單不顯示「套用規則」列，不會出錯。
- 設定欄位：不新增也不改變 `openspec/config.yaml` 或 `.speclink.yaml` 的欄位。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `review-station`: 工單的建立與追加新增 `**Rules**:`／`**Rule**:` 文法與守門；工單的讀取新增每輪 `rules` 欄位。
- `review-skill`: 新增「首輪工單記錄套用的規則」——discovery 輪依讀到的 review 物件寫入規則行。
- `desktop-app`: 詳情頁的工單分頁新增「套用規則」列，工單資料形狀新增 `rules` 與引擎提供的 12 條 smell 名稱。

前置掃描命中的正式規格：review-station、review-skill（含第一刀新增的「專案審查規則的讀取與套用」「審查呈現標明套用的規則」）、desktop-app（「詳情頁的工單分頁」「已封存詳情頁的工單分頁」）、verify-station（共用解析器，本刀不改其規格）。

## Impact

- Affected specs: review-station、review-skill、desktop-app
- Affected crates／apps：speclink-core（工單解析與序列化、審查技能範本、技能版號）、speclink-protocol（輪次 wire 形狀）、speclink-host 的 speclink-server（輪次 DTO 對映）、speclink-cli（remote 輪次對映）、packages/ui（工單分頁元件與新的共用 smell 名稱清單元件）、apps/desktop（工單 payload 帶入 smell 名稱）
- 生成的技能：speclink-review，影響 claude、codex、copilot 三種工具；本 repo 自身的技能檔由 `speclink update` 再生
- Affected code:
  - New:
    - packages/ui/src/components/BaselineSmellList.tsx
  - Modified:
    - crates/engine/speclink-core/src/quality/station.rs
    - crates/engine/speclink-core/src/quality/station/tests.rs
    - crates/engine/speclink-core/src/workspace/config.rs
    - crates/engine/speclink-core/assets/skills/review.md
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/protocol/speclink-protocol/src/command.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/tests/it/api/review_api.rs
    - crates/host/speclink-server/tests/it/api/verify_api.rs
    - crates/adapters/speclink-cli/src/verbs/station.rs
    - crates/adapters/speclink-cli/tests/it/review_verbs.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - apps/desktop/core/src/query.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - apps/desktop/src/views/project-settings/ReviewRulesPage.tsx
    - packages/ui/src/adapter.ts
    - packages/ui/src/components/TicketView.tsx
    - packages/ui/src/i18n.tsx
    - packages/ui/src/index.ts
    - packages/ui/src/__tests__/ticketView.test.tsx
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
  - Removed: (none)
