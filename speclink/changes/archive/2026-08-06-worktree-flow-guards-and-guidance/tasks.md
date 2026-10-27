## 1. 封存防呆（引擎，TDD 先行）

- [x] 1.1 於 crates/speclink-cli/tests/it/archive_readiness_gate.rs 新增整合測試（先紅）：(a) 在分支 speclink/<name> 的 linked worktree 內執行封存 → exit code 非零、stderr 同時含 worktree 事實與 worktree-merge 指路、change 目錄仍在原位且無正典規格寫入與備份目錄；(b) .git 為檔案但分支為 feature/x → 封存成功（與主 checkout 同行為）；(c) 主 checkout 封存行為不變（沿用既有測試綠燈確認）。驗證：cargo test --test it 顯示新測試失敗、既有測試通過 <!-- speclink-task:tsk_01KZAHB6VB98WMYZXFNPW1W32R -->
- [x] 1.2 落實 change-lifecycle 需求「封存的 linked worktree 環境守門」：在 crates/speclink-core/src/archive.rs 的 archive() 入口（任何檔案效果之前）實作守門：先以 fs 判定 workspace root 的 .git 是否為檔案（目錄即短路、不 spawn git），是檔案時以 util::git 取 branch --show-current，具 speclink/ 前綴即回傳 Refusal（訊息含 worktree 事實與 worktree-merge 指路）；git 不可用、指令失敗或輸出空 → 放行。驗證：1.1 全部轉綠、既有 archive 相關測試（archive_merge_gate、archive_evidence_gate）維持綠燈 <!-- speclink-task:tsk_01KZAHB6VC9C40GVGY4HXRNZZA -->

## 2. 生成指令檔範本（init.rs）

- [x] 2.1 落實 workspace-tools 需求「marker 技能指引跟隨 worktree 政策」：於 crates/speclink-core/src/init.rs 的 instructions_body 加入政策閘內容：Workflow 段主流程線之下新增 worktree 流程線（claude／neutral 目標為 apply-with-worktree ⇄ ingest → (review? ∥ verify?) → worktree-merge → archive 於 main checkout；codex 目標沿既有分支慣例以 review? 取代並列站），Workflow bullet 清單新增品質站指引（品質站建議於 worktree 內完成、Apply baseline 在 worktree、封存僅在主 checkout、worktree 內封存會被引擎拒絕），兩者僅於 worktree 參數為 true 時輸出；同步提升 MARKER_VERSION 一個 minor 版位。驗證：render_golden 測試對 worktree 開啟變體顯示預期差異（此時 golden 未再生、紅燈屬預期） <!-- speclink-task:tsk_01KZAHB6VCVM712RCZNVKEPNN3 -->

## 3. 技能 asset 文案改向

- [x] 3.1 落實 worktree-merge-skill 需求「worktree-merge 技能的收尾流程指示」：crates/speclink-core/assets/skills/worktree-merge.md 的合併成功交棒段改為正典順序——下一步為主 checkout 封存（品質站建議已於 worktree 內完成）；補一句品質站未完成時仍得於主 checkout 補跑、惟無 Apply baseline 屬降級路徑。流程步驟、preflight 與守則清單零變動。驗證：內容審視對照 worktree-merge-skill delta 的「內文含清理與正典順序交棒指示」scenario <!-- speclink-task:tsk_01KZAHB6VCJW3JXYMEGKVXYKYZ -->
- [x] 3.2 落實 worktree-apply-skill 需求「apply-with-worktree 技能的收尾指示」：crates/speclink-core/assets/skills/apply-worktree-post.md 的收尾交棒自僅點名 worktree-merge 擴充為「建議先於 worktree 內執行品質站（review ∥ verify，由使用者判斷；蓋章寫入 meta 後提示補提交），再走 worktree-merge」；不合併、不移除 worktree 的停點文字不變。驗證：內容審視對照 worktree-apply-skill delta 的 scenario <!-- speclink-task:tsk_01KZAHB6VCBTHGCZFZJ9Q9MSH1 -->
- [x] 3.3 落實 archive-skill 需求「worktree 環境的技能敘述」：crates/speclink-core/assets/skills/archive.md 補一段 worktree 提示——封存於主 checkout 執行，linked worktree（speclink/ 分支）內封存會被引擎拒絕，先以 worktree-merge 收尾。驗證：內容審視對照 archive-skill delta 的 scenario <!-- speclink-task:tsk_01KZAHB6VCP9BT72GJGH3C11M8 -->

## 4. 三連動收斂（golden／assets.lock）

- [x] 4.1 再生 golden 快照與 assets.lock：crates/speclink-core/tests/golden/ 之 claude.snapshot.md、claude-worktree.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md 與 assets.lock 依 2.1／3.x 的內容變更再生。驗證：render_golden 與技能同步測試全綠；人工 diff 確認 worktree 政策關閉變體與現行輸出僅差 MARKER_VERSION、開啟變體僅多 worktree 指引內容 <!-- speclink-task:tsk_01KZAHB6VCADVCTXK2RA4TM8RG -->

## 5. 本 repo 生成物刷新與收尾驗證

- [x] 5.1 以 ./target/debug/speclink update 刷新本 repo 生成物：CLAUDE.md、AGENTS.md、.claude/skills/speclink-worktree-merge/SKILL.md、.claude/skills/speclink-apply-with-worktree/SKILL.md、.claude/skills/speclink-archive/SKILL.md。驗證：git diff 僅上述生成物；CLAUDE.md 的 Workflow 段含 worktree 流程線與品質站指引 bullet <!-- speclink-task:tsk_01KZAHB6VC5ZC9DPVM3NQ0SYS3 -->
- [x] 5.2 全量測試與手動驗收：cargo test --workspace 全綠；於本 repo 任一掛起的 speclink worktree 內執行 ./target/debug/speclink archive <該 change> 確認被拒且訊息指路 worktree-merge、change 目錄零變動。驗證：測試輸出與手動執行結果 <!-- speclink-task:tsk_01KZAHB6VCFFV3C4TAQGTQ7MK6 -->
