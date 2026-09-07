---
title: 本手冊的來源
section: 附錄
order: 900
keywords: [來源, 矛盾, 限制, 編纂日期, 規格]
sources: []
generated: 2026-09-07T13:20:04+08:00
---

# 本手冊的來源

## 取材範圍

本手冊全部內容只取材自 `openspec/specs/` 底下的正典規格。README、`docs/` 與原始碼都不是來源。規格沒寫的，手冊就不寫，或在該處標明「規格未載」。每一頁的最後一行列出它取材的能力名稱。

正典共 81 個能力。其中 56 個是使用者會操作或看到的東西（畫面、指令、技能、輸出、檔案），已入冊。其餘 25 個是引擎內部（儲存、wire 契約、host 執行期、測試骨架、建置與發布管線），不入冊：

`client-protocol`、`command-runtime`、`delivery-baseline`、`desktop-release`、`dev-harness`、`host-runtime`、`node-sdk`、`node-sdk-release`、`phase2-acceptance`、`phase3-acceptance`、`postgres-team-store`、`reference-server`、`remote-board-order`、`remote-workspace-data`、`server-context-api`、`server-drift-api`、`server-event-stream`、`server-read-api`、`server-verb-api`、`serverfs-team-store`、`sqlite-team-store`、`store-abstraction`、`teamstore-contract`、`ui-copy-vocabulary`、`workspace-session`。

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
4. **指令檔過期怎麼判**：`desktop-app`「指令檔過期提示」（2026-08-06）以 CLAUDE.md 是否存在、SPECLINK 標記是否被移除來判；`workspace-tools`「技能檔過期探測」（2026-08-23）改以 skills 目錄下有無 speclink- 技能檔、以及技能檔版號比對來判。內文採技能檔版號。
5. **專案設定頁有幾個頁簽**：`desktop-config`「設定頁圖形化讀寫兩層設定」（2026-08-23）寫本地兩簽（config.yaml、.speclink.yaml）、remote 單一 Workflow 簽；同規格「設定頁的產出流程頁籤」（2026-08-22）與其後六條產出流程需求寫本地三簽（config.yaml → Schema → .speclink.yaml）、remote 兩簽（Workflow → Schema）。內文的頁簽列採兩簽版；產出流程的內容仍照 2026-08-22 那組需求列出，並註明日期。
6. **唯讀角色叫什麼**：`server-policy-write`（2026-07-20）、`user-documentation`（2026-07-24）與 `desktop-config`（2026-08-23）寫 reader；`server-identity` 的一條 scenario（2026-07-28）寫 viewer。內文採 reader（最晚一次提到的是 2026-08-23）。
7. **server 的官方發布物**：`server-release`「Server 交付物內嵌同版本 SPA 資產」（2026-07-25）寫 release binary 與「tag 觸發 server binary 與 Docker image 發布」；同規格「release 產物含 server 與部署文件」（2026-08-14）寫 server binary 不上傳 GitHub Release，官方通路只有 Docker 映像與 npm 套件。內文採後者。
8. **討論結論後的路**：`user-documentation`「討論結論後的轉出與併入分流完整」（2026-07-17）把 `speclink discuss promote` 列為結論後四條路之一；`discuss-skill`「結論後交棒單推 propose 入口」（2026-08-27）與 `skill-routing` 交棒邊表（2026-09-01）寫結論後只建議 `/speclink-propose --from-discussion`，promote 留給中途轉出。內文採後者。
9. **verify 是不是可呼叫的站**：`user-documentation`「Getting Started 僅使用已驗證入口」（2026-07-17）寫入門文件不要求呼叫未安裝的 `$speclink-verify`；同規格「工作流正典逐站列出技能與完成判準」（2026-09-01）與 `verify-skill`（2026-08-11）把 verify 列為正式站。內文採 verify 是可呼叫的站。
10. **討論隨變更封存的條件**：`user-documentation`（2026-07-17）寫最後一個存活變更封存時討論一併封存；`discussion-docs`「討論以 link 動詞併入既有變更」（2026-09-01）加了前提：討論的結論必須已寫入。內文採有前提的版本。
11. **使用者文件可否連到架構文件**：`user-documentation`「目標架構與目前狀態維持清楚邊界」（2026-07-17）要求 README 連到平台架構藍圖與路線圖；同規格「使用者面路線圖與內部交付順序分列」（2026-08-14）寫使用者文件不得引用那兩份維護者文件。與手冊內容無關，僅記錄。
12. **點規格卡會發生什麼、溯源顯示在哪**：`desktop-app`「規格頁提供清單、搜尋與展開檢視」（2026-07-09）寫點卡片標題就地展開全文、下方帶一行來源變更；同規格「桌面 app 呈現 change 與 spec 的清單與內容」的原版（2026-07-11）寫點卡片開啟唯讀的規格詳情面板、清單不提供行內展開，其後的面板互斥（2026-07-17）與卡片收合（2026-08-11）需求也以面板為前提；同一條需求的最新版（2026-09-03）再把溯源變更改為標頭出身列的籤，內文底部不再有溯源文字行。內文採詳情面板與出身列的籤。
13. **品質關卡狀態列顯示什麼**：`desktop-app`「詳情抽屜的審查資訊列」（2026-08-04）與「詳情抽屜的驗證資訊列」（2026-08-06）寫狀態列顯示狀態詞、蓋章時間與審查者／驗證者；同規格「變更詳情抽屜標頭的四層結構」（2026-08-07）寫日期與蓋章者收進指標停留提示、可視文字不直出日期與 email。內文採後者。
14. **封存時未結工單的第一個選項叫什麼**：`desktop-app`「封存入口的未結工單三選項」（2026-08-02）描述為「前往完成蓋章」；同規格「變更與討論抽屜開啟時底層落回看板」（2026-08-11）稱同一個按鈕為「去蓋章」。內文以「去蓋章」為按鈕字面。
15. **討論開場淺掃有幾段**：`discuss-skill`「事實與決策分診及逐節點查證」（2026-08-21）把開場偵察規定為「正典 → 程式碼」兩段漏斗；同規格「開場舊討論查核與第四類對照」（2026-09-05）改為「正典 → 舊討論查核 → 程式碼」三段，並在假設清單的三分對照之外加入第四類「舊討論已定案」。內文採三段與四類。

