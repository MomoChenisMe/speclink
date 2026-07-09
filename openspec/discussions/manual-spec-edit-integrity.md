---
topic: 使用者擅自從文件系統調整 openspec 規格文件，繞過動詞破壞 SDD 流程可靠性
slug: manual-spec-edit-integrity
status: promoted
promoted_to: web-server-postgres
created: 2026-07-09
---

# Discussion: 使用者擅自從文件系統調整 openspec 規格文件，繞過動詞破壞 SDD 流程可靠性

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

個人本地模式下，desktop 與 CLI 的資料來源都是 repo 內 openspec/ 的規格文件（為相容 OpenSpec/Spectra，檔案即唯一真相、允許外部工具直改）。起初的擔憂是「沒開 desktop 就沒有 watcher 自動偵測，手改可能把規格調壞」；偵察後發現 desktop 每次開啟本來就整批重讀，watcher 只服務開著時的即時刷新，且解析層對壞檔刻意靜默退回預設（ChangeMeta::from_text、WorkflowConfig::from_text 均 unwrap_or_default）。第一輪後使用者修正框架：真正的威脅不是解析損壞，而是「語法合法但繞過動詞的狀態變更」——手改 .openspec.yaml 的生命週期欄位（如 started_at）無任何紀錄可查，或手刪 change 目錄讓已轉出的 discussion 懸空——整個 SDD 流程因此不可靠。

模式：assumptions（偵察命中 watch.rs、speclink-fs/lib.rs、model.rs、validate.rs、config.rs 等 5+ 檔案）。相關 in-flight change：discard-change-verb（其 Why 正是「只能手動刪目錄、繞過所有生命週期」）、web-server-postgres／web-role-views（伺服器模式，權威狀態的歸屬處）。相關既有機制：speclink validate（僅驗 delta spec 結構）、desktop 每 change 的 validate 動詞、git（openspec/ 在 repo 內，commit 級證據）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: 手改檔案的風險本質是什麼——解析損壞，還是流程繞過？
**Position**: 威脅被重新定位為「語法合法但繞過動詞的變更」，拆成兩個性質不同的子問題：
- 起初假設缺口在「壞檔被靜默吞掉」（ChangeMeta::from_text 對壞 YAML unwrap_or_default，model.rs:36；status/from_discussion 靜默消失無錯誤）——使用者修正：這不是主要擔憂。
- 子問題甲「未經動詞的狀態變更」：手改 .openspec.yaml 生命週期欄位（started_at/started_by 等）完全合法可解析，卻沒有任何紀錄可判別「這不是動詞做的」——偵測需要歷史（journal 或 git），無法從單一時點的檔案狀態判定。
- 子問題乙「懸空引用」：手刪 change 目錄後，已轉出該 change 的 discussion 的 promoted_to 指向不存在的東西——這可以從檔案現狀無狀態地檢出，不需要歷史。
- 佐證：in-flight change discard-change-verb 的 Why 正是「移除 change 沒有動詞、只能手動刪目錄、繞過所有生命週期」——刪除場景已有動詞化解法在路上，但「偵測手刪」仍是缺口。
**Ruled out**: 常駐 daemon 即時偵測——desktop 重開即整批重讀，「關著時偵測不到」不是真缺口；防止手改（鎖檔/sidecar checksum）——與「檔案即唯一真相、OpenSpec/Spectra 相容」的定位正面衝突。
**Open**: 目標保證等級是偵測、究責、還是防止？若需 journal，放本機 .speclink/（有跨機器誤報）還是 repo 內（可共享但同樣可手改）？懸空引用檢查放 validate（有 Spectra parity 顧慮）還是新動詞？

### Round 2 — assumptions (2026-07-09)

