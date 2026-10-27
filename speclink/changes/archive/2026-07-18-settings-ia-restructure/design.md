## Context

現行 `apps/desktop/src/views/SettingsView.tsx` 是單一設定頁、四頁簽（config.yaml、.speclink.yaml、本機設定、伺服器），整頁由 `App.tsx` 以 `activeSession !== undefined` 守門。三個結構性問題：

- 應用程式層級內容（介面語言、伺服器連線）被鎖在 session 之下：零分頁時設定入口不可達，使用者無法把 remote workspace 當第一個 workspace 開啟。
- remote 分頁下 config.yaml／.speclink.yaml 不適用，現行以 `workspaceSettingsNotice` prop 觸發「四簽＋兩張提示卡」分支——remote-data-source 驗證期的臨時修補，未入規格。
- 專案層（兩個設定檔）與應用程式層（本機、伺服器）混在同一組頁簽，資訊架構混淆；使用者手動驗證時明確要求拆開。

相關正典需求：`desktop-config`「設定頁圖形化讀寫兩層設定」（三頁簽組織）、`desktop-app`「側欄導覽結構」（四導覽項、設定沉底）、`desktop-connections`「伺服器管理最小面」（明載「設定資訊架構重整不屬本能力」——由本 change 承接）。

## Goals / Non-Goals

**Goals:**

- 應用程式設定頁（本機設定＋伺服器）獨立於任何專案分頁，零分頁仍可進入與操作（含從伺服器簽開啟第一個 remote workspace）。
- 專案設定頁（config.yaml＋.speclink.yaml）跟隨 active 分頁；remote 分頁整頁呈現單一不可用說明。
- 移除 `workspaceSettingsNotice` 四簽提示卡分支，行為由頁面拆分正規化。

**Non-Goals:**

- 不改動各設定卡的內容與寫入行為（三卡、雙重解析驗證、解析失敗簽級警示、tools 技能同步全數沿用）。
- 不改動 ServersPanel 的功能面（新增／登入／登出／移除／開啟 workspace 均沿用）。
- 不做獨立設定視窗（macOS Preferences 式）；維持單視窗切頁導覽。
- 不提供 remote workspace 的專案設定編輯（server 端政策管理是未來能力）。
- 不動 `packages/ui`。

## Decisions

### 決策 1：專案設定入側欄頂部群組、應用程式設定沉底

側欄頂部群組成為四項：變更、規格、已封存、**專案設定**——頂部群組語意統一為「現行專案的頁面」；底部沉底項維持標籤「設定」，內容改為應用程式設定頁。替代案「底部放兩項（專案設定＋設定）」被否決：底部會混入專案範圍項，破壞「頂部＝專案、底部＝應用程式」的分層。

### 決策 2：路由狀態擴充而非新機制

`store.ts` 的 `boardView` union 增加 `"project-settings"`；既有 `"settings"` 改指應用程式設定頁。App.tsx 渲染順序調整：`settings` 分支**不再依賴 activeSession、先於零分頁 EmptyState 分支**；`project-settings` 於零分頁時落入既有 EmptyState fallthrough（與變更／規格／已封存同型）。五個導覽項恆常渲染，不做條件顯隱。替代案「專案設定項於零分頁時隱藏」被否決：條件導覽增加狀態面，且與其他專案頁的零分頁行為不一致。

### 決策 3：檔案拆分與命名

`SettingsView.tsx` 拆為兩檔、原檔刪除：

- `apps/desktop/src/views/AppSettingsView.tsx`——本機設定簽（介面語言卡、tray 錯誤呈現、「僅存於此裝置」註記）＋伺服器簽（ServersPanel 接線）。
- `apps/desktop/src/views/ProjectSettingsView.tsx`——config.yaml 簽（三卡）＋.speclink.yaml 簽（AI 工具卡），含簽首等寬字路徑註記、解析失敗簽級警示；remote 時整頁單一不可用說明卡。

