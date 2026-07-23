## Context

Phase 3 七把刀（session 模型、連線與 Keychain、RemoteDataSource、read-api、chooser、policy 編輯、offline/reauth；migration 平行中）各自帶測試，但無一連續走過縫隙。可用基建：src-tauri 既有整合測試模式（in-process speclink-server＋in-memory credential store＋假瀏覽器開啟器）、event manager 的訂閱計數與注入退避、phase2-e2e-chain 的真實 CLI 對真 server 模式與失敗現場慣例、remote-dev-harness 的手動環境。desktop 的 GUI 面無法無頭自動化（既有鐵律：jsdom 測不出真視窗行為）——自動劇本走資料面、GUI 以手動清單對照。

## Goals / Non-Goals

**Goals:**

- §14 Phase 3 第 5 項五情境於單一連續劇本閉合；roadmap §5 Phase 3 gate 六條在劇本中各有對應斷言。
- 失敗可讀：情境名＋現場輸出；CI 必跑。

**Non-Goals:**

- 不動任何產品程式碼——縫隙 bug 修復屬獨立 change；不做 GUI 無頭自動化（真視窗手動清單承擔 UI 面）；不含 migration 情境（平行刀交付後屬其驗收，非 §14 第 5 項正典清單）；不做效能/壓力測試。

## Decisions

### 決策 1：desktop 資料面為劇本主角、CLI 為 RD 分身

自動劇本的「desktop」＝src-tauri 的 remote runtime＋event manager＋session 資料層（與 GUI 之間是已測的薄 TS 接線）；「RD 的 agent」＝真實 CLI binary 於 checkout 資料夾以 remote 模式實走。GUI 面以手動鐵律清單對照五情境——真視窗才測得出的部分不偽裝成自動化。替代案「GUI 驅動自動化」被否：鐵律已載明 jsdom 不可信，真視窗自動化屬另一級基建投資。

### 決策 2：單一連續劇本、兩 server 常駐

一個 test 函式依序走六幕（雙 server 起機＋setup、PM、RD、多 server、多 tab、失聯恢復），前幕產物即後幕前提（PM 建的資料被 RD 引用、多 tab 用 PM/RD 兩 scope）——縫隙驗收的本體。兩個 server 以獨立 tempdir sqlite 起於隨機埠、全程常駐（失聯幕只殺其一，另一 server 的 session 同時斷言不受影響——隔離的最強證據）。事件訂閱邊界遵循既有 locator scope：PM／RD 兩個 scope 各有一條帶 repo binding 的 SSE；同 locator 再開 session 只增加參照計數、不另開流。不同 scope SHALL NOT 共流，因 server 的 resume cursor、ETag 與 outbox 均屬 scope。

### 決策 3：gate 條目逐一錨定

六條 gate 在劇本中各有命名斷言：同一資料層支援 local/remote spec-only/remote+checkout（三形態 session 並存幕）；tab 身分非 root path（locator key 斷言）；capability 停用（PM 幕 role 斷言）；credential 不入持久化 payload（registry 與 tabs 持久化序列化掃描斷言——Keychain 本體屬手動清單）；Polling＋ETag 收斂（失聯幕）；stale 只讀無佇列（失聯幕寫入即拒＋恢復後 server 查無）。

### 決策 4：失敗現場慣例沿 phase2

每幕以情境名前綴斷言訊息；失敗時傾印兩 server 的 stdout/stderr 尾段與各 connection 狀態。CI 以獨立 job 跑劇本（與單元測試分開計時），失敗 artifact 上傳 server 輸出。

## Implementation Contract

- 劇本六幕全綠於本地與 CI；單幕失敗輸出含情境名與現場。
- 多 tab 的 SSE 驗收以 scope 為邊界：PM／RD 各一條流（總數 2），同 RD locator 的第二個 session 不增加連線數；任一 scope 寫入只分發該 locator，不得通知另一 scope 或另一 server。
- 手動鐵律清單（驗收 task；操作前確認使用者未在使用螢幕）：remote-dev-harness 起真 server → 五情境 UI 面走查——PM spec-only 全流程、RD checkout 綁定與 CLI 互通即時反映、兩 server 分頁並存與狀態圖示、多 tab 切換、殺 server／重啟／撤 family 的橫幅與復活；macOS 以 security find-generic-password 確認 Keychain、檢視 localStorage 無 credential。
- 回歸：cargo test --workspace（含新劇本）、npm test -w apps/desktop、cargo build --release -p speclink-desktop 全綠。
