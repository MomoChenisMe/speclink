## Summary

讓品質關卡（審查、驗證與兩站合跑的 `/speclink-quality`）每一輪少等約 40 分鐘的全量測試與一整段排隊，並消除會讓整輪白跑或重來的五種情況，包括 worktree 合併後每次被問 `--base`。

## Motivation

透過 AI 代理跑 SDD 的開發者在 workflow 的 `review? ∥ verify?`／`quality?` 階段，每一輪都要等很久。實測的時間分布（討論 quality-gate-speed）：

- CLI 動詞（`review scope`、`verify scope`、`add-round`）約 0.03 秒，不是瓶頸。
- 兩站技能要求每輪修完跑「完整建置與測試」，本專案一次 `cargo test --workspace` 約 40 分鐘；但專案自己的產出規則早已規定全量只由 CI 守門（`openspec/config.yaml` 的 rules.tasks）。
- `/speclink-quality` 先跑完整個審查站才開始驗證站，首輪多等 4–6.5 分鐘；先後順序只為蓋章不失效，與檢查無關。
- 子代理被叫去讀幾萬字的差異檔、做沒有目標的搜尋時，會 600 秒無進度被中止。
- finding 路徑寫錯（帶 `:行號`、或不在範圍內）要到下一輪才報錯，只能丟掉工單重跑首輪。
- 修補動到規格時沒有規則可循；session 在修補中途被關閉後，接續時可能空跑一輪被判「沒有進展」。
- worktree 流程的 Apply 基準只存在 worktree 裡、合併時被刪掉，回主 checkout 跑品質關卡每次都要人工提供 `--base`；交棒指引因此一律叫使用者在 worktree 內跑，不管有沒有手動任務。

本變更來自討論 quality-gate-speed；排在在途變更 review-rules-config 之後（兩者都改 review 技能與技能版號，規格層 Requirement 不重疊）。

## Proposed Solution

- **修完後的測試門改跑受影響範圍**（審查、驗證兩站）：先跑該修補所屬任務寫的驗證指令；任務沒涵蓋的檔案照專案寫好的測試政策；專案什麼都沒寫才跑全量。建置照跑；停頓報告寫明跑了哪些、略過哪些，略過的交 CI。
- **`/speclink-quality` 三個子代理同時派出**：每輪先做完兩站的準備步驟（各自鎖定範圍、審查站讀專案審查規則），再在同一則訊息派出審查 Standards 軸、審查 Correctness 軸、驗證檢查三個子代理；工單照舊先寫審查、再寫驗證，然後停頓。單獨跑一站時派法不變。
- **子代理簡報的查找紀律**（兩站）：首輪不設工具呼叫次數上限，但禁止整份讀大檔與沒有指定目標的遞迴搜尋；複驗輪工具呼叫上限 5 次；差異 25 KB 以內直接貼進簡報；自動產生的檔案只給路徑；簡報結尾「沒查完就直說，不要猜」。驗證站先在鎖定差異裡找每條 Requirement 的證據，找不到才往外搜尋具體名稱。
- **`add-round` 寫入前檢查 finding 路徑**（兩站；規則是 speclink-core 的純函式，由 CLI 的本機臂與 remote 臂共用）：路徑不得以 `:<數字>` 結尾，且必須落在工單各輪 `**Scope**:` 的聯集加上本輪 Scope 之內——規格目錄與 `.speclink/` 下的路徑免除範圍檢查（複驗本來就略過它們）；上一輪逐字沿用的行豁免。違者非零拒絕、工單零寫入，本機與 remote 模式都在寫入前拒絕。
- **修補動到 artifact 的三種情況**：只改程式碼／規格字面要修但意圖不變（主線直接改並跑 `speclink validate`，改過的段落放進下一輪驗證站簡報）／意圖或範圍改變（走兩站既有的「範圍改變」出口，先 ingest 再重跑）。quality 停頓時先單獨標出會動到 artifact 的 finding。
- **session 中途關閉後的接續**（quality）：再跑一次即可；派子代理前先跑測試門；複驗差異為空時不記一輪、直接回停頓；接續前不要 commit。
- **worktree 流程的 Apply 基準跨合併存活**：`speclink review prepare` 新增 `--base <rev>` 與 `--from <worktree-path>`；worktree-merge 合併前記下主分支 HEAD，合併成功後在主 checkout 以該值寫入基準並承接兩站快照；apply-with-worktree 提交後先 rebase 到主分支最新位置並把基準改到新分岔點；交棒指引依有無未勾手動任務、是否想在蓋章前看到 CI 結果，建議「worktree 內跑完再合併」或「先合併再在主 checkout 跑」。
- **與 review-rules-config 的語意對齊**：quality 停頓處留下 CRITICAL 必修項時逐條列出確認一次，轉交審查站當作確認、審查站不再問第二次；合併報告保留審查站「標明套用的規則」那一行。

### CLI 變更

`speclink review prepare <change> [--base <rev>] [--from <worktree-path>]`，不讀 stdin：

- 不帶旗標：行為與現在完全相同（Apply 技能的呼叫方式）。
- `--base <rev>`：以該 commit 寫一份 confidence 為 initial 的 Apply 基準（baseCommit 為完整 SHA、dirtyFilesAtStart 為當下的 dirty 路徑）。rev 解析不到 commit、或不是 HEAD 的祖先時非零結束且零寫入；目標已有基準且任一站處於審查中／驗證中（工單未結）時非零結束且零寫入——快照綁著舊基準。成功 exit 0、stdout 為空。
- `--from <worktree-path>`：必須與 `--base` 同用；把該 worktree 內這個 change 的兩站快照搬到目前 checkout。來源沒有快照時照常成功（品質關卡還沒跑過）；來源路徑不存在時非零結束且零寫入。
- remote 模式：基準與快照本來就是 host 本機資料，行為與本機相同。

