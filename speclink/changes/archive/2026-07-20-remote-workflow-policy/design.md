## Context

現況：GET /config 經 verb 的 read_doc 讀 DocumentId::WorkflowConfig、以 WorkflowConfig::from_text 解析後只回 schema 名，回應頭已帶 scope ETag（verb 的 scope_etag）；沒有任何 policy 寫入端點。identity 的 memberships 表無 role 欄位（sqlite schema 依註記走「shape 變更即 migration」機制）；binding 的 Capabilities 只有 context_snapshots／authentication／events。desktop 端：settings-ia-restructure 後 remote 專案設定頁為整頁不可用說明、settings provider 對 remote 為一律拒絕的 stub；desktop-core 的 settings 解析/寫入直接吃專案 root 路徑，無文字層入口。wire 錯誤詞彙已有 revision_conflict 與 invalid_config。§3.4 原則：遠端 policy 與 schema 不可由 client 決定——驗證與權限都必須在 server 端守住。

## Goals / Non-Goals

**Goals:**

- §10.6 遠端欄整段落地：revision 顯示、expected revision 儲存、409 保留輸入＋對照、reader 唯讀。
- 寫入的兩道 server 防線：引擎 fail-closed 驗證（壞 YAML 不落盤）與 role 權限（reader 403）。
- 本地設定路徑行為凍結；CLI 零改動。

**Non-Goals:**

- 不做完整 RBAC（僅 editor｜reader 二值、project membership 層級；repo 級與更細動作權限屬後續刀）；不動 server-admin 之外的 role 指定入口；不做 policy 版本歷史瀏覽（store history 已存、UI 屬後續）；不動 .speclink.yaml（tools 屬本機 checkout 概念，remote 分頁無此簽）；不做 desktop 對 409 的自動合併——對照後由使用者決定。

## Decisions

### 決策 1：config 讀寫走文件原文＋scope revision

GET /config 加 content（文件原文；文件缺席為 null）與 revision（與 ETag 同值）欄位；PUT /config body＝{content, expectedRevision}。以文件原文為 wire 單位而非結構化欄位——鍵保留語意（未觸及鍵原樣、設回預設移除鍵）由 client 的文字層改寫承擔，server 不解讀編輯意圖、只驗證與存檔；結構化 PATCH 被否：server 端要重建 desktop 的 targeted-key 改寫等於把編輯器搬進 server，違反薄端點原則。

### 決策 2：PUT 的兩道防線與錯誤語意

順序固定：先驗 role（reader 即 403，不碰文件）→ 再過 WorkflowConfig::from_text 驗證（失敗回 invalid_config、不落盤）→ 最後以 expectedRevision 走 store CAS 提交（不符回 revision_conflict、成功回新 revision）。缺 expectedRevision 為 schema 驗證錯誤直接拒收——PUT 一律 CAS、沒有無條件覆寫路徑。

### 決策 3：role 最小模型與傳播

memberships 表加 role TEXT NOT NULL DEFAULT 'editor'（sqlite 依既有 migration 機制升版；既有列一律 editor——升版後行為不變）；identity trait 的 membership 設定介面帶 role、audit 記錄含 role；invitation 建立的 membership 固定 editor（邀請流程不加選項——admin 事後可調，維持邀請頁最小）。binding 解析 membership role → Capabilities 加 policy_write 布林；desktop 的 capability 描述消費它停用存檔。server 的 403 是最終防線、desktop 停用只是 UX——兩者都要測。

### 決策 4：desktop-core settings 的文字層 seam

settings 解析與 targeted-key 改寫抽出 from-text／to-text 函式（現有以 root 路徑進出的函式改為讀檔→文字層→寫檔的薄包裝）；remote 路徑以 GET /config 的 content 進同一文字層——鍵保留語意單一實作、本地遠端零分歧。本地行為凍結以既有 settings 測試全綠＋新增文字層直測保證。

### 決策 5：remote 專案設定頁＝單一 Workflow 簽

三卡與本地 config.yaml 簽同形（專案說明、產出規則、產出政策），簽首以等寬字顯示 policy revision（取代檔案路徑註記的位置語意）；讀取走 session 的 settings provider（remote 實作以 /config content 過文字層 seam）；reader 時三卡表單唯讀、儲存鈕停用附「你的角色為檢視者」繁中說明。無 .speclink.yaml 簽。

### 決策 6：409 對照流程——informed resubmit，非 force overwrite

儲存收到 revision_conflict 時：使用者輸入原樣保留、浮出衝突面板——逐欄位對照「server 現值｜我的輸入」（重新 GET 取最新 content 與 revision）；兩個出口：「以 server 版重載」（捨棄輸入、表單回最新值）或「重新提交」（以檢視時的最新 revision 為 expectedRevision 再送我的輸入）。重新提交是經對照的知情決定，與被禁止的 force overwrite（未經對照直接蓋）語意區隔；對照期間 server 再前進則再次 409、流程遞迴成立。

## Implementation Contract

- server 整合測試（crates/speclink-server/tests/policy_write.rs，in-process＋memory identity）：GET 回 content＋revision 且與 ETag 一致；PUT 成功後 GET 見新內容新 revision；expectedRevision 過期回 revision_conflict 且內容未變；壞 YAML 回 invalid_config 且不落盤；reader 403 且內容未變；缺 expectedRevision 拒收；admin 設 role 入 audit；binding 回應 policy_write 隨 role 翻轉。
- desktop：文字層 seam 直測（from-text 解析各欄位、targeted-key 改寫的鍵保留與移除語意——與既有 root 路徑測試同斷言）；ProjectSettingsView remote 分支 vitest（revision 顯示、editor 可存、reader 唯讀附說明、409 面板對照與兩出口、輸入保留）。
- GUI 鐵律雙帳號手動（remote-dev-harness；操作前確認使用者未在使用螢幕）：npm run dev → admin 設一 editor 一 reader → editor 於 remote 分頁改政策存檔、CLI instructions 輸出反映新政策 → 兩視窗模擬並發改動實走 409 對照與重新提交 → reader 帳號見唯讀與停用說明 → 本地分頁設定頁行為與前一版一致。
- 回歸：cargo test --workspace、npm test -w apps/desktop、cargo build --release -p speclink-desktop 全綠；identity sqlite 升版後既有帳號 role 為 editor（migration 測試）。
