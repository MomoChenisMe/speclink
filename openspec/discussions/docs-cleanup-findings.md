---
topic: 文件整理過程中發現的問題，逐項決定怎麼處理
slug: docs-cleanup-findings
status: promoted
created: 2026-10-07
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: server-reader-write-guard, ingest-accepts-change-name, sync-specs-after-docs-cleanup
---

# Discussion: 文件整理過程中發現的問題，逐項決定怎麼處理

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

2026-10-07 重新整理 README 與 docs/ 時，稽核代理對照 0.8.0 程式碼找到五類文件以外的問題：官方 server 的 reader 寫入權限缺口、規格與程式碼不一致、ingest 技能參數與其他技能矛盾、幾個小問題（init 範本網址、CI smoke test、桌面設定頁文案、server 內容檢查回 500），以及截圖過舊。需求都很具體，不需要先收斂（grill 階段為零題），直接以假設清單進行。相關正式規格：server-verb-api、server-policy-write、remote-workspace-data、client-protocol、verb-contract、user-documentation、ingest-skill、drift-computation、skill-routing、desktop-app；偵察時沒有進行中的變更，因此沒有 in-flight delta。
Prior discussions: change-execution-order, improve-repo-layout, node-sdk-completion-and-doc-alignment, web-service-navigation-redesign, system-tray-status, collab-scenario-replan, manual-spec-edit-integrity, sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-07)

**Focus**: 五類問題各自怎麼處理、要切成幾個變更
**Position**: 五類問題全部決定，分三刀依序轉出（server 權限 → ingest 參數 → 規格同步），使用者確認全部假設。
- reader 寫入：server 在單一位置擋下 reader 的所有寫入，取代目前逐函式的 `if !binding.editor`（只有 routes.rs:394、465、497、1267 等約 10 處）；依據是 remote-workspace-data/spec.md:179（reader 的寫入面停用）與 2026-07-20-remote-workflow-policy 設計把 reader 定為檢視者
- server 內容檢查失敗（tasks.md 沒有 checkbox、審查輪缺 Scope 行）回 500 `internal`，改回 4xx `invalid_argument`；與 reader 寫入同屬「server API 回應正確性」，放同一刀
- 規格與程式不一致一律改規格、程式不動：user-documentation 的 macOS 換 CLI 時機改成「沒裝或版本不同才換」（依 cliInstall.ts needsRedeploy）；刪掉指向已移除架構文件的要求；405、中英 H2 字串相同、server 文件只有中文等條文改成現況；openspec/config.yaml 引用 docs/design 的那一行一起改；verb-contract/spec.md:73 的 409 reason 改成 protocol 實際的 `revision_conflict`、`refused`（speclink-protocol/src/error.rs:54-61）
- ingest：改 ingest.md 讓參數符合進行中變更名稱時當成目標變更，否則照舊當計畫檔；其他約 10 處呼叫（drift.md、apply.md、archive.md、apply-worktree-post.md、discuss.md、quality/drift.rs:651-655）不動
- init 範本的錯網址、CI smoke test 的無效 `&&` 檢查（ci.yml:45）、桌面 `settings.rulesHelp` 舊文案，併入規格同步那一刀
- 截圖：用 docs-screenshots.mjs 的示範 workspace 重拍全部 9 張，沒用到的 4 張放進文件（規格頁 → 入門封存步驟、討論頁 → workflow discuss、已封存頁 → workflow archive、設定頁 → 設定說明）；拍攝是手動任務，放在規格同步那一刀最後
- 前提：這次的文件整理先單獨提交，規格同步照提交後的文件寫
**Ruled out**: reader 可寫討論的例外清單——桌面已把 reader 的全部寫入停用，例外會讓 CLI 與桌面不一致；把 macOS 改成每次啟動都換 CLI——現行「版本不同才換」不會無故蓋掉使用者同版本的 CLI；改約 10 個 ingest 呼叫端成不帶參數——改動面大且 Agent 失去明確指定變更的方法；小問題另開獨立變更——三件都是文字對齊、風險低，併入規格同步即可；刪掉 4 張沒用到的截圖——文件以白話為目標，截圖有助理解，重拍本來就要做

