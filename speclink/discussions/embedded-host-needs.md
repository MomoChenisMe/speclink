---
topic: 內嵌宿主、遠端能力與「規格不在 checkout 裡」的形態：需求清單
slug: embedded-host-needs
status: promoted
created: 2026-11-10
created_by: MomoChen <momochenisme@gmail.com>
hold: true
promoted_to: store-port-contract, engine-on-store-port, shared-verb-parsing, shared-verb-output, workflow-config-commands, host-kit-agent-door, client-browser-login, skills-verbs-first-wave, server-routes-on-store-port, host-kit-client-door
---

# Discussion: 內嵌宿主、遠端能力與「規格不在 checkout 裡」的形態：需求清單

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因是一份外部的需求清單：一個團隊要把 speclink 內嵌進自己的系統（Node 後端、自己的帳號與 Postgres），非工程角色經 AI Agent 做討論與提案，工程師用 CLI 與 Speclink Desktop 連到宿主，封存由宿主發起。文件列了 SDK（S1–S9）、協定與用戶端（C1、C2、P1、P2）、技能（K1–K4）與遠端模式既有缺口共 22 項主張。需求已夠明確，沒有做 grill；輸入是文件，所以第一輪是逐項對照程式碼，不另列假設。
相關正式規格：node-sdk、teamstore-contract、host-runtime、store-abstraction、postgres-team-store、remote-auth、server-device-auth、client-protocol、server-verb-api、server-event-stream、server-policy-write、skill-routing、verb-contract、spec-dir-migration、workflow-schemas。進行中的變更：station-subagent-model、add-pi-cursor-tools；in-flight delta 命中 add-pi-cursor-tools: node-sdk。
背景文件：已刪除的平台藍圖 docs/design/platform-architecture.zh-TW.md 與重構路線圖 docs/design/implementation-refactor-roadmap.zh-TW.md（刪於 147076be，可從 git 歷史讀回），兩者是 TeamStore 與 Node 路徑的原始設計依據。
Prior discussions: agent-tool-sdk-layering, sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios, collab-scenario-replan, node-sdk-completion-and-doc-alignment, schema-engine-openspec-parity, improve-wire-convert-seam
Source doc: /Users/momochenisme/Documents/GitHub/speclink-embedded-host-needs.md

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-10)

**Focus**: 需求文件的 22 項主張，哪些與程式碼相符、哪些講錯、哪些是真正要決定的事
**Position**: 14 項相符、8 項講錯或只對一半；真正要決定的事收成十個節點，根節點是 Node SDK 的自訂儲存要接在哪一層。
- 相符：S1（Node 只有 fs 與 JS 物件兩種儲存，Postgres driver 只有 speclink-server 在用；歷史列只存 digest）、S2（dispatch 只有 list、status、new change、new artifact、claim、審查與驗證各兩個，共 9 個）、S3（SDK 與引擎都沒有執行前檢查；server 只查 editor 角色；PUT /config 整份覆寫）、S6（app.rs:27 的 32 MB 常數，無壓縮，SDK 無匯入匯出）、C1（只有裝置登入與存取金鑰；握手先驗 token 且 authentication 永遠是空陣列；五個登入路徑寫死在用戶端）、P1（線上 ChangeSummary 無審查與驗證欄位；六種審查與驗證事件被 server 歸成 unknown）、K1 的 discuss（discuss.md:80 用路徑讀 spec.md）、K2（init.rs:718 直接讀本機 config.yaml 的 worktree 鍵）、K4（無唯讀引用）、遠端五缺口（trace、plan --strict-overlap、change rank 明寫拒絕；手冊產生拒絕；自訂產出流程 server 不存）
- 講錯或只對一半：K3（spec_dir 沒有檢查，原樣接到根目錄後面，絕對路徑與 ../ 依 Path::join 會被接受，無測試涵蓋）；K1 前提（技能範本沒有任何分支，本機與遠端拿到同一份技能檔，工作是把路徑讀檔改成動詞）；K1 的 apply（查的是變更目錄的歷史，apply.md:109）；drift 只用動詞、遠端可用，ingest 與 propose 會呼叫遠端拒絕的 change rank；S8（只有 server 路徑載入整個範圍，Node 自訂儲存不是）；S4（事件來自引擎，fs 形式也被丟掉）；S7（無寫入動詞，但檔案裡的未知鍵逐字保留）；S5、C1、C2、P2 各對一半（Rust 有 router() 但綁官方身分系統；用戶端原樣開啟 verification_uri；只檢查 API 版本；task done 有帶 headCommit）
- S1 另有文件沒提的現況：Postgres driver 一條連線加一把鎖、不用 TLS、舊資料表不自動遷移、schema 靠連線字串的 search_path 隔離
- 記錄方式：另開本記錄並引用 agent-tool-sdk-layering，使用者未反對
- 使用者要求以圖解說明遠端架構，已發佈 artifact（一顆引擎四個入口、三種用法、目標形態與缺口、問題總表）
**Ruled out**: 接續 agent-tool-sdk-layering 記錄 — 那一場談工具名片與技能怎麼送進 agent，和本文件只在 S2、S3、S4 重疊
**Open**: D1 Node SDK 的自訂儲存接哪一層（決定 S1、S4、S9、S8）；D2 S5 遠端入口的形狀與每個請求的身分；D3 S2 動詞怎麼補齊；D4 S3、S4 的檢查與事件放哪一層；D5 S8 效能；D6 C1 瀏覽器登入與免登入的宣告端點、C2 版本提醒；D7 P1、P2、S7 線上欄位，S6 匯入匯出；D8 K1–K4 技能與規格位置；D9 遠端五缺口補哪幾項；D10 分刀與順序

### Round 2 — interview (2026-11-10)

**Focus**: Store 介面目前還在哪裡用；TeamStore 契約還缺什麼
**Position**: Store 是引擎的內層介面，全部入口都經過它；TeamStore 有七個缺口，前四個直接影響內嵌宿主。
- 引擎的 execute 只收 &dyn Store（command/mod.rs:779）。六個實作：FsStore（CLI 與 Desktop 本機模式、Node SDK 的 fs 形式）、BridgeStore（官方 server，把 TeamStore 轉成 Store）、JsStoreBridge（Node SDK 自訂 JS 儲存）、WorktreeOverlay、RemoteDriftStore、TestStore
- 正確的圖是兩層：引擎 → Store 介面 → 三條主要的路；只有 BridgeStore 那條往下接 TeamStore
- TeamStore 缺口：(1) 快照只能點讀，列舉只能靠 export 整個範圍，這是 S8 的根源（bridge.rs:143）；(2) 三個官方 driver 都宣告 SingleNode，文件明寫不支援多個 server 共用資料庫；(3) 合格測試只能測 Rust 實作，npm 套件裡沒有；(4) 每個範圍只有一個事件已讀位置，且找不到清舊事件的程式；(5) 歷史只存摘要；(6) 文件種類固定 9 種，沒有自訂產出流程；(7) Postgres driver 無連線池、無 TLS、手動遷移
- 讀程式碼看到、未經測試證實的風險：Postgres 的鎖是每個 Project 加 Repo 一把，版本計數器是每個 Project 一個（speclink-store-postgres/src/lib.rs:413、:419），多個程序同時寫同一 Project 的兩個 Repo 可能算出同一個版本號
**Ruled out**: 「舊的 Store 介面與新的 TeamStore」這個說法 — 官方 server 也經 Store（BridgeStore），Store 沒有被淘汰；第一輪圖解的用詞已更正
**Open**: D1（改寫為：Node SDK 的自訂儲存要不要從 JsStoreBridge 那條路搬到 BridgeStore 加 TeamStore 那條路）；為什麼會有三條路；TeamStore 是否適合內嵌宿主的情境；D2–D10 不變

### Round 3 — interview (2026-11-10)

**Focus**: 為什麼儲存有三條路；TeamStore 適不適合內嵌宿主的情境
**Position**: 三條路是先後順序造成的，不是刻意並存；TeamStore 適合這個情境，而且平台藍圖本來就規劃 Node 路徑要搬上去，只是第四階段一直沒做。
- 2026-07-04 store-trait-and-fs-adapter：把引擎與檔案系統切開，產生 Store 介面與 FsStore
- 2026-07-05 node-sdk：第一個 Node 使用者要內嵌引擎並接自己的資料庫；當時只有 Store 這個介面，所以 JS 儲存接在它上面
- 2026-07-11 前後 collab-scenario-replan 採納平台藍圖：官方 server 改為 Rust；因為 Store 表達不了版本比對、交易、歷史、事件與錯誤類別，另立 TeamStore
- 2026-07-13 teamstore-contract-v2：獨立 crate；決策二寫明「Node async Store bridge 在 N-API facade 層轉接（Phase 4）」
- 2026-07-14 server-http-adapter：BridgeStore 是 Store 的第二個供應者，背後接 TeamStore，引擎不改
- 第四階段（@speclink/host、@speclink/store-* 的 N-API facade、Node async Store bridge）沒有動工；node-sdk-completion-and-doc-alignment 只補了 actor，並把 @speclink/host 延後到「Host 面需求超過 actor 注入」
- 使用者的觀察成立：官方 server 與內嵌宿主是同一個概念。藍圖 §13 寫三種用法（內嵌 Engine/Host 加自己的 Store；在既有服務掛載 Protocol adapter；部署官方 server），並寫明「兩者只有 adapter 不同」。需求文件的情境是前兩種合用
- 需求對照藍圖：S1 即 @speclink/store-* facade；S9 即通過同一合格測試的 async Store bridge；S5 即「在既有服務掛載 Speclink Protocol adapter」；每個請求的身分即 §8 的 speclinkHost.dispatch({ actor, project, repo, argv, stdin })
- TeamStore 適合的理由：封存與蓋章要一次交易；S4 的至少送達一次就是待送佇列；S5 的線上協定本身建立在 revision 與 ETag 上，沒有版本號的儲存無法服務 CLI 與 Desktop
- 建議的第一步：Node SDK 新增一種建構形式，在 N-API 內直接連結官方 Postgres driver 與 host 橋接層；自訂 JS 版 TeamStore（S9）延後，因為這個情境用的是 Postgres
- 做法 A 要先補的缺口：列舉與延遲讀取（S8）、Postgres 多程序共用的保證、每個消費者各自的事件位置
**Ruled out**: 做法 B（在現有 Store 每個方法加版本比對） — 重構路線圖 §3.1 已明文不建議，TeamStore 就是為此另立；用 TypeScript 重寫儲存 driver — collab-scenario-replan 已否決，理由是在 Store 層出現第二份語意
**Open**: 使用者是否同意 D1 走做法 A、第一步只開放官方 Postgres driver；內嵌與「旁掛官方 server 並接宿主的身分」兩條路的取捨留到 D2 談；D2–D10 不變

### Round 4 — interview (2026-11-10)

