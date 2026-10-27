## 1. 引擎共通碼提升與驗證站實例（speclink-core）

- [x] 1.1 待前置 change converge-review-remediation-rounds 完成後，依 design「D1 共通碼提升」建立 crates/speclink-core/src/station.rs（工單骨架產生／驗證／解析、structured phase／patch、蓋章原子寫入、指紋計算、失效純函式、archive 守門檢查，站別以常數組參數化），並把 crates/speclink-core/src/review.rs 薄化為委派實例。硬性驗收：change code-review-stage 與 converge-review-remediation-rounds 的全部既有 review／archive／listing／structured-round 測試零修改通過；驗證：`cargo test -p speclink-core` 全綠 <!-- speclink-task:tsk_01KYWEDXSC20XH70TYJQWMVZK3 -->
- [x] 1.2 撰寫驗證站工單測試（規格「驗證工單的建立與追加」「驗證工單的讀取」「放棄驗證」；design「D3 add-round 引擎守門」「D8 共用 frozen scope 與站別 snapshot」）：任務未全完成時 `verify add-round` 拒絕／首輪建檔／追加 append-only／缺 `**Scope**:` 拒絕／Phase 與 Patch 成對／discovery→validation 序列／legacy null／`verify show` 解析 rounds 與 lastRound／`verify discard` 只清除 verify snapshots。檔案 crates/speclink-core/src/verify.rs 的 `#[cfg(test)]`；驗證：`cargo test -p speclink-core verify` 新測試先全紅 <!-- speclink-task:tsk_01KYWEDXSCDYXXAE0P0NHKE5Y6 -->
- [x] 1.3 實作 crates/speclink-core/src/verify.rs 薄實例（常數組＋委派 station.rs，structured round parser 與 cleanup namespace 保持站別化）與 crates/speclink-core/src/model.rs 的 `verified_at`／`verified_by`／`verified_with`／`verified_tasks_total`／`verified_scope` 欄位（全部 serde default，缺席讀作未驗證，既有 .openspec.yaml 可讀）；驗證：1.2 測試全綠且既有 legacy review ticket 測試不變 <!-- speclink-task:tsk_01KYWEDXSC5DV3QD7BGK14T6VG -->
- [x] 1.4 依 design「D2 蓋章守門同構」撰寫並實作蓋章（規格「驗證蓋章守門與蓋章效果」「驗證指紋錨與失效判定」）：守門（全任務完成＋末輪零未解 findings、`--accept` 豁免）；五欄位寫入＋工單刪除同一原子寫入；指紋與失效由 station.rs 共用碼保證與審查站位元級同構（含 CRLF→LF 與路徑正規化）。檔案 crates/speclink-core/src/verify.rs。驗證：`cargo test -p speclink-core` 全綠 <!-- speclink-task:tsk_01KYWEDXSCT4RWGP882Y80TCK9 -->
- [x] 1.5 依 design「D4 archive 雙工單守門」撰寫並實作規格「封存的驗證工單守門與雙工單並存」：偵測 verify.md 預設拒絕三處置；雙工單並存 stderr 並列兩組處置；`--carry-verify` 帶走且可與 `--carry-review` 同時帶；皆無工單行為位元級不變（回歸斷言）。檔案 crates/speclink-core/src/archive.rs。驗證：`cargo test -p speclink-core archive` 全綠 <!-- speclink-task:tsk_01KYWEDXSCSCSKC40BQ4YFZTP0 -->
- [x] 1.6 依規格「CLI 清單輸出的驗證欄位釘住」延伸 parity pin 測試：meta 帶全套 verified 欄位時 `list --json` 項目欄位集合同形。檔案 crates/speclink-core/src/listing.rs。驗證：`cargo test -p speclink-core listing` 全綠 <!-- speclink-task:tsk_01KYWEDXSC36M5SD51VQCD3YQM -->

## 2. CLI 子命令（speclink-cli）

