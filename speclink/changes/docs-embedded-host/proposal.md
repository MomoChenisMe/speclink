## Why

這個系列的前二十個變更把 speclink 的遠端能力重做成「引擎＋宿主套件＋宿主實作的儲存接口」，並刻意不動使用者文件：每個變更只在提案的 Impact 列出「文件影響」，留到最後一次更新。原因是整個系列完成才發版，中途改 README 會讓它描述還沒發版的功能。

現在 README 與 docs 描述的是系列開始之前的產品：

- **定位過期。** README 說有「Local Repo」與「Remote Store」兩條路徑，遠端靠「自架 server」，官方 server 是「參考實作」。討論的裁定是：使用者看到的是本機專案與遠端專案；遠端專案的規格由一台 Server 持有，Server 可以是內嵌了引擎的宿主，也可以是官方的示範用 server；官方 server 的定位是示範與試用。
- **整合者找不到路。** Node SDK 文件教的是已經移除的 31 個方法的 Store 物件，完全沒有提到 `createHost`、三個儲存函式、掛鉤、HTTP 入口與合格測試。
- **有幾句已經是錯的。** `change rank`、`trace`、`plan --strict-overlap` 在遠端專案已經可用，文件仍寫成只限本機；專案能力狀態列著已經刪除的三種 TeamStore。
- **新功能沒有說明。** 瀏覽器登入、技能由 Server 提供、`evidence show`、`change landed`、自由欄位、匯出匯入、備份格式第 2 版、從舊版示範用 server 升級的步驟。
- **延後的項目沒有寫成缺口。** 討論規定延後的項目必須在文件裡明確寫出來。

正式規格 user-documentation 也把舊的說法寫死了：README 的 H2 清單含「自架 server」、雙路徑段寫「Local Repo／Remote Store」、官方 server 是「參考實作」、只限本機的動詞清單「含 demo 與 trace」。文件的守門依這份規格檢查，所以規格要一起改。

- **目標使用者**：第一次讀 README 的開發者；要把 speclink 內嵌進自己系統的整合者；架設或升級示範用 server 的人；團隊裡的 PO 與 PM（他們經宿主的 AI Agent 參與，README 要讓他們看得懂自己在哪一條路徑上）。
- **使用情境**：決定用本機專案還是遠端專案；決定內嵌還是先試官方 server；照文件寫出三個儲存函式並通過合格測試；把 0.12.0 的示範用 server 升級到新版。
- **既有規格的對照**：user-documentation 有 25 條需求，其中 7 條的文字與這次的定案衝突（清單見 Capabilities）。ui-copy-vocabulary 規定詞彙守門掃 README、docs、介面文案與技能原稿，守門的詞來自 LANGUAGE.md 的 avoid 清單，所以新增 avoid 詞會立刻照出整個掃描面的存量。其餘規格不受影響：這個變更不改任何程式的行為。

Source discussion: embedded-host-needs（cut 21，最後一個變更）。

## What Changes

**用詞**（README、docs 與 LANGUAGE.md 同一批改完，中英兩版同步）

| 對象 | 中文 | 英文 |
| --- | --- | --- |
| 使用者看到的 | 本機專案、遠端專案、Server | local project、remote project、Server |
| 整合者看到的 | 宿主、內嵌、SDK、宿主套件、儲存接口、入口、合格測試 | host、embed、SDK、host kit、store port、entry point、conformance tests |

- 不再用「Remote Store」「自架 server」「團隊模式」「主機套件」，也不再用「SDK」統稱整塊遠端功能（SDK 是 npm 套件 `@speclink/engine` 這個工具箱的名字）。
- LANGUAGE.md 新增上表的詞條；「儲存後端」的定義改成示範用 server 的儲存（三個儲存接口的範例之一）與它的狀態。
- 「遠端模式」「本地」不列入 avoid：它們不違反定案的用詞表，存量也大。新寫的段落用「本機專案」「遠端專案」。

**README（中英兩版）**

- 第一個 H2 之前的雙路徑段改用本機專案與遠端專案。
- 「自架 server」一節拆成兩節，順序是：
  1. **把 Speclink 內嵌進你的系統**：列出宿主要做的四件事——實作三個儲存函式並通過合格測試、掛一條路由把請求交給入口、驗不出身分時傳空值、要支援瀏覽器登入時讓自己的登入服務提供標準的說明文件並接受本機回呼。放整體形態的圖。
  2. **試用官方 server**：以定位句開頭——「官方 server 是示範與試用用的。它讓你不寫程式就能試遠端專案。正式使用請把引擎內嵌進自己的系統。」三種跑法的表保留。
