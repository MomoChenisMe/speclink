## 1. 引擎:反向動詞與零痕跡守門(speclink-core)

對應 design 決策 D1 反向動詞落在 speclink-core 的 inprogress 模組,守門在引擎函式內;以及 D2 守門與冪等語意(閉集,逐條可測)。

- [x] 1.1 撰寫規格需求「in-progress 標記可自 change meta 移除(零工作痕跡守門)」的引擎單元測試:放行時移除 started_at/started_by/started_with 三行且其餘內容逐字保留(不重序列化);已勾任務 > 0 拒絕;touched v1 清單非空拒絕;v2 entries 非空拒絕;證據為兩清單聯集去重;未知 change 報錯;未開工冪等回報未移除且零寫入;meta 損毀 fail-closed 不動檔案。檔案:crates/speclink-core/src/inprogress.rs(#[cfg(test)] 模組)。驗證:cargo test -p speclink-core inprogress 先紅。 <!-- speclink-task:tsk_01KYV6BGZ7EJKJ9RHH71PS1ZPT -->
- [x] 1.2 實作 remove 函式與守門:收 store、Workspace 與 change 名;以 tasks.md 勾選數(缺檔視為 0)與 TouchedRecord 兩清單判零痕跡;拒絕時回結構化證據(已勾任務數、檔案清單);放行時行過濾寫回 meta。檔案:crates/speclink-core/src/inprogress.rs。驗證:1.1 測試全綠,既有 add 測試不變。 <!-- speclink-task:tsk_01KYV6BGZ77FYN3KRV025DJBAF -->
- [x] 1.3 撰寫 command 層測試:Command::InProgressRemove 放行時 outcome 記錄實際移除並發 DomainEvent::ChangeInProgressRemoved;冪等未移除不發事件;守門拒絕走 CommandError 且證據隨錯誤可取。檔案:crates/speclink-core/src/command/mod.rs(#[cfg(test)] 模組)。驗證:cargo test -p speclink-core command 先紅。 <!-- speclink-task:tsk_01KYV6BGZ77RR8AMX9354RYN2X -->
- [x] 1.4 實作 Command::InProgressRemove 變體、outcome 與事件接線。檔案:crates/speclink-core/src/command/mod.rs。驗證:1.3 測試全綠。 <!-- speclink-task:tsk_01KYV6BGZ7NRYAZ5Z4YKMMTD9M -->

## 2. CLI 本地路徑

對應 design 決策 D3 Command 面:新 Command 變體、拒絕以錯誤回報、成功發事件。

- [x] 2.1 撰寫「in-progress 標記可自 change meta 移除(零工作痕跡守門)」的 CLI 整合測試:speclink in-progress remove 零痕跡成功 exit 0 且 stdout 印移除確認;未開工冪等 exit 0;已勾任務/touched 擋下 exit 非 0 且 stderr 含已勾任務數、檔案清單與出路說明;未知 change exit 非 0 報找不到;speclink in-progress add 的既有輸出與 exit code 完全不變。檔案:crates/speclink-cli/tests/in_progress_remove.rs(新檔)。驗證:cargo test -p speclink-cli 先紅。 <!-- speclink-task:tsk_01KYV6BGZ7S30R8CG2M15YF3J7 -->
- [x] 2.2 實作 in-progress 子指令群補 Remove 與本地 dispatch。檔案:crates/speclink-cli/src/main.rs、crates/speclink-cli/src/commands.rs。驗證:2.1 測試全綠,既有 golden 與 CLI 測試不變。 <!-- speclink-task:tsk_01KYV6BGZ7PNAJBYVQX6DY98AG -->

## 3. server 端點與 wire 形狀

對應 design 決策 D4 HTTP 形狀:DELETE /changes/{name}/in-progress。

- [x] 3.1 撰寫規格需求「in-progress 標記移除端點與加入端點成鏡像」的路由測試:DELETE /changes/{name}/in-progress 零痕跡 200 Ack 且 outbox 出現退回事件;未開工 200 且不 commit 不發事件;有痕跡 409 且 error payload 斷言 "checkedTasks"(數字)與 "touchedFiles"(字串陣列)camelCase 欄位存在與型別;未知 change 404;既有 POST 端點測試不變。檔案:crates/speclink-server/src/routes.rs(測試)或 crates/speclink-server/tests/ 既有整合測試檔。驗證:cargo test -p speclink-server 先紅。 <!-- speclink-task:tsk_01KYV6BGZ794ND6TFE97NRRFE1 -->
- [x] 3.2 實作 DELETE 路由與 409 證據載荷 wire 形狀(serde camelCase rename,欄位只增不改)。檔案:crates/speclink-server/src/routes.rs、crates/speclink-protocol/src/command.rs。驗證:3.1 測試全綠。 <!-- speclink-task:tsk_01KYV6BGZ7K02951Z9A59Q60AG -->

## 4. remote 客戶端與 CLI remote 路徑

- [x] 4.1 typed client 補 in-progress 移除方法(409 證據反序列化為結構化錯誤),CLI 於 remote workspace 對 speclink in-progress remove 走 DELETE 端點且輸出與本地一致——先補 crates/speclink-remote/tests/typed_client.rs 與 crates/speclink-cli/tests/remote_write_path.rs 測試再實作。檔案:crates/speclink-remote/src/client.rs、crates/speclink-cli/src/remote_commands.rs。驗證:cargo test -p speclink-remote -p speclink-cli 全綠。 <!-- speclink-task:tsk_01KYV6BGZ7GSXANA33140CQYPW -->

## 5. desktop bridge(本地與 remote 雙模式)

對應 design 決策 D5 desktop:按鈕直呼引擎、確認後執行、被擋開證據對話框 的 bridge 分層。

- [x] 5.1 本地 bridge:先寫 apps/desktop/core 測試(退回成功回傳、守門拒絕透傳含 checkedTasks/touchedFiles 的結構化錯誤 JSON),再實作 desktop core 函式與 Tauri command 單行委派註冊。檔案:apps/desktop/core/src/manage.rs、apps/desktop/src-tauri/src/lib.rs。驗證:desktop core 的 cargo test 全綠(不依賴 Tauri)。 <!-- speclink-task:tsk_01KYV6BGZ76PJ3SPNA9BJDXAT3 -->
- [x] 5.2 remote bridge:實作 remote command 打 DELETE 端點,409 證據轉為與本地同形狀的結構化錯誤(前端對話框單一消費入口)。檔案:apps/desktop/src-tauri/src/remote.rs。驗證:cargo build 通過,錯誤形狀與 5.1 一致(以 desktop core 測試斷言共用轉譯)。 <!-- speclink-task:tsk_01KYV6BGZ7JBMN9DTRZEZ31HZ2 -->

## 6. UI:退回動作、守門對話框、討論抽屜封存

對應 design 決策 D6 討論抽屜補封存動詞(順帶修復)。

- [x] 6.1 撰寫規格需求「進行中變更可自看板退回提案中」的 UI 測試:退回動作僅於派生進行中的 ChangeCard 與 RichDetailDrawer 出現;點擊先出確認;守門拒絕時 RevertBlockedDialog 渲染已勾任務數與 touched 檔案清單且無任何清理/強制按鈕;SpeclinkDataSource 新方法 revertChangeToProposed 在兩個 adapter 的呼叫對應(tauri command 與 remote command)。檔案:packages/ui/src/__tests__/、apps/desktop/src/__tests__/。驗證:npm test -w packages/ui -w apps/desktop 先紅。 <!-- speclink-task:tsk_01KYV6BGZ7P5AMMZSCHRBZG0P0 -->
- [x] 6.2 實作退回動作全鏈:adapter 介面新方法與雙 adapter 實作、ChangeCard 與 RichDetailDrawer 動作(樣式沿討論卡封存按鈕)、確認流程、RevertBlockedDialog、i18n 文案「退回提案中」、App.tsx 接線與成功後重載(卡片依派生回提案中欄,不手動搬卡)。檔案:packages/ui/src/adapter.ts、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/RevertBlockedDialog.tsx(新檔)、packages/ui/src/i18n.tsx、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/App.tsx。驗證:6.1 測試全綠。 <!-- speclink-task:tsk_01KYV6BGZ7ZCC7836XRC7J64GK -->
- [x] 6.3 討論抽屜補封存動詞(規格需求「討論抽屜檢視與轉出變更」的抽屜內封存場景):先寫測試(concluded 且未封存才出現;走與討論卡同一確認流程與 onArchiveDiscussion 呼叫),再實作抽屜動作與 App.tsx 接線。檔案:packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/__tests__/、apps/desktop/src/App.tsx。驗證:npm test -w packages/ui 全綠。 <!-- speclink-task:tsk_01KYV6BGZ75QEX27TS1KPSQZTD -->

## 7. 技能與詞彙

對應 design 決策 D7 詞彙與技能。

- [x] 7.1 apply 技能正典補「開錯工怎麼退」小節:指認 speclink in-progress remove、守門被擋的兩條出路(取消勾選重試;touched 請 agent 判斷),並同步再生 claude 與 codex 生成物。檔案:crates/speclink-core/assets/skills/apply.md、.claude/skills/speclink-apply/SKILL.md。驗證:技能同步的 golden 測試通過(cargo test 對應 skill 正典案例)。 <!-- speclink-task:tsk_01KYV6BGZ7PH20A5W225VNS5AR -->
- [x] 7.2 openspec/LANGUAGE.md 立「退回提案中」詞條(definition/avoid:撤回開工、取消開工、unstart(中文散文)/why:動詞直說結果、與看板欄名呼應)。檔案:openspec/LANGUAGE.md。驗證:speclink language show 列出新詞條。 <!-- speclink-task:tsk_01KYV6BGZ7P9HBMK6E8DYA7R5C -->

## 8. 收尾驗證

- [x] 8.1 全量驗證與真機演練:cargo test 全 workspace 與 npm test 全綠;desktop 真機(本地模式)演練誤開工→退回→卡片回提案中欄、有痕跡→守門對話框列證據;remote 模式同流程一致;speclink in-progress add 與既有 golden 迴歸全數不變。驗證:上述觀察逐項確認。 <!-- speclink-task:tsk_01KYV6BGZ7RCPGFNMNEF55GZG2 -->
