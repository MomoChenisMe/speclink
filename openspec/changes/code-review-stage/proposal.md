## Why

apply 完成後、封存之前，speclink 只有 verify 一個品質站——它以 spec 為中心檢查合規，但工藝品質（repo 慣例、code smells、spec 沒寫到的 bug）沒有任何檢查點。本變更新增與 verify 並行的可選審查站 `/speclink-review`：參考 Matt Pocock code-review skill 的兩軸平行架構、把 Spec 軸讓給既有 verify，並以「工單＋章」讓審查狀態可跨 session／跨機器交接、審查結果在看板上可見。

目標使用者：透過 AI 代理跑 SDD 的開發者。使用情境：workflow 品質站階段 `apply ⇄ ingest → (review? ∥ verify?) → archive`——任務全數完成（已就緒）後、封存之前，由使用者自行判斷是否對高風險 change 執行審查。

## What Changes

- **引擎品質站機制**（speclink-core）：以可參數化的生命週期實作，本變更僅出審查站實例（verify 對稱補齊為後續變更）：
  - 工單 `openspec/changes/<name>/review.md`：round append-only、每輪記本輪範圍與分級 findings（CRITICAL／WARNING／SUGGESTION）、僅由 CLI 動詞讀寫、蓋章即刪；本地模式隨 git、remote 模式走既有 store 文件管道；工單不進 artifact DAG（sidecar，validate／status 不理它）
  - 章：change metadata（.openspec.yaml）新增 `reviewed_at`／`reviewed_by`／`reviewed_with`、`reviewed_tasks_total`（任務錨）、`reviewed_scope`（內容指紋錨：範圍檔 path＋hash 清單）；欄位缺席讀作未審查
  - 失效：任務狀態偏離蓋章時的全完成，或範圍檔內容指紋不符 → 標示降級「已審查·其後有變動」；指紋比對不依賴 git
- **CLI 新增 `speclink review` 子命令家族**（speclink-cli）：
  - `review add-round <change> --stdin`：追加一輪（工單不存在則建立）；內容格式不符 → 非零 exit code 與錯誤說明
  - `review show <change> [--json]`：印出工單；`--json` 供 skill 與續輪 subagent 取末輪待辦
  - `review stamp <change>`：蓋章並刪除工單；守門＝全任務完成＋末輪零未解 findings；`--accept` 允許帶保留蓋章；守門不過 → 非零 exit code 與原因
  - `review discard <change>`：刪除工單、不蓋章
- **archive 動詞**（speclink-core）：偵測未結工單時預設不放行，並說明處置選項（先蓋章／先放棄／明示帶走——帶走的化石工單即封存後「曾審查未通過」標示的證據）；無工單時行為完全不變
- **新 skill `/speclink-review`**（skills.rs 正典化＋golden 再生，claude 與 codex 兩工具皆生成）：主線 orchestrator——初審以 touched 檔集定界（無 touched 記錄時詢問 git 基準）、平行發 Standards（repo 慣例＋逐字內嵌的 smell baseline：12 條 Fowler smells（Refactoring, ch.3）與兩條約束規則，照抄 Matt Pocock code-review skill（MIT）、repo 文件優先）與 Correctness（bug 獵捕）兩個 read-only sub-agent、change artifacts 當判準脈絡但不產合規裁決、結果並列呈現不合併不重排、迴圈詢問（修正後重審／接受蓋章／先不蓋）、修正一律回主線；迴圈收斂機制——逐筆 finding 裁量分類（必修／可裁）且三選項帶推薦、修正後下一輪前專案建置與測試全綠的驗證門、已接受事項續輪前饋（sub-agent 不重報、主線帶入續輪記錄）；產出語言綁定——sub-agent 指示攜帶 workflow config 解析後的 locale、呈現與工單記錄同語言不翻譯（severity 標籤與軸前綴留英文，locale 未設定則全英文）
- **生成指令檔更新**（instructions.rs）：CLAUDE.md／AGENTS.md 的 workflow 行改為 `discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive`、技能使用清單加入審查站
- **desktop 標示**（packages/ui＋apps/desktop＋協定）：卡片行內小章、抽屜審查資訊列；狀態機 active＝無標示／審查中（工單存在）／已審查／已審查·其後有變動，archived＝已審查／曾審查未通過；desktop 封存入口的未結工單三選項提示
- **README／docs** 同步 workflow 圖與審查站說明

相容性影響：

- `speclink list --json` 不變（parity pin：新欄位不進 CLI 公開輸出，僅進 desktop 協定），回歸對照不破壞
- archive 僅在「存在未結工單」這個新情境改變行為；既有 change 無工單、完全不受影響
- metadata 新欄位缺席讀作未審查，pre-migration change 不需遷移

## Non-Goals

- verify 品質站對稱補齊（`verified_*` 章、verify.md 工單、驗證標示）——設計已於討論 code-review-stage 定案，作為後續變更另行轉出
- server-web console 的凍結度計算——無工作樹可重算指紋，顯示章但凍結度標 unknown
- 蓋章後「新增的範圍外檔案」之指紋偵測——影響小，接受為代價
- per-finding ID 與逐項 resolve 動詞——末輪清單即工單
- 審查 sub-agent 自動修正 code——審查者 read-only，修正一律回主線
- git 錨定（HEAD／commit hash）——蓋章時工作樹通常 dirty，HEAD 不代表被審狀態
- CLI `list --json` 輸出擴充

## Capabilities

### New Capabilities

- `review-station`: 引擎品質站機制——review 動詞家族、工單生命週期、章與雙錨失效、archive 未結工單守門
- `review-skill`: `/speclink-review` 技能——兩軸平行審查、迴圈詢問、蓋章；裁量分類、驗證門與接受前饋的收斂機制；產出語言跟隨 locale；skill 模板與 workflow 文案正典化

### Modified Capabilities

- `desktop-app`: 卡片／抽屜審查標示與封存入口三選項提示
- `client-protocol`: desktop 協定新增審查狀態欄位（含凍結度）

## Impact

- Affected specs: `review-station`（新增）、`review-skill`（新增）、`desktop-app`（修改）、`client-protocol`（修改）
- Affected code:
  - New: crates/speclink-core/src/review.rs
  - Modified: crates/speclink-core/src/model.rs、crates/speclink-core/src/archive.rs、crates/speclink-core/src/skills.rs、crates/speclink-core/src/instructions.rs、crates/speclink-core/tests/golden（再生）、crates/speclink-cli（review 子命令註冊，入口檔於 design 定位）、crates/speclink-host 與 crates/speclink-protocol（審查狀態欄位向 desktop 曝光）、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/ArchivedDrawer.tsx、packages/ui/src/adapter.ts、apps/desktop（tauri adapter 與 i18n 文案）、README.md、README.en.md
  - Removed: 無
