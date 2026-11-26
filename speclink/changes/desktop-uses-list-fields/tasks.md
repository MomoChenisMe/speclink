## 1. 資料：desktop-core 與 Tauri 殼

- [ ] 1.1 本機清單帶階段與識別（design D1、D2）：依需求「桌面清單與單一變更資料帶階段、建立者識別、就緒日與修改時間」，先在 apps/desktop/core/src/query.rs 的測試寫失敗的斷言，再讓 `list_changes_at` 用引擎的階段函式疊上 `stage`、從 metadata 疊上 `createdById`。斷言 payload 結構：`stage` 是 camelCase 鍵、值為 `proposed`／`in-progress`／`ready` 之一；`createdById` 沒有識別時為 null、有時為字串；metadata 損壞的項目沒有 `stage`；討論清單項帶出引擎給的 `modifiedAt` 與 `createdById`；其餘鍵不變。驗證：`cargo test -p speclink-desktop-core query` 通過。 <!-- speclink-task:tsk_01M8AN1GTHXCXMSEVT86879RWD -->
- [ ] 1.2 本機單一變更資料帶就緒日（design D4）：先在 apps/desktop/core/src/manage.rs 的測試寫失敗的斷言，再讓 `change_meta_at` 帶 `readyAt` 與 `createdById`——`readyAt` 用引擎的顯示規則，只在目前至少一個任務且全部勾完、而且 metadata 記有就緒時間時有值（日期）；取消勾選一個任務後再取沒有值。驗證：`cargo test -p speclink-desktop-core manage` 通過，既有 `change_meta` 測試不改斷言。 <!-- speclink-task:tsk_01M8AN1GTH6KZ8X5ZZ6HRVNVSK -->
- [ ] 1.3 殼的階段判定改用引擎的來源（design D1）：依需求「看板欄位以清單帶的階段為準」，先在 apps/desktop/src-tauri/tests/it/remote_data.rs 寫失敗的測試（回應帶 `stage` 為 `ready` 而任務 1/3 時落在已就緒欄；`stage` 是不認得的值時照推導），再把 apps/desktop/src-tauri/src/remote.rs 的 `change_stage` 改成有 `stage` 就用 `Stage::parse` 的結果、沒有時呼叫引擎的階段函式，移除自己寫的優先序。驗證：`cargo test -p speclink-desktop --test it remote_data` 通過，既有三個階段的斷言不改。 <!-- speclink-task:tsk_01M8AN1GTHGTVB07MSH02CDDBQ -->

## 2. packages/ui：階段與建立者篩選

- [ ] 2.1 階段以清單的值為準（design D1）：先在 packages/ui/src/__tests__/stage.test.ts 加 design Implementation Contract「階段」表的四列（失敗），再在 packages/ui/src/adapter.ts 的 `ChangeItem` 加選填的 `stage`、`createdById`、`modifiedAt`，並讓 packages/ui/src/stage.ts 的 `changeStage` 在 `stage` 是三個值之一時直接回它、否則照既有規則算。驗證：`npm test -w @speclink/ui -- stage` 通過，既有的階段測試不改斷言。 <!-- speclink-task:tsk_01M8AN1GTHSXB3K19XZS0NKVCH -->
- [ ] 2.2 建立者的歸屬規則（design D2）：依需求「看板搜尋過濾卡片」的新情境，先在 packages/ui/src/__tests__/search.test.ts 加「卡片的歸屬與選項」Example 的四列與「本機專案的建立者篩選維持名稱比對」（失敗），再在 packages/ui/src/search.ts 加一個純函式——輸入變更卡與討論卡，輸出每張卡的歸屬鍵與選單選項（有識別依識別；沒有識別而名稱只對應一個識別時併入；標籤取最先出現的名稱；標籤相同時接識別的最後 4 個字元），`matchesFilters` 改成比對歸屬鍵；`DiscussionItem` 加選填的 `createdById` 與 `modifiedAt`。驗證：`npm test -w @speclink/ui -- search` 通過，既有的篩選測試不改斷言。 <!-- speclink-task:tsk_01M8AN1GTH5BST5Z9C5HFJNGHZ -->
- [ ] 2.3 篩選面板接上歸屬規則（design D2）：先在 packages/ui/src/__tests__/kanban.test.tsx 加一個測試（遠端資料的三張卡 A、B、C：選單只有一個「Ann」，選定後三張都留下、開關鈕的啟用計數為 1），再讓 packages/ui/src/components/KanbanBoard.tsx 的建立者選單用 2.2 的選項，選定值存歸屬鍵。驗證：`npm test -w @speclink/ui -- kanban` 與 `npm test -w @speclink/ui -- boardSearchBar` 通過。 <!-- speclink-task:tsk_01M8AN1GTHDCJXVFM01S2ZZR1Z -->

