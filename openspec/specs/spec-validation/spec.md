# spec-validation Specification

## Purpose

正式規格與 delta 規格的 Purpose 品質驗證：新開 capability 的 change 側早期檢查、archive 守門共用的合格判準與門檻常數，以及 validate --specs 的正式規格驗證面（缺席／過短／佔位偵測與嚴重度分級）。本 capability 保證「能力邊界說明」在寫入正式規格之前被要求、寫入之後可被追蹤。

## Requirements

### Requirement: Purpose 合格判準單一定義

引擎 SHALL 以單一判準函式定義 delta 與正式規格的 Purpose 合格性：存在 `## Purpose` 區段、內容非空、且 trim 後長度達最低門檻 50 字元（以字元計，非 bytes）。門檻常數 SHALL 單一定義；change 驗證的早期檢查、封存守門與正式規格驗證 SHALL 共用同一判準與常數，SHALL NOT 各自持有副本。archive 佔位文字的偵測前綴 SHALL 於引擎單一定義並公開，佔位產生器、正式規格驗證與桌面清單的佔位偵測 SHALL 引用同一常數。

#### Scenario: 三處守門共用同一門檻

- **WHEN** 門檻常數值調整
- **THEN** change 驗證、封存守門與正式規格驗證的合格判定同步改變，無任何一處殘留舊值

#### Scenario: 中文內容以字元計長

- **WHEN** 某 delta 的 Purpose 內容為 50 個中文字元
- **THEN** 判準以字元數 50 判定合格，不因 UTF-8 多位元組編碼被計為超過或不足


<!-- @trace
source: spec-purpose-gates
updated: 2026-08-11
-->

---
### Requirement: 新開 capability 的 change 驗證早期檢查

change 驗證 SHALL 對每個 delta capability 判斷正式規格是否已存在該 capability：不存在（本 change 新開）且 delta 的 Purpose 不合格時 SHALL 報 error、使該 change 驗證結果為 invalid；已存在時 SHALL NOT 因 Purpose 檢查報任何 error 或 warning（既有 capability 的 delta Purpose 屬忽略語意，不構成違規）。錯誤訊息 SHALL 自帶修復指引：說明規則（新 capability 的 delta 以 `## Purpose` 開頭、一兩句、50 字元以上）並附含 `## Purpose` 的範例骨架，SHALL NOT 僅報缺失而不附修法。

#### Scenario: 新開 capability 缺 Purpose 驗證報 error

- **WHEN** 某 change 的 delta 新開一個正式規格尚無的 capability 且 delta 檔無 Purpose 區段，執行該 change 的驗證
- **THEN** 驗證結果 invalid，error 訊息指出該 capability 需以 `## Purpose` 開頭並附範例骨架

#### Scenario: 既有 capability 的 delta 不受 Purpose 檢查影響

- **WHEN** 某 change 的 delta 僅修改正式規格已有的 capability 且 delta 無 Purpose 區段，執行該 change 的驗證
- **THEN** 驗證不因 Purpose 產生任何 error 或 warning

#### Scenario: 新開 capability 的 Purpose 合格則通過

- **WHEN** 新開 capability 的 delta 以 60 字元的 `## Purpose` 開頭，執行該 change 的驗證
- **THEN** 驗證不因 Purpose 產生任何 error 或 warning


<!-- @trace
source: spec-purpose-gates
updated: 2026-08-11
-->

---
### Requirement: validate --specs 驗證正式規格

CLI validate 動詞的 `--specs` 旗標 SHALL 對正式規格逐份驗證並依既有 validate 渲染呈現（逐項通過／不通過與訊息清單，任一 invalid 非零收尾）：缺 `## Purpose` 區段或內容為空 SHALL 報 error；內容 trim 後不足 50 字元 SHALL 於 strict 模式報 warning；內容以 archive 佔位前綴開頭 SHALL 報 warning（不依附 strict）。`--specs` 單獨傳入時 SHALL 僅驗規格；`--all` SHALL 同時驗 changes 與 specs；兩旗標皆缺席時行為 SHALL 維持現行 change 驗證不變；`--specs` 與名稱（item）同傳 SHALL 以參數錯誤拒絕（--specs 驗全部正式規格、無法指定單一份），錯誤訊息 SHALL 指路單獨 `--specs` 或 `--all`，SHALL NOT 靜默作聯集或忽略旗標。remote 模式下 `--specs` SHALL 由 client 以既有正式規格讀取動詞取得內容並本地執行同一驗證器，輸出形狀 SHALL 與 fs 模式一致，SHALL NOT 新開 server 端點。

#### Scenario: 佔位 Purpose 以 warning 顯形

- **WHEN** 正式規格含一份 Purpose 仍為 archive 佔位文字的規格，執行 validate --specs（非 strict）
- **THEN** 該規格報含佔位語意的 warning；佔位句雖長於 50 字元，長度門檻不攔截它

#### Scenario: 缺 Purpose 區段報 error

- **WHEN** 正式規格含一份無 `## Purpose` 區段的規格，執行 validate --specs
- **THEN** 該規格 invalid、報 error，命令非零收尾

#### Scenario: 過短 Purpose 僅 strict 報 warning

- **WHEN** 正式規格含一份 Purpose 內容 30 字元的規格，分別執行 validate --specs 與 validate --specs --strict
- **THEN** 前者不報該項；後者報不足門檻的 warning

#### Scenario: 預設行為不變

- **WHEN** 執行 validate（無 --specs 無 --all）
- **THEN** 僅驗 changes，輸出與旗標接線前一致

#### Scenario: --specs 與 change 名稱同傳被拒

- **WHEN** 執行 validate <change-name> --specs
- **THEN** 命令以非零收尾，錯誤說明 --specs 不能與名稱同傳，並指路單獨 --specs 或 --all

