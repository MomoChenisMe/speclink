## Context

封存時 delta 併入正典的合併引擎位於 speclink-core 的 archive 模組。現行行為：ADDED 撞名、MODIFIED／REMOVED／RENAMED 缺目標時靜默跳過；逐 capability 讀取、驗證、寫入交錯進行；MODIFIED 以 delta 區塊整段替換正典需求，delta 漏抄的 scenario 無聲消失；正典不存在時 MODIFIED 也會物化成新規格；新正典 Purpose 固定產生 TBD。這些行為被單元測試凍結，且守門只存在於 drift 的 assumption 提醒與 bulk archive 的預檢——單筆 archive 引擎不拒絕。討論 post-archive-spec-value 裁決改為 fail-closed，並以 OpenSpec specs-apply 的錯誤清單為驗收參照。

## Goals / Non-Goals

**Goals:**

- 過期或自相矛盾的 delta 在任何寫入發生前被拒絕，錯誤逐條列明
- 多 capability 封存不再可能半完成：全部驗證通過才寫入
- 新 capability 的 Purpose 可由 delta 提供
- drift 提醒、bulk 預檢與引擎守門共用同一判定，不得分歧

**Non-Goals:**

- 不動 @trace 內容、evidence 來源與 dirty worktree fallback（第二刀 change 處理）
- 不做 sync 動詞與 early-sync no-op 例外
- 不做 requirement fingerprint／CAS
- 不動 UI 與 server 路由（錯誤沿既有 Refusal 通道）

## Decisions

### 守門落點：speclink-core 合併引擎單點裁決

守門實作在 speclink-core 的 archive 合併路徑（領域演算法，不含 ANSI、不寫死儲存媒介），CLI、desktop、server 全部經同一 command 層取得相同裁決——不在任何呼叫端平行實作第二套判定（回歸對照 remote_verb_parity）。拒絕以既有 error 通道回傳，人眼與 `--json` 呈現由呼叫端組裝。

### 兩階段合併：先產完整 merge plan、全數驗證後才寫入

套用改為 plan 與 commit 兩階段。plan 階段讀取全部 capability 的 delta 與正典、逐條驗證並產生合併結果，任何違規即回傳聚合錯誤——此時**零檔案效果**（無 snapshot、無正典寫入、change 不移動）。commit 階段依序：先寫全部 snapshot 備份，再寫全部正典，最後移動 change 目錄。寫入順序的失敗語意：snapshot 全數落地後才動正典，故 commit 階段的 I/O 失敗（磁碟、權限）留下的任何半套狀態皆可由 snapshot 與 Git 恢復——接受此殘餘風險並以順序保證可恢復性，不引入交易機制（本地檔案系統無交易可用，過度設計）。

### 違規清單與聚合錯誤形狀

plan 階段收集**全部**違規後一次回報（不是 first-fail），每條含 capability、操作、需求名、原因，結尾指引補救動線（先 speclink drift、再以 ingest 更新 delta）。理由：agent 一輪修完所有問題，避免逐條試錯的多輪封存。違規清單：

1. ADDED 需求名已存在於正典
2. MODIFIED／REMOVED／RENAMED 來源需求名不存在於正典
3. 同一需求名出現在多個操作區段（含 RENAMED 的 FROM／TO 與其他區段互撞）
4. RENAMED 目標名已存在於正典
5. MODIFIED 缺正典既有 scenario 且未聲明刪除
6. 正典不存在的 capability 出現 ADDED 以外的操作

`--no-validate` 不解鎖任何一條（它只略過文件驗證）；`--skip-specs` 維持整段跳過規格套用的既有語意。

### scenario superset check 與明示刪除聲明

MODIFIED 驗證：正典目標需求的每個 `#### Scenario:` 名稱必須出現在 delta 區塊中，或以刪除聲明註解 `<!-- REMOVED-SCENARIO: <scenario 名> -->`（一行一個，置於 MODIFIED 區塊內）明示放棄。聲明註解與既有 `<!-- BEFORE: -->` 同層處理：供變更審閱、寫入正典前剝除。名稱比對與需求名同語意：trim 後完全相等。換行以 `\r\n` 正規化為 `\n` 再解析（Windows 對照）。

