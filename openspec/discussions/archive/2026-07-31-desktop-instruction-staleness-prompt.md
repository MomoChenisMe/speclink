---
topic: 開啟 desktop 時檢查專案的 speclink skills 與 CLAUDE.md/AGENTS.md 是否過期並提示更新
slug: desktop-instruction-staleness-prompt
status: promoted
promoted_to: desktop-instruction-staleness-prompt
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 開啟 desktop 時檢查專案的 speclink skills 與 CLAUDE.md/AGENTS.md 是否過期並提示更新

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出：desktop 開啟專案時，檢查該專案的 speclink skills 與 CLAUDE.md/AGENTS.md 受管區塊是否為舊版（原設想以 skill frontmatter 的 metadata.version 與 marker 版號比對），過期則提示使用者更新；不更新可保留自行調整的 skill 內容，並需釐清不更新的系統性風險。

模式：assumptions——偵察命中足量原始碼：crates/speclink-core/src/init.rs（MARKER_VERSION 與 update()）、crates/speclink-core/src/skills.rs（skill frontmatter 生成）、apps/desktop/core/src/project.rs（open_project 三態探測）、apps/desktop/src-tauri/src/cli_install.rs（sidecar CLI 佈署）。

關鍵前提修正：skill frontmatter 的 metadata.version 是寫死的 "1.0"（skills.rs:232、266），從未隨內容遞增——逐 skill 版號比對今天不可行；真正有版本語意的是 CLAUDE.md/AGENTS.md 的 <!-- SPECLINK:START v1.2.0 --> marker（init.rs:9 MARKER_VERSION）。

相關 changes/specs：speclink list 目前無進行中 change、無其他開放討論。desktop 已直接依賴 speclink-core（可行程內 render 期望內容），CLI 已有 update 動詞（整套再生受管檔、冪等）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: 過期偵測的機制基礎——拿什麼比對才能同時偵測過期又不誤傷使用者自訂
**Position**: 以版本戳比對為基礎，不做內容比對；五項假設呈給使用者：
- 單一版本源：MARKER_VERSION 同時蓋進 marker 區塊與 skill frontmatter，資產變動即 bump（現況 skill version 寫死 1.0 無版本語意，skills.rs:232）
- 偵測邏輯放 Rust core、desktop 只加唯讀 probe（desktop 直依 speclink-core，open_project 已是零寫入探測慣例）
- 「更新」動作復用既有 speclink update()（init.rs:327，冪等整套再生），不另寫覆蓋邏輯
- 「保留自訂」第一版整包跳過、不逐檔挑選——系統無歷史 render，「內容不同」無法區分過期與自訂；版本戳比對刻意繞開此問題
- 不更新的系統性風險定位為「工作流語意漂移」（舊 skill 不知新動詞如 discuss link）而非引擎故障，提示可溫和不強制
**Ruled out**: 內容/hash 比對偵測——使用者自訂即被誤判為過期，與「保留自訂彈性」的目標直接衝突
**Open**: 版本粒度怎麼切（skills／CLAUDE.md+AGENTS.md／desktop+cli 各自獨立 vs 全系統統一——使用者本輪即拋出此問）；使用者按「保留現狀」後決定記在哪、是否每次開專案重問

### Round 2 — assumptions (2026-07-31)

**Focus**: 版本粒度——skills／CLAUDE.md+AGENTS.md／desktop+CLI 各自獨立版號，還是全系統統一
**Position**: 兩層制定案（使用者下一輪已以 MARKER_VERSION 為前提續問，視為採納）：
- 產物層：skills 與 CLAUDE.md/AGENTS.md 受管區塊共用單一 MARKER_VERSION，只在 render 內容變動時 bump，同時蓋進 marker 區塊與 skill frontmatter
- 程式層：desktop＋CLI 維持既有 release 同版機制（sidecar 佈署，cli_install.rs 已比對 app_version 與 deployed_version_output），零新增
- 設計尺度：版本戳唯一職責是回答「有無更新可裝」，應恰在 render 內容變動時改變；「哪些檔會被覆蓋、哪些有自訂」的精確度由提示當下的 render-time diff 提供，與版本粒度解耦
**Ruled out**: 產物戳＝release 版號（每次發版必跳過期提示，內容沒變也狼來了；golden 每版翻動）；三層獨立（只買到提示文案精確度，update() 整套再生下行為無差；兩套 bump 紀律讓靜默失效面加倍）
**Open**: 漏 bump 的紀律如何強制——尤其改動者常是 LLM agent（使用者本輪即問）；使用者按「保留現狀」後決定記在哪、是否重問

