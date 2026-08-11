## 1. 引擎移除

- [x] 1.1 依 design D1 以 TDD 落實修訂後的「任務行的手動測試標記與解析」:crates/speclink-core/src/tasks.rs 移除 Task 的 parallel 旗標,前綴剝除迴圈保留對 `[P] ` 的剝離但不落旗標。行為結果:delta 的前綴解析 Example 表逐列成立([P] 行描述乾淨、無旗標;[M] 行為不變);無標記任務逐位元不變。驗證:cargo test -p speclink-core 之 tasks 單元測試綠燈(含 [P] 只剝不承載、[P][M] 混用案例)。 <!-- speclink-task:tsk_01KZND8ZPPS53KF056A3PREG4S -->
- [x] 1.2 依 design D2 以 TDD 落實修訂後的「任務 payload 的 manual 欄位與寫碼進度」與「動詞 --json 輸出形狀凍結」:crates/speclink-core/src/instructions.rs 的 TaskJson、crates/speclink-protocol/src/query.rs 的 TaskEntry、crates/speclink-server/src/routes.rs 的 task_entry 搬運、crates/speclink-cli/src/verbs/instructions.rs 的 remote 回程轉接四處移除 parallel 欄位;wire fixture(query.rs roundtrip、crates/speclink-cli/tests/it/remote_read_path.rs、crates/speclink-remote/tests/it/typed_client.rs、crates/speclink-cli/tests/it/manual_task_gates.rs)同步。行為結果:instructions apply --json 任務項欄位集合為 id/description/done/manual,本機與 remote 同形。驗證:cargo test -p speclink-core -p speclink-protocol -p speclink-cli -p speclink-remote 全綠。 <!-- speclink-task:tsk_01KZND8ZPP99WKRXY5C460GN0R -->

## 2. desktop 資料與 UI

- [x] 2.1 依 design D6 以 TDD 落實「變更清單的寫碼進度欄位」:apps/desktop/core/src/query.rs 的變更清單項增 codeTotal/codeComplete/codeRemaining(取自引擎任務雙組計數入口),packages/ui/src/adapter.ts 與 apps/desktop/src/adapter/tauriDataSource.ts 的清單項型別同步。行為結果:desktop 協定清單項帶三欄且既有欄位不變;CLI list --json 不含三欄(輸出逐位元不變)。驗證:cargo test -p speclink-desktop-core 綠燈;cargo test -p speclink-cli --test it 之 list 相關測試綠燈。 <!-- speclink-task:tsk_01KZND8ZPPV8MGM4EXHCQBBSCF -->
- [x] 2.2 依 design D3 以 TDD 落實修訂後的「UI 剝離 ID 註解並以 stable ID 操作」:packages/ui/src/tasks.ts 的任務行解析增行首標記剝離(`[M]`/`[P]`,順序不敏感、各至多一次)與 manual 旗標,`[P]` 剝離後無任何旗標;勾選與拖放寫回維持行級改寫、前綴原樣保留。行為結果:delta 的「標記前綴剝離且寫回保留」情境成立。驗證:pnpm -C packages/ui test 之 tasks 解析測試綠燈(先寫紅測試再實作)。 <!-- speclink-task:tsk_01KZND8ZPP60A43JX95PDAP4PM -->
- [x] 2.3 依 design D4 以 TDD 落實「任務列的手動測試徽章」:packages/ui/src/components/TaskList.tsx 對 manual 任務於該列行尾右對齊渲染「✋ 圖示+手動測試」次要色小 chip(圖示取 lucide 手部同義圖示),編號起始欄與無徽章列同位、長文字換行徽章錨定首行行尾;勾完文字劃線、徽章保留不劃線;packages/ui/src/i18n.tsx 增「手動測試」詞條(tw/en)。行為結果:delta 三個 Scenario(徽章呈現/勾完保留/舊 [P] 行無徽章)成立。驗證:pnpm -C packages/ui test 之 taskList 測試綠燈(斷言徽章存在、位置容器與編號欄結構、[P] 列無徽章)。 <!-- speclink-task:tsk_01KZND8ZPPDAYXMJNNJPYGPWSE -->
- [x] 2.4 依 design D5 以 TDD 落實「看板卡片的待手測標示」:packages/ui/src/components/ChangeCard.tsx 於 codeRemaining=0 且 remaining>0 時渲染「待手測」行內小章(沿審查標示家族樣式,tooltip「待手測·剩 N 項」),其他狀態與資料源缺欄位(remote)時無章且呈現逐位元不變;packages/ui/src/i18n.tsx 增「待手測」詞條(tw/en)。行為結果:delta 的浮現判定 Example 表逐列成立。驗證:pnpm -C packages/ui test 之卡片測試綠燈(四態各一案例)。 <!-- speclink-task:tsk_01KZND8ZPPQ7Z24K0SD7BD2C4F -->
- [x] 2.5 依修訂後的 design D4 以 TDD 把手動測試徽章從行尾移到描述上方(2026-08-11 實測回饋:行尾灰 chip 讀不出來、視線起點在描述左緣):packages/ui/src/components/TaskList.tsx 的 TaskRowBody 改以描述欄為 flex-col——徽章獨佔描述正上方一行、左緣與描述切齊,配色改取語意色票(非 muted 灰階);checkbox 與描述的既有欄位結構不動,無徽章列不留空行。行為結果:delta 修訂後的四個 Scenario(徽章呈現於描述上方且左緣切齊/長描述換行不動徽章/勾完保留/舊 [P] 行無徽章行)成立,且 DragOverlay 與 readOnly 兩條共用路徑呈現一致。驗證:pnpm -C packages/ui test 之 taskList 測試綠燈(既有徽章測試改斷言上方版型:徽章與描述同屬一個 flex-col 容器、徽章為其首子元素、無徽章列不多出容器;斷言配色 class 非 muted)。 <!-- speclink-task:tsk_01KZNGQKP4XQFV8N3T7M2WJDR9 -->