測試同型拆分：`settingsView.test.tsx` 拆為 `appSettingsView.test.tsx` 與 `projectSettingsView.test.tsx`、原檔刪除——既有案例依所屬頁面搬移，不可用提示卡案例改寫為單卡斷言。替代案「保留 SettingsView 名稱作專案設定頁」被否決：名稱與內容不符，徒留誤導。

### 決策 4：簽序與預設簽

專案設定頁簽序 config.yaml、.speclink.yaml，預設 config.yaml（沿用正典）。應用程式設定頁簽序本機設定、伺服器，預設本機設定。現行「remote 時預設落伺服器簽」的補丁行為隨提示卡分支一併移除——remote 下使用者要找伺服器面，入口就是側欄「設定」。

### 決策 5：remote 專案設定呈現

remote 分頁下專案設定頁不渲染頁簽，整頁單一說明卡（沿用 `data-testid="settings-unavailable"` 與 `remote.settingsUnavailable` 文案）；導覽項照常可點。App.tsx 以 `activeSession.locator.kind === "remote"` 判定傳入 notice——`ProjectSettingsView` 收 `notice?: string`，有值即渲染說明卡並跳過 settings 讀取（沿用現行 useEffect 守門式）。

## Implementation Contract

**行為（使用者可觀察）：**

- 側欄五導覽項：頂部依序變更、規格、已封存、專案設定；底部「設定」。各項皆為切頁＋高亮語意（沿用）。
- 點「設定」：任何狀態（含零分頁、remote active 分頁）皆進入應用程式設定頁——本機設定簽（介面語言三選即時生效）與伺服器簽（連線清單與操作、開啟 workspace）。零分頁時自伺服器簽開啟 remote workspace 成功後切至看板並出現新分頁。
- 點「專案設定」：local active 分頁見 config.yaml（專案說明、產出規則、產出政策三卡）與 .speclink.yaml（AI 工具卡）兩簽，預設 config.yaml，簽首等寬字註記檔案路徑；remote active 分頁見單一不可用說明卡、無頁簽；零分頁見空狀態引導頁。
- 兩設定檔的讀寫、驗證、解析失敗警示行為與現行逐項一致（回歸保護）。

**介面／資料形狀：**

- `store.ts`：`boardView: "board" | "specs" | "archived" | "settings" | "project-settings"`。
- `AppSettingsView` props：`localePref`、`onLocalePrefChange`、`trayPanelError`、`servers`（形狀沿用現行 `SettingsViewProps` 對應欄位）。
- `ProjectSettingsView` props：`settings`（`WorkspaceSettings` 介面，沿用）、`notice?: string`。
- i18n 新鍵：`app.navProjectSettings`（zh「專案設定」／en「Project Settings」），zh 與 en 鍵集合維持相等。

**失敗模式：**

- config.yaml／.speclink.yaml 解析失敗：僅影響專案設定頁對應簽（警示點＋表單停用），應用程式設定頁不受任何專案檔解析失敗影響。
- remote 專案設定：不可用說明卡為終態，不發出 settings 讀取呼叫。

**驗收判準：**

- `npm test -w apps/desktop` 全綠，含新測試：（a）零分頁時設定頁可進入且伺服器簽可操作；（b）remote 分頁專案設定頁呈現單一說明卡且無 settings 讀取；（c）五導覽項渲染與切頁高亮；（d）既有 settingsView 案例於拆分後對應檔全數保留通過。
- 手動 GUI 驗證：零分頁開 app → 設定 → 伺服器簽開啟 remote workspace 成功；local 分頁專案設定頁三卡讀寫如常。

**範圍邊界：**

- in scope：`apps/desktop/src`（views、App.tsx、store.ts、i18n、tests）。
- out of scope：`packages/ui`、`src-tauri`（Rust 端零改動）、設定卡內容行為、ServersPanel 功能面。
