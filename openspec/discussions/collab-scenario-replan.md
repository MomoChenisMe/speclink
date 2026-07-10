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

## Conclusion

**Decision**: 全面重置多人協作路線圖：4 個 in-flight 變更與 3 份路線圖討論已 discard（快照 commit ed505ef 保底）、manual-spec-edit-integrity 已歸檔；本討論為單一源頭，重出四把新刀、依序逐一 propose、不留 promote 骨架。
- 統一堆疊（三情境共用）：儲存層可換（Store 介面：postgres｜fs（本地/NAS/網路磁碟）｜任何自家系統）→ 服務層（使用者自實作 createEngine＋自家 Store｜開箱 team-server）→ 介面層（自建 UI 走動詞契約｜speclink desktop）。agent 永遠外部（CLI 技能／MCP／第三方自帶）；speclink 不自營 web GUI、不自營內嵌 agent；情境 3（全本地 git）已交付、零新工程。
- 刀 1 sdk-store-seam：@speclink/engine dispatch 補完全遠端託管動詞集；analyze/validate/drift server 端運算；Store 縫文件化為公開整合面；動詞契約修訂——遠端＝server 端運算、可選推播宣告欄 events:{url,transport}、傳輸中立的最小 invalidate 事件酬載 schema（新增）、遠端 agent 一律經動詞讀文件。
- 刀 2 team-server：開箱 headless 團隊服務——createEngine＋雙儲存選項（PostgreSQL 參考 Store｜fs 目錄，version/If-Match 以內容雜湊＋服務進程內序列化；目錄為 git checkout 時規格自帶 git 歷史）＋認真規劃的 admin 認證（PAT 建/列/撤＋repos 註冊表＋啟動 bootstrap）＋SSE 推播（LISTEN/NOTIFY、宣告 "sse"）＋docker-compose；繼承 manual-spec-edit-integrity「防繞過強保證唯遠端可達」定位。
- 刀 3 desktop-remote：desktop 遠端模式——resolve_mode 後端替換複用 RemoteClient；設定面分叉（config 遠端唯讀、.speclink.yaml 遠端卡＋PAT 使用者層級憑證）；運算走端點；輪詢地基＋SSE 與 WebSocket 雙推播 client、宣告欄自動選路。
- 刀 4 server-mcp：team-server 的 MCP 端點（streamable HTTP）——tools＝dispatch 動詞（in-process、不疊 REST）、prompts＝discuss/propose 技能 MCP 變體（第四個技能渲染標的）、PAT 認證；交付情境 1 的 PM/PO 面（Claude Desktop／Codex app connector）。
**Rationale**: 亂源是舊「四情境」分類殘留——hosted-agent 全流程（舊情境 2）無繼承者、web-agent-channel 疊在被砍的 web 應用上、新情境 1（外部 GUI agent host）無載具、內嵌 agent 從未建立。四變更皆 0 任務完成，重置只花 artifacts 重寫成本，換單一敘事並把三個增量（酬載入契約、雙傳輸、fs 儲存選項）內建進新提案、免去 link＋ingest 疊補丁。「自實作服務升頭牌」驅動全部三增量：酬載不入契約則自實作服務對 desktop 永遠只有輪詢；雙傳輸成本不對稱（desktop 做一次 vs 每個 WS 生態整合者改造）；fs 選項讓網路磁碟/NAS 零 postgres 起跑。
**Rejected alternatives**: link＋ingest 在舊三刀上疊補丁（敘事仍碎）；保留 web-agent-channel／hosted-agent 全流程（未來要時 MCP 已備、剩 apply 沙箱位置另議）；desktop 內嵌 ACP agent（純檢視器定位勝出、情境 2 由第三方檢視器自帶 agent）；speclink 自營 web GUI 與 web-role-views；OpenSpec 式 store registry／workset（無伺服器解耦屬利基、遞延）；僅 SSE 或僅 WebSocket（單傳輸鎖定，且僅 WS 需重工 SSE＋LISTEN/NOTIFY 設計）；team-server 僅 pg（網路磁碟情境無開箱路徑）。
**Deferred**: 本地 stdio MCP（GUI agent host 直操本地 repo、情境 3 GUI 化）；SSE/WS 以外傳輸的 client（宣告欄已傳輸中立、未來純加法）；server↔git 正式同步橋（fs 儲存＋git checkout 已給低成本路徑，正式橋待真需求）；多人直掛共享 fs 路徑（不經服務）的警語位置（team-server 定位文件任務內定）。
**Capture to**: proposal（四刀各自 propose 時自本結論取材）
**Next**: /speclink-propose --from-discussion collab-scenario-replan --name sdk-store-seam（其餘三刀輪到再 propose）
