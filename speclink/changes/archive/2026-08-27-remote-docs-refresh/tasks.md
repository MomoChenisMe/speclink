## 1. 前置與盤點

- [x] 1.1 確認 remote-claim-ownership 已完成品質站並封存（其 delta 併入正典）——文件查核以封存後正典＋程式碼為準；未封存前不開始修正輪 <!-- speclink-task:tsk_01M0Z6NWZN6JN60F4CDE59WBAA -->
- [x] 1.2 逐檔盤點 docs/ 下 21 份遠端相關文件（roadmap、product-status、remote-getting-started、verb-contract、workflow、getting-started、configuration、development、sdk-node 各雙語版，及 platform-architecture、server-deployment、server-backup、server-store-drivers、implementation-refactor-roadmap 繁中版）與 README.md、README.en.md：對照正典 specs 與實作，逐檔記下「文件敘述 vs 實況」偏差清單（含「無偏差」的檔案也記一筆，證明掃過） <!-- speclink-task:tsk_01M0Z6NWZP8AQC7WKAZ73AGGRK -->

## 2. 已知偏差修正（雙語同步）

- [x] 2.1 roadmap（雙語）：遠端協作線重寫——已閉合的縫（capability 清單與 change 詮釋資料、promotedTo、認領持久化與呈現）自「還沒鋪平」清單移除；離線衝突呈現依討論結論改寫為「殘餘面小、暫不立案」的準確敘述；下一個里程碑更新為仍未做的項目（桌面遠端勾任務 touched files 等），不把規劃寫成現行 <!-- speclink-task:tsk_01M0Z6NWZPG677SCY7WEB1PY0Q -->
- [x] 2.2 product-status（雙語）：Desktop Remote Workspace 列移除已消除的縫；認領（claim）列自「回聲確認」更新為持久化語意（寫入 meta、跨重啟可見、409 ownership 衝突）；認領人與詮釋資料呈現入列現行能力 <!-- speclink-task:tsk_01M0Z6NWZPDR8HW0PYXPKN4CW6 -->
- [x] 2.3 remote-getting-started（雙語）：依修訂後的「Remote Getting Started 提供可重複的完整操作路徑」requirement——第 4 節把「第一位管理員也必須為自己授予 membership」明示為必經步驟（現行以邀請他人情境帶過，實跑時 auth status 卡 access denied 即此縫）；npx 最短路徑與 checkout 開發者路徑的分工敘述對齊 requirement 正文 <!-- speclink-task:tsk_01M0Z6NWZP4JRATH9H5CC1DTZW -->
- [x] 2.4 verb-contract（雙語）：claim 段落更新為持久化語意與 ownership_lost 409 實際可觸發；單 change 讀取與清單端點的欄位敘述補 remote-read-parity 落地的歸屬欄位 <!-- speclink-task:tsk_01M0Z6NWZPXSG0Y85ATNT3GFKE -->

## 3. 盤點發現修正與雙語對等

- [x] 3.1 修正 1.2 盤點清單中其餘檔案的偏差（低頻文件的過期遠端敘述優先——上次誤判來源），每處修正註明依據（spec 條文或程式碼行為）；無偏差檔案零改動 <!-- speclink-task:tsk_01M0Z6NWZP86Q5VREP6KT3QFQ4 -->
- [x] 3.2 逐檔核對中英兩語版本結構與事實對等（user-documentation「中英文文件保持結構與事實對等」requirement）：本刀改過的每個段落兩語內容一致、僅語言不同 <!-- speclink-task:tsk_01M0Z6NWZPWCP5H67GNVD05R9M -->

## 4. 驗證與人工驗收

- [x] 4.1 跑 scripts/remote-docs.test.mjs 文件查核腳本與內部連結檢查全綠；腳本斷言若與新現況衝突，依實況更新斷言（歸文件查核面、非行為改動） <!-- speclink-task:tsk_01M0Z6NWZPVNYZZE42JFVGHG1Z -->
- [x] [M] 4.2 過目更新後的 roadmap 與 product-status 對外敘述（鐵人賽文章將引用這兩份），確認語氣與事實符合預期；抽查 remote-getting-started 第 4 節照新敘述能一次走通首位 Admin 的 membership 步驟 <!-- speclink-task:tsk_01M0Z6NWZPV5FJ2CHD6RZ89Y82 -->
