---
topic: 多人協作情境下的 speclink 架構重新規劃（PM/PO 建規格、RD 實作、規格檢視應用）
slug: collab-scenario-replan
status: concluded
created: 2026-07-10
---

# Discussion: 多人協作情境下的 speclink 架構重新規劃（PM/PO 建規格、RD 實作、規格檢視應用）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者對多人協作規劃現況「總覺得有點亂」，以三個角色情境重述願景：(1) PM/PO 在外部 agent 工具（Codex app／Claude Desktop 等）跑 discuss+propose，RD 於本地 repo 以 claude code／codex cli 跑 apply+verify，第三方應用或 speclink desktop 檢視規格；(2) PM/PO 在可看規格的應用上以其內嵌 agent 跑 discuss+propose，RD 同上；(3) 全員本地 git repo＋agent 工具跑完整流程。要求重新思考與規劃。

模式：assumptions——相關素材遠超 3 處：三份來源討論（四情境預設-gui-工具矩陣、sdk-storage-seam-and-remote-desktop、server-auth-and-push-transport）、四個 0 進度變更提案（speclink-sdk-and-store-seam ①、web-server-postgres ②、desktop-remote-mode ③、web-agent-channel ④）、正典 specs（verb-contract、remote-connection、remote-auth、node-sdk、store-abstraction）。

關鍵事實：四變更全部 0 任務完成（重排僅動 artifacts、無程式碼可退）；desktop 無任何 ACP agent 程式碼（舊刀 desktop-acp-agent 從未建立）；codebase 無 MCP 蹤跡；speclink discard 動詞已存在。外部參照：使用者指定 OpenSpec stores-beta user-guide 的 store 概念為方向。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-10)

**Focus**: 「亂」的病灶診斷與目標分層——舊四情境殘留 vs 新三情境的落差
**Position**: ①②③ 為三情境共同地基原封不動；亂集中在舊情境殘塊與兩個缺口。
- 病灶四點：舊情境 2（hosted agent 跑全流程含 apply）於新三情境中消失；web-agent-channel 疊在三刀重切後已砍掉的「web 應用」上（0 任務、依賴 stale）；新情境 1 的 PM/PO 面（Claude Desktop／Codex app 等外部 agent host）無交付物覆蓋（codebase 無 MCP）；新情境 2 的內嵌 agent 不存在（desktop-acp-agent 從未建立、desktop 無 ACP 碼）。
- 目標分層「一條脊椎、可插拔邊緣」：agent 面永遠外部（技能／MCP／ACP 三接法）；檢視面＝desktop 雙模＋第三方經 REST/SDK；脊椎＝動詞契約＋引擎＋兩種文件家（git repo｜team server）。
- 假設新刀序 ①→②→③→④' server-mcp（取代 web-agent-channel）→⑤' desktop-acp-agent。
- 使用者回應以架構重述定錨：儲存媒介三情境皆可替換（pg／本地 fs／網路磁碟）、SDK 引擎負責規格流程、使用者自實作服務串接儲存與引擎（不自實作則用開箱簡易服務）、介面自建或用 speclink desktop——Store 縫為頭牌、② 為便利品、desktop 為預設 GUI；指定參照 OpenSpec stores-beta 的 store 概念。
**Ruled out**: 重切或重寫 ①②③（三提案無服務已死情境的內容，重寫近 60 任務 artifacts 零收益）；speclink 自營 web GUI 與 web-role-views（使用者的檢視面敘述僅列第三方應用或 speclink desktop）。
**Open**: desktop↔自實作服務的推播選 SSE 還是 WebSocket（使用者點名）；④' MCP／⑤' desktop-ACP 假設待裁決；舊情境 2 確死或僅暫緩；情境 1/2 規格真相留 server 不回 git 是否可接受。

### Round 2 — assumptions (2026-07-10)

