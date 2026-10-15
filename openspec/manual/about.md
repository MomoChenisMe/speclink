---
title: 本手冊的來源
section: 附錄
order: 900
keywords: [來源, 矛盾, 限制, 編纂日期, 規格]
sources: []
generated: 2026-10-15T12:36:56+08:00
---

# 本手冊的來源

## 取材範圍

本手冊全部內容只取材自 `openspec/specs/` 底下的正式規格。README、`docs/` 與原始碼都不是來源。規格沒寫的，手冊就不寫，或在該處標明「規格未載」。每一頁的最後一行列出它取材的能力名稱。

正式規格共 87 個能力。其中 59 個是使用者會操作或看到的東西（畫面、指令、技能、輸出、檔案），已入冊。其餘 28 個是引擎內部（儲存、wire 契約、host 執行期、測試骨架、建置與發布管線、本 repo 自用的發版技能、更新日誌的資料契約、技能原稿的寫法規範），不入冊：

`client-protocol`、`command-runtime`、`delivery-baseline`、`desktop-release`、`dev-harness`、`host-runtime`、`node-sdk`、`node-sdk-release`、`phase2-acceptance`、`phase3-acceptance`、`postgres-team-store`、`reference-server`、`release-notes`、`release-skill`、`remote-board-order`、`remote-workspace-data`、`server-context-api`、`server-drift-api`、`server-event-stream`、`server-read-api`、`server-verb-api`、`serverfs-team-store`、`skill-authoring`、`sqlite-team-store`、`store-abstraction`、`teamstore-contract`、`ui-copy-vocabulary`、`workspace-session`。

`release-notes` 定義的更新日誌，使用者看得到的那一面（桌面 app 的更新日誌）由 `desktop-app` 載明，寫在[自動更新、CLI、技能檔過期與關於頁](desktop-update.md)。`desktop-app` 裡有兩段需求講前端元件怎麼共用（「前端元件庫與資料源解耦」「共用元件唯一來源」），屬於內部結構，沒有任何一頁取材。

## 旅程主幹

手冊的章節順序轉寫自兩份驗收劇本規格：

- `phase2-acceptance`：setup 開箱 → 邀請與存取金鑰 → 提案 → 政策 → 完成任務 → 規格投影 → 漂移 → 封存。
- `phase3-acceptance`：PM 無 checkout → RD 有 checkout → 多 server → 多分頁 → 失聯與恢復。

劇本沒有涵蓋的站（基準盤點、討論、審查、驗證、品質關卡合跑、提交、worktree、操作手冊），順序依 `skill-routing` 的入口情境與交棒邊表補齊：基準盤點放在 SDD 工作流最前，操作手冊放在工具技能之末。「開始使用」與「桌面 app」兩章依功能領域排列，沒有對應的劇本。

## 規格內的矛盾

以下是規格之間、或同一規格內新舊說法不一致的地方。內文原則上採 `@trace updated` 時戳較晚的說法；沒有照這個原則寫的，下面會寫明理由。這裡照實記錄，實際行為以產品為準。

