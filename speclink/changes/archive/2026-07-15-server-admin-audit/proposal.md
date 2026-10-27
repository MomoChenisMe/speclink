## Why

setup 之後的日常營運還是缺門：邀請、停權、registry 增補、token 撤銷全部要 ssh 進主機跑子命令或根本沒有介面（停權目前只有 identity 層的測試支援方法，無任何營運入口），且管理動作零軌跡——誰邀了誰、誰撤了哪個 token 無從稽核。藍圖 §13.2 規定 /admin 最低功能集：使用者邀請/停權、role、Project/Repo registry、Engine/API/Store schema versions、Store/outbox health、token revocation 與 audit log；所有管理動作寫 audit log；headless 環境可關 Admin UI 改用 server CLI/Admin API；Admin UI 只管 installation/administration，不提供規格看板與日常編輯。identity（admin 旗標）、registry（已遷庫）與 session/同源防護全部就緒，本刀補齊管理面與稽核。

目標使用者：server 運維者與團隊 Admin（瀏覽器完成日常管理、稽核有據）；headless 部署者（CLI/API 等效路徑）。

## What Changes

- 新增 Admin API（admin 旗標門禁）：使用者管理（列表、邀請、停權/復權、membership 與 admin 旗標調整）、registry 管理（建立 project/repo、改名）、憑證監督（列全站 PAT 與 device credential families 的 metadata——不含明文、不可讀回——與強制撤銷）、系統資訊（engine/API 版本、store manifest 與 health、identity schema version、各 scope 的 outbox 積壓）。session（Web）與 bearer（API）皆可通行 admin 門禁，非 admin 一律 403。
- 新增 /admin Web UI（server-rendered，沿用既有 session 與同源防護；非 admin 使用者訪問回 403）：上述功能的最小頁面組——使用者、registry、憑證、系統狀態、audit log 檢視。Admin UI 不含任何規格內容頁（changes/specs/discussions 不出現）。
- 新增 audit log：identity 資料庫新表（schema 演進一版），每筆管理動作記操作者、動作種類、對象、UTC 時間與來源（web、api、cli）；涵蓋 admin API 全部變更型動作、invite 子命令與 setup 流程的建立動作。audit log 只增不改，admin 介面唯讀檢視；一般使用者不可見。
- server CLI 補齊 headless 等效子命令：使用者停權/復權、token 撤銷、registry 建立——與 admin API 同一實作路徑、同樣寫 audit。
- registry 改名語意定案：project/repo 的 key 不可改（binding 與 URL 的穩定識別），顯示名可改；不提供刪除（資料保留議題屬 backup 刀之後再議）。

## Capabilities

### New Capabilities

- `server-admin`: 管理面的功能集與門禁——admin API 與 /admin UI、使用者/registry/憑證/系統資訊管理、headless CLI 等效、audit log 的記錄與唯讀檢視。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增管理路由、頁面與子命令；一般使用者路徑（API 動詞、/account、device flow）零變更；identity 資料庫 schema 遞增一版（audit 表，migrate 自動升級）。CLI/桌面/本地模式零變更，parity 31 項、color 16 項、twin 8 情境凍結不動。前置依賴：server-setup-registry 刀（registry 在庫、admin 旗標可經 setup 建立）。
- Affected specs: `server-admin`（新增）
- Affected code:
  - New: crates/speclink-server/src/admin.rs、crates/speclink-server/src/audit.rs、crates/speclink-server/tests/admin_api.rs、crates/speclink-server/tests/audit.rs
  - Modified: crates/speclink-server/src/identity.rs、crates/speclink-server/src/identity_sqlite.rs、crates/speclink-server/src/web.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/main.rs、crates/speclink-server/src/state.rs
  - Removed: 無