**Focus**: 不以平台藍圖與 TeamStore 為依據，規格不在 repo 裡時儲存該怎麼設計
**Position**: 立場改變：第三輪以藍圖為依據建議「搬上 TeamStore、先開放官方 Postgres driver」，使用者指出藍圖與 TeamStore 都是未經驗證的舊設計、官方 server 只是示範，因此改從需求本身推導，建議改為由宿主實作的小型儲存接口。
- 使用者給的三個前提：藍圖是較舊的設計，不能當依據；TeamStore 未經真實驗證，不能假設是唯一解；官方 server 只是示範或試用，不是正式可用的產品
- 規格離開 repo 後失去 git 給的四件事：一次提交改多個檔、兩人不互相覆蓋、每個版本留存、看得到誰改了什麼。前兩件儲存一定要補；後兩件由持有資料庫的一方決定
- 引擎在官方 server 上已經是「先讀、跑流程、交出寫入清單加事件、一次提交」：bridge.rs 的 execute 先 materialize，再收集 StagedOp（Put 或 Delete，各帶預期版本），最後 commit_with_events。所以儲存必要的動作只有三個：列出文件、讀文件（帶版本）、一次提交（全部寫入、預期版本、事件）
- 建議的形狀：宿主實作三個函式，在自己的資料庫交易裡完成 commit。連線池、schema、遷移、TLS、歷史保存、事件表都由宿主用自己的工具處理，S1 的子項多數因此不需要 speclink 來做
- 這個形狀順帶解掉的需求：S4（事件在 commit 裡交給宿主，宿主寫進自己的事件表）、S8（用到才讀，不再整個範圍載入）、S9（合格測試只測三個函式，可放進 npm）、S6（匯入匯出可建在 list、read、commit 上，與媒介無關）、S3 的一部分（commit 經過宿主的程式，看得到寫入前後的內容）
- TeamStore 做對而要保留的三件事：一個動詞一次寫入、版本比對、事件與寫入同一次提交。不適合內嵌宿主的地方：speclink 自己開連線與遷移資料表、快照不能列舉、每個 Project 一個版本計數器、多節點未驗證、driver 只能用 Rust 寫
- 官方 server 是示範這個前提帶出一個結構問題：線上協定的完整實作今天只在 speclink-server 裡（api/routes.rs 2174 行、events.rs 707 行、read_api.rs 395 行）。要讓宿主服務 CLI 與 Desktop，這些要下移成共用的主機套件，官方 server 變成用同一套零件組出來的示範
- 這是新設計，還沒有程式；「三個函式足夠」的依據是 BridgeStore 今天只靠全部文件加版本就能供應引擎的全部讀取
**Ruled out**: 以藍圖當設計依據 — 使用者裁定它是較舊的設計；「TeamStore 加官方 Postgres driver 內建進 Node 套件」當第一步 — 宿主已有連線池、遷移工具與交易，第二套連線無法併進宿主的交易，且該 driver 無連線池、無 TLS、宣告單一節點；旁掛官方 server 並接宿主的身分 — 使用者裁定官方 server 只是示範
**Open**: 使用者是否同意 D1 改為「宿主實作三個函式的儲存接口，交易由宿主做」；既有 TeamStore 與三個官方 driver 的去留（改實作小接口、保留當示範、或並存）；既有 31 方法 JS 儲存的退場；讀取一致性要不要另給快照；文件識別用封閉的 9 種或開放的邏輯鍵（影響自訂產出流程）；D2–D10 不變

### Round 5 — interview (2026-11-10)

**Focus**: 改用小型儲存接口後，宿主是否要自己寫版本比對；既有的 TeamStore 怎麼處理
**Position**: 維持第四輪的方向，補上兩點：官方提供一份跑在宿主連線上的 Postgres 實作，宿主不用自己寫版本比對；既有的 TeamStore 留在內部，不對宿主公開。
- 一個引擎實例只綁一個身分（node-sdk 規格的建構期 actor）。服務多位使用者的宿主會為每個請求建立引擎實例。儲存物件因此必須能被多個實例共用，連線不能跟著實例走
- 官方 Rust Postgres driver 是一個 store 一條連線加一把鎖，沒有連線池與 TLS，宣告單一節點。要給內嵌宿主用，speclink 得在 Node 套件裡維護一整套資料庫連線層；宿主自己已經有一套
- 宿主選官方 driver 的合理動機是「不想自己寫版本比對與交易」。這個動機用另一種方式滿足：speclink 提供建表用的 SQL 與一份官方的 Postgres 實作，它收宿主給的查詢函式或連線池，跑在宿主的連線、schema、TLS、連線池設定上
- 合格測試跟著三個函式走，放進 npm；官方的 Postgres 實作與宿主自己寫的實作跑同一套
- 既有的 TeamStore 與 host 橋接層不丟：三個函式在內部轉接成 TeamStore，橋接、提交與事件的既有程式照用。要補的是「列出文件」與用到才讀
- 官方 Rust driver（SQLite、Postgres、serverfs）留給示範用的官方 server，不對 Node 開放
- 未驗證：三個函式是否足以供應引擎全部的讀取；Rust driver 加連線池與 TLS 的實際工作量，所以兩條路的交付時間我無法比較
**Ruled out**: 把官方 Rust Postgres driver 內建進 Node 套件當主要路徑 — 連線、連線池、TLS、遷移會變成 speclink 要長期維護的第二套資料庫層，而且非 Postgres 的宿主仍然需要自訂儲存的契約
**Open**: 使用者是否同意這個方向；用 JS 寫官方的 Postgres 實作是否牴觸先前「不用 TypeScript 重寫儲存 driver」的裁決（理由是否仍成立）；讀取一致性；文件識別用封閉的 9 種或開放的邏輯鍵；既有 31 方法 JS 儲存的退場；D2–D10 不變

### Round 6 — interview (2026-11-10)

**Focus**: 官方的 Postgres 實作是否還需要；儲存媒介是否整個交給宿主
**Position**: 立場改變：第五輪補的「官方 Postgres 實作」撤回，儲存媒介整個交給宿主；speclink 只負責契約與合格測試。
- 改變的原因：使用者指出其他宿主不一定用 Postgres，而已經有自己資料存取層（ORM、連線池、遷移工具）的宿主也用不到官方實作。官方實作只服務「用 Postgres 又沒有資料存取層」的宿主，這種宿主很少
- speclink 負責的四件事：三個函式的契約與它的保證（一次提交全部成功或全部不做、版本對不上就整筆拒絕、事件與寫入同一次提交）；放進 npm 的合格測試；一份記憶體版的參考實作（給合格測試自我驗證，也給宿主寫測試用）；本機模式的檔案實作（這是 speclink 自己的產品形態）
- 宿主負責的事：用什麼媒介、資料表長什麼樣、連線與連線池、遷移、要不要保存每個版本的內容、事件送去哪
- Postgres 的寫法放在文件當範例，不當成要維護的套件
- 「媒介由宿主決定」要成立，前提是合格測試夠嚴格；否則會變成每個宿主的正確性都不一樣而且沒人知道
- S1 因此從需求清單移除：連線設定、schema、連線池、遷移、保存完整歷史、多實例保證都變成宿主自己的事，speclink 只在契約裡寫明版本號的型別與「提交必須可序列化」這類要求
**Ruled out**: 官方的 Postgres 實作當成 speclink 維護的套件 — 只服務一種資料庫，而且有資料存取層的宿主用不到；S1（Node SDK 開放官方 Postgres driver） — 儲存媒介改由宿主負責
**Open**: 使用者確認 D1 定案；示範用官方 server 的三個 Rust driver 是否繼續維護；讀取一致性；文件識別用封閉的 9 種或開放的邏輯鍵；既有 31 方法 JS 儲存的退場；D2–D10 不變

### Round 7 — interview (2026-11-10)

**Focus**: D1 定案；官方 server 的 Postgres driver 去留
**Position**: D1 定案：儲存媒介由宿主負責，宿主實作三個函式（列出、讀取、提交）；speclink 提供契約、合格測試、記憶體版參考實作，並保留本機模式的檔案實作。
- 契約的三項保證：一次提交全部成功或全部不做；版本對不上就整筆拒絕；事件與寫入在同一次提交裡
- 使用者補充：官方 server 的 Postgres driver 也移除，改用同一份三函式契約重新實作，定位是給使用者參考的範例
- 推論：官方 server 是 Rust，所以契約要有 Rust 的定義，Node 的三個函式是它的對映；合格測試只有一份，Rust 實作與 JS 實作都跑它
- 需求清單的變動：S1 移除；S9 從最後一組變成第一步；S4、S8、S6 由這個形狀帶出解法（事件在提交時交給宿主、用到才讀、匯入匯出建在三個函式上）
- 使用者對 D2 的提問：以前只有存取金鑰，現在有人用 OAuth，希望兩種都能走，並問「入口」是不是這個意思。這題把 D2（入口）與 D6（登入）連在一起，下一輪一起處理
**Ruled out**: 保留 TeamStore 版的官方 Postgres driver — 使用者裁定移除，改以三函式契約重寫成參考範例
**Open**: D2 與 D6 合併：入口的形狀、登入由誰做、每個請求的身分怎麼給；官方 server 的 SQLite 與 serverfs driver 是否同樣改寫；三個函式是否足以供應引擎全部的讀取（未驗證）；讀取一致性；文件識別用封閉的 9 種或開放的邏輯鍵；既有 31 方法 JS 儲存的退場；D3–D5、D7–D10 不變

### Round 8 — interview (2026-11-10)

**Focus**: 官方 server 其餘兩個 driver 的去留；登入由誰負責；入口除了身分還要什麼
**Position**: 官方 driver 全部不再提供、改成範例；OAuth 與登入的設計整個交給宿主，speclink 不管；入口的輸入分成「建立時給一次」與「每次呼叫給」兩組。
- 使用者裁定：SQLite 與 serverfs 的官方 driver 不再提供，可以當範例留著。加上第七輪的 Postgres，三個官方 driver 都改成三函式契約的範例
- 使用者裁定：OAuth 這類登入設計都交給宿主，speclink 不應該管。發 token、驗 token、支援哪些登入方式都是宿主的事
- 入口與登入是兩件事：登入是使用者怎麼拿到 token；入口是宿主驗完 token 後，把「請求加已驗證的身分」交給 speclink 處理的函式。線上不論哪種登入都只帶一個 token
- 入口有兩個門、同一條處理路徑：給宿主自己的 Agent 工具用的（收 argv 與 stdin），以及給 CLI 與 Desktop 用的（收一個 HTTP 請求）
- 建立時給一次：儲存（三個函式）；選填的執行前檢查；選填的事件接收；列出「這個人可以用哪些範圍」的函式；上限設定（例如匯入大小）
- 每次呼叫給：身分（唯一且改名後不變的識別，加顯示名稱）；範圍（哪一套規格）；這個人的能力開關（給握手回應用，例如能不能改流程政策）
- 入口自己做：檢查 API 版本、把請求轉成動詞、執行前檢查、引擎、提交、把結果與錯誤轉成線上格式、握手回應、依範圍版本號回答「有沒有變」的輪詢
- 入口不做：登入端點、帳號與成員、管理介面、備份
- 身分由宿主的程式在呼叫時另外給，不放在動詞參數裡。先前否決「dispatch 參數帶 actor」的理由是模型可以偽造身分；身分不經過模型時這個理由不成立
**Ruled out**: speclink 定義或實作 OAuth 伺服器端的任何部分 — 使用者裁定登入設計屬於宿主；保留 SQLite 與 serverfs 的官方 driver — 使用者裁定只留範例
**Open**: 使用者確認入口的輸入分工；用戶端（CLI 與 Desktop）怎麼啟動宿主的登入——speclink 不管 OAuth 之後，用戶端是只收 token，還是仍要有通用的登入流程（留到 D6）；推送給已連線用戶端的事件在多個宿主實例下怎麼送；三個函式是否足以供應引擎全部的讀取（未驗證）；讀取一致性；文件識別；31 方法 JS 儲存的退場；D3–D5、D7–D10 不變

