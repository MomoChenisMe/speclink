## MODIFIED Requirements

### Requirement: SPA 資產與 fallback 具可驗證的安全邊界

Production SPA SHALL 以 compile-time asset embedding 進入 `speclink-server` binary，runtime SHALL NOT 依賴相鄰 `dist`、Node、CDN 或外部字型服務。`/assets/*` SHALL 只回傳 build manifest 內的資產與正確 MIME；內容雜湊資產 SHALL 回 `Cache-Control: public, max-age=31536000, immutable`，SPA shell SHALL 回 `Cache-Control: no-cache` 與 self-only Content Security Policy。SPA fallback SHALL 僅匹配已定義 browser GET route，其中包含管理列表的詳情頁 `/admin/users/<使用者 id>` 與 `/admin/registry/<專案代號>`（列表路徑之後恰好一段非空的路徑片段）；未知 browser path、asset、`/api/*`、`/auth/*`、health、readiness 與下載 route SHALL NOT 回傳 `index.html`。

#### Scenario: 無外部靜態檔仍可載入 SPA

- **WHEN** 在沒有 Node、`dist` 目錄與外網連線的 runtime 啟動 release binary 並請求 `/login`
- **THEN** 回應載入內嵌 index、hashed JavaScript、CSS、字型與圖示，且所有資產來自相同 origin

#### Scenario: 拼錯 API 不被 SPA fallback 吞掉

- **WHEN** client GET `/api/speclink/v1/web/unknown`
- **THEN** Server 回 JSON 404，回應內容不是 SPA index

#### Scenario: 未知資產不回 shell

- **WHEN** browser GET `/assets/missing.js`
- **THEN** Server 回 404，回應內容不是 SPA index

#### Scenario: 詳情頁直接開啟與重新整理回傳 SPA shell

- **WHEN** browser 直接 GET `/admin/users/<任一使用者 id>` 或 `/admin/registry/<任一專案代號>`（例如在詳情頁按重新整理，或開啟別人分享的詳情頁連結）
- **THEN** Server 回 200 與 SPA index，由 SPA 依網址呈現該筆詳情頁或其「找不到」狀態；Server SHALL NOT 回 JSON 404

#### Scenario: 詳情頁以外的多段或空片段路徑不回 shell

- **WHEN** browser GET `/admin/users/a/b`、`/admin/users/` 或 `/admin/changes`
- **THEN** Server 回 404，回應內容不是 SPA index
