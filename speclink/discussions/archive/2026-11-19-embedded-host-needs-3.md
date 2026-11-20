---
topic: 內嵌宿主：第二輪修正後的第三輪（三項）
slug: embedded-host-needs-3
status: promoted
created: 2026-11-19
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: show-detail-and-export-check
---

# Discussion: 內嵌宿主：第二輪修正後的第三輪（三項）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

同一個導入規劃拿 main（33781529，加上尚未 commit 的 engine-export-scope）重新對照自己的設計，第二輪的九項都接上了，剩三項：T1 讀取動詞拿不到蓋章時的檔案指紋、T2 宿主在伺服端匯入規格資料夾的做法、T3 釋放認領的規則。來源文件已寫明需求與待確認的點，不需要先釐清需求（不跑 grill）；這份紀錄只談通用的內嵌宿主情境，不寫那個系統的名稱與內部細節。相關的正式規格是 node-sdk、host-runtime、workspace-migration、review-station、verify-station、client-protocol、change-lifecycle；進行中的變更是 engine-export-scope（尚未 commit，品質關卡第 1 輪進行中），它的 delta 改寫 engine-export-scope: node-sdk、engine-export-scope: workspace-migration。
Prior discussions: embedded-host-needs-2, embedded-host-needs
Source doc: ../speclink-embedded-host-needs-3.md

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-19)

**Focus**: 來源文件三項的現況主張，哪些和程式碼相符、哪些不同、哪些要使用者決定
**Position**: 現況描述大多相符，不同處只有 T3 兩點；要決定的有四件事加分刀。
- T1 相符：show --json 的 review、verify 不輸出 scope，是 quality/station.rs 的 to_json 刻意不帶；引擎內部的 StationStatus 已經有 scope
- T1 相符：client-protocol 規格只讓單筆變更的回應帶 scope，清單的項目不帶；掛鉤的 read 接受 show
- T1 補充：指紋算法寫在 docs/verb-contract（能以 UTF-8 讀的檔 CRLF→LF 後 SHA-256，其他檔取原始位元組），docs/sdk-node 只寫「文字檔」不夠精確；蓋章時已刪除的檔不進 scope
- T2 相符且是刻意設計：root 不是專案時得到零份文件，engine-export-scope 的 design 寫「規格資料夾不存在：不算錯，照舊回傳空的清單」；引擎已有 is_spec_folder（含 changes/、specs/、discussions/ 或 config.yaml）可判斷
- T2 相符只缺文件：createEngine 的 store.specDir 指定資料夾名稱，未給時讀 root/.speclink.yaml 的 spec_dir；只下載規格資料夾時 root 指上一層、specDir 給資料夾名稱
- T2 相符且型別說明已寫：程序內的 importScope 也比對 limits.importBytes（預設 32 MiB），31 MB 的專案接近上限；匯出沒有上限
- T2 相符：匯入只接受空的範圍（workspace-migration 正典）
- T3 相符：命令名稱 claim-release、參數 { name }，引擎與參考 server 都不限定誰能釋放
- T3 不同：docs/verb-contract 已有「只讓認領的人釋放」的 Node 範例（沒有 byId 時比對名稱），缺的是放進 SDK 文件的掛鉤段落與管理者的例外
- T3 不同：Speclink Desktop 沒有釋放認領的操作，只有認領；CLI 收到拒絕時是 409 refused，speclink-remote 原樣印出宿主的 reason 並非零結束
**Open**: D1 T1 指紋放預設輸出還是旗標；D2 exportScope 對非專案的 root 是否拒絕、放哪個變更；D3 再匯入是否只補文件；D4 引擎是否維持不限定釋放；D5 分刀

### Round 2 — interview (2026-11-19)

