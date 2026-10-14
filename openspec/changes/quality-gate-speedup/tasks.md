## 1. 開工前確認排序前提

- [x] 1.1 確認 review-rules-config 已封存，並以它封存後的狀態作為本變更的底稿（與 review-rules-config 的語意對齊、技能版號與生成物兩項決策的前提）：`speclink list --json` 不再列出 review-rules-config；`speclink show review-skill` 含「接受 CRITICAL 必修項的逐條確認」與「審查呈現標明套用的規則」兩條 Requirement；記下 crates/engine/speclink-core/src/workspace/init.rs 的 ASSET_VERSION 現值（預期 v1.45.0）與 crates/engine/speclink-core/assets/skills/review.md 第 5 步、第 9 步的現行文字。驗證：上述三個指令／讀檔結果符合預期；不符時停下回報，不開工 <!-- speclink-task:tsk_01M4WA6CEYCCZGTD8EEVDMQF3Y -->

## 2. add-round 的 finding 路徑檢查

- [x] 2.1 紅燈（add-round 的 finding 路徑檢查）：crates/engine/speclink-core/src/quality/station/tests.rs 新增 `finding_path_violations` 的測試——`src/a.rs:42` 與 `src/a.rs:42-50` 判為帶行號；本輪與既有各輪 Scope 都不含的程式碼路徑判為範圍外；Round 1 Scope 內的路徑於 Round 2 接受；`openspec/` 與 `.speclink/` 下的路徑免除範圍檢查但仍檢查行號；與上一輪逐字相同（含結尾 `(accepted)`）的行豁免；`src\a.rs` 正規化後比對。驗證：`cargo test -p speclink-core` 這批測試失敗（尚未實作） <!-- speclink-task:tsk_01M4WA6CEY0MBNT7QAPK1KK2DE -->
- [x] 2.2 綠燈：crates/engine/speclink-core/src/quality/station.rs 實作純函式 `finding_path_violations`，聯集沿用既有 `scope_union`，回傳每條違規的路徑與原因（帶行號／不在工單範圍內）；`add_round` 簽名與行為不變。驗證：`cargo test -p speclink-core` 全綠 <!-- speclink-task:tsk_01M4WA6CEYX73A4GVH5Z8HM9FB -->
- [x] 2.3 紅燈（finding 路徑落在工單範圍內、驗證 finding 路徑落在工單範圍內）：crates/adapters/speclink-cli/tests/it/review_verbs.rs 與 verify_verbs.rs 新增——帶行號與範圍外的 finding 讓 add-round 非零結束、stderr 逐一點名違規路徑、`review.md`／`verify.md` 位元級不變（verify 無工單時不建立）；錨定在 delta 規格檔的 finding 成功寫入；合法內容的 stdout 與既有輸出相同。crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs 新增 remote 模式違規時於送出前拒絕、server 端工單零寫入。驗證：`cargo test -p speclink-cli --test it -- review_verbs verify_verbs remote_verb_parity` 這批測試失敗 <!-- speclink-task:tsk_01M4WA6CEYPW06KYZAJ8A3G0VF -->
- [x] 2.4 綠燈：crates/adapters/speclink-cli/src/verbs/station.rs 的 add-round 本機臂（呼叫 core `add_round` 之前）與 remote 臂（以既有 station show 讀取工單後、送出之前）共用同一個檢查函式，以本機 Workspace 的 `spec_dir_name` 與 `.speclink` 提供免檢查前綴；違規時以 invalid_argv 類錯誤非零結束。驗證：`cargo test -p speclink-cli --test it -- review_verbs verify_verbs remote_verb_parity` 全綠 <!-- speclink-task:tsk_01M4WA6CEYV69FVN64VN6MDFX5 -->

## 3. Apply 基準以可信 commit 重設並承接快照