附註（規格自己宣告的例外，不是矛盾）：

- `worktree-overlay`（2026-08-04）讓 `speclink list` 讀到壞掉的政策檔時照常輸出，與 `workflow-config` 的「壞檔即拒絕」不同；規格自述為僅限觀察面的例外。
- `change-lifecycle`（2026-07-27）寫 remote 模式下 `speclink discard` 回報不支援；`server-verb-api`（2026-07-23）寫 server 提供 discard 語意的刪除端點。一個講 CLI 動詞、一個講 server 端點。
- `review-station`（2026-08-03）對審查工單的續輪沒有寫碼任務守門；`verify-station`（2026-08-11）對驗證工單的續輪有。兩站不對稱，各頁照各自規格寫。
- `desktop-app` 早期需求（2026-07-05、2026-07-17）寫「歸檔」，後期一律寫「封存」；規格也把面板稱為「抽屜」、把品質關卡稱為「品質站」。內文依 `openspec/LANGUAGE.md` 的正典詞彙一律用「封存」「詳情面板」「品質關卡」，引用需求名稱時保留原字。

## 已知限制

- 沒有截圖。畫面文字與按鈕名稱逐字取自規格，實際畫面若不同，以執行中的產品為準。
- apply、ingest、drift、analyze、audit 五個技能沒有各自的規格，手冊只寫 `skill-routing` 與 `user-documentation` 載明的入口情境與交棒關係。
- 過期判定分段比較：頁的生成時戳與規格的更新時戳都帶時區時比到秒，同一秒不算；任一邊只有純日期時比到日，同一天也算。從本次起每一頁的 `generated` 都寫成帶時區的秒級時戳；較早封存的規格，其更新時戳仍是純日期，封存不會回改。對這些純日期規格，同日生成的頁在桌面 app 的手冊頁仍會被標為「可能過期」，下一次生成也會被判為過期並重生；來源規格在那之後沒再改時，重生只更新生成時戳、內文不變。這是 `manual-pages` 刻意的保守設計，不是錯誤。
- 規格裡的內部識別符（欄位名、型別名、旗標）不進手冊，改以白話描述效果。

## 編纂日期

2026-09-07

**出處**：本頁為說明頁，不直接取材自單一能力；各頁末行列出自己的出處。
