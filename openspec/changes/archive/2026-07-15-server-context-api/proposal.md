## Why

Client Protocol 的三柱（藍圖 §4.5：Command、Query、Context）只剩 Context 沒接線：protocol 的 ContextSnapshot DTO 與 host 的投影機制（staging、原子切換、digest fail-closed、stale——context-projection 能力）在 Phase 1E 就緒，但 server 沒有任何 context 端點，remote 動詞流程的投影靠一個明標過渡的供應者硬撐——逐 artifact 分次 GET 拼裝（僅 proposal/design/tasks，無 delta specs 也無正典 specs）、跨請求非原子（併發寫入可撕裂「快照」）、snapshot id 是內容拼湊的假識別、無 policy revision。結果是 remote Agent 的投影殘缺：apply/verify 技能指示讀投影，投影裡卻沒有規格正典。roadmap §3.8 的接線清單（snapshot ID、policy revision 與 digest 上線）與架構 §14 Phase 2 第 5 項的全鏈 e2e（含 context）都以本刀為前提。

目標使用者：remote 模式跑 SDD 的 Agent 與開發者（投影完整、快照一致）；Phase 3 Desktop 與後續全鏈 e2e（Context API 有真端點可打）。

## What Changes

- server 新增 context snapshot 端點（project-scoped，沿用 bearer 前置與 binding 裁決）：接受 protocol 的 ContextSnapshotRequest（change 縮小、flow 透傳），回 ContextSnapshot——全部文件讀自同一個 TeamStore snapshot（一致性由 store 契約保證，不逐檔分次讀）、snapshot id 與 scope 狀態記號同源（任何 commit 後必變）、含 workflow config 文件的 revision 作 policy revision（存在時）、逐文件契約 digest。支援 If-None-Match：scope 狀態未變回 304。
- store 契約補齊 LANGUAGE 文件種類（DocumentId 新增 `Language` 變體）：TeamStore 原本無 shared-vocabulary 文件種類（bridge 的 `read_language` 寫死回 None），server 模式因此永遠取不到 LANGUAGE。本刀為 DocumentId 封閉集新增 `Language`，並讓 sqlite 編解碼與 host bridge 的 `read_language` 支援它——context 端點的 change 縮小文件集需要 LANGUAGE 才完備（規格既有要求）。fs 模式的 LANGUAGE.md 讀取不變。
- typed client 新增 context snapshot 方法（handshake 前置與錯誤翻譯沿用既有請求骨架）；remote 動詞流程的過渡供應者汰換為 Context API 供應者——投影從此含正典 specs、delta specs、config 與 LANGUAGE（context-projection 既有佈局需求的完整實現）。
- 縮小分工定案：server 依 change 縮小資料量（指定 change 時回該 change 文件＋其 delta specs＋正典 specs＋config/LANGUAGE；未指定回全量），flow 縮小維持 materializer 職責（§7.3）。
- 投影刷新效率與韌性：manifest 的 snapshot id 與 server 現值相同時 refresh 免重寫（304 路徑）；Context API 失敗維持既有行為——響亮警告、不阻斷動詞、既有投影標 stale。

## Capabilities

### New Capabilities

- `server-context-api`: server 的一致快照端點（單一 store snapshot、snapshot id 同源 scope 狀態、policy revision、digest、304）、change 縮小語意與 typed client 的 context snapshot 方法。

### Modified Capabilities

- `context-projection`: 新增遠端供應需求——remote 投影 SHALL 以 Context API 一致快照為來源（汰換逐 artifact 拼裝的過渡供應者），snapshot 未變時 refresh 免重寫，API 失敗保留既有投影並標 stale。

## Impact

- 相容性影響：wire 為純新增端點與 client 方法；remote 動詞的人眼與 --json 輸出不變（投影是 side effect，contextFiles 的 key 與集合邏輯不變、值仍指投影路徑——投影內容變完整）；本地 fs 模式零變更；parity 31 項、color 16 項、twin 8 情境凍結不動（twin stub 不含 context 端點，既有情境不觸發投影斷言）。apply/verify 技能內容不動——無三處同步風險。
- Affected specs: `server-context-api`（新增）、`context-projection`（修改）
- Affected code:
  - New: crates/speclink-server/src/context.rs、crates/speclink-server/tests/context_api.rs
  - Modified: crates/speclink-store/src/types.rs（DocumentId 新增 Language）、crates/speclink-store-sqlite/src/lib.rs（Language 編解碼）、crates/speclink-host/src/bridge.rs（read_language 讀 Language 文件）、crates/speclink-remote/src/client.rs、crates/speclink-cli/src/remote_commands.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/verb.rs、crates/speclink-host/src/projection.rs、crates/speclink-server/tests/e2e_cli.rs
  - Removed: 無
- 範圍調整（apply 期發現）：原提案假設 server 能提供 LANGUAGE，但 TeamStore 契約缺 shared-vocabulary 文件種類，server 模式下永不可得。經確認採「擴增 store 契約新增 LANGUAGE 種類」，故 Affected code 納入 speclink-store 與 speclink-store-sqlite；本地 fs 模式仍零變更。