**Focus**: OpenSpec store 概念對照、堆疊確認、推播傳輸與開箱儲存兩項裁決
**Position**: 骨架零改動（使用者堆疊＝①②③ 現形，createEngine 已受 fs 選項或自實作 Store），三增量中兩項當輪定案。
- OpenSpec store（獨立規劃 git repo＋機器註冊＋config 指標；無伺服器、無同步，git 即傳輸）≠ 儲存縫；其三概念各有對應物：儲存可換＝① Store 介面、外部規格家指標＝.speclink.yaml remote 區段、跨 repo 規劃＝② repos 註冊表。
- Δ1 酬載入契約（前提）：現況 events:{url,transport} 僅供發現、酬載明文在契約外（① design D3）——自實作服務升頭牌後 desktop 看不懂其事件、永遠只有輪詢；契約補最小 invalidate 事件 schema（傳輸中立）→ 併入 ①。
- Δ2 裁決＝雙傳輸：③ 同時實作 SSE＋WS client、宣告欄選路、輪詢兜底；② 仍只做 SSE 並宣告 "sse"。明示推翻 server-auth-and-push-transport「WS client 遞延」裁定——自實作服務升頭牌後成本不對稱翻轉（desktop 做一次 vs 每個 WS 生態整合者改造）→ 併入 ③。
- Δ3 裁決＝pg＋fs 雙選項：② 開箱服務可指向目錄（本地/NAS/網路磁碟）零 postgres 起跑；實工為 fs 後端 version/If-Match（內容雜湊＋服務進程內序列化寫入）；目錄為 git checkout 時規格自帶 git 歷史（回應「規格真相回 git」開放問題）→ 併入 ②。
**Ruled out**: 引入 OpenSpec 式 store registry／workset（「無伺服器又要規格搬出 code repo」屬利基，遞延）；僅 SSE（逼 WS 生態整合者改造或忍輪詢延遲）；僅 WebSocket（反向鎖定、② 的 SSE＋LISTEN/NOTIFY 重工）；② 僅 pg（網路磁碟情境無開箱路徑）。
**Open**: ④' server-mcp 取代 web-agent-channel 與 ⑤' desktop-acp-agent 復活待裁決（含舊情境 2 確死確認）；多人直掛共享 fs 路徑（不經服務）的警語歸屬文件。

### Round 3 — assumptions (2026-07-10)

**Focus**: agent 面收尾與「打掉重排」的全面重置提議
**Position**: 內嵌 agent 裁定不做；使用者把 web-agent-channel 的處置升級為全面重置——現行討論＋變更全 discard、以單一新討論重出乾淨刀組。
- ⑤' desktop-acp-agent 裁定不做：desktop 維持純檢視器；情境 2 由第三方檢視器自帶 agent（SDK/REST＋MCP）滿足；speclink 不自營內嵌 agent 為刻意取捨。
- 全面重置：4 個 in-flight 變更全 discard（皆 0 任務、無程式碼可退）、舊路線圖討論一併清場；Δ1/Δ2/Δ3 內建進全新提案，不走 link＋ingest 疊補丁。
- 重置後新刀組（皆自本討論轉出，序固定）：sdk-store-seam（SDK dispatch 完備＋server 端運算＋Store 整合面文件化＋契約推播宣告欄與事件酬載 schema）→ team-server（createEngine 服務＋pg｜fs 雙儲存＋admin 認證＋SSE 推播＋docker-compose）→ desktop-remote（後端替換＋設定分叉＋輪詢地基＋SSE/WS 雙 client＋宣告發現）→ server-mcp（tools＝動詞、prompts＝技能 MCP 變體、PAT 認證）。
**Ruled out**: desktop 內嵌 ACP agent（純檢視器定位勝出）；hosted agent 全流程（舊情境 2）正式死亡、無繼承刀；link＋ingest 疊補丁路線（敘事仍碎，重置更乾淨，代價為重跑 propose）。
**Open**: wipe 邊界——manual-spec-edit-integrity 屬規格完整性主題非本路線圖，砍或歸檔待裁；本討論自身「也砍掉重開」vs「留作新刀組的單一源頭」待裁。

### Round 4 — assumptions (2026-07-10)

