# Review — remote-skill-delivery

## Round 1

**Phase**: discovery
**Patch**: sha256:7585974016c91bee64485d9af5d46829fb30eaca7b639fd18bf2e3d310b2f994
**Rules**: extend
**Rule**: CRITICAL 呼叫端不得以 className 覆蓋 @speclink/ui 元件變體的顏色、圓角與陰影，需要的外觀改由變體提供
**Rule**: WARNING 全系統共通的行為（例如返回導覽、快捷鍵、提示框）要在共用層實作一次，各頁面只接上共用實作；新增的頁面要接上既有的共用實作，不要在頁面裡各自重寫
**Rule**: WARNING 新增功能時，repo 已有相同用途的元件、編輯器或工具函式（例如 @speclink/ui 的元件、設定頁共用的程式碼編輯器）要直接沿用，不要另做一套平行實作
**Rule**: WARNING 修正 bug 的變更：同一種寫法或同一類元件在 repo 其他位置若有同樣的問題，要在這次一併修正，或在 tasks.md 寫明不修的理由
**Rule**: WARNING 同一種操作在不同頁面要用同一個元件與同一種變體呈現（例如按鈕、選單、分頁工具、空狀態），不要同一種操作在各頁長得不一樣
**Scope**: apps/desktop/core/src/project.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/tests/it/remote_data.rs, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/assetNotice.test.tsx, apps/desktop/src/__tests__/assetPrompt.test.ts, apps/desktop/src/__tests__/helpers/assetFixtures.ts, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/adapter/workspace.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/components/AssetNoticeCard.tsx, apps/desktop/src/components/AssetNoticeDialog.tsx, apps/desktop/src/i18n/messages.ts, apps/desktop/src/store.ts, crates/adapters/speclink-cli/Cargo.toml, crates/adapters/speclink-cli/src/main.rs, crates/adapters/speclink-cli/src/remote_base.rs, crates/adapters/speclink-cli/src/skill_notice.rs, crates/adapters/speclink-cli/src/verbs/init.rs, crates/adapters/speclink-cli/tests/it/main.rs, crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs, crates/adapters/speclink-cli/tests/it/remote_skills.rs, crates/adapters/speclink-node/__test__/handle-request.spec.ts, crates/adapters/speclink-node/index.d.ts, crates/adapters/speclink-node/index.js, crates/adapters/speclink-node/src/host.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/init/tests.rs, crates/host/speclink-host/src/documents.rs, crates/host/speclink-host/src/http/mod.rs, crates/host/speclink-host/src/http/router.rs, crates/host/speclink-host/src/http/routes.rs, crates/host/speclink-host/src/http/skills.rs, crates/host/speclink-host/src/kit.rs, crates/host/speclink-host/tests/http_door.rs, crates/host/speclink-host/tests/station_wire.rs, crates/host/speclink-server/src/state.rs, crates/host/speclink-server/tests/it/api/write_guard.rs, crates/protocol/speclink-protocol/src/binding.rs, crates/protocol/speclink-protocol/src/lib.rs, crates/protocol/speclink-protocol/src/skills.rs, crates/protocol/speclink-remote/src/client.rs, crates/protocol/speclink-remote/src/lib.rs, crates/protocol/speclink-remote/src/skills.rs
- [WARNING] crates/adapters/speclink-cli/tests/it/remote_skills.rs — Standards: 違反專案審查規則「新增功能時，repo 已有相同用途的元件、編輯器或工具函式（例如 @speclink/ui 的元件、設定頁共用的程式碼編輯器）要直接沿用，不要另做一套平行實作」：新寫的 `struct Door`／`fn door()`（HostKit 的 HTTP 門接上 MemoryStorePort 與 tiny_http）和同資料夾 `remote_read_verbs.rs` 既有的 `Door`／`door()` 幾乎逐行相同（約 55 行），只差沒有預先放文件、多留一個 `store` 欄位。應把既有的那份搬到共用位置（像 `fake_server.rs` 那樣 `pub(crate)`）給兩邊用。
- [SUGGESTION] crates/adapters/speclink-cli/src/verbs/init.rs — Standards: possible Duplicated Code：`skills_from_server` 的流程（`AppConfig::load` → `ToolSelection::resolve` → `targets_of` → 看 `capabilities.skills` → `fetch` → `rendered` → `SyncPlan::from_rendered` → `apply`）在 desktop 的 `served_skills`＋`update_remote_assets_at` 又組了一次；可把「取回並安裝」收成一個共用函式。
- [SUGGESTION] crates/engine/speclink-core/src/workspace/init.rs — Standards: possible Primitive Obsession：`RenderedTool.skills` 與 `ServedSkills.digests` 都是 `Vec<(String, String)>`，`probe_served_skills` 用 `RenderedTool { tool: s.tool.clone(), skills: s.digests.clone() }` 把摘要塞進原本放全文的欄位，之後 `target.files` 裡是摘要還是全文只能靠註解分辨；可給「技能名稱＋全文／摘要」各一個小型別。
- [SUGGESTION] crates/engine/speclink-core/src/workspace/init.rs — Standards: possible Middle Man／Mysterious Name：`is_newer_version` 只轉呼叫 `workspace_is_newer(version, than)`；`worktree_skills_on` 與 `worktree_skills_enabled` 名字近乎同義，看不出一個吃文字、一個吃路徑。可直接把原函式改名公開。
- [SUGGESTION] apps/desktop/src/store.ts — Standards: possible Mysterious Name：`AssetSubject.root` 在 remote 專案裝的是分頁的鍵（`root: key`），不是專案根，並一路傳進 `assetPrompt(probe, subject.root, …)`；改叫 `skipKey` 之類較貼切。
- [SUGGESTION] crates/protocol/speclink-remote/src/skills.rs — Standards: possible Middle Man：`fetch` 只有一行 `client.skills(&SkillsRequest { targets: targets.to_vec(), contents })`；呼叫端可直接用 `Client::skills`。
- [WARNING] crates/engine/speclink-core/src/workspace/init.rs — Correctness: `probe_served_skills` 用 `let missing = present.is_empty();` 判缺失，沒看 Server 對這個工具是否真的有給技能。Server 對某工具回零個技能時（host 的 `skills` 只列 `analyze` 這種 Claude 專用技能而 checkout 有 codex；或只列 worktree 技能而政策關閉），該工具永遠是 `missing`。桌面版按「安裝」後 `apply` 寫入零檔，重探仍是 missing，提示收不掉，只能按「保留現狀」。建議改成 `present.is_empty() && !target.files.is_empty()`。現有測試沒蓋到「Server 給零個技能」的情境。
- [SUGGESTION] crates/host/speclink-host/src/kit.rs — Correctness: `InvalidSkills::check` 只擋空清單與未知名稱（`names.is_empty() && !listed.is_empty()`）。註解說要避免「checkout 更新後失去全部技能」，但清單經工具或 worktree 政策過濾後對某工具變空的情況照樣通過。此時 `speclink update` 與桌面更新會經 `prune_orphan_skills(&skills_root, &[])` 刪光該工具所有 `speclink-*` 資料夾，仍回報成功。
- [SUGGESTION] apps/desktop/src-tauri/src/lib.rs — Correctness: `update_remote_assets` 的 `state.while_paused(ticket, update)` 把握手與 `POST /skills` 的網路時間也包進暫停。`pause` 會直接丟掉監看（`self.current = None`），暫停期間 `rearm` 一律不掛。使用者在等待時切到本機分頁，該分頁這段時間沒有監看，結束後重掛也不補刷新。remote 更新不會改資料夾名稱，其實不必暫停，或只包住寫入那段。
- [SUGGESTION] crates/adapters/speclink-cli/src/skill_notice.rs — Correctness: `after_handshake` 寫入 `speclink_config_dir()`。HOME（Windows 為 USERPROFILE）未設時它退回 `./speclink`，`create_dir_all(dir)` 會在 remote 專案的目前資料夾建出與規格資料夾同名的 `speclink/`，之後每個動詞都會出現「remote 與 speclink/ 並存」警告。另外 `std::fs::write` 不是原子寫入，平行跑多個動詞時同一天可能重複提醒，但不影響 stdout 與結束碼。

