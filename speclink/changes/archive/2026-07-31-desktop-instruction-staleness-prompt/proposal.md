## Why

專案裡由 speclink 生成的指令檔（skills、CLAUDE.md/AGENTS.md 受管區塊）在 CLI／desktop 升版後不會跟著更新，使用者無從得知手上的技能文字已過期；過期技能造成的是靜默的工作流語意漂移（舊 skill 不知道新動詞，例如 discuss link 出現前的技能會讓已結論討論永遠留在看板上），而非可見的引擎錯誤，等使用者察覺時已繞了遠路。同題的另一面是「從未安裝」：tools 清單宣告了工具、指令檔卻不存在（如 clone 下來但指令檔未進版控的專案）——這樣的專案在 desktop 開啟時同樣毫無提示，AI 代理拿不到任何 speclink 技能（討論 desktop-workspace-auto-init 定案納入本變更）。同時，現行 skill frontmatter 的 metadata.version 寫死 "1.0"、從未隨內容遞增，任何版本比對機制都無從建立——這是本變更要先修的地基。

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM，使用情境有二：desktop 開啟專案時的工作區維運面（偵測與提示，不屬於任一 workflow 階段、在所有階段之前把關），以及 speclink 本身開發者修改內嵌資產時的版本紀律面。

## What Changes

- **產物層單一版本源**：內嵌資產的版本以 MARKER_VERSION 為唯一權威——skill frontmatter 的 metadata.version 從寫死 "1.0" 改為蓋入 MARKER_VERSION，與 CLAUDE.md/AGENTS.md 受管區塊的 marker 版號同源；MARKER_VERSION 只在 render 內容變動時 bump。程式層（desktop＋CLI）維持既有 release 同版機制，零新增。
- **過期偵測（引擎側）**：speclink-core 新增唯讀的過期探測——讀取工作區受管檔上的版本戳、與當前 MARKER_VERSION 比對，回報四態（缺失／過期／現版／無法判定）與「哪些受管檔的內容與現版 render 不同」。缺失＝tools 清單宣告的指令檔不存在（從未安裝），與「檔案在但標記被移除＝退出受管、不提示」明文區分。純讀不寫，desktop 直嵌呼叫，不新增 CLI 子指令或旗標。
- **desktop 開專案提示**：開啟本地專案時搭載過期探測；偵測到過期或缺失時以非阻斷方式提示，主動作依態分文案——過期為「更新」、缺失為「安裝」，同呼叫既有 update() 冪等整套再生（執行前列出將被新建或覆蓋且內容有異的檔案）——另有「保留現狀」。保留決定記在 desktop 本地持久化（鍵值：專案路徑 → 已略過的 MARKER_VERSION），同版不重問、下次 bump 出新版再提示；不寫入 .speclink.yaml 或 openspec/config.yaml。
- **bump 紀律 CI 紅燈**：speclink-core 新增 version–hash 鎖定測試與提交進 repo 的鎖定檔——render 輸出 hash 與鎖定檔不符而 MARKER_VERSION 未變即測試失敗，失敗訊息寫明修復步驟（遞增版號＋以環境變數重生鎖定檔）；鎖定檔重生繼承 golden 的「乾淨樹」慣例。紀律全數由該測試的失敗訊息承載（訊息寫明 bump 與重生步驟，並指明 UPDATE_GOLDEN 不會更新鎖定檔）——原規劃的 CLAUDE.md 開發備忘一行於實作期取消：該備忘已於 c0303d8 被移除，CLAUDE.md 現僅剩引擎受管區塊，手寫內容會被 update 覆蓋。
- **附帶 UI 收尾**：刪除側欄底部常駐版號，設定頁軟體更新卡成為 app 版號唯一住所（更新感知已由 UpdateBanner 橫幅負責）。

相容性影響：skill frontmatter 的 metadata.version 值改變使所有內建技能的 render 輸出變動，四份 golden snapshot（claude／codex／neutral-cli／neutral-tool-call）與 repo 內兩套技能實例（claude、codex 工具）同批刻意再生並於此記載；MARKER_VERSION 因本變更自身的 render 內容變動而 bump（本變更是自身紀律的第一個適用者）。update 指令的人眼輸出與 --json shape 不變；既有專案無須遷移——舊戳記（含 "1.0" frontmatter 與舊 marker 版號）正是偵測要識別的對象。實作排程注意：進行中變更 discuss-propose-from-docs 亦觸碰內嵌資產與同四份 golden，建議其先落地、本變更後實作，一次 bump 涵蓋兩者的資產變動。

## Non-Goals

- 不做逐檔挑選更新或逐檔 diff 檢視 UI——第一版為整包更新或整包保留；系統無歷史 render，內容差異無法區分「過期」與「自訂」，逐檔語意不成立。
- 不以內容或 hash 比對作為過期判定——使用者自訂即被誤判過期，與保留自訂彈性的目標直接衝突；內容比對只用於更新前的覆蓋警告清單。
- 不新增或變更任何 CLI 子指令、旗標與輸出契約——探測是引擎函式供 desktop 直嵌，更新復用既有 update()。
- 不將提示或略過狀態寫入 .speclink.yaml 或 openspec/config.yaml——個人 UI 決定不進團隊共用設定。
- 不處理 CLI 與 desktop 的版本漂移——既有 sidecar 同版佈署機制已涵蓋。
- 不做強制更新或阻斷開啟——不更新的風險是工作流語意漂移而非引擎故障，提示維持溫和。
- 不以 harness hooks 強制 bump 紀律——per-harness 不可攜，測試對所有貢獻者一視同仁。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-tools`: 新增需求——生成的 skill frontmatter 版本戳與 marker 版號同源（MARKER_VERSION）；引擎提供唯讀過期探測（版本戳比對＋覆蓋差異清單，四態回報含「指令檔不存在＝缺失」）；內嵌資產的 version–hash 鎖定紀律（render 內容變動未 bump 版號即測試失敗）。
- `desktop-app`: 新增需求——開啟本地專案時搭載過期探測與非阻斷提示（過期時「更新」、缺失時「安裝」＋「保留現狀」、per 專案 per 版本的略過記憶）；移除側欄常駐版號、設定頁軟體更新卡為 app 版號唯一住所。

## Impact

- Affected specs: `workspace-tools`（修改）、`desktop-app`（修改）
- Affected code:
  - Modified:
    - crates/speclink-core/src/init.rs
    - crates/speclink-core/src/skills.rs
    - crates/speclink-core/tests/render_golden.rs
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - .claude/skills/（speclink-* 全數技能實例的 frontmatter 版號同批再生）
    - .agents/skills/（同上）
    - apps/desktop/core/src/project.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src/App.tsx
    - apps/desktop/src/store.ts
    - apps/desktop/src/i18n/messages.ts
    - CLAUDE.md
  - New:
    - crates/speclink-core/tests/golden/assets.lock
    - apps/desktop/src/components/InstructionUpdatePrompt.tsx
  - Removed: (none)
- 影響的 crate／app：speclink-core（版本戳、探測、鎖定測試）、apps/desktop/core 與 apps/desktop/src-tauri（探測搭載與 IPC）、apps/desktop 前端（提示 UI、略過持久化、側欄版號刪除）。引擎 CLI 指令面不動。
- 影響的技能與工具：全部內建技能（frontmatter 版號欄位），claude（.claude/skills/）與 codex（.agents/skills/）兩者；CLAUDE.md/AGENTS.md 受管區塊版號語意不變、值隨 bump 前進。