### Round 9 — interview (2026-11-10)

**Focus**: 入口的用途是什麼；用戶端怎麼啟動宿主的登入
**Position**: D2 定案：入口的輸入照第八輪的分工，推送事件預設關閉、只做輪詢；用戶端的登入使用者傾向內建通用的瀏覽器登入流程。
- 使用者回覆入口的分工「看起來合理」，但無法想像用途，要求說明
- 用途：原版 CLI 與 Speclink Desktop 連到 server 時會送出 47 種請求（speclink-server 的 app.rs 在專案路徑下註冊 47 條路由），每一種有固定的網址、請求格式、回應格式與錯誤碼。入口把這 47 種格式收在 speclink 這一邊；宿主只寫一條路由，驗完 token 後把「請求加身分」交給入口
- 沒有入口時，宿主要自己實作並維護這 47 種端點，而且要跟著 speclink 改版；這等於先前否決過的「宿主自行實作動詞契約」
- 第二個門給宿主自己的 Agent 工具：收動詞與身分，不經 HTTP，進同一條處理路徑
- 用戶端登入的兩種做法：只收 token（使用者從宿主拿到 token 再貼進來）；或內建通用的瀏覽器登入流程（開瀏覽器、PKCE、回到本機埠），網址由宿主提供。使用者認為後者比較符合現代化的需求
- 後者與「speclink 不管 OAuth」不衝突的界線：speclink 的用戶端只實作公開標準的用戶端那一半（RFC 8252 給桌面程式的做法、RFC 7636 PKCE），不定義帳號、不發 token、不驗 token；宿主用哪個身分來源、登入頁長什麼樣都由宿主決定
**Ruled out**: 宿主自行實作 47 種端點 — 格式要逐一對上並跟著改版，先前的討論已否決宿主重做動詞契約
**Open**: 使用者確認用戶端內建瀏覽器登入的界線；用戶端怎麼在登入前知道宿主支援哪些登入方式與網址（探索端點的格式，優先沿用公開標準）；CLI 與 Desktop 的登入迴圈今天是兩份，要不要收成一份；裝置登入是否保留；D3–D5、D7–D10 不變

### Round 10 — interview (2026-11-10)

**Focus**: 入口的做法是否成立；接著 D3：宿主的 Agent 工具那個門要怎麼涵蓋全部動詞
**Position**: 使用者確認入口的做法是對的；D3 建議把「動詞文字轉成引擎命令」收成一份，CLI 與 SDK 共用。
- 使用者的理由：沒有入口的話，使用者要自己理解 CLI 的 47 種請求，實作起來很累
- 用戶端登入：使用者上一輪傾向內建通用的瀏覽器登入流程，這一輪沒有反對；先記為方向，探索端點的格式與裝置登入的去留留到登入節點定案
- D3 的現況：引擎的 Command 有 43 種。把輸入轉成 Command 的程式今天有三份——CLI 用 clap 解析動詞文字（涵蓋 33 種）、Node SDK 手寫解析（9 種，speclink-node/src/lib.rs:326）、官方 server 把 HTTP 請求轉成 Command（api/routes.rs）
- clap 只有 speclink-cli 與 speclink-server 兩個 crate 相依，引擎沒有；所以 Node SDK 當初才另外手寫一份
- 建議：把「動詞文字轉成 Command」抽成共用的一份，CLI 與 SDK 都用它。SDK 一次得到全部動詞，之後新增動詞不會漏
- 需要本機程式碼的資訊（改了哪些檔、檔案指紋、commit）不由引擎收集，當成動詞的輸入送進來；審查與驗證的蓋章今天已經是這樣
- 發現的缺口：流程設定的讀寫不在 43 種 Command 裡。CLI 在本機直接寫檔，在遠端直接送 PUT /config。宿主需要它經過同一條處理路徑，執行前檢查才看得到改了哪些鍵
- 未驗證：CLI 的 clap 定義和「本機或遠端」的分流寫在一起，抽出來的工作量我沒有量過
**Ruled out**: 在 Node SDK 的手寫解析裡逐個補動詞 — 第三份解析會持續落後，每新增一個動詞要改兩處
**Open**: 使用者確認 D3；流程設定的讀寫是否新增為引擎命令；匯入匯出、看板順序、evidence 讀取這類今天只有 HTTP 端點的操作，Agent 工具那個門要不要也能用；D4、D5、D6（登入的探索端點與裝置登入）、D7–D10 不變

### Round 11 — interview (2026-11-10)

**Focus**: D3 定案；「Node SDK 是不是就是引擎」的釐清
**Position**: D3 定案：把「動詞文字轉成引擎命令」抽成共用的一份，CLI 與 SDK 都用它；流程設定的讀寫新增為引擎命令。
- 使用者同意做法甲
- 使用者的疑問：以為 Node SDK 就是引擎。釐清：引擎是 Rust 的 speclink-core，有 43 種命令；Node SDK 是「引擎加一層給 Node 用的轉接」，套件名稱是 @speclink/engine，所以這個理解很自然
- 轉接層今天有三樣：動詞文字的手寫解析（只認 9 個）、JS 儲存的橋接、技能的渲染。S2 的限制在轉接層，不在引擎
- 照 D1 到 D3 的定案，Node SDK 之後的內容是：引擎，加上主機套件（入口的兩個門、三個函式的儲存接口、執行前檢查、事件），加上技能渲染
**Open**: D4 執行前檢查與事件放哪一層；D5 效能是否還需要單獨處理（用到才讀之後）；D6 登入的探索端點與裝置登入；D7 線上欄位；D8 技能；D9 遠端缺口；D10 分刀

### Round 12 — interview (2026-11-10)

**Focus**: D4 的執行前檢查是不是掛鉤；掛鉤多一點是否比較有彈性
**Position**: 立場調整：第十一輪後我建議「只有執行前檢查，事件只走提交」；使用者問掛鉤多一點是否較有彈性，我改為「掛鉤的形狀留給日後擴充，先提供三個點，並訂一條規則：掛鉤可以拒絕或旁觀，不能改內容」。
- 是掛鉤。一個命令的處理路徑有三個宿主能介入的點：引擎之前（執行前檢查）、提交（儲存的第三個函式，看得到全部寫入與事件，丟出錯誤就整筆不寫）、提交之後
- 提交之後的點我原本不給，理由是「寫入成功、通知掉了」。調整為：給一個只能旁觀的掛鉤，明寫它是盡力而為、不是可靠通道；需要「至少送達一次」的宿主仍然在提交裡把事件寫進自己的表。好處是最小的宿主不必先建事件表就能做即時通知
- 掛鉤分三類：把關（可以拒絕）、旁觀（只收通知）、改寫（修改輸入或輸出）。前兩類提供，第三類不提供
- 不提供改寫類的理由：CLI 與 Speclink Desktop 必須看得懂線上的每個回應，宿主改寫內容後原版用戶端就可能看不懂；這和「宿主不加新動詞」是同一個理由
- 掛鉤用一個具名的集合傳入（每個都是選填），日後新增掛鉤不會破壞既有宿主；移除或改簽名才會破壞
- 不做「每個動詞一個掛鉤」：掛鉤收到命令名稱，由宿主自己分辨；這樣掛鉤的數量不會跟著動詞增加
- 需求文件列的用途對照：封存前的額外條件、政策鍵只有特定的人能改、只開放部分階段，都用執行前檢查；即時通知與工單狀態對應用事件
- 還沒有掛鉤能解的用途：宿主想在建立變更或討論的同一次寫入裡順便記下自己的資料（來源、工單號）。這要靠 S7 的自由欄位，放到 D7 談
**Ruled out**: 改寫類的掛鉤 — 會讓原版用戶端看不懂回應；每個動詞各一個掛鉤 — 介面會跟著動詞數量長大
**Open**: 使用者確認 D4 的三個點與「可以拒絕或旁觀、不能改內容」的規則；D5–D10 不變

### Round 13 — interview (2026-11-10)

**Focus**: D4 定案；接著 D5：改成用到才讀之後，效能是否還要另外設計
**Position**: D4 定案：三個介入點（引擎之前、提交、提交之後），規則是掛鉤可以拒絕或旁觀、不能改內容；D5 建議先不設計快取，先量測。
- 補做的查證：逐一看過引擎 Store 介面全部方法的簽名（speclink-core/src/store.rs），操作只有六種——列出、是否存在、讀、寫、刪、搬移，範圍各自限定在進行中的變更、正式規格、已封存的變更、討論這幾類。它們都能對應到列出、讀取、提交三個函式；「三個函式足夠」由此得到較強的依據（看的是簽名，不是每個呼叫端的用法）
- 契約要補的兩個資料：每份文件的最後修改時間（updated_at_secs 會用到）；範圍的版本號（回答「有沒有變」的輪詢與日後的快取都要它，可以放在列出與提交的回傳值裡）
- 封存資料佔約 84%。列出是分類進行的，所以一般動詞不會碰到封存資料；只有瀏覽封存、搜尋討論、溯源這幾類動詞會讀
- 剩下的成本在往返次數：引擎每讀一份文件，就要跨到 JS 再查一次資料庫。對策是讀取函式一次收多個鍵
- 建議：第一個變更做用到才讀與批次讀取，並用真實大小的資料（參考量 165 個已封存變更、31 MB）量一次；量出來慢才加依範圍版本號的快取
**Ruled out**: 現在就設計依版本號的快取 — 用到才讀之後是否還慢沒有數字，先做會是沒有依據的複雜度
**Open**: 使用者確認 D5；讀取一致性（一個動詞讀到一半時別人寫入）要不要處理；D6 登入的探索端點與裝置登入；D7 線上欄位與自由欄位；D8 技能；D9 遠端缺口；D10 分刀

