# 更新日誌

## 0.7.0（2026-09-22）

### 新功能

- 桌面 app 的變更詳情面板在審查或驗證進行中時，多出「審查」「驗證」分頁，直接看到工單裡卡了哪些 findings：每輪可收合、最後一輪展開，每條 finding 標嚴重度、路徑與描述，已接受的標「已接受」籤；工單蓋章或放棄後分頁自動消失。已封存的變更若曾審查或驗證未通過，也能看到帶走的工單。
- `/speclink-discuss` 寫結論時改用條列：Decision 一句定論起頭、其後一點一行，Rejected alternatives 與 Deferred 每項一行寫方案與理由，分幾刀轉出就每刀一個 bullet；桌面 app 的結論分頁因此不再出現整段長句。舊的討論記錄不會被改寫。

## 0.6.0（2026-09-17）

### 新功能

- `speclink plan` 會算出所有未封存變更的執行順序：分成一波一波（同一波可以並行），列出每個變更被誰擋住，並指出下一個可開工的變更；`speclink change depends` 可以宣告「這個變更要等哪些變更先做完」，apply 技能開工前會先看 plan、被擋住就停下，propose 收尾會把判定出的依賴落檔。
- 桌面 app 的看板、系統匣與詳情面板都照 plan 的順序排列：卡片標出波次、被擋住的變更變淡，詳情面板新增「排程」分頁可以查看與增減前置，拖排不能跨越宣告的依賴，依賴成環時看板會提示。
- remote 模式也有同一套執行順序：server 提供 plan 與前置寫入，桌面連 server 的看板、系統匣與 CLI 的 remote 模式行為與本機一致；排程分頁新增前置的候選改取自該變更所在的名冊，worktree 內的變更不會再選到被引擎拒絕的名字。
- 封存完成後，archive 與 commit 技能會提示「下一個可開工」的變更；commit 技能的「先封存再一起提交」子流程補上封存前的順序建議與手冊過期提醒。
- apply-with-worktree 建 worktree 之前先以 plan 守門：多個變更名時分出可並行與須等前置的，被擋住的變更不會留下空 worktree；ingest 收尾會重判本變更的依賴並落檔。
- `/speclink-discuss` 開場偵察納入進行中變更的 delta 規格：假設清單會標出「進行中變更 X 將改動」哪些需求，避免討論建立在正被改掉的規格上；`speclink list --json` 每筆變更多帶 delta 的 capability 清單。

### 修正

- 技能檔提示（指令檔過期、缺失或較新）出現時，看板、規格頁、手冊頁與已封存頁不再被視窗底緣裁掉，每欄捲到底可以看到最後一張卡與沉底的換頁控制列。
- macOS 從系統匣面板點變更列喚起主視窗後，hover、游標樣式與提示文字不再全部失效；面板會先收合再顯示主視窗。
- 用 Yarn 全域安裝 CLI 時保留轉接檔，安裝後可正常執行；Homebrew formula 的校驗值改自 npm registry 取回，重跑發版不會與 registry 脫節；install.sh 在 musl Linux 上說明只支援 glibc。

## 0.5.0（2026-09-14）

### 新功能

- Release 頁從 21 個檔收成 6 個：macOS 一個 universal dmg 同時跑 Apple Silicon 與 Intel、Windows 安裝器、Linux 兩種架構的 AppImage，加上自動更新用的 .app.tar.gz 與 latest.json；CLI 改由 npm 單一來源供給——`npm i -g @speclink/cli` 裝到的是原生執行檔，Homebrew 與 `install.sh` 抓的是同一份套件，無圖形介面的 Linux 也能 curl 一行裝好；Linux 桌面版不再出 .deb，既有 .deb 使用者請移除後改裝 AppImage。
- 桌面 app 的視窗回到前景時會重新檢查更新，長時間開著 app 也能發現新版本，不必再到設定頁手動按「檢查更新」。

## 0.4.0（2026-09-13）

### 新功能

- 一份討論分多刀轉出變更時，最後一刀帶 `--last`（`discuss promote`、`new change --from-discussion`、`discuss seal` 都收），最後一個變更封存後討論記錄自動隨行封存，不用再手動跑 `speclink discuss archive`。

### 改善

- 封存後合併的規格在繁中統一叫「正式規格」，與變更裡的「delta 規格」成對；桌面 app 的規格頁、手冊、README 與文件同步改字，英文維持 specs。

## 0.3.0（2026-09-11）

### 新功能