**Focus**: docs/platform-architecture.zh-TW.md 平台藍圖評審——對既有結論的吸收、增補與牴觸
**Position**: 建議採納為目標架構：藍圖幾乎完整吸收本討論結論，補上三個關鍵缺層，僅一處直接牴觸（WebSocket）。
- 吸收確認：儲存縫可換＋官方 server fs/pg 雙儲存（Δ3）、事件酬載入契約（§9.1 invalidate 形狀，Δ1）、MCP adapter、server 端 validate/analyze/drift、desktop 純檢視器雙模、不自營 web GUI／不自營內嵌 agent、規格正典唯一（防繞過）。
- 增補一 Speclink Host（§4.2）：把 actor/授權、lifecycle 裁決、CAS/idempotency/Unit of Work、事件發布自「服務層」抽為可複用 library；HTTP server 降為 Host 的一個 adapter。原結論讓每個自實作服務重做這些正確性關鍵件，是實質缺陷——Host 補上後「自實作服務」故事才安全。
- 增補二 Context Projection（§7）：遠端正典＋本地唯讀帶 revision 快照（.speclink/context/，gitignored、可丟棄）——直接修正 ① 沿用的「遠端 agent 一律經動詞讀文件」：該設計是 round-trip 地獄且無跨文件 grep，先前接受得太快；寫入仍一律 Command＋expectedRevision。
- 增補三 typed commands＋domain events＋Protocol 正式化（§4.1/§4.5）：dispatch 保留為相容層；Command/Query/Context/Event 四 API＋OpenAPI＋conformance suite，使「desktop 對任何合契約服務可用」成為可驗證命題；In-process Tool Adapter（§8）收編舊 web-agent-channel 的有效核心（Copilot SDK defineTool→Host）而不由 speclink 自營 agent。
- 牴觸：§9.2/§15 把 WebSocket 排除於基本契約（polling＋ETag 必備、SSE 建議標準、WS 留給真雙向需求）——與本輪 Δ2「desktop 雙傳輸」相反，實為翻回 server-auth-and-push-transport 原裁定；藍圖理由更有紀律（invalidation 單向、宣告欄傳輸中立、未來 WS 純加法）。
- 尖角：(a) 範圍暴漲——平台藍圖全量遠大於四刀，conformance suite／typed commands／@speclink/ui 發布可薄做或後置；(b) Projection 寫入不對稱——agent 習慣 Edit 剛 Read 的檔，需機制性防護（唯讀屬性＋digest 髒快照偵測＋remote 技能強導，銜接 manual-spec-edit-integrity 結論）；(c) 快照一致性繫於 Store 能力等級（fs 單寫序列化、pg 交易），conformance 需涵蓋；(d) 無 checkout 的 PM/PO（Claude Desktop MCP）之 context 策略未明——FS projection 無處落地，MCP adapter 需以 resources／search tools 供 context。
**Ruled out**: 拒收藍圖、維持四刀原結論不動（藍圖補的 Host／Context 縫正是自實作服務故事的正確性關鍵，原結論確有缺層）。
**Open**: WS 裁定是否翻回（採藍圖立場）；路線圖依 §14 九步重切的刀數與 v1 邊界；本討論 re-conclude 收編藍圖。

### Round 5 — assumptions (2026-07-10)

