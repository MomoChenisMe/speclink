## ADDED Requirements

### Requirement: 驗證 finding 路徑落在工單範圍內

`speclink verify add-round <change> --stdin` SHALL 在寫入之前檢查本輪每一條 finding 的路徑，規則與審查工單相同。這項檢查排在既有的 change 存在檢查、寫碼任務守門與輪次序列檢查之前：同一份輸入同時違反多項時，先回報路徑違規；格式錯誤照舊回報同一則錯誤。規則如下：

- 路徑在最後一個冒號之後只剩行號寫法（例如 `src/a.rs:42`、`src/a.rs:42-50`、`src/a.rs:42-`、`src/a.rs:L42`），或以單獨的冒號結尾時 SHALL 拒絕。
- 路徑（`\` 正規化為 `/` 後）不在「工單既有各輪 `**Scope**:` 的聯集，加上本輪 `**Scope**:`」之內時 SHALL 拒絕。路徑位於規格目錄（預設 `openspec/`）或 `.speclink/` 之下時 SHALL 免除這一項——驗證站常把 Requirement 層級的 finding 錨定在 delta 規格檔上，這類路徑不進驗證 patch，複驗的範圍解析會略過它們。
- 與工單上一輪某條 finding 的 severity、路徑與描述完全相同的行（描述結尾的 `(accepted)` 標記不計），視為沿用的未解項，SHALL NOT 受上述兩項檢查。

違規時 SHALL 以非零 exit code 結束、stderr 逐一列出違規的路徑與原因，`verify.md` SHALL 零寫入、位元級不變。合法內容的成功行為與 stdout SHALL 維持既有輸出不變。remote 模式下 CLI SHALL 以同一套規則先讀取工單再檢查，違規時於送出前非零結束、stderr 帶違規路徑，server 端工單零寫入；離線、認證失效與 revision 衝突沿用既有 remote 動詞行為。

#### Scenario: 帶行號的路徑被拒

- **WHEN** 對任務全數完成、無工單的 change 寫入 discovery 輪，Scope 為 `src/a.rs`，finding 路徑寫成 `src/a.rs:7-9`
- **THEN** exit code 非零，stderr 點名 `src/a.rs:7-9` 帶有行號，`verify.md` 不建立

#### Scenario: 範圍外的程式碼路徑被拒

- **WHEN** 工單各輪與本輪 Scope 都不含 `src/caller.rs`，新 finding 的路徑為該檔
- **THEN** exit code 非零，stderr 點名該路徑不在工單範圍內，`verify.md` 位元級不變

#### Scenario: 錨定在 delta 規格檔的 finding 可接受

- **WHEN** 本輪 Scope 只含程式碼檔，新 finding 寫成 `- [CRITICAL] openspec/changes/demo/specs/auth/spec.md — Completeness: Requirement not found: 登入逾時`
- **THEN** exit code 0，該 finding 照原文寫入

#### Scenario: 早輪 Scope 內的路徑可接受

- **WHEN** Round 1 的 Scope 含 `src/b.rs`，Round 2 寫入一條路徑為 `src/b.rs` 的新 finding
- **THEN** exit code 0，`verify.md` 新增 `## Round 2`

#### Scenario: 沿用的已接受項不受檢查

- **WHEN** Round 1 有一條帶 `(accepted)` 的 finding，其路徑在檢查上線前已寫成帶行號，Round 2 原文沿用
- **THEN** exit code 0，該行照原文寫入 Round 2
