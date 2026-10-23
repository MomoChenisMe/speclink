# 更新頁與 Proxy 同類位置排查

搜尋：`if (!updater)`、`checkFailed`、`update_failed`，2026-10-20。

| 位置 | 判斷與處置 |
| --- | --- |
| UpdatesPage | 原本無 updater 即只回標題，改成版本／內附日誌始終可見；網路動作停用並說明。剩餘 `if (!updater)` 僅決定由頁面自行開內附日誌。 |
| AppSettingsView／App | App 一律傳入已知版本、日誌及偏好，ready 單獨表示網路動作可用；View 仍直接委派。 |
| store | 保留背景 checkFailed → idle，不發錯誤 toast；另存最後結果、固定錯誤碼與 operationId，不存外部訊息。 |
| UpdateInstallRow | 只呈現同意／下載／重啟／安裝錯誤，沒有隱藏內附資料的分支；沿用下載同意與取消，不改。 |
| updater_commands | 工作狀態與診斷結算移入 desktop-core；殼只轉接 plugin。回傳具體安全錯誤碼與 operationId，未知安裝錯誤仍為 update_failed；不解析含 URL 的錯誤字串或虛構 HTTP 狀態。 |
| network 診斷／測試入口 | 沿用 ProxyPolicy 快照與相同 operationId，CONNECT 407 保留 proxy-auth，無公開狀態碼時保持 null。模擬 CONNECT 成功後 TLS 錯誤／斷線及壞回應都不會當成功。 |

驗證：Host 157 + 5 測試、Desktop core 348 測試通過；Tauri cargo check、前端更新相關測試與型別檢查通過。這些是本機模擬與程式檢查，未證明公司 Proxy 已可用。公司實際根因及下載結果仍待人工任務 12.3。
