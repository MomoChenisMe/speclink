## Why

任務清單的閱讀畫面靠「標題／內文／驗證」三段來收合任務：Speclink Desktop 的任務分頁如此，內嵌宿主自己畫的任務頁也照同一條規則。但 tasks 的指示與範本只要求每個任務寫出行為與驗證方式，沒有說要先寫一個短標題。Agent 照範本寫出來的任務常是一句沒有分隔的長句，整句被當成標題，畫面就不能收合。這是討論 embedded-host-needs-10 的六項裡，唯一由看畫面的人實際指出的問題。

另外，Desktop 的切分規則只認全形「：」與「驗證：」，所以英文與日文寫的任務永遠切不開；宿主那一側則只能猜規則，因為沒有任何文件寫明任務行怎麼切。

目標使用者有兩種：在 propose 與 ingest 階段寫任務清單的 Agent（它讀 tasks 的指示），以及在 Desktop 或宿主的畫面上追進度的開發者、PM 與 PO。

## What Changes

- 兩個內建產出流程（spec-driven 與 no-spec-change）的 tasks 指示加一條建議：每個任務先寫短標題，接分隔符號，再寫行為，最後一段寫驗證方式。三種語言各有一組固定的標記，照 instructions JSON 的 locale 選用：繁體中文是全形「：」與「驗證：」，日文是全形「：」與「検証：」，英文與其他語言是半形冒號加一個空白，以及「Verify: 」。指示裡的範例改成同一個形狀。
- 任務範本（兩個產出流程共用同一份）的佔位文字與開頭的說明改成同一個形狀。
- packages/ui 的任務切分函式認得三組標記。原本切得開的任務，切法完全不變；只有原本整句被當成標題的任務，才會多出內文或驗證。
- docs/verb-contract 的中英兩份各加一節，寫明任務行怎麼切成編號、標題、內文與驗證。讀 show 的 tasks 全文、自己畫任務頁的程式照這一節解析。
- 不新增也不變更任何 CLI 指令、旗標與 `--json` 欄位；不改設定欄位；不改任何技能檔，所以技能的版號不變。

### 相容性影響

- tasks 的指示文字與範本會變：對任一變更取 tasks 指示時，instruction 與 template 兩個欄位的內容和以前不同，其他欄位不變。釘住這兩段文字的測試只有 speclink-core 的 schema 單元測試，同批更新。
- 已經寫好的任務清單不遷移，也不會被改寫。任務的解析、勾選、stable ID、需求名稱的覆蓋檢查都以整行任務文字為準，不受寫法影響。
- 複製過產出流程的專案（專案層或使用者層的自訂 schema）用的是自己那一份指示，拿不到這條建議；要的話自行加進自己的 schema。
- Desktop 與 server-web 共用 packages/ui，所以兩邊的任務列同時生效。沒有全形冒號、但含有「半形冒號加空白」的舊任務，標題會變短、其餘成為內文；這只改顯示，不改檔案。

### 三種形態

- 本機 repo：可用。指示由內建產出流程提供。
- 連到遠端的 checkout：可用。Server 內嵌的引擎回同一段指示；要等 Server 升級到含這個變更的版本。
- 沒有本機規格檔的內嵌宿主：可用。宿主經呼叫入口取 tasks 指示時得到同一段文字；宿主自己畫任務頁時照 docs/verb-contract 的新章節解析。

## Non-Goals (optional)

見 design 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-schemas`: 新增一條需求，規定兩個內建產出流程的 tasks 指示建議短標題的寫法與三組標記，範本的佔位文字為同一個形狀。
- `desktop-app`: 修改「任務群組標題與章節標籤同款式」的切分規則，讓任務列認得日文與英文的標記；全形冒號與「驗證：」的既有切法不變。
- `user-documentation`: 新增一條需求，規定動詞參考文件載明任務行的切分規則。

查過的相關正式規格：workflow-schemas（instruction 單一來源、no-spec-change 內建 schema 的組成）、desktop-app（任務群組標題與章節標籤同款式、任務列的手動任務徽章）、task-identity（任務行內嵌不可變 stable ID）、change-analysis（需求名稱的覆蓋檢查）、manual-task-marker、user-documentation。後三者的行為不變。

## Impact

- Affected specs: workflow-schemas、desktop-app、user-documentation
- Affected crates and apps: speclink-core（內建產出流程的資產與它的單元測試）、speclink-cli（只加測試）、packages/ui（apps/desktop 與 apps/server-web 經它生效）
- Affected code:
  - Modified:
    - crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml
    - crates/engine/speclink-core/assets/schema/no-spec-change/fork.schema.yaml
    - crates/engine/speclink-core/assets/schema/spec-driven/tasks.template.md
    - crates/engine/speclink-core/src/workspace/schema.rs
    - packages/ui/src/docModel.ts
    - packages/ui/src/__tests__/docModel.test.ts
    - packages/ui/src/__tests__/taskList.test.tsx
    - crates/adapters/speclink-cli/tests/it/new_artifact.rs
    - crates/adapters/speclink-cli/tests/it/instructions_policy.rs
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
  - New: (none)
  - Removed: (none)