### Round 3 — assumptions (2026-07-31)

**Focus**: 漏 bump 的紀律如何強制——改動者常是 LLM agent，如何確保動了資產必 bump MARKER_VERSION
**Position**: 不依賴記憶性紀律，把紀律做成 CI 紅燈（使用者已確認採納）：
- speclink-core 加 version–hash 鎖定測試＋提交進 repo 的鎖定檔：render 輸出 hash ≠ 鎖定檔 hash 時，MARKER_VERSION 必須 ≠ 鎖定檔版本，否則紅燈
- 失敗訊息即給 LLM 的指令：寫明「遞增 init.rs 的 MARKER_VERSION 後跑 UPDATE_LOCK=1 重生鎖定檔」——agent 工作迴圈必跑測試，做錯被擋下並當場被告知修法
- 專案已有同款先例（render_golden＋UPDATE_GOLDEN=1 三處同步），鎖定測試可掛在同一測試檔共用 render 管線；繼承既有地雷：重生必須在乾淨樹上跑
- 輔助層：CLAUDE.md 三處同步備忘後補半句「動 assets 或 marker 模板時同步 bump MARKER_VERSION」
- 只 bump 沒改內容＝綠燈無害；改內容沒 bump＝紅燈擋下
**Ruled out**: Claude Code hooks 強制——per-harness 不可攜（Codex 等其他 harness 的 agent 吃不到），測試對所有貢獻者一視同仁
**Open**: 使用者按「保留現狀」後決定記在哪（desktop 本地設定 vs .speclink.yaml vs 每次重問）

### Round 4 — assumptions (2026-07-31)

**Focus**: 使用者按「保留現狀」後，決定記在哪
**Position**: 記在 desktop 本地設定，鍵值「專案路徑 → 已略過的 MARKER_VERSION」（使用者已確認）：
- 略過語意是 per 版本：記「這個專案我看過 vX、決定不要」；下次 bump 出新版會再提示（新內容＝新決定），同版本內不重複問
- 零 repo 污染；提示是 UI 層關切，決定留在 UI 層；desktop 已有本地持久化慣例可搭（store.ts persistTabs）
- 「不跨機器」為可接受代價：每台機器對每個版本只問一次，且自訂內容本就因人因機而異
**Ruled out**: 寫入 .speclink.yaml——使用者剛按「不要動我的檔案」卻回應以改檔弄髒 git status，自相矛盾，且把個人 UI 偏好烙進團隊共用設定；每次都問——對重度自訂使用者每次開專案轟炸，打臉保留彈性的目標
**Open**: 使用者追加的 UI 收尾題——側欄底部 app 版號（v0.1.0）的擺放位置

### Round 5 — assumptions (2026-07-31)

**Focus**: 附帶 UI 收尾——側欄底部 app 版號（v0.1.0）的擺放位置
**Position**: 方案 A——刪除側欄版號，設定頁軟體更新卡為 app 版號唯一住所（使用者裁定）：
- 常駐版號的三種工作全數落空：更新感知（唯一高頻工作）被 UpdateBanner 主動橫幅接走；更新後確認與回報 bug 皆低頻主動、一鍵進設定，且完整回報需 app＋CLI 雙版號、只有設定頁齊全
- 按「稍後」延後更新後，側欄版號仍顯示現版，連 pending 更新的殘留提醒都做不到——「主頁可見」的原始理由（App.tsx:566 註解）已空心化
- 實作＝刪除 App.tsx:566-571 的條件渲染；currentVersion 值設定頁仍用，非死碼
**Ruled out**: B 收進設定列 trailing 插槽——成本經修正後近零（NavItem 已有 trailing 插槽，已封存 114 徽章即用它），兩案以品味分勝負，使用者裁定不留常駐版號；C 另立視窗狀態列——為一行字新增常駐 chrome，過度工程
**Open**: 無——全部節點已解，進結論