- [x] 2.1 撰寫 CLI／Host 整合測試（規格「驗證 frozen scope 與續輪 snapshot」「驗證動詞的 remote 模式行為」；design「D8 共用 frozen scope 與站別 snapshot」）：`verify scope --json` 的 discovery／validation、old/new hunk ranges、dirty-at-start／overlap／candidate 漂移／snapshot 缺失 fail-closed、hash-pinned selection、local／remote parity 與兩站 cleanup 隔離；並覆蓋 `verify add-round --stdin`（任務未完與 structured sequence）／`show --json`（`phase:string|null`、`patchHash:string|null`、既有欄位與 `lastRound`）／`stamp [--accept]`／`discard`／`archive --carry-verify` 的 exit code 與 stdout/stderr。檔案 crates/speclink-host/src/change_diff.rs、crates/speclink-cli/tests/it/、crates/speclink-remote/tests/it/、crates/speclink-server/tests/it/；驗證：對應新測試先全紅 <!-- speclink-task:tsk_01KYWEDXSCEP92MEW54ADMX0FK -->
- [x] 2.2 復用既有 Host change-diff resolver 實作 verify 站別 adapter與 `.speclink/review-scopes/<change>/verify-snapshots/` namespace，註冊 `verify scope`、`add-round`、`show`、`stamp`、`discard` 與 `archive --carry-verify`，同步 crates/speclink-protocol、crates/speclink-remote、crates/speclink-server 的 structured verify round mapping；不得新增 server Git endpoint，`--no-color` 下無 ANSI。檔案 crates/speclink-host/src/change_diff.rs、crates/speclink-cli/src/、crates/speclink-protocol/src/、crates/speclink-remote/src/、crates/speclink-server/src/；驗證：2.1 測試全綠、review resolver fixtures 零修改通過，並依 audit checklist 檢查 base／candidate／hunk 參數的 fail-closed 邊界 <!-- speclink-task:tsk_01KYWEDXSC039V0530EN256N27 -->

## 3. desktop 協定與 desktop-core

- [x] 3.1 依 design「D5 desktop 同構呈現」撰寫 query 增列測試（規格「變更清單的驗證狀態欄位」「已封存清單的驗證結局欄位」）：active 四態（none／inVerify／verified／verifiedStale）與 archived 三態（none／verified／verifiedNotPassed）fixture；`verifyStatus`／`verifiedAt`／`verifiedBy` camelCase 斷言；兩站狀態獨立判定（審查章＋驗證工單並存的組合）；既有欄位不動。檔案 apps/desktop/core/src/query.rs。驗證：`cargo test -p speclink-desktop-core` 新測試全紅 <!-- speclink-task:tsk_01KYWEDXSCV541KKS0P53K07MC -->
- [x] 3.2 實作 desktop-core 的 verifyStatus 判定（共用 station.rs 失效純函式重算凍結度；封存側讀化石工單不重算）；Tauri command 維持單行委派。檔案 apps/desktop/core/src/query.rs、apps/desktop/src-tauri/src/（委派處）。驗證：3.1 全綠；`npm test -w apps/desktop` 通過 <!-- speclink-task:tsk_01KYWEDXSC31A6E85V7NMQ5BX4 -->

## 4. desktop UI（packages/ui）

- [x] 4.1 撰寫 UI 測試（規格「看板卡片的驗證標示」「詳情抽屜的驗證資訊列」「已封存側的驗證標示」「封存入口三選項擴及驗證工單」）：卡片兩章並排順序固定（審查前、驗證後）與單章情境；抽屜驗證資訊列三態；ArchivedList／ArchivedDrawer 的 verifiedNotPassed；封存入口 inVerify 三選項與雙工單分別處置流程。檔案 packages/ui/src/__tests__/。驗證：`npm test -w packages/ui` 新測試全紅 <!-- speclink-task:tsk_01KYWEDXSCGSDWEPGG72QHRWCJ -->
- [x] 4.2 實作規格「看板卡片的驗證標示」與「詳情抽屜的驗證資訊列」：adapter 型別增列 verifyStatus 等欄位；第二顆行內小章與 tooltip；驗證資訊列與審查資訊列同構並列。verify tone 表引用審查章樣式表同值常數（色=狀態、形=站別，討論 card-drawer-header-colors 裁決），圖示採盾牌系（Shield/ShieldCheck/ShieldAlert/ShieldX）。檔案 packages/ui/src/adapter.ts、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx。驗證：對應測試轉綠 <!-- speclink-task:tsk_01KYWEDXSCMTHRJC0EHE3PS1CN -->
- [x] 4.3 實作規格「已封存側的驗證標示」與「封存入口三選項擴及驗證工單」（含雙工單分別處置後才封存）。檔案 packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/ArchivedDrawer.tsx、封存入口所在元件（沿 change code-review-stage 落地的對話框擴充）。驗證：4.1 測試全綠 <!-- speclink-task:tsk_01KYWEDXSC19W665HCRMJNPT7K -->
- [x] 4.4 增列 i18n 詞條：tw 正典詞（驗證中／已驗證／已驗證·其後有變動／曾驗證未通過）＋en 對應。檔案 packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts。驗證：`npm test -w packages/ui` 與 `npm test -w apps/desktop` 全綠 <!-- speclink-task:tsk_01KYWEDXSCNPBCT4BWQAPN4K05 -->

