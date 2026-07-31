## Context

speclink 生成的指令檔分兩類：CLAUDE.md/AGENTS.md 內的受管區塊（帶 MARKER_VERSION 版號的 SPECLINK 標記）與技能檔（frontmatter 的 metadata.version 現為寫死的 "1.0"）。引擎已有冪等的 update()（speclink-core 的 init 模組）依 .speclink.yaml 的 tools 清單整套再生受管檔；desktop 直嵌 speclink-core，開專案走純探測 probe（open_project 三態判定、零寫入）；render golden 測試以 UPDATE_GOLDEN=1 在乾淨樹上重生為既定紀律。缺的是：版本戳無單一來源（skill 版號死值）、無過期偵測、無更新提示、無 bump 紀律的機械強制。

相關討論記錄 desktop-instruction-staleness-prompt 已定案機制五件套與附帶 UI 決定；本設計落實其遞延細節。討論 desktop-workspace-auto-init 追加定案「缺失」回報態——tools 清單宣告的指令檔不存在＝從未安裝（如 clone 後指令檔未進版控的專案），提示以安裝語意補裝，與「標記被移除＝退出受管」明文區分。

## Goals / Non-Goals

**Goals:**

- 產物層版本單一權威：MARKER_VERSION 同時出現在 marker 區塊與 skill frontmatter，比對即字串相等
- 引擎側唯讀過期探測（含「從未安裝」的缺失偵測），desktop 開專案搭載、非阻斷提示，更新／安裝復用 update()
- 「保留現狀」per 專案 per 版本記憶於 desktop 本地，不污染 repo
- version–hash 鎖定測試把 bump 紀律變成紅燈
- 刪除側欄常駐版號

**Non-Goals:**

- 逐檔挑選更新、內容 diff 檢視 UI（第一版僅列檔名）
- 內容比對作為過期判定（僅用於覆蓋警告清單）
- CLI 指令面變動、.speclink.yaml／openspec/config.yaml 新欄位
- CLI 與 desktop 版本漂移處理（sidecar 同版機制已涵蓋）
- 強制更新或阻斷開啟

## Decisions

### 決策 1：版本戳同源與字串相等比對

skill frontmatter 的 metadata.version 值改為 MARKER_VERSION 原字串（如 "v1.3.0"），與 marker 區塊同源。過期判定＝「工作區戳記 ≠ 當前 MARKER_VERSION」的字串不等，不解析 semver、不比大小——版本戳唯一職責是偵測「不同」，方向與距離無意義。舊工作區的 "1.0" frontmatter 與舊 marker 版號自然不等於現版，首次過渡零遷移成本。
替代方案：semver 解析比大小——多引入解析失敗的錯誤路徑，卻買不到任何行為差異（「工作區比引擎新」在 sidecar 同版機制下不是要處理的情境），否決。

### 決策 2：marker 權威判定與退出語意

探測的檢查對象由 .speclink.yaml 的 tools 清單決定（與 update() 同一資料源）。對每個內建工具讀其 instruction 檔（CLAUDE.md／AGENTS.md）：檔案不存在 → 該工具缺失（從未安裝，如 clone 後指令檔未進版控）→ 提示補裝；檔案存在則讀 SPECLINK 標記版號：版號 ≠ 現版 → 該工具過期；檔案存在但標記不存在（使用者整塊移除）→ 視為手動退出受管，跳過不提示——尊重移除意圖，與 update() 對無標記檔的 upsert 行為刻意區隔（探測不提示 ≠ 更新不會寫回；提示層不引導使用者走向重新植入）。「檔案不存在」與「檔案在但無標記」是意圖完全不同的兩個狀態，判定上明文分離：前者從未表達過任何意圖、應引導安裝，後者表達過移除意圖、應尊重。skill frontmatter 戳記不參與過期判定，僅作為受管檔差異清單的一部分。
替代方案：檔案不存在也視為退出受管——把「從未安裝」誤讀成「主動移除」，clone 下來缺指令檔的專案永遠不會被提示，正是本缺口的成因，否決。
替代方案：逐 skill 檔讀 frontmatter 判定——多讀十餘檔卻與 marker 判定同值（update() 整套再生使兩者恆同步）；skill 戳記與 marker 不一致只可能是手動編輯，屬自訂範疇，不應觸發過期。否決。

### 決策 3：探測回報形狀

speclink-core 新增唯讀探測函式（與 update() 同居 init 模組），輸入專案根路徑，回報：目前引擎版本、各工具的狀態（工作區版號、是否過期、是否缺失）、以及「更新將新建或改寫且內容有異」的受管檔相對路徑清單（render 期望內容 vs 磁碟內容，不存在的檔案內容視為空、必列入；比對前正規化換行以免 Windows CRLF 誤報）。整體判定四態：缺失（任一工具檔案不存在）優先於過期（任一工具版號不等），再來現版；設定解析失敗或存在檔讀取錯誤為無法判定。差異清單不區分「過期」與「自訂」（無歷史 render 無法區分，討論已裁定），提示文案以「若你曾手動調整，更新會覆蓋」一體提醒。第一版僅列檔名，不呈現內容差異。
替代方案：回報逐檔 diff 內容——UI 端沒有對應的檢視設計（Non-Goal），引擎先回傳大塊 diff 是無消費者的過度設計，否決。

