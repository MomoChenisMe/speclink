## 1. LANGUAGE.md 正典詞彙落點

- [ ] 1.1 於 `openspec/LANGUAGE.md` 的「詞彙」段新增「詳情面板」與「品質關卡」兩個詞條，各含 definition／avoid／why 且 why 載明裁定日期 2026-08-14——落實 D2：兩個詞立為 LANGUAGE.md 正典詞條，舊詞入 avoid，同時滿足規格需求「兩個收斂詞以 LANGUAGE.md 詞條承載，且正典本身排除於守門之外」。驗證：讀取該段確認兩個詞條齊備、avoid 欄位分別逐字含「抽屜」與「品質站」。 <!-- speclink-task:tsk_01KZYZ21NPT303QVG03BVK617B -->
- [ ] 1.2 依 D6：替換規則避免「詳情詳情面板」疊字（`詳情抽屜`→`詳情面板`、`討論抽屜`→`討論詳情面板`、`已封存抽屜`→`已封存詳情面板`、裸用`抽屜`→`詳情面板`；「品質站」→「品質關卡」直換），換掉 `openspec/LANGUAGE.md` 既有詞條定義文與 why 文中的舊詞。驗證：全檔搜尋確認無疊字，且除兩條明文例外外無殘留舊詞。 <!-- speclink-task:tsk_01KZYZ21NPSVS1XKG7PEV525A5 -->
- [ ] 1.3 依 D5：LANGUAGE.md 兩條明文例外只換用詞，裁定內容與紀錄一字不動——處理討論 slug 例外與 worktree 例外兩行，只替換其中的「抽屜」字面，裁定語句、適用範圍其餘項目與四筆範圍擴充紀錄（desktop-card-identity 2026-07-09、desktop-ux-polish 2026-07-11、tray-copy-and-panel-mode 2026-07-16、change-drawer-header-redesign 2026-08-04）全部逐字保留。驗證：`git diff openspec/LANGUAGE.md` 確認這兩行只有詞彙差異。 <!-- speclink-task:tsk_01KZYZ21NPVEV6T7CPABYTFCX8 -->

## 2. 詞彙守門測試（先紅）

- [ ] 2.1 新增 `scripts/vocabulary-guard.test.mjs` 實作規格需求「詞彙守門測試釘死使用者可見文案面」，落實 D4：以守門測試釘死使用者可見文案面——掃描兩個 i18n 訊息檔、`crates/speclink-core/assets/skills/` 全部 `.md`、`docs/` 全部 `.md`、`README.md` 與 `README.en.md`，命中 avoid 詞時以非零 exit code 失敗且訊息含檔案路徑、行號與命中詞。此時因文案尚未收斂，測試 SHALL 失敗（TDD 紅燈）。驗證：`node --test scripts/vocabulary-guard.test.mjs` 失敗且訊息點名 `apps/desktop/src/i18n/messages.ts` 與 `apps/server-web/src/i18n/messages.ts`。 <!-- speclink-task:tsk_01KZYZ21NP66HMZ0NHCXR8F4TM -->
- [ ] 2.2 於同一測試檔補齊規格需求「使用者可見文案面的範圍與詞彙約束」的邊界案例：植入 avoid 詞會失敗的反向案例、ASCII 識別符（`RichDetailDrawer`、`archivedDrawerBase`）不觸發誤判、英文文案不受約束、CRLF 與 LF 行尾判定一致，以及 `openspec/LANGUAGE.md` 與 `openspec/specs/` 不在掃描範圍內。驗證：`node --test scripts/vocabulary-guard.test.mjs`，上述邊界案例全數如預期。 <!-- speclink-task:tsk_01KZYZ21NPB0CXZMCJPPV7WE3D -->

## 3. 使用者可見文案收斂（轉綠）

