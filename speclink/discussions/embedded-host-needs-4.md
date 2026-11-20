---
topic: 內嵌宿主第四輪（三項需求、四個觀察）
slug: embedded-host-needs-4
status: promoted
created: 2026-11-20
created_by: MomoChen <momochenisme@gmail.com>
hold: true
promoted_to: sdk-types-parse-check
---

# Discussion: 內嵌宿主第四輪（三項需求、四個觀察）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

同一個導入規劃照設計實作了第一段，用的是 npm 上的 @speclink/engine 0.13.0，回報三項需求與四個觀察：F1 發佈的型別定義檔解析不了、F2 SDK 沒有匯出命令名稱的清單、F3 匯出與匯入的失敗沒有穩定的原因代碼；O1 --json 的欄位命名不一致、O2 new artifact 的 path 不是儲存的鍵、O3 空字串把 locale 設回未設定、O4 平台套件只在 macOS arm64 實跑過。來源文件已寫明需求與待確認的點，不需要先釐清需求（不跑 grill）；這份紀錄只談通用的內嵌宿主情境，不寫那個系統的名稱與內部細節。查證用 main（9f373f91，index.d.ts 與 0.13.0 發佈的那一份逐位元相同）讀碼，加上在暫存資料夾安裝 0.13.0 搭配記憶體儲存實跑。相關的正式規格是 node-sdk、node-sdk-release、host-kit、host-runtime、command-runtime、verb-contract、workflow-config、change-analysis；沒有進行中的變更。
Prior discussions: embedded-host-needs-3, embedded-host-needs-2, embedded-host-needs
Source doc: ../speclink-embedded-host-needs-4.md

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-20)

**Focus**: 來源文件的三項需求與四個觀察，各自的現況描述是否成立
**Position**: 七項的現況大多成立，三處和程式不一樣；要決定的是八件事（見 Open）。
- F1 成立：實跑 tsc 回報 91 個錯誤，第一個是 `index.d.ts(524,59): error TS1131`，`skipLibCheck: true` 關不掉；把那一行的 `'*/*'` 改成 `'*\/*'` 之後，不開 skipLibCheck 也是零錯誤
- F1 成立：`.github/workflows/node-sdk.yml` 只跑 vitest，vitest 不解析 index.d.ts，套件的 devDependencies 沒有 TypeScript，所以沒有任何檢查會解析型別定義檔
- F2 成立：引擎的命令名稱 54 個（`crates/engine/speclink-core/src/command/describe.rs` 第 14 到 67 行），宿主套件另有 `scope-import`、`scope-export`（`crates/host/speclink-host/src/transfer.rs` 第 49、52 行）；`index.js` 只有七個匯出，沒有清單
- F2 成立：`agentTool.description` 由動詞樹逐行產生（`crates/adapters/speclink-verbs/src/tool.rs`），列的是動詞不是命令名稱
- F2 補充：`crates/host/speclink-host/src/read_only.rs` 的 `is_read` 已經逐命令判定「不寫入」，沒有萬用分支
- F3 成立：實跑的 Error 自己的屬性只有 `code`；SDK 文件要宿主比對訊息開頭
- F3 補充：匯入時範圍不是空的，和掛鉤拒絕，`code` 都是 `refused`，宿主只能靠訊息分辨
- F3 補充：本機匯出的失敗在 `transfer::local_documents` 是沒有型別的字串；匯入的失敗有型別（`TransferError`），超過大小上限歸在 InvalidBundle
- O1 成立，而且是刻意凍結的：`discuss archive` 的 `archived_to`、`task done` 的 `task_id` 與 `task_desc` 實跑重現；verb-contract 的情境釘住 `task_desc`、`task_id`；change-analysis 明寫 `analyze --json` 的 snake_case 欄位維持不改
- O2 成立，而且和正式規格衝突：內嵌時 `new artifact --json` 的 `path` 是 `speclink/changes/demo/proposal.md`，儲存的鍵是 `changes/demo/proposal.md`，來源是 `crates/host/speclink-host/src/port_view.rs` 的 `write_artifact`；同一個宿主裡 `discuss new` 的 `path` 是鍵；node-sdk 的需求「Agent 工具的說明指出 path 是儲存的鍵」已經承諾是鍵
- O3 成立，是刻意的：workflow-config 的規格與 docs/configuration.md 都寫了空字串會移除鍵；只有 `set --help` 沒寫
- O4 部分不成立：node-sdk.yml 的建置矩陣在 Windows x64、Linux x64、macOS arm64 都跑整套測試；沒跑過測試的是 Linux arm64（交叉編譯）與 macOS x64；測試跑的是建置產物，不是打包好的 npm 套件
- 背景段落不成立的一句：流程設定的值不合法時，只有 `locale` 與 `spec_locale` 是 `invalid_config`；`tdd`、`audit`、`worktree` 與 `review baseline` 的值不合法是 `invalid_argv`
- 前幾輪討論沒有否決過這七項裡的任何方向
**Open**: F1 要不要先單獨發版；F2 匯出的形狀（只給名稱，或加「會不會寫入」）；F3 的範圍（只有匯出與匯入，或全部動詞）與原因清單；O1 補文件或改名；O2 改程式或改文件；O3 補在哪裡；O4 要不要開變更；怎麼切變更與發版

