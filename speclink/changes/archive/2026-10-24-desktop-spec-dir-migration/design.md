## Context

前一刀 spec-dir-migration 在 speclink-core 建立了規格資料夾解析（`Workspace::discover` 的佈局狀態）、workspace/migrate.rs 的遷移計畫與執行，以及 `update` 回報的遷移報告。桌面這邊：

- 開專案時 apps/desktop/src/store.ts 呼叫 `probeAssets`，由 apps/desktop/src/assetPrompt.ts 決定是否顯示 `AssetNoticeCard` 與對話框，按「更新」呼叫 apps/desktop/core/src/project.rs 的 `update_assets_at`（委派引擎 `update`）；`probe_assets_at` 委派引擎 `probe_assets`，回傳的 `AssetProbe` 只有技能檔狀態，且以寫死的 `openspec` 為預設。
- 「啟用」走引擎 init.rs 的 `adopt`，它對 `openspec/` 就地補齊；「初始化」走 `init(path, tools, false, "openspec")`。
- 檔案監看（apps/desktop/src-tauri/src/watch.rs）由 `resolve_watch_targets` 依專案探索解析目標，`rearm` 以 ticket 判斷是否重掛，前端 store 已有 `rearmWatch`。
- 本機遷移到遠端在 apps/desktop/src-tauri/src/remote.rs：要求 `spec_dir_name == "openspec"`、備份名寫死 `openspec.migrated-<date>`、上傳時以 `.openspec.yaml` 為已封存鍵。
- 文案：apps/desktop/src/i18n/messages.ts 的初始化、空狀態、來源選擇、並存衝突、遷移備份與設定分類欄共 11 組中英字串寫死 `openspec/`；四個專案設定頁的 `PageNote` 寫死 `openspec/config.yaml`；packages/ui 的 `card.invalidMetaTitle` 寫死 `.openspec.yaml`；`DiscussionDetailPage` 以 `openspec/discussions/<slug>.md` 組記錄路徑。

## Goals / Non-Goals

**Goals:**

- 桌面使用者經由既有的技能檔提示完成遷移，知道會發生什麼、遷移後畫面持續更新。
- 桌面「啟用」OpenSpec 專案與 CLI `init` 匯入的結果相同。
- 畫面不再寫死 `openspec/` 與 `.openspec.yaml`。

**Non-Goals:**

- 不新增獨立的遷移畫面或背景遷移。
- 不改 CLI 的遷移與匯入行為（前一刀已完成）；唯一例外是 D6 讓 `workflow-config set worktree` 的技能同步不再遷移。
- 不支援明寫自訂 `spec_dir` 的專案遷移到遠端（維持拒絕）。
- server-web 不在範圍內（它沒有本機規格資料夾）。

## Decisions

### D1：探測回報遷移狀態

引擎的 `AssetProbe` 新增 `migration` 欄位（camelCase 序列化：`state` 為 `none`／`pending`／`deferred`，`pending` 附 `metaFiles`，`deferred` 附 `reason` 與 `worktrees`），以 migrate.rs 的 `plan` 計算，與 `update` 同源；`probe_assets` 的預設資料夾改用 D1（前一刀）的解析。替代方案「桌面自己判斷」被否決：判定會與 CLI 分岔。

### D2：提示規則

assetPrompt.ts 在技能檔過期、缺失、較新之外，`migration.state` 為 `pending` 時也產生提示；技能檔已是現版而遷移暫緩時不提示——略過沿用「專案＋版號」的既有鍵，暫緩時的略過會壓掉使用者排除暫緩原因後的待遷移提示，所以暫緩原因只在技能檔過期或缺失的對話框裡說明。`AssetNoticeCard` 與對話框依 `migration` 顯示待遷移內容或暫緩原因；`update_assets_at` 的回傳加上遷移報告，對話框完成態顯示要 commit 的路徑與兩則提醒。完成且確實遷移時，store 呼叫既有的 `rearmWatch`，監看依 `resolve_watch_targets` 重新解析出 `speclink/`。

### D3：adopt 匯入

init.rs 的 `adopt` 在規格資料夾為舊佈局 `openspec/`（且無 `speclink/`）時，先執行與 init 匯入相同的 git 檢查與 migrate.rs 搬移，失敗即回錯誤且零寫入；之後對 `speclink/` 執行 `store_init` 補缺件、`.gitignore` 與 `reconcile_builtin_tools`。`.speclink.yaml` 不寫 `spec_dir`。桌面啟用對話框在 `Unadopted` 探測結果附帶「需要匯入」旗標時顯示改名說明。

