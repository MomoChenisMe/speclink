## Summary

遠端縫補齊（remote-read-parity、remote-claim-ownership）落地後，把 docs/ 下全部遠端相關文件一次對齊新現況——這是討論 remote-remaining-gaps 結論的第三步（刀 C），目的是避免縫閉合而文件反而落後。

## Motivation

前兩刀改變了遠端模式的可觀察行為：change 詮釋資料與 capability 清單在遠端直達、討論的已轉出狀態可見、認領持久化且看板呈現認領人。但文件仍記載舊現況：roadmap 遠端協作線的「還沒鋪平」清單仍列已閉合的縫、product-status 的 Desktop Remote Workspace 與 claim 列描述 stub 時代行為、verb-contract 的 claim 段落未反映 ownership 衝突已可觸發。此外同日實跑 remote-getting-started 最短路徑時發現：教學對「第一位管理員也必須自己授予 membership」不夠明確（user-documentation 正典的「第一位 Admin 完成 Desktop Remote Workspace」scenario 早已要求教學明示此步）——這是文件對既有正典的偏差，一併修正。文件漂移在本專案有前科（2026-08-23 討論曾因 product-status 誤載「桌面開不出遠端看板」把小縫高估成三刀新功能），總整理就是在還債並防止再犯。

## Proposed Solution

分三步：（1）盤點——以封存後的正典與程式碼為準，逐檔掃 docs/ 下 21 份遠端相關文件（含雙語版）與兩份 README，產出「文件敘述 vs 實況」偏差清單；（2）修正——已知偏差（roadmap 遠端協作線、product-status 兩列、remote-getting-started membership 明確化、verb-contract claim 段）加上盤點新發現，中英兩語同步改（user-documentation「中英文文件保持結構與事實對等」requirement）；（3）驗證——跑既有的文件查核腳本 scripts/remote-docs.test.mjs 與內部連結檢查，確認修正不引入新偏差。全程執行 user-documentation 既有 requirement 的承諾，不改變文件架構與任何 spec 條文。

## Non-Goals

- 除 user-documentation 一條 requirement（Remote Getting Started 路徑更新至 npx 首選）外，不動任何 openspec/specs/ 正典條文——其餘全是文件內容對齊，承諾本身不變
- 不重排文件架構、不增刪文件——單一責任與漸進揭露的既有編排維持
- 不寫入規劃能力為現行操作——「桌面遠端勾任務 touched files」「離線衝突殘餘呈現」等未做的項目保留在規劃側，只更新其敘述準確性
- 不含程式碼與測試改動（scripts/remote-docs.test.mjs 只執行、若其斷言與新現況衝突則屬盤點發現，修法歸本刀文件面）
- 鐵人賽文章側的引用更新——那在 iTHome2026-challenge repo，不在本專案範圍

## Alternatives Considered

- 隨每刀零星修文件（remote-task-evidence 模式）：前兩刀已各自修了直接相關段落，但跨文件的一致性（21 份、雙語對等）只有總整理輪能保證，且討論結論已裁定收尾一次對齊
- 只修四份高頻文件（roadmap、product-status、remote-getting-started、verb-contract）：驗證輪實測 21 份文件提到 remote，低頻文件的過期敘述正是上次誤判的來源，不縮範圍

## Capabilities

### New Capabilities

(none) — 規格掃描：user-documentation（文件集的全部承諾）、verb-contract（動詞模式歸屬正典）皆已存在且覆蓋本刀範圍，無新 capability。

### Modified Capabilities

- `user-documentation`: 「Remote Getting Started 提供可重複的完整操作路徑」更新——npx 最短路徑成為教學首選入口（checkout 的 npm run dev 路徑保留為開發者路徑），第一位 Admin 自授 membership 由 scenario 升為 requirement 正文

## Impact

- Affected specs: user-documentation——「Remote Getting Started 提供可重複的完整操作路徑」更新為 npx 最短路徑首選（@speclink/server 已上 npm，正典仍寫 checkout 的 npm run dev 為唯一流程），並把第一位 Admin 自授 membership 升為 requirement 正文；其餘文件修正執行既有承諾、不動條文
- Affected code:
  - New: (none)
  - Modified（候選全集，盤點後僅實際偏差者改動）: docs/roadmap.md、docs/roadmap.zh-TW.md、docs/product-status.md、docs/product-status.zh-TW.md、docs/remote-getting-started.md、docs/remote-getting-started.zh-TW.md、docs/verb-contract.md、docs/verb-contract.zh-TW.md、docs/workflow.md、docs/workflow.zh-TW.md、docs/getting-started.md、docs/getting-started.zh-TW.md、docs/configuration.md、docs/configuration.zh-TW.md、docs/development.md、docs/development.zh-TW.md、docs/sdk-node.md、docs/sdk-node.zh-TW.md、docs/platform-architecture.zh-TW.md、docs/server-deployment.zh-TW.md、docs/server-backup.zh-TW.md、docs/server-store-drivers.zh-TW.md、docs/implementation-refactor-roadmap.zh-TW.md、README.md、README.en.md
  - Removed: (none)
