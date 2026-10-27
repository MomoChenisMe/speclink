## Why

遠端模式的讀取面比 server 實際能給的窄一截：server 的單 change 讀取回應早已組好 capability 清單與部分 meta（見 server-verb-api「單 change 讀取回應攜帶 show 組合欄位」正典），桌面 remote 分頁卻仍把 change 詮釋資料與 capability 清單標為「server 尚未提供」而停用；討論的 promotedTo 在引擎 frontmatter 有存、有獨立查詢函式，wire 卻不帶，桌面以空清單補——遠端看板因此分不出「已轉出變更的討論」、詳情面板缺建立者與開工歸屬、看板卡片缺建立者頭像。這些縫全是「資料在、管線沒接通」：晚接一天，遠端團隊看板就少一天完整資訊。本 change 依討論 remote-remaining-gaps 的結論（含 subagent 反證驗證輪）補齊讀取對等。

## What Changes

- wire 增欄位（皆選填、缺席容錯，舊 server 不送、舊 client 忽略）：ChangeStatus 增 createdBy、createdWith、startedAt、startedBy 四個 meta 歸屬欄位；ChangeSummary 增 createdBy、created、fromDiscussions 三個清單欄位；DiscussionInfo 增 promotedTo（空清單省略）
- server 讀取面組裝：單 change 讀取回應自既有 parsed meta 補上四個歸屬欄位；GET /changes 清單項補建立者與來源討論欄位；討論列表於 route 邊緣以引擎既有的 promoted_to 查詢函式組裝 promotedTo——引擎 DiscussionInfo 結構不動，遵守其 design D2（discuss list --json 逐位元不變）
- 桌面 remote 翻開兩個 capability：change_capabilities 與 change_meta 由寫死 false 翻真；TS 端 changeCapabilities 與 changeMeta 改以 remote_status 已送達的 ChangeStatus payload 映射實作，移除 unsupported 拒絕；promotedTo 改映射 wire 新欄位、移除空清單補丁
- 過期註解修正：桌面 remote 橋接「ChangeStatus/ChangeSummary 皆不帶 metadata 與 capability 名清單」與 App 內「無 server 來源」兩處註解已被現況推翻，隨實作更正
- 規格過期句修正：remote-resilience「remote 破壞性操作確認一致」中「deleteChange 於 remote SHALL 維持停用」已被 2026-07-31 archive-readiness-gating 的實作推翻（remote 刪除走 discard 守門語意），本 change 以 delta 更正該句

## Non-Goals

- server 的 durable claim 語意與桌面認領／開工歸屬操作面——後續獨立 change（討論結論的刀 B），本刀不動 claim
- 桌面遠端的 in-progress 標記操作入口——桌面本地也沒有，留待刀 B 設計時決定
- 遠端文件總整理——後續獨立 change（討論結論的刀 C）
- CLI 人眼輸出與動詞 argv 面零改動：remote show 是否消費新欄位不在本刀
- 離線佇列或任何「先存後送」語意（remote-resilience 既有紅線）
- 封存討論 slug 重用時 promoted_to 查詢的歧義——本地與 remote 同病的既有邊界，不在本刀
- 引擎 DiscussionInfo 結構與 discuss list --json 輸出——design D2 明訂逐位元不變，本刀不碰

## Capabilities

### New Capabilities

(none) — 規格掃描：client-protocol（wire contract 唯一定義、既有欄位增補 requirement 群）、server-verb-api（單 change 讀取組合欄位、討論端點面）、remote-workspace-data（capability 驅動停用且不偽造缺口）、remote-resilience（remote 破壞性操作確認）皆已存在且直接覆蓋本刀範圍，全為修改、無新 capability。

### Modified Capabilities

- `client-protocol`: 增三條欄位 requirement——變更清單的建立者與來源討論欄位、單 change 讀取回應的 meta 歸屬欄位、討論資訊 payload 的 promotedTo 欄位
- `server-verb-api`: 「單 change 讀取回應攜帶 show 組合欄位」擴充四個歸屬欄位；新增變更清單欄位組裝與討論列表 promotedTo 邊緣組裝的端點承諾
- `remote-workspace-data`: 「capability 驅動停用且不偽造缺口」把 change 詮釋資料與 capability 清單自「停用」面移入「直達」面，並定義舊 server 下欄位缺席即缺席的誠實降級語意
- `remote-resilience`: 「remote 破壞性操作確認一致」更正 deleteChange 過期句，對齊已落地的 discard 守門語意

## Impact

- Affected specs: client-protocol、server-verb-api、remote-workspace-data、remote-resilience
- Affected code:
  - New: (none)
  - Modified: crates/speclink-protocol/src/query.rs、crates/speclink-host/src/bridge.rs（promoted_to 的 route 邊緣組裝點）、crates/speclink-server/src/routes.rs、crates/speclink-server/tests/it/（讀取面整合測試）、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src-tauri/tests/it/remote_data.rs、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/App.tsx、apps/desktop/src/__tests__/remoteDataSource.test.ts、apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - Removed: (none)