**Focus**: D1 蓋章時的檔案指紋要放進 show --json 的預設輸出還是用旗標帶出
**Position**: 給 show 加旗標（暫名 --detail），show --json --detail 的 review、verify 在已蓋章時帶 scope，形狀和線上單筆回應的 scope 相同。
- 預設輸出不變；list --json --detail 維持不帶 scope，和線上清單一致
- --detail 在 list 與 show 都是「對齊線上格式」的意思
- 依據：329 份已封存 metadata 的指紋筆數（兩關卡合計）中位數 8、第 90 百分位 80、最多 284，每筆約 110 bytes；10 支技能讀 show --json，archive、commit、review、verify 都在蓋章後跑，Agent 用不到雜湊
- 要改寫 review-station 與 verify-station 列出 show --json 欄位的需求，參考文件補新旗標
**Ruled out**: 預設輸出就帶 scope — 蓋過章的變更每次讀都多 9 到 31 KB 的雜湊，技能用不到；review show／verify show 在蓋章後回章 — 改掉這兩個動詞讀工單的意思，工單蓋章後已刪
**Open**: D2 exportScope 對非專案的 root 是否拒絕、放哪個變更；D3 再匯入是否只補文件；D4 引擎是否維持不限定釋放；D5 分刀

### Round 3 — interview (2026-11-19)

**Focus**: D2 root 不是專案時 exportScope 是否拒絕、改動放在哪個變更
**Position**: Node 的 exportScope 在 root 底下的規格資料夾不是規格資料夾時拒絕，併入進行中的 engine-export-scope（走 ingest）。
- 判斷沿用引擎既有的 is_spec_folder：含 changes/、specs/、discussions/ 或 config.yaml 才算；拒絕時 code 是 error，訊息點名資料夾名稱（暫定 `no spec folder at "speclink" under root`）
- 規格資料夾存在但沒有文件（例如只有空的 specs/）照常成功、零份文件，代表真的是空的
- 只改 Node 的匯出；共用的 local_documents 不動，Speclink Desktop 的「遷移到 Server」照舊：Desktop 新增本機資料夾時不檢查規格資料夾，把空資料夾遷移到 Server 從零開始是合理的路
- 這推翻 engine-export-scope design 的「規格資料夾不存在：不算錯」：匯出是一次性的搬遷，零份的結果看起來像成功，宿主不會再回頭看；dispatch 讀錯位置時呼叫端馬上看得到
- 進行方式：engine-export-scope 的品質關卡第 1 輪進行中，可能有別的 session 在跑；ingest 等那邊停下或在那個 session 裡做，之後重跑兩站是複驗
**Ruled out**: 檢查放進共用的 local_documents — 會擋掉 Desktop 把空資料夾遷移到 Server 的路；之後另開新變更 — 要等 engine-export-scope 封存才能提案，同一份規格和 transfer.rs、lib.rs 要再改一次；不拒絕只補文件 — 每個宿主都要自己先檢查資料夾，放錯位置時仍是看似成功的零份匯入
**Open**: D3 再匯入是否只補文件；D4 引擎是否維持不限定釋放；D5 分刀

### Round 4 — interview (2026-11-19)

**Focus**: D3 匯入後 repo 的規格資料夾又改過時，「再匯入一次」要做成功能還是只補文件
**Position**: 只補文件，不加新功能。
- 依據：importScope 判斷範圍是否為空只看儲存裡有沒有文件（transfer.rs 的 port.list），不看事件；匯入零份時不寫入、直接回傳成功
- 文件寫明三點：匯入是一次性的搬遷不是同步；要重新匯入，宿主先清空自己儲存裡這個範圍的全部文件再呼叫 importScope；清空會刪掉匯入後在宿主裡做的全部改動（變更、討論、蓋章）
**Ruled out**: 持續同步 repo 的規格資料夾進宿主 — 等於「規格放在 checkout 之外」，前兩輪都延後，來源文件寫明不重提；匯入覆寫非空的範圍 — 違反 workspace-migration「只接受 CreateNew」，合格測試釘住、四支儲存實作都要改
**Open**: D4 引擎是否維持不限定釋放；D5 分刀

