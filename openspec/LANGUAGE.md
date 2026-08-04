# Language — 共用詞彙

專案的正典詞彙。GUI 文案、artifacts 散文、skills 說明遵循此表；Rust 識別符、CLI 輸出（英文）、
結構標記（### Requirement: 等）不在此範圍。

## 原則

- 動詞直說結果：使用者看到動詞就能推出「按下去會發生什麼」。
- 一個概念一個詞：同義詞在 avoid 列出，舊文案陸續汰換；歷史 artifacts（已封存的討論／變更）不回改。
- 工程詞（meta 欄位名、kebab-case、slug 等）不出現在使用者可見文案，只出現在給 agent 的文件。
  - **明文例外**：設定檔檔名（config.yaml、.speclink.yaml）得直出作為設定頁的頁簽標籤——與本原則刻意抵觸，經使用者裁定：開發者工具中檔案即最直觀的心智模型，人話標籤（如「專案設定／整合」）反而多一層對應。僅限頁簽標籤；其他使用者可見文案仍禁工程詞（desktop-window-and-settings-polish，2026-07-08）。
  - **明文例外**：討論（discuss）的識別錨點得以 slug（kebab-case 檔名）直出——與本原則刻意抵觸，經使用者裁定：slug 是執行 CLI 動詞（如 `--from-discussion <slug>`）的把手，開發者工具中檔名即最直觀且可複製的識別，與 change 卡以名稱為題對稱；topic 一律降為描述／副標。僅限討論識別錨點（討論全卡標題、已轉出細列首行、討論抽屜標題、系統匣討論項父標籤、變更詳情抽屜與已封存抽屜的來源討論籤及其溢出浮層）與其複製鈕／複製動作（含系統匣的「複製 slug」）；其他使用者可見文案仍禁工程詞（desktop-card-identity，2026-07-09；範圍擴充：desktop-ux-polish，2026-07-11；範圍擴充：tray-copy-and-panel-mode，2026-07-16；範圍擴充：change-drawer-header-redesign，2026-08-04）。
  - **明文例外**：「worktree」得直出於 worktree 流程的使用者可見文案——與本原則刻意抵觸，經使用者裁定：worktree 是 git 的原生概念，本專案的使用者就是 git 使用者，直出即最短且無歧義的心智模型；任何中譯（「工作樹」「副本工作區」）都要多一層對應，且與使用者實際輸入的 `git worktree` 指令對不上。先例為 config.yaml 頁簽與討論 slug（同一條「開發者工具中原生詞即最直觀」的裁定線）。適用範圍：worktree 流程的使用者可見文案，含 `speclink list` 的 `[worktree]` 標示、兩個 worktree 技能的說明與輸出，以及後續 desktop 的卡片 worktree 標示與抽屜分支資訊；其他使用者可見文案仍禁工程詞（討論 worktree-parallel-apply，2026-08-04）。

## 詞彙

### 轉為變更

- **definition**: 把一份已結論的討論升級成一個新的變更（change）——建立變更卡、提案以討論結論開頭、討論記為已轉出。對應引擎動詞 `discuss promote`。
- **avoid**: 促轉、promote（中文散文中）
- **why**: 「促轉」是自造縮譯，無法從字面推出結果；「轉為變更」與看板「變更」頁名直接呼應。

### 已轉出變更

- **definition**: 討論的 promoted 狀態——至少連結一個變更（轉出新變更，或以引擎動詞 `discuss link` 併入既有變更）。看板上以「已轉出變更的討論」群組收合呈現。
- **avoid**: 已促轉
- **why**: 同上；名詞化後仍可讀。定義自「轉出過至少一個變更」放寬：ingest 型結論經 link 併入既有變更也走同一狀態與生命週期（discuss-link-verb，2026-07-08）。

### 再轉出一個變更

- **definition**: 對同一份討論再次轉為變更（一份討論可扇出多個變更）。
- **avoid**: 再促轉
- **why**: 扇出語意明確。

### 封存