**Focus**: 單人（規格跟隨 repo、跨機器）與多人（防成員繞過引擎）兩種需求如何同時滿足？
**Position**: 不硬做一套通吃機制，改採分層模型——檔案模式做「不變量檢查＋git 證據」，強制力交給伺服器模式：
- 使用者確認威脅模型是「兩者都要」：單人時規格必須跟隨 repo（換機器開發），多人時要防成員繞過引擎，自覺矛盾、求解。
- 關鍵重塑：檔案模式該追的不是「來源正當性」（provenance）而是「狀態一致性」（invariants）——只要終態一致，怎麼到達的其實不影響流程續行；OpenSpec 相容本來就允許多寫入者。
- Layer 1（兩種情境通用）：無狀態不變量檢查——懸空 promoted_to/from_discussion、status 與所在位置矛盾（如 archive/ 內無日期前綴、meta 不可解析浮現而非吞掉）。desktop 載入時跑＋看板徽章、CLI 動詞供 agent、CI 可跑。
- Layer 2（單人跨機器）：git 就是 journal——不另造平行紀錄檔（可被手改、跨機器過期、多人誤報成災）；手改有 git diff/blame/restore 兜底。
- Layer 3（多人檔案模式）：共享分支的 CI gate 跑 Layer 1 檢查——擋不住本機亂改，但擋得住壞狀態進入共享分支；git 同步的團隊裡只有共享分支的狀態算數。
- Layer 4（真要防止）：伺服器模式（web-server-postgres 已在路上）持有權威狀態——不在檔案模式重造它的保證。
- 佐證：本機 journal 在多人情境必然誤報（同事機器上合法動詞產生的變更本機沒見過）；desktop 內嵌 git 呼叫已知每次 ~3s（跨機器備忘），故 desktop 層只做純檔案讀取的無狀態檢查。
**Ruled out**: repo 內平行 journal——本身可手改、無強制力、多人誤報、與 git 職責重疊；desktop 內做 git 歷史比對——已知效能坑（spawn git ~3s 凍主執行緒）。
**Open**: 樞紐問題待使用者裁定——「狀態一致但來歷不明」的變更（如手改 started_by 冒名）是否可接受不偵測？不可接受則誠實答案是伺服器模式而非更多檔案模式機關。檢查落點：擴充 validate（Spectra parity／CLI 輸出回歸顧慮）vs 新動詞（doctor 類）。

### Round 3 — assumptions (2026-07-09)

**Focus**: 檔案模式的取捨定案，以及「headless 遠端伺服器＋desktop 遠端模式」提案的可行性。
**Position**: 使用者裁定檔案模式維持現狀＋明載限制，強保證由重塑後的遠端模式承接——headless server 沿用 desktop 當 GUI，與既有架構高度契合：
- 使用者對樞紐問題的裁定：個人模式下狀態（started_by 等）維持記錄於文件、跟隨 repo（跨機器是硬需求）；團隊沿用檔案模式屬「知情使用」，不能接受限制就轉遠端模式。
- 提案評估：desktop 前端已隔著 SpeclinkDataSource 介面（tauriDataSource 是其一實作），web-server-postgres 提案本就預告「HTTP adapter 對應動詞契約端點」——新增 httpDataSource 即讓 desktop GUI 直連 server，等於原計畫的 web GUI 前端寄宿在桌面殼裡，server 轉頭做 headless，③ 的範圍縮小而非擴大。
- 完整性收益：遠端模式下 Postgres 即真相、沒有可手改的檔案面；PAT 身分讓 started_by 不再是自我宣告（冒名問題在此模式根治）；轉換語意由 server 端點強制（verb-contract 既有 If-Match/409、claim/ownership 雛形）。
- 已知設計項（歸 ③ propose 階段）：即時刷新的 server push（LISTEN/NOTIFY → SSE/WS，提案已列）；desktop 遠端模式下 agent 面板（②）的動詞須走 CLI remote 對同一 server，GUI 與 agent 兩條線的遠端設定要一體。
**Ruled out**: 在檔案模式加 journal／防篡改機關——使用者裁定維持檔案真相＋知情使用，強保證不在檔案模式硬做；③ 同刀交付 web GUI——改 headless，瀏覽器版 GUI 延後到情境 1/2 真需要零安裝時再加（React 元件庫共用，成本延後不消失）。
**Open**: 情境 1 的 PO 角色原設定用瀏覽器零安裝，改需安裝 desktop 是否可接受；「開箱即用」與 PostgreSQL 外部依賴的張力（docker-compose 打包？）；Layer 1 最小不變量檢查（懸空引用＋meta 解析失敗浮現）做或遞延。

### Round 4 — assumptions (2026-07-09)

