## Context

自動封存由變更側驅動：archive 流程封存變更時檢查其 meta 的 from_discussion，若無其他在途變更引用同一討論即自動帶走討論記錄。該鏈目前只有 promote 流程（建新變更 scaffold 時）會寫入；ingest 是純技能、非引擎動詞，結論併入既有變更時兩側皆無連結，討論以 concluded 滯留看板。

既有基建齊備：討論側 mark_promoted（speclink-core 的 discuss 模組——status 翻轉為 promoted、promoted_to 逗號累加、重複值去重）；Store trait 的 read_change_meta／write_change_meta（fs 與測試 store 皆已實作）；CLI 的 discuss 子指令族（promote 形態：位置參數＋--json）。

約束：core／cli 邊界（流程歸 speclink-core、輸出與色彩歸 speclink-cli）；既有人眼與 --json 輸出逐位元不變；內嵌技能資產三處同步（crates/speclink-core/assets、.claude/skills、.agents/skills）與 render golden 乾淨樹再生；TDD 紅綠重構。

## Goals / Non-Goals

**Goals:**

- 引擎動詞 discuss link：對既有變更鑄 from_discussion 鏈、討論側標記 promoted，使 ingest 型結論接上看板群組、抽屜互跳與自動封存。
- 守衛完備：任何拒絕情境下兩側檔案逐位元不變。
- 技能指示配套（discuss conclude 步驟、ingest 提示）與 LANGUAGE.md 詞條修訂。

**Non-Goals:**

- 桌面 GUI／node bridge 曝露 link（無 GUI 需求，另刀）。
- from_discussion 多值累加（欄位維持單值，衝突即拒絕）。
- 自動封存規則改動（archive 側機制照舊）。
- conclude 結論文字解析自動連結（討論階段已否決）。

## Decisions

**D1：獨立 link 流程函式，與 promote 共用 mark_promoted。**
流程邏輯落在 speclink-core 的 discuss 模組，新增獨立的 link 函式；討論側標記直接呼叫既有 mark_promoted，確保 promoted_to 累加與去重行為和 promote 逐位元同款。
替代：promote 加 --into 旗標——旗標把「scaffold 新變更」改成「不 scaffold」，同一動詞兩種本質，語意混淆，否決；複製一份標記邏輯——兩處累加規則遲早分岔，否決。

**D2：變更側寫入沿既有 meta 讀-改-寫模式。**
read_change_meta → 驗證 → 追加 from_discussion 行 → write_change_meta，含 trailing newline 容錯（inprogress 模組同款模式）。全程走 Store trait 既有 API，不新增儲存假設——朝 storage 解耦的規格驅動引擎靠攏。
替代：Store 加專用 link API——既有兩個 API 已足夠，多一個 trait 方法徒增各實作負擔，否決。

**D3：守衛全過才寫，變更側先寫、討論側後標。**
順序：驗證討論（存在、未封存）→ 驗證變更（存在、無其他 from_discussion）→ 寫變更側 → 標討論側。兩寫之間若中斷，殘局為「變更側有鏈、討論側未標」——自動封存由變更側驅動，此殘局仍會在變更封存時帶走討論，朝自癒方向傾倒。
替代：討論側先標——殘局為「promoted 但無鏈」，正是本刀要修的病灶重現，否決。

**D4：接受 open 與 concluded 討論，拒絕已封存。**
與 promote 的前置條件一致（mark_promoted 本就翻轉兩種狀態）。技能流程天然在 conclude 之後執行 link，引擎不重複把關。
替代：限定 concluded——promote 未限定，link 單獨收緊會讓兩動詞前置條件不一致，且擋不住真正的誤用（技能才是流程守門人），否決。

**D5：CLI 契約與 promote 對齊。**
`speclink discuss link <slug> <change>`：兩個位置參數、旗標僅 --json、不吃 stdin。人眼輸出一行 ✓ 成功訊息（含 slug 與變更名）；--json 輸出 slug 與 change 兩欄位（單詞欄位名，天然符合 camelCase 慣例）。守衛失敗：非零 exit code、stderr 一句原因。冪等：同一組合重跑視為成功（mark_promoted 已去重；變更側 from_discussion 等於同 slug 時放行不改檔）。
替代：--change 旗標取代第二位置參數——promote 已立位置參數慣例，否決。

**D6：技能指示三處同步＋golden 乾淨樹再生。**
discuss 技能 conclude 步驟：Capture to 指向既有變更時，先執行 link 再導向 /speclink-ingest；ingest 技能加一句來源討論確認。assets 與 .claude/skills、.agents/skills 同步改（claude 與 codex 皆生效），render golden 於乾淨樹 UPDATE_GOLDEN 再生並審視 diff。
替代：只改 repo 技能實例不動 assets——init/update 生成的技能會回退到舊指示，三處不同步是本 repo 既知事故源，否決。

**D7：LANGUAGE.md 修訂「已轉出變更」定義。**
自「至少轉出過一個變更」放寬為「至少連結一個變更（轉出或併入）」，why 註記 link 動詞。
替代：新增「併入變更」獨立詞條——GUI 不曝露 link，無使用者可見文案需要新詞，徒增詞彙表面積，否決。

## Implementation Contract

**行為**：執行 `speclink discuss link <slug> <change>` 成功後——①該變更 meta 含 from_discussion 指向該討論；②討論 frontmatter status 為 promoted、promoted_to 含該變更名（原有值保留、逗號累加）；③日後封存該變更且無其他在途變更引用同討論時，討論自動移入 discussions/archive/（既有 archive 機制，本刀不改其行為，僅使其對 ingest 型結論生效）。

**介面**：子指令 discuss link，位置參數 slug 與 change，旗標僅 --json；成功 exit 0，人眼一行含兩名稱的成功訊息，--json 輸出 slug 與 change。無 stdin。

**失敗模式**（皆為非零 exit、stderr 單句原因、兩側檔案逐位元不變）：討論不存在；討論已封存；變更不存在；變更已有指向其他討論的 from_discussion。同組合重跑為冪等成功、不報錯不改檔。

**驗收判準**：
- speclink-core 單元測試群（TDD 先紅後綠）：成功鑄鏈兩側可見、四種守衛各自拒絕且不落檔、冪等重跑、promoted_to 既有值累加不覆蓋、meta 無尾換行容錯。
- render golden 測試以乾淨樹再生後的基準通過，diff 僅含新指示文字。
- 手動驗證：對測試 repo 走一次 conclude → link → archive 變更，確認討論自動進封存。
- 既有 parity／color 回歸對照不受影響（未動既有指令輸出）。

**範圍邊界**：in——core 的 link 流程函式與測試、CLI 子指令 wiring、discuss／ingest 技能指示三處同步、golden 再生、LANGUAGE.md 一句修訂。out——Non-Goals 四項（GUI/bridge 曝露、多值鏈、封存規則、文字解析）。

## Risks / Trade-offs

- **回歸對照**：新增子指令不動既有輸出；唯 discuss --help 清單會多一行——parity／color 對照套件若涵蓋該 help 畫面，屬刻意更新並記錄。緩解：實作前先跑一次對照確認基線。
- **golden 於 dirty 樹再生**（本 repo 已發生過一次，main 長紅）：緩解——技能 assets 改動全部提交後，於乾淨樹 UPDATE_GOLDEN 再生。
- **跨平台**：meta 檔讀-改-寫沿既有容錯模式，不假設換行風格；無 git 互動、無平台特定路徑行為。
- **殘局**（D3 兩寫之間中斷）：機率極低且朝自癒方向設計，不引入交易機制（避免過度設計）。
