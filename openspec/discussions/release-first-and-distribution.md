---
topic: 先 release 還是繼續推進後續功能——desktop 與 CLI 的發佈通路設計
slug: release-first-and-distribution
status: promoted
promoted_to: release-signing-and-channels
created: 2026-08-12
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 先 release 還是繼續推進後續功能——desktop 與 CLI 的發佈通路設計

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者盤點本地 CLI／desktop／本地 SDD 功能（含 worktree 中兩個變更）皆已規劃完成，問接下來是繼續推進 implementation-refactor-roadmap.zh-TW 與 platform-architecture.zh-TW 描述的後續工作，還是先 release。

模式：assumptions——證據充足（roadmap 十一把刀的封存紀錄、release 基建、product-status 能力矩陣都在 repo 內可查）。

Codebase 掃描發現：roadmap G0～Phase 3（順位 9 之前）已全數封存（2026-07-11～07-31）；`product-status.zh-TW.md`（查核 2026-07-17）只剩 MCP/Copilot tools 與 SSO/cluster 標 Planned；`.github/workflows/release.yml` 已能產出五形態 artifact（CLI／server 壓縮檔、Docker、桌面安裝檔＋updater manifest），但 repo 零 git tag、從未發過版；進行中變更：manual-marker-placement-lint（1/7）、desktop-refreshing-inflight-set（0/9），皆在 worktree。

相關文件：docs/implementation-refactor-roadmap.zh-TW.md、docs/platform-architecture.zh-TW.md、docs/product-status.zh-TW.md、.github/workflows/release.yml、scripts/release-latest-json.mjs。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-12)

**Focus**: 先 release 還是先推進兩份架構文件描述的後續工作？
**Position**: 先 release——這兩件事不是二選一，發版是後續工作的前置。
- roadmap 的刀組 G0～順位 9 已全數封存，文件基線（§8 的 npm ci 失敗、CI 不跑測試）實測已修復，文件進度落後於現實
- release 基建（release.yml 五形態、updater minisign 簽章、版本閘門）齊備但零 tag——首發管線從未端到端演練，早發早驗證
- platform-architecture §14 Phase 4 第一條即「發布 N-API binary 與 adapter」——沒有版號紀律與 tag 節奏，後續的套件發布無從掛起
- 使用者修正：後續功能方向不預設 MCP／Copilot adapter，要依目前最新實作另行討論（不在本討論內解）
- 發版前先收掉兩個 worktree 變更（manual-marker-placement-lint 修的是已知解析陷阱，帶著出貨不划算），並刷新 product-status 查核日
**Ruled out**: 先做完後續功能再首發——首發管線的除錯會疊在更大的功能面上，且 updater 升級路徑要有已發布版本才能被真實驗證。
**Open**: desktop 在 GitHub 上的快速安裝設計（各平台）；CLI 安裝通路（npm？brew？install script？）；版號（0.1.0 vs 1.0.0）；@speclink/engine 是否上 npm；後續功能方向（另案討論）。

### Round 2 — assumptions (2026-08-12)

**Focus**: macOS 簽章值不值得花、Homebrew 門檻、@speclink/engine 文件缺口的處置。
**Position**: 首發不買 Apple Developer——開源、受眾是開發者，未簽章＋放行文件可接受；通路走安裝腳本＋自有 Homebrew tap（零申請門檻）。
- 簽章 US$99/年對零使用者的新開源專案不划算；受眾（用 CLI/agent 的開發者）能處理 Gatekeeper 放行，且 macOS 15 起放行步驟雖變繁瑣（設定→隱私權與安全性→強制打開）仍可文件化
- 觸發條件明確化：出現安裝摩擦 issue、或想觸及非開發者使用者時再買；管線簽章開關已留好（填 secrets 即啟用，另補公證一小段），晚買不用重工
- 桌面自動更新不受影響：Tauri updater 走自家 minisign 簽章，與 Apple 簽章無關
- Homebrew tap 不用申請任何帳號：就是自己 GitHub 帳號下開一個 `homebrew-tap` repo 放 formula；`brew install <owner>/tap/speclink` 直接可用；homebrew-core（免 tap 前綴）才有知名度門檻與審核，屬之後的事
- 使用者裁定：@speclink/engine 缺口先改文件（標示尚未發布），npm 發布留待 v0.2 與 engine 一起規劃
**Ruled out**: 首發即買 Apple 簽章——錢不是重點，重點是現階段換不到對應價值；首發上 npm——公開契約成本與 engine 發布綁定，不該在首發倉促決定。
**Open**: 結論待使用者確認後落章；發版準備工作（安裝腳本、tap、README 改寫、文件修正、版號 bump）是否 promote 成一個 change。

