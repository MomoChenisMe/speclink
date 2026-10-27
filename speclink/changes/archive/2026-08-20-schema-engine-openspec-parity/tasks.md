## 1. 單一正典載入

- [x] 1.1 先寫紅測（單一正典載入）：內建 schema 與正典 YAML 的欄位一致——spec_driven() 回傳的 name、description、apply 區塊與四個 artifact 的中繼資料同於正典 YAML；四個 artifact 的 template 內容與對應內嵌 template 資產逐字相同。驗證：cargo test -p speclink-core 出現預期紅燈 <!-- speclink-task:tsk_01M0CK35MNGKEPBSK1PER5T2PR -->
- [x] 1.2 實作 D1 單一正典載入：spec_driven() 保名保簽名，函式體改為 OnceLock 快取「解析內嵌 fork.schema.yaml ＋依 template 檔名附掛內嵌 template」；刪除手寫的內建定義；builtin_template() 改查快取結果。驗證：1.1 測試轉綠且 cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M0CK35MN1ASCK758VN3FYRYG -->
- [x] 1.3 同源與 description 行為測試（單一正典載入）：schema fork spec-driven 產出的 schema.yaml 與正典 YAML 逐位元組相同；speclink schemas 顯示的內建 description 為正典字面（含 design optional 字樣）。驗證：cargo test -p speclink-cli 對應測試轉綠 <!-- speclink-task:tsk_01M0CK35MNDS6ZQ4CW462YF82P -->

## 2. instruction 單一來源

- [x] 2.1 先寫紅測（instruction 單一來源）：正典 YAML 的 specs instruction 含三段現行內容——Purpose section (new capabilities only) 段、BEFORE 註記步驟、REMOVED-SCENARIO 合併門檻。驗證：cargo test -p speclink-core 出現預期紅燈 <!-- speclink-task:tsk_01M0CK35MN9DQEPSDNGG0806VR -->
- [x] 2.2 實作 D2 飄移收斂：以 crates/speclink-core/assets/schema/spec-driven/specs.instruction.md 全文整檔覆蓋正典 YAML 的 specs instruction 區塊（其餘四份已逐位元組相同）；刪除五份 instruction 資產檔與其 include_str! 常數；既有的雙份標記同步測試改為只釘正典 YAML。驗證：2.1 轉綠；speclink instructions specs 的 instruction 欄位與改動前輸出逐字相同（改動前先存快照檔比對） <!-- speclink-task:tsk_01M0CK35MNWTEDVEGQRTG4K84E -->

## 3. schema 驗證規則

- [x] 3.1 先寫紅測（schema 驗證規則，表驅動）：重複 artifact id、懸空 requires（指名 artifact 與缺席 id）、循環相依訊息含完整環路徑（a → b → a）、version 缺席與 0 與 1.5、description 鍵缺席、description 空字串通過、template 鍵缺席與空字串——各案例的載入錯誤與訊息斷言。驗證：cargo test -p speclink-core 出現預期紅燈 <!-- speclink-task:tsk_01M0CK35MNQBY9EQXR8S6X87FR -->
- [x] 3.2 實作 D3 驗證強化：載入口補六項檢查（內建與自訂同一驗證函式）；循環檢測改回傳完整環路徑；version 必填正整數、description 鍵必填、template 鍵必填非空並移除以 artifact id 推導預設檔名的容錯。驗證：3.1 轉綠；內嵌正典 YAML 通過全部檢查且 cargo test --workspace 不因內建載入失敗而紅 <!-- speclink-task:tsk_01M0CK35MNEA0TDX655Z11HEAB -->
- [x] 3.3 名稱格式守門（schema 驗證規則）：schema fork 與 schema init 的目的名稱不符小寫 kebab-case 時以非 0 exit code 拒絕並說明格式。驗證：新增整合測試於 crates/speclink-cli/tests/it/schema_commands.rs，含 speclink schema init My_Schema 退出碼非 0 案例 <!-- speclink-task:tsk_01M0CK35MNQJE7T1H30YJ7MND0 -->

## 4. schema 指令旗標

- [x] 4.1 先寫紅測（schema 指令旗標，CLI 整合）：which --all 列出內建與自訂各自的解析位置與來源層級；validate 對缺席 template 檔非 0 退出並指名檔名；validate --verbose 逐項印出驗證步驟；init --default 寫入 schema 鍵且其餘內容逐位元組保留、config.yaml 缺席時建立僅含 schema 鍵的檔案。驗證：cargo test -p speclink-cli --test it 出現預期紅燈 <!-- speclink-task:tsk_01M0CK35MN9KRQNY00DGQ3ZBDK -->
- [x] 4.2 實作 D4 旗標與 config 寫入：toolchain.rs 落實三旗標行為；validate 本體補 template 檔存在檢查（自訂查 templates 目錄、內建查內嵌附掛表）；config.rs 新增 byte-preserving 的 schema 鍵 setter（寫後重讀驗證，無法解析的 config.yaml fail closed 拒寫且訊息明說骨架已建、預設未設）。驗證：4.1 轉綠 <!-- speclink-task:tsk_01M0CK35MNC6ZVXT2J18KMSPE2 -->

## 5. init 骨架可載入

- [x] 5.1 先寫紅測（schema init 產出可載入的骨架）：speclink schema init my-flow（不帶 --description）後，schemas 列得出 my-flow 且 schema validate my-flow 以 exit code 0 通過；templates 目錄含每個 artifact 宣告的 template 檔。驗證：cargo test -p speclink-cli --test it schema_commands 出現預期紅燈 <!-- speclink-task:tsk_01M0D5AWPBGYR02QFX1ZR98WZ6 -->
- [x] 5.2 實作骨架修補：init_schema 的 schema.yaml 純量值改走 YAML 序列化（消除未跳脫的 description），並為每個 artifact 產出 templates/<template>.md 骨架檔。驗證：5.1 轉綠 <!-- speclink-task:tsk_01M0D5HHEMHBM9SMDPNZ7D2P1C -->

## 6. 收尾

- [x] 6.1 全量回歸與孤兒清理：cargo test --workspace 全綠；grep 確認無殘留對已刪 instruction 資產檔的引用；speclink validate schema-engine-openspec-parity 通過。驗證：三項指令輸出全部通過 <!-- speclink-task:tsk_01M0CK35MNGBHSHF0FZ50D4N04 -->