`speclink review add-round` 與 `speclink verify add-round`：stdin 格式不變；新增「finding 路徑帶行號、或超出工單範圍（規格目錄與 `.speclink/` 除外）即拒絕」的檢查，exit code 非零、stderr 點名違規的路徑；remote 模式由 CLI 先讀工單再檢查，違規時不送出。

### 相容性影響

- `review prepare` 不帶旗標的輸出與行為不變；新旗標只增不改。
- `add-round` 以前接受、但會在下一輪讓 scope 解析失敗的路徑，現在寫入時就被拒絕；合法路徑的行為不變。人眼與 `--json` 輸出不變。
- 審查、驗證、quality、apply-with-worktree、worktree-merge 五份技能內文變更：引擎的技能版號（ASSET_VERSION）以 review-rules-config 封存後的版號為底再升一版（預期 v1.45.0 → v1.46.0），golden 對照同批更新；影響所有已啟用工具（claude、codex、copilot）的技能檔；既有專案執行 `speclink update` 即取得新版。
- 已在 worktree 裡、用舊技能完成 apply 的變更：合併時仍沒有基準可承接，主 checkout 照舊需要人工 `--base`；新版合併技能之後的變更才會自動承接。

## Alternatives Considered

- 拿掉每輪停頓、設輪數上限：先前討論 quality-skill-pause-and-ui-polish、code-review-convergence-boundary 已否決。
- 子代理改用較小模型：複驗輪本來就只要 1–1.5 分鐘，省得少還可能漏問題。
- 首輪固定 15 次工具呼叫上限：最近 25 個封存變更的 delta 規格平均 45 個 Scenario，驗證站查不完；卡住的真因是讀大檔與無目標搜尋。
- 審查只查和規格有關的差異段落：順手改的共用函式、測試工具會變成沒有任何一站在看。
- 品質迴圈內一律走完整 `/speclink-ingest`：太重，且可能新增或退回任務，讓兩站的寫碼任務守門拒收。
- 把 worktree 的 baseline.json 原樣複製到主 checkout、或開工時把基準寫進 commit 的 metadata：基準是 rebase 前的舊分岔點，會混入別人的修改。
- 拆成多個變更：每多一個變更就多跑一次品質關卡。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `quality-skill`: 三個子代理同一則訊息派出；停頓時標出會動到 artifact 的 finding；CRITICAL 必修項在停頓處逐條確認並轉交審查站；合併報告保留審查站套用規則那一行；新增 session 中途關閉後的接續規則。
- `review-skill`: 修復迴圈的驗證門改跑受影響範圍並納入修補動到 artifact 的三種情況；新增子代理簡報的查找紀律；新增 quality 時序轉交的 CRITICAL 確認。
- `verify-skill`: 新增驗證修復迴圈的驗證門（同審查站的受影響範圍與 artifact 三種情況）；新增子代理簡報的查找紀律與「先查差異」的證據順序。
- `review-station`: 新增 add-round 寫入時的 finding 路徑檢查。
- `verify-station`: 新增 add-round 寫入時的 finding 路徑檢查。
- `change-diff-scope`: `review prepare` 可帶 `--base`／`--from`；新增以可信 commit 重設 Apply 基準並承接快照。
- `worktree-merge-skill`: 合併前記下主分支 HEAD、合併成功後在主 checkout 寫入基準並承接快照，移除「降級路徑」說法。
- `worktree-apply-skill`: 提交後先 rebase 並把基準改到新分岔點；交棒依手動任務與 CI 需求分兩條路徑建議。
- `skill-routing`: 交棒句邊集中 apply-with-worktree 與 worktree-merge 兩列改為分岔建議。

## Impact

- Affected specs: quality-skill、review-skill、verify-skill、review-station、verify-station、change-diff-scope、worktree-merge-skill、worktree-apply-skill、skill-routing
- Affected crates／apps：speclink-core（五份技能範本、技能版號、finding 路徑檢查的純函式）、speclink-host（Apply 基準的指定 commit 寫入與快照承接）、speclink-cli（`review prepare` 旗標、add-round 寫入前的路徑檢查）
- 生成的技能：speclink-review、speclink-verify、speclink-quality、speclink-apply-with-worktree、speclink-worktree-merge，影響 claude、codex、copilot 三種工具；本 repo 自身的技能檔由 `speclink update` 再生
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/assets/skills/quality.md
    - crates/engine/speclink-core/assets/skills/review.md
    - crates/engine/speclink-core/assets/skills/verify.md
    - crates/engine/speclink-core/assets/skills/apply-worktree-post.md
    - crates/engine/speclink-core/assets/skills/worktree-merge.md
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/src/quality/station.rs
    - crates/engine/speclink-core/src/quality/station/tests.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/host/speclink-host/src/change_diff.rs
    - crates/adapters/speclink-cli/src/verbs/station.rs
    - crates/adapters/speclink-cli/tests/it/review_verbs.rs
    - crates/adapters/speclink-cli/tests/it/verify_verbs.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - docs/workflow.md
    - docs/workflow.zh-TW.md
  - Removed: (none)
