## Why

桌面 app 的「新增 Workspace」chooser 第一步，右側來源卡寫「Speclink Server」。這個字把一個通用概念（要連的那台 server）綁死在產品品牌上：使用者要選的是「哪一台 server」，不是「哪一個品牌的 server」。同一個流程裡的其他文案早已直接寫「Server」（「遷移到 Server…」「以 Server 為準」），只有這張卡與它的下一步標題還帶品牌前綴，形成同一畫面前後不一致。

目標使用者是透過 AI 代理跑 SDD 的開發者、PO 與 PM；使用情境是 desktop app 的 workspace 開啟入口——這是使用者接觸 Remote SDD 的第一個畫面。影響範圍限於 apps/desktop 與其規格、手冊，不涉及任何 crate 的行為。

## What Changes

- **desktop 使用者可見文案去品牌**（`apps/desktop/src/i18n/messages.ts`，中英各一份共 6 行）：
  - `chooser.server`：`Speclink Server` → `Server`（截圖中的來源卡標題）
  - `chooser.serverTitle`：`選擇 Speclink Server` → `選擇 Server`；英文 `Choose a Speclink Server` → `Choose a Server`
  - `servers.help`：「已儲存的 Speclink server 連線。」→「已儲存的 server 連線。」；英文同步去掉 `Speclink` 前綴
- **規格字面同步**（`workspace-chooser` capability 的兩條 Requirement）：「來源分流」與「最近開啟清單」的 SHALL 內文把「Speclink Server」改為「Server」。UI 改而規格不改，封存時兩邊對不上。
- **正典補詞條**（`openspec/LANGUAGE.md`）：新增「Server」詞條，definition 說明它指 speclink server 服務端，`avoid` 列 `Speclink Server`，`why` 沿用既有的「開發者工具中原生詞即最直觀」裁定線（先例：config.yaml 頁籤、討論 slug、worktree 直出）說明為何直出英文而非中譯。
- **測試定位改為錨定開頭**（`apps/desktop/src/__tests__/workspaceChooser.test.tsx`，5 處）：現行以 `/Speclink Server/` 取按鈕。改文案後若直接換成 `/Server/`，在「最近開啟」清單同時渲染的案例會同時命中來源卡與名為「團隊 Server」的 remote 條目，`getByRole` 會因多重命中而拋錯。定位須改為錨定開頭的形式。
- **手冊與架構文件同批更新**：`openspec/manual/desktop-projects.md`（2 處）、`openspec/manual/desktop-remote.md`（1 處）、`docs/platform-architecture.zh-TW.md`（2 處）。

**相容性影響**：不新增、不修改任何 CLI 子指令、旗標、stdin 或 exit code，`--json` shape 與人眼輸出皆不變，golden 與 CLI 整合測試無涉。唯一的行為面改動是 desktop UI 的三段顯示字串；不涉及 i18n 鍵名更名，故無任何呼叫端需要遷移。生成的技能資產（`crates/speclink-core/assets/skills/`）不含這批字面，ASSET_VERSION 不需要 bump。

## Non-Goals

- **不統一「Server」與「伺服器」的中英用詞**。設定頁頁籤、`servers.*` 全族與系統匣 `tray.recovery.server` 目前用中文「伺服器」，chooser／migration 族用英文「Server」。這是與品牌前綴正交的既存矛盾，一次做完會把 6 行擴大到約 25 個文案鍵加對應測試，讓這一刀失焦。留待另一場討論。
- **不擴充詞彙守門測試以涵蓋純 ASCII 詞**。`scripts/vocabulary-guard.test.mjs` 只把含中日韓文字的 avoid 詞納入守門集，`ui-copy-vocabulary` 規格對此有明文要求，測試中亦有專條斷言釘死。因此本次新增的 `Server` 詞條屬人工判斷用的正典記錄，機械守門不會涵蓋它。改動這個機制等於改 `ui-copy-vocabulary` 的契約，超出本變更範圍。防漂回改由 chooser 測試的錨定定位承擔。
- **不回改歷史 artifacts**。`openspec/changes/archive/` 與 `openspec/discussions/archive/` 下帶「Speclink Server」的封存內容維持原狀，依正典原則「歷史 artifacts 不回改」。
- **不動指產品本身的 Speclink 字**。`tray.open`（「開啟 Speclink」）、`assets.*` 技能檔說明、`apps/server-web` 的 `setup.title` 等處，Speclink 指的是產品而非 server，保留。
- **不更名任何 ASCII 識別符**。i18n 鍵名 `chooser.server`、`chooser.serverTitle`、`servers.help` 及元件名維持不變。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-chooser`：「新增 Workspace 的來源分流」與「最近開啟清單」兩條 Requirement 的 SHALL 內文，把來源卡的字面約束由「Speclink Server」改為「Server」。既有規格掃描（`speclink list --specs`）顯示只有此 capability 把該字面寫進 Requirement 內文；`ui-copy-vocabulary` 雖管使用者可見文案的詞彙約束，但其守門集只收中日韓詞彙，本次新增的是純 ASCII 詞，該 capability 的 Requirement 無需改動；`desktop-connections` 管的是 connection registry 與登入路徑，不涉及 chooser 的顯示字面。

## Impact

- Affected specs: `workspace-chooser`
- Affected code:
  - Modified:
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/workspaceChooser.test.tsx
    - openspec/LANGUAGE.md
    - openspec/manual/desktop-projects.md
    - openspec/manual/desktop-remote.md
    - docs/platform-architecture.zh-TW.md
  - New: (none)
  - Removed: (none)
- Affected app: `apps/desktop`（僅顯示字串與其測試定位）
- 不受影響：全部 crate（`speclink-core`、`speclink-cli`、`speclink-host`、`speclink-server` 等）、`apps/server-web`、`packages/ui`、`openspec/config.yaml` 與 `.speclink.yaml` 的任何欄位、生成的技能與 Agent 指令。
- 回歸面：`npm test -w @speclink/desktop -- workspaceChooser`（文案與定位改動的唯一驗證點）、`node --test "scripts/**/*.test.mjs"`（確認 LANGUAGE.md 新詞條不改變守門集，且不照出既有存量）。
