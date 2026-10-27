## Why

討論中途轉出（promote）或併入（link）的 change 先完成封存時，連帶封存機制只檢查「是否還有其他在途變更引用」，完全不看討論是否已有結論。尚未寫結論的討論會被掃進 openspec/discussions/archive/，之後 add-round 與 conclude 全被拒絕，只能手動搬檔救援。此行為與 discuss 技能檔「promotion does not close the record」的承諾矛盾，且 desktop 看板把 promoted 討論一律收進「已轉出」收合列，進行中的討論在轉出當下就從上區消失，使用者看不出它還沒有結論。目標使用者是透過 AI 代理跑 SDD 的開發者與 PO／PM：本變更對應 discuss 技能的中途轉出流程與 desktop 看板的討論欄呈現。

## What Changes

引擎與 desktop 兩側對齊同一語意：討論的生命由結論決定，不由轉出決定。

- 連帶封存守門擴充（speclink-core）：change 封存時，來源討論除「無其他在途變更引用」外，還必須「Conclusion 已寫入」（以既有 conclusion_text 判斷內文，不用 status——promoted 討論寫完結論後 status 仍為 promoted）才隨行封存；未有結論的討論留在途，add-round 與 conclude 照常可用。
- conclude 閉環（speclink-core／speclink-cli）：discuss conclude 寫入結論後，回頭檢查 promoted_to 清單中的變更是否全數已封存；是則順手封存討論並於輸出告知，不留無人收尾的孤兒討論。
- 討論資訊增 concluded 欄位（speclink-protocol／speclink-server／speclink-host）：DiscussionInfo 增選填 concluded 布林欄位，由 server route 邊緣與 host bridge 以 conclusion_text 組裝；引擎討論列表結構與 CLI 的 discuss list JSON 輸出維持逐位元不變（沿 promotedTo 前例）。
- 看板討論欄分區改語意（packages/ui／apps/desktop）：promoted 但未有結論的討論留在上區全尺寸卡並帶「已轉出・尚無結論」標示；已有結論的 promoted 討論才收進欄底「已轉出」收合列。tray 面板與系統匣討論列表同步採同一分區判準。
- 技能檔措辭對齊（speclink-core assets，工具 claude 與 codex 的 speclink-discuss 與 speclink-improve 技能）：兩份 asset 內文「最後一個變更封存時自動封存討論」的敘述補上「且結論已寫入」條件。improve 與 discuss 共用同一套討論機制，improve asset 的 fan out 段帶同一句舊承諾；引擎守門與閉環不分討論 kind，improve 討論已被引擎範圍涵蓋，此處僅措辭對齊。兩份 asset 搭同一次 ASSET_VERSION、golden、assets.lock 三連動。

相容性影響：

- CLI discuss list 與 discuss show 的人眼與 --json 輸出逐位元不變（concluded 只在 server／host 邊緣組裝）。
- discuss conclude 僅於「全數轉出變更已封存」時多一行順手封存訊息與 --json 的對應欄位；未觸發時輸出逐位元不變，不破壞回歸對照。
- archive 動詞對「來源討論已有結論」的既有情境輸出逐位元不變；僅「來源討論未有結論」時該討論不再出現於隨行封存清單——此為本變更要修正的行為。
- GET /discussions 增選填 concluded 欄位，舊 client 忽略未知欄位不受影響；新 client 接舊 server 以欄位缺席容錯。
- 不涉及 openspec/config.yaml 與 .speclink.yaml 設定欄位。

## Non-Goals

- 不提供 unarchive 救援動詞：守門修正後不再需要（討論已於方向 C 否決）。
- 不以討論未結論阻擋 change 的封存（方向 B 已否決：change 完工不該被在途討論扣住）。
- 不改動已封存頁的討論節呈現：歷史封存討論即使缺結論也僅唯讀陳列。
- 不處理「已轉出・尚無結論」標示的最終視覺樣式細節——文案與配色於 design 定案，樣式微調不回寫 spec。
- 不回溯修理既有被誤掃進 archive 的討論記錄（如有，使用者手動搬回即可，本變更不做遷移）。

## Capabilities

### New Capabilities

（無——六個相關承諾皆已有既有 capability 承載，掃描結果見下）

### Modified Capabilities

- `discussion-docs`: 「討論以 link 動詞併入既有變更」內的連帶封存承諾加結論守門；新增 conclude 閉環需求（全數轉出變更已封存時順手封存討論）。
- `client-protocol`: DiscussionInfo 增選填 concluded 欄位的序列化與缺席容錯（沿 promotedTo 需求前例）。
- `server-verb-api`: GET /discussions 於 route 邊緣組裝 concluded；討論結論端點回填順手封存事實。
- `remote-workspace-data`: 討論清單的 concluded 與 promotedTo 同律——映射 wire 欄位、不以 client 端固定值補齊。
- `desktop-app`: 看板討論欄的分區判準由「status 為 promoted」改為「promoted 且已有結論」才收進收合列；未結論 promoted 卡留上區並帶標示。
- `tray-status-menu`: 討論列表「已轉出」分區判準同步改為「promoted 且已有結論」。

規格掃描軌跡：連帶封存承諾位於 discussion-docs；DiscussionInfo 形狀與 promotedTo 前例位於 client-protocol；GET /discussions 邊緣組裝位於 server-verb-api；remote 映射紀律位於 remote-workspace-data；看板討論欄與 tray 討論列表分別位於 desktop-app 與 tray-status-menu。無「討論生命週期守門」的獨立 capability，故全數以修改既有 capability 承載，不新增。

## Impact

- Affected specs: discussion-docs、client-protocol、server-verb-api、remote-workspace-data、desktop-app、tray-status-menu
- Affected code:
  - Modified:
    - crates/speclink-core/src/archive.rs（連帶封存守門）
    - crates/speclink-core/src/discuss.rs（conclude 閉環）
    - crates/speclink-cli/src/verbs/discuss.rs（conclude 輸出）
    - crates/speclink-protocol/src/query.rs（DiscussionInfo 增欄位）
    - crates/speclink-server/src/routes.rs（route 邊緣組裝 concluded 與結論端點回填）
    - crates/speclink-host/src/bridge.rs（本地橋接組裝 concluded）
    - packages/ui/src/adapter.ts（DiscussionItem 增欄位）
    - packages/ui/src/components/DiscussionColumn.tsx（分區判準與標示）
    - apps/desktop/src/panel/TrayPanel.tsx（面板分區判準）
    - apps/desktop/src/adapter/tauriDataSource.ts（欄位映射）
    - apps/desktop/src/adapter/remoteDataSource.ts（欄位映射）
    - apps/desktop/src/tray.ts（系統匣討論分區判準）
    - apps/desktop/core/src/query.rs（桌面本地討論清單查詢組裝 concluded）
    - apps/desktop/src/i18n/messages.ts（「已轉出・尚無結論」文案）
    - discuss 與 improve 技能 asset 內文（crates/speclink-core/assets/skills/discuss.md、improve.md）與其 ASSET_VERSION、golden、assets.lock 連動檔（speclink-core crate 內）
  - New: （無）
  - Removed: （無）