## 3. packages/ui：卡片與詳情頁

- [ ] 3.1 卡片的最後更新（design D3）：依需求「看板卡片顯示最後更新」，先在 packages/ui/src/__tests__/kanban.test.tsx 與 packages/ui/src/__tests__/discussionColumn.test.tsx 寫失敗的測試（`modifiedAt` 為今天顯示「今天」、3 天前顯示「3 天前」、缺席或解析不出來時沒有這一段；提示含「最後更新」；拖曳浮起的卡片沒有這一段），再在 packages/ui/src/components/ChangeCard.tsx 與 packages/ui/src/components/DiscussionColumn.tsx 的 meta 列尾端加這一段，沿用 `relativeDays`；packages/ui/src/i18n.tsx 加「最後更新」提示的詞條（繁中、英文、日文）。驗證：`npm test -w @speclink/ui -- kanban` 與 `npm test -w @speclink/ui -- discussionColumn` 通過。 <!-- speclink-task:tsk_01M8AN1GTHSV9BTGTQ5ZHASY3D -->
- [ ] 3.2 出身行的就緒日（design D4）：依需求「變更詳情頁的出身行顯示就緒日」，先在 packages/ui/src/__tests__/changeDetailPage.test.tsx 寫失敗的測試（帶 `readyAt` 時就緒段緊接在開工段之後；缺席時沒有這一段且其他段不變；沒有開工戳記但有 `readyAt` 時接在建立相對時間之後），再在 packages/ui/src/adapter.ts 的 `ChangeMetaInfo` 加選填的 `readyAt` 與 `createdById`、packages/ui/src/components/ChangeDetailPage.tsx 的出身行加這一段、packages/ui/src/i18n.tsx 加「就緒 {date}」詞條（三種語言）。驗證：`npm test -w @speclink/ui -- changeDetailPage` 通過。 <!-- speclink-task:tsk_01M8AN1GTH42027HBZCDYN5RA0 -->

## 4. 收尾

- [ ] 4.1 受影響面的全部測試：確認 CLI 的輸出與 render 的 golden 沒有被這個變更改到，三個受影響面都通過。驗證：`cargo test -p speclink-desktop-core`、`cargo test -p speclink-desktop --test it remote_data`、`npm test -w @speclink/ui`、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過；crates/engine/speclink-core/tests/golden 沒有任何檔案被改。 <!-- speclink-task:tsk_01M8AN1GTH9YP9ZHEHAGX9GPK7 -->
- [ ] [M] 4.2 實機確認四項畫面：用 root 的 `npm run dev:desktop` 開一個本機專案——看板的變更卡與討論卡 meta 列尾端有最後更新；把一個變更的任務全部勾完後，詳情頁出身行出現「就緒」加日期，取消勾選一個之後消失；卡片所在的欄與改動前相同；篩選面板的建立者選單與改動前相同（本機專案沒有識別）。驗證：四項都符合，卡片高度沒有改變、頁面沒有水平捲軸。 <!-- speclink-task:tsk_01M8AN1GTHNEGW3AC32M1HP7XK -->
