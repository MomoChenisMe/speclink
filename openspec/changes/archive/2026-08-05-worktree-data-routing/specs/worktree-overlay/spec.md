## MODIFIED Requirements

### Requirement: desktop 看板的 worktree 呈現

<!-- BEFORE: worktree 呈現僅及卡片標示、抽屜分支名與路徑、任務計數與看板自動更新；抽屜文件原文、meta 欄位、狀態報告、驗證／分析與全文搜尋仍讀主 checkout。 -->

local workspace 的 desktop 看板 SHALL 經與 CLI list 同一觀察面組裝取得 worktree facts：有 worktree 映射的 change，卡片 SHALL 帶 worktree 標示，變更抽屜 SHALL 顯示分支名（speclink/<change名>）與 worktree 路徑（OS 原生路徑形式）。文案於 zh-TW 與 en 介面語言下均直出「worktree」一詞。worktree 的增減、或其內 openspec/changes/<change名>/ 目錄的變動，SHALL 使看板自動更新（無需手動重整）；worktree 移除（merge 收尾）後標示與抽屜資訊 SHALL 退場。git 不可用時看板照常呈現且無任何 worktree 標示（fail-open，沿用 discovery 慣例）。remote 工作區不適用本需求。

對有 worktree 映射的 change，desktop 的全部 per-change 讀取面 SHALL 解析到該 worktree 副本：變更抽屜各分頁的文件原文（提案／設計／任務／規格）、metadata 欄位（建立與開工資訊）、狀態報告、驗證與分析報告，以及看板全文搜尋的 artifact 掃描。同一 change 於同一畫面的計數與內容 SHALL 同源（SHALL NOT 出現計數來自 worktree、內容來自主 checkout 的劈半）。observed_facts 為空（政策關閉、非主 checkout、git 不可用）時，上述讀取面行為 SHALL 與本需求擴充前完全相同；worktree 內 change 目錄不可讀時該條目依 discovery 慣例靜默回讀主副本。

#### Scenario: 卡片標示與抽屜資訊

- **WHEN** 存在分支 speclink/add-auth 的活躍 worktree 且 add-auth 為未封存 change 時開啟 desktop 看板
- **THEN** add-auth 卡片帶 worktree 標示，開啟其變更抽屜可見分支 speclink/add-auth 與該 worktree 的路徑

#### Scenario: worktree 內進度即時反映

- **WHEN** 於 worktree 副本內將一個任務勾為完成
- **THEN** 主看板該 change 卡片的任務計數自動更新，無需手動重整

#### Scenario: 抽屜任務內容與計數同源

- **WHEN** worktree 副本內的 tasks.md 已勾 9/10，主 checkout 的同名檔全未勾，開啟該 change 的變更抽屜任務分頁
- **THEN** 分頁徽章顯示 9/10 且下方任務清單有 9 項呈已勾狀態，兩者同源於 worktree 副本

#### Scenario: 分析報告反映 worktree 現值

- **WHEN** worktree 副本內的 proposal.md 已更新而主 checkout 仍為舊內容，於抽屜觸發分析
- **THEN** 報告依 worktree 副本的 artifact 內容產出

#### Scenario: 全文搜尋命中 worktree 內容

- **WHEN** 某字串僅存在於 worktree 副本的 design.md、不存在於主 checkout，於看板全文搜尋該字串
- **THEN** 該 change 卡片出現於命中清單，snippet 取自 worktree 副本內容

#### Scenario: 收尾後標示退場

- **WHEN** worktree 經 merge 收尾流程移除後看板刷新
- **THEN** 卡片不再帶 worktree 標示，抽屜不再顯示分支與路徑

#### Scenario: git 不可用時看板照常

- **WHEN** git 於環境中不可用時開啟 desktop 看板
- **THEN** 看板照常列出 changes 且無任何 worktree 標示，不顯示錯誤

---

### Requirement: worktree 掛著時的 desktop 動詞防護

<!-- BEFORE: 防護僅及封存與退回提案中兩動詞；刪除無守門，粒度寫入動詞（任務勾選等）未定義落點、實作寫入主 checkout。 -->

對有 worktree 映射的 change，desktop 動詞 SHALL 分兩級處理。破壞性生命週期動詞——「封存」「退回提案中」「刪除」——SHALL 拒絕執行，並提示先執行 worktree-merge 收尾；其語意為主 checkout 的 change 目錄存廢，SHALL NOT 路由至 worktree。粒度寫入動詞——任務勾選、全部勾選、任務拖排、卡片拖排、放棄審查工單——SHALL 路由至該 worktree 副本執行：檔案效果（tasks.md、change metadata、審查工單）與其側效（touched 記錄、首次完成的開工章、git 髒檔歸因）SHALL 全數落在 worktree 副本內，主 checkout 的對應檔案 SHALL NOT 被寫入。唯讀呈現（抽屜、diff 檢視）不受防護影響。observed_facts 為空時兩級動詞行為 SHALL 與無 worktree 時完全相同。此防護僅及 desktop 動詞層；CLI 對應動詞不在此限。

#### Scenario: 封存被擋

- **WHEN** 對有活躍 worktree 映射的 change 於 desktop 觸發封存
- **THEN** 動詞拒絕執行，訊息含先收尾的指引，change 目錄與看板狀態不變

#### Scenario: 刪除被擋

- **WHEN** 對有活躍 worktree 映射的 change 於 desktop 觸發刪除
- **THEN** 動詞拒絕執行，訊息含 change 名與先收尾的指引，主 checkout 與 worktree 的 change 目錄均不變

#### Scenario: 任務勾選寫入 worktree

- **WHEN** 對有活躍 worktree 映射的 change 於抽屜勾選一個任務
- **THEN** worktree 副本的 tasks.md 該任務變為已勾且 touched 記錄落在 worktree 副本，主 checkout 的 tasks.md 位元級不變

#### Scenario: 卡片拖排位置保持

- **WHEN** 於看板拖動有 worktree 映射的 change 卡片至新位置後看板重新整理
- **THEN** 卡片停留於新位置（rank 寫入 worktree 副本的 change metadata，與看板讀取同源）

#### Scenario: 收尾後解禁

- **WHEN** 該 change 的 worktree 經 merge 收尾移除後，於 desktop 再次觸發封存
- **THEN** 封存照常執行