1. **政策解析層數**：`workflow-config` 的 Purpose 寫「四層解析：環境變數 ＞ .speclink.yaml 舊鍵 ＞ 正典檔 ＞ 內建預設」且舊鍵命中會出警告；同規格的需求「工作流政策的正典歸屬與三層解析順序」（2026-08-23）改為三層，.speclink.yaml 的同名鍵一律不生效、不出警告。內文採三層。
2. **綁定 checkout 後產生什麼**：`remote-connection`「Remote Workspace bootstrap 跨入口一致性」（2026-07-24）寫 CLI 的 remote init 與桌面綁定會產生 AGENTS.md／CLAUDE.md 的 Speclink 區塊；`workspace-chooser`「checkout 綁定驗證與 marker 寫入」與 `workspace-tools`「built-in tools 權威收斂」（都是 2026-10-08T11:52:22+08:00），以及 `remote-connection` 自己的「remote 初始化與連接指令」（2026-08-23），都寫只生成技能檔、並剝除指令檔裡遺留的 SPECLINK 區塊。內文採只生成技能檔。
3. **可選的 AI 工具有幾個**：`remote-connection`「remote 初始化與連接指令」（2026-08-23）寫 remote init 要一個 Claude／Codex 的選集，非互動終端只指引 `--tools claude`、`codex` 或 `claude,codex`；同規格「Remote Workspace bootstrap 跨入口一致性」（2026-07-24）也寫「相同 Claude／Codex 選集」。`workspace-tools`「init 內建 Agent 工具選擇」（2026-10-08T11:52:22+08:00）寫所有 `speclink init`（含 remote）都接受 claude、codex、copilot 三個，`workspace-chooser` 與 `desktop-config` 的勾選框也已是三個。內文採三個，[CLI 連接 remote](cli-remote.md) 以附註指出差異。
4. **技能檔過期怎麼判**：`desktop-app`「指令檔過期提示」（2026-10-10T23:13:14+08:00）的情境仍以「CLAUDE.md 不存在就提示安裝」「指令檔的 SPECLINK 標記被移除不提示」舉例；`workspace-tools`「技能檔過期探測」（2026-10-08T11:52:22+08:00）以 skills 目錄下的 speclink- 技能檔與版號判定。`desktop-app` 時戳較晚，但那次改的是提示卡的位置與按鈕；它的需求本文只說「執行指令檔過期探測」，不定義判定方式。內文照探測規格寫。
5. **唯讀角色叫什麼**：`server-policy-write`（2026-07-20）、`user-documentation`「帳號、PAT 與 membership 的操作邊界明確」（2026-10-08T10:00:10+08:00）、`desktop-config`「設定頁圖形化讀寫兩層設定」（2026-10-14T16:37:17+08:00）與 `server-web-console`「管理列表以詳情頁與對話框承載檢視與建立」（2026-10-15T10:20:04+08:00）寫 reader；`server-identity`「帳號 browser API 保持憑證祕密邊界」的一條 scenario（2026-07-28）與 `server-verb-api`「task done 消費 touchedFiles 且 evidence 有唯讀端點」（2026-08-25）寫 viewer。內文採 reader。
6. **server 的官方發布物**：`server-release`「Server 交付物內嵌同版本 SPA 資產」（2026-07-25）寫 release binary 與「tag 觸發 server binary 與 Docker image 發布」；同規格「release 產物含 server 與部署文件」（2026-08-14）寫 server binary 不上傳 GitHub Release，官方通路只有 Docker 映像與 npm 套件。內文採後者。
7. **討論結論後的路**：`user-documentation`「討論結論後的轉出與併入分流完整」（2026-07-17）把 `speclink discuss promote` 列為結論後四條路之一；`discuss-skill`「結論後交棒單推 propose 入口」（2026-08-27）與 `skill-routing`「出口交棒由技能結尾承載」的交棒邊表（2026-10-14T21:31:03+08:00）寫結論後只建議 `/speclink-propose --from-discussion`，promote 留給中途轉出。內文採後者。
8. **verify 是不是可呼叫的站**：`user-documentation`「Getting Started 僅使用已驗證入口」（2026-10-08T11:52:22+08:00）的 Codex 情境仍寫入門文件不要求呼叫未安裝的 `$speclink-verify`；`skill-routing`「出口交棒由技能結尾承載」（2026-10-14T21:31:03+08:00）把 verify 列為流程鏈上的技能，`workspace-tools`「update 清除孤兒技能目錄」（2026-10-08T11:52:22+08:00）寫 Codex 拿到 analyze 以外的全部技能。內文採 verify 是可呼叫的站。
9. **討論隨變更封存的條件**：`user-documentation`（2026-07-17）寫最後一個存活變更封存時討論一併封存；`discussion-docs`「討論以 link 動詞併入既有變更」（2026-09-09T22:31:02+08:00）加了前提：討論的結論必須已寫入、記錄沒有 `hold: true`，且判定「無其他變更引用」時壞掉的狀態檔視為仍在引用。內文採有前提的版本。
10. **封存時未結工單的第一個選項叫什麼**：`desktop-app`「封存入口的未結工單三選項」（2026-10-10T15:55:28+08:00）描述為「前往完成蓋章」；同規格「變更與討論詳情頁的宿主為看板」（2026-10-14T13:25:41+08:00）稱同一個按鈕為「去蓋章」。內文以「去蓋章」為按鈕字面。驗證那一組的第一個選項，「封存入口三選項擴及驗證工單」（2026-08-06）寫「前往完成驗證蓋章」，規格沒有別處給出字面，內文照抄。
11. **propose 收尾盤點的母體與做法**：`skill-routing`「出口交棒由技能結尾承載」的交棒邊表 propose 列寫「提案中變更 ≥2 時先盤點執行順序，worktree 政策開啟時分可平行／須依序」；`propose-skill`「收尾盤點提案中變更的執行順序」（2026-09-24T22:20:29+08:00）寫以「作用中變更」為母體、對本次建立的變更判定軟依賴並以 `speclink change depends` 落檔，再判定插隊、以 `speclink plan` 的波次呈現。`skill-routing` 那段的時戳現為 2026-10-14T21:31:03+08:00，比 `propose-skill` 晚；但那次封存只改了表中 worktree 兩列，propose 列的文字沒動，時戳是整段重新蓋章帶上來的。[提案](propose.md)本次沒有過期、沒有重寫，仍照 `propose-skill` 寫；[工作流總覽](workflow-overview.md)照 `skill-routing` 的摘要寫，兩頁互相指路。
12. **delta 重疊還算不算阻擋**：`change-plan` 的 Purpose 寫「以宣告依賴與 delta capability 重疊做拓樸修正，算出波次與每個 change 的阻擋清單」；同規格「執行順序的基底與拓樸修正」（2026-09-24T22:20:29+08:00）改為 delta 重疊不推後波次、不構成有向邊、不進阻擋清單，改以同名 requirement 重疊算封存順序，只有 `speclink plan --strict-overlap` 退回舊算法。Purpose 也沒有提到 `speclink change rank`。內文採需求段的說法。
13. **拖排時被拖的卡缺順序鍵要不要整欄補章**：`board-card-order` 的 Purpose 寫「欄內出現缺 rank 的卡時整欄補章」；同規格「欄內存在缺 rank 卡時整欄補章」（2026-09-24T22:20:29+08:00）排除被拖的卡本身：只有它缺鍵時只寫它一檔。內文採需求段的說法。
14. **前置要一次落檔還是逐一落檔**：`ingest-skill`「ingest 收尾重判本變更的軟依賴」（2026-09-24T22:20:29+08:00）要求每個前置各執行一次 `speclink change depends`，並規定這一段與 propose 收尾的同一段逐字一致；`propose-skill`「收尾盤點提案中變更的執行順序」（同一時戳、同一次封存）寫的是一次帶多個前置的指令，也沒有提到被拒時的處置。兩者時戳相同、出自同一次封存，無法依較晚者裁定。[提案](propose.md)照 `propose-skill` 寫，[續作與需求變更](drift-ingest.md)照 `ingest-skill` 寫，提案頁以附註指出差異。
15. **ingest 技能規格管到哪**：`ingest-skill` 的 Purpose 寫只涵蓋 ingest 收尾的依賴判定、來源解析仍由技能本文承載；同規格「ingest 參數可指定要更新的變更」（2026-10-07T20:24:30+08:00）已規定參數怎麼解析。內文採需求段。
16. **否決替代案的分隔號**：`discuss-skill`「討論記錄的樹慣例與格式不變」與 `improve-skill`「conclude 範例與討論結論條列規則同形」（都是 2026-10-08T16:45:46+08:00）寫「方案——落敗理由」；`desktop-app`「討論結論以欄位標籤呈現」（2026-10-13T22:22:52+08:00）寫 discuss 技能檔的寫法為前後帶空白的「方案 — 理由」，桌面 app 兩種分隔號都認得。內文採後者。
17. **審查關卡什麼時候讀審查規則**：`quality-skill`「兩站時序的編排行為」（2026-10-14T21:31:03+08:00）列審查關卡的準備步驟時，把「讀專案審查規則」排在鎖定範圍之後；`review-skill`「專案審查規則的讀取與套用」（2026-10-14T16:37:17+08:00）寫開審前先讀規則，讀不到就停、不算範圍。`quality-skill` 只管兩站合跑，所以[品質關卡總覽](quality-stations.md)照 `quality-skill` 的順序寫合跑，[審查站](review.md)照 `review-skill` 寫單站。
18. **派不出子代理時怎麼辦**：`quality-skill`「兩站時序的編排行為」（2026-10-14T21:31:03+08:00）寫依兩站既有的「無法派子代理即依序自跑」退路執行，但 `review-skill` 與 `verify-skill` 都沒有這條退路；同一段需求的本文寫「無法同時派出三個子代理」，它的 scenario 寫「執行環境無法派出子代理」。內文照 scenario 寫：派不出子代理時，技能依序自己跑三份檢查。
19. **分析結果卡放在哪**：`desktop-app`「桌面 app 提供動詞操作面」（2026-10-10T15:55:28+08:00）寫結果在詳情頁「狀態」卡之下；同規格「變更詳情頁的標頭與狀態卡」（2026-10-13T23:23:36+08:00）把狀態卡改成左欄「狀態」列，分析結果以一張卡出現在閱讀卡之上。內文採後者。
20. **生命週期階段的顏色**：`desktop-app`「介面狀態語意色分層」（2026-10-12T16:18:33+08:00）與「看板欄與卡片的容器外觀」（2026-10-09T18:18:49+08:00）寫提案中、進行中、已就緒三欄的色相是主色、藍、綠；同規格「討論於看板第 0 欄兩級呈現」（2026-10-13T12:56:33+08:00）寫已轉出細列的階段籤「各對應該階段欄的 teal 濃度」。後者時戳較晚，但它講的是階段籤，不是欄的色相。[認識桌面 app](desktop-overview.md)的欄色照「介面狀態語意色分層」寫；[看板與任務](desktop-board.md)的階段籤只寫「對應各自階段欄的配色」，沒有寫 teal 濃度。同族還有一處：「討論詳情頁檢視與轉出變更」（2026-10-13T22:22:52+08:00）寫衍生變更的「已刪除」用警示色加刪除線，但它引用的階段籤矩陣寫危險色；內文只寫刪除線。
21. **已封存討論的「開啟 ▾」有幾項**：`desktop-app`「詳情頁的文件動作」（2026-10-12T22:59:10+08:00）寫從變更詳情頁的來源討論籤開啟一筆已封存的討論時，選單三項齊全；「已封存項目以詳情頁檢視」（2026-10-13T23:23:36+08:00）寫從已封存頁開的已封存討論詳情頁只有「複製內容」。同一個檔案依入口不同出現不同選單。內文兩者都寫並標明入口。
22. **已封存討論有哪幾份文件**：`desktop-app`「已封存頁含討論節」（2026-10-10T19:39:25+08:00）寫背景、討論過程、結論三份、背景在前；「已封存項目以詳情頁檢視」與「詳情頁的文件樹、閱讀卡與本頁目錄」（都是 2026-10-13T23:23:36+08:00）寫結論、討論過程、背景、衍生變更四份。內文採後者。
23. **選單的「關於 Speclink」去哪**：`desktop-app`「macOS 原生選單」（2026-10-09T21:29:17+08:00）寫在關於頁落地前先進入應用程式設定頁；「關於頁」（2026-10-12T09:12:54+08:00）寫直接開啟關於頁。內文採後者。同一段選單需求的「檢查更新…」仍寫進入「軟體更新卡」，設定現在已有獨立的「更新」頁；內文照選單需求的字面寫。
24. **零專案時能不能點「手冊」**：`desktop-manual-page`「無手冊與 remote 模式的空狀態」（2026-09-02）寫零分頁時點側欄「手冊」會看到空狀態引導；`desktop-app`「側欄導覽結構」（2026-10-14T16:37:17+08:00）寫零專案時不渲染專案欄、主區是「開啟一個專案開始」，「macOS 原生選單」也規定零專案時「手冊」停用。內文採後者。
25. **品質關卡的章是不是只限本地**：`desktop-app`「看板卡片的待手動標示」（2026-08-14）順帶寫「與審查標示的 local-only 行為同構」；同規格「詳情頁的工單分頁」（2026-10-15T09:31:43+08:00）有 remote 變更處於審查中的情境，也規定遠端怎麼取工單。內文採後者，不再寫 remote 卡片不顯示章；「待手動」章在 remote 不出現，仍照它自己的需求寫。
26. **系統匣面板建立失敗的錯誤浮在哪**：`tray-status-menu`「面板樣式」（2026-10-14T23:04:04+08:00）寫在設定頁的「本機設定」簽浮出單行錯誤；`desktop-config`「設定頁圖形化讀寫兩層設定」（2026-10-14T16:37:17+08:00）的應用程式設定分類只有一般、更新、CLI、伺服器、關於，沒有「本機設定」。內文照較晚的 `tray-status-menu` 字面寫，並加附註。
27. **checkout 步驟的按鈕字面**：`workspace-chooser`「checkout 綁定驗證與 marker 寫入」（2026-10-08T11:52:22+08:00）把按鈕寫作「開啟 Workspace」；同規格「新增專案的來源分流」（2026-10-09T12:33:58+08:00）規定使用者可見文案不得出現「Workspace」，但沒有給新字面。內文只寫「開啟按鈕」，不引字面。

