## 1. 契約文件定稿

- [ ] 1.1 撰寫 `docs/verb-contract.md`（英文正典）與 `docs/verb-contract.zh-TW.md`：全部動詞端點、request/response payload（camelCase 與既有 --json 對齊）、409 reason 列舉值與範例 body、If-Match 樂觀並行、X-Speclink-Api-Version 與 X-Speclink-Repo header、認證方式；逐項對照 wadpilot docs/sdd-research/04-speclink-final-design.md §5.3 的 response 形狀，差異處標注理由；明載 change 的 repo 歸屬規則（v1 一 change 一 repo：建立時取自請求 repo、列舉依 repo 過濾、跨 repo 需求拆分為多個 change）；明定狀態轉移與 409 reason 的裁決語意為契約正典（各 host 不得自行變體，未來由選用的 speclink-team 模組承載參考實作——見討論第 17 輪）；標注 server 實作自由度僅限 gate 政策設定與 repos 註冊表管理
- [ ] 1.2 驗證：契約文件通過內部審閱清單——每個 CLI remote 動詞都有對應端點、每個 409 reason 都有 CLI 建議動作、無未定義錯誤路徑

## 2. speclink-remote crate（請求層與認證儲存）

- [ ] 2.1 撰寫測試：於 `crates/speclink-remote/tests/` 以 dev-dependency 極簡 mock server 斷言——請求攜帶 Authorization／X-Speclink-Repo／X-Speclink-Api-Version header、401/403/404/409（各 reason）/5xx/連線失敗的錯誤翻譯結果、SPECLINK_TOKEN 優先於憑證檔、憑證檔寫入位置與 Unix 0600 權限——紅燈
- [ ] 2.2 實作使測試轉綠：新增 `crates/speclink-remote/Cargo.toml`（ureq、serde；加入 `Cargo.toml` workspace members）、`crates/speclink-remote/src/client.rs`（單一請求層＋逐動詞路徑映射）、`crates/speclink-remote/src/auth.rs`（憑證檔讀寫、token 解析順序）、`crates/speclink-remote/src/lib.rs`（錯誤翻譯層：非 2xx → 語義化訊息與建議動作，絕不裸狀態碼）
- [ ] 2.3 重構：錯誤翻譯表集中單處，cargo clippy 無新警告；驗證 cargo test -p speclink-remote 全綠（覆蓋需求：API 版本協商與錯誤翻譯紅線、PAT 登入與憑證儲存、憑證失效的處理）

## 3. 模式解析與 CLI 讀路徑路由

- [ ] 3.1 撰寫測試：`crates/speclink-core/src/workspace.rs` 的模式解析（無連接檔＝fs、有＝remote、並存警告一行、SPECLINK_STORE_URL 覆寫）與連接檔 YAML 解析（url 必填、repo 選填、缺 url 報語義化錯誤）——紅燈
- [ ] 3.2 實作：workspace.rs 增加連接檔偵測與解析；`crates/speclink-cli/src/main.rs` 與 `crates/speclink-cli/src/commands.rs` 指令進入點依模式路由，先接讀路徑動詞（speclink list、speclink status、speclink instructions、speclink discuss list/show）走 remote client——綠燈
- [ ] 3.3 驗證：mock server 整合測試斷言讀路徑動詞的 stdout JSON 欄位名與 fs 模式一致；fs 模式 cargo test 全綠無回歸（覆蓋需求：連接檔與模式解析、動詞契約的涵蓋面與 payload 形狀）

## 4. 寫路徑動詞與 repo 驗證鏈

- [ ] 4.1 撰寫測試：寫路徑動詞（speclink new change、speclink new artifact 帶 If-Match、speclink task done、speclink discuss new/context/add-round/conclude、speclink claim、speclink archive）的成功與 409 各 reason 情境；claim 的 repo_mismatch 訊息同時含兩個 repo 名；change 歸屬情境（new change 歸屬當前 repo、list 依 repo 過濾使他 repo 的 change 不出現）——紅燈
- [ ] 4.2 實作：寫路徑動詞接上 client（artifact 寫入攜帶讀取時版本；每請求帶 X-Speclink-Repo）——綠燈
- [ ] 4.3 驗證：整合測試全綠；手動以 mock server 走一輪 list → claim → task done → archive 流程（覆蓋需求：樂觀並行控制與 409 語意、repo 身分攜帶與歸屬防呆、change 的 repo 歸屬規則）

## 5. remote 初始化、link/unlink 與 auth 子指令

- [ ] 5.1 撰寫測試：speclink init --store remote --url --repo（生成 marker/技能/連接檔、不建 openspec/ 樹）、speclink link（有憑證時即時 whoami 驗證 repo∈專案、驗證失敗不寫檔並列可用名單、無憑證時提示 auth login）、speclink unlink（移除連接檔）、speclink auth login/status（含未登入非 0 exit code）；link 與 auth status 的 git remote 參考值輔助比對（fork 情境警告一行且結果與 exit code 不受影響、無參考值或非 git 目錄時靜默）——紅燈
- [ ] 5.2 實作：`crates/speclink-core/src/init.rs` remote 分支（workspace init＋連接檔，跳過 store init）；`crates/speclink-cli/src/commands.rs` 新增 link、unlink、auth login、auth status 子指令，link 與 auth status 加入 git remote 參考值輔助警告（server 提供參考值時比對本地 git remote）——綠燈
- [ ] 5.3 驗證：cargo test 全綠；於暫存目錄實跑 init --store remote 確認檔案效果與提示訊息（覆蓋需求：remote 初始化與連接指令、git remote 參考值的輔助警告）

## 6. 文件讀取動詞、技能動詞化與 marker 變體

- [ ] 6.1 撰寫測試：speclink artifact cat 與 speclink language show 的兩模式行為（fs 讀檔、remote 走端點、缺文件非 0 exit code 與語義化訊息）；技能資產掃描斷言（全部 SKILL.md 不含直接讀取規格目錄檔案的指示）；marker remote 變體 golden（不含 openspec/ 路徑句、含動詞指引）——紅燈
- [ ] 6.2 實作：新增 artifact cat 與 language show 動詞（雙模式）；更新 `crates/speclink-core/assets/skills/` 全部技能資產的讀檔指示為動詞（含 discuss 的詞彙載入、propose 的依賴閱讀）；`crates/speclink-core/src/init.rs` 的 marker 渲染增加 store 維度——綠燈
- [ ] 6.3 驗證（回歸對照刻意更新）：fs 模式 golden 因技能措辭更新而刻意重錄並記錄變更清單；parity／color／twin 全數通過（覆蓋需求：store 文件讀取動詞、指令區塊的 remote 變體）

## 7. 團隊模式文件

- [ ] 7.1 撰寫 `docs/team-mode.md` 與 `docs/team-mode.zh-TW.md`：連接檔與模式解析、init/link/auth 流程、repo 識別三層運作、錯誤訊息對照表、自純本地情境升級的手動遷移指引（store push 尚未提供的替代步驟）；`README.md` Documentation 章節增列雙語連結
- [ ] 7.2 驗證：README 引用路徑存在；cargo build --release 成功；speclink --version 正常
