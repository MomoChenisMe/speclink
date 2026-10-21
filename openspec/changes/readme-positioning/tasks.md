## 1. 桌面關於頁的產品描述（先紅後綠）

- [ ] 1.1 先讓測試紅：apps/desktop/src/__tests__/aboutPage.test.tsx 與 apps/desktop/src/__tests__/App.test.tsx 裡，關於頁品牌卡產品描述的斷言改為新標語「讓 AI 寫的每個變更，都對得上規格、查得到證據」。驗證：`npm test -w apps/desktop -- aboutPage App` 這兩個斷言失敗，失敗訊息顯示畫面仍是舊標語 <!-- speclink-task:tsk_01M5EPDVFR8RZZH651WTE20CEF -->
- [ ] 1.2 讓測試綠：apps/desktop/src/i18n/messages.ts 的 `about.tagline` 繁中改為「讓 AI 寫的每個變更，都對得上規格、查得到證據」、英文改為「Every AI-built change, checked against its spec」，關於頁品牌卡的產品描述隨 UI 語言顯示新標語。驗證：同一指令兩支測試通過；`grep -rn -e "一套 SDD Engine" -e "One SDD Engine" apps packages README.md README.zh-TW.md docs` 沒有命中 <!-- speclink-task:tsk_01M5EPDVFRYMSV2SSZGJ7B5PMB -->

## 2. README 定位

- [ ] 2.1 README.zh-TW.md 與 README.md 第一個 H2 之前改為可驗收定位（需求「使用者文件採漸進揭露與單一責任」）：標語改為新字樣；定位段說明 Speclink 是以 Rust 實作的 SDD 引擎，每個變更先寫一小份規格、交給 AI agent 實作，封存前由審查與驗證兩道品質關卡檢查並留下證據，規格是 repo 裡的純文字，且不出現 PM、PO；雙路徑段改為「一個人用 Local Repo 就夠；團隊要共用規格再接 Remote Store」，並承接 PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意與 PM、PO 在團隊這一層參與討論與提案的說明。驗證：內容檢查兩版第一個 H2 之前的段落一一對應、第一段沒有 PM 與 PO；`grep -n "^## " README.md README.zh-TW.md` 的 H2 清單與改動前相同 <!-- speclink-task:tsk_01M5EPDVFR989KAA2DTFHASP1E -->
- [ ] 2.2 README.zh-TW.md 與 README.md 特色段的「純文字，相容 OpenSpec」改為純文字可攜性（需求「使用者文件載明本地產物的純文字可攜性」，取代被移除的「使用者文件載明本地產物的 OpenSpec 結構相容性」）：保留目錄位置、Markdown 與 YAML、不裝 Speclink 也能讀寫、Git diff 可見、`discussions/` 與 `.openspec.yaml` 兩項擴充、只適用 Local 模式；拿掉「相容 OpenSpec」與「沿用 OpenSpec 的目錄結構」，不寫匯入；致謝段不動。驗證：`grep -n "OpenSpec" README.md README.zh-TW.md` 只剩致謝段的命中 <!-- speclink-task:tsk_01M5EPDVFRPRTS2AA6CVS8R8GV -->

## 3. 入門教學

- [ ] 3.1 docs/getting-started.zh-TW.md 與 docs/getting-started.md 的本地產物說明拿掉「沿用 OpenSpec 的目錄結構」（英文版的 OpenSpec folder layout），保留目錄樹、純文字、Git diff 與兩項擴充的說明（需求「使用者文件載明本地產物的純文字可攜性」）。驗證：`grep -n "OpenSpec" docs/getting-started.md docs/getting-started.zh-TW.md` 沒有命中，且兩版該段落一一對應 <!-- speclink-task:tsk_01M5EPDVFRXFVT8YQYYAVWDNNC -->

## 4. 工作流設定

- [ ] 4.1 openspec/config.yaml 的 `context` 專案段改以「可驗收的 SDD 引擎」描述 speclink（保留 Local Repo／Remote Store 兩條路徑與技術棧段落）；`rules.proposal` 第一條的目標使用者改為「主要是透過 AI agent 開發的開發者；團隊情境下包含 PO 與 PM」；其他欄位逐位元組不變。驗證：`speclink instructions proposal --change readme-positioning --json` 的 `context` 與 `rules` 顯示新字樣；`git diff openspec/config.yaml` 只動這兩處 <!-- speclink-task:tsk_01M5EPDVFR0W9KA42YW9PBZ3G6 -->

## 5. 收尾

- [ ] 5.1 文件守門與變更檢查全部通過：詞彙守門、中英對照與連結檢查掃整個 README 與 docs 面。驗證：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過，且 `speclink validate readme-positioning` 通過 <!-- speclink-task:tsk_01M5EPDVFRWR65W9CNYA4RQT1X -->
