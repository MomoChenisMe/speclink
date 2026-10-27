## Context

正式規格的需求以「整塊取代」合併：MODIFIED delta 必須帶齊整條需求與所有情境，封存時覆蓋正式規格。本變更的 delta 是在 propose 時由腳本從當下的正式規格機械產生的；spec-dir-migration 與 desktop-spec-dir-migration 會先封存，之後也可能有其他變更改到同一批需求。若本變更封存時仍用 propose 時的快照，就會把別人後來的修改蓋回去。

## Goals / Non-Goals

**Goals:**

- 正式規格裡的舊名稱（D2 排除者除外，含描述 speclink repo 自身檔案的需求）與使用者文件裡描述使用者專案的舊名稱全部改為新名稱，行為不變。
- 封存時的 delta 一定是從當下正式規格產生的，不會覆蓋其他變更的修改。

**Non-Goals:**

- 不改遷移兩刀新增或修改的需求；不改使用者文件與 LANGUAGE.md 裡描述 speclink repo 自身檔案的文字與相對連結（見 proposal Non-Goals）。正式規格不在此限。
- 不改程式。

## Decisions

### D1：改寫規則

對每條需求依序套用：`.openspec.yaml` → `.meta.yaml`；`openspec/` → `speclink/`；前後都不是英數字、底線、斜線、句點或連字號的獨立字 `openspec` → `speclink`（例如「本地 openspec 樹」；`openspec.migrated`、`has_local_openspec` 這類不改）。大寫的 OpenSpec 不改。需求標題不得含舊名稱（目前沒有）；情境名稱含舊名稱時，改名並在 MODIFIED 區塊加 `<!-- REMOVED-SCENARIO: <舊名> -->`。每個 MODIFIED 區塊在標題下加一行 `<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->`。例外：兩處依語意改寫、不做機械替換，重新產生時一併套用——workspace-tools「update 清除孤兒技能目錄」的情境「不帶 force 的 init 改寫殘留技能檔」裡的「無 `openspec/`」改為「無規格資料夾」（只有舊資料夾的目錄也算專案根，換成「無 `speclink/`」會失去「未初始化」的意思）；change-diff-scope「Git-backed discovery scope 解析完整 worktree patch」裡的「openspec artifacts」改為「規格資料夾內的 artifacts」（換成「speclink artifacts」會讀成泛指 Speclink 的所有產物）。

### D2：排除清單

不改寫的需求：spec-dir-migration capability 的全部需求；store-abstraction「預設檔案系統佈局」「遠端封存變更資料檔的鍵」與已移除的「預設檔案系統佈局不變」；context-projection「投影佈局與 manifest」；user-documentation「使用者文件載明本地產物的純文字可攜性」「入門教學與設定說明載明規格資料夾的遷移」；workspace-tools「工作區補齊入口」；desktop-config「未初始化目錄經確認後自動初始化」「未啟用資料夾經確認後補齊啟用」「畫面依實際規格資料夾與資料檔名顯示」；desktop-app「技能檔提示承載規格資料夾遷移」；workspace-migration「遷移成功後才轉換本地且一律備份」「新舊預設資料夾都可遷移到遠端」。這些需求由遷移兩刀撰寫，已用新名稱，且刻意以舊名稱描述相容行為。描述 speclink repo 自身檔案的需求不在排除清單：正式規格一律改為新名稱，repo 由 self-spec-dir-migration 跟上（見 proposal Non-Goals）。

### D3：apply 時重新產生

apply 的第一個任務在遷移兩刀封存之後執行：以 D1 與 D2 從當下 openspec/specs 重新產生每份 delta，用 `speclink new artifact spec <capability> --change spec-dir-path-rewrite --force --stdin` 覆蓋；不再需要改寫的 capability 刪除其 delta，新出現的補上；proposal 的 Capabilities 同步更新。理由：propose 時的快照可能已過時；重新產生的成本只有一次腳本執行。

### D4：Purpose 與文件

六份正式規格的 Purpose 段直接修改（Purpose 不經 delta）。使用者文件只改描述使用者專案的路徑與 sdk-node 的 `specDir` 預設值；指向 repo 內檔案的相對連結與描述 repo 自身的段落留給 self-spec-dir-migration。openspec/LANGUAGE.md 的詞條定義裡的產品路徑一併改寫。

## Implementation Contract

- 封存前，`grep -rnE 'openspec/|\.openspec\.yaml'` 對本變更的 delta 只命中 BEFORE 與 REMOVED-SCENARIO 註解；每個 MODIFIED 區塊的情境名稱與當下正式規格相同（改名者除外）。驗收：`speclink validate spec-dir-path-rewrite` 與封存的合併守門。
- 文件：中英兩版一一對應，`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過（連結檢查確保沒有改壞 repo 內連結）。
- **範圍內**：delta、六段 Purpose、列出的文件與 LANGUAGE.md。**範圍外**：D2 排除的需求、使用者文件與 LANGUAGE.md 裡 repo 自身的描述與連結、程式。

## Risks / Trade-offs

- [合併時蓋掉別人的修改] → D3 在封存前重新產生；`speclink plan` 的 requirementOverlap 也會在有其他進行中變更改到同一條需求時排出封存順序。
- [誤改 repo 自身的連結] → D4 只改產品路徑；連結檢查會抓出改壞的相對連結。
- [獨立字規則誤判] → 改寫後逐份檢查 diff 中被改的 `speclink` 字樣都指規格資料夾。
- [規格與 repo 暫時不一致] → 本變更封存到 self-spec-dir-migration 提交之間，描述 repo 自身的需求寫 `speclink/`、repo 仍在 `openspec/`；兩者同一版發布，self-spec-dir-migration 依這些需求修正 repo 的工具與文件。

## Migration Plan

不需要。依賴 spec-dir-migration 與 desktop-spec-dir-migration，在兩者封存之後封存，與它們同一版發布。

## Open Questions

(none)
