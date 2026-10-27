## Why

Speclink 的流程規則目前分散於 README、入門教學、生成 skills、CLI help、正典 specs 與目標架構文件；部分使用者文件又已落後實作，造成開發者、PO、PM 無法快速判斷該走哪條流程、哪些能力現在可用、哪些仍是規劃。需要建立漸進揭露且有單一責任的文件資訊架構，讓首次使用者能走完第一輪，也讓進階使用者能查到每個階段的用途、使用時機、分流與限制。

## What Changes

- 新增中英文完整 SDD 工作流指南，涵蓋 onboard、discuss、propose、apply、ingest、drift、analyze／validate、audit、commit、archive，以及討論結論後「直接 propose／快速轉為變更／link→ingest→seal／不做而封存」的選擇；每個階段都說明目的、何時使用或跳過、輸入、產物、Agent skill 與底層 CLI、完成判準、下一步與常見恢復路徑。
- 新增中英文產品能力狀態文件，以「可用／部分可用／規劃中／已棄用」分類 Local CLI、skills、Desktop、Node SDK、Server、Remote Workspace 與營運能力；每列附現行證據、限制與目標文件連結，避免把已存在 crate、已封存 spec 或目標架構單獨當成已交付證據。
- 保留並僅微調 README 首段的專案定位與起源敘事，包括品牌圖片、標語「一套 SDD Engine，支援 Local Repo 與 Remote Store」、繁體中文／English 切換、Rust SDD 引擎定位、PM／PO／RD／AI Agent 共用語意、Local Repo 與 Remote Store 雙路徑、Spectra App 2.3.1 CLI 相容基線，以及目前狀態與路線圖入口；校正既有英文 README 的過時事實、術語與連結，並將詳細流程與完整能力矩陣導向各自正典文件。
- 將中英文 getting-started 收斂為經實際 CLI／skill 驗證的 Local Repo 最短成功路徑，修正不存在的 verify skill、不同 Agent Host 的呼叫語法、optional artifacts、討論轉出與封存詞彙等現況落差。
- 在平台架構藍圖與實作重構路線圖只補文件定位、目前能力狀態連結與查核日期；前者維持唯一目標架構正典，後者維持其下的執行伴隨文件，不改既有 Phase 順序。
- 更新 openspec/config.yaml 的專案說明，使後續 Agent 不再以過時的「只有 core／cli、無 async／無網路」背景產生新 artifacts；不改 workflow policy 欄位或預設值。
- 建立可重複執行的文件驗證清單：相對連結存在、中英文成對文件章節對齊、範例指令可由目前 CLI help／skills 觀察、禁止將規劃中能力寫成已可用、正典詞彙一致。

## Non-Goals

- 不改 CLI、Rust crates、Desktop、Server、Node SDK 或生成 skills 的執行行為；文件若揭露產品缺口，只標示現況或拆出後續 change，不在本 change 補功能。
- 不重寫平台架構藍圖或調整 implementation roadmap 的 Phase／Gate 順序，也不建立第二份競爭性的架構或 Roadmap。
- 不把 README 改成只有導覽連結的極簡入口，也不移除或大幅壓縮既有品牌圖片、專案目的、雙部署路徑與相容性起源敘事。
- 不把歷史 changes／discussions 當成使用者手冊重新整理或回寫；它們維持稽核資料。
- 不在本 change 重建已移除但仍被舊 spec 引用的完整 docs/verb-contract.md；先在產品狀態與文件地圖揭露該進階契約文件缺口，完整契約整理另立範圍。

## Capabilities

### New Capabilities

- `user-documentation`: Speclink 使用者文件的資訊架構、完整工作流說明、產品能力狀態分類、中英文一致性與可驗證準確性。

### Modified Capabilities

（無）

## Impact

- Affected specs: 新增 `user-documentation`。
- Affected code:
  - New: `docs/workflow.md`、`docs/workflow.zh-TW.md`、`docs/product-status.md`、`docs/product-status.zh-TW.md`
  - Modified: `README.md`、`README.en.md`、`docs/getting-started.md`、`docs/getting-started.zh-TW.md`、`docs/platform-architecture.zh-TW.md`、`docs/implementation-refactor-roadmap.zh-TW.md`、`openspec/config.yaml`
  - Removed: （無）
- 目標使用者與情境：首次導入 Local Repo 的開發者、透過 AI Agent 參與需求與規格的 PO／PM、需要續作／改需求／封存的 RD，以及評估 Desktop／Server／Remote 能力是否可用的部署與整合人員。
- 影響的 crate：`speclink-core`、`speclink-cli` 與其他 crates 皆不修改；只讀其現行命令、skills、specs 與測試作為文件證據。
- CLI 相容性：無子指令、旗標、stdin、stdout、stderr、exit code 或 `--json` 變更，parity 回歸對照不受影響。
- 設定相容性：只更新 `openspec/config.yaml` 既有 `context` 散文，不新增或改變 `.speclink.yaml`／`openspec/config.yaml` 欄位、預設值與解析順序。
- 技能相容性：Claude 與 Codex 既有 skills 內容與注入區塊不變；使用者文件會分別說明 Claude 的 slash command、Codex 的 `$skill` 與直接 CLI 三種呼叫層級。