### Round 14 — interview (2026-11-10)

**Focus**: D5 定案；接著 D6：登入前怎麼知道宿主支援哪些登入方式、裝置登入的去留、版本差異的提醒
**Position**: D5 定案：先做用到才讀與批次讀取，量測後才決定要不要快取；純讀取的短暫不一致第一版接受。D6 建議四點如下。
- 探索：用戶端在登入前要先知道宿主支援哪些登入方式。握手要先帶 token（GET /binding），所以握手裡的 authentication 欄位做不到；這個欄位今天永遠是空陣列，也沒有用戶端讀它
- 建議一：入口多回答一個不用登入的路徑，內容來自宿主建立入口時給的設定，宿主不必自己寫這個端點。內容只有兩樣——支援哪些方式（存取金鑰、裝置登入、瀏覽器登入），以及瀏覽器登入的發行者網址
- 建議二：瀏覽器登入其餘的資訊（授權網址、換 token 的網址）沿用公開標準 RFC 8414 的文件格式，由宿主的登入服務提供；speclink 不自己發明這部分
- 建議三：用戶端的預設順序是瀏覽器登入、裝置登入、存取金鑰；宿主沒有宣告時維持今天的行為，舊版 server 不受影響。登入程式收成一份放在 speclink-remote，CLI 與 Speclink Desktop 共用（今天兩邊的登入迴圈分開寫，存取金鑰一個寫檔案、一個寫鑰匙圈）
- 建議四（C2）：握手已經送出 engineVersion，但沒有用戶端讀它。用戶端改為讀它，在 server 的引擎比本機技能舊時顯示一行提醒，不擋操作。注意握手送的是套件版本，本機技能帶的是另一個版號（speclink --version 顯示的 engine 版號），比對哪一個留到設計時定
- 宿主要配合的事：瀏覽器登入會回到本機的隨機埠，宿主的登入服務要接受本機回呼位址帶任意埠（RFC 8252 的做法；條文的確切用字這次沒有查到原文，設計時要再確認）
**Ruled out**: 用握手宣告登入方式 — 握手本身要先登入；speclink 自訂整套登入資訊的格式 — 公開標準已有，自訂會讓宿主多做一份
**Open**: 使用者確認 D6；D7 線上欄位（關卡狀態、commit 回報、自由欄位）與匯入匯出；D8 技能與規格位置；D9 遠端缺口；D10 分刀

### Round 15 — interview (2026-11-10)

**Focus**: D6 的兩個追問——宿主是否要知道自己得實作探索端點；握手後由遠端提供技能下載與安裝是否更好
**Position**: 立場調整兩處：宿主不必知道哪個路徑免登入，驗不出身分就傳空值、由入口自己決定；技能改由遠端提供，我同意這個方向，它把版本落差搬到工程師自己能解決的那一邊。
- 追問一：照第十四輪的設計，探索端點由入口回答，宿主不用實作；但宿主的登入檢查若先擋下沒有 token 的請求，那個路徑就到不了入口。這是宿主容易漏掉的地方
- 調整：宿主對每個請求都呼叫入口；驗得出身分就帶身分，驗不出就帶空值。哪些路徑免登入由入口決定，其餘回未授權。宿主因此不需要知道任何路徑
- 文件仍要有一份「宿主要做的事」清單：實作三個函式並通過合格測試、掛一條路由、驗不出身分時傳空值、要支援瀏覽器登入時讓自己的登入服務提供 RFC 8414 的文件並接受本機回呼
- 追問二的現況：技能檔由工程師本機的 CLI 產生（init_remote 用本機內嵌的範本），版號跟著本機 CLI；本機已有「技能比引擎新就拒絕覆寫」的守門（init.rs 的 refuse_downgrade）
- 由遠端提供的好處：技能文字和宿主的引擎同版；產生哪些技能跟著宿主那邊的流程設定（K2 因此解掉）；整個團隊拿到同一組技能
- 版本落差不會消失，只是換邊：今天是「本機技能比宿主新，要請宿主更新」；改了之後是「本機 CLI 比技能舊，工程師自己更新 CLI」。後者工程師自己就能處理
- 條件：技能由入口用引擎內建的範本渲染，宿主只能選擇提供哪幾支，不能改文字（和掛鉤不能改內容是同一條規則）；用戶端安裝後仍是 checkout 裡的檔案
- 要說清楚的風險：技能是本機 agent 會照著執行的指示，由遠端提供等於遠端能影響本機 agent 的行為。今天的 instructions 輸出已經來自遠端，所以信任邊界沒有變寬，但技能檔會留在 repo 裡，影響更持久
- 連動：第十四輪建議四的提醒方向反過來，改為「本機 CLI 比遠端提供的技能舊時，提醒更新 CLI」
**Ruled out**: 要求宿主自己放行免登入的路徑 — 宿主容易漏掉，改由入口決定
**Open**: 使用者確認 D6（含這兩處調整）；D7 線上欄位與匯入匯出；D8 技能其餘部分（用路徑讀檔改成動詞、規格放在 checkout 外、唯讀引用）；D9 遠端缺口；D10 分刀；D11 使用者新提的問題——引擎改動時如何避免再改一次 SDK、是否每次改動都要本機與遠端一起考慮、要寫進流程設定的哪一條規則

### Round 16 — interview (2026-11-10)

**Focus**: D6 定案；提醒更新 CLI 時，CLI 要不要有更新機制、Speclink Desktop 的畫面怎麼呈現
**Position**: D6 定案（第十四輪四點加第十五輪兩處調整）；CLI 不做自我更新，只做提醒；Desktop 沿用既有的技能檔提示卡，把比對基準換成遠端提供的技能版本。
- 使用者另提：全部定案後要討論 README 文件的更新，列為 D12
- CLI 不做自我更新的理由：CLI 有四種安裝通路（安裝腳本、Homebrew、npm、Desktop 佈署），自我更新要分辨是哪一種；而且 Desktop 啟動時偵測到 CLI 版本不符會自動重新佈署自己那一版（desktop-app 規格「安裝 CLI 指令到 PATH」），會把自我更新的結果蓋回去
- CLI 的提醒：本機 CLI 比遠端提供的技能舊時，在 stderr 印一行，寫明兩邊版號與「用你安裝 Speclink 的方式更新」；同一台 server 一天最多提醒一次，避免 agent 每個動詞都讀到；不擋操作
- 既有守門要調整：今天 CLI 遇到技能檔比引擎新會拒絕覆寫（refuse_downgrade）。連到遠端的專案，技能版本改為跟著遠端，所以安裝比本機 CLI 新的技能是正常情況，改成安裝並提醒
- Desktop 的現況：本機專案已有技能檔過期探測（缺失、過期、較新、現版、無法判定五態）與專案欄底部的琥珀提示卡；「較新」態已經是「技能檔比 Speclink 新」加「更新 Speclink」按鈕，會開到應用程式設定的更新頁。規格明寫遠端專案不觸發探測
- Desktop 的建議：遠端專案也啟用探測，比對兩件事——專案裡的技能檔對遠端提供的版本（過期顯示「更新技能檔」、缺失顯示「安裝技能檔」，動作改為從遠端下載）；本機 Speclink 對遠端的技能版本（Speclink 較舊時沿用「較新」態的呈現與「更新 Speclink」按鈕）。確認框多一列「Server 提供的技能版本」
- Desktop 在 macOS、Windows、Linux AppImage 會佈署與自己同版的 CLI，所以用 Desktop 的工程師按「更新 Speclink」後 CLI 一起更新；只用 CLI 的工程師靠那一行提醒
**Ruled out**: CLI 自我更新指令 — 四種安裝通路各自不同，且會和 Desktop 的自動佈署互相覆蓋
**Open**: 使用者確認這一輪的 CLI 與 Desktop 做法；D7 線上欄位、自由欄位與匯入匯出；D8 技能其餘部分；D9 遠端缺口；D10 分刀；D11 引擎改動如何避免再改 SDK 與流程設定的規則；D12 README 文件更新

### Round 17 — interview (2026-11-10)

**Focus**: 第十六輪定案；接著 D7：線上要加的欄位（關卡狀態、程式碼落地的回報、自由欄位）與匯入匯出
**Position**: 第十六輪定案：CLI 只提醒不自我更新，Desktop 沿用技能檔提示卡、基準換成遠端提供的技能版本。D7 建議四點，原則是「speclink 定義的事實用正式欄位，宿主自己的資料用自由欄位，兩者引擎都只存不解讀」。
- 關卡狀態（P1）：變更的清單與單筆回應加選填欄位，審查與驗證各自帶狀態（未開始、進行中、已通過）、誰蓋的章、何時、幾輪。「蓋章後檔案又改過」要比對本機檔案，遠端算不出來，維持由有 checkout 的用戶端自己判斷
- 關卡事件（P1）：引擎已經發出審查與驗證的六種事件，官方 server 的分類把它們歸成 unknown 且不帶變更名稱（api/events.rs:36）。改為歸到該變更，用戶端就會重新讀取。宿主要的有型別事件已由 D4 的提交取得，線上不需要新的事件種類
- 程式碼落地（P2）：三個選填的正式欄位——蓋章時的 commit（蓋章請求多帶一項）、變更的分支名稱（開工時記下）、程式碼落地的 commit（合併後回報，需要一個新動詞）。引擎只存，不附帶任何規則；「commit 是否已在正式分支」由宿主在執行前檢查裡自己判斷
- 現況補充：task done 已經會帶當下的 headCommit 並存進 evidence，蓋章只送檔案指紋
- 自由欄位（S7）：變更與討論各有一組宿主自訂的鍵值，引擎只存不解讀；有通用的寫入與移除動詞，原版 CLI 也能用；清單與單筆回應會帶出來；宿主的程式可以在呼叫入口時附上（不經過模型），和建立變更或討論在同一次寫入——這解掉 D4 留下的「建立時順便記下來源」。存放在既有的 metadata 文字裡的一個專屬區塊，不新增文件種類
- 匯入匯出（S6）：建在三個函式上，兩個門都能用；大小上限由宿主在建立入口時設定；壓縮屬於宿主的 HTTP 層，speclink 不做；匯入維持只能進空的範圍
**Ruled out**: 分支與落地 commit 放進自由欄位並約定鍵名 — 這兩項是 speclink 自己的技能要寫入的事實，放進宿主的自由欄位會讓兩種鍵混在一起；speclink 內建「commit 必須在正式分支才能封存」的規則 — 合併規則屬於宿主；speclink 做壓縮 — 宿主的路由層已有
**Open**: 使用者確認 D7；自由欄位的鍵名規則、數量與大小上限（設計時定）；舊版用戶端遇到新欄位是否照常運作（設計時查證）；D8 技能其餘部分；D9 遠端缺口；D10 分刀；D11 引擎改動與 SDK；D12 README

### Round 18 — interview (2026-11-10)

