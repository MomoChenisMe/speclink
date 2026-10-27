---
topic: 桌面視窗預設與設定頁重整
slug: desktop-window-and-settings-polish
status: promoted
promoted_to: desktop-window-and-settings-polish
created: 2026-07-08
---

# Discussion: 桌面視窗預設與設定頁重整

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者提出四項桌面需求，其中三項納入本討論：①視窗預設尺寸加大＋啟動置中（現值 1100×720，無置中）；②側欄「設定」項沉底；③設定頁重新設計（現況四張卡順序混亂：config.yaml 拆成不相鄰兩卡、本機偏好夾在中間）。第四項「卡住的 concluded 討論」已直接處理（封存 專案選擇對齊-spectra），其暴露的引擎設計缺口另立討論。
模式：assumptions——tauri.conf.json、apps/desktop/src/App.tsx、views/SettingsView.tsx 皆已定位，Rust 端確認無視窗覆寫，脈絡充分。
相關 changes/specs：desktop-config-rules-context、desktop-config-card、desktop-nav-reorder（皆已封存，設定頁與側欄現狀的來源）；specs/desktop-config。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: 視窗預設、側欄配置、設定頁資訊架構的初步裁決
**Position**: ①視窗現值 1100×720（tauri.conf.json:13-14）、Rust 端無覆寫——改大尺寸＋`center: true` 是純設定變更，暫定 1440×900；②側欄僅「設定」以 `mt-auto` 沉底（App.tsx:255-282，aside 本為 flex-col），變更／規格／已封存留頂部；③設定頁改為三頁簽分割：config.yaml／.speclink.yaml／本機設定——使用者裁定，理由是未來功能成長需要擴充空間，推翻首輪提案的單頁捲動＋作用域雙群組。
**Ruled out**: 單頁捲動＋「專案／此電腦」群組標題（使用者選頁簽，成長性考量）；tauri-plugin-window-state 記憶視窗狀態（未被要求，YAGNI，僅要預設值）；Spectra 式設定子導航（頁簽已滿足）。
**Open**: 頁簽標籤用人話（專案設定／整合／此電腦，檔名降為面板內 mono 註記）還是檔名直出；「專案設定」簽內是否拆三卡（專案說明、產出規則、產出政策）以避免巢狀頁簽；1440×900 是否合使用者螢幕。

### Round 2 — assumptions (2026-07-08)

**Focus**: 頁簽標籤命名與殘餘開放點清零
**Position**: 使用者裁定頁簽標籤檔名直出（config.yaml／.speclink.yaml／本機設定）——開發者工具中檔案即分割單位，新設定的歸屬以檔案為準最不易誤置。此與 LANGUAGE.md「工程詞不進使用者文案」原則相抵，記為明文例外（檔名得作為設定頁頁簽標籤），屬詞彙飄移、隨變更落地時寫入 LANGUAGE.md。mockup 其餘全案未被反對視為核可：config.yaml 簽內拆三卡（專案說明／產出規則／產出政策，各自編輯儲存、移除巢狀頁簽）、各簽首行 mono 檔名註記、解析失敗橫幅＋頁簽警示點。視窗 1440×900 兩輪未被反對，定案。
**Ruled out**: 人話頁簽標籤（專案設定／整合／此電腦）——使用者選檔名直出。
**Open**: 無——收斂，進 conclude。

## Conclusion

**Decision**: 三項桌面調整定案：①視窗預設 1100×720 → 1440×900 並加 `center: true`（tauri.conf.json 純設定變更，Rust 端無覆寫）；②側欄「設定」項以 `mt-auto` 沉底，變更／規格／已封存留頂部；③設定頁重構為三頁簽、標籤檔名直出——config.yaml（專案說明卡＋產出規則卡＋產出政策卡：原合併卡拆開、移除巢狀頁簽，各卡獨立編輯／儲存）、.speclink.yaml（AI 工具卡；未來 remote 連線落此簽）、本機設定（介面語言；未來本機偏好落此簽）。各簽首行 mono 檔名註記（本機設定簽改為「僅存於此裝置」說明）；檔案解析失敗時該簽橫幅＋頁簽警示點＋表單停用；預設簽 config.yaml。既有讀寫邏輯、各卡獨立儲存與編輯態行為不變。
**Rationale**: 頁簽給未來功能成長擴充空間（使用者核心訴求）；檔案即分割單位，新設定歸屬以檔案為準最不易誤置；拆卡避免頁簽套頁簽。
**Rejected alternatives**: 單頁捲動＋作用域雙群組（成長性不足）；人話頁簽標籤（使用者選檔名直出）；巢狀頁簽（混淆）；tauri-plugin-window-state 記憶視窗狀態（僅需預設值，YAGNI）；Spectra 式設定子導航（頁簽已滿足）。
**Deferred**: 無。
**Capture to**: proposal／tasks（轉出新變更）；LANGUAGE.md 例外條目「檔名得作為設定頁頁簽標籤」（詞彙飄移，隨變更落地寫入）。
**Next**: /speclink-propose --from-discussion desktop-window-and-settings-polish
