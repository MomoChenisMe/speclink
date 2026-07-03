## Context

store-trait-and-fs-adapter 已把引擎與儲存切開（fs 為預設實作），config-system-rework 已把政策歸屬 store 側並拆分 init。本 change 交付團隊情境的接縫：一份「領域動詞層」REST 契約（server 端由團隊系統以 SDK 內嵌引擎實作；第一個消費者為 wadpilot，其 docs/sdd-research/04-speclink-final-design.md 的 §5.3 response 形狀為對齊參考），以及 CLI 端的 remote 薄 client。核心約束：speclink-core 維持「無 async runtime、無網路呼叫」紅線；fs 模式的既有行為是回歸保護對象；契約必須通用（gate 為 per-project 政策，非 wadpilot 專屬形狀）。

## Goals / Non-Goals

**Goals:**

- 連接檔 `.speclink.remote.yaml` 與「檔案存在即模式訊號」的模式解析。
- v1 動詞契約：涵蓋 RD 本地全流程（列舉、讀寫、討論、認領、勾選、歸檔、instructions、政策 side-car、身分查驗），payload 與既有 `--json` 對齊。
- PAT 認證與憑證管理（檔案存於使用者層級設定目錄；SPECLINK_TOKEN 覆寫）。
- repo 身分三層驗證鏈與 fail-loud 錯誤翻譯。
- 技能資產動詞化與 marker remote 變體，使單一來源技能於兩模式通用。

**Non-Goals:**

- 不實作 server 端（團隊系統自行以 SDK 內嵌引擎實作；契約文件是交付物）。
- store push／pull 遷移指令延後為獨立 change（本 change 只交付 link/unlink 與空專案的手動遷移指引）。
- 不做離線快取（連不上即明確失敗——討論第 3 輪否決快取）。
- 不做 OAuth device flow（PAT 先行，討論第 3 輪定案）；不做 OS keyring（原生依賴矩陣，延後）。
- 不做 monorepo 巢狀綁定（v1 一 repo 一綁定，最近者勝）。
- 被否決方案（見討論記錄）：文件級 CRUD 契約（承載不了 gates 與原子性）、以 git remote URL 推斷 repo 身分（fork／鏡像不可靠）、連不上時 fallback 到 fs（分岔假真相）。

## Decisions

1. **契約切在領域動詞層，動詞集 v1 如下**（base path 形如 /api/speclink/v1/projects/{project}/…，project 範疇編入連接檔 url）：
   - changes：列舉（GET changes）、讀取含 artifact 狀態與版本（GET changes/{name}）、建立（POST changes）、認領（POST changes/{name}/claim，原子）、歸檔（POST changes/{name}/archive，server 端 check-all-then-apply）。
   - artifacts：讀取（GET changes/{name}/artifacts/{artifact}，回內容與 version）、寫入（PUT 同路徑，必帶 If-Match: version）。
   - tasks：勾選（POST changes/{name}/tasks/{taskId}/done）。
   - instructions：GET changes/{name}/instructions/{artifact}（server 以引擎計算 payload）。
   - discussions：列舉、建立、讀取、context、add-round、conclude、archive、promote（與 CLI 動詞一對一）。
   - 政策 side-car：GET config（workflow-config 有效值）；詞彙：GET language。
   - 正典規格：GET specs、GET specs/{capability}。
   - 身分：GET whoami（回 token 身分與可用 repos）。
   - 替代案：dispatch(argv) 直接轉發整條指令——被否決：server 端需要逐動詞做權限、gate 與原子性治理，粗粒度轉發等於把治理面拱手讓出。
2. **共通契約規則**：payload 欄位 camelCase 且與既有 `--json` 同名對齊；409 一律附機器可判 reason 欄（version_conflict、ownership_lost、change_busy、repo_mismatch、project_not_empty 等列舉值）；API 版本以請求 header X-Speclink-Api-Version: 1 協商，server 不支援時回明確錯誤；repo 身分以 X-Speclink-Repo header 隨每個請求攜帶；認證 Authorization: Bearer <PAT>。gate 為 per-project 政策：契約只保證「狀態轉移由 server 裁決」，CLI 不假設任何 gate 存在與否。
3. **HTTP client 選型**：新 crate speclink-remote 採 ureq（同步、rustls、無 tokio），speclink-cli 依賴之；speclink-core 完全不依賴——維持核心無網路紅線。替代案：reqwest blocking——被否決（拖入 tokio 執行緒池，體積與紅線皆不利）。
4. **模式解析與路由**：workspace.rs 的專案探索增加連接檔偵測——有 `.speclink.remote.yaml` 即 remote；與 openspec/ 並存時 remote 勝出、stderr 一行警告。speclink-cli 的指令進入點依模式路由：fs 走既有引擎＋fs 實作，remote 走 speclink-remote client；SPECLINK_STORE_URL 可覆寫連接 url（個人/CI）。
5. **認證與憑證**：auth login 讀取 PAT（互動貼上或 --token-stdin），依 url origin 存入使用者層級設定目錄的 credentials 檔（Unix 權限 0600；Windows 沿使用者目錄 ACL）；SPECLINK_TOKEN 優先於檔案；auth status 呼叫 whoami 顯示身分與 repo 驗證結果。init/link 時若已有可用 token 即以 whoami 驗證 repo∈專案並回報，無 token 則提示執行 speclink auth login（首次動詞時仍會驗證）——對討論第 16 輪流程的精緻化：驗證時點「有憑證即提前、無憑證即延後」，不阻塞離線 init。
6. **錯誤翻譯紅線**：任何非 2xx SHALL 翻譯為單行語義化訊息＋建議動作——401→「執行 speclink auth login」、403（repo_mismatch）→指出 change 歸屬 repo 與當前 repo 名、404→change/專案不存在、409→依 reason 逐值對應動作、連線失敗/5xx→「server 不可用，檢查連接 url」；絕不輸出裸狀態碼給使用者或 agent。
7. **技能動詞化與 marker 變體**：新增兩個雙模式讀取動詞——speclink artifact cat（fs 讀檔、remote 走 GET artifact）與 speclink language show；技能資產中的直接讀檔指示全數改為動詞（含 discuss 的詞彙載入、propose 的依賴 artifact 閱讀）；marker 區塊渲染增加 store 維度，remote 變體以「文件在團隊系統、一律走 speclink 動詞」取代路徑句。fs 模式 golden 因此刻意更新並記錄。

