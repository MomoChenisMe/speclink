## Why

討論 code-review-stage 定案兩個並行品質站，change code-review-stage 落地審查站；驗證側仍是舊形態——verify 的 findings（尤其 Correctness／Coherence 類：需求實作錯、偏離 design）只活在對話裡，關掉 session 即蒸發，使用者回來可能忘了修，乾淨 subagent 也無工單可接手；驗證結果在看板上不可見。本變更把品質站第二實例接上 verify：`verify.md` 工單＋`verified_*` 章＋驗證標示，與審查站同構、同一套使用者體驗。

目標使用者：透過 AI 代理跑 SDD 的開發者。使用情境：workflow 品質站階段 `apply ⇄ ingest → (review? ∥ verify?) → archive`——verify 檢查隨時可跑（中途＝進度盤點），驗證收尾（工單＋蓋章）發生在任務全數完成後、封存之前。

**前置依賴**：本變更以 change code-review-stage 落地的品質站機制為基礎，並以 change converge-review-remediation-rounds 建立的 Host change-diff resolver、Apply baseline 與 structured round 契約為前置；兩者完成後才實作本變更。

## What Changes

- **引擎第二品質站實例**（speclink-core）：`verify.md` 工單＋章欄位 `verified_at`／`verified_by`／`verified_with`／`verified_tasks_total`（任務錨）／`verified_scope`（內容指紋錨）；共通生命週期自審查站實例提升為站別參數化共用碼——review 的對外行為與 CLI 面零變化（回歸由 change code-review-stage 的既有測試釘住）
- **蓋章守門與失效規則與審查站同一條**：全任務完成＋工單末輪零未解 findings（`--accept` 豁免後者）；失效＝任務狀態偏離蓋章時的全完成，或範圍檔內容指紋不符 → 「已驗證·其後有變動」
- **刻意不對稱（引擎守門）**：`verify add-round` 於任務未全數完成時拒絕——verify 檢查可中途跑，工單語意限定為「成品驗證」，防止盤點輪誤落工單；審查站的執行守門維持在技能層（其技能起點即自檢）
- **CLI 新增 `speclink verify` 子命令家族**（speclink-cli，與 review 家族同形）：
  - `verify scope <change> [--json] [--base <rev>] [--candidate-hash <sha256>] [--include-hunk <id>]...`：復用 Host change-diff resolver 與 Apply baseline，凍結驗證 discovery／validation patch；歧義、snapshot 缺失或 hash 漂移時 fail closed
  - `verify add-round <change> --stdin`：追加一輪（任務未全完成 → 非零 exit code 與原因）
  - `verify show <change> [--json]`：印出工單；每輪帶 `phase`／`patchHash`，`--json` payload 與 structured review show 同構
  - `verify stamp <change> [--accept]`：蓋章＋刪工單，守門同審查站；不過 → 非零 exit code 與原因
  - `verify discard <change>`：刪工單不蓋章
- **archive 守門擴充**（speclink-core）：偵測未結 verify 工單同樣預設拒絕，三處置（stamp／discard／`--carry-verify` 明示帶走）；review 與 verify 工單並存時 stderr 並列兩組處置；皆無工單時行為不變
- **verify skill 更新**（skills.rs 正典化＋golden 再生，claude 與 codex）：檢查段維持 fork——任務全完成時，Round 1 對 frozen change patch 與全部 change artifacts 執行唯一一次 Completeness／Correctness／Coherence discovery；Round 2+ 只驗收上輪未解 findings、修正 patch 與直接回歸，不重新探索未修改區域。只有必修集合嚴格縮小才允許再次修正；第一個無進展輪立即以未通過停止、保留工單且不蓋章。中途盤點仍只作對話報告、不落工單
- **desktop 標示**（packages/ui＋apps/desktop＋協定）：卡片第二顆章（與審查章並排、順序固定）；狀態機 active＝無標示／驗證中／已驗證／已驗證·其後有變動，archived＝已驗證／曾驗證未通過；抽屜驗證資訊列；封存入口三選項提示擴及 verify 工單
- **系統匣面板站章**（apps/desktop，討論 tray-station-badges 定案）：macOS 面板的變更列於名稱與任務數之間並排渲染審查章與驗證章（審前驗後，圖示／色調／tooltip 詞條與看板卡片共用同一組樣式表與 i18n 詞條）；判準為「tray 只收行動訊號」——建立者頭像、來源討論標記、restale 與 metaError 標記不進 tray；原生選單（非 macOS 與 macOS 面板失敗後備）的變更列標籤維持現狀無章
- **i18n 詞條**：tw 正典詞（驗證中／已驗證／已驗證·其後有變動／曾驗證未通過）＋en 對應
- **README／docs**：驗證站收尾流程與兩站分工表更新；分工表補一句兩站都跑時的蓋章時序慣例（兩站檢查先不蓋章 → 統一修正 → 各自複驗 → 接連蓋章，避免後蓋站的修正把先蓋的章轉「其後有變動」——討論 cross-station-staleness 定案，純文件慣例、引擎與規格零變更）

