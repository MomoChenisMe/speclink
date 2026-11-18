---
topic: 內嵌宿主：對照 main 實作後的第二輪需求
slug: embedded-host-needs-2
status: promoted
created: 2026-11-18
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: client-os-trust-store, hook-read-function, claim-release-and-output, verb-output-for-embedded, language-replace-and-discuss-restore, event-payloads-and-push-url, engine-export-scope
---

# Discussion: 內嵌宿主：對照 main 實作後的第二輪需求

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

同一個導入規劃拿 main（5ba701c2，CLI 0.12.0、engine v1.58.0，尚未發版）對照自己的設計，列出九個接不上的點（N1 到 N6、M1 到 M3），寫成一份文件當作討論的輸入。那個系統是私有的，本記錄只談通用的情境。文件的每一項都有可檢驗的現況與需求，所以不需要先磨需求，直接逐項對照程式碼分類。
相關的正式規格：verb-contract、review-station、verify-station、change-lifecycle、host-kit、node-sdk、client-protocol、server-event-stream、remote-connection、workflow-config、discussion-docs。目前沒有進行中的變更。
Prior discussions: embedded-host-needs
Source doc: ../speclink-embedded-host-needs-2.md

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-18)

**Focus**: 來源文件的九項主張，對照程式碼之後哪些成立、哪些不成立、哪些要由使用者決定
**Position**: 九項的現況描述都成立，只有 M1 有一句和文件不同；每一項的修法都是要使用者決定的事。
- N1 成立：建立 HTTP 用戶端的六處（speclink-remote 的 client.rs:108、events.rs:85 與 237、device.rs:48 與 137、oauth.rs:87）都只設逾時；Cargo.lock 裡 ureq 2.12.1 只依賴 webpki-roots，不讀作業系統的信任庫。Speclink Desktop 的事件串流走同一個用戶端（apps/desktop/src-tauri/src/lib.rs:1943 呼叫 speclink_remote::events::subscribe），前端不直接連 server。正式規格沒有談 TLS 信任
- N2 成立：執行前掛鉤只收到身分、範圍、命令、參數與 changedKeys（speclink-host 的 kit.rs:110）。沒有章的變更可以封存是 change-lifecycle 的明文；上一輪討論否決了引擎內建封存規則，所以由掛鉤判斷是對的方向。掛鉤裡等內層 dispatch 會完成：每個 dispatch 開自己的執行緒（speclink-node 的 lib.rs:56）。index.d.ts:205 寫的「would never finish」比實際嚴格；真正的風險是宿主自己的鎖，以及內層呼叫會再問一次掛鉤
- N3 成立：list --json 的欄位由 review-station 與 verify-station 各一條需求固定；station-status-and-landing 的設計寫明 show --json 是給內嵌宿主讀關卡狀態的路。show --json 沒有認領的人；claimed_by_id 寫進 metadata（speclink-core 的 command/mod.rs:1984）但沒有任何輸出帶出；claim 沒有 --json
- N4 成立：從本機規格資料夾做匯入包的程式只在 Speclink Desktop（apps/desktop/src-tauri/src/remote.rs:471 的 build_import_bundle）；createEngine 沒有匯出；CLI 沒有匯入匯出的動詞，已列在已知缺口
- N5 成立：language 只有 show；discuss 沒有把封存搬回來的動詞；兩項都在 docs/product-status.zh-TW.md 的已知缺口
- N6 成立：蓋章、丟棄工單、認領的事件內容只有變更名稱。送給用戶端的事件只指出哪個資源變了，這是刻意的（speclink-protocol 的 events.rs 註解）。流程設定沒有事件也是刻意的（workflow-config-commands 的設計把它留給掛鉤的變更，而宿主套件只在執行前掛鉤給了 changedKeys），但使用者文件沒有寫原因。引擎事件的內容並不一致：change-landed 帶 commit 與 branch，自由欄位的事件帶 keys
- M1 大致成立：Speclink Desktop 只檢查握手有沒有宣告 SSE（remote.rs:1412），然後連固定路徑（lib.rs:1932），不用宣告的 url。不同處：docs/verb-contract.md 有寫閒置時送心跳，沒寫的是間隔（server 預設 15 秒）、用戶端 45 秒沒收到資料就斷線（events.rs:33）與固定路徑
- M2 成立：沒有釋放認領的動詞，錯誤訊息卻寫「ask them to release it」（command/mod.rs:1969）
- M3 成立：verb-contract「動詞人眼輸出的兩模式同形」第 1、5 項規定遠端不印 new change 與 discuss promote 的 Path 行；內嵌是規格沒列到的第三種模式，所以照本機印。JSON 的 path 在遠端 CLI 也有，verb-contract 已把 list --specs 的 path 定義成儲存的鍵
- 來源文件「不在這一輪」的項目和上一輪的延後清單一致
**Open**: N1 信任庫的做法；N2 掛鉤讀狀態的形式；N3 動詞輸出怎麼補；N4 匯入路徑開在哪裡；N5 兩個動詞；N6 事件內容；M1 推送網址；M2 釋放認領；M3 路徑輸出；怎麼分刀與順序