### 決策 4：獨立唯讀 command 搭載

新增 Tauri command（單行委派 speclink-desktop-core，再呼叫 speclink-core 探測），前端於本地專案分頁成為活躍時呼叫一次；更新動作完成後與 workspace-changed 事件到達時重查（外部跑 update() 也能自然消提示）。open_project 維持三態判定單一職責不變胖。remote workspace 分頁不觸發（探測只對本地 checkout 的指令檔有意義；remote 綁定的本地 checkout 同樣有指令檔，依 status=project 與否分流）。
替代方案：掛進 open_project payload——省一次 IPC，但讓純探測 probe 承擔第二職責，且分頁重開頻率遠高於需要重查的頻率，否決。

### 決策 5：更新動作復用既有再生入口

提示的「更新」動作呼叫既有 update()（desktop 行程內、與 CLI 同一入口），冪等整套再生、依 .speclink.yaml 記錄的 store mode 維持 marker 措辭。執行前提示已列出將被改寫的差異檔清單；執行後重跑探測，過期旗標消失即收合提示。失敗（如檔案唯讀、config 解析錯誤）以非阻斷錯誤呈現於提示原位，不擋專案使用。
替代方案：desktop 自寫覆蓋邏輯——與 CLI 分岔、違反既有「單一收斂入口」設計（reconcile 註解明文），否決。

### 決策 6：略過記憶存前端本地持久化

「保留現狀」寫入 desktop 前端本地持久化（與分頁持久化同一套 localStorage 慣例），鍵值「專案路徑 → 已略過的 MARKER_VERSION」。探測回報過期但已略過同版 → 不顯示提示；引擎版本再 bump → 鍵值不匹配 → 重新提示。不進 Rust 側、不進任何 repo 檔案。
替代方案：Tauri 側 app 設定檔——多一層 IPC 與序列化卻無跨前端消費者；.speclink.yaml——討論已否決（污染 repo、個人偏好入團隊設定）。

### 決策 7：提示形態為分頁內非阻斷橫幅

提示採 UpdateBanner 同構的橫幅語彙，置於過期或缺失專案的分頁內容頂部（per 專案，非 app 層）：一行說明（含差異檔數）＋主動作＋「保留現狀」。主動作依探測態分文案：過期為「更新」、缺失為「安裝」——從未安裝的專案說「更新」是語意錯置；兩者呼叫同一個再生入口，僅文案分支。不用 modal（阻斷開啟違反溫和定位）、不用系統通知（離開情境）。文案遵循 openspec/LANGUAGE.md 原則：不出現 marker、frontmatter、MARKER_VERSION 等工程詞，以「指令檔」「技能」等產品詞表述。
替代方案：app 層全域橫幅——與 UpdateBanner 疊撞且過期是 per 專案狀態，否決；設定頁內卡片——被動可見性等於沒有提示，否決。

### 決策 8：version–hash 鎖定測試與重生防呆

render_golden 測試檔新增鎖定測試，鎖定檔 assets.lock 與 golden 同目錄提交，記錄 MARKER_VERSION 與全部 render 輸出的指紋。hash 用測試檔內自寫的 FNV-1a（十餘行、演算法寫死、跨 toolchain 穩定）——std 的 DefaultHasher 不保證跨 Rust 版本穩定會假紅燈，引入 sha2 等密碼學依賴對變更偵測是過度配備。
判定：render 指紋 ≠ 鎖定檔指紋且 MARKER_VERSION ＝ 鎖定檔版本 → 失敗，訊息寫明修法（遞增 MARKER_VERSION 後以 UPDATE_ASSETS_LOCK=1 重生）。重生防呆：UPDATE_ASSETS_LOCK=1 執行時若指紋變了而版本未變，拒絕重寫並同樣失敗——防 agent 反射性重生繞過紀律。UPDATE_GOLDEN 與 UPDATE_ASSETS_LOCK 為兩個獨立開關：golden 重生不觸碰 lock。鎖定檔重生繼承乾淨樹慣例。
替代方案：以 golden 本身當指紋來源——golden 被 UPDATE_GOLDEN 無腦重生、無法承載「上次 bump 時點」語意，否決；CI 腳本比對 git diff——測試框架外的旁路、本地跑不到，否決。

### 決策 9：側欄版號刪除

刪除側欄底部常駐版號的條件渲染（設定沉底列之後的版本文字），設定頁軟體更新卡為 app 版號唯一住所。currentVersion 狀態保留（設定頁仍消費）。無其他連動。

## Implementation Contract

**行為**：

