## ADDED Requirements

### Requirement: finding 路徑落在工單範圍內

`speclink review add-round <change> --stdin` SHALL 在寫入之前檢查本輪每一條 finding 的路徑。這項檢查排在既有的 change 存在檢查與輪次序列檢查之前：同一份輸入同時違反多項時，先回報路徑違規；格式錯誤照舊回報同一則錯誤。規則如下：

- 路徑在最後一個冒號之後只剩行號寫法（例如 `src/a.rs:42`、`src/a.rs:42-50`、`src/a.rs:42-`、`src/a.rs:L42`），或以單獨的冒號結尾時 SHALL 拒絕。
- 路徑（`\` 正規化為 `/` 後）不在「工單既有各輪 `**Scope**:` 的聯集，加上本輪 `**Scope**:`」之內時 SHALL 拒絕。此聯集與蓋章指紋範圍採同一定義。路徑位於規格目錄（預設 `openspec/`）或 `.speclink/` 之下時 SHALL 免除這一項——這類路徑本來就不進驗證 patch，複驗的範圍解析會略過它們。
- 與工單上一輪某條 finding 的 severity、路徑與描述完全相同的行（描述結尾的 `(accepted)` 標記不計），視為沿用的未解項，SHALL NOT 受上述兩項檢查。

違規時 SHALL 以非零 exit code 結束、stderr 逐一列出違規的路徑與原因，`review.md` SHALL 零寫入、位元級不變。合法內容的成功行為與 stdout SHALL 維持既有輸出不變。remote 模式下 CLI SHALL 以同一套規則先讀取工單再檢查，違規時於送出前非零結束、stderr 帶違規路徑，server 端工單零寫入；離線、認證失效與 revision 衝突沿用既有 remote 動詞行為。

#### Scenario: 帶行號的路徑被拒

- **WHEN** 對已有 Round 1（Scope 含 `src/a.rs`）的工單追加 validation 輪，新 finding 寫成 `- [WARNING] src/a.rs:42 — Correctness: 邊界未處理`
- **THEN** exit code 非零，stderr 點名 `src/a.rs:42` 帶有行號，`review.md` 位元級不變

#### Scenario: 範圍外的路徑被拒

- **WHEN** 本輪 Scope 為 `src/a.rs`、既有各輪 Scope 也不含 `src/caller.rs`，新 finding 的路徑為 `src/caller.rs`
- **THEN** exit code 非零，stderr 點名 `src/caller.rs` 不在工單範圍內，`review.md` 位元級不變

#### Scenario: 早輪 Scope 內的路徑可接受

- **WHEN** Round 1 的 Scope 含 `src/b.rs`，Round 2 驗證 patch 只改 `src/a.rs`，Round 2 寫入一條路徑為 `src/b.rs` 的新 finding
- **THEN** exit code 0，`review.md` 新增 `## Round 2`

#### Scenario: 規格目錄下的路徑免除範圍檢查

- **WHEN** Scope 只含 `src/a.rs`，新 finding 的路徑為 `openspec/changes/demo/design.md`
- **THEN** exit code 0，該 finding 照原文寫入

#### Scenario: 沿用的未解項不受檢查

- **WHEN** 舊工單 Round 1 已有 `- [WARNING] src/a.rs:12 — Correctness: x`，Round 2 原文沿用該行
- **THEN** exit code 0，該行照原文寫入 Round 2，不因帶行號被拒

#### Scenario: 反斜線路徑正規化後比對

- **WHEN** Scope 含 `src/a.rs`，新 finding 的路徑寫成 `src\a.rs`
- **THEN** 路徑正規化為 `src/a.rs` 後落在範圍內，exit code 0

#### Scenario: remote 模式同樣拒絕

- **WHEN** remote 模式下對工單追加一條範圍外路徑的 finding
- **THEN** CLI 非零結束、stderr 帶該路徑，不送出寫入，server 端工單零寫入
