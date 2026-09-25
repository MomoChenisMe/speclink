---
title: 本手冊的來源
section: 附錄
order: 900
keywords: [來源, 矛盾, 限制, 編纂日期, 規格]
sources: []
generated: 2026-09-25T08:39:09+08:00
---

# 本手冊的來源

## 取材範圍

本手冊全部內容只取材自 `openspec/specs/` 底下的正式規格。README、`docs/` 與原始碼都不是來源。規格沒寫的，手冊就不寫，或在該處標明「規格未載」。每一頁的最後一行列出它取材的能力名稱。

正式規格共 86 個能力。其中 59 個是使用者會操作或看到的東西（畫面、指令、技能、輸出、檔案），已入冊。其餘 27 個是引擎內部（儲存、wire 契約、host 執行期、測試骨架、建置與發布管線、本 repo 自用的發版技能與更新日誌的資料契約），不入冊：

`client-protocol`、`command-runtime`、`delivery-baseline`、`desktop-release`、`dev-harness`、`host-runtime`、`node-sdk`、`node-sdk-release`、`phase2-acceptance`、`phase3-acceptance`、`postgres-team-store`、`reference-server`、`release-notes`、`release-skill`、`remote-board-order`、`remote-workspace-data`、`server-context-api`、`server-drift-api`、`server-event-stream`、`server-read-api`、`server-verb-api`、`serverfs-team-store`、`sqlite-team-store`、`store-abstraction`、`teamstore-contract`、`ui-copy-vocabulary`、`workspace-session`。

`release-notes` 定義的更新日誌，使用者看得到的那一面（桌面 app 的更新日誌彈窗）由 `desktop-app` 載明，寫在[自動更新、安裝 CLI 與指令檔過期](desktop-update.md)。

## 旅程主幹

手冊的章節順序轉寫自兩份驗收劇本規格：

- `phase2-acceptance`：setup 開箱 → 邀請與存取金鑰 → 提案 → 政策 → 完成任務 → 規格投影 → 漂移 → 封存。
- `phase3-acceptance`：PM 無 checkout → RD 有 checkout → 多 server → 多分頁 → 失聯與恢復。

劇本沒有涵蓋的站（基準盤點、討論、審查、驗證、品質關卡合跑、提交、worktree、操作手冊），順序依 `skill-routing` 的入口情境與交棒邊表補齊：基準盤點放在 SDD 工作流最前，操作手冊放在工具技能之末。「開始使用」與「桌面 app」兩章依功能領域排列，沒有對應的劇本。

## 規格內的矛盾

以下是規格之間、或同一規格內新舊說法不一致的地方。內文一律採 `@trace updated` 日期較晚的說法，這裡照實記錄，不另行裁決。實際行為以產品為準。

