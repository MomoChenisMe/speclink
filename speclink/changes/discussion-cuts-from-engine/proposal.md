## Why

討論結論裡的「刀」現在有兩份解析：引擎的 `parse_cuts`（plan-unpromoted-cuts 新增，`plan` 用它列出尚未轉出的刀）與桌面前端 packages/ui/src/discussionModel.ts 的 `parseCuts`、`cutPromotion`（討論詳情頁的刀卡、「N 刀尚未轉出」提示框、衍生變更分組）。兩份規則是逐條移植的，但已經有差異（前端認 `*`、`+`、編號列點與列點外的粗體段落，引擎只認頂層 `- ` 列點；引擎先去掉 HTML 註解，前端不去），之後任何一邊改規則，路線圖、`plan` 與討論詳情頁就會對「哪些刀還沒轉出」給出不同答案。討論 herdr-speclink-panel 定案由引擎解析一次、各處讀同一份，本變更是其中的第 4 刀（最後一刀）：討論詳情頁改用引擎解析的刀，移除前端的分刀解析。

## What Changes

- 引擎的分刀解析多回傳兩項：首行其餘的說明文字（刀卡的第一條細線列）與該刀列點的首行原文（讓前端把刀對回結論裡的那一條列點）；並提供「一份記錄的刀」的單一入口，刀與 `promoted_to` 取自同一次讀取，判定每刀尚未轉出的變更名
- server 的 GET /discussions/{slug} 回應增列 `cuts`（live 與已封存的記錄都有）；protocol 的 `ShowDiscussionResponse` 增 `cuts` 欄位，舊 server 缺鍵時讀作空陣列
- 桌面讀討論記錄時一併取得引擎解析的刀（local 經 speclink-desktop-core，remote 經 GET /discussions/{slug}），詳情頁的刀卡、「定案細節」計數、「分刀」數字磚、「N 刀尚未轉出」提示框與衍生變更分組都改用這份資料
- 移除 packages/ui/src/discussionModel.ts 的 `parseCuts`、`cutPromotion` 與其專用的解析函式；刀卡畫面不變，卡內細線列照舊由該刀列點的巢狀內容組成
- 行為變化（刻意）：以 `*`、`+`、編號起頭的列點與列點之外的粗體段落不再被當成刀，改留在定案細節，與 `plan` 的判定一致；連到沒有 `cuts` 的舊 server 時，討論詳情頁不呈現分刀區段，刀列留在定案細節
- 相容性影響：CLI `discuss show --json`（fs 與 remote 模式）與 `discuss list --json` 的輸出逐位元不變；`plan --json` 不變；GET /discussions/{slug} 只在頂層新增 `cuts`，既有 `info` 與 `content` 不變

## Non-Goals

- 不改 `discuss show`、`discuss list` 的 CLI 輸出：刀只經 server 回應與桌面查詢提供
- 不改分刀的判定規則本身（識別籤、範圍句、變更名籤的規則沿用 change-plan「plan 列出討論結論中尚未轉出的刀」）；只多回傳說明文字與首行原文
- 不讓引擎解析刀卡的巢狀子項：子項是一般的 markdown 列表，照舊由前端的列表拆分呈現
- 不改刀卡、提示框與衍生變更分組的外觀
- 不改路線圖（desktop-roadmap）與 Herdr 端

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「討論結論以欄位標籤呈現」的分刀判定改為取自引擎，並寫明舊 server 沒有分刀資料時的呈現
- `client-protocol`: 新增「討論詳情回應的分刀 payload」；「討論搜尋回應 payload」中「詳情回應逐位元不變」改為除 `cuts` 外不變
- `server-verb-api`: 新增「討論詳情回應攜帶分刀」——GET /discussions/{slug} 以引擎計算回應的 `cuts`

## Impact

- Affected specs: desktop-app、client-protocol、server-verb-api
- Affected code:
  - New:
    - packages/ui/src/__tests__/discussionCuts.test.ts
  - Modified:
    - crates/engine/speclink-core/src/lifecycle/discuss/cuts.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/tests/it/api/discussion_routes.rs
    - crates/adapters/speclink-cli/tests/it/remote_read_path.rs
    - apps/desktop/core/src/discussions.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src/adapter/tauriDataSource.ts
    - apps/desktop/src/adapter/remoteDataSource.ts
    - apps/desktop/src/App.tsx
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/remoteDataSource.test.ts
    - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
    - packages/ui/src/adapter.ts
    - packages/ui/src/index.ts
    - packages/ui/src/discussionModel.ts
    - packages/ui/src/components/ConclusionDoc.tsx
    - packages/ui/src/components/discussionDocSummary.tsx
    - packages/ui/src/components/PromotedDoc.tsx
    - packages/ui/src/components/DiscussionDetailPage.tsx
    - packages/ui/src/components/ArchivedDetailPage.tsx
    - packages/ui/src/__tests__/discussionModel.test.ts
    - packages/ui/src/__tests__/conclusionDoc.test.tsx
    - packages/ui/src/__tests__/discussionDetailPage.test.tsx
    - packages/ui/src/__tests__/promotedDoc.test.tsx
    - packages/ui/src/__tests__/archivedDetailPage.test.tsx
- 影響的層：speclink-core（純函式擴充）、speclink-protocol、speclink-server、speclink-desktop-core、Tauri 殼、@speclink/ui、apps/desktop；speclink-cli 的程式不動（remote 模式照舊只取 `info` 與 `content`），只在 remote_read_path.rs 加一個測試
