---
topic: remote 模式本地開發一鍵啟動（無 docker 的 pnpm dev 等價物）
slug: remote-dev-harness
status: promoted
promoted_to: remote-dev-harness
created: 2026-07-16
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: remote 模式本地開發一鍵啟動（無 docker 的 pnpm dev 等價物）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

Phase 2 收官前（server-drift-api 與 server-release-packaging 已完成、phase2-e2e-chain 待實作），使用者提出：想在 dev 環境一鍵啟動 remote 模式全套（server＋desktop＋server web /setup 初始化），像 pnpm dev 一樣，不經 docker 手動測試；並問兩份正典文件（docs/platform-architecture.zh-TW.md、docs/implementation-refactor-roadmap.zh-TW.md）該怎麼補充、e2e 是否要納入。

模式：assumptions（碼庫脈絡充足）。codebase scout 結果：
- phase2-e2e-chain 提案明寫「真實 CLI binary 對真 server、SQLite driver、tempdir 隔離」——e2e 本來就是無 docker 的 cargo 整合測試；docker 只存在於 packaging 刀交付的部署產物（deploy/docker-compose.yml、deploy/docker-compose.postgres.yml、crates/speclink-server/Dockerfile）。
- server 本地啟動已可行：cargo run -p speclink-server -- --config <yaml>，首跑於 stdout 印一次性 /setup?token=… 連結（crates/speclink-server/src/main.rs 的 ensure_setup_token 段）。repo 內沒有裸的 config 範例（compose 於 up 時插值生成）。
- desktop 是 Tauri app（apps/desktop/src-tauri＝speclink-desktop crate＋vite 前端），dev 模式既有一鍵：tauri dev。
- root package.json 只有 test:all script，無 dev script——編排層完全缺席。
- 架構 §13.4 開箱流程僅寫 docker compose 路徑；§13.4 尾段有「若只提供流程範例，則應明確命名為 example/dev server」條款（措辭需避開「dev server」一詞）。roadmap 全文無 docker 字樣。
- 相關 change：phase2-e2e-chain（in-progress）、server-release-packaging（done 未封存）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-16)

**Focus**: 「不用 docker 測試」的真正需求是文件還是工具？
**Position**: 需求是 remote 模式的 pnpm dev 等價物——一鍵拉起 server（dev 設定）＋ desktop（tauri dev）＋ server web /setup，供手動測試；不是文件補充。
- 第一版假設（docs-only、e2e 不動）被使用者釐清修正：要的是可執行的一鍵啟動編排。
- e2e 判定維持不變：phase2-e2e-chain 本來就是「真實 CLI binary 對真 server、tempdir 隔離」的 cargo 測試，與 docker 無涉，不需為此增改。
- 既有零件盤點：server 端 cargo run＋--config 已可跑、首跑印 /setup token（main.rs ensure_setup_token）；desktop 端 tauri dev 已一鍵；唯獨缺編排層（root 無 dev script）與 dev 用 config 範例。
**Ruled out**: docs-only 補充——使用者明示要可執行工具，不是說明文字。e2e 加「無 docker」步驟——e2e 已天然無 docker，加了是把功能鏈驗收與部署形態驗證混淆。
**Open**: 一鍵的範圍（只起 server，還是 server＋desktop 同起）；dev 資料持久化語意（重啟保留免重跑 /setup？要不要 reset 指令）；要不要成刀（speclink change）與兩份正典文件的落點。

### Round 2 — assumptions (2026-07-16)

**Focus**: server 的 dev 設定來源——env 直入 server，還是編排層插值？
**Position**: 採 .env（gitignored）＋ .env.example（committed），由 dev script 讀取插值生成 .dev/config.yaml 後帶 --config 啟動 server；server 產品碼不動。
- 使用者要求：dev 下以 env 配置 store driver（fs/sqlite/postgresql 等）與必要設定，且必須有 .env.example 讓人知道怎麼配。
- 與 compose 同構：deploy/docker-compose.yml 明註「組態 YAML 不做環境變數展開，由 compose 於 up 時插值產生」——dev harness 在編排層做同一件事，既有 fail-closed 決策（設定單一來源）保持成立。
- env 清單依 crates/speclink-server/src/config.rs 實際欄位圈定：SUPPORTED_DRIVERS＝sqlite|serverfs|postgres|memory；sqlite/serverfs 吃 path、postgres 吃 url；identity 另有 driver＋path；再加 port 與 public_url（沿用 compose 已有的 SPECLINK_PORT／SPECLINK_PUBLIC_URL 命名）。
- identity 在 dev 固定 sqlite 落 .dev/——identity 用 memory 會在重啟後丟帳號/PAT，違反「setup 一次、之後直接測」的持久化體感。
**Ruled out**: server 原生吃 env（加第二設定來源＋優先序歧義，違反 config fail-closed 單一來源設計，且是產品碼改動）；把 .env 交給 server 自己讀 dotenv（同前因）。
**Open**: dev 資料持久化默認與 reset 指令（建議：保留＋dev:reset）；成刀與否（建議：小刀，與 phase2-e2e-chain 零共檔可平行）——皆待使用者確認。

## Conclusion

**Decision**: 開一把小刀 remote-dev-harness——repo root 新增 `npm run dev` 一鍵編排：dev script 讀 .env（gitignored、附 committed 的 .env.example）插值生成 .dev/config.yaml 後以 --config 啟動 speclink-server，並同時起 desktop 的 tauri dev；.dev/ 資料跨重啟持久化（setup/invite/PAT 做一次即可），`npm run dev:reset` 清空重來。env 鍵：SPECLINK_STORE_DRIVER（sqlite|serverfs|postgres|memory，預設 sqlite）、SPECLINK_STORE_PATH、SPECLINK_POSTGRES_URL、SPECLINK_IDENTITY_PATH（identity 固定 sqlite）、SPECLINK_PORT、SPECLINK_PUBLIC_URL（後兩者沿用 compose 既有命名）。正典文件同刀補充：架構 §13.4 補「本地開發啟動」一段（措辭避開「dev server」以繞過 example/dev server 定位條款）、roadmap §4.2 記入此刀（定位 Phase 3 前置基建）。phase2-e2e-chain 不動。
**Rationale**: env 插值放編排層與 deploy/docker-compose.yml 同構——「組態 YAML 不做環境變數展開」的 fail-closed 單一來源決策保持成立，server 產品碼零改動；desktop 現階段雖連不上 server（remote UI 是 Phase 3），但納入編排成本為零，且 harness 本質是 Phase 3 每把刀的開發迴圈前置基建。
**Rejected alternatives**: server 原生吃 env（第二設定來源＋優先序歧義，違反 fail-closed 設計）；docs-only 補充（使用者要的是可執行工具）；e2e 加「無 docker」步驟（phase2-e2e-chain 本來就是真實 binary＋tempdir 的 cargo 測試）；每次啟動全新資料庫（重走 /setup 太煩，改以 dev:reset 顯式重置）。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion remote-dev-harness