### Round 2 — assumptions (2026-11-18)

**Focus**: N2 到 M3 八項各自的修法
**Position**: 使用者裁定 N2 選「把唯讀的讀取函式傳進掛鉤」，其餘七項照建議。
- N2：beforeCommand 多收一個讀取函式（暫定寫法 beforeCommand(info, { read })）；read(argv) 只接受讀取類的動詞，結果和 dispatch 的 --json 相同，不再問一次掛鉤，沿用同一個身分與範圍。接縫在宿主套件，Rust 與 Node 各一個轉接。文件寫明讀到的內容和封存那一次讀到的不是同一份，中間有人改時由引擎自己的守門擋；SDK 文件「永遠不會完成」那一句同批改正
- N3：show --json 加 claimed（by、byId、at），只在被認領過時出現，和 landed 同一種寫法；list 的預設輸出不動，加一個旗標（暫名 --detail）讓每一項帶關卡狀態、認領、分支、落地記錄，形狀對齊 GET /changes；claim 加 --json；線上格式加選填的 claimedById
- N4：build_import_bundle 從 Speclink Desktop 搬到共用的 crate，Desktop 改用共用的版本；Node 的 createEngine 加 exportScope()，輸出第 2 版的匯入包，和 host.exportScope 同形
- N5：新增 language replace --stdin（整份寫回，用一般的寫入權限）與 discuss restore <slug>（搬回同 slug 最新的一份封存，同 slug 已有進行中的記錄時拒絕）；兩者都是引擎命令，經過執行前掛鉤；discuss 技能寫詞彙表的步驟改用新動詞
- N6：文件寫明事件只指出哪裡變了、收到後重新讀取；章記下 commit 時蓋章事件帶 commit；新增流程設定變動的事件，內容是變動的鍵。這推翻 workflow-config-commands 當時的不做項目，理由是 changedKeys 只到執行前掛鉤，宿主在 commit 裡寫自己的事件表時拿不到
- M1：Speclink Desktop 改用宣告的 SSE 網址，相對路徑接在 origin 後面，只接受同一個 origin；verb-contract 寫明心跳間隔與 45 秒斷線的規則
- M2：新增釋放認領（暫定 claim <change> --release）；引擎不限定誰可以釋放，有寫入權限即可，要限定的宿主用掛鉤擋
- M3：host.dispatch 的人眼輸出不印 Path 行，verb-contract 的分歧清單延伸到內嵌；JSON 的 path 保留，Agent 工具的說明寫明它是儲存的鍵
**Ruled out**: N2 正式允許在掛鉤裡呼叫 dispatch — 只靠文件約束，掛鉤對讀取命令也去讀就會無限遞迴，也擋不住掛鉤裡的寫入；N2 在掛鉤參數附上狀態摘要 — 每個命令都要先多讀一次，多一份要和 show --json 同步的格式，還會打破「拒絕發生在儲存收到任何呼叫之前」；N3 直接改 list --json 的預設輸出 — 要改兩條正式規格，每支技能呼叫 list 都會讀到更多內容；N4 這一輪就做 CLI 的匯入匯出動詞 — 寫腳本的宿主用 SDK 就夠，CLI 的缺口維持列在已知缺口；M1 用戶端實作 WebSocket — 宣告 SSE 就能推送，只宣告 WebSocket 時照舊輪詢
**Open**: N1 信任庫的做法；怎麼分刀與順序

### Round 3 — interview (2026-11-18)

**Focus**: N1 用戶端怎麼信任組織自己的 CA
**Position**: 使用者裁定只做「讀作業系統的信任庫」，不加指定 CA 檔的設定。
- 用戶端改由作業系統的憑證檢查決定（暫定 rustls-platform-verifier），IT 裝進系統的簽發單位自動生效，使用者不用設定
- speclink-remote 建立 HTTP 用戶端的六處收成一個共用函式；CLI 與 Speclink Desktop 都經過 speclink-remote，所以行為相同
**Ruled out**: 加一個指定額外 CA 檔的設定（環境變數或設定檔） — 目前沒有人遇到不能改系統信任庫的機器；多一個設定要維護，設錯不容易查；之後要加也不影響讀系統信任庫的做法
**Open**: 怎麼分刀與順序

