> **出身**: link-seal-timing 討論結論明列 Deferred 的**第三支柱**（前兩支柱由 discussion-reflection-seal 交付：link 只鑄鏈、seal 標記已轉出）。該討論已封存，故不經 --from-discussion，出身記於此。

## Why

「已轉出」在討論被重新結論後會說謊。一張 change 已 seal（內容折好、討論 promoted、promoted_to 寫上），之後重新討論同一份討論並 re-conclude——新結論一落，change 的內容相對它就過期，但目前沒有任何機制標出。這種 staleness 在**封印之後、從討論側**長出，link/seal 任何時機都在它之前，結構上擋不掉。

現況佐證（本刀 verify 的 V1）：重新討論一份已 promoted 的討論**今天就能跑、零新機器**——add_round 不看 status 直接補輪，conclude 只做 `status: open→concluded` 的字串 replacen（對 promoted 討論命不中），故 `status: promoted` 與 `promoted_to` 原封保留。唯一缺的是「蓋章」那一步。

## What Changes

- **conclude 蓋章**：discuss conclude 作用在 `promoted_to` 非空的討論時，對其中每張 **active** change 的 meta 累加 restale_from（per-slug、冪等、**跳過已歸檔 change**）。
- **seal 清除**：discuss seal 標記 promoted 後，順手清掉目標 change meta 的 restale_from 中該 slug。
- **meta 新欄位** restale_from（逗號清單），ChangeMeta 加 restale_from() accessor，既有欄位保留。
- **CLI 浮現**：show／list --json 吐 restaleFrom（camelCase）；analyze 對 restale_from 非空的 change 出資訊性 finding。
- **看板浮現**：desktop 看板卡片讀 restale_from 亮「待重新反映」徽章（apps/desktop/core 看板查詢序列化疊加欄位、tauriDataSource 透傳、packages/ui ChangeCard）。
- **技能指引**：ingest 技能說明「目標 change 帶 restale_from 時，re-ingest 後再 seal 清除」。

## Non-Goals

- **不重造 reopen 機制**：add_round／conclude 對 promoted 討論已足（V1），本刀不新增 discuss reopen。
- **不做 per-load 掃描**：只在 conclude 事件寫入、其餘處讀既存 meta 欄位（落實「不靠主動檢查增加流程時間」）。
- **不動 remote 模式**：remote 尚未開工，file 模式單人無競態；多人併發語意待 remote 刀。
- **不對已歸檔 change 蓋章**：其 delta 已套進正典、re-ingest 不可能。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `discussion-docs`: conclude 對已反映討論的 active 變更蓋 restale_from、seal 清除該旗標——使「已轉出」在討論被重新結論後不再說謊，補上 link-only 與 seal 之外、討論側變動的缺口。
- `change-lifecycle`: 新增 restale_from meta 欄位與 accessor，並經 show／list --json 與 analyze 觀測。
- `desktop-app`: 看板卡片對 restale_from 非空的變更亮「待重新反映」徽章。

## Impact

- Affected specs: discussion-docs（conclude 蓋章、seal 清除）、change-lifecycle（restale_from 欄位與 CLI 觀測）、desktop-app（看板徽章）。
- Affected code:
  - Modified: crates/speclink-core/src/model.rs（restale_from 欄位與 accessor）
  - Modified: crates/speclink-core/src/discuss.rs（conclude 蓋章、seal 清除）
  - Modified: crates/speclink-core/src/analyzer.rs（restale finding）
  - Modified: crates/speclink-cli/src/commands.rs（show／analyze 浮現）與 crates/speclink-core/src/listing.rs（list --json restaleFrom）
  - Modified: apps/desktop/core/src/query.rs（看板清單序列化疊加 restale_from）與 apps/desktop/src/adapter/tauriDataSource.ts（透傳）
  - Modified: packages/ui/src/adapter.ts（ChangeItem restaleFrom 型別）
  - Modified: packages/ui/src/components/ChangeCard.tsx 與 packages/ui/src/i18n.tsx（徽章渲染與文案）
  - Modified: crates/speclink-core/assets/skills/ingest.md 與 render golden 基準（技能指引同步）
  - 外部依賴: 無