## Implementation Contract

- **行為**：
  - 無連接檔＝fs 模式，全部既有行為不變；有連接檔＝remote 模式，list/status/instructions/new/discuss/task done/archive/artifact cat/language show 走契約端點，人眼與 `--json` 輸出形狀與 fs 模式一致。
  - 連接檔與 openspec/ 並存：remote 生效＋stderr 一行警告。
  - 未登入執行 remote 動詞：單行訊息提示 speclink auth login，exit code 非 0。
  - claim 已被他人持有：409 ownership 訊息含持有人；artifact 寫入版本過期：409 version_conflict 訊息提示重新拉取。
  - 在 repo A 對歸屬 repo B 的 change 執行動詞：403 訊息同時指出兩個 repo 名。
- **介面／資料形狀**：連接檔 YAML 兩欄位（url 必填、repo 選填）；契約端點與 payload 形狀以 docs/verb-contract.md 為正典參考（含每個 409 reason 列舉值與範例 body）；credentials 檔 YAML（url origin → token）。
- **失敗模式**：連線逾時／拒絕＝明確失敗（不重試迴圈、不快取）；server 版本不符＝明確錯誤與升級提示；憑證檔缺失＝視為未登入。
- **驗收條件**：
  - 契約整合測試：以 dev-dependency 的極簡 mock server（tiny_http）覆蓋每個動詞的成功、401、403 repo_mismatch、404、409（各 reason）、版本協商失敗、連線失敗——斷言 CLI 輸出訊息與 exit code。
  - 模式解析單元測試：無檔／有檔／並存警告／SPECLINK_STORE_URL 覆寫。
  - auth 測試：login 寫檔權限、status 輸出、SPECLINK_TOKEN 優先序。
  - 技能資產斷言：全部 SKILL.md 資產不再含直接讀取 spec 目錄檔案的指示（掃描測試）；marker remote 變體 golden。
  - fs 模式全量回歸：parity／color／twin 通過（技能與 marker 的 golden 刻意更新已記錄）。
- **範圍邊界**：in scope＝連接檔與模式解析、動詞契約文件、remote client 與路由、auth、repo 驗證鏈、錯誤翻譯、artifact cat／language show、技能動詞化、marker 變體、團隊模式文件；out of scope＝server 端實作、store push/pull、離線快取、OAuth、keyring、巢狀綁定。

## Risks / Trade-offs

- [契約形狀與 wadpilot 04 §5.3 漂移] → 撰寫 docs/verb-contract.md 時逐項對照 04 的 response 形狀與 409 reason 命名；差異須在文件中標注理由。
- [動詞集過大導致 change 過胖] → 動詞實作共用單一 client 請求層與錯誤翻譯層，逐動詞只是路徑與 payload 映射；push/pull 已切出。
- [技能動詞化造成 fs 模式 golden 大面積更新] → 獨立任務刻意更新並在對照記錄中標注；內容變更僅限讀檔指示句。
- [憑證檔在 Windows 的權限保護較弱] → 文件明載 PAT 視同 SSH 金鑰的運維準則（洩漏即撤銷）；keyring 列為後續強化。
- [mock server 與真實 server 行為差距] → 契約文件即正典；wadpilot 端實作時以契約文件驗收，發現缺漏回饋修訂契約版本。

## Migration Plan

1. 契約文件（docs/verb-contract.md 與 zh-TW 版）先行定稿——動詞、payload、錯誤、版本協商。
2. speclink-remote crate：請求層、錯誤翻譯層、auth 儲存。
3. 模式解析與 CLI 路由骨架（先接 list/status 讀路徑）。
4. 寫路徑動詞（new change/artifact、task done、discuss 系列、claim、archive）。
5. artifact cat／language show 雙模式動詞＋技能資產動詞化＋marker 變體。
6. auth 子指令與整合測試全量覆蓋。
7. fs 回歸與 golden 刻意更新。
8. 團隊模式雙語文件與 README 連結。

## Open Questions

（無——契約層級、認證方式、repo 識別、錯誤紅線皆由討論記錄第 4、13、15、16 輪定案；未定細節〔repos 註冊表管理動詞、gate 政策形狀〕屬 server 端範疇，由契約文件標注為 server 實作自由度。）
