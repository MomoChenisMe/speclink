---
topic: release 版本附更新日誌，desktop 版本更新後首次開啟彈出、看過即止
slug: release-changelog-whats-new
status: promoted
promoted_to: release-changelog-whats-new
created: 2026-09-09
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: release 版本附更新日誌，desktop 版本更新後首次開啟彈出、看過即止

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者希望每個 release 版本有可看的更新日誌，desktop 在版本更新後首次開啟彈出更新內容，看過後不再自動彈出直到下一次更新。第一輪列假設後，使用者把「日誌從哪來」改成：做一個發版技能，執行即代表發版——壓 tag、從上次發版到現在的 commit 產生 changelog；changelog 放 git 儲存庫外，也包一份進 desktop。
未經 grill 階段：目標可驗證（每版有日誌、版號變了才彈一次），直接以假設清單開場。
掃描事實：Release 說明目前只有下載指南＋GitHub 自動產的「Full Changelog」比較連結（commit 直進 main、無 PR，自動 changelog 是空的）；repo 無 CHANGELOG 檔；v0.1.3→v0.2.0 有 95 個 conventional commit（繁中 description），封存變更的 commit 帶中文摘要；desktop 已有 localStorage「記版號、看過不再顯示」先例（assetPrompt.ts）與 modal 先例（MigrationDialog.tsx）；設定頁軟體更新卡已顯示現版號；latest.json 無 notes 欄位；發版手動步驟只有 Cargo.toml／tauri.conf.json 兩處版號＋tag；v0.1.x tag 為附註 tag、v0.2.0 為輕量 tag；release.yml 的 checkout 未設 fetch-depth（CI 內拿不到 tag 間的 commit 歷史）。
相關規格：desktop-app（桌面自動更新、指令檔過期提示）、desktop-release（更新描述檔隨 release 發布、Release 說明含下載指南）。相關變更：無。
Prior discussions: release-first-and-distribution, desktop-instruction-staleness-prompt

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-09)

**Focus**: 更新日誌從哪來、怎麼進 desktop、怎麼「看過即止」——七個節點的初始假設，與使用者對來源節點的改向
**Position**: 七節點假設清單開場，使用者把來源節點（A）改為「發版技能的產出」，其餘節點待驗。
- 決策樹：A 日誌來源 → B 進 app 方式 → C 彈出形式與看過判定 → D 首次安裝 → E 平常入口 → F 範圍 → G 語言
- 初始假設：A 手寫 CHANGELOG.md＋發版 fail-closed；B build 時打包進 desktop；C modal＋localStorage 記現版號（沿 assetPrompt.ts 先例）；D 首次安裝不彈只記版號；E 設定頁軟體更新卡加「更新日誌」入口；F 只做 desktop；G 內文只繁中
- 使用者改向 A：做一個發版技能，執行即發版——壓 tag、由上次 tag 到 HEAD 的 commit 產 changelog、changelog 放 git 儲存庫外、也包一份進 desktop
- 掃描事實支撐：Release 自動 changelog 目前是空的（無 PR）；95 個 conventional commit 太細需摘要；封存變更 commit 帶中文摘要可當素材；CI checkout 無歷史深度
**Open**: A′ 「放 git 儲存庫外」的確切意思與 desktop 打包的矛盾（CI 從 tag 建 desktop，內容不在 repo 就包不進去）；發版技能的落點（repo 本地技能 vs 產品技能）；技能的步驟與守門；B～G 各節點待確認

### Round 2 — assumptions (2026-09-09)

**Focus**: 使用者釐清「儲存庫外」＝CHANGELOG.md 進 repo、另做一份 JSON 打包進 desktop，比照 wadpilot——兩份檔案的真相歸屬與技能是否壓 tag
**Position**: 採 wadpilot 模式改造：JSON 為唯一真相、CHANGELOG.md 由腳本算出，發版技能負責壓 tag 但 push 前停一次。
- wadpilot 現況（`wadpilot/.claude/skills/release-notes/SKILL.md`、`packages/web/src/features/release-notes/`）：單一 release-notes.json（最新在前，version／date／sections 三類「新功能／修正／改善」）同時是版號真相、bundle 資料、彈窗比對依據；技能只寫檔＋commit，tag 由 CI 讀檔頂端版號建立；useWhatsNew 以 localStorage lastSeenVersion 比對頂端版號，dev 不自動彈，瀏覽模式關閉不寫記錄
- 差別 1（兩份檔）：JSON 放 apps/desktop/src/（Vite 直接 import），CHANGELOG.md 放 repo 根、由腳本自 JSON 產出，同 commit；release job 重算比對不一致即擋（比照 latest.json fail-closed）；同一支腳本把該版那筆畫成 markdown 接在 Release 說明下載指南後
- 差別 2（壓 tag）：release.yml 由 tag 觸發，技能壓 tag＝發版，與使用者「執行即發版」一致；守門＝在 main、樹乾淨、與 origin 同步、HEAD CI 綠、無平行 session 在途變更；push tag 前呈現版號＋日誌草稿由使用者點頭
- 版號真相四處（Cargo.toml、tauri.conf.json、JSON 頂端、tag）技能一次改齊，CI 同版守門加一處；bump 規則沿 wadpilot（feat→minor、fix→patch、0.x breaking 封頂 minor）
- 素材：封存變更 commit 優先一變更一條，其餘 feat／fix 逐筆，chore／test／refactor／docs 不進
**Ruled out**: CHANGELOG.md 為真相、解析成 JSON（解析 markdown 脆弱）；兩份都手寫（兩個真相必漂移）；技能止於寫檔、tag 交 CI（wadpilot 模式，與使用者「執行即發版」定義不合，且 speclink 管線本就由 tag 觸發）
**Open**: 使用者是否接受「JSON 真相、md 產出物」；技能落點（repo 本地 .claude/skills/release，是否鏡射 .agents）；B～G 節點：modal 形式、首次安裝不彈、設定頁入口、只做 desktop、內文只繁中；latest.json notes 欄位是否順手填（更新橫幅預覽）