- 每個 release 版本都有白話的更新日誌：repo 新增 CHANGELOG.md，桌面 app 更新後首次啟動會彈出「X.Y.Z 更新內容」，之後也能從設定頁的「更新日誌」再看一次。
- `speclink validate` 提早抓出封存時會被拒收的 delta（目標需求不存在、撞名、少宣告移除的 scenario），不用做到封存才發現。
- 手冊頁的出處可以只錨定規格裡的一段需求，別段的封存不會再把整頁標成「可能過期」；重生後內文沒變的頁只換時戳。
- `/speclink-improve` 新增第六條摩擦訊號：資料夾與模組的分組整理有了自己的准入判準。

### 修正

- `speclink analyze` 收斂四條誤報：design 標題不再要求整串抄進 tasks，REMOVED 區塊不再要求 scenario；四個面向的規則正式寫進規格。
- 用 `--hold` 保留在途的討論，轉出下一刀時不再被清掉旗標、也不會被連帶封存；看板上與「已轉出」的討論分開呈現，收尾由 `speclink discuss archive` 明示解除。
- `speclink archive --mark-tasks-complete` 遇到結構不合法的變更時，封存被拒也不會再把 tasks.md 全勾。
- 討論記錄的狀態改寫只動 frontmatter，內文出現相同字串不再被誤改；封存時對 meta 損壞的在途變更改成保守處理，討論不會被誤掃進封存區。
- 發版流程對剛上架的 npm 套件改成輪詢查詢，不再因 registry 延遲可見而誤判失敗。

## 0.2.0（2026-09-08）

### 新功能

- 新增 `/speclink-manual` 技能：一句話就能從正式規格生成 wiki 式的新人手冊，或讓 AI 在對話中帶你導覽系統。
- 桌面 app 新增「手冊」頁：側欄樹、搜尋與上一頁／下一頁由手冊頁自動推導，可能過期的頁面會標示出來。
- 新增 `speclink trace` 動詞與 `/speclink-trace` 技能：沿「規格 → 變更 → 討論 → 程式碼」回答某個功能怎麼來的、為什麼這樣設計。
- 新增 `speclink discuss search`：依關鍵字找出封存討論裡的定案；discuss 與 improve 技能開場會先查舊討論，不再重提已否決的方向。
- 討論可用 `discuss conclude --hold` 保留在途，一份討論可以分期轉出多個變更。
- 討論的生命週期改由結論決定：還沒寫結論的討論不會被連帶封存，看板也分得出「已轉出但未結論」的討論。
- 桌面 app 的「新增 Workspace」記住最近開過的 workspace，分頁關掉之後也能一鍵再開。
- 桌面 app 設定頁新增「產出流程」頁籤：檢視、切換、fork、建立與刪除產出流程。
- 桌面 app 的規格、變更與討論抽屜的溯源籤都可以點：規格 → 變更 → 討論三跳閉環。
- 遠端模式的認領真正落盤：桌面遠端看板可以認領變更，並看到是誰在做。
- 遠端模式勾任務時，觸及檔案的證據會保存到 server，之後 drift、commit 與封存溯源都查得到。
- `@speclink/engine` 發布到 npm：不用自備 Rust 工具鏈，就能把引擎接進自己的腳本。
- Node SDK 的 createEngine 可以注入操作者身分並提供蓋章動詞，多人系統的章不再匿名。
- 新開 capability 時擋下重複命名並建議近似名，不會再產出兩份語意重複的規格。

### 修正

- `speclink init --force` 切換工具時會清掉前一個工具的技能目錄；技能檔過期提示也涵蓋自訂描述子。
- 手冊頁的「可能過期」判定改比到時戳，封存後立刻重生的手冊不再被誤標。
- 討論記錄內文含「## 」開頭的行時，輪次不再錯亂，也不會被誤當成結論。
- 遠端模式的審查與驗證改讀 server 上的證據，不再因本機沒有證據檔而卡住要求手動輸入。
- baseline 技能會套用 config.yaml 裡的 specs 產出規則。
- 遠端看板補齊建立者、開工歸屬、capability 清單與已轉出討論的分區。
- 系統匣面板的變更列補上 worktree 標記。
- 品質關卡的章支援非 UTF-8 的範圍檔。

### 改善

- onboard 技能改名為 baseline（`/speclink-baseline`），與 OpenSpec 的 Onboard 區隔。
- 技能收尾的交棒句更清楚：propose 後盤點執行順序、apply 後可以直接封存、worktree 收尾列出 `/speclink-quality`、archive 後提醒提交。
- 技能路由改由 description 與交棒句承載，不再注入 marker。
- discuss 技能流程重寫：先讀正典再偵察、依需求清晰度分流、多需求討論有固定慣例。
- 內建產出流程收斂為單一正典，驗證強度對齊 OpenSpec。
- apply 指令的 payload 直接帶 TDD 與 audit 的有效值，舊的政策鍵不再相容。
- 桌面「新增 Workspace」的來源卡改稱「Server」，去掉品牌前綴。
