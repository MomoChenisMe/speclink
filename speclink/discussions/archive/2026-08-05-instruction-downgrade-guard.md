---
topic: 降級被包裝成更新——指令檔過期偵測分不出檔案比引擎新還是舊，按「更新」會靜默降級
slug: instruction-downgrade-guard
status: promoted
promoted_to: instruction-downgrade-guard
created: 2026-08-05
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 降級被包裝成更新——指令檔過期偵測分不出檔案比引擎新還是舊，按「更新」會靜默降級

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：2026-08-05 實際事故——repo 檔案已再生為 v1.14.0，但安裝中的 desktop app 引擎停在 v1.11.0，橫幅把這個狀態標成「舊版」並提供「更新」主動作；使用者按下後 30 個受管檔被改寫回 v1.11 內容（worktree-merge 技能的 rebase-first 階梯等新內容全數消失）。根因是過期判準只有「不等於」（stale = 檔案版號 ≠ MARKER_VERSION），分不出檔案比引擎新還是舊，降級被包裝成「更新」呈現。

模式：assumptions——掃到 crates/speclink-core/src/init.rs（probe_instructions、marker_version_of、stale 判準）、apps/desktop/core/src/project.rs（probe 序列化）、apps/desktop/src/instructionPrompt.ts（橫幅裁決與略過記憶），足以列假設。

相關變更：desktop-instruction-staleness-prompt（1ae63b9，引入 probe 與橫幅）；記憶中的「安裝版 CLI 過期陷阱」屬同一類事故（舊引擎對新 workspace 動手）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-05)

**Focus**: 修法落點與方向判準——五條假設一次過堂
**Position**: 全數成立，方向偵測落在 speclink-core 的 probe、以版號排序判定，desktop 與 CLI 兩個出口各自守門：
- 落點在 probe（init.rs:832 的 stale: v != MARKER_VERSION 是唯一判準來源），不是 desktop 文案——只修文案的話 CLI 照樣降級
- 方向靠版號排序（v1.14.0 拆段比大小）；解析不了的版號退回現行 stale 行為，不硬排序
- 加新狀態 newer，不改 stale 語意（InstructionStatus 序列化是契約，加變體是擴充；stale 從此只表示「檔案舊於引擎」）
- desktop 檔案較新時換文案為「你的 app 是舊版」、拿掉改寫檔案的主動作——降級動作完全不出現在 UI
- CLI speclink update 同守門：偵測任一檔案比引擎新即拒絕並說明，--force 可越過（rollback 逃生門）
- 使用者證實實際事故：v1.14 按「更新」退到 v1.11（安裝引擎當時停在 v1.11.0），倒退三個 minor，非假設性風險
- 使用者裁定聚合規則：任一工具檔案比引擎新→整體判 newer（涵蓋一新一舊混版與 missing 並存的情況，寧可不給破壞性動作）
**Ruled out**: 只改 desktop 文案（CLI 出口不設防）；為解析不了的版號硬排序（誤判方向比分不出更糟）；保留「更新（降級）」按鈕只換文案（footgun 換標籤）
**Open**: --force 現語意為「覆蓋既有檔案」，降級越過是否共用同一旗標或另立旗標——留給 propose 定細節

### Round 2 — assumptions (2026-08-05)

**Focus**: 如何保證「請 LLM 安裝 desktop」每次都裝到最新——今天 16:43 那顆 v1.11 引擎 app 的成因類
**Position**: 新鮮度靠機械驗證，不靠代理紀律；補一個版號查詢面＋把安裝流程收成帶斷言的 script：
- 事實：CLI 完全沒有 MARKER_VERSION 查詢面——--version 只印 CARGO_PKG_VERSION（0.1.0）＋架構（main.rs:14-28），今天驗新舊只能 grep binary 字串，屬臨時手段
- 失效模式三種：(A) 從舊源碼樹建置（舊 checkout／未拉最新）；(B) 跳過 desktop-sidecar.mjs → src-tauri/binaries/ 殘留的舊 CLI 被 externalBin 靜默打包；(C) 裝完不驗證、憑「剛建的」信任
- 修法一：speclink --version 加印引擎版號（如 0.1.0 (arm64, engine v1.14.0)）——任何 binary 的新舊變成一條指令可問、可斷言
- 修法二：安裝流程收成 repo script（sidecar → vite → tauri build → 斷言 bundle 內 CLI 引擎版號 == 源碼 MARKER_VERSION → 安裝 → 斷言安裝版同版）——LLM 只跑一個入口，新鮮度由建構保證、兩道斷言證明
- GUI binary 的內嵌引擎與 sidecar CLI 分開編譯，但同一次 script 執行同一棵樹，斷言 sidecar 即涵蓋樹狀態
- 主決策的 newer 守門是最後防線：舊 app 就算溜進來，也不再能靜默降級檔案
**Ruled out**: 靠代理記憶或紀律保證新鮮度（今天就是這樣失效的）；grep binary 字串當正式驗證（換個技能內文就失效）
**Open**: 版號查詢面與安裝 script 收進 instruction-downgrade-guard 同一個 change 或另立——傾向同一個（同屬版號可見性與防降級一條故事線），propose 時定

## Conclusion

**Decision**: 兩件事一起解。(1) 方向感：probe 以版號排序分辨「檔案舊於引擎」（維持 stale）與「檔案新於引擎」（新狀態 newer，任一工具檔案較新即整體判 newer）；desktop 於 newer 改示「你的 app 是舊版」且不提供改寫檔案的動作；CLI speclink update 偵測到 newer 即拒絕並說明，留旗標越過。(2) 安裝新鮮度：speclink --version 加印引擎版號（MARKER_VERSION），任何 binary 的新舊一條指令可斷言；desktop 本機安裝流程收成 repo script——建置後斷言 bundle 內 CLI 引擎版號等於源碼 MARKER_VERSION、安裝後再斷言一次，LLM 安裝只跑這個入口。
**Rationale**: 降級與更新是兩件事，判準只有「不等於」分不出方向——已實際發生 v1.14 按「更新」退到 v1.11 的事故（30 個受管檔被舊引擎改寫）。修在 probe（唯一判準來源）一次保護 desktop 與 CLI 兩個出口。而事故的另一半是那顆 v1.11 app 本身：安裝新鮮度過去靠代理紀律與記憶，無查詢面可驗——版號查詢面＋帶斷言的安裝 script 把「裝到最新」從信任變成證明；newer 守門則是舊 app 溜進來時的最後防線。
**Rejected alternatives**: 只改 desktop 文案（CLI 出口不設防，同類事故照發）；為解析不了的版號硬排序（誤判方向比分不出更糟，一律退回現行 stale 行為）；保留「更新（降級）」按鈕只換文案（footgun 換標籤）；靠代理記憶／紀律保證安裝新鮮度（今天就是這樣失效的）；grep binary 字串當正式驗證（換個技能內文就失效）。
**Deferred**: 越過守門的旗標設計（update 現有 --force 語意為「覆蓋既有檔案」，共用或另立，propose 時定）；版號查詢面與安裝 script 是否與方向感同一個 change（傾向同一個，propose 時定）。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion instruction-downgrade-guard