- [x] 3.1 紅燈（以可信 commit 重設 Apply baseline 並承接快照）：crates/host/speclink-host/src/change_diff.rs 的測試模組新增——`prepare_at` 以祖先 commit 寫入 confidence=initial、baseCommit 為完整 SHA 的 baseline；非祖先與解析不到的 rev 失敗且零寫入；目標已有 baseline 且傳入任一站工單未結時失敗且 baseline 位元級不變；`carry_snapshots` 把來源的 `snapshots`、`verify-snapshots` 兩個目錄逐檔複製、略過已存在的同名檔、來源無快照時成功且不建目錄、不複製來源 baseline.json；承接後對同一工單執行 validation scope 可 resolved；模擬第二個檔案寫入失敗後重跑可補完。驗證：`cargo test -p speclink-host` 這批測試失敗 <!-- speclink-task:tsk_01M4WA6CEY9VG24QWD48NHHC9P -->
- [x] 3.2 綠燈：change_diff.rs 實作 `prepare_at`（`git rev-parse --verify <rev>^{commit}`、`git merge-base --is-ancestor`、原子寫入 baseline）與 `carry_snapshots`（套用同一套 sidecar 路徑規則、逐檔暫存檔＋rename），寫入順序為「驗證 → 複製快照 → 最後寫 baseline」；既有 `prepare` 不變。驗證：`cargo test -p speclink-host` 全綠 <!-- speclink-task:tsk_01M4WA6CEY7Q000E96KFEAEA0R -->
- [x] 3.3 紅燈：crates/adapters/speclink-cli/tests/it/review_verbs.rs 新增 `speclink review prepare` 旗標測試——`--base <祖先>` 成功 exit 0、stdout 為空；`--from` 未搭配 `--base` 非零；`--base` 非祖先、解析不到非零且 stderr 說明原因；已有 baseline 且審查工單未結時非零、stderr 點名審查站；`--base M --from <來源>` 後 `speclink review scope --json` 不再回報 baseline 缺失；不帶旗標時「Apply 開始前記錄 host-local baseline」的既有測試原樣通過。驗證：`cargo test -p speclink-cli --test it -- review_verbs` 這批新測試失敗 <!-- speclink-task:tsk_01M4WA6CEYH77QED9HYZ6D2450 -->
- [x] 3.4 綠燈：crates/adapters/speclink-cli/src/verbs/station.rs 的 `ReviewCommands::Prepare` 新增 `--base <rev>` 與 `--from <worktree-path>`（`--from` 依附 `--base`），本機臂與 remote 臂都經 `run_review_prepare` 呼叫 host；CLI 依兩站工單狀態判斷是否「工單未結」後傳入 host。驗證：`cargo test -p speclink-cli --test it -- review_verbs remote_verb_parity` 全綠 <!-- speclink-task:tsk_01M4WA6CEYED1H01E914JNQPGR -->
- [x] 3.5 依 sharp-edges 稽核清單（`speclink instructions --skill audit`）檢查 `review prepare` 的新旗標：以 `-` 開頭的 rev 不得被 git 當成選項（拒絕或以 `--end-of-options` 隔開）、`--from` 指向不存在或非 checkout 的路徑時的錯誤訊息要指出正確用法、錯誤訊息不得洩漏多於路徑本身的資訊；發現的問題補測試並修正。驗證：review_verbs.rs 新增「rev 以 - 開頭被拒」測試，`cargo test -p speclink-cli --test it -- review_verbs` 全綠 <!-- speclink-task:tsk_01M4WA6CEY9HYMCMSPR9KAXMS8 -->

## 4. 審查與驗證技能範本

- [x] 4.1 crates/engine/speclink-core/assets/skills/review.md 落實四項決策：測試門改跑受影響範圍與修補動到 artifact 的三種情況（「修復迴圈的驗證門」三段判斷、建置照跑、報告列出跑了與略過的範圍；規格字面修正跑 `speclink validate` 並把改過段落附進下一輪簡報；意圖改變走範圍改變出口）；子代理簡報的查找紀律（「審查子代理簡報的查找紀律」的 25 KB 門檻、golden 只給路徑、禁止整份讀大檔與無目標搜尋、複驗上限 5 次、「沒查完就直說」、差異外呼叫端錨定到範圍內檔案且不帶行號）；三個子代理同一則訊息派出的一句（quality 時序中由編排方一起派出）；與 review-rules-config 的語意對齊（「quality 時序轉交的 CRITICAL 確認」：收到轉交不再詢問、轉交不完整時只補問未列者）。驗證：逐條對照本變更 review-skill delta 的 Requirement 與 Scenario 做內容檢視；`cargo test -p speclink-core --test it skill_verbization` 全綠 <!-- speclink-task:tsk_01M4WA6CEYFJV57T0CX869E5VE -->
- [x] 4.2 crates/engine/speclink-core/assets/skills/verify.md 落實：「驗證修復迴圈的驗證門」（取代第 13 步選項 1 的全量建置與測試一句，含三段判斷與 artifact 三種情況）；「驗證子代理簡報的查找紀律與證據順序」（簡報紀律同審查站；第 7、8 步改為先在鎖定差異裡找每條 Requirement 的證據、找不到才以具體名稱往外搜尋，移除「搜尋整個 codebase 的關鍵字」寫法）；quality 時序中由編排方一起派出的一句。驗證：逐條對照 verify-skill delta 做內容檢視；`cargo test -p speclink-core --test it skill_verbization` 全綠 <!-- speclink-task:tsk_01M4WA6CEY7SYA5KT5G2H04JPV -->

## 5. quality 技能範本

