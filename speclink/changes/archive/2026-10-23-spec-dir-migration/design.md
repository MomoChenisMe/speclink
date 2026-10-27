## Context

引擎早有 `.speclink.yaml` 的 `spec_dir`（workspace/config.rs 的 `AppConfig`），`Workspace::discover` 在沒寫時以 `"openspec"` 為預設，並把含 `openspec/` 或殘留 `.speclink.remote.yaml` 的資料夾認作專案根。實際上許多地方繞過這個設定、直接寫死 `"openspec"`：init.rs（遠端 init、`app_config_text` 的預設判斷、`update` 與探測的預設值、一處 `store_init`）、CLI verbs/init.rs 的預設與「Not initialized」判斷、Node SDK 的兩個預設、server api/context.rs 的投影路徑、CLI verbs/instructions.rs 的投影根、host bridge.rs 與 drift.rs 的顯示路徑，以及 model.rs、plan.rs、change_schema.rs、command/mod.rs、analyzer.rs 訊息裡的 `openspec/...` 字面。變更資料檔名 `.openspec.yaml` 寫死在 speclink-fs lib.rs 六處、host bridge.rs 三處、server read_api.rs 一處，另有 CLI query.rs、plan.rs 的「invalid .openspec.yaml」標示。

技能原稿大多已用 `{{SPEC_DIR}}`（workspace/skills.rs 以 `SyncPlan` 的 spec_dir 取代），只剩 commit、apply、discuss、propose、ingest、archive、drift 七份有寫死的 `openspec/` 或 `.openspec.yaml`。CLI 的 common.rs 已有殘留連接檔的「每次一行 stderr 警告」前例（`warn_leftover_remote_file`）。核心的 `util::git` 已可執行 git。桌面的寫死處由 desktop-spec-dir-migration 處理。

## Goals / Non-Goals

**Goals:**

- 新專案用 `speclink/` 與 `.meta.yaml`；舊專案不遷移也照常可用；`speclink update` 在安全時自動遷移、不安全時暫緩並說明。
- OpenSpec 專案以 `speclink init` 匯入，不覆蓋既有內容。
- 新 CLI 能讀新舊兩種 server 的投影。

**Non-Goals:**

- 桌面 app 的提示、監看與文案（desktop-spec-dir-migration）。
- 正式規格與其餘文件裡的舊路徑改寫（spec-dir-path-rewrite）；本 repo 自身的遷移（self-spec-dir-migration）。
- 遠端資料庫的鍵搬移；讓舊版 CLI 讀新佈局；自動 commit 或 push。

## Decisions

### D1：規格資料夾在 discover 單點決定

`Workspace::discover` 依「明寫 spec_dir → `speclink/` 存在 → `openspec/` 存在 → `speclink`」決定 `spec_dir_name`，並另外帶一個佈局狀態（明寫、預設、舊佈局、OpenSpec 專案、新舊並存），專案根的判定加上 `speclink/`。`speclink/` 要像規格資料夾（其中有 `changes/`、`specs/`、`discussions/` 或 `config.yaml`）才算數，免得上層資料夾裡的 speclink repo clone 或同名套件被誤認成專案根。所有寫死 `"openspec"` 的程式改讀 `Workspace::spec_dir_name`；沒有 workspace 的地方（`update`、探測、Node SDK）改呼叫同一個解析函式。替代方案「把預設值常數從 openspec 改成 speclink 就好」被否決：舊專案會立刻變成空專案。

### D2：資料檔名集中在 speclink-core

