## Context

Remote（team）模式的 CLI 動詞覆蓋缺七處：discuss 的 --slug、discard、link、seal 明文拒絕；show 與 in-progress add 靜默走本機 store 給出錯誤結果；demo 靜默在本機建資料。引擎層（speclink-core）對全部動詞早已支援——Command::DiscussNew 收選填 slug、DiscussDiscard 含 force guard、DiscussLink 與 DiscussSeal 齊備、InProgressAdd 含「未知 change 靜默成功」的凍結基線——缺口全在傳輸鏈：protocol DTO、server 路由、remote client、CLI remote 分支。前一輪封存變更 remote-verb-parity（2026-07-23）已建立動詞契約的骨架（verb-contract 規格與 docs/verb-contract.md 正典文件、CLI 雙沙盒 parity 測試），本變更沿同一骨架補齊。

現況細節（討論 remote-verb-parity 五輪盤點）：

- 討論建立請求只有 topic 欄位，server 端硬編不帶 slug；CLI remote 分支對 --slug 直接報錯。
- server 無討論 discard、link、seal 路由；change 側已有 discard 的 DELETE 端點前例（force 為 query 參數、引擎 guard 於 server 端生效）。
- show 於 CLI 完全沒有 remote 分支，直接開本機 store。
- in-progress add 同上——本機無該 change 時命中引擎「未知名稱靜默成功」，exit 0 無輸出，呼叫端（apply 技能）完全無感；started 站因此三層斷：CLI 不路由、wire 無 startedAt 欄、消費端明文將就為「完成數 > 0 即進行中」。
- created 與 archived 兩站在 remote 已完整（server 端以認證身分蓋章）。

## Goals / Non-Goals

**Goals:**

- remote 模式補齊 discuss new --slug、discuss discard、discuss link、discuss seal 四動詞，語意與 fs 模式一致（同引擎、同 guard、同錯誤語意）。
- remote 模式 show 改為 server 讀取組合，與 fs 模式同輸入同輸出形狀。
- remote 模式 in-progress add 路由至 server，started 站三層一次接通：路由、wire 欄位、看板推導。
- remote 模式 demo 明確拒絕。
- fs 模式輸出逐位元不變（迴歸對照不受影響）。

**Non-Goals:**

- 不啟用 wire 保留槽 lifecycle 與 claimed_by 的 server 狀態機或 durable 認領（等權限與管理基建）。
- 不改引擎 slugify 的非 ASCII fallback、不修 bulk archive、不動 --schema 與 claim 的刻意拒絕。
- 不動 Store trait 與三個 TeamStore driver——本變更零儲存層改動，conformance gate 不受影響。
- 不含 wad-old-web 既有空記錄的清理動作。

## Decisions

**D1：驗證與 guard 單一事實來源在引擎，server 直通、client 不重複。**
slug 的 ASCII kebab 驗證（引擎既有 gate）與 discard 的有輪拒刪 guard 都只住在 speclink-core；server 路由經 Command gateway 直通，引擎錯誤映射為語義化 API 錯誤，CLI 呈現非零 exit 與 stderr 訊息。替代方案「CLI 端先驗省一趟網路」被否決：兩份驗證會漂移，且 fs 與 remote 的錯誤訊息來源不同會破壞 parity。

**D2：討論建立請求增列選填 slug 欄位，serde default 向後相容。**
protocol 的建立討論請求 DTO 增 slug 選填欄位（--json 與 wire 一律 camelCase）；舊 client 不帶欄位時 server 收到預設 None，行為與現行完全相同；舊 server 收到新 client 的額外欄位依 serde 預設忽略。不開新版本協商——選填欄位即足夠。

**D3：討論 discard 複製 change 側 DELETE 模式；link 與 seal 比照 promote 的 POST 模式。**
DELETE /discussions/{slug} 帶 force query 參數，直通引擎 DiscussDiscard；POST /discussions/{slug}/link 與 POST /discussions/{slug}/seal 帶 change 名稱 body，直通對應引擎命令。三者皆為 unit-of-work 寫入（scope revision 前進、事件照引擎 outcome 發布），錯誤形狀沿 change 側 discard 與 promote 的既有映射慣例。

**D4：show 的 remote 版為 CLI 端讀取組合，不開新 server 端點。**
以既有讀 API（change 讀取、artifact 讀取、規格文件讀取、兩份清單）在 speclink-cli 的 remote 分支組出與 fs 模式相同的輸出；item 與 --type 的判別序以引擎現行為準，驗收以「同內容下 fs 與 remote 輸出一致」的對照測試釘死。替代方案「server 開 show 聚合端點」被否決：新端點是純轉發、無深度。
實作期修正：盤點發現 change 讀取回應原缺 show 需要的 meta 欄位（context projection 明文排除 change metadata、清單與 drift 也不帶 from_discussion），而 discussion-docs 的「remote link 鑄鏈可經 show 觀察」場景強制要求該鏈上 payload——故沿 D2 選填欄位模式在既有 GET /changes/{name} 回應增列選填 created（沿 ShowChange 的 schema+created 成對規則，成對才出現）、fromDiscussions、deltaCapabilities，由 server 於既有路由自 meta 與 scope 文件組裝（與清單 startedAt 同法；bridge 每個 verb 本就 full-scope 物化，成本同級）。仍不開新端點；舊 client 忽略新欄位、舊 server 缺欄位時 CLI 對應區塊維持缺席。

