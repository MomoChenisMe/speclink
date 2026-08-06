## Context

過期探測與其消費鏈現況：speclink-core 的 probe_instructions（crates/speclink-core/src/init.rs）回報 InstructionStatus 四態（Missing／Stale／Current／Unknown），判準為標記版號與 MARKER_VERSION 的字串相等；desktop 經 speclink-desktop-core 的 probe_instructions_at 序列化為 camelCase JSON、由 Tauri command 轉交前端，apps/desktop/src/instructionPrompt.ts 依 status 裁決橫幅；CLI 的 update 動詞無任何守門，無條件再生受管檔。--version 只印 CARGO_PKG_VERSION 與架構，MARKER_VERSION 沒有查詢面。正典 workspace-tools「指令檔過期探測」明文「SHALL NOT 解析版本語意」——本 change 推翻該條款，delta spec 同批修訂。

事故鏈（2026-08-05）：v1.11 引擎的 app 對 v1.14 工作區判「過期」、按「更新」靜默降級 30 檔。兩個根因：判準無方向、安裝新鮮度無法機械驗證。

## Goals / Non-Goals

**Goals:**

- 探測分得出方向：檔案比引擎新（newer）與檔案比引擎舊（stale）是不同狀態、不同出口
- desktop 與 CLI 兩個出口對 newer 的預設都是「不動手」
- 任何 binary 的引擎版號一條指令可問（--version）
- 本機安裝收成單一入口、以兩道版號斷言取代信任

**Non-Goals:**

- 不動 updater 自動更新流程；不做降級備份回復；不改 stale／missing／current 語意與略過記憶
- 不保證源碼樹等於 origin 最新（腳本印 HEAD 供操作者確認，保證「安裝版＝這棵樹」）
- CLI 訊息不新增本地化（維持英文輸出）

## Decisions

1. **方向判定落在 speclink-core 的 probe，不在消費端**。probe 是唯一判準來源，desktop 與 CLI 共用同一裁決——只修 desktop 文案的話 CLI 照樣降級（本地路徑限定；remote 分頁不觸發探測，正典既有紅線不動，無 local/remote 雙實作問題）。
2. **InstructionStatus 加變體 Newer，不改既有變體語意**。序列化沿用現行小寫慣例（"newer"）；欄位名與既有值不動，屬向後相容擴充。消費端僅 desktop，前端與引擎同 bundle 出貨、無版本錯開。ToolInstructionState 同步加 per-tool 的方向資訊（布林 newer，serde rename camelCase），聚合與呈現都取自引擎、前端不重算。
3. **版號比較：去 v 前綴、以點拆段、逐段數值比較**。段數不等時缺段補零。任一邊無法完整解析為數字段時，該工具退回現行字串相等判定（不等即 Stale），SHALL NOT 硬排序——手改壞的標記寧可誤報過期（改寫即恢復受管狀態），不可誤報較新（會封鎖 update）。這是 audit 視角的安全預設：守門只在「可證明較新」時觸發。
4. **聚合優先序 Newer > Missing > Stale > Current**。任一工具較新即整體 Newer——與既有「缺失與過期並存回報缺失」同一種一錘定音式聚合，但 Newer 插在最前：只要有任何檔案領先引擎，就不提供任何會改寫它的動作。差異檔清單（differing files）在 Newer 時照常回報（語意不變：update 若執行將改寫的檔案），desktop 用它顯示數字但不掛動作。
5. **守門用專屬旗標 --allow-downgrade，不共用 --force**。--force 現語意是「覆蓋既有檔案」，慣性帶它的使用者不應被靜默視為同意降級；降級是獨立決定，旗標名直說結果（kebab-case 慣例）。同一條理由使 `init --force` 的重建也受守門——把「覆蓋」讀成「同意降級」正是要避免的事。守門時序：判定在任何寫入之前，拒絕即零寫入、單行英文錯誤（含工作區版號與引擎版號）、非零 exit code——無半套狀態問題。

   審查修正（2026-08-06）：守門下沉到引擎的受管檔再生入口（`init::update` 與各 init 進入點），不再只掛在 CLI 動詞——`workflow-config` 的技能足跡同步（CLI 與桌面設定頁）、桌面更新入口、工具選集收斂原本都繞得過。判定目標改取自該次的**實際寫入集**（tools 選集、無清單時的目錄偵測、自訂描述子指令檔），不再依賴只看內建工具的探測面，否則 legacy 與描述子工作區完全不設防。工具選集收斂的方向檢查提前到 `.speclink.yaml` 寫入之前，避免「config 已改、受管檔未同步」的半狀態；`workflow-config` 路徑則沿用其既有的同步失敗形狀（設定已寫入、足跡未同步）。