1. **政策解析層數**：`workflow-config` 的 Purpose 寫「四層解析：環境變數 ＞ .speclink.yaml 舊鍵 ＞ 正典檔 ＞ 內建預設」且舊鍵命中會出警告；同規格的需求「工作流政策的正典歸屬與三層解析順序」（2026-08-23）改為三層，.speclink.yaml 的同名鍵一律不生效、不出警告。內文採三層。
2. **啟用資料夾是否產生 CLAUDE.md**：`desktop-config`「未啟用資料夾經確認後補齊啟用」（2026-07-31）寫啟用會產生 CLAUDE.md 的受管區塊；`workspace-tools`「工作區補齊入口」（2026-08-23）與 `desktop-config`「未初始化目錄經確認後自動初始化」（2026-08-23）都寫不產生 CLAUDE.md。內文採不產生。
3. **綁定 checkout 後產生什麼**：`workspace-chooser`（2026-07-24）寫綁定後會生成 Skills 與 AGENTS.md／CLAUDE.md 的 Speclink 區塊；`workspace-tools`「built-in tools 權威收斂」（2026-08-23）寫只生成技能檔，並剝除指令檔裡遺留的 SPECLINK 區塊。內文採只生成技能檔。
4. **指令檔過期怎麼判**：`desktop-app`「指令檔過期提示」（2026-08-06）以 CLAUDE.md 是否存在、SPECLINK 標記是否被移除來判；`workspace-tools`「技能檔過期探測」（最新 2026-09-07）改以 skills 目錄下有無 speclink- 技能檔、以及技能檔版號比對來判，自訂描述子也納入。內文採技能檔版號。
5. **專案設定頁有幾個頁簽**：`desktop-config`「設定頁圖形化讀寫兩層設定」（2026-08-23）寫本地兩簽（config.yaml、.speclink.yaml）、remote 單一 Workflow 簽；同規格「設定頁的產出流程頁籤」（2026-08-22）與其後六條產出流程需求寫本地三簽（config.yaml → Schema → .speclink.yaml）、remote 兩簽（Workflow → Schema）。內文的頁簽列採兩簽版；產出流程的內容仍照 2026-08-22 那組需求列出，並註明日期。
6. **唯讀角色叫什麼**：`server-policy-write`（2026-07-20）、`user-documentation`（2026-07-24）與 `desktop-config`（2026-08-23）寫 reader；`server-identity` 的一條 scenario（2026-07-28）寫 viewer。內文採 reader（最晚一次提到的是 2026-08-23）。
7. **server 的官方發布物**：`server-release`「Server 交付物內嵌同版本 SPA 資產」（2026-07-25）寫 release binary 與「tag 觸發 server binary 與 Docker image 發布」；同規格「release 產物含 server 與部署文件」（2026-08-14）寫 server binary 不上傳 GitHub Release，官方通路只有 Docker 映像與 npm 套件。內文採後者。
8. **討論結論後的路**：`user-documentation`「討論結論後的轉出與併入分流完整」（2026-07-17）把 `speclink discuss promote` 列為結論後四條路之一；`discuss-skill`「結論後交棒單推 propose 入口」（2026-08-27）與 `skill-routing` 交棒邊表（2026-09-01）寫結論後只建議 `/speclink-propose --from-discussion`，promote 留給中途轉出。內文採後者。
9. **verify 是不是可呼叫的站**：`user-documentation`「Getting Started 僅使用已驗證入口」（2026-07-17）寫入門文件不要求呼叫未安裝的 `$speclink-verify`；同規格「工作流正典逐站列出技能與完成判準」（2026-09-01）與 `verify-skill`（2026-08-11）把 verify 列為正式站。內文採 verify 是可呼叫的站。
10. **討論隨變更封存的條件**：`user-documentation`（2026-07-17）寫最後一個存活變更封存時討論一併封存；`discussion-docs`「討論以 link 動詞併入既有變更」（最新 2026-09-09）加了前提：討論的結論必須已寫入、記錄沒有 `hold: true`，且判定「無其他變更引用」時壞掉的狀態檔視為仍在引用。內文採有前提的版本。
11. **使用者文件可否連到架構文件**：`user-documentation`「目標架構與目前狀態維持清楚邊界」（2026-07-17）要求 README 連到平台架構藍圖與路線圖；同規格「使用者面路線圖與內部交付順序分列」（2026-08-14）寫使用者文件不得引用那兩份維護者文件。與手冊內容無關，僅記錄。
12. **點規格卡會發生什麼、溯源顯示在哪**：`desktop-app`「規格頁提供清單、搜尋與展開檢視」（2026-07-09）寫點卡片標題就地展開全文、下方帶一行來源變更；同規格「桌面 app 呈現 change 與 spec 的清單與內容」的原版（2026-07-11）寫點卡片開啟唯讀的規格詳情面板、清單不提供行內展開，其後的面板互斥（2026-07-17）與卡片收合（2026-08-11）需求也以面板為前提；同一條需求的最新版（2026-09-03）再把溯源變更改為標頭出身列的籤，內文底部不再有溯源文字行。內文採詳情面板與出身列的籤。
13. **品質關卡狀態列顯示什麼**：`desktop-app`「詳情抽屜的審查資訊列」（2026-08-04）與「詳情抽屜的驗證資訊列」（2026-08-06）寫狀態列顯示狀態詞、蓋章時間與審查者／驗證者；同規格「變更詳情抽屜標頭的四層結構」（2026-08-07）寫日期與蓋章者收進指標停留提示、可視文字不直出日期與 email。內文採後者。
14. **封存時未結工單的第一個選項叫什麼**：`desktop-app`「封存入口的未結工單三選項」（2026-08-02）描述為「前往完成蓋章」；同規格「變更與討論抽屜開啟時底層落回看板」（2026-08-11）稱同一個按鈕為「去蓋章」。內文以「去蓋章」為按鈕字面。
15. **討論開場淺掃有幾段**：`discuss-skill`「事實與決策分診及逐節點查證」（2026-08-21）把開場偵察規定為「正式規格 → 程式碼」兩段漏斗；同規格「開場舊討論查核與第四類對照」（2026-09-05）改為「正式規格 → 舊討論查核 → 程式碼」三段，並在假設清單的三分對照之外加入第四類「舊討論已定案」。內文採三段與四類。
16. **保留在途的討論，卡片與詳情面板有沒有封存動作**：`desktop-app`「討論抽屜檢視與轉出變更」（2026-08-04）寫已結論且未封存的討論，在討論卡與討論詳情面板都有封存動作；同規格「討論於看板第 0 欄兩級呈現」（2026-09-10）寫已轉出、已結論但保留在途的卡片不提供任何動詞按鈕，收尾由 CLI 的 `speclink discuss archive` 明示解除，對詳情面板沒有另作規定。內文採卡片沒有按鈕；詳情面板照舊寫規格所載。
17. **propose 收尾盤點的母體與做法**：`skill-routing` 交棒邊表的 propose 列（2026-09-16T16:37:51+08:00）寫「提案中變更 ≥2 時先盤點執行順序，worktree 政策開啟時分可平行／須依序」；`propose-skill`「收尾盤點提案中變更的執行順序」（最新 2026-09-24T22:20:29+08:00）寫以「作用中變更」為母體、對本次建立的變更判定軟依賴並以 `speclink change depends` 落檔，再判定插隊、以 `speclink plan` 的波次呈現。後者時戳較晚，[提案](propose.md)的收尾一節照 `propose-skill` 寫。[工作流總覽](workflow-overview.md)的交棒邊表照 `skill-routing` 的摘要寫；該頁的來源沒有更新，本次沒有重寫，兩頁互相指路。
18. **delta 重疊還算不算阻擋**：`change-plan` 的 Purpose 寫「以宣告依賴與 delta capability 重疊做拓樸修正，算出波次與每個 change 的阻擋清單」；同規格「執行順序的基底與拓樸修正」（2026-09-24T22:20:29+08:00）改為 delta 重疊不推後波次、不構成有向邊、不進阻擋清單，改以同名 requirement 重疊算封存順序，只有 `speclink plan --strict-overlap` 退回舊算法。Purpose 也沒有提到新的 `speclink change rank`。內文採需求段的說法。
19. **拖排時被拖的卡缺順序鍵要不要整欄補章**：`board-card-order` 的 Purpose 寫「欄內出現缺 rank 的卡時整欄補章」；同規格「欄內存在缺 rank 卡時整欄補章」（2026-09-24T22:20:29+08:00）排除被拖的卡本身：只有它缺鍵時只寫它一檔。內文採需求段的說法。
20. **前置要一次落檔還是逐一落檔**：`ingest-skill`「ingest 收尾重判本變更的軟依賴」（2026-09-24T22:20:29+08:00）要求每個前置各執行一次 `speclink change depends`，並規定這一段與 propose 收尾的同一段逐字一致；`propose-skill`「收尾盤點提案中變更的執行順序」（同一時戳、同一次封存）寫的是一次帶多個前置的指令，也沒有提到被拒時的處置。兩者時戳相同、出自同一次封存，無法依較晚者裁定。[提案](propose.md)照 `propose-skill` 寫，[續作與需求變更](drift-ingest.md)照 `ingest-skill` 寫，提案頁以附註指出差異。