## Conclusion

**Decision**: 文件整理找到的問題分三刀依序處理，開工前先單獨提交這次的文件整理。
- 前提：README 與 docs/ 的整理（含移除 docs/design/、logo 概念稿與守門測試改為錨點比對）先單獨提交，三刀都以提交後的文件為準
- **cut 1 `server-reader-write-guard`**: 官方 server 在單一位置擋下 reader 的所有寫入，並把內容檢查失敗從 500 改成 4xx
  - 所有專案範圍的寫入端點都要求 editor 角色，reader 一律回 403 `permission_denied`、內容零改動；不設例外清單
  - 檢查集中在一個位置（例如寫入專用的 extractor 或路由層），取代 routes.rs 逐函式的 `if !binding.editor`
  - handshake 的 capabilities 與 docs/verb-contract 的端點表依新規則更新
  - 內容檢查失敗（例如寫入的 tasks.md 沒有 checkbox、審查輪缺 `**Scope**:` 行）回 `invalid_argument`，不再回 500 `internal`
- **cut 2 `ingest-accepts-change-name`**: ingest 技能接受變更名稱作為參數
  - 參數等於某個進行中變更的名稱時，當成要更新的變更、以對話內容為來源；否則照舊當成計畫檔
  - 只改 crates/engine/speclink-core/assets/skills/ingest.md，其他技能與 drift 引擎的 `/speclink-ingest <變更名稱>` 寫法不動
  - 依技能資產規則升 ASSET_VERSION、更新 golden 與 assets.lock
- **cut 3 `sync-specs-after-docs-cleanup`**: 規格與周邊文字對齊程式碼現況，最後重拍截圖
  - user-documentation：macOS 換 CLI 改成「沒裝或版本和 app 不同時才換」；刪除指向已移除的平台架構與實作路線圖的要求；中英對等改成 H2 錨點序列相同；GET 存取金鑰端點回 405 的條文改成現行的 `/api/speclink/v1/web/account/tokens`；刪掉「server 文件只有中文不在範圍」的例外
  - verb-contract：409 reason 改成 protocol 實際的名稱（`revision_conflict`、`refused`），與 speclink-protocol 的 8 值清單一致
  - openspec/config.yaml 引用 docs/design/platform-architecture.zh-TW.md 的那一行改掉
  - `speclink init` 產生的 .speclink.yaml 註解網址改成 https://github.com/MomoChenisMe/speclink
  - .github/workflows/ci.yml 的 smoke test 拆成逐行檢查，改查 init 實際產生的技能檔，不再查 CLAUDE.md／AGENTS.md
  - 桌面 `settings.rulesHelp` 文案改成「改設定只動目標那幾行，註解會保留」的現況
  - `[M]` 用 scripts/docs/docs-screenshots.mjs 的示範 workspace 重拍全部 9 張截圖；沒用到的 4 張放進文件：規格頁 → 入門的封存步驟、討論頁 → workflow 的 discuss、已封存頁 → workflow 的 archive、設定頁 → 設定說明
**Rationale**: reader 能寫入是安全問題，排第一；ingest 矛盾每天都會碰到，修一個檔就解決，排第二；規格同步要以 cut 1 之後的 server 行為與提交後的文件為準，而且截圖要等桌面文案修好才拍，所以排最後。原則是程式碼與現行文件為準、規格跟著改，改動面最小的修法優先。
**Rejected alternatives**:
- reader 可寫討論的例外清單 — 桌面已把 reader 的全部寫入停用，例外會讓 CLI 與桌面不一致
- macOS 改成每次啟動都換 CLI — 現行「版本不同才換」不會無故蓋掉使用者同版本的 CLI
- 改約 10 個 ingest 呼叫端成不帶參數 — 改動面大，Agent 也失去明確指定變更的方法
- 小問題另開獨立變更 — 三件都是文字對齊、風險低，併入規格同步即可
- 刪掉 4 張沒用到的截圖 — 截圖有助白話理解，重拍本來就要做
**Deferred**:
- repo 級或更細的角色權限 — 2026-07-20 的設計已明定只做 editor／reader 二值，本次只補齊二值的執行面
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion docs-cleanup-findings