**Focus**: D7 定案；接著 D8：技能的其餘部分（K1 用路徑讀檔、K3 規格放在 checkout 之外、K4 唯讀引用）
**Position**: D7 定案（四點）。D8 建議：K1 把既有的規則做完並加嚴守門；K2 已由 D6 解掉；K3、K4 延後。
- K1 的現況：「技能要經過動詞讀文件、不用檔案路徑」已經是規則，也有守門測試（speclink-core/tests/it/skill_verbization.rs）。守門只抓同一行出現 read、glob、open 加規格路徑的寫法，所以 compare against、cat、Check for、git add、git log、rewrite in place，以及 discuss 用清單回傳的路徑去讀 spec.md，都沒被抓到
- K1 建議一：規則改成「技能讀寫規格文件一律經過動詞」，守門跟著加嚴到涵蓋上述寫法。這樣同一份技能文字在本機、遠端、沒有任何本機檔案三種情況都能用，讀寫規格這件事不需要分支
- K1 建議二：真正需要分支的只有「對規格檔做 git 操作」的步驟（commit 技能把規格資料夾加進暫存、worktree 開工前把變更資料夾加進 git、封存後提示提交）。這些步驟只在規格是這個 checkout 裡的檔案時才有意義，由產生技能時決定要不要放進去；遠端提供的技能不含這些步驟
- K1 要補的動詞或欄位：讀 evidence（server 已有端點，CLI 沒有動詞）；apply 判斷變更是否閒置太久，今天靠 git log 與本機的 .meta.yaml，改用變更的建立時間與最後修改時間
- K1 要修的一處：tool-call 形式把交棒給其他技能的句子渲染成像動詞的寫法（例如 speclink worktree-merge），agent 會把它當成工具呼叫
- K2：遠端專案的技能由遠端提供、依那邊的流程設定產生，worktree 技能不產生的問題已由 D6 解掉
- K3 的現況：spec_dir 沒有任何檢查，絕對路徑與 ../ 會被接受是意外的結果，沒有測試；解析順序只有設定檔與預設值，沒有旗標或環境變數
- K3、K4 建議延後：內嵌宿主的情境用不到；它們服務的是「把規格隔到另一個資料夾或 repo」這種形態，需求文件自己也排在最後一組。規格放在另一個 git repo 時，技能裡的 git 步驟要對另一個 repo 操作，這是另一個設計題
**Ruled out**: 在每支技能裡加「規格在不在這個 checkout」的執行期判斷 — 讀寫改走動詞後不需要，只剩 git 步驟要分，而那在產生技能時就能決定
**Open**: 使用者確認 D8；K3、K4 延後時是否仍要先把 spec_dir 的路徑規則定下來並補測試；D9 遠端缺口；D10 分刀；D11 引擎改動與 SDK；D12 README

### Round 19 — interview (2026-11-10)

**Focus**: D8 定案；接著 D9：遠端模式原有的五個缺口要補哪幾項，以及文件的識別方式
**Position**: D8 定案：K1 把「技能讀寫規格一律經過動詞」做完並加嚴守門，K2 視為已由 D6 解掉，K3 與 K4 延後。D9 建議補一項、兩項隨前面的定案帶出、兩項延後，並把文件的識別改成字串鍵。
- 使用者的條件：K3 與 K4 延後可以，但之後建立文件時要明確寫出這些缺口。同一條件我套用到 D9 延後的項目
- change rank：建議補。propose 與 ingest 技能都會呼叫它（propose.md:436、ingest.md:254），遠端今天明寫拒絕。非工程角色做討論與提案是第一階段的用途，所以第一階段就會踩到
- trace 與 plan --strict-overlap：引擎的 Command 已有 Trace 與 PlanStrict。照 D1（用到才讀的儲存）與 D3（全部命令共用一份轉譯）之後，宿主的 Agent 那個門應該可以直接用；給 CLI 的那個門要各補一條路徑，trace 技能裡用路徑讀封存 proposal 的步驟由 K1 的規則處理。未驗證：CLI 註明 trace 是從本機規格資料夾組資料，能否直接跑在三個函式上要到設計時查
- 自訂產出流程：延後。今天 server 沒有地方存它，只有內建的兩種可選
- 手冊產生：延後。手冊頁今天由技能直接寫檔，遠端沒有地方存；導覽模式仍可用
- 文件的識別：建議三個函式用字串鍵（引擎訂命名規則，例如進行中的變更、正式規格、已封存、討論各有前綴），儲存不需要認得文件種類。今天 TeamStore 是封閉的 9 種，新增一種要改契約與每個實作。改成字串鍵後，之後補自訂產出流程或手冊頁都不用動宿主的儲存
- 先前否決「把路徑當成跨媒介的身分」針對的是實體檔案路徑；字串鍵是邏輯名稱，不是檔案位置，而且官方的 Postgres 實作內部本來就把文件種類編成一個字串欄位
**Ruled out**: 三個函式沿用封閉的文件種類清單 — 每新增一種文件，全部宿主的儲存都要跟著改
**Open**: 使用者確認 D9；遠端的看板順序今天是另一份獨立的文件，change rank 要寫到哪裡（設計時定）；D10 分刀與順序；D11 引擎改動與 SDK；D12 README

### Round 20 — interview (2026-11-10)

**Focus**: D9 的清單是否漏了 evidence（.evidence.json）
**Position**: 漏了。第十八輪只在 K1 底下用一行帶過「補讀 evidence 的動詞」，它應該是 D9 清單上獨立的一項，而且範圍比一個動詞大。
- 遠端今天做得到的：task done 會把改過的檔案與當下的 commit 送到 server；審查與驗證算範圍時會從 server 讀回（speclink-cli/src/verbs/station.rs:501 的 remote_evidence_files）；用戶端函式庫有讀取的方法（speclink-remote/src/client.rs:298）
- 缺的第一樣：CLI 沒有讀 evidence 的動詞，所以技能拿不到它
- 缺的第二樣：五支技能直接用路徑讀這個檔——commit（commit.md:24，用它決定哪些原始檔屬於這個變更）、worktree 開工前（apply-worktree-pre.md:113，cat）、worktree 收尾（apply-worktree-post.md:12，依它決定要暫存哪些檔）、archive（archive.md:146，零 evidence 的提示）、improve（improve.md:52，讀已封存變更的 evidence）
- 本機與遠端的一個差別要寫清楚：本機的 evidence 是變更資料夾裡的檔案，會跟著 commit；遠端沒有這個檔，commit 技能裡「把它一起提交」的步驟不適用
- 建議：新增讀 evidence 的動詞（進行中的變更與已封存的變更都能讀），五支技能改用它；這一項和 K1 同一刀做
- D9 清單因此是六項：change rank 補；evidence 的讀取補；trace 與 plan --strict-overlap 隨前面的定案帶出；自訂產出流程與手冊產生延後
**Ruled out**: 把 evidence 的讀取只當成 K1 底下的一個小項 — 它牽涉五支技能與一個新動詞，要獨立列出才不會漏
**Open**: 使用者確認 D9（六項加上文件用字串鍵）；D10 分刀與順序；D11 引擎改動與 SDK；D12 README

### Round 21 — interview (2026-11-10)

**Focus**: D9 定案；接著 D10：定案的內容分成幾個變更、照什麼順序做
**Position**: D9 定案：六項（change rank 與 evidence 讀取補上、trace 與 plan --strict-overlap 隨前面的定案帶出、自訂產出流程與手冊產生延後並在文件寫明），文件用字串鍵。D10 建議分四個階段、十三個變更。
- 階段 A 的目標：宿主能內嵌、非工程角色能經 Agent 做討論與提案、工程師能用 CLI 與 Desktop 連線
- A1 store-port-contract：三個函式的契約（Rust 定義、字串鍵、文件版本、範圍版本、最後修改時間）、合格測試、記憶體版參考實作；純新增，不接線
- A2 engine-on-store-port：引擎經這個契約讀寫——用到才讀、批次讀取、寫入收成一次提交並帶事件；用真實大小的資料量測一次（D5）；示範用的 server 先透過轉接沿用現有的 driver，維持可用
- A3 shared-verb-parsing：把「動詞文字轉成引擎命令」抽成 CLI 與 SDK 共用的一份；流程設定的讀寫新增為引擎命令。和 A1、A2 無相依，可以並行
- A4 host-kit-agent-door：主機套件與給宿主 Agent 工具的門——每次呼叫帶身分與範圍、三個掛鉤點；Node SDK 開放它並附上 JS 版的契約橋接與 npm 裡的合格測試
- A5 host-kit-client-door：給 CLI 與 Desktop 的門——把請求轉命令、結果轉線上格式、錯誤對應、握手、輪詢從官方 server 搬到主機套件；身分可為空值；範圍清單；登入方式的宣告路徑；示範用的 server 改用它
- A6 client-browser-login：用戶端的瀏覽器登入、登入程式收成一份、Desktop「新增專案」的畫面。可以和 A5 並行
- A7 skills-verbs-first-wave：discuss、propose、ingest 改成全部經過動詞；change rank 在遠端可用。可以和 A4 並行
- 階段 B 的目標：工程師的完整流程。B1 skills-verbs-second-wave（archive、commit、worktree 兩支、apply、improve 改走動詞；讀 evidence 的動詞；trace 在遠端可用；git 步驟依產生條件放入；守門加嚴；tool-call 的交棒句修正）。B2 remote-skill-delivery（技能由遠端提供、用戶端安裝與更新、降級守門調整、版本提醒、Desktop 的提示卡）
- 階段 C 的目標：追蹤與封存。C1 station-status-and-landing（關卡狀態欄位、事件歸類修正、蓋章帶 commit、分支名稱、落地 commit 的動詞）。C2 host-free-fields（自由欄位、通用寫入動詞、呼叫入口時附上）。C3 import-export-on-port（匯入匯出建在契約上、上限由宿主設定）
- 階段 D 收尾：D1 retire-teamstore-and-drivers（SQLite、Postgres、serverfs 改寫成契約的範例；移除 TeamStore 與既有的 31 方法 JS 儲存）
- 每個變更完成後，示範用的 server 與既有測試都要維持可用，所以移除舊東西排在最後
- 既有的 31 方法 JS 儲存在 A4 的文件標示為即將移除，到階段 D 才真的移除
**Ruled out**: 先移除舊的再做新的 — 中間會有一段時間示範用的 server 與測試不能跑
**Open**: 使用者確認分法與順序；每個階段結束就發版，或全部做完才發；進行中的變更 add-pi-cursor-tools 會改 node-sdk 的渲染 API，A4 要排在它之後；開啟中的討論 agent-tool-sdk-layering 和 A3、A4 重疊，要結論或併入；D11 引擎改動與 SDK；D12 README

### Round 22 — interview (2026-11-10)