## 3. 指引與詞彙

- [x] 3.1 起草指引的翻譯保留規則改點名 `[M]`:crates/speclink-core/assets/schema/spec-driven/tasks.instruction.md 與 crates/speclink-core/assets/schema/spec-driven/fork.schema.yaml 中「`[P]` markers」改為「`[M]` markers」;asset 內文變動依三連動慣例收斂——跑 render_golden,紅燈即提升 crates/speclink-core/src/init.rs 的 MARKER_VERSION 並再生 golden 與 crates/speclink-core/tests/golden/assets.lock,綠燈即證免除。行為結果:speclink update 產出的指引檔點名 `[M]`、不再提及 `[P]`。驗證:cargo test -p speclink-core --test it 全綠。 <!-- speclink-task:tsk_01KZND8ZPP5SV5C265NGSYZ53R -->
- [x] 3.2 依 design D8 落實共用詞彙:openspec/LANGUAGE.md 增「手動測試」與「待手測」兩條目(各含 definition/avoid/why,avoid 涵蓋人工測試、手工測試、等待驗收等同義詞)。行為結果:兩詞條的定義與 UI 詞條文案一致。驗證:speclink language show 輸出含兩條目,人工核讀與 i18n 詞條對齊。 <!-- speclink-task:tsk_01KZND8ZPP9CS57YHKA3116Y13 -->

## 4. 整體驗收

- [x] 4.1 全套回歸與收尾:workspace 測試、packages/ui 測試與 lint 全綠;確認無因本 change 孤兒化的 imports、型別欄位或死碼(特別是 parallel 相關殘留)。驗證:cargo test 與 cargo clippy 與 pnpm -C packages/ui test 全綠;grep 全 repo 無 parallel 任務欄位殘留(平行 worktree 等無關詞除外)。 <!-- speclink-task:tsk_01KZND8ZPPGYJYS08GAN1JW1CV -->
- [x] [M] 4.2 手動驗證 desktop 呈現:啟動 desktop 開啟含 `[M]` 任務的變更,實際檢視 (a) 任務列徽章位於描述正上方獨立一行、左緣與描述切齊、編號縱向對齊不被破壞、長描述換行時徽章不動、配色在滿頁灰階文字中一眼可辨,(b) 勾選後文字劃線徽章保留,(c) 看板卡片於寫碼全完成時浮現「待手測」章、tooltip 載明剩餘項數、勾完全部任務後章消失。驗證:三項情境以實際操作確認並記錄結果。 <!-- speclink-task:tsk_01KZND8ZPPFWSP0AR6MA82A5SW -->
  - 實測留痕:2026-08-11 使用者以安裝版(徽章上移+sky 配色版,引擎 v1.19.10)自行實測後勾銷。
