## Context

移除 change 沒有動詞，手動刪目錄繞過生命週期機制：討論的 promoted_to 留死名、promoted 孤兒討論無收尾入口、「砍掉另開」無正規流程。討論 discard-change-verb 已裁定：新增頂層廢棄動詞＋討論側解鏈與狀態回退，守衛擋動工後誤刪（使用者確認砍 change 幾乎都在動工前；動工後異動走 discuss＋ingest 或 archive 後新開）。

既有先例：discuss discard（有 rounds 拒絕、--force 放行、remote 模式報不支援）、Store trait 的 delete_live_discussion、mark_promoted 的 frontmatter 改寫。相依：rediscuss-promoted-change 把 from_discussion 改為逗號累積器，解鏈須逐 slug 走其讀取函式。

## Goals / Non-Goals

**Goals:**

- speclink discard 一步完成：守衛檢查 → 刪除變更目錄與 touched 紀錄 → 對每份連結討論解鏈並在清單空時回退狀態 → 報告結果。
- 「砍掉另開」成為正規流程：discard c1 後討論回 concluded，可再 promote 開後繼或以 GUI 既有封存動詞收尾。
- README 兩份文件同步指令參考與工作流。

**Non-Goals:**

- 不支援 remote store 的廢棄（報不支援，鏡射 discuss discard；docs/verb-contract 遠端契約不動）。
- GUI 不提供捨棄動詞（比照討論 discard 排除，屬 agent/CLI 領域）。
- 不支援「動工後砍除」為一級流程——守衛拒絕、--force 是例外通道；動工後異動走 ingest 或 archive 後新開。
- 不動內嵌技能資產與 render golden。

## Decisions

### D1 — 頂層動詞 discard 與動工痕跡守衛

CLI 形狀：speclink discard <change> [--force] [--json]，拼寫鏡射頂層 archive。守衛順序：變更不存在 → 報錯；meta 有 started_at 或 tasks.md 有任何已勾任務（沿用既有任務解析）→ 拒絕並提示 --force；守衛通過才動檔案。刪除範圍：變更目錄整棵＋workspace touched 目錄下該變更的紀錄檔（若存在；touched 只在 task done 後出現，正常僅 --force 路徑會遇到）。

- 替代方案：子指令形 speclink change discard——引擎現行頂層動詞（archive、validate、analyze）皆不掛 change 前綴，新形狀反而不一致。拒絕。
- 替代方案：守衛只看未提交 git diff——與 git 狀態耦合、乾淨樹上動過工的變更會漏擋；started_at＋已勾任務是引擎自己的真相。拒絕。

### D2 — 解鏈與狀態回退

對變更 from_discussion 清單的每個 slug（逐 slug 走 rediscuss-promoted-change 的累積器讀取；該變更須先實作）：自討論 frontmatter 的 promoted_to 逗號清單移除本變更名；清單仍有值則保留 promoted 狀態；清單變空則移除 promoted_to 行並回退狀態——記錄的 Conclusion 區非空回 concluded、為空回 open（link 允許 open 討論併入，回退還原真實狀態）。slug 無對應記錄時跳過不報錯（記錄可能已被手動處理）。frontmatter 連結欄位本就由 mark_promoted 改寫，解鏈是同層 metadata 維護，rounds／結論不動。

- 替代方案：留 stale 條目＋放寬「promoted 孤兒討論可封存」守衛——看板衛生差、機制多一套，且「已轉出變更＝至少連結一個變更」的詞彙定義在最後連結死亡後不再真實。討論已否決。
- 替代方案：一律回退 concluded——open 討論經 link 後被廢棄會憑空獲得「已結論」狀態，說謊。拒絕。

### D3 — Store trait 新增變更刪除方法

流程邏輯歸 core：discard 編排（守衛、解鏈、報告）放 speclink-core 新模組（鏡射 archive 的頂層動詞模組模式），實際刪除經 Store trait 新方法（沿 delete_live_discussion 先例），檔案系統細節歸 speclink-fs adapter（跨平台由既有 fs 工具承擔）。speclink-node 的 store bridge 對映新方法至 JS 回呼。remote 模式在 CLI 層 bail「不支援」（鏡射 discuss discard 的既有訊息模式），不進 verb-contract。serde 面零變動：meta 檔只被讀與刪，promoted_to 改寫沿 mark_promoted 的字串替換模式，既有檔案格式不變。