**Focus**: D10 定案；接著 D11：引擎改動時如何避免再改一次 SDK，是否每次都要本機與遠端一起考慮，要寫進流程設定的哪一條規則
**Position**: D10 定案：四個階段十三個變更，全部完成才發版。D11 建議兩層：先靠結構與守門讓「漏掉」變成編譯或測試失敗，再加一條提案規則要求每個變更寫明三種形態的結果。
- 結構：這個系列做完後，引擎的新命令經共用的動詞轉譯（A3）自動到達 CLI 本機路徑與宿主的 Agent 那個門；新的文件種類只是多一個鍵的前綴，宿主的儲存不用改。大部分的二次改動因此消失
- 仍然要多做一步的四個地方：給 CLI 與 Desktop 的門（新命令要有線上的請求格式）、技能文字、Desktop 的畫面、任何直接碰本機檔案或 git 的程式
- 守門一（放在 A5）：主機套件用一個涵蓋全部引擎命令的對照，每個命令必須選一個——有線上格式，或明寫不上線與理由。引擎新增命令而沒處理時編譯就不過
- 既有的守門沿用：CLI 的模式對照表已經要求每個動詞宣告本機或遠端（verb-contract 規格）；remote_verb_parity.rs；技能的 skill_verbization（B1 加嚴）
- 流程設定的現況：context 有一條「遠端路徑不可沿用純本地假設」；rules.design 第二條已要求同一對外契約同時落本機與遠端時指明唯一實作落點、不得平行實作兩套
- 建議的規則落點一，rules.proposal 新增一條：新增或變更引擎命令、動詞、文件種類或技能步驟時，逐一寫明三種形態下的結果（本機 repo、連到遠端的 checkout、沒有本機規格檔的內嵌宿主）——可用、不適用並寫理由、或這次不做並寫成已知缺口。放在提案是因為這是最早也最便宜的時間點，而且「不做」必須留下文字
- 建議的規則落點二，改寫 rules.design 既有的第二條：把內嵌宿主加進去，並補上「引擎的功能不得直接讀寫檔案或呼叫 git；文件經儲存介面，本機事實由呼叫端收集後當成輸入」
- context 的專案說明提到 TeamStore 與三種 driver，等階段 D 的收尾變更完成時一起改
- 規則檔的修改走 /speclink-config，不在這場討論裡動手
**Ruled out**: 只靠規則提醒、不加守門 — 規則靠人記得，守門讓漏掉的情況直接失敗；要求每個變更都把三種形態做齊 — 成本太高，允許寫明「不適用」或「這次不做」
**Open**: 使用者確認 D11 的做法與規則文字；提案規則要現在就加（讓十三個提案都照它寫）或等系列完成再加；D12 README；開啟中的討論 agent-tool-sdk-layering 的處置

### Round 23 — interview (2026-11-10)

**Focus**: D11 定案；使用者新增兩題——三個設定檔在遠端模式下的現況（D13），以及這整塊功能的統一用詞（D14）
**Position**: D11 定案並已寫入流程設定；D13 是現況說明，沒有新的決定；D14 建議把用詞分成「使用者看到的」與「整合者看到的」兩組，並訂一條分辨規則。
- D11 的落實：使用者要求現在就加。已用 workflow-config rules 寫入兩處——rules.proposal 新增第七條（三種形態逐一寫明結果）；rules.design 第二條改寫（加入內嵌宿主與 Node SDK，並補上本機事實由呼叫端收集）。第二條的補句寫成「新增的引擎功能」，因為既有的 drift 與 archive 本來就會呼叫 git，另一條設計規則也還在規範它
- 順序調整：README（D12）會用到統一後的用詞，所以排在 D14 之後
- D13 .speclink.yaml：留在 checkout 裡並進版控。欄位是 spec_dir、tools、remote（url 與 repo）。remote 區段存在就代表這個 checkout 是遠端專案；url 也可以由環境變數 SPECLINK_STORE_URL 給。token 不在檔案裡，放系統鑰匙圈或環境變數
- D13 config.yaml：遠端專案的 checkout 裡沒有這個檔，內容存在 Server。用 workflow-config show 讀；寫入是整份覆寫並帶預期版本，要 editor 角色
- D13 .meta.yaml：每個變更的 metadata，遠端時是 Server 上的一份文件；線上沒有直接讀它的路徑，內容以欄位的形式出現在變更的清單與單筆回應裡。看板順序在遠端是另一份獨立的文件
- D13 另有本機的工作資料夾 .speclink/（不進版控）：touched、審查範圍的基準、產生的工具檔、給 agent 讀的規格唯讀副本
- D13 受前面定案影響的地方：config.yaml 的讀寫變成引擎命令、執行前檢查看得到改了哪些鍵（D3、D4）；技能的組合跟著 Server 上的 config.yaml（D6）；.meta.yaml 多一個自由欄位的區塊與分支、commit 欄位（D7）；.speclink.yaml 的角色不變
- D14 的問題來源：「遠端」說的是規格在哪裡，「SDK」說的是用什麼工具蓋出 Server，兩者不是同義詞，但常被拿來指同一塊功能。我自己在這場討論也漂移過：把 SDK 裡負責入口與掛鉤的那一層叫「主機套件」，它和「宿主」是同一個英文 host，容易混
- D14 建議的詞：使用者看到的——本機專案、遠端專案、Server；整合者看到的——宿主、內嵌、SDK、宿主套件、儲存接口、入口。分辨規則：講規格在哪裡用「本機」「遠端」；講怎麼蓋出 Server 用「SDK」「內嵌」「宿主」
**Ruled out**: 用「SDK」統稱整塊遠端功能 — SDK 是工具的名字，用官方 server 的遠端專案完全沒用到 SDK；「主機套件」這個叫法 — 主機讀起來像一台機器，改稱「宿主套件」
**Open**: 使用者確認 D14 的用詞表；LANGUAGE.md 既有的「儲存後端」詞條在三個官方 driver 改成範例後要跟著改；D12 README；開啟中的討論 agent-tool-sdk-layering 的處置

### Round 24 — interview (2026-11-10)

**Focus**: D14 定案；接著 D12：README 與使用者文件怎麼更新
**Position**: D14 定案：用詞分使用者與整合者兩組，分辨規則是「講規格在哪用本機與遠端，講怎麼蓋出 Server 用 SDK、內嵌、宿主」。D12 建議新增一個放在最後的文件變更，五點如下。
- README 的現況：開頭寫「兩條部署路徑 Local Repo 與 Remote Store」；有一節「自架 server」把 speclink-server 稱為官方的參考實作、「讓你開箱即用，或拿來試遠端功能」，同一節最後一段才提到可以用引擎自己寫 server；「目前狀態」列出三種 TeamStore、單節點 server 與後台、server 營運
- 正式規格 user-documentation 已有一條「官方 server 定位為參考實作而非唯一路徑」，所以改定位要寫 delta
- 建議一（時機）：十三個變更期間不動使用者文件，每個變更在提案的 Impact 寫明文件影響；最後新增第十四個變更 docs-embedded-host（階段 D）一次更新，緊接著發版。理由是文件規格要求只寫已驗證可走的入口，而且全部完成才發版
- 建議二（開頭的定位）：改用 D14 的用詞——本機專案與遠端專案；遠端專案的 Server 有兩種來源，主要的是把引擎內嵌進自己的系統，另一種是官方的示範 server
- 建議三（章節）：「自架 server」拆成兩節。前一節「把 Speclink 內嵌進你的系統」寫宿主要做的四件事並連到 SDK 文件；後一節「試用官方 server」寫明它是示範與試用用、正式使用請內嵌
- 建議四（狀態與缺口）：「目前狀態」三份清單照新的結構改寫；延後的項目（規格放在 checkout 之外、唯讀引用另一套規格、遠端的自訂產出流程、遠端的手冊產生）在專案能力狀態與路線圖逐項列為已知缺口，README 用一行連過去
- 建議五（用詞）：README、docs 與 LANGUAGE.md 在同一個變更裡一起改，舊說法的存量同批清掉（詞彙守門掃整個文件面），中英兩版同步
- 受影響的文件：README 兩版、專案能力狀態、路線圖、Node SDK 文件（改寫成宿主套件、入口、掛鉤、儲存接口、合格測試）、Server 儲存後端（改寫成儲存接口的範例）、Remote 入門、設定說明、動詞與旗標契約
**Ruled out**: 每個變更各自更新 README — 中途的 README 會描述還沒發版的功能，和文件規格「只寫已驗證可走的入口」衝突
**Open**: 使用者確認 D12 的五點與官方 server 的定位文字；Server 部署、Server 備份與還原這兩份營運文件在「示範用」的定位下要保留、加註或刪減；開啟中的討論 agent-tool-sdk-layering 的處置；結論草稿

### Round 25 — interview (2026-11-10)

**Focus**: D12 定案；接著處理開啟中的討論 agent-tool-sdk-layering 和這場的重疊
**Position**: D12 定案：新增第十四個變更 docs-embedded-host，五點照第二十四輪；官方 server 的定位句為「示範與試用用，正式使用請把引擎內嵌進自己的系統」；兩份營運文件保留並加註。對 agent-tool-sdk-layering 建議以「併入本討論」結論後封存，不另外轉出變更。
- 那場討論留下的待決，對照這場的定案：工具粒度（一顆收動詞文字的工具）——已由 D2 的第二個門與 D3 定案；工具的處理函式接在哪——接宿主套件的 Agent 門；工具落點——不成新套件，放在 Node SDK
- 那場還沒解、這場也沒談的小項：把工具的名稱、說明與輸入格式做成現成的匯出（宿主不用自己寫給模型看的工具說明）；按需回傳技能本文的第二顆工具；pi 的套件；Copilot SDK 的技能目錄細節
- 建議：工具說明的匯出併進 A4（host-kit-agent-door），因為它就是那個門的使用說明，量很小；其餘三項列為延後，目前沒有需求——pi 與 Cursor 已由進行中的變更 add-pi-cursor-tools 以內建工具的方式支援，技能由遠端提供後第二顆工具的用途也變小
- 型別化的 JS 方法（每個動詞一個函式）那場列為待決，這場的方向是單一的動詞文字入口，建議不做
**Ruled out**: 讓 agent-tool-sdk-layering 繼續開著並另外轉出變更 — 它的主要問題已在這場定案，兩份記錄並存會讓同一個決定有兩個出處
**Open**: 使用者確認 agent-tool-sdk-layering 的處置；結論草稿

### Round 26 — interview (2026-11-10)

**Focus**: 定案的圖解要不要進使用者文件
**Position**: 要。定案的圖解轉成圖檔放進文件，在 Markdown 裡搭配文字說明呈現；這件事併入最後一個變更 docs-embedded-host，不另開變更。
- 範圍：整體形態圖、SDK 的組成圖、一個動詞的四個步驟圖、宿主與示範 server 共用宿主套件的圖、技能用路徑讀與用動詞讀的對照圖
- 時機：照 D12，前十三個變更不動使用者文件，所以圖在最後一個變更才放進文件
- 圖的內容以當時的實作為準；實作和這場的定案不同時，先改圖再放
- 圖檔自帶背景色，深色的閱讀介面也看得清楚；中英兩版各一套；放在 docs/assets 底下
- 使用者要的是截圖；向量圖是否更合適（文字可以翻譯、檔案可以比對差異）在設計時評估
**Ruled out**: 另開一個只放圖的變更 — 文件已定為最後一次更新，圖是文件的一部分
**Open**: 無

