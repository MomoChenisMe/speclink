## Context

speclink-core 的每個流程模組（model、discuss、archive、status、validate、analyzer、drift、inprogress、tasks、newcmd）都直接以 std::fs 讀寫 `openspec/` 下的規格文件；佈局知識（路徑組裝、目錄列舉、排序用 mtime）集中在 `crates/speclink-core/src/paths.rs` 與各模組內。`crates/speclink-fs` 是空殼 crate（不在 workspace members）。專案紅線要求：core 不得把儲存假設寫死進引擎、既有 CLI 輸出是回歸保護對象（parity_suite 31 項、color_suite 16 項、twin harness 8 情境）、跨平台（Windows/macOS/Linux）。本 change 源自十六輪討論的第①刀，是後續設定體系（config-system-rework）、動詞契約（verb-contract-and-remote-client）、Node SDK（node-sdk）三個 change 的地基。

## Goals / Non-Goals

**Goals:**

- 在 speclink-core 定義領域層級的儲存介面（以 change／artifact／discussion／spec／workflow-config 為語彙），引擎對規格文件的所有存取一律經由它。
- speclink-fs 成為此介面的預設實作，持有現行 openspec/ 佈局的全部知識，並加入 workspace。
- 對外行為位元級不變：三套回歸對照全數通過是硬性驗收條件。
- 建立中英雙語文件骨架（架構篇、入門篇）與 README 引用。

**Non-Goals:**

- 不做任何行為變更、不新增 CLI 指令或旗標。
- 不做遠端儲存、HTTP 契約、認證（屬 verb-contract-and-remote-client）。
- 不搬設定欄位歸屬、不引入 .speclink.remote.yaml、不動 tools 描述子（屬 config-system-rework）。
- 不做 napi 綁定與 SDK 公開 API 整理（屬 node-sdk）。
- 被否決方案（見討論記錄，不再重提）：VFS 式以路徑為語彙的介面——強迫非檔案系統後端模擬目錄樹與 mtime 語意；async 介面——把 async runtime 拖進 core，違反「無 async runtime」紅線且 CLI 情境不需要。

## Decisions

1. **介面語彙採領域層級、同步、object-safe**
   - 介面名 `Store`（PascalCase），方法 snake_case，全部同步簽名，不含泛型方法（保持 object-safe，為 node-sdk 的動態注入鋪路）。
   - 方法集依現行 fs 呼叫盤點劃定：列舉 changes 與其 metadata、artifact 讀／寫／存在檢查、delta spec 檔列舉與能力名列舉、正典 spec 讀寫、archive 搬移（含歸檔目錄命名）、discussion 的建立／讀取／附加／歸檔、workflow-config 讀取、變更時間（updated_at）查詢。
   - 排序語意：現行以「change 目錄內最新 mtime（截秒）」排序；介面改為回傳 updated_at 秒值，fs 實作由 mtime 推導——行為輸出不變，語意從「檔案系統屬性」升級為「儲存中繼資料」。
   - 替代案：以 PathBuf 為參數的檔案介面——被討論第 1 輪否決（介面過淺、洩漏佈局）；async trait——被第 2 輪否決（違反紅線）。

2. **paths.rs 一分為二：規格佈局歸 speclink-fs，宿主工作路徑留 core**
   - 規格文件佈局（spec 目錄、specs/、changes/、archive/、discussions/、config.yaml 路徑）遷入 speclink-fs 的佈局模組，成為實作細節。
   - 宿主側路徑（`.speclink/` 工作目錄、touched、snapshots、`.speclink.yaml` 位置、專案根 walk-up 探索）留在 core 新模組 workspace.rs——工作資料與 workspace 檔案不是規格文件，不進 Store。
   - 替代案：整個 Paths 搬入 speclink-fs——被否決，因為 touched/snapshots 與 bootstrap 探索在遠端模式（後續 change）依然是宿主本地行為，放進儲存實作會迫使遠端實作也提供無意義的本地路徑。

3. **注入方式：CLI 組裝點建立 fs 實作，core 以 dyn 引用傳遞**
   - speclink-cli 在指令進入點建立 fs 實作並以 `&dyn Store` 傳入 core 流程函式；core 不知道實作型別。
   - 依賴方向：speclink-cli → speclink-fs → speclink-core（fs 實作依賴 core 的介面定義）；core 不依賴 speclink-fs。
   - 替代案：core 以泛型參數接受實作——被否決：單態化對 CLI 無收益，且 object-safe dyn 是 node-sdk 動態注入的前置條件。

