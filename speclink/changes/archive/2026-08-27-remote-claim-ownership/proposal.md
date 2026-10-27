## Why

claim 是唯一的 RemoteOnly 動詞，設計目的是多人共用 Remote Store 時「認領一個 change 防止撞工」——但 server 的 claim 端點今天是不落盤的回聲 stub：讀 meta 確認存在後把呼叫者身分回傳一次，什麼都不寫，清單的 claimedBy 恆為空。認領一刷新就蒸發，command-runtime 正典承諾的命令層覆蓋（事件 change-claimed）與 verb-contract 承諾的認領衝突 409 訊息都成了測不到的邊界；桌面遠端看板也完全沒有認領操作與「誰在做」的呈現——對 PM／SA 與多位工程師共看一個看板的團隊情境，這是核心畫面缺口。本 change 依討論 remote-remaining-gaps 的結論（刀 B）補上持久化的認領語意與桌面認領面。

## What Changes

- 引擎 Claim 命令由一律拒絕改為依 store 分流：本地 fs store 維持既有明確拒絕（RemoteOnly 語意零改動）；支援團隊模式的 store 上，未認領的 change 把 claimed_by（ExecutionContext 的 actor）與 claimed_at 寫進 change meta、發布 change-claimed 事件、scope revision 前進；同一身分重複認領為冪等成功；已被他人認領則以 ownership 衝突拒絕並附目前持有人
- change meta 增 claimed_by 與 claimed_at 兩個選填欄位，沿「meta 新欄位向後相容」既有語意：舊版讀者忽略、缺席即未認領；壞 meta 沿 fail-closed 守門拒絕
- server 的 POST /changes/{name}/claim 改經 Command gateway 直通引擎（移除回聲 stub），ownership 衝突映射 HTTP 409、reason 為八值封閉 registry 的 refused、message 含目前持有人與建議動作；變更清單與單 change 讀取回應的 claimedBy 改自 meta 組裝（wire 欄位既有，今日恆缺席）
- CLI 的 claim 認領衝突訊息路徑（verb-contract 既有承諾）隨 server 真的會回 409 而首次可驗證，補整合測試釘住（reason refused 走 typed client 的訊息原樣轉印，零 CLI 改動）
- 桌面遠端認領面：Tauri command 曝露既有的 Rust 橋 claim、RemoteCapabilities 增 claim 位（依 role：editor 以上可用）；詳情抽屜提供認領操作、看板卡片與抽屜呈現認領人；認領撞 409 時呈現持有人與建議動作；reader 呈現停用附繁體中文說明

## Non-Goals

- 認領的釋放與搶佔（takeover／force）動詞——衝突一律 409，持有人消失的解法（release 動詞或 admin 介入）視實際需要另立案
- 桌面顯式的 in-progress 開工標記入口——開工蓋章已由 Command gateway 靜默覆蓋（task done 等首次寫入時蓋 started_at／started_by），不需要獨立按鈕
- wire DTO 新欄位與 error reason registry 擴充——claimedBy 在 ChangeSummary、ChangeStatus 與 ClaimResponse 皆已存在，reason 沿用既有 refused，本刀零 protocol 改動
- CLI 人眼輸出與 argv 面新增——claim 指令介面不變
- 刀 A（remote-read-parity）範圍內的詮釋資料／capability 清單／promotedTo——平行 change，本刀不碰其 delta 條文
- 遠端文件總整理——刀 C

## Capabilities

### New Capabilities

(none) — 規格掃描：command-runtime（claim 已在命令層覆蓋表、事件表載明 change-claimed）、verb-contract（認領被搶佔的 409 訊息已承諾）、change-lifecycle（meta 欄位相容與壞 meta 守門）、server-verb-api（動詞端點面）、remote-workspace-data（remote 操作面與 role 呈現）皆已存在且覆蓋本刀範圍，全為修改、無新 capability。

### Modified Capabilities

- `change-lifecycle`: 新增「認領標記欄位」requirement——claimed_by／claimed_at 進 change meta 的寫入語意、冪等與 ownership 衝突、與既有欄位相容規則的關係
- `server-verb-api`: 新增「claim 端點持久化與 ownership 衝突語意」requirement——經 Command gateway、409 refused 含持有人、清單與單 change 讀取的 claimedBy 自 meta 組裝
- `remote-workspace-data`: 新增「認領操作與認領人呈現」requirement——remote 分頁的認領入口、認領人呈現、409 呈現與 reader 停用

## Impact

- Affected specs: change-lifecycle、server-verb-api、remote-workspace-data
- Affected code:
  - New: (none)
  - Modified（實作）: crates/speclink-core/src/command/mod.rs、crates/speclink-core/src/model.rs、crates/speclink-core/src/store.rs、crates/speclink-core/src/teststore.rs、crates/speclink-host/src/bridge.rs、crates/speclink-server/src/routes.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/src/config.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/App.tsx、apps/desktop/src/session.ts、apps/desktop/src/store.ts、apps/desktop/src/i18n/messages.ts、packages/ui/src/adapter.ts、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/i18n.tsx
  - Modified（測試）: crates/speclink-server/tests/it/verb_api.rs、crates/speclink-server/tests/it/e2e_cli.rs、apps/desktop/src-tauri/tests/it/remote_data.rs、apps/desktop/src-tauri/tests/it/common/mod.rs、apps/desktop/src/__tests__/remoteDataSource.test.ts、apps/desktop/src/__tests__/remoteCapabilities.test.tsx、apps/desktop/src/__tests__/tauriDataSource.test.ts、apps/desktop/src/__tests__/helpers/remoteFixtures.ts、apps/desktop/src/__tests__/store.test.ts、packages/ui/src/__tests__/kanban.test.tsx、packages/ui/src/__tests__/richDrawer.test.tsx
  - 立案時列入但實際未動: crates/speclink-server/src/error.rs（Refused→409 refused 的映射本就存在，零改動即達成）、packages/ui/src/components/KanbanBoard.tsx（認領人標記落在 ChangeCard.tsx）、crates/speclink-cli/tests/（CLI 409 訊息測試落在 crates/speclink-server/tests/it/e2e_cli.rs，能力等價）
  - Removed: (none)