### Round 3 — assumptions (2026-09-09)

**Focus**: 使用者確認「JSON 真相、CHANGELOG.md 腳本產出」後，收攏剩餘節點（技能落點、彈窗形式、首次安裝、平常入口、範圍、語言、dev 行為）
**Position**: 剩餘節點沿第一輪假設定案，加上 wadpilot 的兩個細節。
- 技能落點：repo 本地 `.claude/skills/release/SKILL.md`，不是產品技能（speclink-* 是裝進每個使用者專案的資產，發版技能只服務本 repo）；`.agents/skills/` 目前無本地技能先例，不鏡射
- 彈窗：modal（先例 MigrationDialog.tsx），兩種模式——版號變了自動開的「更新內容」模式，與設定頁手動開的「瀏覽全部」模式；只有自動開那種在關閉時寫 localStorage lastSeenVersion（先例 assetPrompt.ts 的鍵值做法）
- 自動開的內容：JSON 裡比 lastSeen 新的全部條目（跳版更新時一次看齊），lastSeen 不在清單內時只顯示頂端一筆
- 首次安裝（無 lastSeen）：不彈，直接記現版號；dev 建置（Vite DEV）永不自動彈——dev 與安裝版共用 localStorage 且同版號
- 平常入口：設定頁軟體更新卡加「更新日誌」按鈕（AppSettingsView.tsx 已有現版號與檢查更新）
- 範圍：只做 desktop；CLI／server 使用者看 CHANGELOG.md 與 Release 頁
- 語言：日誌內文只繁中；介面標籤走 i18n
- 詞彙：LANGUAGE.md 無此概念，正典詞定「更新日誌」
**Ruled out**: 產品技能（會裝進所有使用者專案）；橫幅（看過的判定模糊，且一版只彈一次 modal 重量可接受）；首次安裝也彈（對新使用者無意義且打斷選 workspace）；CLI 加 changelog 子指令（描述都是桌面行為，另立規格不值）
**Open**: latest.json 的 notes 欄位（更新橫幅預覽）——延後

## Conclusion

**Decision**: 做一個 repo 本地的發版技能 `/release`（`.claude/skills/release/`），執行即發版：守門（在 main、工作樹乾淨、與 origin 同步、HEAD CI 綠、無平行 session 在途變更）→ 由上次 tag 到 HEAD 的 conventional commit 判定版號（feat→minor、fix／perf→patch、0.x 階段 breaking 封頂 minor；只有 chore／test／refactor／docs 時警告）→ 把 commit 策展成白話繁中日誌（封存變更 commit 優先一變更一條，其餘 feat／fix 逐筆，雜訊不進；分組「新功能／修正／改善」）→ 呈現版號＋草稿給使用者點頭 → 一次改齊 Cargo.toml、tauri.conf.json、`apps/desktop/src/…/release-notes.json` 頂端條目（version／date／sections，最新在前）、cargo metadata 刷 Cargo.lock、腳本自 JSON 產出 repo 根 CHANGELOG.md → commit、壓 tag、push。JSON 是唯一真相；CHANGELOG.md 與 Release 說明的更新內容（接在下載指南後）都由同一支腳本從 JSON 畫出；release job 重算比對 CHANGELOG.md 不一致即擋、同版守門由三處擴為四處（含 JSON 頂端）。desktop 打包 JSON：啟動時現版號≠localStorage lastSeenVersion 就以 modal 彈出「更新內容」（列出比 lastSeen 新的全部條目；lastSeen 不在清單只顯示頂端），關閉即寫現版號；首次安裝不彈只記版號；dev 建置永不自動彈；設定頁軟體更新卡加「更新日誌」按鈕開瀏覽全部模式（關閉不寫記錄）。只做 desktop，日誌內文只繁中。
**Rationale**: 白話日誌需要人（或 Claude）把 95 個 commit 消化成幾句話，這件事只能在技能裡做、CI 裡沒有 Claude；一份真相加腳本衍生避免 md／json 漂移；技能壓 tag 對齊「執行即發版」且 speclink 管線本就由 tag 觸發，push 前停一次是唯一不可逆點的守門。
**Rejected alternatives**: 手寫 CHANGELOG.md 為真相（解析 markdown 成 JSON 脆弱）；md 與 json 都手寫（兩個真相必漂移）；技能止於寫檔、tag 交 CI 的 wadpilot 模式（與使用者定義不合）；changelog 放 git 外靠附註 tag 訊息或 Release 說明（CI 打包 desktop 拿不到、多網路依賴；使用者澄清本意是「進 repo 另出 JSON」）；執行期抓 GitHub 取日誌（離線看不到、API 頻率限制）；橫幅代替 modal（看過判定模糊）；首次安裝也彈；產品技能（會裝進所有使用者專案）；CLI changelog 子指令。
**Deferred**: latest.json 的 notes 欄位改由 JSON 條目填入、讓更新橫幅先預覽內容；`.agents/skills/` 鏡射本地技能（目前無先例，用到 Codex 發版時再補）；CLI／server 專屬的日誌入口。
**Capture to**: proposal（新變更）；LANGUAGE.md 新增詞條「更新日誌」（avoid：更新資訊、release notes、changelog 於使用者可見文案）
**Next**: /speclink-propose --from-discussion release-changelog-whats-new