speclink-core 定義 `META_FILE = ".meta.yaml"` 與 `LEGACY_META_FILE = ".openspec.yaml"`。speclink-fs 的 `Layout` 提供變更與已封存變更的資料檔路徑；讀取先新後舊；寫入寫新檔並在同一個方法內刪除舊檔（刪除失敗回報錯誤，新檔已寫入，下次寫入會再刪）。host bridge.rs 封存時以 `META_FILE` 為已封存鍵，server read_api.rs 與 bridge 讀已封存資料檔時先新鍵後舊鍵。訊息裡的路徑照 command-runtime「錯誤訊息指出 metadata 檔的 workspace 相對路徑」寫成 `<規格資料夾>/changes/<name>/.meta.yaml`：`Store` 新增 `spec_dir_display()`（預設回傳 `speclink`，speclink-fs 回傳實際資料夾名）與 `change_meta_display()`（speclink-fs 在只有舊檔名時顯示 `.openspec.yaml`，讓訊息指向實際存在的檔案），`MetaError` 帶 `file` 欄位，`require_valid_meta`、`check_meta_text` 與 plan.rs、change_schema.rs、command/mod.rs 的訊息由它組路徑；analyzer.rs 的正式規格路徑同樣以它組成。舊佈局顯示 `openspec/…`、新佈局與遠端顯示 `speclink/…`。apps/desktop/src-tauri/src/remote.rs 直接建構 `MetaError` 的一處與斷言舊訊息的一個測試隨之連動（遠端用預設資料夾），其餘桌面改動仍屬 desktop-spec-dir-migration。替代方案「只寫 `.meta.yaml`、不帶資料夾」被否決：違反 command-runtime 的 workspace 相對路徑要求，且 invalid 訊息看不出是哪個變更。

### D3：佈局警告在 CLI 單點印出

common.rs 新增與 `warn_leftover_remote_file` 同位置呼叫的函式，依 D1 的佈局狀態印三種一行警告之一；明寫 spec_dir、遠端模式與預設佈局不印；`init` 與 `update` 自己處理佈局，也不印（避免遷移當下又提示去執行 update）。stdout 與 exit code 不受影響。

### D4：遷移邏輯集中在 workspace/migrate.rs

`plan(root, app) -> MigrationPlan`（`update` 用：無事可做、暫緩並附原因、搬資料夾、整理殘留舊檔名；OpenSpec 佈局一律暫緩為 `OpenSpecProject`）、`plan_import(root)`（`init` 用：OpenSpec 佈局走同一套搬移檢查）與 `apply(root, plan) -> MigrationReport`。暫緩原因以 `Display` 提供同一套文案，`update` 的提示與 `init` 的拒絕共用。`speclink` 已被規格資料夾以外的東西佔用時暫緩為 `SpecFolderTaken`，不讓 `update` 每次在同步技能前失敗；旁邊已有 `.meta.yaml` 的 `.openspec.yaml` 是過時舊檔，不算殘留。整理殘留舊檔名時，`speclink/` 完全沒有被 git 追蹤的檔案代表遷移本身還沒 commit（例如資料夾已搬、資料檔改名中途失敗），不算未提交修改，讓再次執行的 `update` 能接著改名。git 檢查：`git rev-parse --git-dir` 與 `--git-common-dir` 不同即為 linked worktree；`git worktree list --porcelain` 有主目錄以外的項目即有其他 worktree；`git status --porcelain --untracked-files=all -- openspec` 有輸出即有未提交修改；不在 git repo 裡（`git rev-parse` 失敗）時跳過這三項。搬移順序：先以單一 `std::fs::rename` 改名資料夾（失敗即整體失敗、沒有任何檔案移動），再逐一改名 `changes/*` 與 `changes/archive/*` 下的資料檔；中途失敗回報該路徑，已改名的保留，下次 `update` 由「整理殘留舊檔名」接手。替代方案「用 git mv」被否決：未追蹤檔案無法用 git mv 搬，且不在 git repo 的專案也要能遷移。

### D5：update 與 init 共用遷移

