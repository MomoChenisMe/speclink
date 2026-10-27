## 1. 前置：回歸基線

- [x] 1.1 保存變更前 baseline exe 供自我基線雙沙盒對照：cargo build --release -p speclink-cli 後將 target/release/speclink.exe 拷貝到 repo 外（勿放 scratchpad，基建會消失）；驗證：baseline 拷貝可執行且 speclink --help 輸出正常

## 2. core：from_discussion 累積器與 link 追加

- [x] 2.1 撰寫失敗測試（crates/speclink-core/src/model.rs 與 crates/speclink-core/src/discuss.rs 的 #[cfg(test)]）：from_discussion 逗號清單的分割讀取（單值、多值、項目前後空白）；link 對已連結其他討論的變更追加本 slug 且既有值保留；追加後同組合重跑冪等不改檔——對應 spec「討論以 link 動詞併入既有變更」的「出身自討論的變更再併入新討論」與「同一組合重跑為冪等」情境；驗證：cargo test -p speclink-core --lib 新案例紅燈
- [x] 2.2 實作 design「D1 — from_discussion 以逗號累積字串存於 meta，讀取端分割」與「D2 — link 守衛改為追加且維持冪等」：meta 欄位維持原始字串、core 提供分割讀取，link 守衛表的「已連結其他討論」改為尾端累加（沿現行無結尾換行則補的寫入邏輯）、其餘守衛不變；驗證：cargo test -p speclink-core --lib 全綠
- [x] 2.3 手動驗證 CLI 行為：沙盒中對 meta 已有 from_discussion 的變更執行 speclink discuss link 另一討論 → exit 0、成功訊息形狀同現行、meta 累加、討論標 promoted；重跑同組合 → exit 0 且 git status 顯示無檔案變動

## 3. core＋cli：封存共行逐 slug

- [x] 3.1 撰寫失敗測試（crates/speclink-core/src/archive.rs 的 #[cfg(test)]）：spec「多來源討論的變更封存逐一共行」——兩份皆無引用則皆隨行封存、一份仍被其他在途變更引用則僅另一份隨行；單一來源情境結果與現行一致；驗證：cargo test -p speclink-core --lib 新案例紅燈
- [x] 3.2 實作 design「D3 — 封存共行逐 slug 判定，ArchiveOutcome 改複數」：archive 對 from_discussion 清單逐 slug 檢查存活引用，共行結果改複數承載；crates/speclink-cli/src/commands.rs 的 archive 人眼輸出改為逐討論各一行共行訊息；驗證：cargo test -p speclink-core --lib 全綠、cargo build --release -p speclink-cli 通過
- [x] 3.3 回歸對照：以 1.1 的 baseline exe 與新 exe 在雙沙盒對「單一來源討論變更的封存」情境比對 speclink archive 人眼輸出逐位元一致；驗證：diff 無差異

## 4. bridge＋GUI：fromDiscussions 多值呈現

- [x] 4.1 撰寫失敗測試（apps/desktop/core/src/query.rs 與 apps/desktop/core/src/manage.rs 的 #[cfg(test)]）：變更清單與詳情的 fromDiscussions 為 camelCase 陣列欄位——多值依 meta 順序、無來源討論時為空陣列、不再輸出單值 fromDiscussion 鍵；驗證：cargo test --lib（apps/desktop/core）新案例紅燈
- [x] 4.2 實作 design「D4 — bridge 欄位改名 fromDiscussions 陣列，GUI 多值呈現」的 bridge 側：query.rs、manage.rs 改送陣列，verbs.rs 的 archive 結果 camelCase 組裝隨共行複數調整；驗證：cargo test --lib（apps/desktop/core）全綠
- [x] 4.3 撰寫失敗測試（packages/ui 與 apps/desktop 的 vitest）：spec「變更的來源討論多值呈現」——變更卡單一討論徽章以清單第一份（出身）為代表且提示列出全部（packages/ui/src/components/ChangeCard.tsx）、詳情抽屜列出全部來源討論可互跳（packages/ui/src/components/RichDetailDrawer.tsx）、同源變更清單以來源討論交集非空判定（apps/desktop/src/App.tsx）、單一來源呈現不變；驗證：npm test -w packages/ui 與 npm test -w apps/desktop 新案例紅燈
- [x] 4.4 實作 GUI 多值呈現：packages/ui/src/adapter.ts 型別改 fromDiscussions 陣列、ChangeCard.tsx 徽章代表值與提示全列、RichDetailDrawer.tsx 來源討論清單、packages/ui/src/i18n.tsx 多值文案、apps/desktop/src/App.tsx 同源改集合交集；驗證：npm test -w packages/ui 與 npm test -w apps/desktop 全綠且 tsc 無型別錯誤
- [x] 4.5 GUI 真實視窗驗證（依開發備忘：先確認使用者未使用螢幕）：npm run build -w apps/desktop、關閉執行中 exe 後 cargo build --release -p speclink-desktop，啟動後以截圖檢視多來源變更卡的徽章提示、詳情抽屜來源討論清單與同源變更清單；驗證：截圖內容與 spec「變更的來源討論多值呈現」四個情境一致

## 5. 收尾

- [x] 5.1 全面回歸與 artifact 驗證：cargo test --workspace --lib、npm test -w packages/ui、npm test -w apps/desktop 全綠；speclink validate rediscuss-promoted-change 通過
- [x] 5.2 端對端流程驗證 spec「討論以 link 動詞併入既有變更」：沙盒中討論 A promote 成變更 → 新討論 B conclude 後 link 同一變更 → meta 為 A, B 順序累加 → speclink archive 該變更 → A 與 B 各出一行共行封存訊息且記錄皆移入 openspec/discussions/archive/；驗證：逐步 CLI 輸出與檔案效果符合上述
