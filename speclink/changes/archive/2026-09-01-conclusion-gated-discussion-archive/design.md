## Context

連帶封存邏輯位於 speclink-core 的 archive 流程：change 封存時逐一檢查來源討論，唯一守門是「無其他在途變更引用」，隨後呼叫的封存函式不看討論狀態與結論。結論偵測已有現成材料：core 的 conclusion_text 回傳 Conclusion 段內文（佔位註解不算），且不依賴 status——promoted 討論寫完結論後 status 仍為 promoted。呈現鏈上，DiscussionInfo（protocol）與 DiscussionItem（packages/ui）皆無結論欄位；promotedTo 欄位已有「server 於 route 邊緣組裝、引擎列表結構不動」的前例。看板討論欄與 tray 皆以 status 是否為 promoted 二分。利害關係人：跑 SDD 的開發者（討論被誤封存後動詞全拒）、看板使用者（分不出轉出後是否還在討論中）。

## Goals / Non-Goals

**Goals:**

- 未有結論的討論不隨 change 封存，轉出後仍可 add-round 與 conclude。
- conclude 在全數轉出變更已封存時順手封存討論，兩個方向都有人收尾。
- desktop（看板、tray 面板、系統匣）能分辨「已轉出・尚無結論」與「已轉出且已有結論」。
- CLI 既有輸出（discuss list／show、archive 對已有結論來源的情境）逐位元不變。

**Non-Goals:**

- 不做 unarchive 動詞、不以討論阻擋 change 封存、不回溯遷移歷史誤封存記錄（proposal Non-Goals 已列）。
- 不改已封存頁的討論節。
- 標示的視覺樣式微調不回寫 spec。

## Decisions

**D1 守門判準＝結論內文，不是 status。** 連帶封存過濾器在既有「無其他在途變更引用」之上，加「Conclusion 已寫入」條件，判斷唯一落點是 core 既有的 conclusion_text（回傳 Some 即已寫）。否決以 status 判斷：promoted 討論寫完結論後 status 仍為 promoted，status 分不出兩態。守門落在 speclink-core 的封存流程（領域演算法歸 core，經 Store trait 讀取，不寫死儲存媒介），local 與 remote（server 的 archive 端點）走同一條 core 路徑，無平行實作。

**D2 conclude 閉環的觸發條件＝「promoted_to 非空」且「無在途變更引用」。** conclude 寫入結論後，讀 frontmatter 的 promoted_to：非空、且沒有任何在途變更的 from_discussion 引用本 slug 時，順手呼叫既有的討論封存函式。否決掃描封存區比對 promoted_to 逐名驗證：需要列舉 archive 目錄，成本高且對判斷無增益——「非空且無在途引用」已涵蓋主線情境。已知邊界（接受）：change 經 link 併入但尚未 seal（promoted_to 未寫）就先封存，之後才 conclude——此時不觸發閉環，討論留在途，使用者以 discuss archive 手動收尾；此序偏離技能檔規定的 conclude→link→ingest 順序，屬罕見狀態且可救援。

**D3 concluded 欄位＝邊緣組裝，引擎列表結構不動。** 沿 promotedTo 前例（server-verb-api 已明訂該模式）：core 新增單一查詢函式 discussion_concluded（包裝 conclusion_text 為 bool），server route 邊緣與 host bridge 各自呼叫同一函式組裝欄位——契約唯一實作落點在 core，兩端只是呼叫點（回歸對照 crates/speclink-cli/tests/it/remote_verb_parity.rs 不受影響，因 CLI JSON 不帶此欄位）。protocol 的 DiscussionInfo 與 packages/ui 的 DiscussionItem 增選填 concluded 欄位：serde 為 Option<bool>、camelCase、default None、None 時省略序列化；邊緣組裝時每筆討論恆填 true／false，欄位缺席僅發生在舊 server——client 據此容錯（見 D4）。否決把欄位塞進引擎列表結構：會改動 CLI discuss list --json 的逐位元基線。

**D4 UI 分區判準＝「promoted 且 concluded 為 true」才收合；欄位缺席退回現行為。** 看板討論欄、tray 面板、系統匣三處同一判準：concluded === true 的 promoted 討論進「已轉出」收合列；concluded === false 的 promoted 討論留上區全尺寸卡、帶「已轉出・尚無結論」狀態標，並計入欄計數徽章的 active 數。concluded 欄位缺席（舊 server）時退回現行「promoted 一律收合」，避免把已有結論的討論誤標成尚無結論。標示文案定為「已轉出・尚無結論」，經 i18n 資源落地；否決只在收合列加小標（討論已否決：治不了進行中的卡從上區消失）。

**D5 conclude 輸出增量＝只在觸發時出現。** 人眼輸出：閉環觸發時多一行「已順手封存討論」訊息；--json payload 增 autoArchived 欄位（camelCase、serde skip 於 false）——未觸發時人眼與 JSON 皆逐位元不變，回歸對照不破。server 的討論結論端點以同一 core 結果回填同名欄位。

