> design 對應：第 1 組＝D1 合格判準單一函式、門檻常數單一定義＋D5 佔位偵測前綴移至 core 單一定義；第 2 組＝D2 change 早期檢查掛在 validate_change、僅對「新開 capability」觸發；第 3 組＝D3 archive 硬擋取代佔位寫入、加入既有 fail-closed 守門清單；第 4 組＝D4 validate --specs 的接線與兩模式同形；第 5 組＝D6 asset 內文變更的三連動。

## 1. 合格判準與常數（core）

- [ ] 1.1 實作需求「Purpose 合格判準單一定義」：crates/speclink-core 新增公開的 Purpose 合格判準函式（存在 ## Purpose 區段、內容非空、trim 後 chars().count() >= 50）與常數 MIN_PURPOSE_LENGTH = 50。先寫單元測試：合格、缺段、空內容、49 字元、50 個中文字元五個 case <!-- speclink-task:tsk_01KZNE6PBCD68DR8GFMN4CS9AH -->
- [ ] 1.2 佔位前綴「TBD - created by archiving」上移為 speclink-core 公開常數；archive.rs 佔位產生處與 apps/desktop/core/src/query.rs 的 PURPOSE_TBD_PREFIX 改引用同一常數，desktop 既有一致性測試（list_specs_purpose_tbd_flags_archive_placeholder）改釘 core 常數後仍綠 <!-- speclink-task:tsk_01KZNE6PBCEX5HQ500F4JFK616 -->

## 2. change 驗證早期檢查（core）

- [ ] 2.1 實作需求「新開 capability 的 change 驗證早期檢查」：crates/speclink-core/src/validate.rs 的 validate_change 對每個 delta capability 查正典存在性，新開且 Purpose 不合格 → error；既有 capability 零報。先寫測試：新開缺 Purpose 報 error、新開合格通過、既有 capability 無 Purpose 零報三個 case <!-- speclink-task:tsk_01KZNE6PBCRWGRN6VQGZKGWWBR -->
- [ ] 2.2 error 訊息帶修復指引與範例骨架（含「## Purpose」與一兩句、50 字元以上的說明，照上游 GUIDE_MISSING_SPEC_SECTIONS 的附例形式）；測試斷言訊息含範例骨架關鍵行 <!-- speclink-task:tsk_01KZNE6PBCSSR2VNVKP8619TP3 -->

## 3. archive 硬擋（core）

- [ ] 3.1 實作 archive-merge 需求「新 capability 的 Purpose 自 delta 帶入」的守門面：crates/speclink-core/src/archive.rs 的 merge_violations 對「正典尚無此 capability 且 delta Purpose 不合格」回 violation（沿用 D1 判準），拒絕時零檔案效果。先寫測試：缺 Purpose 拒絕、過短拒絕、合格帶入、既有 capability 帶 Purpose 忽略不拒、skip_specs 不觸發五個 case <!-- speclink-task:tsk_01KZNE6PBCWATZZ0A3VZMFP06R -->
- [ ] 3.2 確認佔位寫入分支保留（守門後理論不可達）且既有 archive 測試全綠：cargo test -p speclink-core archive <!-- speclink-task:tsk_01KZNE6PBCEQKFM93CDSYZBR19 -->

## 4. validate --specs 接線（core＋CLI）

- [ ] 4.1 實作需求「validate --specs 驗證正典規格」的 core 面：新增正典規格驗證函式（缺段／空＝error；不足 50 字元＝strict 下 warning；佔位前綴開頭＝warning），Command::Validate 增 specs 分支。先寫測試：四種分級各一 case、--specs 單獨僅驗規格、--all 聯集、預設不變 <!-- speclink-task:tsk_01KZNE6PBCQ6ZY0AETTZKAW7JT -->
- [ ] 4.2 crates/speclink-cli/src/verbs/checks.rs 把 a.specs 接進 Command::Validate（消滅空轉旗標），沿用 render_validate_results 渲染；CLI 整合測試：validate --specs 對含佔位規格的專案輸出 warning、非 strict 不報過短、任一 error 非零收尾 <!-- speclink-task:tsk_01KZNE6PBCS1ZRJDMJDQKF5YW1 -->
- [ ] 4.3 remote 模式的 --specs：client 以既有正典規格讀取動詞取內容、本地跑同一驗證器，輸出與 fs 模式同形；補兩模式同形測試（比照既有 Dual 動詞測試風格） <!-- speclink-task:tsk_01KZNE6PBCQTMCZMH66F3CNCZM -->

## 5. 規格指引 asset（規則單一正典）

- [ ] 5.1 crates/speclink-core/assets/schema/spec-driven/specs.instruction.md 於 Format requirements 與 MODIFIED workflow 之間補 Purpose 規則段：新 capability 的 delta 以 ## Purpose 開頭（一兩句、50 字元以上，validate 會擋）；既有 capability 的 delta 不要加（會被忽略）；要改既有 Purpose（含殘留佔位）直接編輯正典檔；附範例（對齊上游 schema.yaml 同段落、融入現有行文） <!-- speclink-task:tsk_01KZNE6PBCK299K06GZFY0XAWP -->
- [ ] 5.2 asset 三連動：MARKER_VERSION 遞增、golden 快照再生（crates/speclink-core/tests/golden/*.snapshot.md 與 marker）、assets.lock 更新；cargo test -p speclink-core render_golden 綠 <!-- speclink-task:tsk_01KZNE6PBCFFWTXDE2DS8MDCC2 -->

## 6. 收尾驗證

- [ ] 6.1 全量：cargo test --workspace 通過（含 CLI 整合測試） <!-- speclink-task:tsk_01KZNE6PBCNXX8K34Z0H3ZTN1S -->
- [ ] 6.2 實測本專案：./target/debug/speclink validate --specs 對 66 份佔位規格輸出 66 筆 warning、archive-merge 一份通過；validate --all 聯集正常 <!-- speclink-task:tsk_01KZNE6PBCW4F18NPN781DGGQ9 -->