4. **AppConfig（.speclink.yaml）維持宿主側直讀；WorkflowConfig 改經 Store**
   - `.speclink.yaml` 是宿主 bootstrap（討論第 5、11 輪），由 core 的設定模組照現行方式讀取。
   - `openspec/config.yaml` 是儲存側文件，讀取改經 Store 的 workflow-config 方法；序列化格式與解析行為（serde_yaml、缺檔回預設）完全不變，既有檔案可讀（向後相容）。

5. **git 互動不動**：drift 與 archive 對 git 的呼叫屬引擎流程而非儲存，維持現狀；archive 對規格文件的搬移改經 Store，對 git 的查詢不變。跨平台注意：fs 實作沿用既有 util 的路徑與換行處理，不新增平台假設。

6. **文件骨架**：`docs/architecture.md` 與 `docs/architecture.zh-TW.md`（三層架構、縫線位置、與後續 change 的關係）、`docs/getting-started.md` 與 `docs/getting-started.zh-TW.md`（純本地情境：init → discuss → propose → apply → verify → archive 走一遍）；README.md 增加 Documentation 章節雙語連結。中英內容語意對等，各自成篇（不留「稍後補」空節——後續 change 各自新增自己的檔案與連結）。

## Implementation Contract

- **行為**：使用者可觀察的一切（人眼輸出、`--json` payload、exit code、檔案系統效果、錯誤訊息文字）與重構前完全一致。任何指令在既有專案、無專案目錄、損壞 metadata 等情境下的輸出均不得改變。
- **介面／資料形狀**：core 公開儲存介面 `Store`（同步、object-safe），speclink-fs 公開其檔案系統實作與建構函式（以專案根與 spec 目錄名建構）；speclink-cli 是唯一的組裝點。workspace 的 members 含 speclink-fs。介面方法命名以領域動詞為準（列舉、讀取、寫入、歸檔、附加），不以路徑為參數。
- **失敗模式**：儲存錯誤沿用 anyhow 錯誤鏈，對外錯誤訊息文字與現況一致；fs 實作對缺檔／缺目錄的容錯行為（回空清單、回預設值）逐一保持。
- **驗收條件**：
  - cargo test 全 workspace 綠燈（既有測試不修改斷言即通過）。
  - parity_suite 31 項、color_suite 16 項、twin harness 8 情境全數通過。
  - 檢查性測試：speclink-core 原始碼中不存在對規格文件目錄的直接 std::fs 呼叫（宿主側 workspace.rs 與通用 util 除外），以測試或審查腳本佐證。
  - cargo build --release 全 workspace 成功。
- **範圍邊界**：in scope＝介面定義、fs 實作平移、各模組改接介面、CLI 注入、文件骨架；out of scope＝任何輸出變更、新指令、設定搬家、遠端、SDK。

## Risks / Trade-offs

- [parity 迴歸：搬移過程改變輸出或錯誤文字] → 逐模組遷移，每步跑 cargo test 與 parity/color 對照；錯誤訊息以既有字串常數平移不重寫。
- [排序行為漂移：updated_at 推導與現行 mtime 截秒不一致] → fs 實作沿用「遞迴取最新 mtime、截整秒」的既有演算法平移，並以既有排序測試覆蓋。
- [Windows 路徑差異] → 佈局程式碼沿用既有 util 與 PathBuf 組裝，不引入字串拼接；CI 於三平台驗證。
- [介面切太細或太粗] → 方法集嚴格依「現行 fs 呼叫盤點」劃定，不預先為遠端情境加方法（後續 change 需要時再擴充，向後相容地加）。

## Migration Plan

1. 於 core 定義 `Store` 介面與領域資料形狀（含 updated_at 語意），不動任何呼叫端。
2. speclink-fs 加入 workspace，平移 paths.rs 的規格佈局知識與各模組的 fs 操作，實作介面；宿主側路徑移至 core 的 workspace.rs。
3. 逐模組（model → discuss → status/validate/analyzer → newcmd/tasks/inprogress → archive/drift）把直接 fs 呼叫替換為介面呼叫，每個模組完成即跑測試與 parity。
4. 移除 core 的 paths.rs；CLI 組裝點注入 fs 實作。
5. 撰寫雙語文件骨架與 README 連結。
6. 全量驗收（cargo test、三套回歸對照、release build）。

## Open Questions

（無——介面粒度、async 與否、佈局歸屬皆已由討論記錄定案。）