**D6 技能檔措辭對齊走既有 asset 三連動，discuss 與 improve 兩份 asset 同批。** discuss 與 improve 技能 asset（crates/speclink-core/assets/skills/）內文中「最後一個變更封存時自動封存討論」的敘述補上「且結論已寫入」；improve 走同一套討論機制，其 fan out 段帶同一句舊承諾。improve 的正典流程先寫結論再轉出，該句在其語境內原本不假——但句子描述的是引擎機制，機制加了守門條件，描述跟上；兩份 asset 搭同一次改動，避免之後為一句話再 bump 一版。隨改動 bump ASSET_VERSION、再生 golden、更新 assets.lock，再以 speclink update 再生兩工具（claude／codex）的 SKILL.md。discuss-skill 與 improve-skill 的 spec 皆未明文鎖定此句措辭，故無該兩 capability 的 spec delta。引擎面不需擴充：守門（archive 來源討論過濾器）與 conclude 閉環的判準只讀引用與結論內文、不讀 kind，improve 討論已由既有實作涵蓋，不另補 improve 專屬測試（程式碼無 kind 分支可測）。

## Implementation Contract

**行為（使用者可觀察）：**

- 討論 d 轉出 change c1 後、未寫結論時封存 c1：d 留在 openspec/discussions/，discuss add-round 與 conclude 照常成功；archive 的人眼輸出與 --json 的隨行封存清單不含 d。
- 之後對 d 執行 discuss conclude：結論寫入、status 保持 promoted，且因 promoted_to（c1）非空、無在途引用，d 被順手移入 openspec/discussions/archive/；stdout 多一行告知，--json 含 autoArchived: true。
- conclude 時仍有轉出變更在途：只寫結論，不封存，輸出無增量；最後一個變更封存時守門看到結論已寫，連帶封存照舊（輸出與現行為逐位元一致）。
- desktop 看板：concluded 為 false 的 promoted 討論呈上區全尺寸卡＋「已轉出・尚無結論」標；concluded 為 true 的 promoted 討論進欄底收合列。tray 面板與系統匣討論列表同判準。

**介面／資料形狀：**

- core：discussion_concluded(store, slug) -> bool（包裝 conclusion_text）；封存流程的來源討論過濾器新增此條件；conclude 回傳值攜帶 auto_archived 事實供 CLI 與 server 呈現。
- protocol DiscussionInfo 與 ui DiscussionItem：增 concluded（Option<bool>／可缺席布林），camelCase，None 省略；GET /discussions 與 host bridge 每筆恆填。
- CLI discuss conclude --json：增 autoArchived（僅 true 時出現）。
- CLI discuss list／show 的人眼與 --json：逐位元不變。

**寫入順序與失敗模式：**

- conclude 為兩步寫入：先寫結論（既有語意），再嘗試閉環封存。封存步失敗時結論寫入不回滾——可觀察狀態為「已結論、仍在途」，重跑 discuss archive 即可收尾；stderr 說明封存步失敗原因、exit code 非零。此半套狀態為明訂接受：結論是主要語意，封存是收尾便利。
- 連帶封存守門對「討論記錄讀取失敗」視同未有結論（不隨行封存）——寧可留在途讓人處理，不吞進 archive。

**驗收判準：**

- speclink-core 單元測試：未結論來源討論不隨行封存（補上現行零覆蓋的情境）；已結論 promoted 來源討論隨行封存輸出與變更前逐位元一致；conclude 閉環在「promoted_to 非空＋無在途引用」觸發、在「仍有在途引用」不觸發。
- CLI 整合測試（crates/speclink-cli/tests/it/）：conclude 觸發時 stdout 與 --json 形狀；未觸發時輸出比對既有基線。
- server 測試：GET /discussions 每筆含 concluded；討論結論端點回填 autoArchived；remote 動詞對等測試維持綠。
- 前端 vitest：DiscussionColumn 對 concluded false／true／缺席三態的分區與標示；TrayPanel 與 tray 選單同判準。
- 手動驗收：desktop 看板實際呈現未結論轉出卡於上區（列為 [M] 任務）。

**範圍邊界：**

- In scope：core 守門與閉環、protocol／server／host 欄位、packages/ui 與 apps/desktop 分區與標示、discuss 與 improve 技能 asset 措辭與三連動（同一次版號變動）。
- Out of scope：unarchive 動詞、封存阻擋、已封存頁、歷史記錄遷移、標示樣式微調。

## Risks / Trade-offs

- **回歸對照（golden 與 CLI 測試）**：archive 與 conclude 的既有情境輸出必須逐位元不變——守門與閉環的輸出增量都設計成「僅於新情境出現」；asset 內文改動必然波及 ASSET_VERSION／golden／assets.lock，按既有三連動程序處理，speclink update 再生的 SKILL.md 於收尾 commit 以 git status 盤點帶上。
- **跨平台**：無新增 git 或路徑語意——閉環封存重用既有討論封存函式（同日撞名由 store 解決）；Windows CI 依「新目標第一次跑」原則觀察首輪。
- **舊 server 相容**：concluded 缺席時 UI 退回現行為，代價是接舊 server 的使用者看不到新分區——可接受，欄位隨 server 升級自然出現。
- **效能**：邊緣組裝對每筆討論多一次記錄讀取以判結論；討論數量級小（個位數到數十），不做快取。
