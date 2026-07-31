## Why

封存與刪除的守門在各入口間不一致:引擎批次封存會守任務完成度、單筆封存不守;引擎 discard 動詞會守開工痕跡,desktop 刪除卻直接刪目錄繞過守門、來源討論解鏈與 touched 記錄清理(已轉出討論的 change 被刪後留下懸空的已轉出狀態),desktop remote 刪除更固定帶 force 硬刪。對透過 AI 代理跑 SDD 的開發者(desktop 看板+CLI 並用),這代表一張進行中的卡可以被拖進封存、一個做到一半的 change 可以被一鍵刪掉——生命週期的收檔與反悔語意失守。源自討論 archive-readiness-gating(從 revert-in-progress-to-proposed 的後續疑問展開)。

## What Changes

- **引擎單筆封存補任務完成度守門**(speclink-core):core 封存流程在既有 metadata 檢查旁補上 fail-closed 守門——任務總數大於零且未全完成時拒絕,錯誤列證據(N/M)與出路;豁免沿用既有 --mark-tasks-complete 旗標(先全勾再封存),不新增旗標。守門位於 core 封存函式本體,一次涵蓋 CLI 單筆、desktop 直呼、server 三個入口;批次封存的預過濾(跳過並回報)行為不變。**BREAKING**:單筆 speclink archive 對任務未完成的 change 由成功改為拒絕(非零 exit code)。
- **desktop 刪除改接引擎 discard 動詞**(apps/desktop):本地刪除由直接刪目錄改為走 discard 全語意(force=false)——開工痕跡守門、來源討論解鏈、touched 記錄清理一次到位;remote 刪除由固定 force=true 改為 force=false,與 server 端 DELETE change 的守門語意對齊。desktop 不提供任何 force 通道,--force 豁免僅限 CLI。
- **UI 三表面收斂到階段守門**(packages/ui + apps/desktop):拖曳封存落點僅於拖曳已就緒變更卡時浮現(拖曳非就緒卡=純排序,落點不出現);詳情抽屜的封存鈕於非已就緒、刪除鈕於非提案中時 disabled 並以 tooltip 說明原因(沿用既有 UnavailableAction 呈現模式);看板卡鈕維持現狀(封存僅已就緒、退回僅進行中)。併發情境(可見期間階段已變)由引擎拒絕擋下,走既有失敗 toast。

## Non-Goals

- 不動批次封存的跳過語意與 --mark-tasks-complete 旗標本身。
- 不將 stale delta assumptions(drift)檢查收斂進單筆封存守門——drift 把關留給 verify skill 流程。
- 不移除拖曳封存手勢(spec 已釘住且為最快收檔路徑),也不在卡片上新增任何按鈕。
- 不提供 desktop 的 force 刪除通道;不新增「留紀錄不套 specs 的放棄型封存」動詞(討論列為 Deferred)。
- 不處理 0 任務 change 的封存邊界:引擎守門條件與批次一致(總數為零不擋),desktop 以派生階段收斂即可。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-lifecycle`: 新增單筆封存的任務完成度守門需求(拒絕證據、--mark-tasks-complete 豁免、跨入口涵蓋)。
- `desktop-app`: 拖曳封存落點的浮現條件收斂為僅已就緒變更卡;詳情抽屜封存/刪除鈕的階段守門呈現(disabled + 原因);desktop 刪除改走 discard 全語意與 remote force 語意對齊。
- `board-card-order`: 「跨欄拖曳不改變變更階段」中封存落點行為的表述隨落點浮現條件更新。

## Impact

- Affected specs: `change-lifecycle`、`desktop-app`、`board-card-order`
- Affected code:
  - New:
    - crates/speclink-cli/tests/archive_readiness_gate.rs(單筆封存拒絕與 --mark-tasks-complete 的整合測試)
  - Modified:
    - crates/speclink-core/src/archive.rs(單筆封存任務完成度守門)
    - crates/speclink-core/src/command/mod.rs(守門錯誤分類與 runtime 測試)
    - apps/desktop/core/src/manage.rs(刪除改接 discard)
    - apps/desktop/src/adapter/remoteDataSource.ts(remote 刪除 force=false)
    - packages/ui/src/boardDnd.ts(封存落點浮現條件加階段)
    - packages/ui/src/components/KanbanBoard.tsx(落點與 dragEnd 的就緒守門)
    - packages/ui/src/components/RichDetailDrawer.tsx(封存/刪除鈕階段 disabled + 原因)
    - packages/ui/src/i18n.tsx(守門原因文案)
    - apps/desktop/src-tauri/tests/remote_data.rs(write-through 測試封存前補完任務——守門一體適用 server 通道)
    - apps/desktop/src/\_\_tests\_\_/(App、remoteResilience 與 remoteFixtures 的 fixture 改為就緒／提案中以配合階段守門)
  - Removed: (none)

## 相容性影響

- 單筆 speclink archive:對任務未完成(總數>0 且未全勾)的 change,人眼輸出由成功訊息改為 stderr 拒絕(列 N/M 證據與出路)、exit code 由 0 改非零;--json 同樣拒絕。遷移:完成任務後封存,或明示 --mark-tasks-complete。任務全完成與 0 任務 change 的行為逐位元不變;批次封存(--all/多名)輸出不變。
- desktop 本地刪除:對無開工痕跡的 change 行為不變(多了討論解鏈與 touched 清理的正確收尾);對有開工痕跡的 change 由可刪改為引擎拒絕(UI 已先 disabled,拒絕僅於併發情境以 toast 呈現)。
- desktop remote 刪除:由 force=true 硬刪改為 force=false,已開工 change 的刪除被 server 拒絕——與本地一致。
- 拖曳:非就緒卡拖曳時不再浮現封存落點;就緒卡拖曳與封存確認流程不變。