附註（規格自己宣告的例外，或只是寫法不對稱，不是矛盾）：

- `worktree-overlay`（2026-08-04）讓 `speclink list` 讀到壞掉的政策檔時照常輸出，與 `workflow-config` 的「壞檔即拒絕」不同；規格自述為僅限觀察面的例外。
- `change-lifecycle`（2026-07-27）寫 remote 模式下 `speclink discard` 回報不支援；`server-verb-api`（2026-07-23）寫 server 提供 discard 語意的刪除端點。一個講 CLI 動詞、一個講 server 端點。
- `review-station`「審查工單的建立與追加」（2026-10-15T09:31:43+08:00）不論哪一輪都沒有寫碼任務守門（只有蓋章時有）；`verify-station`「驗證工單的建立與追加」（2026-08-11）有。2026-10-14 新增的兩條路徑檢查需求也反映這個差異。兩站不對稱，各頁照各自規格寫。
- 接受 CRITICAL 時要逐條確認，只有 `review-skill`（2026-10-14）有規定；`verify-skill` 沒有，`quality-skill` 也只把確認轉交給審查關卡。各頁照各自規格寫。
- `config-skill` 把確認表的七項都稱為政策項目（含審查規則模式）；`workflow-config` 則說審查規則「不屬政策欄位」。行為不衝突，[工作流政策與設定](policy-config.md)寫成「確認表七項」。
- `propose-skill` 把「只動到同一段程式碼」列為要落檔的軟依賴；`ingest-skill` 則規定只動到同一段程式碼不記為前置、改向使用者提出，理由是本變更若已在進行中，等一個還沒開工的變更會卡住它。兩者時機不同（新建的變更一定還沒開工），各頁照各自規格寫。
- `tray-status-menu` 好幾段需求仍寫「macOS 面板」「非 macOS 平台用原生選單」，但「面板樣式」自己宣告這些規定也適用 Windows 面板。
- 詞彙：`desktop-app` 的需求名與內文仍有舊詞——「歸檔」見於「歸檔清單經衍生快取加速且可重建」與幾段 2026-10-10 改寫過的需求；「品質站」見於「品質站蓋章配色與主色分離」等；「workspace」「工作區」見於「看板首訪以 skeleton 佔位」與 `desktop-connections`。詳情自 2026-10-10 起改成佔滿主區的頁，需求內文已改稱「詳情頁」。桌面 app 各頁依 `openspec/LANGUAGE.md` 一律用「封存」「詳情頁」「品質關卡」「專案」，引用需求名稱時保留原字。CLI 與 remote 模式的頁沿用規格的「工作區」。

