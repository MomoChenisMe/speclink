## Why

討論結論「Capture to 指向既有變更 → `speclink discuss link` → ingest」的既有流程，在該變更本身出身自討論時斷路：change meta 的 `from_discussion` 為單值，link 對已連結的變更直接拒絕。而出身自討論的變更正是最可能需要再討論的——修正提案方向的新討論 link 不回目標變更，只能永遠掛在看板上或手動封存，討論↔變更的推理鏈也就斷裂。re-discussion 情境揭示：討論與變更的關係本質是多對多，change 側的單值連結是模型缺口。

## What Changes

- change meta 的 `from_discussion` 從單值升級為逗號累積器，鏡射討論側 `promoted_to` 的既有模式：第一個條目保留「出身討論」語意，後續 link 逐一追加；單值為退化情形，既有記錄零遷移。
- `speclink discuss link` 的守衛「變更已連結其他討論 → 拒絕」改為「追加」；同一組合重跑維持冪等成功；其餘守衛（討論不存在、討論已封存、變更不存在）不變。
- 封存共行改為逐 slug 判定：變更封存時，其連結的每份討論各自檢查「是否仍被存活變更引用」，不被引用者隨行封存。單一連結情境的行為與 CLI 輸出逐位元不變。
- 桌面端多值呈現：變更卡「來自討論」徽章以出身討論（第一個條目）為代表，詳情抽屜列出全部來源討論並可互跳；同源變更清單改為「來源討論集合有交集」判定。
- bridge 對 GUI 的 `fromDiscussion` 欄位（string|null）改為陣列形 `fromDiscussions`，GUI 型別與元件同步。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `discussion-docs`: link 動詞的「已連結其他討論」守衛改為累加語意；封存共行改逐 slug 判定。
- `desktop-app`: 來自討論徽章、詳情抽屜來源討論、同源變更清單的多值呈現。

## Impact

- Affected specs: `discussion-docs`（修改）、`desktop-app`（修改）
- Affected code:
  - Modified:
    - `crates/speclink-core/src/model.rs` — ChangeMeta 的 from_discussion 累積器解析存取
    - `crates/speclink-core/src/discuss.rs` — link 追加語意與測試
    - `crates/speclink-core/src/archive.rs` — 逐 slug 共行封存；ArchiveOutcome 承載複數共行結果
    - `crates/speclink-cli/src/commands.rs` — archive 人眼輸出消費新形狀（單一連結情境輸出逐位元不變）
    - `apps/desktop/core/src/verbs.rs` — archive 結果 camelCase 組裝隨新形狀調整
    - `apps/desktop/core/src/query.rs`、`apps/desktop/core/src/manage.rs` — fromDiscussions 陣列欄位
    - `packages/ui/src/adapter.ts` — 型別多值化
    - `packages/ui/src/components/ChangeCard.tsx` — 徽章代表值與多值 title
    - `packages/ui/src/components/RichDetailDrawer.tsx` — 來源討論清單與互跳
    - `packages/ui/src/i18n.tsx` — 多值文案（如需）
    - `apps/desktop/src/App.tsx` — 同源判定改集合交集
  - New: （無）
  - Removed: （無）
- 不受影響的回歸面：CLI `speclink list --json` 不含 fromDiscussion 欄位；`discuss link` 成功訊息形狀不變；內嵌技能資產與 render golden 無需變動（技能文字未文件化單值限制）。