**Focus**: 藍圖 v2（新增 §4.6/4.7 binding、§9.2 傳輸重寫、§15 P0/P1 漏洞清單）對照現況程式碼的完整性審查
**Position**: 藍圖在平台正確性面已高度完整（§15 自行抓到大多數缺口、§9.2 以「Query＋ETag 地基＋可宣告 SSE/WS 單選傳輸」解掉 WS 矛盾）；程式碼比對找出四主題遺漏，其中「Host 單一實作」屬架構成立層級。
- 主題一 動詞內在結構與藍圖切法不合：drift 五維僅 Specs 維純 store，Time/Structure/Tasks/Environment 皆 shell-out 本機 git／工作樹（drift.rs:274-536；cmd_drift fs-only 無 remote 分支）——§6.4「規格面 drift server 算」需先分解 drift 動詞；verify 無引擎後端，evidence 雛形（.speclink/touched/ 記錄、archive 收集 git 變動組 @trace）焊死本機且 gitignored——server 端 archive 的 @trace 斷炊，需「code 事實以 evidence 上行」設計（task-done/verify 酬載帶 touched/commit SHA/測試摘要）。
- 主題二 Host 單一實作問題（最重）：本地路徑為 Rust 直嵌（CLI、desktop-core），遠端 Host 規劃為 Node——若裁決/CAS/UoW 落在 TS Host，本地就沒有 Host 或需第二個 Rust Host，違反 §2.1 一套流程語意；且 Rust 動詞內部多次呼叫 Store（archive 逐檔寫 canonical＋搬移，非交易、無 journal），Host 級 UoW 需 Store contract 增交易邊界（bridge begin/commit/rollback）或動詞改發 write-set。藍圖需明文分配：engine（Rust、共享的裁決與生命週期規則）vs Host（組合/認證/傳輸/事件發布）。
- 主題三 markdown 真相 vs 平台原語：stable task ID（P0）vs 可直編 tasks.md 的 ID 機制未定；board_rank 存 change meta／討論 frontmatter（model.rs:34-37）——remote 下變共享文件寫入（多人拖卡互打＋CAS 流量），per-user 呈現狀態無家；config/meta 讀取解析失敗一律靜默退預設（AppConfig::load、WorkflowConfig::from_text、ChangeMeta::from_text），壞 .speclink.yaml 靜默掉回 fs 模式——直接違反 §4.7「不得自動選」的 fail-closed 精神；LANGUAGE.md 契約只有 GET、無寫入動詞。
- 主題四 涵蓋面小洞：Query API 無封存列舉端點（desktop 封存頁遠端無著落）；discuss link/seal（ingest 流）remote 現被拒、未入 Command API；藍圖 ETag 用語 vs 契約現實為整數 version＋If-Match；@speclink/store-fs 的 P0 不可變 revision 機制未定（git-backed journal 為自然解）；MCP 無 checkout 時的 context 與 skills 送達策略；Projection 唯讀需機制性防護（唯讀屬性＋digest 髒偵測）；WorkspaceAdapter（專案/設定第二 UI 契約）不在 §10.3 目標介面內。
- 現況距離快照（債務非遺漏）：engine Store 縫零 revision/CAS/claim（原語只活在繞過引擎的 speclink-remote HTTP client）；FsStore 直接覆寫無 temp+rename；domain event 為零且現契約 §1.3/§7 明文拒 outbox（將被藍圖取代、需 bump api version）；@speclink/engine dispatch 僅 4 動詞；analyze/drift/validate/show 遠端未接線。
**Ruled out**: 「藍圖已可直接定案」（主題二未明文化會使本地/遠端流程語意分叉，正是 §2.1 要防的）。
**Open**: engine vs Host 職責分配明文化；drift/verify 分解與 evidence 上行設計；task ID 與 board_rank 機制拍板；讀取路徑 fail-closed 修法；路線圖依 §14 重切（前輪擱置問題——WS 一題已被 §9.2 重寫自答）。

### Round 6 — assumptions (2026-07-10)

**Focus**: 藍圖 v3（GPT-5.6 Sol 補強版）複審——前輪四主題遺漏的關閉狀況
**Position**: 兩個最重的洞已紮實關閉；唯 drift 分解仍缺，為定案前最後一個實質缺口；其餘為設計期決策、不擋定案。
- 主題二（Host 單一實作）關閉：§4.1 明定「Rust Engine 是唯一流程語意實作」，官方以 N-API 發布 Node addon 而非 TS 重寫流程；§8 補發布契約（win/mac/linux x64/arm64 預編譯 binary、ABI 範圍、版本檢查）並明文「native addon 不可用必須 fail closed、不得靜默切換到另一套 JavaScript 流程實作」；in-process 路徑固定為 @speclink/copilot-tools→@speclink/host→@speclink/engine(N-API)→Node Store Adapter；Node Store bridge 須維持與 Rust Store contract 相同 transaction/CAS/錯誤語意。
- 主題一（evidence 上行）半關閉：verify 側完整落地——task done 上傳 touched files（按 stable task ID/actor/repo/base+head commit 保存，明標「檢查範圍與追蹤線索，不是正確性證明」）；VerifyBundle 固定 spec/tasks/policy revisions；VerifyEvidence 帶逐 task 結果、測試/audit 摘要、commit SHA、basis revisions、工具版本、trust level；basis 變動回 stale_evidence 拒收。§15.4 同步更新。
- 唯一中型缺口：drift 分解仍未寫——現況 drift 五維中四維（Time/Structure/Tasks/Environment）靠本機 git/工作樹（drift.rs），§6.4 仍只有一句「規格面的 drift 由 server 算」；需補：維度歸屬（Specs 維 server 算、其餘於 client 對 Context Projection＋本機 git 跑）、兩半結果如何合併成一份 drift 報告、遠端 CLI 的 drift 接線。
- 新內容引出的小發現：VerifyBundle 的 policyRevision 隱含 policy 住 server store，但 requiredDisciplines 對應的 tdd/audit 旗標現居 .speclink.yaml（AppConfig，本地 workspace 檔）——policy 的家需搬遷或鏡射至 store，藍圖未明說。
- 殘餘設計期決策（各刀 design.md 定，不擋藍圖定案）：stable task ID 於 markdown 的機制；board_rank 與 per-user 呈現狀態在遠端的家；讀取路徑 fail-closed（壞 .speclink.yaml 靜默掉回 fs 模式，建議升為 §15.1 P0 一行）；@speclink/store-fs 的不可變 revision 機制（git-backed journal 為自然解）；MCP 無 checkout 的 context 策略；projection 唯讀的機制性防護；封存列舉與 discuss link/seal 入 Command/Query API 涵蓋；ETag vs 整數 version＋If-Match 用語統一；WorkspaceAdapter（專案/設定第二 UI 契約）的遠端對應面。
**Ruled out**: 「還需要大改」——架構骨架與 P0 正確性面已閉合，剩餘皆局部。
**Open**: 補 drift 分解一節＋P0 加壞設定 fail-closed 一行後即可定案；定案後機械事項：藍圖入版控、本討論 re-conclude 收編、路線圖依 §14 重切（前輪擱置）。

