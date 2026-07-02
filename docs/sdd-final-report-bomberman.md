# SDD 最終測試報告：2D 炸彈超人（Spectra vs Speclink）

日期：2026-07-02 ｜ 測試者：Claude（Fable 5）｜ 對照版本：Spectra 2.3.1 / Speclink（本 repo HEAD）

## 1. 實驗設計

- **雙沙盒**：`bomber-sp`（spectra）與 `bomber-sl`（speclink），皆為全新 git repo，杜絕前次實驗
  （彈珠檯）的非對稱汙染（那次導致 `.spectra`/`.speclink` 內容差異的誤判線索）。
- **相同輸入**：討論內容、proposal/design/specs/tasks、遊戲實作（md5 相同）、指令序列完全一致。
- **逐階段位元級比對**：每個階段的 CLI 輸出（品牌與路徑正規化後 diff）、工作目錄樹、canonical
  specs、快照、SQLite db。
- **spec 驅動 QA**：28 個測試逐一對映 spec 場景（node + mocked DOM，無瀏覽器）。

## 2. 流程紀錄（9 階段）

| # | 階段 | 操作 | 比對結果 |
|---|------|------|---------|
| 0 | init | `init . --tools claude` | ✅ 一致 |
| 1 | discuss | speclink：new → context → add-round ×2 → conclude → promote；spectra：無 CLI（揮發式，直接 new change） | ⚠️ 結構性差異（見 §5） |
| 2 | propose | 6 能力 spec delta＋design＋tasks；validate/analyze/status/instructions/show/list | ✅ 一致（修復缺口 #1 後） |
| 3 | apply | in-progress add → 遊戲 v1 → task done ×8 → commit | ✅ 一致（修復缺口 #2 後）；db 皆為 2 表＋1 列 |
| 4 | ingest | 需求變更（連鎖引爆＋重生無敵窗）→ 更新 proposal/design/spec delta/tasks → 再驗證 | ✅ 一致 |
| 5 | apply（續） | 遊戲 v2 → task done ×2 → 測試 28/28 | ✅ 一致 |
| 6 | drift | 已提交/未提交 design 兩種情境，human＋JSON | ✅ 一致（修復缺口 #3–#5 後） |
| 7 | verify | 兩側各自執行 tests.js（QA 見 §3） | ✅ 28/28 兩側相同 |
| 8 | archive | archive -y → canonical specs／archive tree／snapshots／歸檔後 list/validate/show | ✅ 逐位元一致（僅設計內差異，見 §5） |

## 3. 遊戲 QA（spec 場景 ↔ 測試對映）

六個能力、17 個 Requirement、24 個 Scenario 全數有對應測試；`##### Example:`（速度換算、
範圍 2 全空傳播）直接做為測試資料。28/28 通過（兩沙盒與主 repo 各跑一次）。

| 能力 | 場景數 | 測試 |
|------|-------|------|
| arena-layout | 3 | 外圈/硬柱、同種子重現、出生保留區 |
| player-control | 2+Example | 位移換算 (60,60)→(90,60)、硬牆阻擋 |
| bomb-mechanics | 4 | 同格去重、同時上限、引信 149/150、走出實體化 |
| blast-resolution | 5+Example | 硬牆截斷、軟磚即停、9 格傳播、敵人/玩家傷害、連鎖（ingest） |
| destructibles-and-powerups | 3 | 軟磚摧毀、掉落可重現、拾取生效 |
| game-flow | 8 | 生成距離、接觸擊殺、重生、命盡、全滅、R/P、HUD、無敵窗（ingest） |

ingest 的敘事來自實測：v1 沒有無敵窗時「重生點被敵人壓住 → 連續扣命」真實發生
（命盡測試也因此需要更新——需求變更的漣漪被測試如實捕捉）。

## 4. 本次抓到並修復的 parity 缺口（5 個）

真實內容（中文 spec、camelCase 識別字、多次 task done）觸發了先前 31 項 parity suite
沒覆蓋到的分支：

1. **analyze／ambAbstractScenario 的「具體性」判定**：spectra 認定「行內含 ASCII 數字、
   反引號、或雙引號」即具體；speclink 原本只認數字（中文場景『軟磚分佈可重現』被誤標）。
   單引號、全形引號、全形數字、場景名稱皆不算——逐項探測後對齊。
2. **task done 的 touched 歸因**：spectra 每次只記錄「尚未被先前任務歸因的髒檔」，無新檔
   則完全不追加；speclink 原本每次全量記錄（8 次 task done 記了 8 筆 vs spectra 1 筆）。
