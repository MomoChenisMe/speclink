## Context

前三個 change 已完成：引擎經 Store 介面存取文件（object-safe、同步）、政策歸屬 store 側且渲染矩陣具備（目標 × 措辭 × store 模式）、動詞契約與 remote client 就緒。wadpilot 的最終設計（docs/sdd-research/04-speclink-final-design.md）期待一個 @speclink/engine 供 server 內嵌：以 Postgres/Y.js 實作 storage ports、以 in-process CopilotTool 讓 agent 呼叫引擎。本 change 交付該套件。約束：speclink-core 維持同步、無 async runtime；JS 宿主的 Store 實作天然是 async（資料庫呼叫）；Node 事件迴圈不可被阻塞。

## Goals / Non-Goals

**Goals:**

- npm 套件 @speclink/engine：createEngine、dispatch(argv)、instructions.render、skills.list/render，附 TypeScript 型別。
- 宿主 JS Store（方法可回傳 Promise）與 Rust 同步 Store 介面之間的橋接。
- 各平台預編譯二進位的發佈管線（Windows/macOS/Linux × x64/arm64）。
- SDK 整合雙語文件與教學（Copilot SDK defineTool 範例、Store 實作指南）。

**Non-Goals:**

- 不做 Python/Go/.NET 等其他語言綁定（Rust 消費者直接用 core crate；其他語言待真實需求）。
- 不做 MCP server 封裝（宿主可自行以本 SDK 包裝；wadpilot 情境已定 in-process tool）。
- 不含 crates.io 發佈流程調整（Rust SDK 即既有 crate）。
- 被否決方案（見討論記錄）：WASM 作為主形式（JS 實作 Store 的 async callback 橋接體操差、napi-rs 平台預編譯成熟）；TS 重刻引擎（雙引擎漂移）；async Store trait（tokio 感染 core）。

## Decisions

1. **napi-rs 綁定、核心同步、橋接在邊界**
   - dispatch 一律以 napi 的背景工作（AsyncTask/tokio-free worker thread）執行，回傳 Promise——JS 事件迴圈不被阻塞。
   - JS Store 方法以 ThreadsafeFunction 呼叫：Rust 側（工作執行緒）發出呼叫並以 channel 阻塞等待 JS Promise 解析；因 dispatch 永不在 JS 主執行緒上同步執行，無自我死結路徑。此橋接使 core 的同步 Store 介面原樣可用。
   - 替代案：把 Store 介面改 async——被否決（紅線）；在 JS 主執行緒同步 dispatch——被否決（會與 ThreadsafeFunction 回呼互等死結）。
2. **createEngine 的雙形式儲存**
   - createEngine({ store: { type: 'fs', root, specDir? } })：內部直接使用 speclink-fs（零橋接成本，供本地工具或測試）。
   - createEngine({ store: <JS 物件> })：物件需實作 Store 介面的對應方法（TypeScript 介面於 index.d.ts 定義；方法回傳值或 Promise 皆可）；缺方法在建構時即報錯（fail fast，列出缺少的方法名）。
3. **dispatch 的輸入輸出契約**
   - 輸入：string[]（與 CLI 動詞詞彙一對一，等同 shell 的 argv 去掉程式名）；不支援互動式輸入——需要內容的動詞以 argv 選項或 payload 欄位傳遞（等同 CLI 的 --stdin 內容以參數物件第二形式傳入：dispatch(argv, { stdin?: string })）。
   - 輸出：解析後的 JSON 物件，欄位與 CLI `--json` 完全一致（camelCase）；無 --json 形式的動詞回傳 { output: string }。
   - 錯誤：以 Error 拋出——message 為與 CLI 相同的語義化訊息，附 code 欄位（對應 exit code 與 409 reason 類別），宿主的 tool handler 可直接把 message 回給 agent。
   - 替代案：回傳 { exitCode, stdout, stderr } 原始三元組——被否決：SDK 消費者要的是結構化資料，不是再解析一次文字。
