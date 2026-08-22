## Why

使用者會問「某功能怎麼來的／為什麼這樣設計」。答案其實已經散落在現有 metadata 裡：規格 Requirement 的 @trace source 指向動過它的 change、封存 change 的 .openspec.yaml 帶 from_discussion、討論 frontmatter 的 promoted_to 記錄扇出的兄弟 change、.evidence.json 帶逐 task 的觸及檔案——但沒有任何動詞或技能把這條鏈組起來。溯源目前只能靠人肉翻檔案，跨四種 artifact、認得每個欄位的人才做得到。

本 change 把這條鏈變成產品能力：引擎動詞負責機械組裝與篩選（desktop 未來的溯源面板直接受益），技能負責 agent 擅長的考古降級（git 反查、無規格時的 codebase 線索）與最終 live code 驗證，對使用者輸出統一的敘事答案。（結論見來源討論 feature-provenance-skill；分工制、presence check 取代日期分界、一個 change 一次交付均為該討論定案。）

## What Changes

- 引擎新增動詞 `speclink trace <capability> [--json]`：讀正典規格的 @trace source 收斂出動過該 capability 的 change 集合，對每個 change 補上封存目錄位置、from_discussion 來源討論、該討論 promoted_to 的兄弟 change 與各自觸及的 capability，以及 .evidence.json 的逐 task 檔案清單（檔案不存在時該欄位輸出 null，不猜、不讀舊 @trace 的 code 清單）。人讀輸出與 --json 皆提供。
- 新增產品技能 `/speclink-trace <自然語言問題>`（資產 trace.md，經 speclink update 發佈到各 agent 目錄）：把自然語言問題對應到 capability（canon pass），有規格走引擎鏈——讀來源討論的結論與 rounds、proposal 的 Why、evidence 檔案，最後讀 live code 確認現況；evidence 為 null 時靜默改走 git 反查（commit scope 帶 change 名的慣例），查無規格時靜默改走 codebase 考古（git log／blame）。降級是內部管線，使用者看到的永遠是同一種附來源路徑的敘事答案。
- openspec/LANGUAGE.md 新增「溯源」詞條（中文使用者可見詞目前無正典）。

## Non-Goals

- desktop 溯源面板：--json 已為其鋪路，面板本身不在此 change（討論明列 Deferred）。
- server／remote 端的 trace API 與 node-sdk 綁定：v1 僅本地 CLI。
- 回填舊封存的 evidence 或清理受污染的舊 @trace code 清單：presence check 讓缺失走降級，不做資料遷移。
- 引擎內建 git 考古：git 反查與讀碼屬技能層（討論已否決引擎全包）。

## Capabilities

### New Capabilities

- `trace-verb`: 引擎動詞 speclink trace 的行為契約——鏈組裝規則、evidence 缺失的 null 語意、--json 形狀。掃描結果：最近的 change-lifecycle 管 change 的狀態流轉與封存動作、verify-evidence 管 evidence 的寫入端，皆不涵蓋跨 artifact 的溯源讀取組裝。
- `trace-skill`: /speclink-trace 技能的行為契約——問題到 capability 的對應、敘事答案的組成、兩種靜默降級（evidence null 走 git 反查、無規格走 codebase 考古）與 live code 收尾。掃描結果：各技能一 spec 的慣例（discuss-skill、archive-skill 等）下無既有 spec 涵蓋溯源問答。

### Modified Capabilities

(none)

## Impact

- Affected specs: trace-verb（新）、trace-skill（新）
- Affected code:
  - New: crates/speclink-core/src/trace.rs、crates/speclink-cli/src/verbs/trace.rs、crates/speclink-core/assets/skills/trace.md、crates/speclink-cli/tests/it/trace.rs
  - Modified: crates/speclink-core/src/command/mod.rs、crates/speclink-core/src/lib.rs、crates/speclink-core/src/skills.rs、crates/speclink-core/src/store.rs、crates/speclink-fs/src/layout.rs、crates/speclink-cli/src/verbs/mod.rs、crates/speclink-cli/tests/it/main.rs、crates/speclink-core/tests/golden/assets.lock、openspec/LANGUAGE.md
  - Removed: (none)