- [x] 5.1 crates/engine/speclink-core/assets/skills/quality.md 依「兩站時序的編排行為」改寫步驟：準備（審查站第 1–4 步、驗證站第 1–5 步）→ 三個子代理同一則訊息派出 → 依序完成兩站收尾並寫工單（先審查後驗證）→ 停頓；彙整報告保留審查站套用規則那一行、單獨標出會動到 artifact 的 finding；無法同時派出時依兩站退路依序自跑；修正輪先過兩站驗證門再同派複驗；與 review-rules-config 的語意對齊——「收尾補蓋」或「挑著修」留下 CRITICAL 時於停頓處逐條確認並於補蓋呼叫明示「CRITICAL 已逐條確認」。同時新增「中斷後接續」段（先跑驗證門、驗證 patch 無檔案的站不派子代理不記輪、兩站皆空直接回停頓、提醒接續前不得 commit）。「What this skill owns」改為承載時序與派出時機、仍不重述兩站內容。驗證：逐條對照 quality-skill delta 做內容檢視；`cargo test -p speclink-core --test it skill_verbization` 全綠 <!-- speclink-task:tsk_01M4WA6CEYHS8J5VTNW7PVNRF9 -->

## 6. worktree 技能範本

- [x] 6.1 crates/engine/speclink-core/assets/skills/worktree-merge.md 依「worktree-merge 技能的收尾流程指示」與「worktree-merge 合併後寫入基準」決策：第 3 步開始前記下主 checkout HEAD；合併成功後、第 5 步清理前執行 `speclink review prepare <change> --base <合併前 HEAD> --from <worktree-path>`，失敗即停止且不移除 worktree；第 6 步交棒、成功輸出區塊與 Next steps 依狀態分岔（已跑品質關卡→封存；未跑→主 checkout 直接跑；有未勾手動任務→跑品質關卡與手動任務後封存），移除「降級路徑」與「需要 worktree 的 Apply baseline」說法。驗證：逐條對照 worktree-merge-skill delta 的八個 Scenario 做內容檢視 <!-- speclink-task:tsk_01M4WA6CEYVJ2540BGG2DBG3QR -->
- [x] 6.2 crates/engine/speclink-core/assets/skills/apply-worktree-post.md 依「apply-with-worktree 技能的收尾指示」與「apply-with-worktree 提交後先 rebase」決策：W1 之後新增 rebase 與重設起點一步（兩站皆無工單才做；目標為主 checkout 所在分支，speclink/* 或 detached 略過；衝突即 `git rebase --abort`、回報、略過重設；成功後 `speclink review prepare <change> --base <記下的 commit>`）；W3 交棒、Wrap-up check 與 Next steps 改為兩條路徑並明列 review、verify、quality 三個入口，載明 worktree 分支只在本機、CI 看不到它；交棒邊集與 skill-routing「出口交棒由技能結尾承載」的兩列一致。驗證：逐條對照 worktree-apply-skill 與 skill-routing delta 做內容檢視 <!-- speclink-task:tsk_01M4WA6CEYAHM542Y0DR4PFAYY -->

## 7. 技能版號、生成物與文件

- [x] 7.1 技能版號與生成物：crates/engine/speclink-core/src/workspace/init.rs 的 ASSET_VERSION 以 1.1 記下的值為底升一版（預期 v1.45.0 → v1.46.0）；以 `UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden::` 再生 crates/engine/speclink-core/tests/golden/ 下的 claude、claude-worktree、codex、copilot、neutral-cli、neutral-tool-call 六份 snapshot，並更新 crates/engine/speclink-core/tests/golden/assets.lock；逐份檢視差異只含本變更改的五份技能段落與版號。驗證：`cargo test -p speclink-core --test it render_golden::` 全綠 <!-- speclink-task:tsk_01M4WA6CEYAWPX80NM7Y95ET5V -->
- [x] 7.2 先 `cargo build -p speclink-cli`，再以新 binary 執行 `speclink update` 再生本 repo 的技能檔（.claude/skills 與 .agents/skills 下的 speclink-review、speclink-verify、speclink-quality、speclink-apply-with-worktree、speclink-worktree-merge 等）。驗證：`target/debug/speclink --version` 的 engine 版號為新值；`git status` 列出的再生技能檔與預期一致，收尾提交時一併帶上 <!-- speclink-task:tsk_01M4WA6CEYPZ1W8TPXN6294VYG -->
- [x] 7.3 docs/workflow.md 與 docs/workflow.zh-TW.md 的 worktree 段落改為兩條路徑（無手動任務→worktree 內跑品質關卡再合併；有手動任務或要先看 CI→先合併再於主 checkout 跑），刪除「品質關卡只能在 worktree 內跑，因為起點在那裡」的說法，CLI 清單補上 `review prepare` 的 `--base`／`--from`；兩份文件段落一一對應。驗證：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠 <!-- speclink-task:tsk_01M4WA6CEYD2NE1YEEQCJZT4NM -->

## 8. 收尾驗收

- [x] 8.1 跑本變更觸及的全部 package 測試與 scripts 守門，確認沒有回歸。驗證：`cargo test -p speclink-core`、`cargo test -p speclink-host`、`cargo test -p speclink-cli --test it`、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠 <!-- speclink-task:tsk_01M4WA6CEYTYRMYQAS5CD5V2JZ -->
