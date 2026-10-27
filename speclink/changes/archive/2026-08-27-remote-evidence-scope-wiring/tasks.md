## 1. 測試先行（先紅）

- [x] 1.1 crates/speclink-cli/tests/it/remote_verb_parity.rs：mock server 補 change evidence 端點回應；新增「remote task done 後 scope 自動解析」測試——mock evidence 帶 touched files，review scope 不帶任何手動旗標，斷言回 resolved payload、touched 認領等於 evidence 檔案集合、無 needsInput。以 `cargo test -p speclink-cli --test it` 確認新測試失敗（紅燈基準） <!-- speclink-task:tsk_01M111M4S2KQHSDAZ9AEGVX5V0 -->
- [x] 1.2 同檔新增三個場景測試：(a)「evidence 缺席時 needsInput 與 EmptyTouched 理由」（釘住既有 fail-closed，預期立即綠）；(b)「多 actor entries 的 touched 取聯集」；(c)「另一 active change 的 evidence 重疊認領觸發 other-claims 守門」，(b)(c) 先紅 <!-- speclink-task:tsk_01M111M4S2F5K0YJKEKWPSE514 -->
- [x] 1.3 改造既有 remote scope 測試群（remote_review_scope_uses_local_git_and_uploads_nothing、remote_review_scope_json_matches_fs_mode_field_for_field、remote_verify_scope_json_matches_fs_mode_field_for_field、remote_review_scope_offline_leaves_zero_sidecar_effects）：移除手塞本地 touched 檔的 write_touched 用法，改由 mock evidence 端點供應同一組檔案，跑測試確認因生產碼仍讀本地而轉紅 <!-- speclink-task:tsk_01M111M4S2PN1600XC1X9WH712 -->

## 2. 接線實作（轉綠）

- [x] 2.1 crates/speclink-cli/src/verbs/station.rs：remote 分支組 scope 請求前，以 typed client 的 change_evidence 讀本 change 的 evidence，touched 取全部 entries 聯集餵進 scope 請求；other claims 逐一讀其他 active change 的 evidence，無記錄視為零認領；任何 evidence 讀取失敗即非零結束、不寫 baseline 與 snapshot（design D1、D2、D4、D6）。驗證：1.1 至 1.3 全部測試轉綠 <!-- speclink-task:tsk_01M111M4S2YSBNKJ4DD8MKF207 -->
- [x] 2.2 跑 `cargo test -p speclink-cli --test it` 全量，既有 remote 與 fs 模式測試零回歸 <!-- speclink-task:tsk_01M111M4S25GF7EC1FXXGEKT3K -->

## 3. 正典對齊與收尾

- [x] 3.1 逐一核對 specs/change-diff-scope delta「remote workspace 使用同一 host resolver」requirement 的六個 scenario 與實作及測試對應；執行 speclink validate remote-evidence-scope-wiring 通過 <!-- speclink-task:tsk_01M111M4S2QM55WFMM8Y7XN2T7 -->
- [x] 3.2 確認 crates/speclink-core/assets/skills 零改動（守住 Non-Goal，不觸發 MARKER_VERSION 與 golden 與 assets.lock 連動）；以 git status 盤點僅預期檔案異動 <!-- speclink-task:tsk_01M111M4S29REXGFZSGKXJDGVQ -->