- **definition**: 把完成的變更或收尾的討論移入 archive（`openspec/changes/archive/`、`openspec/discussions/archive/`），於「已封存」頁唯讀檢視。對應引擎動詞 `archive`。
- **avoid**: 歸檔
- **why**: 「封存」是 change 側與已封存頁的既定詞；同概念兩詞（歸檔/封存）曾在討論卡按鈕上並存。

### 退回提案中

- **definition**: 把誤開工的變更自「進行中」退回「提案中」——移除開工戳記（started_* 欄位），僅零工作痕跡（無已勾任務、無 touched 記錄）時可行。對應引擎動詞 `in-progress remove`。
- **avoid**: 撤回開工、取消開工、unstart（中文散文中）
- **why**: 動詞直說結果——按下去卡片回到哪一欄一目了然，與看板欄名「提案中」直接呼應。

### 衍生變更

- **definition**: 一份討論轉出的變更清單（討論抽屜的分頁名；引擎欄位 `promoted_to`）。
- **avoid**: 促轉分頁、子 change（使用者可見文案中）
- **why**: 分頁內容是「生出來的變更們」，不是動作本身。

### 輪

- **definition**: 討論的推進單位（引擎的 round）。文案寫「N 輪」「討論 N 輪」。
- **avoid**: 回合、N 回合
- **why**: 口語、更短。

### 背景

- **definition**: 討論記錄的 Context 區段（討論抽屜分頁名）。
- **avoid**: 脈絡（分頁名中）
- **why**: 較常用的日常詞。

### 專案說明

- **definition**: `openspec/config.yaml` 的 `context` 欄位——注入 AI 指令的專案自由文字說明（設定頁的編輯區段名）。
- **avoid**: context（使用者可見文案中）、背景（此概念上）
- **why**: 「背景」已被討論記錄的 Context 區段佔用，同詞兩義會混淆；對齊 Spectra 用詞。2026-07-07 討論「config-context-與-rules-gui-編輯」定案。

### 產出規則

- **definition**: `openspec/config.yaml` 的 `rules` 欄位——依 artifact 注入產出指令的規則清單（設定頁的編輯區段名）。
- **avoid**: rules（使用者可見文案中）、規則（單獨使用時）
- **why**: 「產出」點明規則作用於 artifacts 的產出過程；對齊 Spectra 用詞。2026-07-07 討論「config-context-與-rules-gui-編輯」定案。

### 待收尾

- **definition**: 等使用者執行動詞的卡片——已就緒（任務全數完成、等待封存）的變更＋已結論未轉出（等待轉為變更或封存）的討論。專案分頁徽章顯示的即為待收尾數。
- **avoid**: 進行中（此概念上）、待處理、pending（使用者可見文案中）
- **why**: 「進行中」代表 agent 在做事、不需要人；徽章要傳達的是「有事等你動手」的行動訊號，兩者混用會讓徽章失去催辦意義。2026-07-11 討論「spec-archive-drawer-ux」定案。

### 換頁

- **definition**: 清單分批瀏覽（pagination）。artifacts 散文稱「換頁」；UI 文案不出現「換頁／分頁」名詞，僅用「上一頁」「下一頁」「第 N／M 頁」。
- **avoid**: 分頁（pagination 語意上）
- **why**: 「分頁」已被抽屜 tabs 語意佔用（提案／設計／任務／規格分頁），同詞兩義會使規格與討論記錄歧義。2026-07-11 討論「specs-archive-pagination」定案。

### 專案代號

- **definition**: project key——建立專案時指定、之後不可變更的識別字串。介面以唯讀文字呈現並標示「建立後不可變更」，與可更名的「專案名稱」分離。
- **avoid**: Project key、project key、專案 key、專案 ID（使用者可見文案中）
- **why**: 「key」在中文文案裡會與「金鑰」混淆（同一個介面上還有存取金鑰）；「代號」點明它是識別而非密鑰。2026-07-25 變更「admin-console-redesign」定案。

### 儲存庫代號