附註（規格自己宣告的例外，不是矛盾）：

- `worktree-overlay`（2026-08-04）讓 `speclink list` 讀到壞掉的政策檔時照常輸出，與 `workflow-config` 的「壞檔即拒絕」不同；規格自述為僅限觀察面的例外。
- `change-lifecycle`（2026-07-27）寫 remote 模式下 `speclink discard` 回報不支援；`server-verb-api`（2026-07-23）寫 server 提供 discard 語意的刪除端點。一個講 CLI 動詞、一個講 server 端點。
- `review-station`（2026-08-03）對審查工單的續輪沒有寫碼任務守門；`verify-station`（2026-08-11）對驗證工單的續輪有。兩站不對稱，各頁照各自規格寫。
- `propose-skill` 把「只動到同一段程式碼」列為要落檔的軟依賴；`ingest-skill` 則規定只動到同一段程式碼不記為前置、改向使用者提出，理由是本變更若已在進行中，等一個還沒開工的變更會卡住它。兩者時機不同（新建的變更一定還沒開工），各頁照各自規格寫。
- `desktop-app` 早期需求（2026-07-05、2026-07-17）寫「歸檔」，後期一律寫「封存」；規格也把面板稱為「抽屜」、把品質關卡稱為「品質站」。內文依 `openspec/LANGUAGE.md` 的正典詞彙一律用「封存」「詳情面板」「品質關卡」，引用需求名稱時保留原字。

