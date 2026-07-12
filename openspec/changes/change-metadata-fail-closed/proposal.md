## Why

engine-typed-core 已讓 `.speclink.yaml` 與 openspec/config.yaml「存在但損壞」時 fail closed，但每個 change 的 metadata 文件 `.openspec.yaml` 仍是 fail-open：解析失敗時靜默退回全預設值。壞檔會被解讀為「預設 schema、未開工、無來源討論、無 restale 旗標、無看板排序」——查詢以錯誤語意呈現，寫入動詞（開工標記、討論鏈結、看板排序）更會在壞檔上直接疊寫，掩埋損壞並可能覆蓋使用者手上的真相。這是平台架構藍圖 §15.1 P0「壞設定會靜默切換語意或模式」的最後一個未閉合入口（P0 驗收條件 7），也是重構路線圖 §3.6 與 §4.2 順位 2 指名的小刀；TeamStore 契約刀動工前必須先閉合。

目標使用者：透過 AI 代理跑 SDD 的開發者、PO 與 PM——list／status／instructions 等查詢與 apply 期間的生命週期動詞都會讀取 change metadata；桌面看板使用者則依賴「一份壞檔不得讓整個清單或看板無法開啟」。

## What Changes

- change metadata 解析改為 typed：`.openspec.yaml`「存在但解析失敗」回帶檔案位置與原因的錯誤；「檔案不存在」與「欄位缺席」維持既有預設行為（change-lifecycle 的向後相容需求不變）。
- 查詢群：list 繼續列出全部 change，metadata 損壞的 change 標為 invalid 並附診斷，不得讓一份壞檔使整個清單、看板或歸檔清單失效；需要 metadata 語意的單一 change 查詢（status、instructions、validate、analyze、drift、artifact cat）對壞檔 fail closed。
- **BREAKING（刻意的行為變更）**：所有讀寫該 change 的生命週期、metadata 或 artifact 寫入動詞（new artifact、task done／undone、claim、in-progress add、archive、discard、discuss link／seal／promote 對象側、看板排序寫入）遇壞 metadata 一律以 typed error 拒絕並停止，不再以預設語意繼續；錯誤碼沿用封閉註冊表的 invalid_config，不新增錯誤碼。
- 桌面看板的欄內補章（對缺 rank 卡片批次寫入 board_rank）遇 invalid change 必須跳過該卡並回報，不得把壞 metadata 當「缺 rank」對壞檔寫入。
- CLI 人眼與 `--json` 輸出僅在「壞 metadata 情境」新增內容（invalid 標記、診斷與錯誤訊息）；有效 metadata 的 workspace 所有動詞輸出逐位元不變。

## Non-Goals

- 不動 Store trait、revision、CAS 與 Unit of Work（teamstore-contract-v2 刀）。
- 不提供壞檔自動修復或遷移工具——使用者依錯誤訊息手動修正 YAML 即可。
- 不改「檔案不存在→預設」與「欄位缺席→預設」的既有向後相容行為。
- 不改桌面看板視覺設計——invalid 卡片以最小標記呈現，操作被引擎錯誤拒絕即可。
- 不擴增錯誤碼註冊表（封閉集合維持五碼）。
- 不動 workflow config 與 remote config 的既有 fail-closed 行為（engine-typed-core 已交付）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `command-runtime`: 錯誤碼對應表納入「`.openspec.yaml` 存在但解析失敗 → invalid_config」；list 對壞 metadata 的 change 回傳帶診斷的 invalid 標記；需要 metadata 語意的單一 change 動詞跨入口一致 fail closed。
- `change-lifecycle`: 壞 metadata 不得解讀為未開工——開工標記寫入、任務完成蘊含的開工標記、claim、discard 的開工判定與 archive 遇壞 metadata 拒絕（向後相容需求維持：僅約束「存在但損壞」，缺檔與缺欄位不變）。
- `discussion-docs`: link 與 seal 對 change 側 metadata 損壞時拒絕，不得解讀為無來源討論或無 restale 旗標。
- `board-card-order`: 看板排序寫入與欄內補章遇壞 metadata 拒絕或跳過，不得把壞檔當「缺 rank」寫入。

## Impact

- 影響的 crate：`speclink-core`（解析點與查詢／變更群 fail closed）、`speclink-cli`（invalid 診斷渲染與錯誤訊息）、`speclink-fs` 與 desktop core（解析簽名跟進）、`@speclink/ui`（看板卡片最小 invalid 標記）；`speclink-node` 經 runtime 自動繼承錯誤分類，dispatch envelope 形狀不變。
- 相容性影響：有效 metadata 的 workspace 人眼與 `--json` 輸出逐位元不變，parity／color／twin 回歸對照必須全綠；唯一刻意變更是壞 `.openspec.yaml` 由「靜默預設」改為「查詢標 invalid、寫入報錯停止」，既有使用者若有壞檔，依錯誤訊息修正該檔即可。list `--json` 僅於壞檔情境新增選填診斷欄位。
- Affected specs: `command-runtime`、`change-lifecycle`、`discussion-docs`、`board-card-order`（修改）。
- Affected code:
  - Modified: crates/speclink-core/src/model.rs、crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/discuss.rs、crates/speclink-core/src/command/mod.rs、crates/speclink-core/src/teststore.rs、crates/speclink-fs/src/lib.rs、crates/speclink-cli/src/commands.rs、apps/desktop/core/src/cache.rs、apps/desktop/core/src/manage.rs、crates/speclink-node/src/store_bridge.rs、packages/ui/src/adapter.ts、packages/ui/src/components/ChangeCard.tsx
  - New: 無
  - Removed: 無