<!-- @trace
source: spec-purpose-gates
updated: 2026-08-11
-->

---
### Requirement: 新開 capability 的近似名 warning

`speclink validate <change>` 對 delta 中「正式規格無同名」（以正式規格 capability 清單逐字比對）的每個 capability，SHALL 以與建立點主閘相同的建議池（正式規格 capabilities 加未封存 change 的 delta capabilities，含同 change 的其他 delta、僅排除受檢 capability 自身）與相同排序規則求取近似名；建議池非空時 SHALL 報 warning 級發現，訊息 SHALL 含近似名清單與指引——同一 capability 就把 delta 目錄改用既有名、確為新 capability 可忽略本警告。此 warning SHALL NOT 改變驗證結果的通過與否，SHALL NOT 影響 exit code。正式規格已有同名規格的 delta capability SHALL NOT 觸發此檢查；建議池為空時 SHALL NOT 報 warning。

#### Scenario: 近似新名報 warning 且驗證仍通過

- **WHEN** 正式規格有 `auth`，change 的 delta 含目錄 `authentication` 且其餘內容全部合法，執行 `speclink validate <change>`
- **THEN** 輸出含一筆指名 `authentication` 與近似名 `auth` 的 warning，驗證結果為通過，exit code 為 0

#### Scenario: 既有 capability 的 delta 不觸發

- **WHEN** change 的 delta 目錄名稱與正式規格同名
- **THEN** 驗證輸出不含近似名 warning

#### Scenario: 無近似名的新 capability 不報

- **WHEN** change 的 delta 含正式規格未收錄、且與所有既有名毫無交集的目錄名稱
- **THEN** 驗證輸出不含近似名 warning（既有的新開 capability Purpose 檢查不受影響、照常執行）

<!-- @trace
source: capability-naming-guard
updated: 2026-08-21
-->

---
### Requirement: change 驗證納入合併守門

change 的驗證（`speclink validate <change>`、無參數、`--all`、`--changes`，fs 與 remote 兩模式，以及 desktop 的結構驗證）SHALL 對每個 delta capability 執行與 archive 相同的合併守門判斷，每筆違規 SHALL 化為一條 error 並使結果 invalid（文字完全相同的違規只列一次）。error 文字 SHALL 為 `specs/<capability>/spec.md: <operation> '<requirement>': <reason> (see: speclink drift <change>)`，`<reason>` 逐字沿用守門的既有字串，路徑一律正斜線。守門 error SHALL 排在所有既有結構 error 之後，既有 error 的文字與順序 SHALL NOT 改變。已由結構檢查報過的項目 SHALL NOT 重複：新開 capability 的 Purpose 不合格只報既有的 Purpose error；同一 delta 內同名需求已報 Duplicate 或 appears in both 時不再報守門的撞名 error，未被結構檢查涵蓋的撞名（含 RENAMED 端點）仍 SHALL 列出。單筆與 bulk archive 的驗證前置 SHALL 只含結構檢查，archive 的拒絕輸出 SHALL 維持既有位元級輸出。守門違規 SHALL 不論 `--strict` 一律為 error。

#### Scenario: MODIFIED 目標不存在時 validate 報 error

- **WHEN** 正式規格 `specs/auth/spec.md` 只有需求 `R1`，change `demo` 的 delta 在 `## MODIFIED Requirements` 寫 `R9`，執行 `speclink validate demo`
- **THEN** 人眼輸出列 `✗ demo — invalid` 與 error `specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift demo)`，以 `Validation failed.` 非零收尾；`--json` 的 `valid` 為 false、`errors` 含該字串

#### Scenario: 未宣告的 scenario 移除被 validate 抓到

- **WHEN** 正式規格 `R1` 有 scenario `ok` 與 `fine`，delta 的 MODIFIED `R1` 只寫 `ok` 且沒有 `<!-- REMOVED-SCENARIO: fine -->`
- **THEN** validate 報一條 error，reason 為守門的 `drops canonical scenario(s) fine` 開頭字串；補上宣告後 validate 通過

#### Scenario: Purpose 與重複需求名不重複列

- **WHEN** 新開 capability `token` 的 delta 缺 `## Purpose`，且 `## ADDED Requirements` 寫了兩次 `Fresh`
- **THEN** errors 恰有 2 條：既有的 Purpose error（含 `## Purpose` 範例骨架）與 `Duplicate requirement 'Fresh' in ADDED section`，沒有守門的第三條

#### Scenario: RENAMED 端點撞名只有守門看得到

- **WHEN** 正式規格有 `R1`，delta 的 `## RENAMED Requirements` 把 `R1` 改成 `R2`，`## ADDED Requirements` 也寫 `R2`
- **THEN** validate 報 1 條守門 error，`<requirement>` 為 `R2`、reason 為守門的撞區段字串

#### Scenario: archive 的拒絕輸出不變

- **WHEN** 同一份 MODIFIED `R9` 的 change 執行 `speclink archive demo`
- **THEN** stderr 為守門的聚合拒絕清單（`change 'demo' cannot be archived — 1 delta operation(s) no longer match the canonical spec:` 起頭），與本需求加入前逐位元相同

##### Example: 守門 error 的組字

| 違規 | error |
| --- | --- |
| MODIFIED `R9` 目標不存在 | `specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift demo)` |
| ADDED `R1` 撞正式規格 | `specs/auth/spec.md: ADDED 'R1': already exists in the canonical spec — archive would refuse it (see: speclink drift demo)` |
| RENAMED 缺 TO | `specs/auth/spec.md: RENAMED 'R1': RENAMED operation names no TO: target (see: speclink drift demo)` |

<!-- @trace
source: validate-merge-gate
updated: 2026-09-09T15:14:16+08:00
-->