### Round 4 — assumptions (2026-11-18)

**Focus**: 九項怎麼切成變更，以及順序
**Position**: 使用者裁定切成七個變更，N1 與 N2 排最前面。
- 順序：client-os-trust-store（N1）、hook-read-function（N2）、claim-release-and-output（M2 加 N3 認領的那一半）、verb-output-for-embedded（N3 清單的那一半加 M3）、language-replace-and-discuss-restore（N5）、event-payloads-and-push-url（N6 加 M1）、engine-export-scope（N4）
- N3 拆兩半：認領那一半要加新命令並改線上格式，清單那一半只改輸出；合在一起估計超過 20 項任務
- M2 併進認領那一半：兩者改同一塊程式；M3 併進清單那一半：兩者都只改動詞輸出；N5 的兩個動詞併成一個：兩者都改 discuss 技能的原稿，分開會在同一個技能檔與版號上衝突；N6 與 M1 併成一個：兩者都改文件裡事件的那一節
- verb-output-for-embedded 排在 claim-release-and-output 之後，因為詳細輸出包含認領的人；其餘變更互相沒有相依，可以並行
- 每個變更自己更新使用者文件，並把已解決的項目從已知缺口移走
**Ruled out**: N3 維持一個變更 — 任務數太多，審查與驗證的範圍太大；N5 拆成兩個變更 — 兩個都改 discuss 技能的原稿，會互相衝突；使用者文件集中到最後一個變更 — 文件已經存在，跟著各個變更改比較不會漏
**Open**: 無

## Conclusion

**Decision**: 來源文件的九項需求全部接受，切成七個變更；N1 與 N2 先做。
- 來源文件的現況描述全部成立，只有一處不同：docs/verb-contract.md 有寫閒置時送心跳，沒寫的是間隔、用戶端的逾時與固定路徑
- 每個變更自己更新使用者文件，並把已解決的項目從 docs/product-status 的已知缺口移走；每份提案照流程設定的規則，把本機專案、連到遠端的 checkout、內嵌宿主三種形態的結果逐一寫明
- 旗標與函式的名稱都是暫定，設計時定案
- **cut 1 `client-os-trust-store`**: 用戶端改信任作業系統的信任庫（N1）
  - 憑證由作業系統的檢查決定（暫定 rustls-platform-verifier），IT 裝進系統的簽發單位自動生效，使用者不用設定
  - speclink-remote 建立 HTTP 用戶端的六處收成一個共用函式：一般 API、事件串流與狀態查詢、裝置登入、瀏覽器登入的說明文件查詢與換 token
  - CLI 與 Speclink Desktop 都經過 speclink-remote，行為相同
  - 不加指定額外 CA 檔的設定
- **cut 2 `hook-read-function`**: 執行前掛鉤多收一個唯讀的讀取函式（N2）
  - 暫定寫法 beforeCommand(info, { read })；read(argv) 只接受讀取類的動詞，結果和 dispatch 的 --json 相同
  - read 不再問一次掛鉤，沿用這次呼叫的身分與範圍
  - 接縫在宿主套件；Rust 的掛鉤與 Node 的掛鉤各一個轉接，兩個入口（host.dispatch 與 host.handleRequest）共用
  - 文件寫明：讀到的內容和命令本身讀到的不是同一份，中間有人改時由引擎自己的守門（工單開著、章過期）擋
  - index.d.ts 與 SDK 文件「在掛鉤裡等 dispatch 永遠不會完成」那一句改成實際的行為：會完成，但會再問一次掛鉤，而且宿主自己持有鎖時可能卡住；建議改用 read
  - 文件的封存範例改成用 read 讀審查、驗證與落地的狀態
- **cut 3 `claim-release-and-output`**: 釋放認領，並讓動詞輸出帶出認領的人（M2 加 N3 的一半）
  - 新增釋放認領（暫定 claim <change> --release）；引擎不限定誰可以釋放，有寫入權限即可，要限定的宿主用掛鉤擋
  - show --json 加 claimed（by、byId、at），只在被認領過時出現，和 landed 同一種寫法
  - claim 加 --json
  - 線上格式加選填的 claimedById