- 替代方案：core 直接操作路徑刪目錄——違反「core 不得寫死儲存假設」紅線，也讓 node/remote store 無從對映。拒絕。
- 朝 storage 解耦靠攏：刪除能力進 trait 後，任何 store 後端（fs、bridge、未來 remote）的廢棄語意由同一 core 編排驅動，遠端支援屆時只補 trait 實作與契約端點。

### D4 — README 文件同步，遠端契約不動

README.md 與 README.en.md：「指令參考——變更生命週期」表各補 discard 一列（含 --force 與守衛說明）；「SDD 工作流」節補一句「砍掉另開」流程（discard 後討論回 concluded、可再轉出）。docs/verb-contract.md 與 zh-TW 版不動——它們是 remote store 的 HTTP 契約，本輪 remote 不支援。

- 替代方案：連 getting-started 也補——入門文件不含廢棄流程，加了增噪。拒絕。

## Implementation Contract

**行為**（實作完成後可觀察）：

- speclink discard <change>：變更無動工痕跡時 exit 0；變更目錄自 openspec/changes/ 消失；touched 紀錄（若存在）一併刪除；stdout 報告已刪除的變更名與每份解鏈討論（slug 與回退後狀態）；--json 輸出 camelCase payload（變更名、解鏈討論清單含回退後狀態）。
- 討論側效果：promoted_to 移除該變更名；仍有其他變更名則狀態不變；變空則該行消失且狀態回退（有結論→concluded、無結論→open）；rounds 與 Conclusion 區逐位元不變。
- 守衛：變更不存在 → 非零 exit code、stderr 說明；meta 有 started_at 或任何已勾任務且未帶 --force → 非零 exit code、stderr 提示動工痕跡與 --force，檔案零變動。--force 時照常執行。
- remote store 模式：非零 exit code，stderr 報 discard 不支援於 remote 模式（訊息模式鏡射 discuss discard）。
- 砍掉另開流程：discard c1 → 討論回 concluded → discuss promote 同一討論 --name c2 → promoted_to 僅含 c2。

**介面／資料形狀**：

- CLI：speclink discard <change> [--force] [--json]；--json payload 欄位 camelCase。
- Store trait：新增刪除變更的方法（fs adapter 實刪目錄；node bridge 對映）。
- 討論 frontmatter：promoted_to 為逗號清單的移除語意；空清單＝整行移除＋狀態回退。

**失敗模式**：任一守衛失敗即整體失敗、零寫入；解鏈時 slug 無對應記錄跳過（不視為錯誤）；刪除目錄失敗（如檔案被占用）以非零 exit code 回報且已完成的解鏈不回滾——輸出明示哪些討論已解鏈。

**驗收**：

- Rust：cargo test -p speclink-core --lib 通過（守衛、解鏈、狀態回退、報告的紅綠測試；本 Windows 機器須帶 --lib）；cargo test -p speclink-fs 的 store 刪除案例通過。
- CLI 實跑：沙盒中完整走「砍掉另開」流程並核對檔案效果與輸出。
- 回歸：既有指令輸出零變動（discard 是純新增動詞），以 parity 慣例快掃確認無波及。
- 文件：README 兩份的表格列與工作流句以內容審視確認。

**Scope 邊界**：

- In：core 廢棄模組、Store trait 刪除方法、fs／node store 實作、CLI 子指令與輸出、remote bail、討論解鏈與狀態回退、README 兩份、對應 spec delta（change-lifecycle、discussion-docs）。
- Out：remote store 廢棄支援與 verb-contract 契約、GUI 捨棄動詞、技能資產與 golden、動工後砍除的流程化、討論記錄內文的任何改寫。

## Risks / Trade-offs

- [與 rediscuss-promoted-change 的順序相依：解鏈需累積器讀取函式] → 實作排程明定在其後；tasks 首項驗證該變更已實作（讀取函式存在）再動工。
- [刪除為不可逆操作，誤刪動過工的變更] → 動工痕跡守衛預設拒絕；--force 是唯一例外通道且 stderr 明示後果；git 歷史仍可救回已提交內容。
- [解鏈中途失敗留半套狀態] → 順序設計為「先解鏈、後刪目錄」；目錄刪除失敗時輸出明示已解鏈清單，重跑 discard 對已解鏈討論冪等（名字已不在 promoted_to 即跳過）。
- [狀態回退寫錯方向（open 討論回退成 concluded）] → 回退判定以 Conclusion 區非空為準，紅綠測試覆蓋 open 經 link 後廢棄的案例。
- [跨平台：Windows 上目錄被開啟的程序鎖住刪不掉] → 經 fs adapter 的刪除以標準 std::fs 遞迴刪除實作，失敗走上述半套回報路徑，不做重試魔法。
