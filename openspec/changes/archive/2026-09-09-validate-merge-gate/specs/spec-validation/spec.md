## ADDED Requirements

### Requirement: change 驗證納入合併守門

change 的驗證（`speclink validate <change>`、無參數、`--all`、`--changes`，fs 與 remote 兩模式，以及 desktop 的結構驗證）SHALL 對每個 delta capability 執行與 archive 相同的合併守門判斷，每筆違規 SHALL 化為一條 error 並使結果 invalid（文字完全相同的違規只列一次）。error 文字 SHALL 為 `specs/<capability>/spec.md: <operation> '<requirement>': <reason> (see: speclink drift <change>)`，`<reason>` 逐字沿用守門的既有字串，路徑一律正斜線。守門 error SHALL 排在所有既有結構 error 之後，既有 error 的文字與順序 SHALL NOT 改變。已由結構檢查報過的項目 SHALL NOT 重複：新開 capability 的 Purpose 不合格只報既有的 Purpose error；同一 delta 內同名需求已報 Duplicate 或 appears in both 時不再報守門的撞名 error，未被結構檢查涵蓋的撞名（含 RENAMED 端點）仍 SHALL 列出。單筆與 bulk archive 的驗證前置 SHALL 只含結構檢查，archive 的拒絕輸出 SHALL 維持既有位元級輸出。守門違規 SHALL 不論 `--strict` 一律為 error。

#### Scenario: MODIFIED 目標不存在時 validate 報 error

- **WHEN** 正典 `specs/auth/spec.md` 只有需求 `R1`，change `demo` 的 delta 在 `## MODIFIED Requirements` 寫 `R9`，執行 `speclink validate demo`
- **THEN** 人眼輸出列 `✗ demo — invalid` 與 error `specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift demo)`，以 `Validation failed.` 非零收尾；`--json` 的 `valid` 為 false、`errors` 含該字串

#### Scenario: 未宣告的 scenario 移除被 validate 抓到

- **WHEN** 正典 `R1` 有 scenario `ok` 與 `fine`，delta 的 MODIFIED `R1` 只寫 `ok` 且沒有 `<!-- REMOVED-SCENARIO: fine -->`
- **THEN** validate 報一條 error，reason 為守門的 `drops canonical scenario(s) fine` 開頭字串；補上宣告後 validate 通過

#### Scenario: Purpose 與重複需求名不重複列

- **WHEN** 新開 capability `token` 的 delta 缺 `## Purpose`，且 `## ADDED Requirements` 寫了兩次 `Fresh`
- **THEN** errors 恰有 2 條：既有的 Purpose error（含 `## Purpose` 範例骨架）與 `Duplicate requirement 'Fresh' in ADDED section`，沒有守門的第三條

#### Scenario: RENAMED 端點撞名只有守門看得到

- **WHEN** 正典有 `R1`，delta 的 `## RENAMED Requirements` 把 `R1` 改成 `R2`，`## ADDED Requirements` 也寫 `R2`
- **THEN** validate 報 1 條守門 error，`<requirement>` 為 `R2`、reason 為守門的撞區段字串

#### Scenario: archive 的拒絕輸出不變

- **WHEN** 同一份 MODIFIED `R9` 的 change 執行 `speclink archive demo`
- **THEN** stderr 為守門的聚合拒絕清單（`change 'demo' cannot be archived — 1 delta operation(s) no longer match the canonical spec:` 起頭），與本需求加入前逐位元相同

##### Example: 守門 error 的組字

| 違規 | error |
| --- | --- |
| MODIFIED `R9` 目標不存在 | `specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift demo)` |
| ADDED `R1` 撞正典 | `specs/auth/spec.md: ADDED 'R1': already exists in the canonical spec — archive would refuse it (see: speclink drift demo)` |
| RENAMED 缺 TO | `specs/auth/spec.md: RENAMED 'R1': RENAMED operation names no TO: target (see: speclink drift demo)` |
