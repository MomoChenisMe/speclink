---
topic: implementation-refactor-roadmap 的層級定位與 G0 交付基線是否先建立
slug: g0-delivery-gate-sequencing
status: promoted
promoted_to: delivery-baseline-and-node-packaging
created: 2026-07-10
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: implementation-refactor-roadmap 的層級定位與 G0 交付基線是否先建立

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者請 GPT-5.6 Sol 對照 docs/platform-architecture.zh-TW.md 分析現況程式碼，產出 docs/implementation-refactor-roadmap.zh-TW.md。GPT 建議：roadmap 不是第二條路線，而是平台架構 Phase 1 的執行展開；G0 是交付 Gate 而非 Phase 0；roadmap §4.1 圖的 1A–1E 五群組應明確標註對應七把 Phase 1 刀。使用者問：下一步是否先建立 G0？

模式：assumptions——scout 找到充分證據（兩份 docs、.github/workflows/ci.yml 與 node-sdk.yml、crates/speclink-node 套件狀態、engine-typed-core/tasks.md）。

相關 change：engine-typed-core（in-progress，0/18 任務）。相關討論：collab-scenario-replan（已轉出，產出 platform-architecture）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-10)

**Focus**: roadmap 的層級定位是否成立，以及下一步是否先建立 G0
**Position**: 層級定位已成立（只差呈現方式），G0 應立即建 change 但不阻塞 engine-typed-core 的純 Rust 段：
- roadmap:5 已寫明衝突時以平台架構為準、platform-architecture:1266 已標 roadmap 為 implementation companion——GPT 的層級建議實質上已是現狀，僅 §4.1 mermaid 五群組（1A–1E）與 §4.2 七把刀的對應需明確標註（1A 含 change-metadata-fail-closed、1D 含 drift-client-server-split）
- G0 缺口逐項實測為真：crates/speclink-node `npm ci` 失敗（lock 根層宣告五個平台 optional deps 但 packages 區段缺條目）；ci.yml 只有 cargo build --release ＋ CLI smoke，無 cargo test 也無 npm tests；node-sdk.yml 第 67、95 行用 npm ci，任何碰 crates/** 的 push 該 workflow 必紅
- engine-typed-core/tasks.md 第 1–4 節為純 Rust 側（cargo test 護欄本機全綠），不需等 G0；但 5.3 明文要求 speclink-node 的 npm run build/test 全綠、6.1 要全量回歸——G0 是第 5、6 節的硬前置
- G0 範圍由 roadmap §5 五條驗收封閉定義，需求清楚可直接 propose；可用 discuss promote --name delivery-baseline-and-node-packaging 從本討論轉出以留追溯鏈
**Open**: G0 執行是否影響本地 CLI 功能；G0 範圍是否保留 React act() warnings 清零；是否從本討論 promote

### Round 2 — assumptions (2026-07-10)

**Focus**: G0 建立並先執行後，會不會影響目前本地 CLI 的功能
**Position**: 不會——G0 五項驗收沒有一項落在 CLI/Engine 產品程式碼上，反而強化 CLI 的回歸保護：
- npm ci 修復只動 crates/speclink-node 的 package.json/package-lock.json 與 napi 平台套件佈局（該目錄無 npm/ 子套件目錄，五個 @speclink/engine-* 平台套件未發佈是 desync 根因）——CLI binary 由 cargo 編譯，完全不經 npm
- root 單指令全測是從零新增：root package.json 的 scripts 目前是空的、workspaces 只含 packages/ui 與 apps/desktop——是加新指令，不是改既有行為
- CI 補 cargo test --workspace 與 npm tests 只改 .github/workflows/*.yml，對本機 CLI 無影響
- React act() warnings 清零只動 packages/ui 與 apps/desktop 的測試檔，與 CLI 無關
- CLI 輸出本身是回歸保護對象；G0 讓 CI 真正開始跑測試，等於替 CLI 輸出契約上保險
**Ruled out**: 「G0 會動到 Engine/CLI 語意」的疑慮——roadmap §4.3 本就以「不改 Engine 語意」為 G0 平行條件，建議寫進 proposal 的 non-goal 並以 git diff --stat 驗收 crates/*/src 零改動
**Open**: 兩個揭露型風險的處理方式——CI 全量測試可能揭露平台性既有紅燈、act() 清零可能揭露元件真 bug（修元件就超出 G0，須另開小刀）；是否從本討論 promote

## Conclusion

**Decision**: 立即建立 G0 change `delivery-baseline-and-node-packaging` 並優先開工；implementation-refactor-roadmap 定位為 platform-architecture Phase 1 的執行展開（非第二條 roadmap）；roadmap §4.1 圖明確標註 1A–1E 與七把 Phase 1 刀的對應（1A 含 change-metadata-fail-closed、1D 含 drift-client-server-split）。
**Rationale**: G0 缺口逐項實測為真（speclink-node `npm ci` 因 lock 缺平台套件條目而失敗、ci.yml 只 build+smoke 不跑測試、node-sdk.yml 用 npm ci 必紅），而 engine-typed-core 第 5 節（Node dispatch）與 6.1（全量回歸）硬依賴這個基線；G0 不碰 crates/*/src 產品碼（packaging、root scripts 從零新增、CI yaml、測試檔），對本地 CLI 零影響且讓 CI 真正開始保護 CLI 輸出契約。
**Rejected alternatives**: G0 完全先行、阻塞 engine-typed-core——不必要的序列化，第 1–4 節純 Rust 側有 cargo test 護欄可先行；roadmap 另立為獨立路線——文件自身（roadmap:5、platform-architecture:1266）已排除。
**Deferred**: 兩個揭露型風險在 G0 執行時個案處置——CI 三平台全量測試揭露的既有紅燈按 bug 修、act() 清零若揭露元件真 bug 則另開小刀，不擴 G0 範圍。
**Capture to**: proposal（promote 產出，non-goal 須明寫「不動 crates/*/src 產品碼、以 git diff --stat 驗收」）＋ docs/implementation-refactor-roadmap.zh-TW.md §4.1 呈現修正
**Next**: speclink discuss promote g0-delivery-gate-sequencing --name delivery-baseline-and-node-packaging，其餘 artifacts 以 /speclink-propose 補齊