- 以 desktop 開啟含過期指令檔的本地專案 → 分頁內容頂部出現非阻斷橫幅，顯示過期事實與將被改寫的檔案數；按「更新」→ 受管檔整套再生為現版、橫幅消失；按「保留現狀」→ 橫幅消失，同版本不再出現，引擎升版後再現。
- 以 desktop 開啟 tools 清單宣告但指令檔不存在的本地專案（從未安裝，如 clone 後指令檔未進版控）→ 同一橫幅以安裝語意呈現，主動作為「安裝」；按「安裝」→ 受管檔生成為現版、橫幅消失；「保留現狀」與略過記憶行為與過期態相同。
- 指令檔為現版、或 instruction 檔存在但無 SPECLINK 標記、或分頁為 remote workspace → 無任何提示。
- speclink 開發者修改內嵌資產（assets/skills 或 marker 模板）而未 bump MARKER_VERSION → cargo test -p speclink-core 失敗，失敗訊息含修復步驟。

**介面／資料形狀**：

- speclink-core：新增唯讀探測 pub fn（init 模組），輸入專案根 Path，回傳含 currentVersion、各工具（tool 名、workspaceVersion、stale 旗標、missing 旗標）、differingFiles（專案根相對路徑清單）的結構；不寫入任何檔案。
- desktop IPC：新增一個唯讀 Tauri command 回傳上述結構的 camelCase JSON；新增一個更新 command 委派 update() 並回傳其 UpdateOutcome 既有形狀。兩者皆單行委派 speclink-desktop-core。
- 前端：localStorage 新增略過記憶（專案路徑 → 已略過版本字串）；i18n 字典新增橫幅文案鍵（zh-TW 與 en 兩語系）。
- assets.lock：純文字兩欄（版本、指紋），格式細節由鎖定測試單方擁有。

**失敗模式**：

- 探測遇 .speclink.yaml 解析失敗或存在檔讀取錯誤 → 回報「無法判定」，前端靜默不提示（探測失敗不得阻斷開專案，與監看不可用的既有降級語意一致）；指令檔單純不存在不是讀取錯誤，判缺失。
- 更新失敗 → 錯誤訊息呈現於橫幅原位，橫幅保持可重試；不回滾（update() 冪等，重試即收斂）。

**驗收準則**：

- speclink-core 單元測試：探測對「現版工作區／舊版 marker／無標記／指令檔不存在／tools 空清單」五情境的回報（含「一工具現版、另一工具檔案不存在 → 缺失優先」）；換行正規化使 CRLF 工作區不誤報差異。
- 鎖定測試自身：改資產不 bump → 紅（含防呆重生路徑）；bump＋重生 → 綠；只 bump 不改 → 綠。
- desktop-core（vitest）：略過記憶的顯示裁決（過期＋未略過 → 提示；缺失＋未略過 → 提示（安裝文案）；過期或缺失＋已略過同版 → 不提示；已略過舊版＋再 bump → 提示）。
- 桌面 GUI 以真實視窗驗證橫幅出現、更新後消失、保留後同版不再現。
- render golden 四份 snapshot 與兩套 repo 技能實例於乾淨樹同批再生，diff 僅含版本欄位變動與本變更的刻意內容。

**範圍邊界**：in scope＝上述引擎探測、desktop 搭載與提示、鎖定測試、側欄版號刪除；out of scope＝CLAUDE.md 開發備忘一行（實作期取消：備忘已於 c0303d8 移除，該檔現僅剩引擎受管區塊；bump 紀律改由鎖定測試的失敗訊息單獨承載）、CLI 指令面、codex 以外自訂工具描述子的探測（tools 清單含自訂描述子時第一版僅涵蓋內建工具，探測回報結構預留 tool 名欄位不需改形狀）、逐檔更新、diff 檢視 UI。

## Risks / Trade-offs

- [golden 大翻動與平行變更 discuss-propose-from-docs 搶同四份 snapshot] → 排程序列化：該變更先落地，本變更後實作、一次 bump 涵蓋兩者資產變動；golden／lock 重生一律乾淨樹。
- [Windows 換行差異使 differingFiles 誤報全數檔案] → 比對前正規化 CRLF；驗收準則含此情境測試。
- [使用者自訂內容被「更新」覆蓋] → 提示先列差異檔名並明文警告；update() 冪等且工作區在 git 下，覆蓋可由版控復原——不做應用層備份（過度設計）。
- [探測誤把探測失敗當「不過期」] → 回報結構明確四態（缺失／過期／現版／無法判定），前端對無法判定靜默但 log，不與現版混同。
- [「從未安裝」被誤讀成「主動移除」而永不提示] → 判定明文分離：檔案不存在＝缺失（引導安裝），檔案在但無標記＝退出受管（尊重不提示）；單元測試各自釘死。
- [鎖定測試與 golden 雙開關混淆（agent 誤跑錯 env var）] → 兩開關語意在失敗訊息中互相指路；防呆重生使錯誤路徑同樣紅燈。

## Migration Plan

單向前進、無部署順序議題：本變更合入即 bump MARKER_VERSION 並首次生成 assets.lock；既有使用者專案下次以新版 desktop 開啟時自然觸發首次提示（"1.0" frontmatter 與舊 marker 皆判過期）。回退＝revert commit，工作區已被更新過的專案戳記比回退後引擎新——字串不等仍判過期、再更新一次即收斂，無資料損失。

## Open Questions

（無——討論遞延的四項細節已在決策 1、2、3、7 定案。）