### Round 7 — assumptions (2026-07-10)

**Focus**: 藍圖 v4 複審——前輪三個待補點的關閉確認與完整性判定
**Position**: 三點全數紮實關閉，判定藍圖完整、可定案。
- §6.5 遠端 Drift 分解（新）：五維歸屬表（Specs＝Server；Time/Structure/Tasks/Environment＝Client）與各維輸入明細，與 drift.rs 現況依賴完全對齊；流程為 prepareDrift→server 端 computeSpecDrift→DriftBundle＋Context Manifest→client materialize＋computeWorkspaceDrift→Rust Engine 共用 merger 輸出 CombinedDriftReport（CLI/Node SDK/Desktop 不各自重寫 scoring 與合併）；無 code checkout 時四維標 unavailable、不得視為 clean 或零分，預設回 workspace_required、僅顯式 --spec-only 出 coverage:"spec-only" 部分報告；報告帶 basis revisions、期間遠端 revision 變動標 stale；drift 為診斷不回寫正典、保存 handoff/evidence 另走帶 revision 的明確 command。
- §4.8 Workflow Policy 歸屬（新）：本地＝openspec/config.yaml；遠端＝Store 內 repo-scoped 可版本化 config.yaml 為唯一 authoritative；schema/context/rules/spec_locale/tdd/audit 由 Host 於固定 Store revision 讀取、交同一份 Rust Engine fail-closed 解析為 EffectiveWorkflowPolicy；policyRevision＋digest 進 instructions／Context Snapshot／apply／verify bundle；遠端 .speclink.yaml 只管 endpoint／binding／本機 client 偏好、不鏡射 policy——隱含 tdd/audit 自 .speclink.yaml（AppConfig 現址）遷居 config.yaml，實作期需處理 deprecated keys。
- P0 新增第 9 行＋驗收條件 #7：「壞設定會靜默切換語意或模式」——缺檔才允許明定預設；檔案存在但 parse/validation 失敗必須回 typed error 並停止，不得退回 fs mode 或產生 instructions；所有入口一致 fail closed 且不改變 binding/mode。正中前輪指出的 AppConfig::load 靜默退預設與壞 .speclink.yaml 靜默掉 fs 的現行違例。
- 完整性判定：架構骨架、單一語意實作（Rust Engine＋N-API、native addon 不可用 fail closed）、P0 正確性面、evidence 鏈（VerifyBundle/VerifyEvidence/stale_evidence）、drift 分解、policy 歸屬、設定 fail-closed——全部閉合。殘餘項（stable task ID 於 markdown 的機制、board_rank 與 per-user 呈現狀態的家、@speclink/store-fs 的 revision 機制、MCP 無 checkout 的 context 策略、projection 唯讀機制性防護、封存列舉與 discuss link/seal 入 API、ETag vs version 用語、WorkspaceAdapter 遠端對應面）皆屬各刀 design.md 層級，不擋定案。
**Ruled out**: 續留「待補」狀態（三個擋定案項已全閉）。
**Open**: 端遊三步待使用者拍板——藍圖入版控、本討論 re-conclude 收編藍圖為正式結論、路線圖依 §14 重切並自本討論逐一轉出。

## Conclusion

