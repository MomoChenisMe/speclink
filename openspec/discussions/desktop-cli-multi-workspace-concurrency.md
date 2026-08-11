---
topic: 目前desktop的cli在多個workspace下，是否有併發的問題？
slug: desktop-cli-multi-workspace-concurrency
status: promoted
promoted_to: desktop-async-commands, atomic-file-writes
created: 2026-08-11
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 目前desktop的cli在多個workspace下，是否有併發的問題？

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者問 desktop 的 CLI 在多個 workspace 下有無併發問題。codebase scout 找到 desktop 內嵌引擎（不 spawn CLI）、process 內全域寫鎖（manage.rs D3）、credential family 的跨 process 檔案鎖（refresh.lock）、speclink-fs 無任何鎖、寫檔為普通 std::fs::write——相關檔案充足，走 assumptions 模式。Round 1 確立「跨 workspace 安全、同 workspace 跨 process 多寫者才是縫隙」後，使用者把題目推進到「多寫者怎麼解＋遠端／本地模式差異」。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-11)

**Focus**: 多個 workspace 之間，desktop（內嵌引擎）與 CLI 有沒有併發問題？
**Position**: 跨 workspace 安全；真正的縫隙是「同一 workspace 的跨 process 多寫入者」：
- desktop 不 spawn CLI，引擎直接內嵌（apps/desktop/core/src/lib.rs:33），每次指令以 root 現建 ProjectContext，無跨專案常駐狀態
- desktop 寫入動詞走 process 內全域寫鎖（manage.rs:24，design D3）——跨 workspace 過度序列化但安全
- 歸檔快取 DB 每 workspace 一份（.speclink/desktop-cache.db，cache.rs:33），快取失敗退回目錄直讀
- 機器上唯一跨 workspace 共用的可變狀態＝credential family，已由 user-level 目錄的 refresh.lock（fs2 檔案鎖）跨 process 序列化，desktop 與 CLI 同走 bearer_for（refresh.rs:249-263、remote.rs:374）
- openspec/ 樹上沒有任何跨 process 鎖：desktop 寫鎖只管自己 process，speclink-fs 零鎖，寫檔為普通 std::fs::write（speclink-core/src/util.rs:18-23，非原子）→ 同 workspace 下 desktop＋CLI 或多 agent 平行寫有讀-改-寫競態（遺失更新）與撕裂讀理論風險
- 現狀由工作流程吸收：worktree 隔離（每 change 一 worktree）＋平行 session 操作衛生
**Ruled out**: 「多 workspace 之間會相撞」——唯一共用狀態（credential）已有跨 process 鎖，其餘狀態皆 per-workspace
**Open**: 同 workspace 多寫入者要不要／怎麼上機制解？遠端模式與本地模式的寫入路徑差異為何、各自的併發語意？

### Round 2 — interview (2026-08-11)

**Focus**: 遠端模式與本地模式的寫入路徑差異為何？多寫入者在各自模式下怎麼解？
**Position**: 遠端模式的多寫入者已被架構性解決，縫隙只剩本地模式；本地推薦「跨 process 檔案鎖＋原子寫」組合：
- 遠端模式真相在 server 的 TeamStore：寫入走 UnitOfWork、每個 staged op 帶 CAS 前提（speclink-store/src/uow.rs）；postgres 驅動 transaction＋per-scope advisory lock（store-postgres/src/lib.rs:401-410）；API 層 If-Match ETag 擋 stale 讀、commit CAS 擋請求中併發（server/src/verb.rs:72-133）
- 呼叫端守規矩：desktop 拖排寫回「讀→重算→PUT 帶 If-Match，409 重讀重試一次」（desktop remote.rs:944、1526）；CLI 遠端模式動詞整包送 server 執行（remote_base.rs）
- 本地 .speclink/context/ 是唯讀投影、digest 驗證 fail-closed，不是第二個可寫真相（host/src/projection.rs）
- 本地模式真相是 openspec/ 檔案樹：零跨 process 鎖、std::fs::write 非原子——多寫入者縫隙全在此
- 解法候選：A. per-workspace 檔案鎖（.speclink/ 下 lock file，fs2，寫入動詞讀-改-寫整段持鎖，refresh.lock 已是同 pattern 先例）；B. 原子寫 temp+rename（解撕裂讀，幾乎零成本）；C. 樂觀 CAS 比對讀時 hash（與遠端心智模型對齊，但版本要穿過所有寫入動詞簽名、本地檔案可被編輯器外改）；D. 單一寫入者 daemon；E. 維持現狀
- 推薦 A＋B；seam 落點為單一 with_workspace_write_lock helper（speclink-fs 或 host），desktop 的 in-process WRITE_LOCK 可順勢被其取代
**Ruled out**: D 單一寫入者 daemon——把本地模式變迷你遠端，違反「檔案即真相、可直接編輯」的本地精神，太重
**Open**: 值不值得現在解（實際痛點頻率）？鎖粒度整 workspace 或 per-change？Windows 上 fs2 鎖行為差異的實作風險？

### Round 3 — interview (2026-08-11)

