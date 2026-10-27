## Context

identity 資料庫已含 users（admin 旗標、active 狀態）、memberships、invitations、PATs、sessions、device credential families 與（setup 刀後的）projects/repos registry；schema 版本守門與 migrate 演進機制運轉中。Web 側有 session cookie（HttpOnly/Secure/SameSite=Strict）與 POST 同源驗證；API 側 bearer 前置支援 PAT 與 access token。停權目前只有 identity 層的 set-active 方法（標註 admin/測試支援），無營運入口；invite 是 server binary 子命令。藍圖 §13.2 列 /admin 最低功能集與「所有管理動作寫 audit log」「same-origin secure cookie、PAT/secret 不進 localStorage」「headless 可關 Admin UI 改 server CLI/Admin API」「Admin UI 不提供規格日常編輯」。

## Goals / Non-Goals

**Goals:**

- 日常營運（邀請、停權、registry、token 撤銷）不再需要主機 shell；每筆管理動作有稽核記錄。
- Web、API、CLI 三入口同一實作路徑同一 audit——無旁路管理動作。
- 憑證監督只見 metadata：明文不可讀回是既有不變式，管理面不打破。

**Non-Goals:**

- 不做 backup/export 與 restore validation 的介面（server-backup-restore 刀；§13.2 清單中的 backup/export、restore validation 與 migration 觸發歸該刀——migration 的營運觸發與備份還原同屬資料操作風險域）。
- 不做 project 內細粒度 role（reader/writer 分級）：本刀的 role 管理範圍是 membership 有無與 admin 旗標——server-identity-pat 刀已定此為現階段授權判準，細粒度屬後續需求出現時再議。
- 不做 registry 刪除（資料保留與孤兒 scope 議題留待 backup 刀之後）；key 不可改，僅顯示名可改。
- 不做 audit log 的輪替/匯出與保留政策（隨 backup 刀的資料治理一併設計）。
- 不做 email 通知、不做 OIDC/SSO、不動一般使用者的任何頁面與 API。

## Decisions

### 決策 1：admin 門禁是既有認證的旗標檢查，Web 與 API 同门

admin 路由（頁面與 API）前置：既有 session 或 bearer 認證成功後，再檢查 user 的 admin 旗標——非 admin 回 403 permission_denied，與非成員的 403 同 reason（不新增 wire reason）。admin API 掛在既有 API 路由樹的 admin 前綴下，套 X-Speclink-Api-Version 檢查；/admin 頁面沿用 web session 與同源驗證。停權的 admin 自己也即時失效（既有逐請求查驗語意自然涵蓋）。

### 決策 2：管理動作單點實作，三入口共用

每個管理動作（邀請、停權/復權、membership/admin 旗標調整、registry 建立/改名、token 強制撤銷）是 server 內單一函式：檢查前置 → 寫 identity/registry → 寫 audit，一個資料庫 transaction 完成。admin API handler、/admin 表單 handler 與 CLI 子命令都呼叫同一函式，只是入口與 audit 的來源欄位不同——不存在只有某入口能做或漏 audit 的動作。invite 子命令與 setup 流程的建立動作改經同一路徑（CLI 來源記 cli、setup 記 web）。

### 決策 3：audit log 只增不改，記錄五元組

identity 資料庫 schema 演進一版新增 audit 表：操作者（user id；CLI 以主機身分記 system）、動作種類（封閉字串集：user-invited、user-suspended、user-reactivated、membership-changed、admin-flag-changed、project-created、project-renamed、repo-created、repo-renamed、token-revoked、setup-completed）、對象識別、UTC 時間、來源（web、api、cli）。無更新無刪除介面；/admin 的 audit 頁唯讀倒序分頁。敏感值不入 audit——記 token id 與 prefix，不記 hash 或明文。