### Round 2 — assumptions (2026-11-20)

**Focus**: 八件要決定的事各自怎麼定
**Position**: 八項都照建議定案，切成兩個變更。
- F1：修掉那一行，並在 node-sdk.yml 的打包工作加檢查——把打包好的主套件裝進暫存資料夾，編譯一支匯入全部匯出名稱的 TypeScript 檔（`tsc --noEmit`，不開 skipLibCheck）；先單獨發 0.13.1
- F2：SDK 匯出 `commandNames`，56 個唯讀字串，含 `scope-import` 與 `scope-export`；引擎的 54 個來自 describe.rs 的一份常數，由既有的逐命令對照測試釘住「清單等於全部命令」
- F3：`reason` 只加在 `engine.exportScope`、`host.exportScope`、`host.importScope` 三個呼叫的失敗，是 Error 上選填的字串；清單固定並寫進文件；不在清單裡的失敗（儲存錯誤、掛鉤拒絕）不帶，訊息維持原樣
- F3：原因清單以來源文件的六個為底；程式裡另外兩種匯出失敗（資料夾無法列出、變更的 metadata 解析失敗）要不要給代碼，提案時逐一對照
- O1：欄位不改名；SDK 文件那句「欄位是 camelCase」改成符合現況，動詞契約文件列出全部例外
- O2：改程式，內嵌時 `new artifact` 的 `path` 是鍵；同一個函式也供 Server 使用，提案時先確認遠端 CLI 的輸出會不會跟著變
- O3：行為不動，`set --help` 與 SDK 文件各補一句
- O4：不開變更，把 CI 的現況回覆給宿主
- 切法：第一個變更只有 F1；第二個變更收 F2、F3、O1、O2、O3
**Ruled out**: `commandNames` 同時帶「會不會寫入」 — 不寫入不等於可以放行，`scope-export`、`workflow-config show`、`discuss show` 都不寫入但都帶出全文，旗標會引導宿主寫出「讀取一律放行」的預設；全部動詞的失敗都加 `reason` — 失敗點有上百個，每個都給穩定代碼等於多凍結一份契約，引擎的設計也寫明細分類留在外層；把 snake_case 欄位改名成 camelCase — 這些名稱被 verb-contract 與 change-analysis 凍結，技能與 Speclink Desktop 都在讀；O2 只改文件 — 要連 node-sdk 的那條需求一起改寫，而且 `discuss new` 已經回傳鍵，同一個宿主裡會有兩種 path；F1 等其他項一起發版 — 宿主要多帶修補檔一段時間
**Open**: 沒有

## Conclusion

**Decision**: 三項需求都做，四個觀察裡三個要動（O1 補文件、O2 改程式、O3 補說明），O4 不開變更；切成兩個變更，第一個先單獨發 0.13.1。
- 來源文件的現況描述大多成立；不同處：Windows x64 與 Linux x64 的建置產物在 Speclink 自己的 CI 已經跑整套測試，沒跑過的是 Linux arm64 與 macOS x64；流程設定的值不合法時只有 `locale`、`spec_locale` 是 `invalid_config`，其餘是 `invalid_argv`；內嵌時 `new artifact` 的 `path` 不只和文件不符，也和 node-sdk 的正式規格不符
- **cut 1 `sdk-types-parse-check`**: 修掉型別定義檔的那一行，並在打包流程加上「型別定義檔能被編譯」的檢查（F1）
  - `crates/adapters/speclink-node/index.d.ts` 第 524 行的 `'*/*'` 改成 `'*\/*'`：字串的值不變，註解不再提早結束
  - `.github/workflows/node-sdk.yml` 的打包工作加一步：把打包好的主套件裝進暫存資料夾，編譯一支匯入全部匯出名稱的 TypeScript 檔，`tsc --noEmit`，不開 skipLibCheck；失敗就讓工作失敗
  - 這一步同時確認打包好的套件裝得起來，補上「測試跑的是建置產物，不是打包好的套件」的缺口
  - 封存後單獨發 0.13.1