### Round 27 — interview (2026-11-10)

**Focus**: 文件裡的圖用哪一種圖檔格式，以及放進文件的時機
**Position**: 兩項都定案：圖檔用 SVG（向量圖檔）；等最後一個變更 docs-embedded-host 才放進文件。
- 選 SVG 的理由：文件有中英兩版，SVG 的文字可以直接翻譯；SVG 是文字檔，版本控制看得到改了什麼；放大不會模糊
- SVG 要自己帶樣式與背景色，不依賴頁面的樣式表與外部字型
- 時機照 D12，使用者確認
**Ruled out**: 用截圖（PNG） — 兩個語言版本都要重新繪製再截圖，而且圖檔的差異無法比對
**Open**: 無

### Round 28 — interview (2026-11-10)

**Focus**: shared-verb-parsing 轉出前的查證結果，以及它要怎麼切
**Position**: 拆成三個變更，使用者確認；變更的總數從十四個變成十六個。
- 查證結果（結論列為未驗證的那一項）：CLI 是只有執行檔的 crate，clap 的型別都不公開；解析和開啟儲存、讀 stdin、印出結果寫在同一批 cmd 函式裡
- CLI 有 82 個末端動詞，其中建出引擎命令的涵蓋 43 種命令裡的 33 種；審查與驗證的八種、認領、搬移任務這十種 CLI 從不建立
- 「把結果轉成輸出」也在 CLI 裡：約 30 個 render 函式、約 300 處直接印出；Node SDK 的 9 個動詞各自另外組一次 --json 的內容。SDK 要涵蓋全部動詞，解析與輸出都要共用，牽動約 4500 行
- 流程設定的 11 個末端動詞今天完全不經命令層；正式規格 workflow-config 明寫 show 不進入命令執行層，那一句要跟著改
- 三個變更：shared-verb-parsing（共用的解析；CLI 改用它；SDK 的 9 個動詞改用它，能用的動詞數不變）、shared-verb-output（輸出改成共用的純函式；SDK 涵蓋全部引擎命令）、workflow-config-commands（流程設定的讀寫新增為引擎命令）
- 順序：三個依序做；第一個仍然和 store-port-contract、engine-on-store-port 無相依
**Ruled out**: shared-verb-parsing 維持一個變更 — 任務會超過 25 項，審查與驗證的範圍太大，中途出問題難定位；解析與輸出合成一個、流程設定另一個 — 第一個仍有約 20 項任務
**Open**: 無

## Conclusion

**Decision**: speclink 的遠端能力重做成「引擎＋宿主套件＋宿主實作的儲存接口」，官方 server 降為示範；分十六個變更依序實作，全部完成才發版。
- 形態：規格存在哪、引擎在哪執行、程式碼在哪是三個獨立的選擇；本系列服務「規格由一台 Server 持有」的形態，Server 可以是內嵌了引擎的宿主，也可以是官方的示範 server
- 分工：儲存媒介、連線、遷移、歷史保存、帳號、登入、權限都由宿主負責；speclink 負責引擎、宿主套件、儲存契約與合格測試、用戶端（CLI 與 Speclink Desktop）、技能
- 官方 server 的定位：示範與試用用，正式使用請把引擎內嵌進自己的系統；它改成用同一套宿主套件組出來
- 擴充規則：掛鉤可以拒絕或旁觀，不能改內容；宿主不加新動詞；宿主只能選擇提供哪幾支技能，不能改技能文字
- 發版：十六個變更全部完成才發版；每個變更完成後，示範用的 server 與既有測試都要維持可用，所以移除舊東西排在最後
- 流程設定：兩條規則已寫入 speclink/config.yaml——rules.proposal 新增「三種形態（本機 repo、連到遠端的 checkout、沒有本機規格檔的內嵌宿主）逐一寫明結果：可用、不適用並寫理由、或這次不做並寫成已知缺口」；rules.design 第二條加入內嵌宿主與 Node SDK，並補上「新增的引擎功能若需要本機事實，由呼叫端收集後當成輸入」
- 用詞：使用者看到的是本機專案、遠端專案、Server；整合者看到的是宿主、內嵌、SDK、宿主套件、儲存接口、入口；講規格在哪裡用「本機」「遠端」，講怎麼蓋出 Server 用「SDK」「內嵌」「宿主」；不再用「SDK」統稱整塊遠端功能，不再用「Remote Store」「團隊模式」「主機套件」
- 延後的項目必須在之後建立的文件裡明確寫成缺口
- 開啟中的討論 agent-tool-sdk-layering 已寫結論並封存，它的主要問題由本討論定案
- 未驗證、留給對應變更查的事：三個函式是否足以供應引擎全部的讀取（cut 2）；舊版用戶端遇到新欄位是否照常運作（cut 12）；trace 能否直接跑在三個函式上（cut 10）
- **cut 1 `store-port-contract`**: 三個函式的儲存契約、合格測試與記憶體版參考實作；純新增，不接線
  - Rust 的定義是正典，Node 的三個函式是它的對映
  - 列出：依鍵的前綴列出文件；讀取：一次收多個鍵，回傳內容、文件版本、最後修改時間；提交：一次帶全部寫入與刪除、每份文件的預期版本（或「必須不存在」）、事件
  - 三項保證：一次提交全部成功或全部不做；版本對不上就整筆拒絕；事件與寫入在同一次提交裡
  - 文件用字串鍵，引擎訂命名規則（進行中的變更、正式規格、已封存的變更、討論、流程設定、詞彙表、看板順序、evidence 各有前綴）；儲存不需要認得文件種類
  - 範圍的版本號每次提交遞增，放在列出與提交的回傳值；版本號的型別與範圍寫進契約
  - 合格測試只有一份，Rust 實作與 JS 實作都跑它；涵蓋版本衝突、整筆不寫、事件同批
  - speclink 保留本機模式的檔案實作
- **cut 2 `engine-on-store-port`**: 引擎經儲存契約讀寫
  - 用到才讀、一次讀多份、寫入收成一次提交並帶事件
  - 驗證三個函式足以供應引擎儲存介面的全部操作（列出、是否存在、讀、寫、刪、搬移）
  - 用真實大小的資料量測一次（參考量 165 個已封存變更、31 MB）；量出來慢才加依範圍版本號的快取
  - 純讀取動詞可能短暫看到新舊混合的內容，第一版接受；寫入類動詞由提交時的版本比對把關
  - 示範用的 server 先透過轉接沿用現有的 driver
- 已查證（原列為未驗證）：CLI 的解析與輸出都寫在只有執行檔的 crate 裡，牽動約 4500 行，所以原本的 shared-verb-parsing 拆成 cut 3、cut 4、cut 5 三個變更
- **cut 3 `shared-verb-parsing`**: 把「動詞文字轉成引擎命令」收成 CLI 與 SDK 共用的一份
  - 共用的解析涵蓋會建出引擎命令的全部動詞；CLI 改用它
  - Node SDK 手寫的 9 個動詞解析由共用的那一份取代；這個變更裡 SDK 能用的動詞仍是 9 個
  - 需要本機程式碼的資訊（改了哪些檔、檔案指紋、commit）由呼叫端收集後當成動詞的輸入
  - 和 cut 1、cut 2 無相依，可以並行
- **cut 4 `shared-verb-output`**: 把「引擎的結果轉成輸出」收成共用的純函式
  - 人眼輸出與 --json 都由共用的函式產生，CLI 只負責印出；既有輸出逐位元不變
  - Node SDK 的 dispatch 涵蓋全部引擎命令，各動詞不再另外組一次 --json 的內容
  - 排在 cut 3 之後
- **cut 5 `workflow-config-commands`**: 流程設定的讀寫新增為引擎命令
  - 本機、示範用的 server 與 SDK 走同一條處理路徑，之後的執行前檢查才看得到改了哪些鍵
  - 正式規格 workflow-config「show 不進入命令執行層」那一句跟著改
  - 排在 cut 4 之後
- **cut 6 `host-kit-agent-door`**: 宿主套件與給宿主 Agent 工具的門
  - 建立時給一次：儲存（三個函式）、掛鉤（選填）、列出「這個人可以用哪些範圍」的函式、上限設定、登入方式的設定
  - 每次呼叫給：身分（唯一且改名後不變的識別，加顯示名稱）、範圍、這個人的能力開關
  - 身分由宿主的程式在呼叫時給，不放在動詞參數裡、不經過模型；儲存物件由全部呼叫共用
  - 掛鉤三個點：引擎之前（收到誰、哪個範圍、命令名稱、參數；流程設定的寫入多帶變動的鍵；拒絕時用引擎既有的 refused 錯誤碼並帶宿主給的原因；沒有提供時預設允許）、提交（就是儲存的第三個函式）、提交之後（只能旁觀，盡力而為、不保證送達）
  - 掛鉤用一個具名的集合傳入、每個都是選填；不做每個動詞各一個掛鉤
  - 需要「至少送達一次」的宿主在提交的同一次交易裡把事件寫進自己的事件表
  - Node SDK 開放宿主套件、JS 版儲存契約的橋接、放進 npm 的合格測試
  - 工具的名稱、說明與輸入格式做成現成的匯出，宿主不用自己寫給模型看的工具說明
  - 既有的 31 方法 JS 儲存在文件標示為即將移除
  - 排在進行中的變更 add-pi-cursor-tools 之後，因為它會改 Node SDK 的技能渲染
- **cut 7 `host-kit-client-door`**: 給 CLI 與 Speclink Desktop 的門
  - 把請求轉命令、結果轉線上格式、錯誤對應、握手、「這個範圍有沒有變」的輪詢，從官方 server 搬到宿主套件
  - 宿主只寫一條路由；對每個請求都呼叫入口，驗得出身分就帶身分、驗不出就帶空值；哪些路徑免登入由入口決定
  - 入口回答一個不用登入的路徑：支援哪些登入方式（存取金鑰、裝置登入、瀏覽器登入）與瀏覽器登入的發行者網址，內容來自宿主建立入口時給的設定
  - 入口回答範圍清單
  - 推送事件預設關閉、只做輪詢；宿主提供跨實例的傳遞時才開推送
  - 宿主套件放一張涵蓋全部引擎命令的對照，每個命令必須選一個：有線上格式，或明寫不上線與理由；引擎新增命令而沒處理時編譯不過
  - 示範用的 server 改用宿主套件
  - 入口不做登入端點、帳號與成員、管理介面、備份