## 已知限制

- 沒有截圖。畫面文字與按鈕名稱逐字取自規格，實際畫面若不同，以執行中的產品為準。
- drift、audit 兩個技能沒有各自的規格，手冊只寫 `skill-routing` 與 `user-documentation` 載明的入口情境與交棒關係。apply 只有挑選變更的第一步（plan 守門）有規格，載於 `change-plan`；ingest 只有參數解讀與收尾的依賴、插隊判定有規格，載於 `ingest-skill`；兩者其餘的內文行為未載。analyze 的技能本身也沒有規格，但 `speclink analyze` 指令的判定規則有 `change-analysis`，寫在[分析：交叉檢查變更的產物](analyze.md)。
- `server-web-console` 的確認框清單列了「刪除專案」，但 `server-admin` 的管理功能只有專案的建立與改顯示名，沒有需求說刪除專案從哪裡進入。[瀏覽器後台](web-console.md)照規格把它列在確認框清單，沒有另外寫入口。
- 過期判定逐項比較：頁的生成時戳與規格的更新時戳都帶時區時比到秒，同一秒不算；任一邊只有純日期時比到日，同一天也算。每一頁的 `generated` 都寫成帶時區的秒級時戳；較早封存的規格，其更新時戳仍是純日期，封存不會回改。只取材某規格幾段需求的頁，`sources` 用「capability#需求名」錨定到那幾段，別段的封存不會把它標成可能過期；錨定的需求標題改名或移除時，該頁會被標為可能過期並整頁重寫。
- 來源規格改了、但改的部分與某一頁的內容無關時，那一頁重生後內文不變，只換生成時戳，「可能過期」標記就消掉。取材自 `desktop-app` 的頁都用錨定寫法：[認識桌面 app](desktop-overview.md)、[規格、討論、已封存與搜尋](desktop-browse.md)、[看板與任務](desktop-board.md)、[桌面上的品質關卡](desktop-quality.md)、[自動更新、CLI、技能檔過期與關於頁](desktop-update.md)、[專案、新增專案與設定頁](desktop-projects.md)、[桌面 app 的手冊頁](desktop-manual.md)。
- 錨定寫法有一個盲點：某能力新增一段需求時，沒有任何頁錨定到它，過期判定不會亮。2026-10-15 這次，桌面 app 改版把多段需求改名（「抽屜」改成「詳情頁」），上述桌面頁整頁重寫並錨定到新名稱；先前盲點中的「詳情頁的工單分頁」「已封存詳情頁的工單分頁」已寫進[桌面上的品質關卡](desktop-quality.md)。目前 `desktop-app` 沒有被任何頁錨定的需求，只有上面說的兩段內部結構需求。
- 規格裡的內部識別符（欄位名、型別名、旗標）不進手冊，改以白話描述效果。

## 編纂日期

2026-10-15

**出處**：本頁為說明頁，不直接取材自單一能力；各頁末行列出自己的出處。