- **cut 2 `sdk-command-names-and-transfer-reasons`**: 命令名稱清單、匯出與匯入的原因代碼，以及三個觀察的修正（F2、F3、O1、O2、O3）
  - F2：SDK 匯出 `commandNames`，唯讀的字串陣列，56 個，含宿主套件的 `scope-import` 與 `scope-export`
  - F2：引擎的 54 個名稱來自 describe.rs 的一份常數，由既有的逐命令對照測試釘住「清單等於全部命令」；清單不帶「會不會寫入」
  - F3：`engine.exportScope`、`host.exportScope`、`host.importScope` 的失敗在 `code` 之外多帶選填的字串屬性 `reason`；清單固定並寫進 SDK 文件與型別說明；訊息維持原樣
  - F3：清單以 `no_spec_folder`、`symlink_in_spec_folder`、`spec_folder_outside_project`、`scope_not_empty`、`bundle_too_large`、`invalid_bundle_document` 為底；資料夾無法列出、變更的 metadata 解析失敗這兩種匯出失敗要不要給代碼，提案時逐一對照程式
  - F3：不在清單裡的失敗（儲存錯誤、掛鉤拒絕、匯出時範圍一直在變）不帶 `reason`；一般動詞的失敗不加這個欄位
  - F3：SDK 文件裡要宿主比對訊息開頭的句子，改成指向 `reason`
  - O1：`--json` 的欄位不改名；SDK 文件「欄位是 camelCase」那句改成符合現況，docs/verb-contract 列出全部 snake_case 的例外（`task done`／`task undone` 的 `task_id`、`task_desc`，`discuss archive` 的 `archived_to`，`analyze` 的 `change_id`、`finding_count` 等；提案時掃過全部輸出補齊）
  - O2：內嵌時 `new artifact` 結果的 `path` 改成儲存的鍵（`changes/<name>/<artifact>`），和 `discuss new` 一致；`port_view` 的同一個函式也供 Server 使用，提案時先確認遠端 CLI 的輸出與「Artifact already exists」訊息會不會跟著變，再定改在哪一層
  - O3：行為不動；`workflow-config set --help` 與 SDK 文件各補一句「空字串把 `locale`、`spec_locale` 設回未設定」
- O4 不開變更：回覆宿主 CI 的現況（Windows x64、Linux x64、macOS arm64 跑整套測試；Linux arm64 與 macOS x64 只建置）
- 例子：`import { commandNames } from '@speclink/engine'` 得到 56 個名稱，含 `new-change`、`claim-release`、`scope-import`；宿主的測試拿它和自己的對照表比對，升級後多出來的命令讓測試失敗
- 例子：`host.importScope` 匯入到已有文件的範圍，Error 的 `code` 是 `refused`、`reason` 是 `scope_not_empty`、`message` 仍是 `import (create-new): the target scope already holds documents`；掛鉤拒絕的 Error 同樣是 `refused`，但沒有 `reason`
**Rationale**: F1 讓 TypeScript 的宿主過不了型別檢查，修法只有一行，所以不等其他項、先發版；真正要防的是同類的錯再發生，檢查因此放在打包好的套件上，那才是宿主拿到的東西。F2 與 F3 都是「引擎早就有這份資訊，只差一個穩定的出口」：命令名稱是對外契約卻沒有清單，匯入的失敗有型別卻只送出訊息。出口刻意開得窄：清單只給名稱，原因代碼只給匯出與匯入，因為每多一個穩定的欄位就多凍結一份契約，而宿主這一輪的用途只有對照表的測試與背景匯入的管理畫面。三個觀察照「誰錯就改誰」處理：O1 是文件說得太滿，名稱已凍結所以改文件；O2 是程式違反正式規格的承諾，所以改程式；O3 是刻意的行為少一句說明。
**Rejected alternatives**:
- `commandNames` 同時帶「會不會寫入」 — 不寫入不等於可以放行，`scope-export`、`workflow-config show`、`discuss show` 都帶出全文，旗標會引導宿主寫出「讀取一律放行」的預設
- 全部動詞的失敗都加 `reason` — 失敗點有上百個，每個都給穩定代碼等於多凍結一份契約；引擎的設計寫明細分類留在外層
- 把 snake_case 的欄位改名成 camelCase — 名稱被 verb-contract 與 change-analysis 凍結，技能與 Speclink Desktop 都在讀
- O2 只改文件 — 要連 node-sdk「path 是儲存的鍵」的需求一起改寫，而且同一個宿主裡 `discuss new` 已經回傳鍵
- 拒絕空字串的 `locale` — workflow-config 的規格明寫空字串移除鍵，是把欄位設回未設定的唯一做法
- F1 等其他項一起發版 — 宿主要多帶修補檔與 lockfile 的修補雜湊一段時間
- 用 `agentTool.description` 代替命令名稱清單 — 它列的是動詞，和命令名稱不是一對一（`claim` 與 `claim --release` 是兩個命令名稱）
**Deferred**:
- Linux arm64 與 macOS x64 的平台套件沒有跑過測試 — 宿主這一輪沒有要求，等宿主的 CI 與部署回報再決定
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion embedded-host-needs-4
