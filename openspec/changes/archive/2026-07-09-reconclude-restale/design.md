## Context

link-seal-timing 討論定案三支柱，前兩支柱（link 只鑄鏈、seal 標記已轉出）由 discussion-reflection-seal 交付。第三支柱處理**封印之後、討論側長出的過期**：change seal 後若同一討論被重新結論，change 內容相對新結論過期，但無機制標出。本刀 verify（V1–V7）已把設計壓在真實程式上——以下決策沿其編號。

## Goals

- discuss conclude 作用在**已反映**（promoted_to 非空）的討論時，對其 active promoted_to changes 蓋 per-slug restale_from 旗標。
- 旗標經 CLI（conclude 輸出、show／list／analyze）與 desktop 看板浮現，**零 per-load 掃描**。
- discuss seal 清除該旗標，形成 re-conclude → re-ingest → seal 的閉環。

## Decisions

### D1 觸發鍵綁 promoted_to 非空

conclude 的 status 處理是脆弱字串 replacen（`discuss.rs:609` 只換 `status: open`）。哪天多一個 reopen 流程把 status 打回 open，綁 status 的觸發會靜默漏掉。判準用 `promoted_to` 非空——語意即「曾被反映」。且 `promoted_to` 由 **seal** 寫入（本專案 seal 已接管此職責），故它非空**正好**等價「內容已封印」，觸發語意精準。

### D2 蓋章跳過已歸檔變更

`promoted_to` 讀取連歸檔記錄一起讀（`discuss.rs:554`），實測 `四情境` 的 promoted_to 就含已歸檔的 desktop-shell-and-browser。對已歸檔 change 蓋「待 re-ingest」無意義——其 delta 已套進正典。走訪時對每個 promoted_to 項判存活：僅 openspec/changes/<name>/ 存在者蓋章，archive/ 者跳過。

### D3 restale_from 記錄哪份討論

一張 change 可反映多份討論（實測 web-server-postgres 的 from_discussion 為 `四情境, manual-spec-edit-integrity`）。若 `四情境` re-conclude，裸 stale=true 不告訴 agent 該重折哪份。restale_from 為逗號分隔 slug 清單——re-ingest 精準、清除 per-slug。

### D4 實作鏡像 unlink_discarded

`unlink_discarded`（`discuss.rs:333`）是事件驅動、只動 frontmatter 連結欄、冪等、走訪 promoted_to 的紮實前案。conclude 蓋章是其鏡像：走訪討論 promoted_to、對每張 active change 讀 meta（store.read_change_meta）、於 restale_from 累加本 slug（已含則冪等不改）、寫回（沿 in-progress add 蓋 started_* 的同款 change-meta 寫入路徑）。既有 meta 欄位逐字保留。

### D5 seal 清除 per-slug

seal 標記 promoted 後，自目標 change meta 的 restale_from 移除本 slug（不在則冪等不改）。一張對兩份討論皆過期的 change 需兩份各自 re-seal。清除綁 seal（誠實的「內容落地」動作），非任何 ingest 觸碰。

### D6 meta 欄位機制

ChangeMeta（`model.rs:10`）加 `restale_from: Option<String>` 與 `restale_from()` accessor → Vec<String>，逗號累加器，平行 from_discussion／from_discussions()。缺席＝空。

### D7 CLI 與看板四處浮現

- **conclude 輸出**：蓋章當下 stdout 報告被旗標的 change（`flagged N change(s) for re-ingest: ...`；--json payload 帶被旗標清單）。
- **show／list --json**：吐 restaleFrom（camelCase 陣列）。
- **analyze**：restale_from 非空時出一條資訊性 finding（named change 反映的討論已重新結論、需 re-ingest）。
- **desktop 看板**：卡片讀 restale_from 亮「待重新反映」徽章（apps/desktop/core 看板查詢序列化疊加欄位 → Tauri invoke → tauriDataSource → packages/ui ChangeCard）。

### D8 技能指引

ingest 技能加：目標 change 帶 restale_from 時，re-ingest 折入新結論後執行 seal 清除該 slug。conclude 側蓋章為引擎自動、agent 無需特別動作，故 discuss 技能不改（減少 golden 觸點）。動 assets 後於**乾淨樹**跑 `UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden` 再生並審視 diff。

## Scope

**In scope**：core（conclude 蓋章、seal 清除、restale_from 欄位＋accessor）、CLI（conclude 輸出、show／list／analyze 浮現）、desktop 看板徽章（序列化→bridge→dataSource→UI）、ingest 技能指引＋golden 再生。

**Out of scope**：discuss reopen 機制（V1：add_round／conclude 已足）；per-load 掃描（僅 conclude 事件寫入）；remote 併發語意（remote 未開工）；已歸檔 change 蓋章（D2）；discuss 技能改動（D8）。

## Implementation Contract

- **conclude(store, slug, content)**：寫入結論後，讀討論 promoted_to；非空時對每個 **active** change：讀 meta、restale_from 累加 slug（冪等）、寫回；既有欄位與討論 Context／Rounds／Conclusion 逐字不變（除 Conclusion 段本身）。回報被旗標的 active change 清單供 CLI 輸出。promoted_to 空或全為已歸檔時不寫任何 change meta。
- **seal(store, slug, change)**：既有守衛與 mark_promoted 不變；額外自該 change meta 的 restale_from 移除 slug（冪等）。
- **ChangeMeta::restale_from()**：缺席回空 Vec；逗號值 trim 後分割；平行 from_discussions() 的既有測試形態。
- **驗證目標**：
  - unit（model.rs）：restale_from() 缺席/單值/逗號多值。
  - unit（discuss.rs）：conclude 對 promoted 討論蓋 active change、跳過 archived、冪等；conclude 對 promoted_to 空討論不蓋；seal 清除 per-slug、冪等。
  - CLI 整合：re-conclude 後 show／list --json 見 restaleFrom；analyze 出 finding；conclude stdout 報告；re-ingest+seal 後旗標消失。
  - desktop／ui：KanbanBoard 對帶 restale_from 的 change 渲染徽章（packages/ui kanban 測試）。
  - golden：ingest 技能 diff 僅限 restale 指引段。

## Risks

- conclude 由「純討論寫入」變為「可寫多張 change meta」的跨檔副作用——但 unlink_discarded 已立此模式，且僅 promoted_to 非空時觸發，風險受控。
- 徽章面碰三處內嵌同步點（Rust 序列化／node bridge／TS dataSource／UI），為本刀最大表面；GUI 改動須真實視窗驗證（見 CLAUDE.md 備忘）。