### Round 3 — assumptions (2026-08-12)

**Focus**: 簽章決策反轉與 Homebrew 作法確認。
**Position**: 使用者裁定 macOS 首發即買 Apple Developer 簽章＋公證、Windows 走 SignPath 開源免費簽章；兩者的申請與設定以逐步教學任務寫進後續 change。
- macOS：受眾體驗優先於省錢——推翻我「首發不簽」的建議；現有管線只差公證環節（notarytool 所需的 APPLE_ID／team 環境變數未接線），change 要補管線＋帳號申請、憑證匯出、secrets 設定的手動教學任務
- Windows：SignPath 對開源專案有免費簽章方案（由 SignPath Foundation 憑證出章）；整合方式與現行 env-gated PFX 路徑不同（走其服務簽章，Tauri 以自訂 signCommand 接），申請未過件的退路是首發未簽章＋SmartScreen 放行說明
- Homebrew：確認自有 tap 就是業界標準起手式，不是變通——HashiCorp、MongoDB 等至今仍以官方 tap 發布；formula 格式與 core 相同，之後達知名度門檻要遷入 core 不用重寫
- 教學任務屬手動步驟，tasks.md 以 [M] 標記緊貼 checkbox（守 manual-marker 慣例）
**Ruled out**: 首發不簽章（使用者裁定推翻）；等 SignPath 過件才發版（過件與否不確定，退路明確即可不阻塞）。
**Open**: 無——所有節點已收斂，進入結論。

## Conclusion

**Decision**: 先 release（v0.1.0），不先推進後續功能。首發套組：GitHub Releases 五形態（已有管線）＋ CLI 安裝腳本（sh／PowerShell）＋ 自有 Homebrew tap ＋ README 三平台桌面下載表。macOS 購買 Apple Developer（US$99/年）做 Developer ID 簽章＋公證（管線補公證接線）；Windows 申請 SignPath 開源免費簽章（Tauri signCommand 整合；未過件退回未簽章＋SmartScreen 放行說明）。兩者的帳號申請、憑證與 secrets 設定，在 change 的 tasks.md 以逐步教學的手動任務（[M]）落地。`@speclink/engine` 文件改標「尚未發布至 npm」。發版前先收掉兩個 worktree 變更（manual-marker-placement-lint、desktop-refreshing-inflight-set）並刷新 product-status 查核日。

**Rationale**: 發版管線齊備但零 tag、從未端到端演練——早發早驗證，且後續的套件發布（原 Phase 4 性質工作）本來就以發版節奏為地基。簽章由使用者裁定首發即納入：使用者體驗優先於省錢，管線簽章開關已預留、增量成本可控。

**Rejected alternatives**: 先做完後續功能再首發（首發管線除錯會疊在更大功能面上，updater 升級路徑也需要已發布版本才能真實驗證）；首發不簽章（使用者裁定推翻——願付費消除安裝摩擦）；首發即上 npm（公開契約成本應與 @speclink/engine 發布一起規劃）；Homebrew 直進 homebrew-core（有知名度門檻與審核，自有 tap 才是標準起手式，之後遷入不用重寫）。

**Deferred**: 後續功能方向——依目前最新實作另案討論，不預設 MCP/Copilot（可用 /speclink-improve 或新 discuss）；npm 通路與 @speclink/engine 發布（v0.2）；homebrew-core／winget／scoop／AUR（有使用量後）；版號策略維持 0.1.0 起跳。

**Capture to**: proposal（promote 成發版準備 change）
**Next**: /speclink-propose --from-discussion release-first-and-distribution
