## Context

這個 change 修正 `phase3-e2e` 真實 Desktop 多分頁驗收中暴露的兩個同源競態。

第一個競態位於 Desktop Tauri adapter。相同 origin 的資料命令共用一個 `TokenManager`，但目前 bearer 快取為空或請求收到 401 時，每個呼叫都能各自讀取 Keychain 中同一枚一次性 refresh credential 並送出 rotation。兩個呼叫幾乎同時發生時，第一個成功輪替，第二個仍使用舊 credential，Server 依正確的重放防護撤銷整個 family，Desktop 因而由 offline 誤入 `needs-reauth`。

第二個競態位於 React store。各 WorkspaceSession 已自帶 dataSource，但看板清單仍投影到一組全域 `changes`、`specs`、`archived` 與 `discussions`。切換 activeKey 後若新 session 的 refresh 失敗，保留「最後成功 snapshot」的既有邏輯會保留前一個 workspace 的全域內容；較舊的非同步回應也沒有以來源 session 檢查可見內容所有權。

此變更只落在 `apps/desktop/src-tauri` 與 `apps/desktop` 的 adapter／UI state 邊界。`speclink-core`、`speclink-cli`、TeamStore、Remote Protocol 與 Server replay 規則均不變。沒有序列化、設定、git 行為、CLI 人眼／`--json` 輸出、外部相依套件或資料遷移。

## Goals / Non-Goals

**Goals:**

- 同一 connection 的併發 runtime 認證恢復只消耗一次 refresh credential，等待者共用成功結果。
- 保留真正撤銷、無效 credential 與第二次 401 進入 `needs-reauth` 的既有安全語意。
- 將每個 locator 的最後成功讀取內容隔離，activeKey 切換後不顯示其他 workspace 的內容。
- 讓較舊或背景 refresh 只能結算到來源 locator，不覆寫目前 active session。
- 以先失敗再通過的 Rust 與 TypeScript 決定性測試重現兩個競態。

**Non-Goals:**

- 不替 Server 增加 refresh credential 寬限期，也不弱化一次性輪替與重放撤銷。
- 不持久化 workspace snapshot，不變更 Keychain schema 或 access token 僅存記憶體的原則。
- 不重構完整 store、不新增通用快取框架、不更換狀態管理套件。
- 不改變明確 device login／PAT 登入流程的使用者介面；使用者主動登入與 runtime rotation 同時發生的額外編排不在本次觀察到的多 session 恢復範圍。
- 不直接修改 `phase3-e2e` 的 change artifact 或代替其剩餘人工驗收。

## Decisions

### 每個 connection 的 refresh singleflight 與雙重檢查

在既有每個 origin 共用的 `TokenManager` 內加入只涵蓋 credential 取得／輪替的同步協調。初次取得 bearer 的呼叫進入協調區後 MUST 再檢查 bearer：若先行者已成功發布 token，等待者直接複用；只有仍無 bearer 的先行者能讀取 refresh credential、呼叫 rotation、回寫 Keychain 並發布 access token。

401 恢復 MUST 攜帶該次被拒的 bearer 進入協調區。取得協調權後，若目前 bearer 已存在且不同於被拒 token，代表另一個呼叫已完成恢復，等待者直接複用新 token；若仍是同一枚或為空，才由該呼叫清除並輪替。網路呼叫期間不持有 bearer／health 狀態鎖，避免巢狀鎖；協調鎖只屬於單一 connection，其他 origin 不受阻塞。成功者發布新 bearer 後，所有等待者各自重試原始讀取一次。

被 Server 明確拒絕的 rotation 仍由先行者原子地標記 `needs-reauth`，等待者取得協調權後看到該狀態即直接拒絕。暫時性 transport、5xx 或 Keychain 錯誤維持 `Unavailable`，不被誤分類為 credential 失效。PAT 無 rotation，仍沿用既有「同一 PAT 重試一次，第二次 401 才進 reauth」語意。