`update` 先呼叫 D4，再以遷移後實際的規格資料夾解析 `SyncPlan`；沒有 `.speclink.yaml` 的 OpenSpec 專案不由 `update` 遷移，只提示改用 `init`；`UpdateOutcome` 多帶 `MigrationReport`，CLI 依序印出改名結果、要 commit 的路徑與兩則提醒，或暫緩原因。`init` 在「沒有 `.speclink.yaml`、有 `openspec/`、沒有 `speclink/`、spec_dir 為預設」時走匯入：先做與 D4 相同的 git 檢查（不通過即拒絕、零寫入），再搬移，然後照常執行 `store_init`（`write_if` 不覆蓋既有 config.yaml）、寫 `.speclink.yaml` 與技能檔，最後印出匯入提醒。CLI verbs/init.rs 的 spec_dir 預設改為 `speclink`，「Not initialized」判斷改用 D1 的根目錄規則。舊佈局專案（`.speclink.yaml`＋只有 `openspec/`）在預設 spec_dir 下執行 `init --force` 時就地在 `openspec/` 重新初始化，`.speclink.yaml` 不寫 `spec_dir`，避免另建空的 `speclink/` 讓既有規格讀不到。`adopt` 以 D1 解析出的資料夾補骨架，不再寫死 `openspec`。

### D6：投影路徑與鏡像根

server api/context.rs 的路徑前綴改為 `speclink/`。CLI verbs/instructions.rs 不再寫死 `join("openspec")`，改從剛 materialize 的快照（manifest 的文件路徑）取第一個路徑段作為鏡像根；快照沒有任何文件時用 `speclink`。新 CLI 因此可讀舊 server（`openspec/`）與新 server（`speclink/`）的投影。

### D7：技能原稿與版號

七份原稿的 `openspec/` 改為 `{{SPEC_DIR}}`、`.openspec.yaml` 改為 `.meta.yaml`；`SyncPlan` 以 D1 的實際資料夾渲染。workspace/skills.rs 裡 manual 技能說明寫死的 `openspec/manual/` 改為 `{{SPEC_DIR}}manual/`，生成 frontmatter 時 description 套用與內文相同的替換。內建 schema 的 instruction 與 proposal 範本是不經專案渲染的靜態文字，其中 8 行寫死的 `openspec/specs/`、`openspec/config.yaml` 改為新預設 `speclink/`；`schema fork` 產出仍與正典 YAML 逐位元組相同，instructions 輸出相應改變並同批更新期望值。`.speclink.yaml` 範本的 spec_dir 註解改為說明規格資料夾（預設 `speclink`）。golden 測試以新預設渲染，所有用到 `{{SPEC_DIR}}` 的 snapshot 會改變。ASSET_VERSION 由 v1.52.0 升為 v1.53.0，再生 golden、assets.lock 與 repo 自用技能檔。

### D8：測試夾具

CLI 整合測試以二進位檔在暫存資料夾執行；手寫 `openspec/` 樹的夾具在新版會觸發佈局警告並落入舊佈局路徑。夾具集中改為 `speclink/` 與 `.meta.yaml`，只有專門驗證舊佈局、遷移與匯入的測試保留 `openspec/`；直接以參數建立 `FsStore`／`Workspace` 的單元測試帶明確的資料夾名，不受預設值影響，維持原樣。

## Implementation Contract

- **佈局解析與警告**：依 spec-dir-migration「規格資料夾的預設位置與舊佈局相容」的五個情境成立。驗收：speclink-core workspace 單元測試與 `cargo test -p speclink-cli --test it spec_dir_layout`。
- **資料檔名**：依 store-abstraction「預設檔案系統佈局」與「遠端封存變更資料檔的鍵」的情境成立。驗收：`cargo test -p speclink-fs`、speclink-core 與 `cargo test -p speclink-server` 的測試。
- **遷移與匯入**：依「update 自動遷移舊佈局」「init 匯入 OpenSpec 專案」的情境成立，含 git repo 與非 git 資料夾。驗收：migrate.rs 單元測試與 `cargo test -p speclink-cli --test it spec_dir_migration`。
- **投影**：依 context-projection 修改後的三個情境成立。驗收：`cargo test -p speclink-host` 與 `cargo test -p speclink-cli --test it remote_read_path`。
- **技能**：依「技能檔依實際規格資料夾渲染」成立，原稿不再含 `openspec/`。驗收：render_golden 與 skill_authoring。
- **範圍內**：proposal Impact 列出的檔案、測試夾具、版號與再生物。**範圍外**：apps/desktop、正式規格路徑改寫、本 repo 自身遷移。

