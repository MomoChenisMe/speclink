# Verify — remote-skill-delivery

## Round 1

**Phase**: discovery
**Patch**: sha256:7585974016c91bee64485d9af5d46829fb30eaca7b639fd18bf2e3d310b2f994
**Scope**: apps/desktop/core/src/project.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/tests/it/remote_data.rs, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/assetNotice.test.tsx, apps/desktop/src/__tests__/assetPrompt.test.ts, apps/desktop/src/__tests__/helpers/assetFixtures.ts, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/adapter/workspace.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/components/AssetNoticeCard.tsx, apps/desktop/src/components/AssetNoticeDialog.tsx, apps/desktop/src/i18n/messages.ts, apps/desktop/src/store.ts, crates/adapters/speclink-cli/Cargo.toml, crates/adapters/speclink-cli/src/main.rs, crates/adapters/speclink-cli/src/remote_base.rs, crates/adapters/speclink-cli/src/skill_notice.rs, crates/adapters/speclink-cli/src/verbs/init.rs, crates/adapters/speclink-cli/tests/it/main.rs, crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs, crates/adapters/speclink-cli/tests/it/remote_skills.rs, crates/adapters/speclink-node/__test__/handle-request.spec.ts, crates/adapters/speclink-node/index.d.ts, crates/adapters/speclink-node/index.js, crates/adapters/speclink-node/src/host.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/init/tests.rs, crates/host/speclink-host/src/documents.rs, crates/host/speclink-host/src/http/mod.rs, crates/host/speclink-host/src/http/router.rs, crates/host/speclink-host/src/http/routes.rs, crates/host/speclink-host/src/http/skills.rs, crates/host/speclink-host/src/kit.rs, crates/host/speclink-host/tests/http_door.rs, crates/host/speclink-host/tests/station_wire.rs, crates/host/speclink-server/src/state.rs, crates/host/speclink-server/tests/it/api/write_guard.rs, crates/protocol/speclink-protocol/src/binding.rs, crates/protocol/speclink-protocol/src/lib.rs, crates/protocol/speclink-protocol/src/skills.rs, crates/protocol/speclink-remote/src/client.rs, crates/protocol/speclink-remote/src/lib.rs, crates/protocol/speclink-remote/src/skills.rs
- [SUGGESTION] crates/engine/speclink-core/src/workspace/init.rs — Correctness: `probe_served_skills` 第 1462 行的 `let missing = present.is_empty();` 沒有區分「Server 對這個工具本來就一支都不給」。例如宿主清單是 `['analyze']`（只給 Claude）而 checkout 有 codex，或清單只有 worktree 的兩支而政策關閉：這個工具永遠是「缺少」，Desktop 會一直顯示「安裝技能檔」，按了也不會消失，只能按「保留現狀」。實作照著規格字面「任一工具沒有任何技能檔為缺失」，所以不算偏離；建議改成 `present.is_empty() && !target.files.is_empty()` 並補一個測試，或在 desktop-app 的 delta 補一句定義這個情況。
- [SUGGESTION] apps/desktop/src/store.ts — Correctness: design D6 寫「探測的時機和本機專案相同（…收到變動事件）」，但 remote 分頁收到的變動事件只有 Server 推來的。`rearmWatch`（第 1413 行）對非本機分頁直接返回，checkout 的檔案沒有被監看。結果是使用者在終端機對這個 checkout 跑 `speclink update`、或手動改技能檔之後，提示卡要等切換分頁或下一個 Server 事件才更新。情境「外部更新自然消提示」只約束本機專案，不構成違反；建議在 design D6 明寫這個差異，或對 `checkoutRoot` 掛監看。
- [SUGGESTION] crates/adapters/speclink-cli/src/skill_notice.rs — Correctness: 規格寫「記錄檔無法讀寫時 SHALL NOT 提醒」，但第 47-48 行把任何讀取失敗（含權限不足）都當成「還沒有記錄」，只要之後寫得進去就會提醒。檔案內容壞掉時重建是刻意的（有測試 `a_record_that_is_not_readable_as_a_record_is_replaced`）；但檔案只能寫不能讀時，每次執行都會提醒，超過「一天最多一次」。建議只把 `NotFound` 與解析失敗當成空記錄，其他讀取錯誤回 `None`。
- [SUGGESTION] speclink/changes/remote-skill-delivery/specs/workspace-tools/spec.md — Coherence: 「CLI 的技能版本提醒」寫「remote 動詞握手之後…SHALL 印出」，沒有寫 `speclink update` 的例外。design D5 與測試 `update_prints_no_notice_and_does_not_use_up_the_day` 都把「update 自己不印、也不用掉當天的額度」定成行為。建議在 requirement 補一句，讓規格和實作對得上。
- [SUGGESTION] speclink/changes/remote-skill-delivery/design.md — Coherence: D4 寫四種對不上的檢查都「在 `SyncPlan::from_rendered` 裡」，實際上「缺少全文」是在 `crates/protocol/speclink-remote/src/skills.rs` 的 `rendered`（第 43-67 行）先擋掉。行為一樣是任何寫入之前、零寫入（`audit_an_answer_for_another_tool_or_without_text_writes_nothing` 有測）；建議把 D4 這一句改成實際的落點。
- [SUGGESTION] crates/adapters/speclink-cli/tests/it/remote_skills.rs — Correctness: requirement 列了四種退回本機範本的原因，測試涵蓋三種（沒有宣告、沒有憑證、連不上）。「握手被 Server 拒絕」（例如 binding 回 401 或 403）沒有測試；程式有處理，在 `crates/adapters/speclink-cli/src/verbs/init.rs` 第 280-283 行，任何 `connect` 的錯誤都退回。建議補一個假 Server 讓 `/binding` 回 403 的測試，斷言 exit code 0、stderr 有那一行、技能檔是本機範本。
- [SUGGESTION] apps/desktop/src-tauri/tests/it/remote_data.rs — Correctness: Rust 這一層有兩句規格沒有直接的測試。一是「握手沒有宣告技能時 `probe_assets` 回 `None`」（`apps/desktop/src-tauri/src/remote.rs` 第 1652-1654 行），目前只有前端用 mock 的 `null` 測到。二是「remote 專案的探測 SHALL NOT 下載技能全文」，`probe_assets` 傳 `contents = false` 讀得出來，但沒有測試斷言送出的請求不帶 `contents`。建議各補一個斷言。