- [ ] 3.1 將 `apps/desktop/src/i18n/messages.ts` 的 `zh-TW` 區塊中 `store.reviewActionUnsupported` 文案由「此工作區不支援品質站工單處置」改為「此工作區不支援品質關卡工單處置」；訊息鍵名與 `en` 區塊字面維持不變。驗證：`npm test -w apps/desktop` 通過。 <!-- speclink-task:tsk_01KZYZ21NPDSXBV6DETNXRNYEK -->
- [ ] 3.2 將 `apps/server-web/src/i18n/messages.ts` 的 `zh-TW` 區塊中 `tour.navUsers.hint` 與 `tour.listPrimary.hint` 兩則導覽提示的「抽屜」改為「詳情面板」；訊息鍵名與 `en` 區塊字面維持不變。驗證：`npm test -w apps/server-web` 通過。 <!-- speclink-task:tsk_01KZYZ21NPVYQKA9J55SP2FJ3H -->
- [ ] 3.3 將 `crates/speclink-core/assets/skills/apply-worktree-post.md` 交棒段對使用者說的「品質站」改為「品質關卡」，使再生後的技能內文符合 delta 規格需求「apply-with-worktree 技能的收尾指示」。驗證：`cargo test -p speclink-core --test it` 通過（golden 於任務 4.1 同批重生）。 <!-- speclink-task:tsk_01KZYZ21NPC0AN3HK82M18GGMN -->
- [ ] 3.4 將 `crates/speclink-core/assets/skills/worktree-merge.md` 交棒段對使用者說的「品質站」改為「品質關卡」，使再生後的技能內文符合 delta 規格需求「worktree-merge 技能的收尾流程指示」。驗證：`cargo test -p speclink-core --test it` 通過（golden 於任務 4.1 同批重生）。 <!-- speclink-task:tsk_01KZYZ21NP64JCV7ZWYGKDM1EJ -->

## 4. 產物層三連動

- [ ] 4.1 依 D7：技能資產改動走既有三連動，不繞過鎖——將 `crates/speclink-core/src/init.rs` 的 `MARKER_VERSION` 自 `v1.19.12` 進版，以 `UPDATE_GOLDEN=1` 重生 golden 快照，再於乾淨樹以 `UPDATE_ASSETS_LOCK=1` 重生 `assets.lock`；不得手改 golden 或 lock 繞過鎖。驗證：`cargo test -p speclink-core --test it render_golden::` 通過，且 `assets.lock` 的 version 為新版號。 <!-- speclink-task:tsk_01KZYZ21NPBMD7TN3618ZWE0AS -->
- [ ] 4.2 執行 `./target/debug/speclink update` 再生 `.claude/skills/` 與 `.agents/skills/` 的受管技能檔，確認兩支 worktree 技能的 SKILL.md 交棒段已為「品質關卡」。驗證：搜尋兩個 skills 目錄確認「品質站」歸零，並以 `git status` 盤點再生產物無遺漏。 <!-- speclink-task:tsk_01KZYZ21NP469TBB7Z9Y79H030 -->

## 5. 邊界確認與收尾

- [ ] 5.1 確認 D1：改動邊界收斂到使用者可見文案面，規格散文與程式碼註解不回改，以及 D3：已封存變更與歷史 artifacts 一律不回改——兩者皆未被越界執行：`openspec/specs/` 除本變更的兩份 delta 外無舊詞改動、程式碼註解與測試名未被回改、`openspec/changes/archive/` 零改動，且無任何識別符更名。驗證：`git diff --stat` 逐檔比對變動範圍與 design 的 Scope boundaries 一致。 <!-- speclink-task:tsk_01KZYZ21NPJFE6W2TQ5PJW7H4G -->
- [ ] [M] 5.2 由使用者確認 `openspec/LANGUAGE.md` 兩條明文例外的裁定內容未被重述——只換用詞、裁定語句與四筆範圍擴充紀錄逐字保留。驗證：使用者檢視 `git diff openspec/LANGUAGE.md` 該兩行後確認接受。 <!-- speclink-task:tsk_01KZYZ21NPG534V37DGXMPNYCP -->
- [ ] 5.3 收尾跑一次跨面測試確認整批綠燈：`node --test "scripts/**/*.test.mjs"`（守門測試由紅轉綠）、`cargo test -p speclink-core --test it`、`npm test -w apps/desktop`、`npm test -w apps/server-web`。驗證：四組指令全數通過。 <!-- speclink-task:tsk_01KZYZ21NPEZXP2S3GKVE3PRP9 -->
