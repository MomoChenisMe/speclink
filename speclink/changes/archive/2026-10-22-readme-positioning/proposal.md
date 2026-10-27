## Why

README 的標語「一套 SDD Engine，支援 Local Repo 與 Remote Store」與第一段「PM、PO、RD 與 AI Agent 用同一套語意協作」把 speclink 說成從個人到團隊的平台，但實際使用者是透過 AI agent 開發的開發者，核心價值是「每個變更都對得上規格、封存前有證據」。特色段的「純文字，相容 OpenSpec」也與已定案的單向相容不符：speclink 不再保證 OpenSpec CLI 讀得回去，而「從 OpenSpec 匯入」目前還沒有乾淨的路徑——在 OpenSpec 專案跑 `speclink init` 會停在 Already initialized，加 `--force` 會把 config.yaml 蓋回範本。本變更是討論 positioning-and-process-cost 分刀計畫的第 1 刀。

目標使用者與情境：第一次打開 README 或入門教學、要判斷 speclink 是什麼的開發者（主要透過 AI agent 開發），以及團隊情境下的 PO 與 PM；另一個讀者是 propose 階段讀取 openspec/config.yaml 專案說明與提案規則的 AI agent。

## What Changes

- README 中英兩版的標語改為「讓 AI 寫的每個變更，都對得上規格、查得到證據」／"Every AI-built change, checked against its spec"。
- README 中英兩版第一段改以可驗收為主軸：Speclink 是用 Rust 寫的 SDD 引擎；每個變更先寫一小份規格、交給 AI agent 實作，封存前由審查與驗證兩道品質關卡檢查並留下證據；規格是 repo 裡的純文字。
- README 中英兩版第二段（兩條部署路徑）改為：一個人用 Local Repo 就夠；團隊要共用規格再接 Remote Store，PM、PO 在這一層參與討論與提案。原本第一段的「PM、PO、RD 與 AI Agent 用同一套語意協作」移到這一段，不刪除。
- README 中英兩版特色段的「純文字，相容 OpenSpec」改為只講純文字：內容是 Markdown 與 YAML、不裝 Speclink 也讀得懂、每次變動看得到 Git diff、Speclink 多放的兩樣東西（`discussions/` 與各變更的 `.openspec.yaml`）、只適用 Local 模式；拿掉「相容 OpenSpec」與「沿用 OpenSpec 的目錄結構」，也不寫「可從 OpenSpec 匯入」。致謝段對 OpenSpec 的致謝保留。
- 入門教學中英兩版的本地產物說明拿掉「沿用 OpenSpec 的目錄結構」，保留目錄樹與純文字說明。
- 桌面 app 關於頁品牌卡的產品描述目前直接使用舊標語，改為新標語（中英兩版），讓 app 與 README 說法一致。
- openspec/config.yaml 的專案說明改以「可驗收的 SDD 引擎」描述專案；提案規則的目標使用者改為「主要是透過 AI agent 開發的開發者；團隊情境下包含 PO 與 PM」。

## Non-Goals

- 不寫「可從 OpenSpec 匯入」：匯入路徑由後續的 spec-dir-migration 變更提供，說明也由那個變更寫。
- 不改規格資料夾名稱與文件裡的 `openspec/` 路徑：屬於後續的 spec-dir-migration 與 spec-dir-path-rewrite 變更。
- 不從目標使用者刪掉 PM、PO：團隊與 Remote Store 情境仍需要他們，只是不放在第一段。
- 不調整 README 其他 H2 的結構與順序，不改 product-status、roadmap、workflow 等其他文件。
- 不改英文文件裡既有的品質關卡英文用詞；正式規格裡「品質站」的用詞漂移由 quality-gate-wording-in-specs 變更處理。
- 不改桌面關於頁的版面與其他文案；desktop-app 規格只要求「一句產品描述」、沒有寫死字面，不需要 delta。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `user-documentation`: 「使用者文件採漸進揭露與單一責任」的 README 標語字面與第一個 H2 之前的定位內容改寫（第一段可驗收定位、第二段雙路徑與 PM／PO）；「使用者文件載明本地產物的 OpenSpec 結構相容性」改名並改寫為純文字可攜性，不再宣稱 OpenSpec 相容，也不宣稱可匯入。

## Impact

- Affected specs: `user-documentation`
- Affected code:
  - Modified: README.zh-TW.md, README.md, docs/getting-started.zh-TW.md, docs/getting-started.md, openspec/config.yaml, apps/desktop/src/i18n/messages.ts, apps/desktop/src/__tests__/aboutPage.test.tsx, apps/desktop/src/__tests__/App.test.tsx
- 影響的 crate／app：只有 apps/desktop 的 i18n 文案與兩支前端測試；沒有 Rust crate 變動。
- 設定欄位：openspec/config.yaml 只改 `context` 與 `rules.proposal` 的內容文字，不新增欄位，預設值不變。
- 生成的技能：沒有技能檔變動，不升 ASSET_VERSION，golden 不變。
- 相容性影響：CLI 人眼輸出與 `--json` 都不變；桌面關於頁的產品描述文字改變。