**Focus**: 使用者的實際痛點重新歸因——切換 workspace 後 desktop UI 卡死，agent 同時在跑 CLI，是檔案鎖問題嗎？
**Position**: 不是檔案鎖問題；根因是「讀取型 command 全是同步、跑在 Tauri 主執行緒」＋讀取路徑每次 spawn git，agent 活動只是放大器：
- Tauri 非 async command 在主執行緒執行；專案自己已認知此事——寫入型 command 全走 async＋spawn_blocking，註解明寫「非 async command 會佔用主執行緒凍結整窗」（lib.rs:94-96，design D2）
- 但讀取型與部分動詞 command 仍是同步 fn：list_changes/list_specs/status/document/search_workspace/validate/analyze/archive/archive_carry/discard_review/discard_verify/archived_changes/list_discussions/promote_discussion 等（lib.rs:39-223）
- 讀取不只讀檔：list_changes_at 等每呼叫必 facts_for → observed_facts → spawn `git worktree list --porcelain`（query.rs:21、host/worktree.rs:41）；desktop D1 註解自承 GUI 進程 spawn git「首抓可能秒級」，macOS 另有 Gatekeeper 掃描稅前科
- 切 workspace 時前端一次抓整批清單＋文件，全部排隊主執行緒；agent 同時操作使 repo 忙碌（git 更慢）並觸發 watcher 事件風暴 → 前端再觸發更多同步讀取 → 主執行緒塞爆 → 整窗凍結
- POSIX 下檔案讀取不會被另一 process 的寫入 block；desktop 讀取路徑也不取寫鎖（manage.rs 註解「讀取路徑不取鎖維持快路徑」）——所以「互相鎖死」的假設不成立
- 修法方向：A. 凡觸檔案系統或 spawn git 的 command 一律 async＋spawn_blocking（與寫入側 D2 對齊，lib.rs 薄包裝層機械式逐支改，desktop-core 零改動）；B. 降低讀取成本（facts 快取——但 D4「每次現取不快取」是刻意設計，動它要重新裁決）；C. watcher 事件風暴前端節流
**Ruled out**: 「卡死由檔案鎖／寫寫競態造成」——desktop 與 CLI 間不存在任何共享鎖可互卡，讀取不取鎖、讀不被寫 block；Round 2 的遺失更新是另一個獨立問題，與本卡死無關
**Open**: async 化全部觸檔 command 是否即為本討論主結論？facts 每次現取（D4）要不要加快取或降頻？Round 2 的遺失更新解（檔案鎖＋原子寫）要做還是降級為 deferred？

### Round 4 — interview (2026-08-11)

**Focus**: async 化是否連 tray 的 workspace 切換一併解決？遺失更新解法的必要性裁量？
**Position**: tray 一併解決（同一條 command 路徑＋同一條主執行緒）；遺失更新的鎖建議刻意延後、原子寫建議順手做：
- tray panel 是薄渲染層：不自建 store、不直呼資料查詢指令，動作以 tray-panel-action 事件回流主視窗執行（apps/desktop/src/panel/main.tsx 開頭註解）→ tray 切換 workspace 觸發的就是主視窗那批 command
- 凍結機制是 app 全域唯一的主執行緒：sync command 佔住它時原生事件迴圈整個停擺（tray 圖示點擊、面板開閉、視窗事件都在上面）→ 在 command 層 async 化，一次解掉所有 surface（主視窗＋tray panel＋tray 圖示）
- 界限誠實標注：async 解「卡死」不解「慢」——agent 忙碌時清單仍可能晚到，但 UI 保持可回應（轉圈而非凍結）
- 遺失更新 A（跨 process 檔案鎖）：必要性低——worktree 已結構性分離平行寫入、desktop 寫入是低頻手動動作、撞上也可由 git 回復；沒咬過人之前不值得付鎖粒度與平台差異的複雜度 → 刻意延後
- 原子寫 B：建議做——引擎寫檔匯流在 util::write_file 單點（speclink-core/src/util.rs:18，speclink-fs 也經它），改 temp+rename 成本近零；且撕裂讀×fail-closed 有真實症狀鏈：.speclink.yaml 被讀到寫一半 → init_core_context 視為非專案 → 專案在 UI 上短暫「消失」，此類靈異現象事後極難歸因
**Ruled out**: 「tray 需要另外修」——panel 無獨立資料路徑，不存在第二個修理面
**Open**: 結論的 change 拆法——async 化一個 change；原子寫併入或獨立小 change？

## Conclusion

**Decision**: 拆兩個 change 依序進行：(1) desktop 觸檔案系統／spawn git 的 Tauri command 全面 async＋spawn_blocking——根治切換 workspace 時的 UI 卡死，主視窗與 tray panel 走同一批 command、同一條主執行緒，一次全解；(2) 引擎 util::write_file 改原子寫（temp+rename）——獨立小 change，消滅撕裂讀。跨 process 檔案鎖刻意延後；遠端模式無需動作（CAS＋per-scope advisory lock＋If-Match 已完備）。
**Rationale**: 使用者實際痛點（切 workspace 後 UI 卡死）根因不是檔案鎖，是同步 command 佔用 Tauri 主執行緒＋讀取路徑每次 spawn git；寫入側 design D2 已有 async＋spawn_blocking 樣板與明文理由，讀取側機械式補齊即可，desktop-core 零改動。原子寫防的是撕裂讀×fail-closed 的症狀鏈（.speclink.yaml 讀到半份→專案短暫「消失」、不可重現），單一 choke point 十行左右改完，除錯成本遠超修法成本。
**Rejected alternatives**: 跨 process 檔案鎖現在做——遺失更新沒咬過人、worktree 已結構性分離平行寫入，YAGNI；樂觀 CAS——讀時版本要穿過所有寫入動詞簽名、本地檔案可被編輯器外改；單一寫入者 daemon——違反本地模式「檔案即真相」；facts 快取——D4「每次現取」是刻意設計且 async 已解卡死；watcher 節流——async 後風暴只是省 CPU 問題，非必要。
**Deferred**: 跨 process 檔案鎖（真的發生遺失更新再啟動）；facts 快取／讀取提速（async 後仍嫌慢再議）。
**Capture to**: proposal（兩個 change）
**Next**: /speclink-propose --from-discussion desktop-cli-multi-workspace-concurrency
