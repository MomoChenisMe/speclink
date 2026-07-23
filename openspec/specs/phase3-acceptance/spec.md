# phase3-acceptance Specification

## Purpose

TBD - created by archiving change 'phase3-e2e'. Update Purpose after archive.

## Requirements

### Requirement: 五情境單一連續劇本

Phase 3 驗收 SHALL 以單一連續劇本走過五情境——PM 無 checkout（登入、handshake、清單/文件讀取、任務寫入、capability 停用）、RD 有 checkout（marker 互通、真實 CLI 於 checkout 以 remote 模式寫入、desktop 資料面經事件數秒內反映）、多 server（雙 server 常駐、憑證逐 origin 隔離、事件互不串流）、多 tab（同 server 的兩 scope 各維持一條 scope SSE，同 locator 的多個 sessions 共用該流，失效按 locator 分發且 SHALL NOT 跨 scope）、失聯恢復（殺 server 的 offline 與寫入即拒、重啟自動收斂含期間變更、撤 device family 的重新認證原地復活）；前幕產物 SHALL 為後幕前提，SHALL NOT 各幕獨立播種。desktop 面 SHALL 以 src-tauri 資料層實走、RD 面 SHALL 以真實 CLI binary 實走。

#### Scenario: RD 寫入即時反映至 PM 資料面

- **WHEN** 劇本中 CLI 於 checkout 資料夾完成 task done
- **THEN** PM 的 session 資料面在數秒內經失效提示重查得到該任務的完成狀態

#### Scenario: 失聯幕不波及另一 server

- **WHEN** 劇本殺掉第一個 server 而第二個 server 存活
- **THEN** 第一 server 的 sessions 進入 offline 且寫入被拒，第二 server 的 sessions 讀寫全程正常

#### Scenario: 多 tab 依 locator scope 共用與隔離事件流

- **WHEN** 同一 server 的 PM／RD 兩 scope 各開一個 session，並為 RD locator 再註冊第二個 session
- **THEN** EventManager 總共只建立兩條 SSE（每 scope 一條），RD 的第二個 session SHALL NOT 增加連線數，且任一 scope 的寫入只失效其 locator


<!-- @trace
source: phase3-e2e
updated: 2026-07-23
code:
  - .github/workflows/ci.yml
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
-->

---
### Requirement: gate 條目逐一錨定於劇本斷言

roadmap Phase 3 gate 六條 SHALL 於劇本中各有命名斷言：三形態 session 並存、分頁身分經 locator key、capability 驅動停用、credential 不出現於任何持久化 payload、Polling 加 ETag 收斂、stale 只讀且恢復後 server 查無離線寫入。任一 gate 斷言缺席即劇本不完整。

#### Scenario: 持久化無 credential

- **WHEN** 劇本完成登入與多分頁建立後掃描 connection registry 與分頁持久化的序列化內容
- **THEN** 其中不含任何 token、PAT 或 refresh credential 內容


<!-- @trace
source: phase3-e2e
updated: 2026-07-23
code:
  - .github/workflows/ci.yml
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
-->

---
### Requirement: 失敗現場可讀且 CI 必跑

劇本任一步失敗 SHALL 報出情境名並傾印兩 server 的輸出尾段與各 connection 狀態；劇本 SHALL 為 CI 必跑路徑（獨立 job、失敗上傳 server 輸出）。

#### Scenario: 失敗輸出含情境名

- **WHEN** 劇本於多 tab 幕的斷言失敗
- **THEN** 測試輸出以該情境名開頭並含兩 server 輸出尾段與 connection 狀態傾印


<!-- @trace
source: phase3-e2e
updated: 2026-07-23
code:
  - .github/workflows/ci.yml
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
-->

---
### Requirement: GUI 面以手動鐵律清單對照

五情境的 UI 面 SHALL 以真實視窗手動清單完成對照走查（remote-dev-harness 環境）：PM 全流程、RD 綁定與即時反映、雙 server 分頁與狀態圖示、多 tab 切換、失聯與重新認證的橫幅與復活；並含 OS Keychain 實體檢查與 localStorage 無 credential 檢視。清單完成 SHALL 為本能力驗收條件之一，SHALL NOT 以 jsdom 測試替代。

#### Scenario: 手動清單完成即驗收

- **WHEN** 自動劇本全綠且手動清單五情境走查完成無異常
- **THEN** Phase 3 驗收面閉合

<!-- @trace
source: phase3-e2e
updated: 2026-07-23
code:
  - .github/workflows/ci.yml
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
-->