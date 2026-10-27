## ADDED Requirements

### Requirement: Server 交付物內嵌同版本 SPA 資產

Release binary、Docker image 與本機 production build SHALL 在編譯期內嵌 `apps/server-web` 同一次 source revision 產生的 Vite `index.html`、manifest、hashed JavaScript／CSS、字型與圖示。建置順序 SHALL 為安裝 lockfile 固定的 npm dependencies、執行 `apps/server-web` production build、再編譯 `speclink-server`；缺少 index 或 manifest 時 server release build SHALL 以非零 exit code 失敗並指出需先完成 Web workspace build，SHALL NOT 產生只有 API 而沒有 UI 的成功 artifact。Runtime SHALL 只需要 non-root server binary，SHALL NOT 需要 Node、外部 `dist` volume、CDN 或第二個 Web service。

#### Scenario: Release binary 在空 runtime 載入 SPA

- **WHEN** 將 release server binary 放入沒有 Node 與 `apps/server-web/dist` 的 runtime，啟動後 GET `/login` 與 HTML 引用的 hashed assets
- **THEN** index 與全部資產成功回應、版本來自同一 binary，且未知 `/api/speclink/v1/web/missing` 回 JSON 404

#### Scenario: 缺少 Web build 使 release build 失敗

- **WHEN** production index 或 manifest 不存在時執行 server release build
- **THEN** build 以非零 exit code 結束並輸出先建置 `apps/server-web` 的可執行提示，不產生可發布 server artifact

#### Scenario: Docker multi-stage 不攜帶 Node runtime

- **WHEN** Docker workflow 依 lockfile 建 Web assets、編譯 server 並檢視最終 image
- **THEN** 最終 image 只有執行 server 所需檔案與 non-root 使用者，沒有 Node runtime 或獨立靜態檔服務

#### Scenario: Release workflow 保持資產與版本對齊

- **WHEN** tag 觸發 server binary 與 Docker image 發布
- **THEN** 每個 server artifact 都先通過同 revision 的 Web test 與 production build，並在無外部 assets 的 smoke test 載入 `/login`
