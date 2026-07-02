# SDD 流程結果報告：spectra vs speclink 雙流程對照（彈珠檯全面改進）

> 實驗日期：2026-07-02。同一需求範圍（6 項初始功能 + 1 項中途變更）分別以 spectra 2.3.1 與 speclink 0.1.0 的完整 SDD 流程各跑一次：speclink 在本倉庫、spectra 在相同基線的沙盒（相同的 pinball/index.html 301 行、相同的正典 `pinball-table` spec、對應的 config）。每個流程由一個獨立代理忠實遵循該品牌的技能文件驅動 CLI，全程留痕。成品由另外兩個獨立 QA 代理以自建 mocked-DOM harness 驗證，流程一致性由第五個稽核代理逐階段比對。

## 1. 需求範圍

初始（discuss 收斂 → propose）：①WebAudio 合成音效＋M 靜音 ②粒子/閃光回饋 ③slingshot ×2＋drop targets ×3（全清 +2000）④combo 倍率 ×1→×5 ⑤localStorage 高分榜前 3 ⑥P 暫停。
中途變更（apply 過半後經 ingest 加入）：⑦Nudge（N 鍵）＋TILT 懲罰（3 秒內 >3 次，本球 flipper 失效）。
約束：單一 HTML 檔、無外部依賴、Canvas 2D、60fps 固定步進、既有操作不變。

## 2. 流程執行結果（九階段，兩邊皆 9/9 完成）

| 階段 | speclink（本倉庫） | spectra（沙盒） | 一致性 |
|---|---|---|---|
| discuss | `discuss new/add-round×2/conclude` 落成 `discussion.md`（concluded） | 依技能於對話內完成，**不留檔案**（技能 disallow Edit/Write） | **刻意分歧** |
| propose | `new change --agent claude --from-discussion` 雙向連結；proposal＋6 個 spec delta＋design＋tasks(14)，寫入即驗證全過 | 同指令序列（無 `--from-discussion`）；proposal＋6 delta＋design＋tasks(13)；技能結尾 `park` | 一致（park 為 spectra 獨有儀式） |
| 品質關卡 | status 4/4 ✓、validate 0 error、analyze 0 Critical/0 Warning（僅 SUGGEST） | 同左（14 個 SUGGEST） | **一致** |
| apply 前半 | TDD 紀律（`instructions --skill tdd`）：任務 1–8，RED→GREEN→`task done`，25 tests | unpark→in-progress add→同紀律：任務 1–8，21 tests；RED 抓到 slingshot 打破的舊測試假設 | **一致** |
| ingest | `--force` 覆寫 artifacts、保留 8 個 `[x]`、任務 14→16；1 個 analyze Warning（design 標題後綴）修掉 | 同模式；**順手用 MODIFIED 修正正典 Bumper 範例與 combo 的語義矛盾**；同類 Warning 同修法 | **一致**（連踩到的 Warning 都是同一類） |
| apply 後半 | 任務 9–16 全完成，state=all_done，47 tests 全綠 | 任務 9–16 全完成，40 tests 全綠；RED 抓到 dead-ball 重複 drain 既有缺陷 | **一致** |
| drift | 12/32 anchors broken → **HEAVY**（全為散文英文詞誤報），拒絕其 `--skip-specs` 建議 | 8/36 → **LIGHT**（同類誤報）；嚴重度差異來自內容比例，非引擎差異 | 一致（誤報對稱） |
| verify | 三維度核對：14/14 requirements、16/16 任務、0 不一致；2 個 SUGGESTION 以 TDD 小步落地 | 同結構：14/14、16/16、0 不一致、1 SUGGESTION | **一致** |
| archive | `Specs applied: 6 capabilities (added: 13, modified: 1)`＋**Discussion archived**（連動歸檔） | `Specs applied: 7 capabilities (added: 13, modified: 3)`＋snapshot | 一致（能力切分屬創意差異） |

## 3. 成品測試（獨立 QA，未信任專案內附測試）

兩邊 **12/12 功能全過**（4 項基礎回歸 + 8 項新功能，含 Nudge/TILT 的滑動窗觸發）。

- speclink 版：534 行、47 個單元測試全綠。品質亮點：固定時步 1/120s×2 substeps＋accumulator＋dt clamp、localStorage try/catch＋資料過濾。小瑕疵：M 靜音無畫面指示、測試 hook 暴露於正式頁。
- spectra 版：40 個單元測試全綠，另通過 60 秒/3600 幀 soak（隨機拍擊+nudge+重開，座標無 NaN、球未逃出檯面）。小瑕疵：靜音狀態仍建立 AudioContext（節點不建）。
- 共同觀察：MAX_SPEED 下 substep 位移（10px）略大於球半徑（9px），理論上有穿隧可能，實測未發生。

## 4. 流程一致性稽核結論