### D4：初始化與文案

apps/desktop/core/src/project.rs 的初始化改用預設資料夾（`speclink`）。設定快照帶出專案實際的規格資料夾相對路徑，`PageNote` 與分類欄組名以它組成；討論詳情的記錄路徑以資料來源帶出的規格資料夾組成（資料來源沒有帶出時以 `speclink` 組成）；`card.invalidMetaTitle` 改為「變更資料檔解析失敗」。i18n 中講新專案的說明文字改為 `speclink/`，講本機既有資料夾的文字改為「本機規格資料夾」。

### D5：本機遷移到遠端

remote.rs 的前置檢查改為資料夾名稱為 `speclink` 或 `openspec` 即放行（含明寫為這兩個名稱的 `spec_dir`），其他明寫的自訂資料夾拒絕；`speclink/` 與 `openspec/` 並存時也拒絕，免得只備份其一、轉成 checkout 後一開就是本機與遠端並存；備份名以實際資料夾名組成 `<name>.migrated-<date>`，序號規則不變；上傳 bundle 時已封存變更的資料檔讀取先新後舊，一律以 `.meta.yaml` 為鍵。

### D6：技能足跡同步不遷移

apply 期發現：切換 worktree 政策後的技能足跡同步（apps/desktop/core/src/settings.rs 的 `write_workflow_fields_at` 與 CLI verbs/config.rs 的 `workflow-config set`）都呼叫引擎 `update()`，舊佈局專案因此會被靜默改名——桌面沒有對話框、沒有暫停監看，CLI 只印「skills synced」。init.rs 新增只同步技能檔的入口（`update` 去掉遷移那一步，降級守門照舊），兩個呼叫端改用它；`update` 本身不變。替代方案「同步時照樣遷移、但補上說明」被否決：設定存檔不是使用者預期改名的時機。

## Implementation Contract

- **探測與提示**：desktop-app「技能檔提示承載規格資料夾遷移」的五個情境成立。驗收：`cargo test -p speclink-core` 的 probe 測試、`cargo test -p speclink-desktop-core` 的 `probe_assets_at`／`update_assets_at` 測試、`npm test -w apps/desktop` 的 assetPrompt 與對話框測試。
- **啟用匯入**：workspace-tools「工作區補齊入口」與 desktop-config「未啟用資料夾經確認後補齊啟用」的情境成立。驗收：speclink-core 的 adopt 測試與 `npm test -w apps/desktop` 的啟用流程測試。
- **文案與標示**：desktop-config「未初始化目錄經確認後自動初始化」與「畫面依實際規格資料夾與資料檔名顯示」的情境成立。驗收：`npm test -w apps/desktop`、`npm test -w packages/ui`。
- **遠端遷移**：workspace-migration 修改與新增需求的情境成立。驗收：`cargo test -p speclink-desktop`（含 sidecar 與 server-web dist 的前置）。
- **同步不遷移**：spec-dir-migration「技能足跡同步不遷移規格資料夾」的情境成立。驗收：speclink-core 的同步入口測試、speclink-cli 的 `workflow-config set worktree` 測試、speclink-desktop-core 的 `write_workflow_fields_at` 測試。
- **範圍內**：proposal Impact 所列檔案與對應測試。**範圍外**：CLI、server-web、背景遷移、自訂資料夾遷移到遠端。

## Risks / Trade-offs

- [回歸對照：desktop 測試以 `openspec/` 建夾具] → 夾具集中改為 `speclink/`，只有驗證舊佈局與匯入的測試保留 `openspec/`；apps/desktop/core/src/testfixture.rs 是集中點。
- [跨平台：Windows 上監看占用資料夾使改名失敗] → 遷移前不先停監看會讓 rename 回錯誤；D2 的「更新」在呼叫 update 前先暫停監看、完成後再 `rearmWatch`，失敗時錯誤原樣顯示在對話框且資料夾不動。
- [遠端遷移備份名改變] → 只影響新遷移；既有 `openspec.migrated-*` 備份照舊保留。
- [AssetProbe 形狀擴充] → 新欄位為選填、camelCase；前端對缺席欄位視為 `none`，舊引擎的回傳仍可用。

## Migration Plan

不需要資料遷移。本變更依賴 spec-dir-migration，並在第一批六刀發版之後才封存。回滾時，已遷移的專案依前一刀的回滾方式處理。

## Open Questions

(none)