### Round 5 — interview (2026-11-19)

**Focus**: D4 引擎是否維持不限定誰可以釋放認領
**Position**: 引擎維持不限定，只補文件。
- 第二輪「規則留在宿主」的理由仍成立；「管理者」引擎判斷不了：HostIdentity 只有 id 與 displayName，沒有角色
- 文件一：docs/verb-contract 的「只讓認領的人釋放」範例搬到 SDK 文件的掛鉤段落、放在封存範例之後，加一行由宿主自己的函式判斷的管理者例外（例如 if (await isAdmin(identity)) return）；docs/verb-contract 只留「命令名稱是 claim-release，只擋 claim 的掛鉤擋不到釋放」並連到 SDK 文件，範例只留一份
- 文件二：SDK 文件「Speclink 不內建…規則」那句補上「也不限定誰可以釋放認領」
- 文件三：掛鉤段落寫明拒絕原因給誰看：CLI 把 reason 原文印到 stderr；Speclink Desktop 沒有釋放認領的操作，認領被拒時失敗提示帶出原因
**Ruled out**: 身分加角色讓引擎判斷管理者 — 等於在掛鉤之外再做一套規則機制；範例在 verb-contract 與 SDK 文件各留一份 — 兩份日後會對不上
**Open**: D5 分刀

### Round 6 — interview (2026-11-19)

**Focus**: D2 的落點在前提改變後是否要換
**Position**: 改成 B：engine-export-scope 照現狀封存，exportScope 對非專案 root 的拒絕放進這次要開的新變更；拒絕的內容與第 3 輪相同。
- 改變的前提：第 3 輪選 A 時以為 engine-export-scope 的審查還要修幾輪；2026-11-19 14:14 兩站已落乾淨章、工單已刪，只差封存與 commit
- 現在選 A 會讓剛落的兩個乾淨章作廢，要再走 ingest、apply、兩站複驗
- B 不多開變更：新變更本來就要做 T1 與文件，會改同一批檔案（speclink-node 的 lib.rs、index.d.ts、docs/sdk-node）；代價只是「規格資料夾不存在：不算錯」先進正式規格再被改掉
- 新變更開工前要先封存 engine-export-scope，新變更的 delta 才能改寫「本機引擎的 exportScope 方法」
**Ruled out**: 維持 A 併入 engine-export-scope — 前提已不成立，代價變成丟掉兩個剛落的乾淨章
**Open**: D5 分刀（建議一個變更做完 T1、T2、T3）

## Conclusion