**Focus**: 三項 roadmap 修正——定位文件化、agent 接線的真實情境、瀏覽器版 GUI 的去留。
**Position**: 定位明載＋agent 移出 speclink desktop＋瀏覽器版取消，desktop remote 成為遠端 GUI 的唯一形態：
- 定位明載於文件：本機個人模式供個人與小型團隊使用；小型團隊須知情既有限制（狀態可被手改且無紀錄、started_by 為自我宣告、究責僅 git）；不能接受 → 轉遠端模式。
- Agent 接線修正（desktop-acp-agent 前提有誤）：speclink desktop 不內嵌 agent 面板。真實情境有二——(A) agent 前端是獨立的 Copilot SDK desktop app（與 speclink 非同一 app、不在 speclink 交付範圍），經 speclink CLI remote 呼叫 headless server 存取；(B) Copilot SDK 作為後端，server 側以 tool＋SDK 直接串接（原 ④ web-agent-channel 領域，繼續有效）。speclink 側交付縮為「CLI remote 對真 server 可用」。
- ⑤ web-role-views（PO 瀏覽器版）取消：desktop remote 模式取代零安裝需求，PO 改裝 desktop app；React 元件庫保留日後補瀏覽器宿主的可能。
- 設定落點：desktop 設定頁既有三頁簽（config.yaml／.speclink.yaml／本機設定）——server URL/repo 入 .speclink.yaml（隨 repo 共享正確）；PAT 沿用 CLI remote 既有正典：使用者層級 config 目錄 YAML（origin→token、0600、SPECLINK_TOKEN 可覆蓋；auth.rs 明載 never inside the repo），絕不入 repo 檔案。
**Ruled out**: speclink desktop 內嵌雙 ACP agent 面板（② 原規格前提）——agent 前端獨立於 speclink 或在 server 側；瀏覽器版 web GUI（⑤）——desktop remote 取代。
**Open**: 無（server 與 desktop 遠端模式同刀或分刀等切刀細節留 ingest/propose）。

## Conclusion

**Decision**: 三段式。(1) 檔案模式（本機個人模式）維持現狀：狀態（started_by 等）記錄於文件、跟隨 repo；不加 journal、防篡改、常駐偵測。定位明載於文件：供個人與小型團隊使用，小型團隊須知情限制（狀態可被手改且無紀錄、started_by 為自我宣告、究責僅 git），不能接受則轉遠端模式。(2) 重塑第 ③ 刀 web-server-postgres：交付 headless 開箱即用的自架 server（不含 web GUI，部署形態傾向 docker-compose 打包 Postgres）；desktop 新增遠端模式——以 httpDataSource 實作 SpeclinkDataSource 直連動詞契約端點，設定落 desktop 設定頁（server URL/repo 入 .speclink.yaml 頁簽；PAT 沿用 CLI 使用者層級憑證存放，絕不入 repo）。(3) Roadmap 連動：⑤ web-role-views 取消（desktop remote 取代 PO 瀏覽器版）；② desktop-acp-agent 前提修正——speclink desktop 不內嵌 agent，agent 為外部獨立 Copilot SDK app（經 CLI remote 連 server）或 server 側 SDK 直接串接（④ 領域），需另行 ingest；④ web-agent-channel 對應「Copilot SDK 作為後端」情境，繼續有效。
**Rationale**: 檔案即 API（OpenSpec/Spectra 相容）使「防止手改」結構上不可達，且「狀態一致但來歷不明」已裁定可接受——檔案模式追一致性與知情使用即可，強制力唯一可達處是 server 權威。desktop 前端既有 SpeclinkDataSource 縫使「沿用 desktop 當遠端 GUI」幾乎免費（HTTP adapter 原計畫本就要做），server 得以 headless、③ 範圍縮小而非擴大；PAT 身分讓 started_by 不再自我宣告，冒名問題在遠端模式根治。
**Rejected alternatives**: 平行 journal（repo 內可手改無強制力、本機版多人情境必然誤報且跨機器即斷、與 git 搶職責）；鎖檔/sidecar checksum（違反檔案即真相的相容性定位）；常駐 daemon 偵測（desktop 重開即整批重讀，「關著時偵測不到」非真缺口）；③ 同刀交付瀏覽器 web GUI（desktop remote 取代，元件庫保留日後補瀏覽器宿主的可能）；speclink desktop 內嵌雙 ACP agent 面板（agent 前端獨立於 speclink，或在 server 側）。
**Deferred**: Layer 1 最小不變量檢查（懸空 promoted_to、meta 解析失敗浮現而非靜默吞掉——對純個人模式亦有防呆價值、成本低，可日後另立小 change）；server push 傳輸形式（Postgres LISTEN/NOTIFY → SSE/WS）；server 與 desktop 遠端模式同刀或分刀；docker-compose 具體形態；定位聲明的文件落點（README 或產品文件，ingest 時定案）。
**Capture to**: proposal（web-server-postgres，經 ingest 折入重塑）
**Next**: /speclink-ingest web-server-postgres（已 link 本討論）；desktop-acp-agent 前提修正與 web-role-views 收場另行處理