6. **--version 執行期組字串**。MARKER_VERSION 是 speclink-core 的 const，跨 crate 無法 concat! 進 CLI 的 VERSION 字面量；改為執行期組合（std::sync::LazyLock<String>，clap 的 version 屬性吃表達式），不引新依賴。格式 `<pkg-version> (<arch>, engine <marker-version>)`。
7. **安裝腳本斷言鏈**：scripts/desktop-install.mjs 依序（a）印 HEAD、分支、dirty 狀態與源碼 MARKER_VERSION（從 crates/speclink-core/src/init.rs 的 const 行讀出）；（b）跑既有 scripts/desktop-sidecar.mjs（永遠重建，堵住 src-tauri/binaries/ 殘留舊 CLI 被 externalBin 靜默打包的洞）；（c）vite 建置與 tauri bundle（簽章 env 缺失時單行錯誤停止）；（d）斷言 bundle 內 sidecar CLI 的 --version engine 版號等於源碼 MARKER_VERSION；（e）帶 --install 時確認 app 未執行（執行中則單行錯誤，不代關）、覆蓋 /Applications/Speclink.app、再斷言安裝版 CLI 同版。GUI binary 的內嵌引擎與 sidecar CLI 分開編譯，但同一次執行同一棵樹，斷言 sidecar 即涵蓋樹狀態。安裝步驟僅支援 macOS，非 macOS 帶 --install 即單行錯誤；建置階段平台中立。
8. **desktop 較新提示沿用既有提示骨架**：instructionPrompt.ts 的 kind 加 "newer"，略過記憶同鍵值（專案路徑 → 產物層版號）照用——app 升版後 MARKER_VERSION 變動自然重新探測，提示自然消失或轉為 stale。i18n 新增 newer 標題與描述鍵（apps/desktop/src/i18n/messages.ts），文案遵循 LANGUAGE.md、不出現工程詞。

## Implementation Contract

- **行為**：
  - 工作區任一工具檔案的標記版號數值大於引擎版號時，探測回報整體 "newer"；desktop 顯示「你的 app 是舊版」語意的非阻斷提示，僅「保留現狀」一個動作，無任何改寫檔案的動作；經引擎再生入口的每一條路徑（`speclink update`、`speclink init --force`、工具選集收斂、`workflow-config` 的技能足跡同步、桌面更新動作）皆拒絕執行（單行含兩個版號、exit code 非零、零檔案寫入），`speclink update --allow-downgrade` 照常再生。
  - `speclink --version` 輸出 `<pkg-version> (<arch>, engine <marker-version>)`；stale／missing／current／unknown 的既有行為與輸出不變。
  - `node scripts/desktop-install.mjs` 建置並斷言；`--install` 附加安裝與安裝後斷言；任一斷言失敗印出兩邊版號並以非零結束。
- **介面／資料形狀**：InstructionStatus 序列化值集合擴為 "missing" | "stale" | "current" | "unknown" | "newer"；ToolInstructionState 增 `newer: bool`（camelCase JSON）；UpdateArgs 增 `--allow-downgrade` 旗標；VERSION 改為執行期組字串。均為擴充，不改既有欄位名與值。
- **失敗模式**：版號無法解析→該工具退回字串相等（Stale），絕不判 Newer；update 守門拒絕＝零寫入無半套；安裝腳本簽章 env 缺失、app 執行中、非 macOS 帶 --install、斷言失敗→各自單行錯誤非零結束，不繼續後續步驟。
- **驗收**：cargo test -p speclink-core（probe 方向與聚合案例）、cargo test -p speclink-cli（update 守門與 --version 格式）、npm test -w apps/desktop（instructionPrompt 裁決與橫幅呈現）、手動跑安裝腳本一次驗斷言鏈；render_golden:: 全綠且 golden 零變動（不動 assets 與 marker 模板）。
- **範圍邊界**：in——probe 方向、desktop newer 提示、update 守門、--version、安裝腳本；out——updater 流程、降級備份、其他狀態語意、CLI 本地化、origin 同步保證。

## Risks / Trade-offs

- **回歸對照**：--version 人眼輸出刻意改變——先掃 crates/speclink-cli/tests/ 是否有釘住舊格式的測試，同批更新並在此列為刻意變更；probe JSON 僅加值加欄，desktop-core 的 camelCase 契約測試同批擴充。不動 assets／marker render，MARKER_VERSION 不推進、golden 與 assets.lock 零變動（誤動即 render_golden:: 紅燈，本身就是回歸訊號）。
- **跨平台**：版號比較純字串數值運算，無平台差異；安裝腳本安裝步驟明文 macOS 限定，建置步驟平台中立；換行正規化沿用既有 eol_normalized，Windows CRLF 誤報防線不動。
- **serde 相容**：舊版 desktop 前端遇到 "newer" 值的風險不存在（同 bundle 出貨）；反向（新前端讀舊引擎）同理。CLI --json 無任何指令輸出 probe，無外部消費者。
- **誤判權衡**：無法解析的版號判 Stale 而非 Newer——可能把手改壞的工作區標成可更新（改寫即恢復），換取 update 永不被假 Newer 封鎖；反向誤判（假 Newer）的代價是守門誤攔，需 --allow-downgrade 越過，兩害相權取其輕。

## Open Questions

（無——旗標名與腳本斷言鏈已在本文件定案，實作層無待決事項。）
