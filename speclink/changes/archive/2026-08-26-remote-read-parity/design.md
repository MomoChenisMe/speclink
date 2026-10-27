## Context

跨四層的讀取管線補齊：wire（speclink-protocol）→ 參考 server（speclink-server routes）→ 桌面 Rust 橋（src-tauri remote）→ 桌面 TS adapter（remoteDataSource）。現況：server 的單 change 讀取回應已組 created、fromDiscussions、deltaCapabilities，且桌面的 remote_status command 把整包 ChangeStatus 序列化交給 TS——資料已經流到前端手上，但 TS adapter 的 changeCapabilities/changeMeta 照舊回 unsupported、RemoteCapabilities 把兩個 capability 寫死 false；討論的 promoted_to 存於 frontmatter、引擎有獨立查詢函式 promoted_to()，但刻意不進 DiscussionInfo（引擎 design D2：discuss list --json 逐位元不變），本地桌面靠 desktop-core 邊緣組裝，remote 的 wire 與 server 都沒接。約束：不動引擎 DiscussionInfo、不偽造 capability 缺口、wire 欄位一律 additive。

## Goals / Non-Goals

**Goals:**

- 遠端分頁的 change 詮釋資料、capability 清單、討論 promotedTo 與本地分頁同形呈現
- 看板清單層的建立者頭像與來源討論標記在遠端可用
- 所有新 wire 欄位 additive：舊 server 不送、舊 client 忽略，雙向相容
- remote-resilience 的 deleteChange 過期句隨刀更正

**Non-Goals:**

- claim 持久化與認領呈現（刀 B）；遠端文件總整理（刀 C）
- CLI 人眼輸出改動；引擎 DiscussionInfo／discuss list --json 的任何改動
- 桌面 in-progress 標記操作入口

## Decisions

**D1：promotedTo 走 server route 邊緣組裝，引擎不動。** 引擎 discuss.rs 明訂 promoted_to 不進 DiscussionInfo（design D2，保 CLI JSON 逐位元不變）並提供獨立查詢函式；本地桌面已是 desktop-core 邊緣組裝的先例。server 的討論列表 route 對每筆討論以同一查詢函式取值後填入 wire 欄位。討論數量級小（單 scope 數十筆內），逐筆讀取可接受，不做批次查詢介面。

**D2：桌面 changeCapabilities/changeMeta 以既有 remote_status payload 映射，不開新 Tauri command。** remote_status 已把 ChangeStatus 全欄位交給 TS；TS adapter 的兩個方法改為呼叫既有 status 路徑並抽取 deltaCapabilities 與 meta 欄位組 ChangeMetaInfo。零新 Rust command、零新 HTTP 請求。

**D3：capability 翻真＋欄位缺席即缺席＝誠實降級。** RemoteCapabilities 的 change_capabilities 與 change_meta 翻真：deltaCapabilities 現行 server 已送；meta 四個歸屬欄位為選填，接舊 server 時缺席，UI 對 optional 欄位既有容錯（不顯示該列），不偽造非空值——符合 remote-workspace-data「不偽造缺口」的紅線，缺的是欄位而非能力。

**D4：ChangeSummary 清單欄位由 server 自 meta 組裝，沿 startedAt 的既有組裝路徑。** GET /changes 清單項現已逐筆讀 meta 組 startedAt；createdBy/created/fromDiscussions 掛同一條組裝路徑，無新讀取成本。

**D5：remote-resilience 過期句以 MODIFIED delta 更正，requirement 與 scenario 名不變。** 僅刪「deleteChange 於 remote SHALL 維持停用」一句並改寫為與 server-verb-api DELETE change（discard 守門）對齊的現況承諾；不新增 scenario。

## Implementation Contract

- **Behavior**：遠端分頁開啟 change 詳情抽屜可見建立者、建立工具、開工時間與開工者（server 有送時）；抽屜的 capability 清單區塊與本地同形；看板卡片顯示建立者頭像與來源討論標記；討論卡依 promotedTo 正確落入「已轉出變更的討論」群組。接 0.1.3 舊 server 時上述新欄位缺席、對應 UI 列不顯示，無錯誤、無偽值。
- **Verification**：crates/speclink-protocol 單元測試斷言新欄位序列化與缺席省略；crates/speclink-server tests/it 讀取面測試斷言單 change 回應四欄位、清單三欄位、討論列表 promotedTo（含未轉出討論省略）；apps/desktop/src-tauri tests/it/remote_data.rs 斷言 RemoteCapabilities 兩位翻真；前端 remoteDataSource.test.ts 斷言 changeMeta/changeCapabilities 映射與 promotedTo 非空映射、remoteCapabilities.test.tsx 斷言 UI 不再呈現「server 尚未提供」停用說明。
- **Scope boundary**：in scope＝上述四層讀取管線與兩處過期註解、四份 spec delta；out of scope＝claim、寫入面、CLI 輸出、引擎結構。

## Risks / Trade-offs

- 討論列表逐筆讀 promoted_to 有 N+1 讀取，接受理由見 D1（量級小、與本地同構）；若未來 scope 討論數上千再議批次化。
- 舊 client 接新 server：serde 對未知欄位預設忽略，protocol 既有 additive 慣例已由 kind 欄位先例驗證。