**Decision**: 採納 docs/platform-architecture.zh-TW.md（可組合平台架構藍圖）為多人協作的目標架構與整合基準，取代本討論先前的四刀結論；路線圖依藍圖 §14 重切六刀，皆自本討論轉出、逐一 propose、不留 promote 骨架：
- 刀 1 engine-typed-core（§14 步 1＋2）：Engine typed commands/outcomes、domain events（dispatch 相容層保留）、Store 能力等級與 revision/CAS/Unit of Work 介面、設定解析 fail-closed（P0 第 9 行）。
- 刀 2 speclink-host（步 3＋4）：@speclink/host（actor/policy/交易/事件集中）、Command/Query/Context/Event Protocol 固定、@speclink/client、Project/Repo binding 契約（§4.6/§4.7）。
- 刀 3 context-projection（步 5）：Context Materializer 與 .speclink/context/、remote skills 的 snapshot/read/write 規則、drift 分解接線（§6.5）、VerifyBundle/VerifyEvidence（§6.4）。
- 刀 4 team-server（步 6）：官方 speclink-server——FS＋PostgreSQL 雙儲存、PAT/身分、Project/Repo registry、binding handshake、SSE＋Polling/ETag、docker-compose。
- 刀 5 desktop-remote（步 7）：Desktop 接 Local/Remote DataSource、subscribe 面、event capability negotiation、Polling/ETag＋SSE（WebSocket 選配）。
- 刀 6 agent-adapters（步 8）：Node N-API Engine binary 發布、@speclink/copilot-tools（In-process Tool）、MCP Adapter、多租戶整合指南。
- 步 9（UI contract 穩定化）隨刀 5/6 收尾不獨立成刀；各刀精確邊界（如 drift/verify 歸刀 3 或再拆）由各自 propose 定。
**Rationale**: 藍圖經四版迭代閉合全部架構層級缺口：v1 評審定方向、v2 補 P0/P1 與 Project/Repo binding、v3 補 Host 單一實作（Rust Engine 唯一流程語意、N-API 發布、native addon 不可用即 fail closed）與 evidence 上行（VerifyBundle/VerifyEvidence/stale_evidence、touched files 按 stable task ID 保存）、v4 補 drift 五維分解（Specs＝server、四維＝client、共用 merger、unavailable≠clean）、workflow policy 唯一歸屬（EffectiveWorkflowPolicy＋policyRevision）、壞設定 fail-closed（缺檔才允許預設）。三情境以「同一套 Engine、兩條執行路徑、可替換入口/儲存/呈現」統一收斂，勝過為情境各做一套交付物。
**Rejected alternatives**: 四刀舊結論（藍圖新增 Host/Context Projection/typed commands 三層後不敷承載）；hosted-agent 全流程與 speclink 自營 web GUI／內嵌 agent（維持先前裁決——agent 永遠外部：CLI 技能/MCP/整合者 In-process Tool）；以 TypeScript 重寫流程規則（Rust Engine 為唯一實作、fail closed 不得靜默切換）；本地 Web UI／localhost bridge（§5.3 明確不建議）；SSE 或 WebSocket 作為正確性地基（Query＋ETag 才是恢復路徑，push 為可宣告的加速、單一訂閱單選傳輸）；OpenSpec 式 store registry/workset（無伺服器解耦屬利基、遞延）。
**Deferred**: 設計期決策由各刀 design.md 定案——stable task ID 於 markdown 的機制；board_rank 與 per-user 呈現狀態在遠端的家；@speclink/store-fs 的不可變 revision 機制（git-backed journal 為候選）；MCP 無 checkout 的 context 策略；projection 唯讀的機制性防護（唯讀屬性＋digest 髒偵測）；封存列舉與 discuss link/seal 入 API 涵蓋；ETag vs 整數 version＋If-Match 用語統一；WorkspaceAdapter 的遠端對應面；tdd/audit 自 .speclink.yaml 遷居 config.yaml 的 deprecated keys 處理。另遞延：本地 stdio MCP、desktop WebSocket client（選配）、server↔git 匯出橋（P1 store push/pull 已定方向）。
**Capture to**: docs/platform-architecture.zh-TW.md（定案為整合基準）＋ proposal（六刀各自 propose 時自本結論與藍圖取材）
**Next**: /speclink-propose --from-discussion collab-scenario-replan --name engine-typed-core（其餘五刀輪到再 propose）
