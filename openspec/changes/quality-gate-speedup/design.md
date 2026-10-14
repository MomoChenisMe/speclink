## Context

品質關卡由三份技能編排（`/speclink-review`、`/speclink-verify`、兩站合跑的 `/speclink-quality`），引擎提供工單動詞（`review|verify add-round／show／stamp／discard`）與 host 端的範圍解析（`review prepare`、`review|verify scope`）。討論 quality-gate-speed 量到的現況：

- CLI 動詞約 0.03 秒；時間耗在技能流程本身。
- 兩站技能的修復迴圈要求「完整建置與測試」，本 repo 一次 `cargo test --workspace` 約 40 分鐘；openspec/config.yaml 的 rules.tasks 早已規定全量只由 CI 守門。
- quality 技能先跑完整個審查站才進驗證站；兩站鎖定範圍各寫各的 snapshot 命名空間（speclink-host 的 change_diff 模組以 `StationNs` 區分 `snapshots`／`verify-snapshots` 目錄），檢查本身互不依賴。
- 子代理在「自己讀幾萬字差異檔＋無目標搜尋」的簡報下會 600 秒無進度被中止；改成內貼差異、列名稱、限次數後七次零卡住（2026-09-17 實測）。
- 引擎 `add_round` 只驗 Phase／Patch 格式與輪次順序，不驗 finding 路徑；帶 `:行號` 或範圍外的路徑要到下一輪 scope 解析才炸。
- 鎖定差異排除 `openspec/` 與 `.speclink/`，所以修補只改規格時，下一輪複驗看不到改了什麼。
- Apply 基準（`.speclink/review-scopes/<change>/baseline.json`）與兩站快照都是 host 本機資料、不進 git；worktree 移除時一起消失。evidence（`openspec/changes/<name>/.evidence.json`）與工單隨 commit 進主 checkout。
- worktree-merge 技能先在 worktree 內 rebase 到合併目標，再於主 checkout `merge --ff-only`；fast-forward 被拒或 rebase 衝突時退回一般 merge。

排程限制：在途變更 review-rules-config 同樣改 review 技能範本與 ASSET_VERSION（v1.44.0 → v1.45.0）。兩者的 review-skill delta 不碰同一條 Requirement（它改「Standards 軸的 smell baseline」「審查結果的裁量分類」並新增三條；本變更改「修復迴圈的驗證門」並新增兩條），但 review.md 的第 5 步簡報與第 9 步選項 1 兩段文字都會被兩邊改。本變更排在它之後：apply 等它封存，版號以 v1.45.0 為底升 v1.46.0。

## Goals / Non-Goals

**Goals:**

- 品質關卡每輪不再等全量測試；`/speclink-quality` 首輪不再排隊等審查站。
- 子代理不再因讀大檔或無目標搜尋卡住；複驗輪維持窄範圍。
- finding 路徑錯誤在寫入當下就被拒絕。
- 修補動到 artifact、session 中途關閉這兩種情況有明確、便宜的處理路徑。
- worktree 流程的 Apply 基準與快照跨合併存活：兩種路徑（worktree 內跑完再合併／先合併再在主 checkout 跑）都不需要人工 `--base`。
- 與 review-rules-config 的 CRITICAL 逐條確認在 quality 時序中不重問、不衝突。

**Non-Goals:**

- 拿掉每輪停頓、設輪數上限、改蓋章守門或接受機制。
- 把驗證站拆成多個子代理（延後：上線後驗證站經常比審查兩軸晚超過約兩倍時間才回來時重議，屆時依 capability 拆）。
- 保存使用者在停頓時的修／不修決定（接續時再答一次）。
- 單獨呼叫審查或驗證站時的「空差異不記一輪」——只寫進 quality 的接續規則。
- 替用舊技能合併的 worktree 變更自動推算 `--base`。
- 改審查站的審查面（仍是鎖定差異加直接影響所需的呼叫端與測試）。
- desktop、server API、工單檔案格式、Apply 基準檔案格式的任何變更。

## Decisions

### 測試門改跑受影響範圍

