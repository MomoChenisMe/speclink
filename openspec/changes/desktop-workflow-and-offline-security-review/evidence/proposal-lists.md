# 提案預設條列

- 共用落點為 `packages/ui/src/components/ProposalDoc.tsx`，沿用 NumberedSteps 與既有 Markdown／清單解析器，沒有專案名稱或目錄判斷。
- `ChangeDetailPage` 與 `ArchivedDetailPage` 已共用此元件，Desktop 的 Local／Remote 專案走同一畫面，無須逐頁複製。
- `apps/server-web` 目前是帳號／管理介面，沒有提案閱讀頁；本次不新增無關入口。其他宿主使用共用 ProposalDoc 時會取得相同預設。
- Why／What Changes／Proposed Solution 的散文區塊與清單依來源順序組成一個編號序列，保留巢狀列點、連結與程式區塊；空內容不造編號。未知章節與無二級標題文件保留原 Markdown 行為。
- 本提案 Why 整理成保留原意的條列；共用提案模板與 propose 技能要求一事一項，透過 init／update 提供所有專案，不批次改寫舊文件。
- 驗證：`npm test -w @speclink/ui -- proposalDoc` 29 項通過；`cargo test -p speclink-core --test it render_golden::` 63 項通過（含三種內建工具、自訂工具、資產版本與鎖定）；`git diff --check` 通過。