4. **渲染 API 直通渲染矩陣**：skills.list() 回傳技能名與描述；skills.render(name, { target: 'claude'|'codex'|'neutral', invocation: 'cli'|'tool-call', store: 'fs'|'remote' }) 與 instructions.render({ 同參數 }) 回傳字串——與 CLI 生成共用同一渲染程式碼，保證內容一致。
5. **套件與發佈**：napi-rs 標準佈局（主套件＋各平台 optionalDependencies 子套件）；CI 於 GitHub Actions 建置五個目標（win-x64、darwin-x64、darwin-arm64、linux-x64-gnu、linux-arm64-gnu）。部署注意：wadpilot 的 Linux server 需對應平台預編譯——文件明載（回應 04「純 Node 領域邏輯」前提與 native module 的差異：需要平台二進位，但無系統依賴、npm install 即用）。
6. **命名慣例**：npm API camelCase（createEngine、dispatch、skills.render）；Rust 綁定內部 snake_case；TypeScript 介面 PascalCase（Engine、Store、RenderOptions）。

## Implementation Contract

- **行為**：
  - const engine = createEngine({ store: { type: 'fs', root } }); await engine.dispatch(['list', '--json']) 回傳與 CLI 於同一專案執行 speclink list --json 相同的物件。
  - 以 JS 物件 Store（模擬兩個 change 的資料）建構後 dispatch(['list','--json'])，回傳該兩個 change 且欄位一致；Store 方法回傳 Promise 亦成立。
  - dispatch(['claim','x']) 於 Store 回報衝突時拋出 Error，message 含語義化訊息、code 反映衝突類別。
  - skills.render('propose', { target: 'neutral', invocation: 'tool-call', store: 'remote' }) 回傳的字串以「呼叫 speclink 工具」措辭且不含本地路徑句；與 CLI 對等參數生成的內容一致。
  - createEngine 缺必要 Store 方法時同步拋錯並列出缺少的方法名。
- **介面／資料形狀**：index.d.ts 定義 Engine、Store（方法集與 core 介面一對一，回傳 T | Promise<T>）、RenderOptions、DispatchOptions（stdin）；dispatch 回傳型別為 unknown（依動詞而異）附型別註記文件。
- **失敗模式**：JS Store 方法拋出或 reject → dispatch 以 Error 傳遞原訊息（附 store 方法名前綴）；dispatch 於背景執行緒 panic → 轉為 Error 而非行程中止。
- **驗收條件**：
  - Node 測試（crates/speclink-node/__test__/engine.spec.ts）：上述行為逐條斷言；fs 形式與 CLI 輸出對照（同一 fixture 專案跑 CLI 與 SDK 比對 JSON）。
  - 橋接壓力測試：連續 dispatch 與並發 dispatch 各百次無死結、無記憶體異常增長（粗檢）。
  - cargo test 全 workspace 綠；npm test 綠；五平台 CI 建置成功。
- **範圍邊界**：in scope＝綁定 crate、Store 橋接、dispatch、渲染 API、型別定義、發佈管線、SDK 文件教學；out of scope＝server 端契約實作、其他語言綁定、MCP 封裝、CLI 行為變更。

## Risks / Trade-offs

- [ThreadsafeFunction 橋接死結] → 鐵律：dispatch 永遠在背景工作執行；以並發壓力測試覆蓋；文件警告宿主勿在 Store 方法內同步呼叫 engine。
- [平台預編譯矩陣維護成本] → 限定五個主流目標；其餘平台以 npm 安裝時源碼編譯 fallback（napi-rs 支援）文件註明需 Rust 工具鏈。
- [SDK 回傳型別鬆散（unknown）] → 文件逐動詞列出 payload 形狀（連結 verb-contract 文件）；後續版本再考慮逐動詞型別。
- [與 CLI 輸出漂移] → SDK 與 CLI 共用同一 core 序列化路徑，且以「同 fixture 對照測試」鎖住。

## Migration Plan

1. 綁定 crate 骨架與 fs 形式 createEngine＋dispatch（讀路徑動詞），Node 測試對照 CLI。
2. JS Store 橋接（ThreadsafeFunction＋channel），行為與錯誤傳遞測試。
3. 寫路徑動詞與 stdin 參數形式；壓力測試。
4. 渲染 API 與 golden 對照。
5. index.d.ts 型別與 npm 佈局、CI 預編譯管線。
6. SDK 雙語文件與教學（Copilot SDK defineTool 範例、Store 實作指南、部署注意）。

## Open Questions

（無——形式選型、橋接方向、輸出契約皆由討論記錄第 2、4、12、13 輪定案。）