## Round 2

**Phase**: validation
**Patch**: sha256:d75c4f40f5daf03f3a65918b2ab4b4f99c73be2227d5d2fff9bbafd7f5ef420e
**Scope**: apps/desktop/core/src/project.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/src/watch.rs, apps/desktop/src-tauri/tests/it/remote_data.rs, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/adapter/workspace.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/store.ts, crates/adapters/speclink-cli/src/skill_notice.rs, crates/adapters/speclink-cli/src/verbs/init.rs, crates/adapters/speclink-cli/tests/it/door.rs, crates/adapters/speclink-cli/tests/it/main.rs, crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs, crates/adapters/speclink-cli/tests/it/remote_skills.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/init/tests.rs, crates/protocol/speclink-remote/src/skills.rs

## Round 3

**Phase**: validation
**Patch**: sha256:8f372d5a479d3090b61d5ac099957597d626738a307163d9aeef263d90043a07
**Scope**: apps/desktop/core/src/project.rs, apps/desktop/src-tauri/src/watch.rs, crates/adapters/speclink-cli/src/skill_notice.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/init/tests.rs

## Round 4

**Phase**: validation
**Patch**: sha256:7497a70457f728f03076dd9cb2ce1bfaeccfda7465312399a3c03a8dc363fe5d
**Scope**: apps/desktop/src-tauri/src/watch.rs