## Round 2

**Phase**: validation
**Patch**: sha256:fafaf70cf9a7d9c205297f8dcec1165768635332a8cdd288606c533ceeaeafca
**Scope**: apps/desktop/core/src/project.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/remote.rs, apps/desktop/src-tauri/src/watch.rs, apps/desktop/src-tauri/tests/it/remote_data.rs, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/adapter/workspace.ts, apps/desktop/src/assetPrompt.ts, apps/desktop/src/store.ts, crates/adapters/speclink-cli/src/skill_notice.rs, crates/adapters/speclink-cli/src/verbs/init.rs, crates/adapters/speclink-cli/tests/it/door.rs, crates/adapters/speclink-cli/tests/it/main.rs, crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs, crates/adapters/speclink-cli/tests/it/remote_skills.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/init/tests.rs, crates/protocol/speclink-remote/src/skills.rs
- [WARNING] crates/engine/speclink-core/src/workspace/init.rs — Correctness: `from_rendered` 先 `plan.targets = served;` 才呼叫 `plan.refuse_shared_skill_dirs()?`，而該檢查只看 `self.targets`。兩個工具解析到同一個技能資料夾（symlink、或描述子指到內建工具的資料夾），Server 對其中一個回空清單時，HEAD 會整筆拒絕，現在照常寫入，並把共用資料夾裡不在另一個工具清單內的 `speclink-*` 清掉，stdout 卻說那個工具的檔案「left as they are」。觸發條件窄（共用資料夾＋單一工具空清單），但結果正是 D9 要避免的靜默刪除。`probe_served_skills` 是用完整 targets 檢查，兩邊因此不一致。把檢查移到 partition 之前即可。
- [SUGGESTION] crates/adapters/speclink-cli/src/skill_notice.rs — Correctness: `Err(_) => return None` 把非 UTF-8 的記錄檔（`read_to_string` 回 `InvalidData`）也算成「讀不到」，之後永遠不提醒。HEAD 會當成空記錄並覆寫；delta spec 寫「內容不是記錄時視為沒有記錄」。
- [SUGGESTION] apps/desktop/src-tauri/src/watch.rs — Correctness: `normalized_for_match(event).strip_prefix(&root).is_ok_and(...)` 是「比不上就丟掉」。Windows 上 `root` 經 `canonicalize` 是 `\\?\` 形式，事件路徑連上一層都已不存在時退回原始路徑，前綴比不上。整個工具資料夾（例如 `.claude`）被刪掉時，深層事件全被丟掉，而 `.claude` 本身的事件 `concerns` 回 false（掛載當下技能目錄存在，不在 `awaited`），提示卡不會更新。這條是讀程式推論，未在 Windows 實測；macOS 不受影響。

## Round 3

**Phase**: validation
**Patch**: sha256:8f372d5a479d3090b61d5ac099957597d626738a307163d9aeef263d90043a07
**Scope**: apps/desktop/core/src/project.rs, apps/desktop/src-tauri/src/watch.rs, crates/adapters/speclink-cli/src/skill_notice.rs, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/src/workspace/init/tests.rs
- [SUGGESTION] apps/desktop/src-tauri/src/watch.rs — Standards: `relative_under` 的新註解寫「已經被刪掉的路徑無法正規化（`normalized_for_match` 會原樣退回）」，和正上方那個函式的實際行為不完全一致：`normalized_for_match` 會先試「正規化父目錄再接回檔名」，只有父目錄也不在時才原樣退回。新測試 `an_event_path_is_taken_relative_to_either_form_of_the_checkout_root` 的註解是同一句話。只是措辭過度概括，不影響行為；改成「父目錄也不在時」即可。

## Round 4

**Phase**: validation
**Patch**: sha256:7497a70457f728f03076dd9cb2ce1bfaeccfda7465312399a3c03a8dc363fe5d
**Scope**: apps/desktop/src-tauri/src/watch.rs