## Conclusion

**Decision**: desktop 開專案時做指令檔過期偵測＋溫和更新提示，機制五件套：
- 偵測＝版本戳比對：兩層版本制——產物層（skills＋CLAUDE.md/AGENTS.md 受管區塊）共用單一 MARKER_VERSION，只在 render 內容變動時 bump，同時蓋進 marker 區塊與 skill frontmatter（現況 skill version 寫死 "1.0" 無版本語意，需一併修正）；程式層 desktop＋CLI 維持既有 release 同版機制，零新增
- 偵測邏輯放 Rust core（desktop 直依 speclink-core），desktop 只加唯讀 probe，開專案時順帶回報；「更新」動作復用既有 speclink update()（冪等整套再生）
- 保留自訂＝整包跳過；「保留現狀」決定記在 desktop 本地設定，鍵值「專案路徑 → 已略過的 MARKER_VERSION」，per 版本語意：同版不重問、下次 bump 再提示
- bump 紀律以 CI 紅燈強制：speclink-core 加 version–hash 鎖定測試（render hash 變動但 MARKER_VERSION 未 bump 即紅燈，失敗訊息寫明修法），輔以 CLAUDE.md 備忘一行；重生鎖定檔繼承「乾淨樹」地雷
- 不更新的系統性風險定位：工作流語意漂移（舊 skill 不知新動詞，如 discuss link），非引擎故障——硬失敗（CLI 拒未知 flag）可見可恢復，故提示溫和不強制
附帶 UI 決定：刪除側欄底部常駐版號（App.tsx:566-571），設定頁軟體更新卡為 app 版號唯一住所——UpdateBanner 接走更新感知後，「主頁可見」已無功能性工作。

**Rationale**: 版本戳唯一職責是回答「有無更新可裝」，應恰在 render 內容變動時改變——這把過期偵測與使用者自訂解耦（改內文不動戳記＝同版含自訂，不誤報），也把提示噪音降到只在真有新內容時出現；「哪些檔會被覆蓋」的精確度由提示當下的 render-time diff 提供，與版本粒度解耦。紀律不靠記憶（人與 LLM 同樣不可靠）而靠測試紅燈，失敗訊息本身就是給 agent 的修復指令，與既有 render_golden＋UPDATE_GOLDEN 模式同構。

**Rejected alternatives**: 內容/hash 比對偵測（自訂即誤報過期，與保留彈性目標直接衝突）；逐 skill 獨立版號（bump 紀律面加倍、漏 bump 靜默失效）；產物戳＝release 版號（每次發版必跳假過期提示）；三層版本制（只買提示文案精確度，update() 整套再生下行為無差）；保留決定寫入 .speclink.yaml（按「不動我的檔案」卻回應以改檔，且個人 UI 偏好污染團隊設定）；每次都問（轟炸重度自訂使用者）；hooks 強制 bump（per-harness 不可攜）；側欄版號收進設定列 trailing 插槽（成本近零但品味裁定不留）、另立狀態列（過度工程）。

**Deferred**: 提示的具體形態（對話框／橫幅／卡片）與文案（遵循 LANGUAGE.md）；render-time diff 的呈現深度（僅列檔名 vs 展示內容差異）；marker 區塊被使用者整個移除時的偵測行為；既有專案首次過渡的細節（現存 skill frontmatter 全為 "1.0"，偵測讀 marker 或 frontmatter 的取捨）——皆留給 propose/design 階段。

**Capture to**: proposal（轉為變更；側欄版號刪除為附帶小任務，建議併入同一變更的 tasks，與版本呈現同一主題、同版發布）

**Next**: /speclink-propose --from-discussion desktop-instruction-staleness-prompt
