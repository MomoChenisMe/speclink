## Purpose

正典規格與 delta 規格的 Purpose 品質驗證：新開 capability 的 change 側早期檢查、archive 守門共用的合格判準與門檻常數，以及 validate --specs 的正典規格驗證面（缺席／過短／佔位偵測與嚴重度分級）。本 capability 保證「能力邊界說明」在寫入正典之前被要求、寫入之後可被追蹤。

## ADDED Requirements

### Requirement: Purpose 合格判準單一定義

引擎 SHALL 以單一判準函式定義 delta 與正典規格的 Purpose 合格性：存在 `## Purpose` 區段、內容非空、且 trim 後長度達最低門檻 50 字元（以字元計，非 bytes）。門檻常數 SHALL 單一定義；change 驗證的早期檢查、封存守門與正典規格驗證 SHALL 共用同一判準與常數，SHALL NOT 各自持有副本。archive 佔位文字的偵測前綴 SHALL 於引擎單一定義並公開，佔位產生器、正典規格驗證與桌面清單的佔位偵測 SHALL 引用同一常數。

#### Scenario: 三處守門共用同一門檻

- **WHEN** 門檻常數值調整
- **THEN** change 驗證、封存守門與正典規格驗證的合格判定同步改變，無任何一處殘留舊值

#### Scenario: 中文內容以字元計長

- **WHEN** 某 delta 的 Purpose 內容為 50 個中文字元
- **THEN** 判準以字元數 50 判定合格，不因 UTF-8 多位元組編碼被計為超過或不足

### Requirement: 新開 capability 的 change 驗證早期檢查

change 驗證 SHALL 對每個 delta capability 判斷正典是否已存在該 capability：不存在（本 change 新開）且 delta 的 Purpose 不合格時 SHALL 報 error、使該 change 驗證結果為 invalid；已存在時 SHALL NOT 因 Purpose 檢查報任何 error 或 warning（既有 capability 的 delta Purpose 屬忽略語意，不構成違規）。錯誤訊息 SHALL 自帶修復指引：說明規則（新 capability 的 delta 以 `## Purpose` 開頭、一兩句、50 字元以上）並附含 `## Purpose` 的範例骨架，SHALL NOT 僅報缺失而不附修法。

#### Scenario: 新開 capability 缺 Purpose 驗證報 error

- **WHEN** 某 change 的 delta 新開一個正典尚無的 capability 且 delta 檔無 Purpose 區段，執行該 change 的驗證
- **THEN** 驗證結果 invalid，error 訊息指出該 capability 需以 `## Purpose` 開頭並附範例骨架

#### Scenario: 既有 capability 的 delta 不受 Purpose 檢查影響

- **WHEN** 某 change 的 delta 僅修改正典已有的 capability 且 delta 無 Purpose 區段，執行該 change 的驗證
- **THEN** 驗證不因 Purpose 產生任何 error 或 warning

#### Scenario: 新開 capability 的 Purpose 合格則通過

- **WHEN** 新開 capability 的 delta 以 60 字元的 `## Purpose` 開頭，執行該 change 的驗證
- **THEN** 驗證不因 Purpose 產生任何 error 或 warning

### Requirement: validate --specs 驗證正典規格

CLI validate 動詞的 `--specs` 旗標 SHALL 對正典規格逐份驗證並依既有 validate 渲染呈現（逐項通過／不通過與訊息清單，任一 invalid 非零收尾）：缺 `## Purpose` 區段或內容為空 SHALL 報 error；內容 trim 後不足 50 字元 SHALL 於 strict 模式報 warning；內容以 archive 佔位前綴開頭 SHALL 報 warning（不依附 strict）。`--specs` 單獨傳入時 SHALL 僅驗規格；`--all` SHALL 同時驗 changes 與 specs；兩旗標皆缺席時行為 SHALL 維持現行 change 驗證不變；`--specs` 與名稱（item）同傳 SHALL 以參數錯誤拒絕（--specs 驗全部正典規格、無法指定單一份），錯誤訊息 SHALL 指路單獨 `--specs` 或 `--all`，SHALL NOT 靜默作聯集或忽略旗標。remote 模式下 `--specs` SHALL 由 client 以既有正典規格讀取動詞取得內容並本地執行同一驗證器，輸出形狀 SHALL 與 fs 模式一致，SHALL NOT 新開 server 端點。

#### Scenario: 佔位 Purpose 以 warning 顯形

- **WHEN** 正典含一份 Purpose 仍為 archive 佔位文字的規格，執行 validate --specs（非 strict）
- **THEN** 該規格報含佔位語意的 warning；佔位句雖長於 50 字元，長度門檻不攔截它

#### Scenario: 缺 Purpose 區段報 error

- **WHEN** 正典含一份無 `## Purpose` 區段的規格，執行 validate --specs
- **THEN** 該規格 invalid、報 error，命令非零收尾

#### Scenario: 過短 Purpose 僅 strict 報 warning

- **WHEN** 正典含一份 Purpose 內容 30 字元的規格，分別執行 validate --specs 與 validate --specs --strict
- **THEN** 前者不報該項；後者報不足門檻的 warning

#### Scenario: 預設行為不變

- **WHEN** 執行 validate（無 --specs 無 --all）
- **THEN** 僅驗 changes，輸出與旗標接線前一致

#### Scenario: --specs 與 change 名稱同傳被拒

- **WHEN** 執行 validate <change-name> --specs
- **THEN** 命令以非零收尾，錯誤說明 --specs 不能與名稱同傳，並指路單獨 --specs 或 --all