否決替代案：OpenSpec 式無逃生（Speclink 無 sync，刻意刪 scenario 將無路可走）；scenario 級 delta 操作區段（OpenSpec Phase 2 構想，解析面擴張過大）；旗標放行（correctness 級不設旁路，裁決明文）。

### 新 capability 的 Purpose 自 delta 帶入

delta 檔的 `## Purpose` 區段（需求區塊之外的獨立段落，現行解析器忽略、不影響操作解析）在建立新正典時複製為 Purpose 內容；未提供才落現行 TBD 骨架。既有 capability 的 delta `## Purpose` 不套用、不報錯（與 OpenSpec 同）。

### 判定共用：drift assumptions 與 bulk 預檢改呼叫 plan 驗證

現行 drift 的 spec assumptions 與 bulk readiness 各自實作名稱層級判定，與新引擎守門形成三套裁決的風險。收斂為單點：plan 階段的驗證函式成為唯一判定，drift 的 Specs 維度與 bulk 預檢改以它為來源（輸出形狀各自維持，reason 文案改為拒絕語意「archive would refuse it」）。golden 與 CLI 測試同批更新。

### 凍結測試翻轉

archive 模組內凍結「靜默跳過」行為的既有測試（ADDED 已存在跳過、MODIFIED 缺目標跳過、fresh canonical 物化 MODIFIED）依新語意翻轉為拒絕斷言；此為刻意行為變更，提案相容性影響段已記載。

## Implementation Contract

- **行為**：對含任一違規 delta 的 change 執行 speclink archive（單筆或 bulk、本地或 remote）→ 非零 exit code，錯誤逐條列出（capability／操作／需求名／原因）並附補救指引；工作區零變化——正典未動、無新 snapshot、change 仍在 openspec/changes/ 原位。多 capability 下任一 capability 違規即全部不寫。
- **成功路徑**：全數通過 → snapshot 先於正典寫入，正典更新後 change 移入封存區，行為與現行一致（@trace 注入邏輯本刀不動）。
- **Purpose**：新 capability 且 delta 含 `## Purpose` → 新正典 Purpose 為該區段內容；無 `## Purpose` → 現行 TBD 骨架。既有 capability 的正典 Purpose 永不被 delta 改動。
- **scenario 刪除聲明**：MODIFIED 區塊含 `<!-- REMOVED-SCENARIO: X -->` 時，正典合併結果不含 scenario X 且不含聲明註解本身；缺聲明又缺 scenario → 拒絕並點名遺失的 scenario。
- **介面**：無新 CLI 子指令、無新旗標；`--json` 既有欄位 shape 不變，新增之錯誤內容經既有 error 呈現通道。drift 的 assumption reason 字串更新為拒絕語意。
- **驗收**：speclink-core 的 archive 單元測試（翻轉後）全綠；cargo test -p speclink-core --test it 的 render_golden 與 crates/speclink-cli/tests/ 整合測試（含 --no-color 人眼斷言、fs／remote 對照、remote_verb_parity）於 golden 同批更新後全綠；新增測試涵蓋六條違規、兩階段零寫入、Purpose 帶入、刪除聲明。
- **範圍界線**：in scope——speclink-core archive 合併與 drift reason 文案、speclink-cli 錯誤呈現、archive 技能文字、對應測試與 golden。out of scope——@trace／evidence、sync、fingerprint、UI、server 路由新增。

## Risks / Trade-offs

- [golden 與 CLI 測試大面積變動掩蓋意外回歸] → 刻意變更逐項列於提案；先跑既有測試取得基線，翻轉測試與 golden 更新分開提交審閱
- [in-flight change 帶過期 delta 突然封不了] → 錯誤訊息內建補救動線（drift → ingest）；bulk 預檢照舊提前過濾，單筆拒絕是新增的最後防線
- [Windows 換行使 scenario 名比對失準] → 解析前 `\r\n` 正規化，測試含 CRLF 樣本
- [commit 階段 I/O 失敗留半套] → snapshot 先行寫入保證可恢復；錯誤訊息指出 snapshot 位置
- [三套判定收斂時 drift 輸出形狀意外變動] → drift 僅換 reason 文案、shape 不動，以 CLI 測試釘住

## Migration Plan

無資料遷移。已封存歷史與既有正典不受影響。部署後第一次封存過期 delta 的使用者會看到拒絕與補救指引；回滾即還原版本，無狀態殘留。

## Open Questions

（無——逃生口與聲明機制已在本設計定案）