**同構度 90%+**。指令面（new change/instructions/new artifact --stdin/status/validate/analyze/in-progress/task done/drift/archive -y）、artifact DAG、instructions JSON 欄位、寫入即驗證、analyze 四維度、verify 三維度、archive 輸出與正典合併語意（ADDED 併入/MODIFIED 整塊替換）、locale 分層（散文繁中/spec 英文）逐點對應——**連缺陷都對稱重現**（drift 散文誤報、design 標題子字串比對、@trace 未注入、`--force` 不合併）。

真正分歧僅三處，全屬設計定位而非漂移：
1. **discuss 持久層**（speclink 專屬強化）：discussion.md 落盤、`from_discussion` 雙向連結、archive 連動歸檔——「想法→變更→歸檔」因果鏈在檔案系統完整可回溯。spectra 側代理自述：結論須人工搬運進 proposal、對話中斷即蒸發。
2. **park/unpark**（spectra 獨有）：單人單變更流程下是兩步純開銷；speclink 依需求移除。
3. **drift 嚴重度**：同一誤報機制在不同內容比例下落在不同檔位（HEAVY vs LIGHT），引擎行為一致。

## 5. 流程評分（10 分制）

| 階段 | speclink | spectra | 評分依據 |
|---|---|---|---|
| discuss | **9.0** | 6.0 | speclink：持久化＋連結＋連動歸檔，跨 session 可交接（扣分：topic 即 slug）。spectra：收斂快、零檔案負擔，但結論不落盤、需人工搬運、中斷即失。 |
| propose | **9.0** | **9.0** | instructions「先給規則再檢查」閉環極佳，兩邊首跑即 0 error/0 Critical。 |
| 品質關卡 | 8.0 | 8.0 | validate/analyze 有效攔截格式問題；但跨 capability 語義矛盾（兩擊=200 vs 300）兩邊都抓不到，靠 agent 人工發現。 |
| apply | **8.5** | 8.0 | TDD＋SBE（spec Example 原值直翻測試）配合極佳、preflight 有用；扣分：`task done` 不驗證測試綠燈（兩邊各發生一次誤標後自救）。spectra 另扣 park/unpark 儀式開銷。 |
| ingest | 7.0 | 7.0 | 模式可用且兩邊踩到同類 Warning 同修法；但 `--force` 覆寫不合併、`[x]` 保留與任務 id 位移全靠 agent 紀律。 |
| drift | 4.0 | 4.5 | 散文英文詞誤報嚴重；speclink 側誤判 HEAVY 並給出危險的 `--skip-specs` 建議（盲從會漏 14 條 delta）。此為兩引擎共同弱項。 |
| verify | **9.0** | **9.0** | 三維度核對表有效，兩邊皆產出 14/14 對映且抓到值得修的 SUGGESTION。 |
| archive | **8.5** | 8.0 | 聚合輸出、正典合併、snapshot 皆正確；共同扣分：技能要求 commit 後才 archive，工作樹已乾淨 → 新 requirement 無 @trace。speclink 加分：討論連動歸檔。 |
| **總分** | **7.9** | **7.4** | |

**一致性總評：95/100**——扣分全來自刻意設計差異的存在本身；在共同功能面上未發現任何一處引擎行為不一致。

## 6. 回饋工具鏈的改進建議（後續工作，兩邊對稱適用）

1. **drift 錨點抽取**：對 design/proposal 的散文段落做符號白名單（僅反引號程式碼與路徑），避免把 GainNode/IIFE/Goals 當 repo 符號；HEAVY 判定前排除純散文 anchor。
2. **@trace 時序**：archive 技能的「先 commit 再 archive」使 @trace 恆為空；應改為 archive 時從**上一次 commit 的 diff**（或 touched.json）取 code 清單，或調整技能順序。
3. **task done 綠燈閘門**：可選旗標 `task done N --verify "<cmd>"`，指令非零退出則拒絕標記。
4. **ingest 合併**：`new artifact tasks --merge` 保留既有 `[x]` 狀態的結構化合併，取代整檔覆寫。
5. **analyzer 語義層**：跨 capability 的數值範例矛盾偵測（同名需求在不同 delta 的 Example 值衝突）。

## 7. 結論

同一需求、同一基線、兩套工具、兩個互不知情的執行代理：**九個階段全部走通、兩個成品 12/12 功能全過、流程行為 90%+ 同構、缺陷對稱重現**。speclink 在保持與 spectra 完全一致的引擎行為之上，以 discuss 持久化（記錄→連結→連動歸檔）取得了實質的流程優勢——這正是第一階段設定的差異化目標。本倉庫的彈珠檯已透過 speclink 流程完成全面改進（301→534 行、6 個正典能力、47 tests、歸檔 `2026-07-02-improve-pinball`）。