兩站修復迴圈的驗證門改為三段判斷：(1) 修補改到的檔案屬於哪個任務的範圍（任務說明列出的路徑），就跑該任務說明寫的驗證指令——`speclink instructions apply --json` 的 `tasks` 已帶任務原文；(2) 任務沒涵蓋的檔案，照專案寫好的測試政策（CLAUDE.md／AGENTS.md／workflow config 產出規則）；(3) 專案什麼都沒寫才跑全量。建置一律照跑。停頓報告列出「跑了哪些、略過哪些」，略過的交 CI。

- 審查站：改寫既有 Requirement「修復迴圈的驗證門」。
- 驗證站：正式規格沒有對應 Requirement（只有技能檔第 13 步的一句），新增「驗證修復迴圈的驗證門」，內容與審查站對稱。
- 替代方案：維持全量——每輪 40 分鐘換不到 CI 之外的保障；寫死一張 crate→指令對照表——只對本 repo 有效，產品技能要通用。

### 三個子代理同一則訊息派出

quality 技能每輪改為四段：準備（依序執行審查站第 1–4 步與驗證站第 1–5 步：選定 change、守門、查工單、鎖定範圍、讀 artifact；含 review-rules-config 加入的 `speclink workflow-config show --json` 讀專案審查規則）→ 派出（照兩站各自的簡報規則組好三份簡報，在同一則訊息派出審查 Standards 軸、審查 Correctness 軸、驗證檢查）→ 收回（依序完成審查站第 6–9 步、驗證站第 10–13 步，工單先寫審查再寫驗證，兩站都走「先不蓋章」出口）→ 停頓（既有第 3 步）。

- quality 技能的「只承載時序」原則不變：它決定什麼時候派，不改寫簡報內容。兩站技能的派出步驟各加一句：在 quality 時序中，簡報照本站規則組，由 quality 與另一站的簡報一起派出。
- harness 無法同時派三個子代理時，沿用兩站既有的「無法派子代理就依序自跑」退路。
- 收尾補蓋呼叫照舊依序（審查章先、驗證章後）；兩站都需要先跑 validation 時，同樣三個一起派。
- 替代方案：維持依序——首輪多等 4–6.5 分鐘，順序只為蓋章，與檢查無關。

### 子代理簡報的查找紀律

兩站技能新增同一套簡報規則（各自一條新 Requirement）：

| 項目 | 首輪 | 複驗輪 |
|---|---|---|
| 工具呼叫次數 | 不設上限 | 上限 5 次 |
| 禁止 | 整份讀大檔（用行號範圍）；沒有指定目標的遞迴搜尋（只搜簡報列出的函式名與檔案） | 同左 |
| 差異 25 KB 以內 | 直接貼進簡報 | 同左 |
| 超過 25 KB | 只給路徑與行號範圍 | 同左 |
| 自動產生的檔案（golden、重新產生的 SKILL.md 等） | 只給路徑，最多搜尋一次 | 同左 |
| 結尾 | 「沒查完就直說，不要猜」 | 同左 |

驗證站另加證據順序：先在鎖定差異裡找每條 Requirement 的實作證據；差異裡完全找不到時（例如由變更前既有程式碼滿足）才往外搜尋，且只搜具體函式名或檔名。這消除技能檔第 6 步（差異為程式碼證據）與第 7 步（每條 Requirement 搜整個 codebase）的矛盾。

- 子代理回報「沒查完」的部分照實列在停頓報告，由使用者決定要不要補查。
- 替代方案：首輪固定 15 次——最近 25 個封存變更平均 45 個 Scenario，驗證站查不完；依 Requirement 數縮放——卡住的真因不是次數。

### add-round 的 finding 路徑檢查

speclink-core 的 `quality::station` 新增純函式 `finding_path_violations`：輸入工單既有各輪、本輪內容與「免檢查的路徑前綴」，回傳違規清單。規則：

