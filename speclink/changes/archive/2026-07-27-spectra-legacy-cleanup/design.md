## Context

speclink 已不再以 Spectra 為對齊目標，但「對齊中」的措辭散佈在四種載體：README／docs、正典規格、內嵌技能資產（連動 golden）、源碼註解。本 design 定義統一的改寫詞彙表與各載體的執行程序，確保批次改寫零行為變更且不誤傷歷史記錄。

範圍內：措辭改寫、golden 再生、三處技能同步。範圍外：任何 CLI 行為／輸出變更、封存 artifacts、LANGUAGE.md、prompt.md、@trace 清單、常駐守衛測試。

## Goals / Non-Goals

- Goal: 全 repo 非歷史載體零 Spectra 進行式指涉；README 保留歷史參考。
- Goal: 改寫後 cargo test 與 npm test 全綠、golden 僅預期 diff。
- Non-Goal: 不重整註解的其他內容——只動 Spectra 指涉的字句，維持最小 diff。

## Decisions

### 決策一：改寫詞彙表（規格與源碼共用）

| 原措辭 | 改寫 | 適用載體 |
| --- | --- | --- |
| 本需求為 parity 敏感 | 本需求為輸出凍結敏感 | 規格 |
| 對 Spectra 2.3.1 的 parity 基線不變／輸出 parity 不變 | 既有輸出基線不變 | 規格 |
| 本指令（動詞／能力）為 Speclink 自有延伸，不在 Spectra 對照範圍 | 本指令（動詞／能力）為 Speclink 自有延伸 | 規格 |
| 屬對 Spectra 2.3.1 的刻意分歧（Spectra 於壞檔時靜默退回預設） | 為刻意設計 | 規格 |
| (matches Spectra) | (frozen output shape) | 源碼註解 |
| Spectra 式／Spectra 風的 | 既定樣式的（或直接描述樣式本身） | 源碼註解 |
| 與 Spectra 一致／對齊 Spectra 的行為說明 | 描述該輸出形狀為既有契約 | 源碼註解 |

替代方案：逐處自由改寫（無表）——被否，兩人（或兩輪 AI）改出不一致措辭，規格內同概念多詞。取捨：表格覆蓋不了的少數長句（如 archive.md 的對比句）個案處理，於 tasks 點名。

### 決策二：驗證載體改指向真實存在的測試

規格中「parity_suite 31 項、color_suite 16 項、twin harness 8 情境」改為 crates/speclink-cli/tests/ 的整合測試與 speclink-core 的 render_golden 測試——兩者是 repo 內實際存在且 CI 可跑的回歸保護。替代方案：重建 parity suite——被否，speclink 已是自我基線，重建外部對照沒有服務對象。此決策使規格朝「storage 解耦的規格驅動引擎」靠攏：凍結權威是自身已發佈契約，不再綁外部產品。

### 決策三：內嵌技能資產的三處同步與 golden 再生程序

crates/speclink-core/assets/skills/archive.md 是唯一事實來源；.claude/skills/speclink-archive/SKILL.md 與 .agents/skills/speclink-archive/SKILL.md 是渲染實例。程序固定為：改 assets → 在乾淨樹（無未提交變更）執行 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生四份 golden → 審視 diff 僅含預期句 → 以 speclink update 或手動比對同步兩個技能實例。風險：於 dirty 樹再生會把未提交狀態烙進 golden（曾發生），緩解：tasks 明定乾淨樹前置檢查。跨平台風險：CRLF 已由 .gitattributes 強制 LF checkout 根除，Windows 機需已重新 checkout。

### 決策四：豁免清單（不動的路徑）

openspec/changes/archive/、openspec/discussions/、openspec/LANGUAGE.md、prompt.md、正典規格內的 @trace 註解區塊（含 .spectra.yaml 等歷史檔名）。理由：歷史 artifacts 不回改（LANGUAGE 原則）；@trace 是封存時點的事實記錄。grep 驗證步驟以此清單過濾。

### 決策五：README 改寫的錨句

兩版 README 的起源段各保留一句歷史參考（繁中：「設計之初以 [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) 所附 CLI 為行為參考」；英文對應句），回歸保護改述為 golden 與 CLI 整合測試；docs/platform-architecture.zh-TW.md 同語氣。與 user-documentation 規格 delta 的「行為參考起源」措辭一致。替代方案：README 全刪 Spectra——被否（使用者要求保留）。

## Risks / Trade-offs

- 批次改寫誤傷字串常數或測試斷言：緩解——只改註解與文件行，改寫後全套 cargo test ＋ npm test 驗證；含 Spectra 的行逐一人工過目（約 128 處，量可控）。
- golden diff 超出預期句：緩解——審視 diff 為必要 task 步驟，超出即中止並回查 assets 改動。
- 規格 delta 與 canonical 漂移（平行 session 改了同一需求）：緩解——當下無 in-flight 變更；archive 時 speclink 會驗證 delta 可套用。

## Migration Plan

單一變更內完成，無資料遷移。commit 建議分批：規格 delta 與 README／docs 一批、技能資產＋golden 一批、源碼註解 mechanical 一批——回溯時可獨立 revert。

## Open Questions

（無）