**D5：in-progress add 走新端點 POST /changes/{name}/in-progress，parity 凍結形狀不動。**
路由直通引擎 InProgressAdd：started_at 與 started_by 由 server 端認證身分蓋進 change meta（與 created_* 同一身分機制）；started_with 維持缺席——CLI 現無 agent 識別來源，fs 與 remote 一致，不發明新欄位。未知 change 時引擎回「未蓋章」，server 照樣回成功，CLI 維持靜默 exit 0（change-lifecycle 規格的凍結基線）。事件僅在實際蓋章時發布（引擎 outcome 既有語意）。

**D6：startedAt 上 wire 為選填欄位；看板推導補回真標記，fallback 保留。**
變更清單摘要 DTO 增選填 startedAt（camelCase、缺席時預設、None 不序列化），server 組清單時自 change meta 帶出。消費端兩處同步：desktop 系統匣的欄位推導（src-tauri）與前端看板的 remote payload 映射補 startedAt 欄位——前端 changeStage 推導本身不改（已讀 startedAt），只補資料管線（實作期確認：前端 remoteDataSource 為 wire 直通，DTO 增欄後 startedAt 自然上行、零程式改動，以測試釘住）。「完成數 > 0 即進行中」fallback 保留，它涵蓋手改 tasks.md 等繞過工具的寫入路徑。替代方案「server 派生 stage 回填」被否決：fs 模式無 server，前端推導必須存在，server 版是第三份實作而非收斂。

**D7：demo 於 remote 一行明確拒絕。**
remote 連線存在時直接報錯（非零 exit、stderr 說明 demo 僅限本機模式），比照 claim 在 fs 模式的 fail-loud 慣例。

**D8：crate 邊界。**
引擎零改動（speclink-core 不動）；DTO 歸 speclink-protocol；路由歸 speclink-server（薄轉發，只做 DTO 映射）；client 方法歸 speclink-remote；CLI 呈現歸 speclink-cli 的 remote 分支；桌面欄位推導歸 apps/desktop 的 src-tauri remote 模組與前端 payload 映射層。

## Implementation Contract

**Behavior（完成後可觀察行為）：**

- remote 模式下 speclink discuss new "主題" --slug some-slug 以該 slug 於 server 建檔；非法 slug 非零 exit、stderr 說明、server 不落檔。
- remote 模式下 speclink discuss discard some-slug 刪除 0 輪記錄；有輪時拒絕並提示 --force；--force 強制刪除。
- remote 模式下 speclink discuss link 與 speclink discuss seal 與 fs 模式同語意、同輸出形狀。
- remote 模式下 speclink show 對 change 與規格輸出與 fs 模式同形內容（server 資料）；找不到時語義化錯誤，不再回本機空 store 的結果。
- remote 模式下 speclink in-progress add 後，server 端該 change meta 含 started_at 與 started_by（認證身分）；CLI 輸出維持靜默 exit 0；後續 GET /changes 清單回應的該 change 帶 startedAt（CLI list --json 維持 fs 同形、凍結不帶 started_*）；desktop 看板與系統匣將該 change 列於進行中欄（即使完成數為 0）。
- remote 模式下 speclink demo 非零 exit 並說明僅限本機模式。
- fs 模式所有指令輸出逐位元不變。

**Interface / data shape：**

- 新端點：DELETE /discussions/{slug}（force 為 query 參數）、POST /discussions/{slug}/link、POST /discussions/{slug}/seal（body 帶 change 名稱）、POST /changes/{name}/in-progress。
- DTO 變更：建立討論請求增選填 slug；變更清單摘要增選填 startedAt；change 讀取回應（ChangeStatus）增選填 created（meta 的 schema+created 成對時才出現）、fromDiscussions、deltaCapabilities（供 show 組合，見 D4 實作期修正）。皆 camelCase、serde default、缺席不序列化。
- CLI：speclink discuss discard <slug> [--force]、speclink discuss link <slug> <change>、speclink discuss seal 參數與 fs 模式相同；show 與 in-progress add 無介面變化。
- docs/verb-contract.md 增列上述端點與 payload 形狀。

**Error handling：**

- 非法 slug：引擎錯誤 → server 語義化錯誤 → CLI 非零 exit＋stderr 原因；不落檔。
- discard 有輪未帶 force：引擎 guard → server 語義化錯誤 → CLI 非零 exit 並提示 --force。
- link、seal、in-progress 對不存在的討論或 change：沿引擎語意（in-progress 靜默成功；link、seal 語義化錯誤）。
- 未升級的舊 server 收到新動詞：404 → CLI 呈現語義化錯誤（既有 RemoteError 映射），不得 panic。

**Verification：**

- CLI 雙沙盒 parity 測試（crates/speclink-cli/tests/remote_verb_parity.rs 既有模式）：同內容下 fs 與 remote 的 show、discuss 四動詞輸出一致。
- crates/speclink-cli/tests/discuss_slug.rs 增 remote 分支案例；remote_write_path.rs 增 in-progress 與 discard 案例。
- server 路由測試：新端點的成功、guard 拒絕、404 形狀。
- desktop 欄位推導單元測試：startedAt 存在且完成數 0 → 進行中。
- 全套 cargo test 與 npm test 通過；fs 模式輸出以既有迴歸測試護住。

**Scope boundary：**

- In scope：上述七項與其測試、docs/verb-contract.md。
- Out of scope：Store trait、三個 TeamStore driver、引擎命令層、lifecycle 與 claimed_by 保留槽、slugify、bulk archive、server-web 前端。