3. **drift／anchor 搜尋語料**：spectra = 「HEAD 已提交內容（全部檔案）∪ 已追蹤 md/txt 的
   工作樹內容」（經 `git ls-files`），**全字、區分大小寫**比對；speclink 原本掃工作樹程式碼
   檔（副檔名白名單、子字串比對、排除 openspec）。注意這代表 spectra 的 design.md 一旦提交，
   其 anchor 永遠「自我命中」→ 正常流程下 Structure 恆為 0 broken。
4. **drift／stopword 名單**：逐詞探測 85 個候選——spectra 過濾 Rust 型別詞＋GWT 關鍵字
   （Given/When/Then/Struct/Enum/Trait/Type/Path/Value/Item/Fn…）但**保留** The/Eq/Ord/
   PartialEq/PartialOrd；speclink 名單修正為探測結果。
5. **drift／反引號 camelCase 抽取**：反引號 span 的開頭識別字若為 camelCase（小寫開頭＋
   內含大寫，如 `pressKey(code)`）也是 anchor；`dotted.pathToken`、`under_scoreCamel` 不是。

（前置階段亦修復：drift Environment `--since` 計數、last_commit 恆 null、Tasks 狀態字串、
archive 快照格式/備份/`.started` 清理——見 comparison 文件 §18。）

## 5. 設計內差異（刻意保留）

| 項目 | spectra | speclink |
|------|---------|----------|
| discuss | 揮發式（對話結束即消失，僅靠 capture） | 持久化文件（Context/Rounds/Conclusion 骨架）＋ promote 種子 proposal ＋ 隨 change 歸檔 |
| change metadata | `created_with: claude` | `from_discussion: <slug>`（promote 時） |
| archive 輸出 | — | 多一行 `Discussion archived: …` |
| 移除功能 | ask/向量搜尋、worktree、park、share、桌面 App 整合 | 不提供（db 亦僅 CLI 所需 2 表） |

## 6. 評分（0–10，依階段實測）

| 階段 | Spectra | Speclink | 依據 |
|------|---------|----------|------|
| init／scaffold | 8.0 | 8.0 | 行為一致；標記區塊、PREPEND、re-init 防護皆同 |
| discuss | 6.0 | 9.5 | spectra 無紀錄可回溯；speclink 骨架化文件（本輪新增 Context/固定模板）讓 propose 可考古、否決選項不翻案 |
| propose（validate/analyze） | 8.0 | 8.0 | 驗證/分析同構；共同弱點：具體性啟發式對 CJK 偏弱（全形數字不算數字） |
| apply（status/instructions/task done） | 8.0 | 8.0 | DAG/指示/touched 歸因一致；touched 的差量歸因對 commit skill 實用 |
| ingest | 7.5 | 7.5 | 皆無專用 CLI，靠 artifacts 慣例＋skill；等價 |
| drift | 4.0 | 4.0 | 同構但兩維度實質失效：Environment 受 `--since` 純日期陷阱（當天恆 0）；Structure 受 anchor 自我命中（提交後恆 0 broken）。忠實復刻，同時如實扣分 |
| verify | 8.0 | 8.0 | skill 驅動；spec 場景可 100% 轉為測試是規格品質所致，工具中立 |
| archive | 8.5 | 9.0 | 快照/回滾資料/canonical 合併逐位元一致；speclink 加分於 discussion 連帶歸檔 |
| 一致性（跨工具可交換性） | 10 | 10 | 同一 repo 兩工具產物逐位元可互換 |
| **加權總分** | **7.6** | **8.0** | |

## 7. 最終建議

1. **CLI 工作流選 speclink**。行為與 spectra 位元級同構（本輪連 5 個深水區缺口都已對齊），
   多出的 discuss 持久化鏈（討論 → promote → 歸檔隨行）是 SDD 流程中唯一能把「為什麼」
   留下來的環節；純 CLI/CI 場景沒有理由選 spectra。
2. **需要桌面 App（GUI、向量搜尋、worktree、分享）才選 spectra**。這些是 speclink 刻意
   不做的範圍。
3. **給兩個工具鏈的改進方向**（speclink 可作為刻意差異實作，spectra 建議上游回報）：
   - drift Environment：`--since` 應傳完整時間戳（`<created> 00:00`），否則當天建立的
     change 永遠 0 commits；
   - drift Structure：anchor 搜尋語料應排除 change 自身的 design.md，否則提交後永不告警；
   - analyzer 具體性：全形數字/引號應視為具體（CJK 內容誤報率高）；
   - ingest 缺 CLI 級支援（例如 `ingest --note` 記錄需求變更軌跡），目前全靠慣例。
4. **測試方法論**：本輪 5 個缺口全部由「真實內容＋真實流程」觸發，而非 31 項合成 parity
   suite——維持「每次大改後跑一輪端到端雙流程」的做法比擴充合成測試更划算。