1. 路徑以冒號加數字結尾（如 `src/a.rs:42`、`src/a.rs:42-50`）→ 違規。
2. 路徑（`\` 正規化為 `/`）不在「工單既有各輪 Scope 的聯集＋本輪 Scope」之內 → 違規；聯集沿用既有 `scope_union`，與蓋章指紋範圍同一個定義。位於規格目錄（`<spec_dir_name>/`）或 `.speclink/` 之下的路徑免除這一項——複驗的範圍解析本來就略過它們（change_diff 的 `normalize_targets`），驗證站也常把 Requirement 層級的 finding 錨定在 delta 規格檔上。
3. 豁免：與工單上一輪某條 finding 的 severity、path、text（去掉結尾的 `(accepted)` 標記後）完全相同的行——沿用的未解項不受檢查，避免舊工單裡既存的壞路徑讓「原文沿用」寫不進去。

呼叫點在 speclink-cli 的 station 動詞：本機臂在呼叫 core `add_round` 之前、remote 臂在呼叫 typed client 送出之前，共用同一個檢查函式（以本機 Workspace 的 `spec_dir_name` 提供前綴；remote 臂先以既有的 station show 讀取工單）。違規時以 invalid_argv 類錯誤非零結束、零寫入，訊息逐一點名違規路徑。

- 為什麼不放進 core `add_round` 本體：core 經 `&dyn Store` 讀寫工單，不知道 repo 的規格目錄名稱（它來自本機設定，server 端也沒有）；把前綴當參數傳進 `add_round` 會牽動 server 路由與 protocol。CLI 是品質技能寫入工單的唯一入口，在 CLI 兩臂共用一個檢查即可涵蓋本機與 remote。
- 範圍外：server 直接收到的工單寫入（非經 CLI）不做這項檢查——技能一律經 CLI 寫入。
- 技能文字同步補一句：問題出在差異外的呼叫端時，finding 錨定在範圍內那個被呼叫的檔案。
- 替代方案：比對 host 端凍結快照——remote 模式的 server 讀不到 host 本機的快照，且快照只在有工單後才存在。

### 修補動到 artifact 的三種情況

寫進兩站的驗證門 Requirement 與 quality 的停頓規則：

- 情況 1，改程式碼符合規格：不碰 artifact。
- 情況 2，規格字面要修、意圖不變：修補步驟由主線直接改 artifact，接著跑 `speclink validate <change>`；停頓報告列出改了哪幾段；改過的段落原文放進下一輪驗證站的簡報（鎖定差異排除 `openspec/`）。
- 情況 3，意圖或範圍改變：走兩站既有的「範圍改變」出口——該站不蓋章結束，建議先 `/speclink-ingest` 再重跑品質關卡。
- 界線：修補需不需要新增任務，或改變某條 Requirement 的意圖。
- quality 停頓時，會動到 artifact 的 finding 先單獨標出。
- 替代方案：一律走完整 ingest——ingest 可能新增或退回任務，寫碼任務一旦未全完成，驗證站 add-round 被引擎拒絕、審查站開頭守門即停，迴圈卡死。

### 中斷後接續

quality 技能新增接續規則：再跑一次 `/speclink-quality` 即可，兩站的 scope 看到工單仍有必修項目就自動判為複驗。接續時多三條：

- 派子代理前先跑一次測試門（前一個 session 的測試狀態未知）。
- 某站鎖定範圍回傳的複驗差異為空（`files` 為空）且該站末輪仍有必修項目時，該站這輪不派子代理、不記一輪；兩站都空就直接回到停頓，拿上一輪 findings 問使用者——照跑會被判「沒有進展 → 未通過」。
- 提醒使用者：關閉後、接續前不要 commit——複驗範圍外新增的檔案靠 `git status` 的 dirty 判定進入複驗，commit 後會悄悄掉出範圍。

### Apply 基準以可信 commit 重設並承接快照

speclink-host 的 change_diff 模組新增兩個入口，CLI 的 `review prepare` 依旗標呼叫：

- `prepare_at(ws, change, base_rev)`：以 `git rev-parse --verify <rev>^{commit}` 解析出完整 SHA，以 `git merge-base --is-ancestor <sha> HEAD` 確認是 HEAD 的祖先；寫入與既有格式相同的 baseline（version 1、baseCommit 為該 SHA、dirtyFilesAtStart 為當下 dirty 路徑、confidence 為 initial）。目標已有 baseline 且任一站處於工單未結狀態時拒絕——該站的快照綁著舊基準。
- `carry_snapshots(ws, change, from_root)`：從來源 worktree 根目錄套用同一套 sidecar 路徑規則，把兩站的快照目錄（`snapshots`、`verify-snapshots`）逐檔複製到目前 checkout；快照檔名是 patch hash 的 digest（內容定址），目標已有同名檔就略過；逐檔以暫存檔＋rename 原子寫入。來源沒有快照目錄時什麼都不做。
- 寫入順序：(1) 驗證 rev、祖先關係、來源路徑存在、未結工單守門——任一失敗零寫入；(2) 複製快照；(3) 最後原子寫入 baseline。(2) 中途失敗時接受「部分快照已複製、baseline 未寫」的半套：快照內容定址，重跑會略過已存在的檔案並補完；多出的快照由既有「無對應工單時清除 orphan」規則處理。
- remote 模式：基準與快照本來就是 host 本機資料；CLI 的本機臂與 remote 臂都經同一個 `run_review_prepare` 呼叫 host，不另寫一份。
- baseline 的 serde 結構不變，新舊 CLI 都讀得懂。
- 跨平台：git 指令三平台行為一致；來源路徑以 `std::path` 處理，Windows 原生路徑可用；快照檔名本來就避開冒號。
- 替代方案：把 worktree 的 baseline.json 原樣複製——其 base 是 rebase 前的舊分岔點，主分支之後若有別的變更改到同檔，比對會混入別人的修改；開工時把基準寫進會 commit 的 metadata——同樣在 rebase 後過期；由技能事後推算——哪些 commit 屬於變更要靠判斷。

### worktree-merge 合併後寫入基準

worktree-merge 技能的合併步驟開始前，記下主 checkout 的 HEAD（`git -C <main-checkout> rev-parse HEAD`，此時主樹已確認乾淨）。合併成功後（fast-forward 或合併節點皆可）、清理 worktree 之前，在主 checkout 執行 `speclink review prepare <change> --base <記下的值> --from <worktree-path>`。失敗時回報並停止，不移除 worktree（快照還在來源）。

- fast-forward 時記下的值是變更第一個 commit 的父 commit；合併節點時它是第一個父 commit，從它比對到合併節點恰為變更帶進的內容（含解衝突）。
- 交棒段拿掉「主 checkout 補跑屬降級路徑」，改為：品質關卡已在 worktree 跑完→封存；還沒跑→可直接在主 checkout 跑，基準已承接；有未勾手動任務→跑品質關卡、做手動測試、再封存。

### apply-with-worktree 提交後先 rebase

apply-with-worktree 的收尾在提交之後、交棒之前新增一步：

1. 兩站都沒有工單（`speclink review show`、`speclink verify show` 都回報找不到）才進行；有工單代表品質關卡已在跑，不在迴圈中途 rebase。
2. 合併目標取主 checkout 目前的分支（主 checkout 路徑由 `git worktree list --porcelain` 取得）；是 `speclink/*` 或 detached 時略過並說明。
3. 記下目標的 commit，於 worktree 內 rebase 到該目標。衝突時 `git rebase --abort`、回報、略過重設基準（分支未變，舊基準仍有效）。
4. 成功後於 worktree 內執行 `speclink review prepare <change> --base <記下的值>`。

這樣品質關卡在 worktree 內蓋章時的內容就是合併後的內容；之後合併只是 fast-forward，章不過期。唯一仍會過期的情況：跑品質關卡的期間主分支又改到同一個檔案。

交棒改為兩條路徑：沒有未勾手動任務→在 worktree 內跑品質關卡、蓋章後補提交、再 worktree-merge；有未勾手動任務（`instructions apply` 的 `remaining - codeRemaining` 大於零），或想在蓋章前看到 CI 結果（worktree 分支只在本機、從不 push，CI 看不到它）→先 worktree-merge，再在主 checkout 跑品質關卡、做手動測試、封存。skill-routing 的交棒邊集兩列跟著改。

- 替代方案：把 rebase 放進三份品質技能的開頭——時機最好，但同一段 git 步驟要寫三份；放在 apply-with-worktree 收尾只寫一份，涵蓋 apply 期間主分支的前進。

### 與 review-rules-config 的語意對齊

review-rules-config 新增「接受 CRITICAL 必修項的逐條確認」，quality-skill 現寫「停下時的使用者裁示即明示授權」。對齊方式：

- quality 停頓時，「進入收尾補蓋」帶保留、或「挑著修」留下 CRITICAL 必修項時，在停頓處逐條列出 CRITICAL 原文、確認一次；之後呼叫審查站時明示「CRITICAL 已逐條確認」。
- review-skill 新增 Requirement「quality 時序轉交的 CRITICAL 確認」：收到這個轉交時，以它作為逐條確認，不再詢問。用新增 Requirement 而不是改寫 review-rules-config 的那一條——那條在本變更 propose 時還不在正式規格裡。
- quality 第 3 步的合併報告保留審查站「審查呈現標明套用的規則」那一行。

### 技能版號與生成物

五份技能範本（quality、review、verify、apply-worktree-post、worktree-merge）改完後，以 review-rules-config 封存後的 ASSET_VERSION 為底升一版（預期 v1.45.0 → v1.46.0），再生 golden（`UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden::`）與 assets.lock，先 `cargo build -p speclink-cli` 再 `speclink update` 再生本 repo 的技能檔。docs/workflow.md 與 docs/workflow.zh-TW.md 的 worktree 段落改成兩條路徑。

## Implementation Contract

**行為**

- 品質關卡修完一輪後，技能跑的是受影響範圍的測試，停頓報告列出跑了與略過的項目；專案沒有任何測試政策時才跑全量。
- `/speclink-quality` 每輪在同一則訊息派出三個子代理；工單先寫審查、再寫驗證；單獨跑一站時派法不變。
- 兩站簡報遵守查找紀律；驗證站先在鎖定差異裡找證據。
- `review|verify add-round` 拒絕本輪新出現、且帶行號或超出工單範圍的 finding 路徑（規格目錄與 `.speclink/` 下的路徑免除範圍檢查）；本機與 remote 模式都在寫入前由 CLI 拒絕。
- quality 停頓時標出會動到 artifact 的 finding；情況 2 的修補後跑 `speclink validate`，改過的段落進下一輪驗證站簡報。
- quality 接續時先跑測試門；複驗差異為空的站不記一輪。
- worktree-merge 合併後，主 checkout 有 initial 基準與承接的快照；在主 checkout 跑品質關卡不再要求 `--base`。
- apply-with-worktree 提交後，分支已 rebase 到主分支最新位置、基準指向新分岔點（無工單、無衝突時）。

**介面**

- `speclink review prepare <change> [--base <rev>] [--from <worktree-path>]`：不讀 stdin；`--from` 必須與 `--base` 同用；成功 exit 0、stdout 為空。
- host：change_diff 模組的 `prepare_at` 與 `carry_snapshots`；既有 `prepare` 簽名與行為不變。
- core：`quality::station` 新增純函式 `finding_path_violations`；`add_round` 簽名與行為不變。
- CLI：station 動詞的 add-round 本機臂與 remote 臂共用一個呼叫該純函式的檢查。
- Baseline、Snapshot、工單檔案格式皆不變。

**失敗模式**

- `--base` 解析不到 commit、不是 HEAD 的祖先：非零、零寫入，stderr 說明原因。
- 目標已有 baseline 且任一站工單未結：非零、零寫入，stderr 點名哪一站。
- `--from` 未搭配 `--base`、來源路徑不存在：非零、零寫入。
- 快照複製中途失敗：非零，已複製的快照留著、baseline 未寫；重跑可補完。
- add-round 違規：非零（invalid_argv）、工單零寫入，stderr 逐一點名違規路徑。
- worktree-merge 的 prepare 失敗：技能停止、不移除 worktree。
- apply-with-worktree 的 rebase 衝突：abort、回報、保留舊基準，交棒照常。

**驗收**

- crates/engine/speclink-core/src/quality/station/tests.rs：`finding_path_violations` 的帶行號、範圍外、早輪 Scope 內接受、規格目錄與 `.speclink/` 免除範圍檢查、上一輪逐字沿用（含 `(accepted)`）豁免、反斜線正規化。
- crates/host/speclink-host/src/change_diff.rs 的測試模組：`prepare_at` 寫 initial 基準、非祖先拒絕、未結工單拒絕改寫、`carry_snapshots` 複製兩站快照、略過已存在檔、來源無快照時成功、承接後 validation scope 可解析。
- crates/adapters/speclink-cli/tests/it/review_verbs.rs 與 verify_verbs.rs：`review prepare` 旗標組合與 exit code、add-round 違規訊息與工單位元級不變；remote_verb_parity.rs：remote 模式 add-round 違規時於送出前拒絕。
- golden 對照涵蓋五份技能的新內文；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 涵蓋 docs 中英對照。

**範圍**

- 範圍內：五份技能範本、station.rs 的路徑檢查純函式與 CLI 兩臂的呼叫點、change_diff 的兩個入口、CLI 旗標、九份正式規格、docs/workflow 兩份、版號與生成物。
- 範圍外：desktop、server 路由、工單／基準／快照的檔案格式、驗證站拆子代理、單站的空差異處理、舊 worktree 變更的自動補救。

## Risks / Trade-offs

- [golden 與 CLI 回歸對照] 五份技能內文變更會讓六份 golden snapshot 與 assets.lock 全部重生 → 以 `UPDATE_GOLDEN=1` 與 `UPDATE_ASSETS_LOCK` 再生後逐份檢視差異只含本變更的段落；CLI 測試新增旗標與拒絕訊息的對照，既有 `review prepare` 不帶旗標的測試維持原樣。
- [跨平台] `git rev-parse`、`git merge-base --is-ancestor`、`git rebase` 在 Windows／macOS／Linux 行為一致；finding 路徑比對前把 `\` 正規化為 `/`；來源 worktree 路徑走 `std::path`；快照檔名本來就避開冒號 → Windows CI 跑 change_diff 與 station 的新測試。
- [只跑受影響範圍會漏掉範圍外的回歸] → 停頓報告列出略過的範圍；蓋章前先讓 CI 跑綠的既有做法會擋下。
- [三個子代理同時派，harness 不支援] → 兩站既有退路：無法派子代理就依序自跑。
- [合法 finding 落在差異外] → 技能指引錨定到範圍內被呼叫的檔案；上一輪逐字沿用的行豁免，舊工單不會卡住。
- [rebase 改寫 worktree 分支] → 分支只在本機、從不 push，worktree-merge 本來就會 rebase；衝突即 abort，分支完整復原。
- [`--base` 被誤用] → 必須是 HEAD 的祖先；有未結工單時不准改寫；與 `review scope --base` 一樣是使用者明示的固定點。
- [與 review-rules-config 的先後] 它若再經 ingest 改動接受 CRITICAL 的語意 → 本變更以 depends_on 排在它之後，apply 前重讀它封存後的正式規格再調整對齊段落。
- [storage 解耦] 路徑檢查的規則是 core 純函式、只依賴工單內容與傳入的前綴，CLI 兩臂共用，任何 Store（本機、remote）行為一致；基準與快照維持 host 本機工作資料，不進 TeamStore——與現行分層相同，不朝也不背離解耦方向。

## Migration Plan

- 無資料遷移：基準、快照、工單格式皆不變。
- 既有專案執行 `speclink update` 取得新技能。
- 已在 worktree 內、以舊技能完成 apply 的變更：合併時沒有基準可承接，主 checkout 照舊需要一次人工 `--base`（變更第一個 commit 的父 commit）。
- 回退：還原技能範本與版號即回到舊流程；`review prepare` 的新旗標不被呼叫時沒有任何效果。