- 「文件」一節的分組與連結跟著改；「目前狀態」改寫，並用一行連到專案能力狀態的已知缺口。

**docs（十二組，中英兩版）**

- **Node SDK**：整份改寫成宿主套件——`createHost`、三個儲存函式與合格測試、兩個掛鉤、`host.dispatch` 與 Agent 工具定義、`host.handleRequest`（一條路由）、登入方式的宣告、技能清單、自由欄位、匯出與匯入、大小上限。`createEngine` 只講本機資料夾。放 SDK 的組成圖與一個動詞的四個步驟圖。移除 Store 物件的全部內容。
- **Server 儲存後端**：改寫成儲存接口的範例——三個範例在哪裡、資料表與檔案的配置、一次提交在 SQL 裡怎麼寫、事件記錄、範例的限制。
- **Server 部署、Server 備份與還原**：開頭加定位句；部署文件新增從 0.12.0 以前升級的三個步驟與回退方式；備份文件說明備份格式第 2 版與新版讀得懂第 1 版。放「你的系統與官方 server 共用宿主套件」的圖。
- **Remote 入門**：三種登入方式與選擇的順序、存取金鑰的存放位置、連上之後執行 `speclink update`、技能來自 Server、連到一台 Server 等於信任它提供的技能；「動詞在遠端的差異」改成現況。
- **動詞與旗標的參考文件**：只限本機的動詞清單改成現況；新增 `evidence show`、`change landed`、自由欄位的四個動詞與 `--field`；`list --json` 與 `show --json` 的新欄位；新的路由與事件的分類。
- **工作流程**：`trace` 與 `change rank` 不再標成只限本機；commit 與合併之後回報落地；遠端專案的 commit 只含程式碼的檔案。放「技能用路徑讀與用動詞讀」的圖。
- **設定說明**：CLI 的技能版本提醒與它的狀態檔；自由欄位的鍵與值的規則；`spec_dir` 今天不檢查絕對路徑與 `../` 的說明。
- **入門教學**：只校正用詞。
- **專案能力狀態**：能力清單的列改成現況（Node SDK 與宿主套件、儲存接口的範例、示範用 server、Speclink Desktop 的遠端專案）；本地與遠端對照改成現況；新增「已知缺口」一節，逐項列出這個系列延後的項目。
- **專案路線圖**：已經完成的方向改寫；延後的項目各有落點。
- **開發指南**：repo 結構表。

**圖解**

- 五張圖做成 SVG 放進文件：整體形態、SDK 的組成、一個動詞的四個步驟、你的系統與官方 server 共用宿主套件、技能用路徑讀與用動詞讀。
- 每張圖自己帶樣式與白色背景，不依賴頁面的樣式表與外部字型。中英兩版各一套，放在 docs/assets/figures 底下的兩個語言資料夾。
- 圖的內容以實作為準：畫之前對照程式與正式規格。

**截圖**

- 重拍初次設定頁的截圖（中英各一張）：儲存狀態的內容在變更 server-on-port-examples 改了。其他截圖拍到的畫面沒有變，不重拍。

**文件的守門**（scripts/docs）

- 中英對照的檢查涵蓋圖解：兩版引用相同的圖檔名序列，各自用自己語言的那一組。
- 新增圖解的檢查：SVG 不含 script 與外部資源、有背景、兩個語言資料夾的檔名相同、每張圖至少被一份同語言的文件引用。
- 中英對照的檢查加上「不得出現 Remote Store、Self-hosting、自架 server」。

**相容性影響**

- 沒有程式行為的改變。CLI、`--json`、線上協定、設定欄位、技能的內容都不變。
- README 的錨點 `self-hosting` 移除，改成 `embedding` 與 `official-server` 兩個。連到舊錨點的外部連結會落在頁首。
- LANGUAGE.md 新增 avoid 詞之後，詞彙守門會在整個掃描面（README、docs、三份介面文案、技能原稿）檢查這些詞；存量在這個變更一起清掉。技能原稿有改動時，技能的資產版號與快照要跟著更新。

## Non-Goals