替代方案一是由 Server 接受短時間重複 refresh；這會削弱重放偵測並掩蓋本機錯誤，因此拒絕。替代方案二是用全應用程式共用鎖；它會讓不同 Server 無故互相阻塞，因此拒絕。選擇既有 per-origin manager 內的 singleflight，因為範圍最小且與 credential 所有權一致。

### 依 locator 隔離最後成功 snapshot 與可見投影

Desktop store 在執行期維護以 locator key 為鍵的最後成功讀取 snapshot；每份 snapshot 包含看板需要的 `changes`、`specs`、`archived`、`discussions` 與 loaded 狀態，但不寫入 localStorage。每次 refresh 在 await 前捕捉來源 locator key 與該 key 的請求世代，四組清單全部成功且仍是該 key 最新世代時，才以全有或全無方式更新該 key 的 snapshot。

activeKey 切換時，store MUST 先同步投影目標 key 的 snapshot，再啟動非同步 refresh。目標有 snapshot 時立即顯示其內容；沒有時清空資料型可見狀態並設為未載入，交由既有 restoring、錯誤或 connection banner 表達壞天氣。切換時同時清除不具跨 workspace 所有權的搜尋命中、detail drawer 與待確認操作，避免殘留上一個 workspace 的衍生資料；查詢文字與純 UI 偏好可保留。

refresh 成功後可更新來源 key 的 snapshot 與該分頁 badge；只有 `activeKey === sourceKey` 時才能投影到全域可見清單並遞增可見 refresh generation。分頁關閉時刪除其 snapshot 與請求世代。這個 snapshot 是 Desktop 呈現層的執行期資料，不進入 `speclink-core` 或 storage abstraction；因此不改變 storage 解耦的規格引擎邊界。

替代方案一是在每次切換時一律清空內容；它會破壞既有 offline／needs-reauth 保留該 session 最後 snapshot 的契約，因此拒絕。替代方案二是將 snapshot 持久化到 localStorage；它會擴大資料生命週期與隱私面，且無此需求，因此拒絕。選擇每個開啟 locator 的記憶體 snapshot，是滿足隔離與 stale 可讀兩項契約的最小變更。

### 以來源世代守衛非同步結果而非取消請求

每個 locator 維護單調增加的 refresh 世代。新的 refresh 發出後，較舊世代即使成功也 SHALL 被丟棄；不同 locator 的結果只更新各自 snapshot，不能依完成順序決定目前畫面。這同時保護「A 請求晚於 B 完成」與「同一分頁較舊 refresh 晚回」兩種情況。

替代方案是切換分頁時取消舊 DataSource 請求；目前各 local／remote adapter 沒有一致的可取消介面，為此擴充 Protocol 與所有 adapter 會超出 bug 範圍，而且取消仍不能取代 commit 時的所有權檢查。因此保留請求並在結算點做 locator＋世代守衛。

### TDD 回歸矩陣與既有安全行為

Rust 測試先以同步屏障讓同一 `TokenManager` 的兩個請求同時走到失效 bearer／refresh 邊界，證明修正前會送出兩次 rotation；實作後驗證只消耗一次 refresh credential、兩個呼叫都成功、最新 refresh 仍可使用且沒有 `needs-reauth` 事件。既有 revoked family、PAT 與暫時性失敗測試維持通過。

TypeScript 測試先以可控制完成順序的兩個 WorkspaceSession 重現 A→B 切換與 A 晚回；另覆蓋 B 已有自己的 snapshot、B 從未成功載入且為 `needs-reauth`、切回 A 等案例。元件測試確認清單、stale／reauth 呈現、搜尋命中與 detail 不跨 workspace。

替代方案是只依賴人工視窗重現；毫秒級 token 競態與 Promise 完成順序不穩定，無法當回歸門檻，因此拒絕。人工 Desktop 驗收保留為自動測試通過後的最後確認，不取代決定性測試。

## Implementation Contract

**Behavior**