- **definition**: repo key——建立儲存庫時指定、之後不可變更的識別字串。與「專案代號」同一語彙，同樣以唯讀文字呈現。
- **avoid**: Repo key、repo key、儲存庫 key
- **why**: 同「專案代號」。2026-07-25 變更「admin-console-redesign」定案。

### 建立專案

- **definition**: 新增一個專案（含指定專案代號與名稱）的動作，管理面「專案與儲存庫」頁的 primary action。
- **avoid**: 建立 project、新增 project
- **why**: 中英夾雜的動作名無法與頁名「專案與儲存庫」呼應；工程詞不出現在使用者可見文案。2026-07-25 變更「admin-console-redesign」定案。

### 存取金鑰

- **definition**: Personal Access Token（PAT）——使用者自助建立、供遠端工作流程以其身分連線的長效憑證。明文只在建立時顯示一次。
- **avoid**: Personal Access Tokens、PAT、權杖、token（使用者可見文案中）
- **why**: 「PAT」是縮寫工程詞，使用者無法從字面推出用途；「存取金鑰」同時說明了「拿來存取」與「是機密」。2026-07-25 變更「admin-console-redesign」定案。

### 登入工作階段

- **definition**: Web session——瀏覽器登入後持有的 cookie 工作階段，帳號頁以唯讀清單呈現其建立與到期。
- **avoid**: Web Sessions、session（使用者可見文案中）
- **why**: 「session」未譯時與「裝置登入」難以區分；「登入工作階段」明示它是這一次瀏覽器登入的存續期。2026-07-25 變更「admin-console-redesign」定案。

### 資料結構版本

- **definition**: identity schema version——識別資料庫的結構版本，呈現於系統頁的執行環境與總覽的系統健康摘要。
- **avoid**: Schema 版本、schema version、識別 schema
- **why**: 「schema」對非開發者無意義，且遷移動作的文案（「執行資料結構遷移」）本來就用「資料結構」，兩處必須同詞。2026-07-25 變更「admin-console-redesign」定案。

### 待送佇列

- **definition**: outbox backlog——某個範圍尚未同步出去的事件筆數，呈現於系統頁的儲存狀態。
- **avoid**: Outbox backlog、outbox、backlog
- **why**: 「backlog」在中文語境常被讀成「待辦事項」，與總覽的「需要處理」撞義；「待送佇列」點明是「還沒送出去的東西排著」。2026-07-25 變更「admin-console-redesign」定案。

### 儲存後端

- **definition**: TeamStore 的實作與其執行狀態——驅動（sqlite／postgres／serverfs 等）、契約版本、等級、能力與健康，統一以「儲存後端」為主詞，呈現於系統頁。
- **avoid**: Store、Store 狀態、store driver（使用者可見文案中）
- **why**: 「Store」單獨出現時看不出是什麼的儲存；統一主詞後，「儲存後端目前無法使用」這類降級訊息才讀得通。2026-07-25 變更「admin-console-redesign」定案。

### 審查

- **definition**: 對 change 實作的 code review——工藝品質檢查（repo 慣例＋code smells＋bug 獵捕），與「驗證」並行的可選品質站。對應 skill `/speclink-review` 與引擎動詞 `review`。狀態詞：審查中（工單未結）、已審查（蓋章）、已審查·其後有變動（失效降級）、曾審查未通過（封存時工單無章）。
- **avoid**: code review、代碼審查、覆審（使用者可見文案中）
- **why**: 「審查」單詞直說動作，與「驗證」形成對仗的兩個品質站——一管工藝、一管合規。2026-07-31 討論「code-review-stage」定案。

### 驗證

- **definition**: 對 change 實作的 spec 合規檢查（verify 三維度：完整、正確、一致），與「審查」並行的可選品質站。對應 skill `/speclink-verify`。狀態詞同構：驗證中、已驗證、已驗證·其後有變動、曾驗證未通過。
- **avoid**: verify（中文散文中）、校驗
- **why**: 與「審查」對仗；兩站在看板上的章與狀態詞必須同構，使用者才能以同一心智模型讀懂兩種標示。2026-07-31 討論「code-review-stage」定案。
