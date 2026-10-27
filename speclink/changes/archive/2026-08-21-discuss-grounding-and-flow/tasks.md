## 1. 改寫 discuss 技能資產（crates/speclink-core/assets/skills/discuss.md）

- [x] 1.1 落實「事實與決策分診及逐節點查證」的漏斗偵察：先 speclink list --specs --json（候選 ≤5、讀 Purpose ≤3、直接目標 capability 才讀全文、零命中靜默略過），以命中 capability 名與正典詞彙轉譯搜尋詞後再掃原始碼（至多 5 檔）；主題含具體檔名或符號時程式碼軌直接開跑。刪除「not docs, not tests」的規格排除字句。完成判準：段落含上述全部上界與順序，且 Context 模板「related changes/specs」有對應產出步驟 <!-- speclink-task:tsk_01M0F0NYPTEFK9Z892WYWMAA3B -->
- [x] 1.2 落實「正典接地與三分對照」：假設清單規則加入三分對照（正典已涵蓋／與正典衝突／正典沒講，各附證據或裁定去向）與紀律句「使用者需求是目標，正典是證據、不是裁決；偏離正典記入記錄」。完成判準：assumptions 段落含三分對照定義與紀律句，衝突呈現為假設而非擋下 <!-- speclink-task:tsk_01M0F0NYPTQGMRP07AGEVMW890 -->
- [x] 1.3 落實「interview 模式以決策樹遍歷提問」的重定位：刪除「3+ 檔走 assumptions／不足走 interview」門檻；分流軸改為需求清晰度——需求鈍先走 grill 階段（一次一題磨目標、範圍、門檻、成功判準；需求利時塌縮為零題），assumptions 為唯一預設姿態；決策樹遍歷與使用者主導停止條件字句保留不動；現有 Push for specifics 範例歸入 grill 題型。完成判準：檔案數門檻字句不存在，grill 觸發與塌縮規則明文，one nudge maximum 與 Deferred 段落原樣 <!-- speclink-task:tsk_01M0F0NYPTGYF7H8CJR0TMY9N0 -->
- [x] 1.4 落實「interview 每題附建議答案與 Evidence」的二分：grill 意圖題附框題脈絡（現況或正典證據）或最佳猜測建議；節點退路題附建議答案＋Evidence；兩類皆禁空白提問。完成判準：兩類問題各有明文義務 <!-- speclink-task:tsk_01M0F0NYPTHSZCT35THBCD7EN9 -->
- [x] 1.5 落實「多需求 backlog 與恢復摘要」：記錄段新增 backlog 慣例（多需求首輪 Open 攤全清單、每輪 Open 復述剩餘項、定案去向由 Position 首句承載）與恢復摘要儀式（續用 open 討論先輸出逐輪 Focus→Position 首句＋最後 Open，再繼續）。完成判準：兩慣例明文且註明摘要自既有欄位推導、零格式變更零遷移 <!-- speclink-task:tsk_01M0F0NYPTK02JXGCB1GT6H5BS -->
- [x] 1.6 落實「中途轉出教學」：單項談定即 promote（引擎無結論以 topic 預填）、討論繼續加輪、最終 conclude 保留 promoted 並標已轉出變更待重新反映（與結論無關時一次確認）。完成判準：教學段含 promote 時機、繼續加輪、補結論三步與標記說明 <!-- speclink-task:tsk_01M0F0NYPTY1YTNWH0ZE9N4C9D -->

## 2. 版號與衍生物（TDD：golden 為測試網）

- [x] 2.1 紅：跑 cargo test -p speclink-core --test it render_golden:: 確認因資產改寫而失敗（golden 鎖住舊文，證明測試網有效） <!-- speclink-task:tsk_01M0F0NYPTMRHXB8XN1VFSQKSD -->
- [x] 2.2 bump MARKER_VERSION（crates/speclink-core/src/init.rs），確認渲染 frontmatter 的 version 隨之更新 <!-- speclink-task:tsk_01M0F0NYPT6PSVQGYPN6S2VZSN -->
- [x] 2.3 綠：逐份審閱渲染輸出無誤後，以 UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden:: 再生 golden 快照與 crates/speclink-core/tests/golden/assets.lock <!-- speclink-task:tsk_01M0F0NYPTTFQV9A93HZCH8FF3 -->
- [x] 2.4 cargo test -p speclink-core 全綠（含 render_golden 與其餘單元測試） <!-- speclink-task:tsk_01M0F0NYPT9C6Z2DNAQ40FA1D8 -->

## 3. 落地與盤點

- [x] 3.1 跑 speclink update 再生工具技能目錄的全部 SKILL.md（版號 bump 波及約 32 份），以 git status 盤點衍生物全數帶進收尾 commit（衍生物不進 evidence） <!-- speclink-task:tsk_01M0F0NYPTKFJJCS6XZ40MB1G0 -->
- [x] [M] 3.2 以一場真實多需求討論驗收新流程：恢復摘要、grill 階段、三分對照至少各出現一次，且記錄格式與既有討論一致 <!-- speclink-task:tsk_01M0F0NYPTENFS1QKCKS6C4ANE -->

## 4. 建檔時機修正（實測回饋 ingest）

- [x] 4.1 落實「記錄建檔以使用者首次回覆為觸發」——把「first substantive round」的觸發點釘死在使用者回覆：Create the record 段明文「觸發＝使用者的回覆（對假設清單的確認或修正、對提問的回答）使主題前進；代理人自身的研究產出或首份假設清單不算 substance，使用者回覆前不得執行 speclink discuss new」。完成判準：該段含觸發定義與明文排除句，且與「At the start」第 3 點及 Guardrails 的 Do record 句一致 <!-- speclink-task:tsk_01M0H65BH556EAVE9S9QTCEGCY -->
- [x] 4.2 紅→綠：cargo test -p speclink-core --test it render_golden:: 先紅確認 golden 鎖住舊文；還原 committed 的 assets.lock 後以 UPDATE_ASSETS_LOCK=1 與 UPDATE_GOLDEN=1 重生（v1.19.17 尚未提交，不再 bump 版號）；cargo test -p speclink-core 全綠 <!-- speclink-task:tsk_01M0H6ATNEPP2ZQXJJS7KR69BJ -->
- [x] 4.3 speclink update 再生工具技能目錄 SKILL.md 與 marker 檔，git status 盤點衍生物全數帶進收尾 commit（衍生物不進 evidence） <!-- speclink-task:tsk_01M0H6FK7Q7ZNVRTQ1KT8KR6XV -->
- [x] 4.4 node scripts/desktop-install.mjs --install 重裝本機 desktop，建置與安裝後兩道引擎版號斷言皆為 v1.19.17，且安裝版 CLI 於本 repo 跑 speclink update 後 git status 零差異 <!-- speclink-task:tsk_01M0H71KNT749TCW2VHFN89YRT -->