- **新增截圖**：登入方式的卡、遠端專案的技能提示、關卡的標示只用文字說明。截圖的清單維持 17 張。
- **新增一份獨立的內嵌指南**：內嵌的說明放在 README 的一節與 Node SDK 文件。文件的成對清單維持十二組。
- **用 Rust 內嵌的完整教學**：Node SDK 文件是主要路徑；Rust 的宿主由三個範例與正式規格 host-kit 帶路，文件只給入口。
- **更新日誌與發版說明**：由發版流程產生。這個變更把要列進發版說明的破壞性變更整理在設計裡，供發版時使用。
- **重新產生手冊**（speclink/manual）。
- **改任何程式的行為**，包含介面文案的語意（只在詞彙守門要求時替換用詞）。
- **把「遠端模式」「本地」全面換掉**。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `user-documentation`:
  - 修改「使用者文件採漸進揭露與單一責任」（雙路徑段的用詞、README 的 H2 清單、內嵌與試用兩節的內容）。
  - 修改「產品狀態以證據分類目前與規劃能力」（能力清單的列）。
  - 修改「中英文文件保持結構與事實對等」（用詞、圖解的對等）。
  - 修改「品質關卡蓋章效果與非成員錯誤碼的文件揭露」（只限本機的動詞清單）。
  - 修改「使用者文件載明本地產物的純文字可攜性」（遠端專案的說法）。
  - 修改「使用者面路線圖與內部交付順序分列」（一條方向的名稱）。
  - 移除「官方 server 定位為參考實作而非唯一路徑」，新增「官方 server 定位為示範與試用」。
  - 新增「內嵌宿主的文件涵蓋宿主套件的對外介面」「儲存接口的範例有專文說明」「遠端專案的登入與技能來源有說明」「動詞參考文件涵蓋遠端可用的動詞與新增的路由」「已知缺口逐項列出」「圖解以自帶樣式的 SVG 呈現」「示範用 server 的升級步驟有說明」「使用者文件採用本機專案與遠端專案的用詞」。

## Impact

- Affected specs: user-documentation（修改六條、移除一條、新增九條）
- Affected crates 與 app: 沒有 crate 的程式改動。README、docs、speclink/LANGUAGE.md、scripts/docs 的守門。詞彙守門照出存量時，才會動到 apps/desktop、apps/server-web、packages/ui 的介面文案與 `speclink-core` 的技能原稿（只換用詞）。
- Affected code:
  - New:
    - docs/assets/figures/zh-TW/embedded-shape.svg
    - docs/assets/figures/zh-TW/sdk-contents.svg
    - docs/assets/figures/zh-TW/verb-four-steps.svg
    - docs/assets/figures/zh-TW/shared-host-kit.svg
    - docs/assets/figures/zh-TW/skill-path-vs-verb.svg
    - docs/assets/figures/en/embedded-shape.svg
    - docs/assets/figures/en/sdk-contents.svg
    - docs/assets/figures/en/verb-four-steps.svg
    - docs/assets/figures/en/shared-host-kit.svg
    - docs/assets/figures/en/skill-path-vs-verb.svg
    - scripts/docs/docs-figures.test.mjs
  - Modified:
    - README.md
    - README.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/server-store-drivers.md
    - docs/server-store-drivers.zh-TW.md
    - docs/server-deployment.md
    - docs/server-deployment.zh-TW.md
    - docs/server-backup.md
    - docs/server-backup.zh-TW.md
    - docs/remote-getting-started.md
    - docs/remote-getting-started.zh-TW.md
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/workflow.md
    - docs/workflow.zh-TW.md
    - docs/configuration.md
    - docs/configuration.zh-TW.md
    - docs/getting-started.md
    - docs/getting-started.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
    - docs/roadmap.md
    - docs/roadmap.zh-TW.md
    - docs/development.md
    - docs/development.zh-TW.md
    - docs/assets/screenshots/en/server-setup.png
    - docs/assets/screenshots/zh-TW/server-setup.png
    - speclink/LANGUAGE.md
    - scripts/docs/docs-parity.test.mjs
    - scripts/docs/remote-docs.test.mjs
    - scripts/docs/docs-links.test.mjs
  - Removed: (none)
- 相依：retire-teamstore-and-drivers（它是前面最後一個變更；文件要描述全部完成之後的狀態）。
- 三種形態下的結果：這個變更不新增或變更引擎命令、動詞、文件種類與技能步驟，所以沒有形態上的行為差異。文件本身要把三種形態寫清楚：
  - 本機 repo：README 與入門教學的主線，用「本機專案」。
  - 連到遠端的 checkout：Remote 入門、動詞參考文件與專案能力狀態的對照，用「遠端專案」。
  - 沒有本機規格檔的內嵌宿主：README 的「把 Speclink 內嵌進你的系統」與 Node SDK 文件；它的已知缺口列在專案能力狀態。
- 文件影響：這個變更就是文件的更新。它要收齊前二十個變更在提案 Impact 列的「文件影響」與「已知缺口」，以及討論結論的 Deferred 清單；第一個任務重新盤點一次，以封存後的提案為準。