相容性影響：

- `speclink list --json` 不變（parity pin 延伸涵蓋 verified 欄位），回歸對照不破壞
- verify skill 的檢查報告內容與三維度邏輯不動；新增的是報告後的工單落地與收尾迴圈，中途盤點行為不變
- touched v1／v2、commit、archive、drift 與 evidence 的檔案層級契約不變；驗證只復用 converge-review-remediation-rounds 的 Host resolver／Apply baseline，另留站別 remediation snapshots，不建立完整 Apply provenance
- archive 僅在「存在 verify 工單」的新情境改變行為
- metadata 新欄位缺席讀作未驗證，pre-migration change 不需遷移

## Non-Goals

- verify 三維度檢查邏輯本身的變更——檢查什麼、怎麼分級完全不動；本變更只補 frozen scope、持久化、有限續輪、收尾與標示
- 中途盤點輪的工單落地——盤點是對話級輸出
- 兩站合併為單一檢查站（討論 code-review-stage 已否決：混合裁決互相遮蔽）
- server-web console 的凍結度計算（同 change code-review-stage）
- CLI `list --json` 輸出擴充
- 審查站行為的任何變更
- touched schema、逐 edit 攔截、無 Git／跨 Host 重播或完整 `capture-apply-change-provenance`
- 系統匣原生選單的站章——純文字 label 無法承載四態色彩與 tooltip，維持「名稱＋進度條＋任務數」不變
- 看板卡片的其他行內符號（建立者頭像／來源討論標記／restale／metaError）進 tray——閱讀脈絡留在看板；restale 是否進 tray 為討論 tray-station-badges 的明文 Deferred 項

## Capabilities

### New Capabilities

- `verify-station`: 引擎第二品質站實例——verify scope、structured 工單生命週期、章與雙錨失效、archive 守門擴充
- `verify-skill`: verify 技能的工單落地、唯一 discovery、有限 validation 收尾迴圈與蓋章行為（技能既有，此為其行為規格首建）

### Modified Capabilities

- `desktop-app`: 卡片／抽屜驗證標示與封存入口三選項擴及 verify 工單
- `client-protocol`: desktop 協定新增驗證狀態欄位
- `tray-status-menu`: macOS 面板變更列的品質站章（審查章＋驗證章並排；原生選單明文排除）

## Impact

- Affected specs: `verify-station`（新增）、`verify-skill`（新增）、`desktop-app`（修改）、`client-protocol`（修改）、`tray-status-menu`（修改）
- Affected code:
  - New: crates/speclink-core/src/station.rs（站別參數化的共通生命週期，自審查站實例提升）、crates/speclink-core/src/verify.rs（驗證站實例）
  - Modified: crates/speclink-core/src/review.rs（改為薄實例，保留 structured round 行為）、crates/speclink-core/src/model.rs、crates/speclink-core/src/archive.rs、crates/speclink-core/src/skills.rs、crates/speclink-core/src/listing.rs（parity 測試延伸）、crates/speclink-core/tests/golden（再生）、crates/speclink-host/src/change_diff.rs（復用 resolver 並加入 verify 站別 snapshot adapter）、crates/speclink-cli（verify 子命令註冊）、crates/speclink-protocol、crates/speclink-remote、crates/speclink-server（structured verify round parity）、apps/desktop/core/src/query.rs、apps/desktop/src-tauri/src/（委派處）、packages/ui/src/adapter.ts、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/ArchivedDrawer.tsx、packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/panel/TrayPanel.tsx、README.md、README.en.md
  - Removed: 無