- **cut 4 `verb-output-for-embedded`**: 清單的詳細輸出，以及內嵌時不印路徑行（N3 的另一半加 M3）
  - list 加一個旗標（暫名 --detail），每一項帶關卡狀態、認領、分支、落地記錄，形狀對齊 GET /changes
  - list --json 的預設輸出不動，review-station 與 verify-station 固定欄位的兩條需求維持
  - host.dispatch 的人眼輸出不印 new change 與 discuss promote 的 Path 行；verb-contract「動詞人眼輸出的兩模式同形」的分歧清單延伸到內嵌
  - JSON 的 path 保留；Agent 工具的說明寫明它是儲存的鍵，不是檔案
  - 排在 cut 3 之後
- **cut 5 `language-replace-and-discuss-restore`**: 寫入詞彙表與搬回封存討論的兩個動詞（N5）
  - language replace --stdin：整份寫回，用一般的寫入權限，不算政策鍵
  - discuss restore <slug>：搬回同 slug 最新的一份封存；同 slug 已有進行中的記錄時拒絕
  - 兩者都是引擎命令，兩個入口都可用，經過執行前掛鉤
  - discuss 技能寫詞彙表的步驟與「封存錯了」的說明改用新動詞
- **cut 6 `event-payloads-and-push-url`**: 事件內容與推送網址（N6 加 M1）
  - 文件寫明：事件只指出哪裡變了，收到後重新讀取，這是刻意的設計
  - 章記下 commit 時，review-stamped 與 verify-stamped 帶 commit，和 change-landed 一致
  - 新增流程設定變動的事件，內容是變動的鍵；這推翻 workflow-config-commands 當時的不做項目
  - Speclink Desktop 改用握手宣告的 SSE 網址；相對路徑接在 origin 後面，只接受同一個 origin
  - docs/verb-contract 寫明心跳間隔與用戶端 45 秒沒收到資料就斷線的規則
- **cut 7 `engine-export-scope`**: 本機規格資料夾做成匯入包的函式開放給 SDK（N4）
  - build_import_bundle 從 Speclink Desktop 搬到共用的 crate，Desktop 改用共用的版本
  - Node 的 createEngine 加 exportScope()，輸出第 2 版的匯入包，和 host.exportScope 同形，可以直接交給 host.importScope
**Rationale**: 九項都是同一類問題：上一輪把遠端能力重做成「引擎加宿主套件」之後，有些能力只接到其中一條路。憑證信任只照顧到公開的 CA；掛鉤能拒絕卻讀不到判斷需要的狀態；認領、關卡狀態與匯入包只有線上格式或 Speclink Desktop 拿得到，只走動詞的 Agent 與宿主的程式拿不到。修法一律選「補齊既有的那條路」而不是開新的機制：掛鉤拿到的是既有讀取動詞的唯讀版本，清單用旗標而不動預設輸出，事件沿用既有事件已經有的欄位寫法。規則仍然留在宿主：引擎不限定誰能釋放認領，也不內建封存條件。
**Rejected alternatives**:
- 加一個指定額外 CA 檔的設定 — 目前沒有人遇到不能改系統信任庫的機器，多一個設定要維護
- 正式允許在掛鉤裡呼叫 dispatch — 只靠文件約束，掛鉤對讀取命令也去讀就會無限遞迴，也擋不住掛鉤裡的寫入
- 在掛鉤參數附上變更的狀態摘要 — 每個命令都要先多讀一次，多一份要和 show --json 同步的格式，還會打破「拒絕發生在儲存收到任何呼叫之前」
- 直接改 list --json 的預設輸出 — 要改兩條正式規格，每支技能呼叫 list 都會讀到更多內容
- 用戶端實作 WebSocket — 宣告 SSE 就能推送，只宣告 WebSocket 時照舊輪詢
- N3 維持一個變更 — 任務數太多，審查與驗證的範圍太大
- N5 拆成兩個變更 — 兩個都改 discuss 技能的原稿，會互相衝突
- 使用者文件集中到最後一個變更 — 文件已經存在，跟著各個變更改比較不會漏
**Deferred**:
- CLI 的匯入與匯出動詞 — 寫腳本的宿主用 SDK 就夠，維持列在已知缺口
- 發版的時機（這七個變更和尚未發版的 0.12.0 是否一起發） — 這次沒有討論，發版時由使用者決定
- 上一輪列為延後的項目（規格放在 checkout 之外、唯讀引用另一套規格、遠端的自訂產出流程、遠端的手冊產生、嚴格的讀取快照、多實例的推送、依版本號的快取） — 來源文件明寫這一輪不重提
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion embedded-host-needs-2