## Risks / Trade-offs

- [回歸對照：golden 大量改變] → 差異只能是 `{{SPEC_DIR}}` 的渲染結果與七份原稿的改動；逐份檢查 diff，出現其他差異即代表渲染路徑被改壞。
- [回歸對照：CLI 測試夾具] → D8 集中修改；先跑一次全部 CLI 整合測試列出因佈局警告或路徑失敗的測試，再逐檔改夾具，避免漏改。
- [跨平台：Windows 上資料夾被占用時 rename 失敗] → D4 的資料夾改名是單一動作，失敗即零移動並回報；桌面 app 開著監看時可能占用，CLI 訊息提示關閉後重試。
- [跨平台：路徑分隔] → 鏡像根與資料檔路徑一律以元件逐段 join，邏輯路徑輸出用正斜線。
- [git 不在 PATH] → `util::git` 回傳失敗時視為不在 git repo，跳過檢查並照常遷移；這與「非 git 專案可遷移」一致，記為已知取捨。
- [版本混用] → 舊版 CLI 遇到已遷移專案會看到空專案；舊版 CLI 讀不到新 server 的投影路徑。兩者都寫進發版說明，提醒團隊與 server 一起升級。

**sharp-edges 稽核結論（task 8.1）**，每項都有測試釘住：

- `openspec` 是符號連結 → 暫緩（`DeferReason::SymlinkedFolder`）：改名只會搬動連結本身，資料檔改名會落到可能與其他專案共用的目標資料夾；`update` 說明原因，`init` 拒絕且零寫入。
- `speclink` 已存在但不是規格資料夾（例如檔案或同名套件）→ 暫緩（`DeferReason::SpecFolderTaken`），`update` 照常同步技能；`apply` 搬移前仍檢查目標，回報「already exists — nothing was moved」作為競態防護，也藉此擋掉 rename(2) 會靜默取代空資料夾的行為。資料檔的目標 `.meta.yaml` 已存在時不改名覆蓋：已有 `.meta.yaml` 的變更不列為殘留，`apply` 遇到目標已存在仍拒絕。
- 資料夾跨檔案系統 → 既定取捨：`openspec/` 與 `speclink/` 是同一層的兄弟，必在同一個檔案系統；若 `openspec` 是掛載點等導致 rename 失敗，單一 rename 失敗即零移動並回報（以唯讀父資料夾模擬 rename 失敗的測試釘住）。
- 變更資料夾名含空白、開頭為 `-`、非 ASCII 或含 `.` → 照常搬移：路徑只經 `Path` API 組合，不經 shell；git 指令的 pathspec 只用固定常數（`openspec`／`speclink`），不帶使用者輸入。
- git 不在 PATH → 既定取捨：視為不在 git repo，跳過三項檢查照常遷移（CLI 測試以空 PATH 釘住）。git 可用但 `git status` 失敗 → 保守處理，視為有未提交修改而暫緩；`--absolute-git-dir` 不支援的舊版 git → 改以 `.git` 是否為檔案判斷 linked worktree。

## Migration Plan

發版後，既有專案在使用者執行 `speclink update`（或桌面 app 的技能檔更新）時於安全條件下遷移；不遷移也可繼續使用並看到警告。回滾：遷移後若退回舊版，把 `speclink/` 改回 `openspec/` 並把 `.meta.yaml` 改回 `.openspec.yaml` 即可，內容未被改寫。本變更依使用者決定在第一批六刀發版之後才封存。

## Open Questions

(none)