- **cut 8 `client-browser-login`**: 用戶端的瀏覽器登入與共用的登入程式
  - 用戶端做標準流程的用戶端那一半：開瀏覽器、PKCE、回到本機的隨機埠、保存 token、token 過期時換新
  - 授權網址與換 token 的網址沿用 RFC 8414 的文件格式，由宿主的登入服務提供
  - 預設順序是瀏覽器登入、裝置登入、存取金鑰；宿主沒有宣告時維持現有行為
  - 登入程式收成一份，CLI 與 Speclink Desktop 共用；存取金鑰的存放位置統一
  - Speclink Desktop「新增專案」的 Server 步驟跟著調整
  - 設計時確認 RFC 8252 對本機回呼位址帶任意埠的條文
  - 可以和 cut 7 並行
- **cut 9 `skills-verbs-first-wave`**: discuss、propose、ingest 改成全部經過動詞
  - discuss 讀正式規格改用既有的 show 動詞，不用清單回傳的路徑
  - change rank 在遠端可用；寫到哪裡在設計時定
  - 可以和 cut 6 並行
- **cut 10 `skills-verbs-second-wave`**: 其餘技能改成全部經過動詞，守門加嚴
  - archive、commit、worktree 開工前與收尾、apply、improve
  - 新增讀 evidence 的動詞，進行中的變更與已封存的變更都能讀
  - apply 判斷變更是否閒置太久，改用變更的建立時間與最後修改時間
  - 對規格檔做 git 操作的步驟只在規格位於這個 checkout 時產生
  - trace 與 plan --strict-overlap 在遠端可用
  - 守門涵蓋 compare against、cat、Check for、git add、git log、直接改寫，以及用清單回傳的路徑讀檔
  - 修正 tool-call 形式把交棒給其他技能的句子渲染成像動詞的寫法
- **cut 11 `remote-skill-delivery`**: 遠端專案的技能由遠端提供
  - 入口用引擎內建的範本渲染；產生哪些技能跟著該範圍的流程設定，含 worktree 的兩支
  - 用戶端的安裝與更新改從遠端取得；安裝後仍是 checkout 裡的檔案
  - 降級守門調整：遠端專案的技能版本跟著遠端，比本機 CLI 新時照裝並提醒
  - CLI 在本機比遠端提供的技能舊時印一行提醒（兩邊版號與更新方式），同一台 server 一天最多一次，不擋操作；CLI 不做自我更新
  - Speclink Desktop 對遠端專案啟用技能檔探測，基準是遠端提供的版本：專案裡的技能檔較舊顯示「更新技能檔」、沒有技能檔顯示「安裝技能檔」、本機的 Speclink 較舊顯示「更新 Speclink」；確認框多一列「Server 提供的技能版本」
  - 握手送的是套件版本、技能帶的是另一個版號，比對哪一個在設計時定
  - 文件寫明：連到一台 Server 等於信任它提供的技能
- **cut 12 `station-status-and-landing`**: 關卡狀態上線與程式碼落地的回報
  - 變更的清單與單筆回應加選填欄位：審查與驗證各自的狀態（未開始、進行中、已通過）、誰蓋的章、何時、幾輪
  - 「蓋章後檔案又改過」維持由有 checkout 的一端判斷
  - 引擎發出的六種審查與驗證事件改為歸到該變更，用戶端據此重新讀取
  - 蓋章的請求多帶 commit；變更記下分支名稱；新增回報落地 commit 的動詞；引擎只存，不附帶任何規則
- **cut 13 `host-free-fields`**: 變更與討論上的自由欄位
  - 宿主自訂的鍵值，引擎只存不解讀
  - 通用的寫入與移除動詞，原版 CLI 也能用；清單與單筆回應會帶出來
  - 宿主的程式可以在呼叫入口時附上，和建立變更或討論在同一次寫入，不經過模型
  - 存在既有 metadata 文字裡的專屬區塊；鍵名規則、數量與大小上限在設計時定
- **cut 14 `import-export-on-port`**: 匯入匯出建在儲存契約上
  - 兩個門都能用；大小上限由宿主在建立入口時設定；不做壓縮；匯入維持只能進空的範圍
- **cut 15 `retire-teamstore-and-drivers`**: 移除舊的儲存層
  - SQLite、Postgres、serverfs 改寫成儲存契約的範例，供使用者參考
  - 移除 TeamStore 與既有的 31 方法 JS 儲存
  - 流程設定 context 的專案說明同步更新
- **cut 16 `docs-embedded-host`**: README 與使用者文件一次更新
  - 前十五個變更不動使用者文件，各自在提案的 Impact 寫明文件影響，本變更照清單收
  - README 開頭改用本機專案與遠端專案；「自架 server」拆成「把 Speclink 內嵌進你的系統」與「試用官方 server」兩節，前者在前並列出宿主要做的四件事（實作三個函式並通過合格測試、掛一條路由、驗不出身分時傳空值、要支援瀏覽器登入時讓自己的登入服務提供標準文件並接受本機回呼）
  - 官方 server 的定位句：「官方 server 是示範與試用用的。它讓你不寫程式就能試遠端專案。正式使用請把引擎內嵌進自己的系統。」；正式規格 user-documentation 的對應需求跟著改
  - 「目前狀態」改寫；延後的項目在專案能力狀態與路線圖逐項列為已知缺口，README 用一行連過去
  - Node SDK 文件改寫成宿主套件、入口、掛鉤、儲存接口、合格測試；「Server 儲存後端」改寫成儲存接口的範例；Server 部署與 Server 備份與還原保留，開頭加上定位句
  - 用詞在 README、docs 與 LANGUAGE.md 同一批改完，舊說法的存量同批清掉，中英兩版同步；LANGUAGE.md 既有的「儲存後端」詞條跟著改
  - 定案的圖解轉成圖檔放進文件，在 Markdown 裡搭配文字說明：整體形態圖、SDK 的組成圖、一個動詞的四個步驟圖、宿主與示範 server 共用宿主套件的圖、技能用路徑讀與用動詞讀的對照圖；圖的內容以當時的實作為準；圖檔用 SVG（向量圖檔），自己帶樣式與背景色，不依賴頁面的樣式表與外部字型；中英兩版各一套，放在 docs/assets 底下
  - 這是最後一個變更
**Rationale**: 規格離開 repo 之後，儲存一定要補上 git 原本給的兩件事：一次提交改多份文件，以及兩個人不互相覆蓋。引擎在官方 server 上本來就是「先讀、跑流程、交出寫入清單與事件、一次提交」，所以儲存真正需要的只有列出、讀取、提交三個動作。把這三個動作交給宿主在自己的資料庫交易裡完成，宿主就不需要第二套連線、連線池與遷移工具，speclink 也不必替每一種資料庫維護 driver。線上協定的完整實作今天只存在於官方 server 裡；既然官方 server 的定位是示範，這些程式就該下移成共用的宿主套件，讓任何宿主掛一條路由就能服務原版的 CLI 與 Speclink Desktop。登入、權限與媒介因宿主而異，所以全部留在宿主那一邊，speclink 只守住「原版用戶端看得懂每一個回應」這條線。
**Rejected alternatives**:
- 把官方的 Rust Postgres driver 內建進 Node 套件當主要路徑 — 連線、連線池、TLS、遷移會變成 speclink 要長期維護的第二套資料庫層，而且非 Postgres 的宿主仍然需要自訂儲存的契約
- 在現有儲存介面的每個方法加上版本比對 — 沒有交易邊界，一個動詞的多筆寫入仍然可能留下半套
- 以舊的平台藍圖與 TeamStore 當設計依據 — 兩者都未經真實驗證，只有示範用的 server 用過
- 旁掛官方 server 並接宿主的身分 — 官方 server 只是示範
- speclink 維護一份官方的 Postgres 實作套件 — 只服務一種資料庫，而且有資料存取層的宿主用不到
- 保留 SQLite 與 serverfs 的官方 driver — 改成範例即可
- speclink 定義或實作 OAuth 伺服器端的任何部分 — 登入設計屬於宿主
- 用握手宣告登入方式 — 握手本身要先登入
- 要求宿主自己放行免登入的路徑 — 宿主容易漏掉，改由入口決定
- 能改寫內容的掛鉤 — 原版用戶端會看不懂回應
- 每個動詞各一個掛鉤 — 介面會跟著動詞數量長大
- 在 Node SDK 的手寫解析裡逐個補動詞 — 第三份解析會持續落後
- 宿主自行實作 47 種端點 — 格式要逐一對上並跟著改版
- CLI 自我更新指令 — 四種安裝通路各自不同，且會和 Speclink Desktop 的自動佈署互相覆蓋
- 現在就設計依版本號的快取 — 用到才讀之後是否還慢沒有數字
- 三個函式沿用封閉的文件種類清單 — 每新增一種文件，全部宿主的儲存都要跟著改
- 在每支技能裡加「規格在不在這個 checkout」的執行期判斷 — 讀寫改走動詞後不需要，git 步驟在產生技能時就能決定
- 分支與落地 commit 放進自由欄位 — 會讓宿主的鍵和 speclink 的鍵混在一起
- speclink 內建「commit 必須在正式分支才能封存」的規則 — 合併規則屬於宿主
- speclink 做匯入匯出的壓縮 — 宿主的路由層已有
- 只靠規則提醒、不加守門 — 規則靠人記得，守門讓漏掉的情況直接失敗
- 要求每個變更都把三種形態做齊 — 成本太高，允許寫明不適用或這次不做
- 每個變更各自更新 README — 中途的 README 會描述還沒發版的功能
- 每個階段結束就發版 — 使用者決定全部完成才發版
- 先移除舊的再做新的 — 中間會有一段時間示範用的 server 與測試不能跑
- 用「SDK」統稱整塊遠端功能 — SDK 是工具的名字，連到官方 server 的遠端專案沒有用到它
- shared-verb-parsing 維持一個變更 — 查證後任務會超過 25 項，審查與驗證的範圍太大
**Deferred**:
- 規格放在 checkout 之外的本機位置（spec_dir 的路徑規則與固定的決定順序） — 內嵌宿主用不到；spec_dir 今天沒有檢查，絕對路徑與 ../ 被接受是未定義的行為，文件要寫明
- 唯讀引用另一套規格 — 目前沒有需求
- 遠端專案的自訂產出流程 — 內建的兩種可用；字串鍵讓之後補它不用改宿主的儲存
- 遠端專案的手冊產生 — 導覽模式仍可用；手冊頁之後可以用一個鍵的前綴存放
- 嚴格一致的讀取快照 — 第一版接受短暫不一致；要嚴格的宿主可以在自己的讀取函式裡處理
- 多個宿主實例下的推送事件 — 預設只做輪詢，宿主提供跨實例傳遞時才開
- 依範圍版本號的快取 — 等 cut 2 的量測結果
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md
**Next**: /speclink-propose --from-discussion embedded-host-needs