- 同一 connection 的兩個 remote session 在 Server 恢復後同時遇到 401，只會發生一次 refresh credential rotation；兩個 session 均自動恢復並維持 online。
- Server 明確撤銷 credential family 時，Desktop 仍進入 `needs-reauth`，不以 singleflight 吞掉錯誤或無限重試。
- 切換 active 分頁後，可見清單、搜尋命中與詳情只屬於 active locator。目標有最後成功 snapshot 時顯示自己的 snapshot；沒有時顯示安全的未載入／恢復狀態。
- 來源分頁的較舊或背景回應不得覆寫另一個 active 分頁；返回來源分頁時可看到它自己最新成功的 snapshot。

**Interface / data shape**

- 對外 Tauri command、Remote Protocol、Server API、事件 payload、Keychain key、localStorage 格式與 CLI 契約全部不變。
- Rust 內部 `TokenManager` 增加 per-connection rotation 協調及「依被拒 bearer 取得替代 token」的內部行為；access token 仍只存在 Rust 記憶體。
- TypeScript store 增加以 locator key 為鍵的執行期 snapshot 與 refresh 世代；既有 UI 消費的可見欄位 shape 不變。
- 支援定位的主要路徑為 `apps/desktop/src-tauri/src/remote.rs`、`apps/desktop/src/store.ts` 與其既有測試；只有在呈現安全未載入狀態確有必要時才外科式調整 `apps/desktop/src/App.tsx`。

**Failure modes**

- rotation 被明確拒絕：維持 `needs-reauth` 事件與重新登入入口。
- transport、5xx 或 Keychain 暫時失敗：本次呼叫回錯並保留可重試狀態，不撤銷本機 credential、不清空該 session snapshot。
- refresh 部分清單失敗：不寫入部分 snapshot；active session 保留自己的上一份 snapshot，從未成功者維持安全未載入狀態。
- 較舊世代完成：靜默丟棄可見與 snapshot 更新，不彈出額外錯誤。

**Acceptance criteria**

- Rust 決定性併發測試能證明一次 rotation、兩個 caller 成功、family 仍有效及無 reauth；既有 refresh rotation、revocation、PAT、offline tests 全綠。
- TypeScript store／React 測試能證明 snapshot 所有權、無 snapshot 安全狀態、跨分頁與同分頁 latest-wins，以及搜尋／詳情不殘留。
- `cargo test -p speclink-desktop --test remote_runtime` 與必要的 `phase3_chain` 回歸通過。
- `npm test -w apps/desktop`、`npm run build -w apps/desktop` 通過。
- 真實 Desktop 以同來源兩 remote 分頁重跑 Server 中斷／恢復與切換 `needs-reauth` 分頁，觀察不到 credential family 自撤銷或跨 workspace 內容。

**Scope boundaries**

- 範圍內：Desktop runtime token 協調、Desktop store snapshot 所有權、決定性測試與針對性真實視窗驗收。
- 範圍外：Server replay policy、Protocol／IPC schema、credential 儲存格式、完整登入 UX、CLI/core、其他 change artifact 與無關重構。

## Risks / Trade-offs

- [單一 connection 的 rotation 期間等待者被同步阻塞] → 協調只包住罕見的 token 取得／輪替路徑，不包一般資料請求，且不同 origin 使用不同 manager。
- [401 交錯時後到呼叫清除先行者的新 bearer] → 以被拒 bearer 做雙重檢查，只有目前 bearer 仍相同時才能輪替。
- [snapshot 增加記憶體占用] → 僅保存開啟分頁的四組既有清單，關閉分頁立即清除，不持久化也不建立通用快取。
- [背景結果更新錯誤分頁或舊資料覆蓋新資料] → locator key 加每 key 世代雙重守衛，並用反向完成順序測試固定競態。
- [跨平台同步差異] → 只使用 Rust 標準同步原語與既有 blocking runtime，不依賴 macOS 專用 Keychain 行為來判定 singleflight；Windows／Linux 由相同單元測試保護。
- [既有行為回歸] → 保留 Server replay、PAT、offline、reauth 測試，並跑完整 Desktop 測試與 build；CLI parity/color fixtures 不在改動路徑且輸出零變更。
