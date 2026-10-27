## 1. audit 基座與門禁

- [x] 1.1 【紅→綠】identity schema 演進一版新增 audit 表（操作者、封閉動作種類集、對象、UTC 時間、來源）與唯讀查詢介面（倒序分頁）；migrate 升級測試——前一版資料庫升級後既有資料完整、audit 可寫可查、較新版本拒開。無更新/刪除介面。 <!-- speclink-task:tsk_01KXG4SFCX491BFCRATZH0XF9N -->
- [x] 1.2 【紅】針對「admin 門禁前置且非 admin 一律 403」寫測試：無 admin 旗標者訪問 /admin 頁與 admin API 皆 403 三元組且不執行動作；admin 經 session 與 bearer 皆可通行；被停權的 admin 下一請求失去通行；admin API 套 API version 檢查。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXG4SFCXRM43YNAENWP21QJ8 -->
- [x] 1.3 【綠】實作 admin 門禁（既有認證後檢查 admin 旗標）與 admin API/頁面路由骨架（crates/speclink-server/src/admin.rs、audit 寫入在 crates/speclink-server/src/audit.rs），1.2 全綠。 <!-- speclink-task:tsk_01KXG4SFCX3PHQD3ZT667MB4TP -->

## 2. 管理動作單點實作

- [x] 2.1 【紅→綠】使用者管理動作（單點函式＋audit 同 transaction）：列表、邀請（改走單點路徑，invite 子命令與 setup 同函式、來源欄位各記 cli/web）、停權/復權、membership 調整、admin 旗標調整；停權最後一位 active admin 拒絕並明示原因。測試涵蓋「最後一位 admin 不可自斷」與「audit 與動作同生死」（資料層失敗時無 audit 記錄）。 <!-- speclink-task:tsk_01KXG4SFCXTZVZ7WV7D6J0ZGXS -->
- [x] 2.2 【紅→綠】registry 管理動作：project/repo 建立（重複 key 拒絕）與顯示名變更；key 無變更介面且 binding 以原 key 照常。測試涵蓋「registry key 不可改」情境。 <!-- speclink-task:tsk_01KXG4SFCXEA92370QNNW8VY9T -->
- [x] 2.3 【紅→綠】憑證監督：全站 PAT 與 device credential families 的 metadata 列表（所屬 user、prefix、名稱、到期、last-used、建立時間；無明文無 hash 無讀回介面）、強制撤銷走自助撤銷同一方法並記 audit（token id 與 prefix，無祕密值）。涵蓋「強制撤銷即時且留痕」情境。 <!-- speclink-task:tsk_01KXG4SFCXS2QCNARTKNGG8H4Q -->
- [x] 2.4 【紅→綠】CLI 子命令 headless 等效：user suspend/reactivate、token revoke、project create、repo create——與 API 同函式、audit 來源記 cli、操作者記 system。測試涵蓋「三入口等效停權」情境（api/web/cli 各停權一人，audit 來源正確）。 <!-- speclink-task:tsk_01KXG4SFCXX1ZK1ZZTGFGXNFBY -->

## 3. /admin 頁面組與系統資訊

- [x] 3.1 【紅→綠】/admin server-rendered 頁面組：使用者（列表＋邀請/停權/復權/membership/admin 旗標表單）、registry（建立與改顯示名）、憑證（metadata 列表＋撤銷）、audit（唯讀倒序分頁）；沿用 session 與同源驗證；頁面組不含任何規格內容（changes/specs/discussions 無路由無連結）。 <!-- speclink-task:tsk_01KXG4SFCXNCGYQ7QY11480TNR -->
- [x] 3.2 【紅→綠】系統狀態頁與對應 API（涵蓋「系統資訊唯讀聚合」）：engine/API 版本、store manifest、health 即時結果、identity schema version、各 scope outbox 積壓（最新序號減 acked cursor）；store 失聯時頁面顯示 health 失敗、identity 側管理功能照常（涵蓋「store 失聯不癱管理面」情境）。 <!-- speclink-task:tsk_01KXG4SFCX07PECEGM6YAYD1QF -->

## 4. 端到端與回歸

- [x] 4.1 【紅→綠】管理面 e2e：真 server（SQLite）走 setup 建 admin → /admin 邀請成員與建第二組 project/repo → 成員接受邀請建 PAT 對新 project 走 CLI 動詞 → admin 強制撤銷該 PAT 後成員 CLI 收 401 → audit 頁含上述全部動作且倒序正確。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXG4SFCXSPPGCF4CYYMVP4E8 -->
- [x] 4.2 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff。驗收：全數通過。 <!-- speclink-task:tsk_01KXG4SFCX2729HMZXZC20BG57 -->