## 5. skill 與生成文件

- [x] 5.1 先修改 skills.rs 的 claude／codex verify golden expectation使其釘住 design「D6 verify skill 收尾迴圈」與「D8 共用 frozen scope 與站別 snapshot」及規格「驗證技能的工單落地」「驗證續輪只驗收修正」「驗證收尾迴圈」「驗證續輪重大晚發問題的安全退出」：中途只盤點；唯一 discovery 讀全部 artifacts 與 frozen patch；validation 只驗收原 findings／remediation patch；未解原文前饋；2→1 可續、1→1 failed、1→0 stamp、accepted 用 `--accept`；重大晚發問題以 scope changed 退出。檔案 crates/speclink-core/src/skills.rs、crates/speclink-core/tests/golden/；驗證：render／golden 測試先紅再以模板實作轉綠，三維度與分級文字保持不變 <!-- speclink-task:tsk_01KYWEDXSCFGBZGP7AXKPGF70Z -->
- [x] 5.2 乾淨樹再生 golden 並於本 repo 執行 `speclink update` 落地更新後的 verify 技能檔；更新 README.md 與 README.en.md 的驗證站收尾流程與兩站分工表，明示首輪 discovery、續輪 validation、無進展未通過與 review／verify 分工，並於分工表補一句兩站都跑時的蓋章時序慣例：兩站檢查都先以「先不蓋章」離場 → findings 統一修正 → 各自複驗 → 兩章接連蓋（design「D6 verify skill 收尾迴圈」末段；討論 cross-station-staleness 定案）。驗證：`cargo test --workspace` 全綠、golden 對照通過、README 分工表與 discussions code-review-stage／code-review-convergence-boundary／cross-station-staleness 的最新結論一致 <!-- speclink-task:tsk_01KYWEDXSC6SX9DF8FY5M4TNRZ -->

## 6. 系統匣面板站章（apps/desktop）

- [x] 6.1 依 design「D7 系統匣面板站章」撰寫面板站章測試（規格「面板變更列的品質站章」）：兩章並排順序固定（審查前、驗證後）與單章情境；兩站皆 none 時零站章且列組成不變；tooltip 取與卡片同組 i18n 詞條；列上不出現建立者頭像／來源討論標記／restale／metaError 元素；原生選單 changeLabel 輸出維持「名稱＋文字進度條＋n/m」不含站章字元（位元級斷言）。檔案 apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/tray.test.ts。驗證：`npm test -w apps/desktop` 新測試全紅 <!-- speclink-task:tsk_01KZ2T6RKKXETNT4AT2EH2MK7D -->

- [x] 6.2 實作面板變更列站章：於名稱與任務數之間並排渲染審查章與驗證章，圖示／色調／tooltip 復用 packages/ui 匯出的章樣式表（既有 reviewStyle 與 4.2 新增的 verify 對應樣式），不另建對照；原生選單 apps/desktop/src/tray.ts 的 changeLabel 不動。檔案 apps/desktop/src/panel/TrayPanel.tsx。驗證：6.1 測試全綠、`npm test -w apps/desktop` 全綠 <!-- speclink-task:tsk_01KZ2T6RKK1STM4PG6W0VMNNX3 -->

## 7. 端到端驗證

- [x] 7.1 E2E 走查（含規格「驗證動詞的 remote 模式行為」「驗證 frozen scope 與續輪 snapshot」及 design「D8 共用 frozen scope 與站別 snapshot」）：demo change 任務全完成後取得 discovery scope並記錄 structured Round 1；分別走 2→1 可續、1→1 立即 failed 無 stamp、1→0 乾淨 stamp、只剩 accepted 以 `--accept` 蓋章，以及 snapshot 缺失 fail-closed；確認 review／verify snapshot cleanup 互不影響、卡片與系統匣兩章並排、修改 verified scope 後轉「已驗證·其後有變動」、archive 三處置與雙工單並存；remote workspace 以 dev harness 驗證 scope 使用 local resolver、工單走 store 管道且離線為非零 exit。驗證：`npm run test:all` 全綠、`speclink validate verify-station-parity` 通過 <!-- speclink-task:tsk_01KYWEDXSC5JQ9VBEYS2P723HH -->
