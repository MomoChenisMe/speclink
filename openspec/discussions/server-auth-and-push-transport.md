---
topic: 參考 server 的認證等級與推播傳輸分層（輪詢地基＋可宣告推播）
slug: server-auth-and-push-transport
status: promoted
promoted_to: web-server-postgres, speclink-sdk-and-store-seam, desktop-remote-mode
created: 2026-07-09
---

# Discussion: 參考 server 的認證等級與推播傳輸分層（輪詢地基＋可宣告推播）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

承接已結論的 sdk-storage-seam-and-remote-desktop（三刀重切）。使用者接受 ② web-server-postgres 的重定範疇機制（link＋ingest、① 先 propose），但修正兩處邊界：(1) ② 的認證不能是丟棄式 demo 級——它也算開箱即用，admin 認證要認真規劃；(2) 推播傳輸：SSE 不該被寫死進 desktop——SSE 是 client/desktop 的擴充、跟引擎無關；整合者若用 WebSocket，寫死 SSE 會讓其 server 資料同步不進 desktop。使用者問「你的想法是？」。

模式：assumptions——甫完成 web-server-postgres propose ＋前一討論，對 verb-contract（push 屬 server freedom、在請求/回應契約外）、desktop 資料源、SSE/LISTEN-NOTIFY 皆有一手掌握。

相關：web-server-postgres（②，本討論精修對象）、三刀 ①②③、verb-contract 正典（§preamble/§7 push 為 host concern）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: ② 的認證等級，以及 live-sync/push 怎麼分層才不被 SSE 鎖死？
**Position**: ② 認證＝認真規劃的 admin 認證（開箱堪用），非丟棄式；推播＝輪詢地基＋可宣告、傳輸無關的選配。
- ② 認證：properly-designed admin 認證（token 建/列/撤＋repos 註冊表＋啟動 bootstrap），真能給小團隊開箱用；帶自家系統的整合者仍換掉整個 server 層、自帶認證——故 ② 認證是 batteries-included 的堪用預設，非強制枷鎖。收回先前「demo 級」。
- 推播分層：desktop 地基＝輪詢（對任何 verb-contract server 都動、不管有無推播或用什麼傳輸）；推播為 OPTIONAL＋ADVERTISED＋TRANSPORT-AGNOSTIC。SSE server＋LISTEN/NOTIFY→②；SSE client＋輪詢 fallback＋發現→③；契約層一個可選宣告欄 events:{url,transport}→①（傳輸無關的互通膠水）。整合者的 WebSocket server 宣告 transport:"websocket"；desktop 支援該傳輸就即時同步、否則退回輪詢——永不鎖死。引擎①除宣告欄外零推播。
**Ruled out**: 對所有 server 規定 SSE（鎖死 WS 整合者的 desktop 同步——使用者主訴）；② 用丟棄式 demo 認證（② 開箱即用）；現在就實作 desktop 的 WS client（未來——先把發現機制設成傳輸無關、現在只實作 SSE＋輪詢）。
**Open**: 宣告欄放契約層①還是降為②↔③約定（互通 vs ① 零推播的取捨）；desktop WS client 現在做還是遞延（傾向遞延）。

### Round 2 — assumptions (2026-07-09)

**Focus**: 推播宣告欄 events:{url,transport} 住哪層？
**Position**: 契約層①——寫進 docs/verb-contract.md 作為可選、傳輸無關的發現約定。
- 明標「push 在請求/回應契約外，此欄僅供發現」；引擎仍零推播機制。
- 互通性最強：任何整合者的 server 都有契約背書的統一方式，讓 desktop（及未來 client）發現其推播通道並自行決定連或退回輪詢。
**Ruled out**: ②↔③ 約定（① 更純，但整合者少契約背書的宣告方式、互通稍弱）；現在做 desktop WS client（遞延——發現機制傳輸無關已備，現只實作 SSE＋輪詢，未來純加法補 WS）。

## Conclusion

**Decision**: 精修三刀中 ② 的認證等級與跨 ①②③ 的 live-sync/push 分層。
- ② 認證：認真規劃的 admin 認證（token 建/列/撤＋repos 註冊表＋啟動 bootstrap），開箱堪用、非丟棄式 demo；整合者帶自家系統時換掉整個 server 層、自帶認證，故 ② 認證為 batteries-included 堪用預設。
- live-sync/push 分層（傳輸無關、可宣告、輪詢地基）：
  - ③ desktop 地基＝輪詢——對任何 verb-contract server 都同步，永不被單一傳輸鎖死；推播為選配優化。
  - ② demo server 實作 SSE＋Postgres LISTEN/NOTIFY，並宣告 transport:"sse"。
  - ① 契約新增可選、傳輸無關的宣告欄 events:{url,transport}（掛 whoami/config metadata；明標 push 在請求/回應契約外、僅供發現）；引擎本體零推播機制。
  - ③ desktop 實作 SSE client＋輪詢 fallback＋讀宣告發現；WebSocket client 遞延（發現機制已傳輸無關，未來純加法）。
**Rationale**: (1) 輪詢地基使 desktop 對任何 server（任何/無推播傳輸）都同步、永不鎖死——WebSocket 整合者的 server 照樣以輪詢同步進 desktop，補 WS client 才升級即時。(2) 宣告欄放契約層給整合者契約背書的統一發現方式（互通最強），同時引擎不沾推播機制（推播確非引擎職責）。(3) ② 開箱即用故認證認真規劃，但整合者替換整個 server 層自帶認證，故為堪用預設非枷鎖。
**Rejected alternatives**: 對所有 server 規定 SSE（鎖死 WS 整合者的 desktop 同步）；② 用丟棄式 demo 認證（違開箱即用）；宣告欄降為 ②↔③ 約定（整合者少契約背書、互通稍弱）；現在就實作 desktop WS client（遞延，純加法）。
**Deferred**: desktop WebSocket client 的實作時機（未來按需）。
**Capture to**: design（① 契約宣告欄＋引擎零推播、② 認證＋SSE server、③ 輪詢地基＋SSE client＋發現）
**Next**: 沿用 sdk-storage-seam-and-remote-desktop 序列——propose ① → link+ingest ② → propose ③，三步各併入本討論的推播分層與認證決策