## 已知限制

- 沒有截圖。畫面文字與按鈕名稱逐字取自規格，實際畫面若不同，以執行中的產品為準。
- drift、audit 兩個技能沒有各自的規格，手冊只寫 `skill-routing` 與 `user-documentation` 載明的入口情境與交棒關係。apply 只有挑選變更的第一步（plan 守門）有規格，載於 `change-plan`；ingest 只有收尾的依賴與插隊判定有規格，載於 `ingest-skill`；兩者其餘的內文行為未載。analyze 的技能本身也沒有規格，但 `speclink analyze` 指令的判定規則有 `change-analysis`，寫在[分析：交叉檢查變更的產物](analyze.md)。
- 過期判定逐項比較：頁的生成時戳與規格的更新時戳都帶時區時比到秒，同一秒不算；任一邊只有純日期時比到日，同一天也算。每一頁的 `generated` 都寫成帶時區的秒級時戳；較早封存的規格，其更新時戳仍是純日期，封存不會回改。只取材某規格幾段需求的頁，`sources` 用「capability#需求名」錨定到那幾段，別段的封存不會把它標成可能過期；錨定的需求標題改名或移除時，該頁會被標為可能過期並整頁重寫。
- 來源規格改了、但改的部分與某一頁的內容無關時，那一頁重生後內文不變，只換生成時戳，「可能過期」標記就消掉。取材自 `desktop-app` 的五頁（[認識桌面 app](desktop-overview.md)、[規格、討論、已封存與搜尋](desktop-browse.md)、[桌面上的品質關卡](desktop-quality.md)、[看板與任務](desktop-board.md)、[自動更新、安裝 CLI 與指令檔過期](desktop-update.md)）都已改成錨定寫法，各自只錨定它取材的需求段；[自動更新、安裝 CLI 與指令檔過期](desktop-update.md) 對 `workspace-tools` 也一併錨定。2026-09-17 這次，[看板與任務](desktop-board.md) 為排程相關的四段需求新增錨定，[工作流總覽](workflow-overview.md)、[實作](apply.md)、[封存](archive.md) 各自只錨定 `change-plan` 裡與該頁相關的一段。 2026-09-22 這次只重生[討論](discuss.md)一頁，納入 `discuss-skill` 與 `improve-skill` 新增的結論條列規則。2026-09-25 這次重生六頁的相關段落，反映同名 requirement 重疊與封存順序、`speclink change rank`、沒有順序鍵的卡改依被依賴數與任務數排，以及排程分頁改成四張卡片：[執行順序：plan 與依賴](plan.md)整頁重寫，[封存](archive.md)、[提交單一變更的檔案](commit.md)、[續作與需求變更](drift-ingest.md)、[提案](propose.md)、[看板與任務](desktop-board.md)逐段改寫；[實作](apply.md)的來源 `verb-contract` 只新增了 remote 模式下兩個只限本機的子情形，與該頁內文無關，只換生成時戳。
- 錨定寫法有一個盲點：某能力新增一段需求時，沒有任何頁錨定到它，過期判定不會亮。2026-09-22 封存進 `desktop-app` 的兩段需求「詳情抽屜的工單分頁」與「已封存抽屜的工單分頁」就是這種情況，尚未入冊；補寫時以範圍提示重生[桌面上的品質關卡](desktop-quality.md)。
- 規格裡的內部識別符（欄位名、型別名、旗標）不進手冊，改以白話描述效果。

## 編纂日期

2026-09-25

**出處**：本頁為說明頁，不直接取材自單一能力；各頁末行列出自己的出處。