**Decision**: 三項都做，合成一個新變更 `show-detail-and-export-check`（名稱暫定），開工前先封存 engine-export-scope。
- 來源文件的現況描述大多成立；不同處：docs/verb-contract 已有「只讓認領的人釋放」的 Node 範例，Speclink Desktop 沒有釋放認領的操作
- T1：show 加 --detail，只能和 --json 一起用，規則與 list --detail 相同（單獨用或配 --specs 是用法錯誤、非零結束，host.dispatch 的錯誤代碼是 invalid_argv）
- T1：show --json --detail 的 review、verify 在 state 為 stamped 時多帶 scope（每項 path 與 hash），形狀與線上單筆變更回應的 scope 相同；inProgress 或沒有章時不帶
- T1：不帶 --detail 的 show --json 逐位元不變；list --json --detail 維持不帶 scope，與線上清單一致
- T1：改寫 review-station 與 verify-station 列出 show --json 欄位的需求，參考文件補上 show 的 --detail
- T1：SDK 文件把指紋算法寫精確——能以 UTF-8 讀的檔 CRLF→LF 後 SHA-256，其他檔取原始位元組；寫明蓋章時已刪除的檔不在 scope 裡
- T2：Node 的 exportScope 在 root 底下的規格資料夾不是規格資料夾時，在讀任何文件前以 Error 拒絕，code 是 error，訊息點名資料夾名稱（暫定 `no spec folder at "speclink" under root`）；判斷沿用 is_spec_folder（含 changes/、specs/、discussions/ 或 config.yaml）
- T2：規格資料夾存在但沒有文件時照常成功、零份文件；共用的 local_documents 不動，Speclink Desktop 的「遷移到 Server」照舊接受空資料夾
- T2：這推翻 engine-export-scope design「規格資料夾不存在：不算錯」，改寫 node-sdk 的「本機引擎的 exportScope 方法」與型別說明
- T2：SDK 文件加「在伺服端匯入」一段，寫明四點：只下載規格資料夾時 root 指上一層、store.specDir 給資料夾名稱（不給時讀 root/.speclink.yaml 的 spec_dir）；limits.importBytes 也管程序內的 importScope（預設 32 MiB）、匯出沒有上限；匯入是一次性的搬遷不是同步；要重新匯入，宿主先清空自己儲存裡這個範圍的全部文件再匯入，這會刪掉匯入後在宿主裡做的全部改動
- T3：引擎維持不限定誰可以釋放認領
- T3：docs/verb-contract 的釋放範例搬到 SDK 文件的掛鉤段落、放在封存範例之後，加一行由宿主自己判斷的管理者例外（例如 if (await isAdmin(identity)) return）；docs/verb-contract 只留「命令名稱是 claim-release，只擋 claim 的掛鉤擋不到釋放」並連到 SDK 文件
- T3：SDK 文件「Speclink 不內建…規則」那句補上「也不限定誰可以釋放認領」；掛鉤段落寫明 reason 給誰看：CLI 原文印到 stderr，Speclink Desktop 認領被拒時的失敗提示帶出原因
- 例子：對已蓋審查章的 demo 執行 speclink show demo --json --detail，review 物件含 state stamped、stampedAt、rounds、tasksTotal、commit 與 scope [{ path: "src/a.rs", hash: "3f9a…" }]；不帶 --detail 時同一物件沒有 scope
**Rationale**: 三項都是「能力已經有、只差一個出口或一段說明」：指紋早就存在引擎裡，只是 show 刻意不輸出；匯出與匯入的規則都在型別說明裡，宿主卻拼不出完整的做法；釋放的範例寫在協定文件而不在 SDK 文件。修法一律走既有的路：指紋用和 list 同義的 --detail 帶出，不讓讀 show --json 的十支技能每次多讀一串雜湊；規則仍然留在宿主，引擎不加角色。唯一改行為的是 exportScope 對放錯位置的拒絕，因為一次性的搬遷得到零份時看起來像成功，宿主不會再回頭看；Desktop 的遷移刻意不跟著改，因為把空資料夾遷移到 Server 是它的合理用法。
**Rejected alternatives**:
- show --json 預設就帶 scope — 已封存 metadata 的指紋筆數第 90 百分位 80、最多 284，每次讀多 9 到 31 KB 雜湊，技能用不到
- review show／verify show 在蓋章後回章 — 改掉這兩個動詞讀工單的意思，工單蓋章後已刪
- exportScope 的檢查放進共用的 local_documents — 會擋掉 Desktop 把空資料夾遷移到 Server 的路
- 把拒絕併入 engine-export-scope — 兩站已落乾淨章，併入要作廢重跑
- 不拒絕只補文件 — 放錯位置時仍是看似成功的零份匯入，每個宿主都要自己先檢查
- 持續同步 repo 的規格資料夾進宿主 — 等於「規格放在 checkout 之外」，前兩輪延後
- 匯入覆寫非空的範圍 — 違反 workspace-migration 的只接受 CreateNew，合格測試釘住
- 身分加角色讓引擎判斷管理者 — 等於在掛鉤之外再做一套規則機制
- 釋放範例在 verb-contract 與 SDK 文件各留一份 — 兩份日後會對不上
**Deferred**: none
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion embedded-host-needs-3
