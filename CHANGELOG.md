# 更新日誌

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
