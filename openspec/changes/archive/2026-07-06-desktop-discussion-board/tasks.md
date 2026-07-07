## 1. 引擎：促轉下沉與查詢（design D1 promote 流程下沉 core；design D2 promoted_to 查詢不動 CLI 輸出）

- [x] 1.1 基線釘住：對 speclink discuss promote（含 --name 與預設名兩形）與 discuss list --json 的現行 stdout／--json 輸出擷取 snapshot 存入測試（重構前先錄）。驗證：snapshot 測試以現行版本綠燈通過。
- [x] 1.2 紅：撰寫促轉流程 core 函式的失敗測試（crates/speclink-core/src/discuss.rs 的 #[cfg(test)]）——archived 討論拒絕、change 名衍生（含日期前綴剝除）、建 change 帶 from_discussion meta、proposal 以結論預填、mark_promoted 累積（單值→逗號多值）；另撰寫 promoted_to 清單查詢的失敗測試（缺席／單值／逗號多值）。驗證：cargo test -p speclink-core 出現預期紅燈。
- [x] 1.3 綠：實作促轉流程 pub 函式與 promoted_to 查詢函式（DiscussionInfo 結構與序列化不動），crates/speclink-cli/src/commands.rs 的 promote 指令改為呼叫下沉函式。驗證：1.2 測試全綠、1.1 的 snapshot 逐位元不變、cargo test --workspace 全綠。

## 2. desktop-core 橋接

- [x] 2.1 紅：新增 apps/desktop/core/src/discussions.rs 的 tempdir 失敗測試——討論清單（active＋archived，項含 slug／topic／status／rounds／created／promotedTo，camelCase）、記錄全文讀取（slug 定址、路徑穿越拒絕）、促轉橋接端到端建出 change、歸檔橋接將記錄移入 discussions/archive/。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 2.2 綠：實作 discussions.rs（消費第 1 章 core 函式），apps/desktop/src-tauri/src/lib.rs 註冊四支 command（list_discussions、discussion_document、promote_discussion、archive_discussion），apps/desktop/src/adapter/tauriDataSource.ts 對映。驗證：2.1 測試全綠、cargo build -p speclink-desktop 通過。
- [x] 2.3 對新增 core 函式、command 與參數處理執行 sharp-edges audit checklist（speclink instructions --skill audit），逐項記錄結論，發現的尖銳邊以紅綠循環修正。驗證：audit 清單逐項有結論、cargo test 全綠。

## 3. 看板討論欄（design D3 討論欄兩級呈現；design D4 chips 狀態由 change 存在性派生）

- [x] 3.1 紅：撰寫 packages/ui 失敗測試，涵蓋 spec 需求「討論於看板第 0 欄兩級呈現」——open 全卡（topic＋回合數、無動詞）、concluded 全卡（促轉／歸檔按鈕）、promoted 欄底收合細列、chips 三態派生矩陣（active 各階段／封存 dated 尾碼命中／兩清單皆無標已刪除）、封存討論不出現、空專案欄空狀態。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 3.2 綠：新增 packages/ui/src/components/DiscussionColumn.tsx（全卡＋細列＋chips 派生），KanbanBoard.tsx 擴為四欄並接受 discussions props，packages/ui/src/adapter.ts 加 DiscussionItem 型別與 SpeclinkDataSource 的討論清單、記錄讀取、促轉、歸檔四方法（design D6 討論瀏覽與促轉進 SpeclinkDataSource）。驗證：3.1 測試全綠、既有看板測試不破。

## 4. 討論抽屜與同源連結（design D5 討論抽屜四分頁與再促轉）

- [x] 4.1 紅：撰寫失敗測試，涵蓋 spec 需求「討論抽屜檢視與 GUI 促轉」——四分頁區段切分渲染（含非預期格式整篇退回）、促轉分頁的子 change 現況與跳轉、再促轉於 concluded／promoted 可用、促轉確認後回呼與失敗錯誤呈現、change 卡討論徽章、change 抽屜「來自討論」與同源清單互跳。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 4.2 綠：新增 packages/ui/src/components/DiscussionDrawer.tsx（四分頁、動詞回呼），ChangeCard.tsx 加討論徽章、RichDetailDrawer.tsx 加來源討論與同源清單區塊（fromDiscussion 經 ChangeItem 新欄位帶入），apps/desktop/src/App.tsx 接上抽屜開閉、促轉／歸檔確認對話框與 store 整批 refresh。驗證：4.1 測試全綠、npm test -w apps/desktop 全綠。

## 5. 已封存頁雙節（design D7 已封存頁雙節）

- [x] 5.1 紅：撰寫失敗測試，涵蓋 spec 需求「已封存頁含討論節」——變更／討論兩節分列、封存討論唯讀展開（無任何寫入按鈕）、搜尋同時過濾兩節。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 5.2 綠：擴充 packages/ui/src/components/ArchivedList.tsx 為雙節（討論節復用抽屜的區段切分渲染），App.tsx 傳入封存討論資料。驗證：5.1 測試全綠。

## 6. 整合驗證

- [x] 6.1 全套自動化：cargo test --workspace、npm test -w packages/ui、npm test -w apps/desktop 全綠；1.1 snapshot 與 parity_suite／color_suite 照常通過；git diff 確認 crates/speclink-cli 僅 promote 指令改呼叫下沉函式、輸出零變更。驗證：全部通過。
- [x] 6.2 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中 exe；操作前確認使用者沒在使用螢幕）：以本 repo 實際討論記錄操作——外部 CLI add-round 後討論卡回合數自動更新、GUI 促轉建出 change 並自動現身提案中、細列 chips 隨子 change 開工／歸檔變化、再促轉累積 chip、封存討論展開檢視、四欄佈局下既有拖曳互動回歸。驗證：每項有截圖或觀察記錄，行為與 specs/desktop-app/spec.md 各 Scenario 一致。
