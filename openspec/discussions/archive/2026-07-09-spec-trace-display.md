---
topic: 規格檢視器是否顯示 @trace（來源變更與相關檔案）
slug: spec-trace-display
status: promoted
promoted_to: spec-source-footer
created: 2026-07-09
---

# Discussion: 規格檢視器是否顯示 @trace（來源變更與相關檔案）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

桌面規格檢視器與 Spectra 的差異盤點之一：Spectra 在規格內直出 `<!-- @trace -->` 原始註解（Image #1，一坨未排版的檔案路徑牆），並在每個規格末尾另渲染一棵「相關檔案 (37)」收合檔案樹（Image #3）；Speclink 則整個過濾掉。要決定 trace 該不該顯示、若顯示怎麼設計。

模式：assumptions（掃到 3+ 相關源檔）。

機制盤點：
- `@trace` 是封存時 `crates/speclink-core/src/archive.rs` 注入正典 spec.md 的 HTML 註解，含 source（最後動這條需求的 change）/updated（封存日）/code（該 change 封存當下 git 改動檔案全集）。
- 每個 requirement 各注一塊（`archive.rs` make_block），同一 change 動過的多條需求檔案清單完全重複。
- `code` 由 `crates/speclink-core/src/tasks.rs` git_changed_files 於封存當下擷取——是凍結快照，非即時實作索引。
- 前端 `packages/ui/src/components/Markdown.tsx:25` 的 `skipHtml` 丟棄所有 raw HTML（含此註解），這就是「過濾掉」的來源；不是 trace 專門過濾。
- trace 資料已在 `loadDocument`（SpecList.tsx）回傳的原始 markdown 字串內，前端已握有，渲染不需新 IPC。

相關：無進行中變更觸及此面向；規格頁為唯讀（SpecList.tsx）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: trace 三部分（原始註解 / updated / source / code）各自要不要在規格檢視器顯示
**Position**: 拆三塊分判，不整包處理——
- 原始 `<!-- @trace -->` 註解：維持 `skipHtml` 過濾。Spectra 直出（Image #1）是渲染缺陷非功能。
- `updated` 封存日：不顯示。規格卡頭已有 `relativeDays(modifiedAt)`（SpecList.tsx:62,79），高度重疊。
- `source` 來源變更：值得做——一行輕量 footer，是真正的 SDD 溯源價值，成本極低。
- `code` 檔案樹：凍結快照＋（桌面目前）不可點擊＝裝飾，還把規格內容擠下去。除非能做成「總管顯示／開檔」，否則 defer。
- 核心取捨：`code` 的價值取決於用途是「溯源」（source 一欄即足）還是「導航」（需可點擊前提，未具備）。
**Ruled out**: 直出原始註解（Spectra 缺陷）；顯示 updated（與 mtime 重疊）；現在就做 code 檔案樹（凍結快照＋不可點擊）
**Open**: source footer 放規格卡頭還是文末；code 檔案樹待桌面有無開檔能力再議

## Conclusion

**Decision**: 規格檢視器對 @trace 三部分分別處理——(1) 原始 `<!-- @trace -->` 註解維持 `skipHtml` 過濾不直出；(2) `updated` 不單獨顯示（卡頭 mtime 已涵蓋）；(3) `source` 來源變更以一行輕量 footer 顯示（推薦落地）；(4) `code` 相關檔案樹 defer，直到桌面具備「在檔案總管顯示／開檔」能力。
**Rationale**: trace 的價值取捨在「溯源 vs 導航」。溯源用 source 單欄即足；導航需「檔案可點擊」前提且 code 清單是封存凍結快照會過時，不可點擊時純屬裝飾並擠壓規格內容。
**Rejected alternatives**: 直出原始註解（Spectra Image #1 做法——是渲染缺陷非功能）；顯示 updated（與規格卡頭 mtime 重疊）；現在就渲染 code 檔案樹（凍結快照＋不可點擊＝裝飾）
**Deferred**: code 相關檔案樹（待桌面開檔／總管顯示能力）；source footer 的確切位置（卡頭 vs 文末）留給 propose
**Capture to**: proposal（新變更：規格檢視器加「來源變更」footer）
**Next**: /speclink-propose --from-discussion spec-trace-display
