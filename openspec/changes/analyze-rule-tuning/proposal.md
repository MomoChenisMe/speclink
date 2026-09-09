## Why

`speclink analyze` 的四個面向（Coverage／Consistency／Ambiguity／Gaps）目前沒有任何正典規格，規則只活在 crates/speclink-core/src/analyzer.rs 裡，而且該檔零個單元測試。另一個 repo（wadpilot）跑 analyze 時留下三個誤報 Warning，回溯後發現是兩條規則的設計問題，而非該 change 的 artifacts 有錯：Consistency 把 design 的 `###` 標題整串（含「決策一：」這類序號前綴）拿去 tasks.md 全文找，實際效果是逼人把標題抄進 tasks（本 repo 封存的 743 個 design 標題有 653 個被整串抄進 tasks）；Ambiguity 對 REMOVED 區塊的需求也要求 scenario，與 speclink 自己的 schema 範本（REMOVED 只寫 **Reason** 與 **Migration**）矛盾。討論 analyze-rule-tuning 已裁定：修規則、並新開 capability 把 analyze 的規則寫進正典。

## What Changes

- 新開 capability `change-analysis`：把 analyze 四個面向的現行規則（含本次修正）寫成正典規格，讓後續的規則調整有 delta 可落。
- Consistency 的 design 標題比對改為「本文或編號任一出現在 tasks 即算引用」：標題先拆成「編號」與「本文」，編號只認 `D1`、`決策一`、`Decision 1` 三種樣式（後接全形或半形冒號皆可）；`D1` 樣式比對時後一個字元必須不是數字，避免被 `D12` 誤命中；沒有前綴的標題行為與現在完全相同。
- Ambiguity 的 no-scenario 檢查跳過 REMOVED 區塊的需求；改對 REMOVED 需求檢查 `**Reason**` 與 `**Migration**` 是否齊備，缺任一者報一筆新的 Warning finding（訊息 key `ambRemovedNoReason`）。
- Ambiguity 的「具體值」判斷字元集加入全形引號「」『』與全形數字０-９；既有的 ASCII 數字、反引號、半形雙引號維持。
- Ambiguity 的英文弱語氣詞（should／may／might／consider／possibly）改用字邊界比對，shoulder、mayor、considerable 不再命中；TBD／TODO／???／TKTK 與 CJK 樣式的比對方式不變。
- analyzer.rs 補 `#[cfg(test)]` 單元測試模組，涵蓋上述每一條規則的正反例與無前綴回歸。
- Coverage 與 Gaps 的規則不動。

**相容性影響**：`analyze` 的人眼輸出與 `--json` 形狀（欄位集合、camelCase 命名、finding 的 `id`／`dimension`／`severity`／`location`／`summary`／`recommendation`／`summaryMsg`／`recommendationMsg`）不變；既有 finding 的訊息文字與 key 不變。變的是「哪些情況會產生 finding」：上述四種情況的誤報消失、REMOVED 缺 Reason／Migration 新增一種 Warning。同一份 artifacts 跑 analyze，finding 的編號（AMB-N）可能因為前面的誤報消失而前移。CLI 指令的子指令、旗標、stdin 與 exit code 都不變。不涉及設定欄位、不涉及生成的技能文字。目標使用者是透過 AI 代理跑 SDD 的開發者，情境是 propose 收尾的 Analyze-Fix Loop 與 archive 前的自查。

## Capabilities

### New Capabilities

- `change-analysis`: `speclink analyze` 對單一 change 的四面向交叉檢查規則——每個面向的前置 artifact、每條規則的觸發條件、嚴重度與訊息 key。掃描既有規格後，最接近的是 spec-validation（只管 validate 的 Purpose 品質與結構驗證）、drift-computation（只管 drift 的五維漂移運算）與 verb-contract（只凍結動詞的輸出形狀，不定義 analyze 的判斷內容）；三者都不涵蓋 analyze 的規則本身。

### Modified Capabilities

（無——verb-contract 對 analyze 的輸出形狀凍結維持不變，本次不改任何欄位。）

## Impact

- Affected specs: 新增 `change-analysis`（archive 後落為 openspec/specs/change-analysis/spec.md）。
- Affected code:
  - Modified: crates/speclink-core/src/analyzer.rs（Consistency 標題拆解與比對、REMOVED 需求的 scenario 跳過與 Reason／Migration 檢查、具體值字元集、英文弱語氣字邊界，以及新增的單元測試模組）
  - New: （無——測試以同檔 `#[cfg(test)]` 模組承載）
  - Removed: （無）
- 影響的 crate：speclink-core。speclink-cli、speclink-server、apps/desktop 只轉發引擎輸出，不需改動；crates/speclink-cli/tests/it/remote_verb_parity.rs 的 analyze 對照用 mock payload，不受規則變更影響。
- 文件：docs/getting-started.zh-TW.md 與 docs/getting-started.md 只以兩條 finding 舉例，不列完整規則，不需改。
