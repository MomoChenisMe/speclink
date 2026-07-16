## 1. protocol DTO 與映射

- [x] 1.1 【紅→綠】涵蓋「wire 與引擎型別映射單點往返」，分兩層落點：(a) protocol 新增 drift 模組，放規格面報告（維度＋規格假設）、basis digests 與該 change 的 store 面輸入（created／design／tasks，缺席與空內容以 Option 區別）的 wire DTO——camelCase、JSON Schema 匯出、serde 往返測試；DTO 無 broken anchors 等工作區/git 欄位；protocol 不得依賴 speclink-core。(b) speclink-host::drift 新增 wire↔引擎型別的單點雙向映射，往返結構相等測試（樣本含多筆規格假設：ADDED／MODIFIED／RENAMED 各式 reason）；core 型別不加 serde 標註。驗收：cargo test -p speclink-protocol 與 cargo test -p speclink-host 全綠。 <!-- speclink-task:tsk_01KXMA384RYNS4KVHTPTEVFP21 -->

## 2. server 端點

- [x] 2.1 【紅】針對「規格面 drift 端點且工作區面不進 wire」寫路由測試：對含 delta specs 與 design 的 change 回規格面維度、規格假設與 basis digests；回應結構無工作區/git 欄位（含 broken anchors，以 DTO schema 斷言）；未知 change 404；未認證/非成員 401/403；store 失聯 503；計算不產生 outbox 事件。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXMA384R369KD09ZVXRWAY08 -->
- [x] 2.2 【綠】實作 drift 端點：speclink-host::drift 新增專用查詢入口 spec_drift(store: &dyn TeamStore, scope, change) -> SpecDriftView{spec, basis, created, design, tasks}——內部取單一 store snapshot（沿 context-api 模式）materialize 私有橋接視圖、跑引擎 compute_spec_drift、由同一 snapshot 算 basis digests 與讀 store 面輸入一起回傳，橋接 Store 視圖不對 host 外公開；server 路由呼叫該入口並以 1.1(b) 的映射回 wire DTO。2.1 全綠。 <!-- speclink-task:tsk_01KXMA384RNWEK7EC31A0DQ766 -->

## 3. client 合併與輸出凍結

- [x] 3.1 【紅→綠】typed client 新增 drift 方法（既有請求骨架與錯誤翻譯）；以 stub server 測試回應反序列化為 wire DTO，並經 1.1(b) 的 host 映射轉回引擎型別。host 另加唯讀最小 Store adapter（以 store 面輸入服務 design/tasks 內容與存在性、created；其餘 store 表面 unreachable!），測試涵蓋缺席 design 不偽裝成空內容。 <!-- speclink-task:tsk_01KXMA384RHKANB5ZRSVT8MK4Z -->
- [x] 3.2 【紅】針對「remote drift 合併於 client 且輸出凍結」寫測試：同一 change 內容 fs 模式與 remote 模式（本機 checkout）的 drift --json 輸出結構同形且規格面內容相同；無 checkout（git 不可用）下 remote drift 成功且工作區面走三值語意的不可得（WorkspaceDimension::Unavailable，coverage 為 spec-only、四維度不計入總分）、無「乾淨」斷言——不可走 Some(facts) 路徑，否則 path 錨點會 stat 失敗而被誤報為 broken；server 503 時動詞以既有錯誤訊息失敗、不輸出部分報告。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXMA384RB6B26GRXNXNP1J16 -->
- [x] 3.3 【綠】remote 攔截層接上 drift 動詞：取規格面、basis 與 store 面輸入 → 以 3.1 的 adapter 包成 Store → 本機 collect_workspace_facts＋compute_workspace_drift（可得時）→ merge_drift_reports 合併（basis 以 DriftBasis{expected: b, current: b} 餵入——規格面與 basis 出自同一 snapshot 故恆等，stale 為 None，與 fs 模式一致）→ 與 fs 模式同一渲染函式輸出。3.2 全綠。 <!-- speclink-task:tsk_01KXMA384RX4P0A9XTSH5Q44W9 -->

## 4. 回歸

- [x] 4.1 執行 npm run test:all 確認全 workspace 回歸。fs 模式 drift 輸出零變更由 core 既有的兩個凍結 oracle 守住：drift::tests::full_coverage_merge_equals_current_drift_report_field_for_field 與 git_unavailable_facts_match_current_analyze_fallback_byte_for_byte（對 analyze 舊實作逐位元）。remote 對 stub 的欄位形狀 parity 由 speclink-cli 的 remote_read_path／remote_write_path、server 的 e2e_cli 守住。驗收：test:all exit 0 且零失敗，上述套件全綠。（原描述的「parity 31 項、color 16 項、twin 8 情境」在本 repo 無對應具名套件，已改為實際驗證目標。） <!-- speclink-task:tsk_01KXMA384RE1D39KCNDHF3DWV5 -->