例外：`setup-completed` 是**完成標記**而非單一資料變更。setup 的建立動作（建 admin、建 project/repo）分散在精靈的多個請求，無法與任一筆同 transaction；故 setup-completed 在 setup 完成（consume token）時以 best-effort 補記一筆，寫入失敗不使 setup 失敗（完成優先於稽核標記）。「audit 與動作同生死」的原子不變式僅適用於單一請求內的變更型管理動作（admin_* 單點函式），不含此完成標記。

### 決策 4：憑證監督頁只列 metadata，撤銷即時

/admin 憑證頁列全站 PAT 與 device credential families：所屬 user、prefix、名稱、到期、last-used、建立時間——無明文、無 hash、無讀回途徑（§13.3：Admin 可檢視 metadata 與撤銷，不能讀回明文）。強制撤銷走與自助撤銷同一 identity 方法（即時生效），另記 audit（token-revoked，操作者為 admin）。

### 決策 5：系統資訊頁聚合既有可觀測面

系統狀態頁唯讀聚合：engine 版本與 API 版本（既有常數）、store manifest（driver、contract version、capabilities、等級）、store health 即時結果、identity schema version、每個 registry scope 的 outbox 積壓（最新序號減 acked cursor）。全部出自既有介面，不新增探針；store 失聯時頁面如實顯示 health 失敗而非 500。

## Implementation Contract

- Behavior：admin 登入 /admin 後可邀請成員（得一次性 URL）、停權/復權、調整 membership 與 admin 旗標、建 project/repo 與改顯示名、檢視與強制撤銷任何憑證、檢視系統狀態與 audit log；每筆動作出現在 audit 頁；headless 部署以 CLI 子命令完成同樣動作且同樣入 audit。非 admin 訪問任何 admin 路由回 403。
- Interface / data shape：admin API 前綴下的 JSON 路由（users 列表/邀請/停權/復權/membership/admin 旗標、registry 建立/改名、tokens 列表/撤銷、system 資訊、audit 分頁）；/admin server-rendered 頁面組；CLI 子命令 user suspend/reactivate、token revoke、project create、repo create；audit 記錄五元組（操作者、動作種類、對象、UTC 時間、來源）。
- Failure modes：非 admin → 403 permission_denied（API 三元組／頁面 403）；停權自己 → 允許但下一請求即失效（audit 留痕）；registry 重複 key → 表單/API 錯誤拒絕；key 改名嘗試 → 無此介面（僅顯示名可改）；store 失聯 → 系統頁顯示 health 失敗、其餘管理功能（identity 庫）照常。
- Acceptance criteria：cargo test -p speclink-server 全綠（門禁、各管理動作三入口、audit 完整性、憑證頁無明文、系統頁聚合）；npm run test:all 全綠且 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- audit 動作種類是封閉字串集 → 新管理動作必須同步增列；以單元測試「每個變更型 admin 動作恰寫一筆 audit」守住，漏記即紅燈。
- 停權 admin 自己可能鎖死全站（唯一 admin 自停權）→ 拒絕停權最後一位 active admin，錯誤明示原因——比允許後靠主機救援safer。
- 撤除最後一位 admin 的 admin 旗標會造成等價的全站管理鎖死，但 spec 只明文要求停權情境的守衛，故 `admin_set_admin_flag` **刻意不加**最後一位 admin 守衛（避免超出規格）。其復原路徑是重啟 server：`ensure_setup_token` 在 has_admin 為 false 時會重新鑄造 setup token 供重新引導。若日後要收斂此風險，應同步更新 spec 需求與此決策，再比照停權加上守衛。
- CLI 來源的操作者記 system 而非真人 → headless 環境的既有信任模型（主機檔案存取即管理權）；audit 的來源欄位讓兩類可區分。
- 憑證頁聚合全站 token metadata → 僅 admin 可見；頁面不含任何祕密值，洩漏面即 metadata。

## Migration Plan

前置依賴 server-setup-registry 已歸檔。identity schema 由 migrate 自動升級（audit 表，只加表）；invite 子命令改走單點實作路徑屬內部重構，參數與輸出不變。回退即回捨 change；audit 表資料隨庫保留無需處理。

## Open Questions

（無）
