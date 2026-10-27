## Context

speclink-server 的測試已有可複用的地基：真 server 啟動 helper（tempdir 組態、setup token 解析、就緒等待）、真 CLI binary 定位、identity 播種、SSE 訂閱測試（sse_events 刀）、投影斷言（context-api 刀）、備援演練（backup 刀）。各刀 e2e 的斷言面互不相連：setup e2e 停在「invite 後 CLI 可連線」、命令路由 e2e 各自播種、drift（server-drift-api 刀落地後）與 archive 各驗各的。架構 §14 Phase 2 第 5 項要求八環節（propose、task、policy、context、evidence、drift、archive、event recovery）以 CLI 端到端連續驗證；§9.2 的收斂規則（漏事件以 Polling＋ETag 恢復、重訂閱自最新序號）已各自有測試但未在真實工作流中演練。

## Goals / Non-Goals

**Goals:**

- 一條劇本從空資料庫走到 archive，環節之間的縫（帳號→動詞、policy→instructions、evidence→事件、archive→正典）被連續驗證。
- event recovery 不是獨立測試而是伴隨劇本的訂閱者視角：漏事件、收斂、重訂，最終視角與正典一致。
- 失敗即可診斷：步驟名、server stderr、workspace 現場一次報出。

**Non-Goals:**

- 不修劇本揭露的產品 bug（開獨立 change，本刀維持純測試）。
- 不做效能/負載量測（劇本是正確性驗收不是壓測）。
- 不對 serverfs/postgres driver 重複整鏈（driver 等價由 conformance 承擔；劇本固定 SQLite——與預設部署形態一致）。
- 不做 Docker 形態的劇本（容器冒煙屬 server-release-packaging 刀；本刀對 cargo 起的 server 跑，兩者互補）。
- 不動 twin harness 既有 8 情境與任何分刀 e2e（保留各自的快速定位價值，不合併）。

## Decisions

### 決策 1：單一 #[test] 劇本、步驟顯性編號

整條鏈是一個測試函式內的順序步驟（helper 函式每步一個，名稱即步驟名），不是八個獨立 #[test]——獨立測試各自播種就驗不到縫。步驟失敗以 panic 訊息攜步驟編號與名稱；測試 harness 在失敗時傾印 server stderr 尾段與 workspace 目錄樹。約 500 行上限的紀律：斷言細節下沉到 helpers，劇本主體保持可讀的流程敘事。

### 決策 2：policy 環節以「政策改變 → instructions 可觀察變化」驗證

步驟 (4) 的斷言不是讀回 config 而是行為：寫入 workflow config 一條可觀察的政策差異（`locale` 政策鍵 → apply instructions 的 `locale` 欄位）→ 以 CLI 取 instructions → 斷言輸出反映該政策；再改回 → 斷言恢復。這驗證 policy 從 store 經 host 的 EffectiveWorkflowPolicy 到 instructions 渲染的整條縫，而非只驗儲存往返。

寫入機制：remote 模式今日沒有 CLI/wire 的 workflow config 寫入面（`/config` 僅 GET；`speclink config` 寫的是全域 CLI 設定），劇本沿既有測試播種縫隙以第二條 store 連線直寫同一 SQLite 檔（WAL＋busy timeout 允許與運行中 server 並行）；被驗證的縫（store → policy 解析 → server 端 instructions 渲染 → CLI 輸出）不變。config 寫入面缺口另列候選 change，不屬本刀。

### 決策 3：evidence 環節同時驗事件與記錄兩面

步驟 (5) task done 帶 touched files 後雙斷言：outbox（經訂閱者收到 task-completed）與 evidence 查詢面（server 端 TouchedRecord 可查、欄位含 taskId/actor/touchedFiles）。這是「寫入 → 事件 → 證據」三位一體的縫，分刀測試各驗過兩兩、未驗過三連。

### 決策 4：event recovery 是伴隨訂閱者，不是插曲

劇本開場（步驟 3 前)建立 SSE 訂閱者記錄事件流；在步驟 (5) 與 (6) 之間強制斷其連線、讓步驟 (6)-(7) 的事件漏掉；訂閱者以 Last-Event-ID 重連——若序號已被保留政策清理則收 reset、走 /sync-state＋查詢全量收斂，否則續傳補齊；步驟 (8) archive 後斷言訂閱者累積視角（事件去重後的最終狀態認知）與直接查詢的正典一致。兩條恢復路徑（續傳/reset）至少各被一次劇本配置覆蓋（以保留筆數組態控制走哪條）。

### 決策 5：劇本進 CI 必跑，與分刀 e2e 同 job

phase2_chain 測試在既有 cargo test -p speclink-server 路徑內（CI 已必跑），不開獨立 job——它依賴的只有 CLI/server 兩個 binary 與 tempdir。若執行時間顯著（>60s）再以測試分組標註，屆時於 ci.yml 顯性列出而非靜默 ignore。

## Implementation Contract

- Behavior：cargo test -p speclink-server phase2_chain 在乾淨機器上（無外部服務）單測全綠；任一環節壞掉時失敗訊息指出步驟名並附 server stderr 尾段。
- Interface / data shape：單一 #[test] 加步驟 helpers；訂閱者 helper 回放事件流供斷言；劇本組態沿用既有測試組態產生器（SQLite tempdir）。
- Failure modes：步驟失敗 → panic 帶步驟編號/名稱＋現場傾印；SSE 收斂失敗（視角與正典不一致）→ 斷言列出差異文件清單；環境缺 CLI/server binary → 測試以明確訊息失敗（沿用既有 binary 定位 helper 的行為）。
- Acceptance criteria：劇本全綠且八環節與兩條恢復路徑皆有斷言；故意注入一處壞斷言驗證失敗訊息含步驟名（開發期驗證後移除）；npm run test:all 全綠且既有凍結零 diff。

## Risks / Trade-offs

- 單一長劇本的失敗定位成本 → 步驟編號＋現場傾印是驗收條件；分刀 e2e 保留作快速定位層。
- 劇本對輸出形狀的斷言可能與未來刀的合法變更打架 → 斷言鎖語意（存在性、一致性、計數）不鎖全文位元，位元級凍結仍由 parity/twin 承擔。
- 兩條恢復路徑靠保留筆數組態控制 → 組態值在劇本內顯性宣告並註解對應路徑，不依賴巧合。
- 劇本揭露縫隙 bug 時本刀不能收 → 紀律上開 bug-fix change；劇本以 #[ignore] 暫標該步並附 change 名，不留假綠。

## Migration Plan

前置：server-drift-api 已歸檔（步驟 7）。與 server-release-packaging 平行。落地後劇本即 Phase 2 的常駐驗收面；Phase 3 desktop 刀動 server 消費面時以它作整鏈回歸。回退即刪測試檔。

## Open Questions

（無）
