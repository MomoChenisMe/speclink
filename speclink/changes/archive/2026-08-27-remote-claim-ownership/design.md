## Context

claim 動詞的三層現況：引擎 Command::Claim 存在但一律拒絕（fs 拒絕文案）；server 端點旁路引擎、以回聲 stub 回應（自述「durable ownership arrives with the auth/admin knife」——那把刀從未到來）；桌面 Rust 橋 RemoteWorkspace::claim 已存在且有整合測試，但無 Tauri command 曝露、無 UI。正典側 command-runtime 已把 claim 列入命令層覆蓋表（事件 change-claimed）、verb-contract 已承諾認領被搶佔的 CLI 訊息——本刀是兌現既有承諾，不是開新架構。約束：RemoteOnly 語意不變（fs 拒絕）、wire struct 零改動（claimedBy 欄位齊備）、與平行刀 A 的 delta 條文零重疊（本刀全 ADDED）。

## Goals / Non-Goals

**Goals:**

- 認領持久化：寫進 change meta、隨 Unit of Work 原子提交、事件與 revision 語意與其他寫入動詞一致
- 衝突可判：他人已認領 → 409 refused 含持有人，CLI 與桌面都有可讀呈現
- 桌面遠端看板長出「認領」操作與「誰在做」呈現
- 三個入口（CLI、server HTTP、桌面）語意一致——全部經同一個引擎命令

**Non-Goals:**

- 釋放／搶佔動詞；桌面顯式開工標記入口；wire struct 改動；刀 A 與刀 C 的範圍

## Decisions

**D1：認領狀態落在 change meta（claimed_by＋claimed_at），不建新儲存面。** 開工章（started_at／started_by）已是 meta 欄位的先例，清單組裝、壞 meta fail-closed 守門、「新欄位向後相容」全部現成；認領與開工同屬生命週期歸屬標記，同居一處。替代方案（獨立 ownership 文件）需要新的文件型別與生命週期規則，為單一欄位不值得。

**D2：引擎 Command::Claim 依 store 能力分流，server 端點改走 Command gateway。** fs store 維持逐字拒絕（RemoteOnly 契約與既有測試不動）；團隊模式 store 上執行持久化語意。server 移除回聲 stub、比照 in-progress 端點直通 gateway——command-runtime「同一動詞各入口同語意」的承諾因此閉合，Node SDK 宿主 Store 走 dispatch 也自動得到同語意。

**D3：衝突語意＝不可搶佔、同人冪等。** 未認領 → 寫章＋事件＋revision 前進；同一身分重複 claim → 冪等成功、零寫入零事件（比照 in-progress 重複蓋章的靜默語意）；他人持有 → 引擎回 ownership 衝突，server 映射 409 refused、message 含持有人與建議動作。不提供 takeover：認領是防撞工的宣告，搶佔語意等真實需求出現再設計（Non-Goal）。持有人消失的死鎖以 discard／archive 既有路徑解（change 消失即認領消失）。

**D4：桌面認領入口放詳情抽屜、認領人呈現在卡片與抽屜。** 抽屜是 change 操作的既有聚集處（archive、discard 同在）；卡片以最小標記呈現認領人（沿建立者頭像的呈現慣例），不另設看板欄位。capability 位 claim 依 handshake role 決定：editor 以上 true、reader false（沿寫入面 role 呈現慣例，停用附繁中說明）。本地分頁不出現認領面（RemoteOnly）。

**D5：409 呈現複用既有錯誤呈現路徑，reason 沿用 refused。** wire 的 error reason 是八值封閉 registry（client-protocol／reference-server 正典），沒有 ownership_lost 這個值——真的送它，typed client 會落進 Unknown 分支印出通用錯誤，持有人資訊反而消失。認領衝突因此與既有「artifact 寫入撞他人持有」同路：引擎回 Refused、server 映射 409 refused、typed client 原樣轉印 message。桌面撞此 409 以 toast／對話框呈現持有人與建議動作，沿 deleteFailed 與 revert 守門的既有分流；CLI 訊息文案 verb-contract 已凍結（含持有人資訊與建議動作），本刀補測試釘住而非新設計。

## Implementation Contract

- **Behavior**：editor 於遠端分頁對未認領 change 按「認領」→ 卡片與抽屜立即顯示認領人，重開 app、換機器、CLI list 皆可見；另一人再認領同 change → 409、畫面呈現「已由〈持有人〉認領」與建議動作；同人重按 → 成功且無變化；reader 看不到可用的認領按鈕；本地分頁無認領面；CLI speclink claim 行為與桌面一致。
- **Verification**：引擎單元測試（未認領寫章＋事件、同人冪等零寫入、他人衝突、fs 拒絕文案不變、壞 meta fail-closed）；server 整合測試（claim 落盤後 GET /changes 與 GET /changes/{name} 的 claimedBy、409 refused 的 reason 與持有人、editor 限定 403）；CLI 整合測試（409 訊息含持有人）；桌面 src-tauri 測試（capability 位依 role）；前端測試（認領操作、認領人呈現、409 呈現、reader 停用）。
- **Scope boundary**：in scope＝引擎 Claim 分流、meta 欄位、server 端點改 gateway＋讀取組裝 claimedBy、CLI 測試、桌面認領面，以及（實作期追加）server 身分字串依 `Actor` 契約組成「顯示名 <email>」——認領的所有權比對需要唯一身分，而 server 過去只塞裸 display，這是既有契約的違反；連帶效果是 server 模式下 created_by／started_by 的形狀一併從裸顯示名變為「顯示名 <email>」，與本機 git 模式對齊。out of scope＝release／takeover、開工標記入口、wire struct、刀 A／C 範圍。

## Risks / Trade-offs

- 與刀 A 共檔（routes.rs、remoteDataSource.ts 等）：立案不衝突（delta 全 ADDED、條文零重疊），實作若平行進行須走 worktree 隔離；建議 A 先落地
- 無 release 動詞的死鎖風險：以 discard／archive 解，並已明列 Non-Goal 與後續立案條件
- 冪等判定以身分字串比對（"Name <email>"，即 `Actor::display()` 的既有契約）：實作時發現 server 端只把裸 display 塞進 ExecutionContext，而 identity 只對 email 做唯一約束——同名的兩個帳號會被讀成同一人，撞工正好從這裡漏掉。修法是讓 server 依契約以「顯示名 <email>」組成身分字串（`auth.rs` 的 `execution_context`），created_by／started_by／claimed_by 三種蓋章一併受惠。殘留限制：同人換顯示名仍會被視為他人，與 started_by 歸屬機制同一限制，不在本刀擴大處理
