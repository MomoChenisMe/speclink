## Context

現行審查工單只記錄每輪的 Scope 檔案清單與 findings；技能的續輪因此只能重讀上輪 finding 所在整檔，再加上修正過程碰過的檔案。這個範圍不是 Matt Pocock 式「固定 diff」，也讓生成式 reviewer 每輪重新探索新的 smell 與 bug。

touched v1／v2 是 change 級檔案歸屬與 task evidence，並被 commit、archive、drift 等流程消費；它既沒有修改前內容，也不會為同檔後續修改建立新歸屬。最新討論已裁定不全面改造 touched，而以 host-local baseline、Git resolver 與 review-time snapshots 提供審查所需精度。

Local Repo 與 Remote Store 都由執行 agent 持有 code checkout；TeamStore 只持有 specs、change artifacts、工單與 metadata。無 Git、跨 Host 或 workspace 已前進後仍要重播原始 Apply patch，屬未來獨立 provenance 能力，不在本變更內。

範圍拆分已明示進行：verify 行為留在既有 verify-station-parity，完整跨 Host provenance 留給未來 change。本變更內的 Host resolver、ticket binding 與 review skill 是同一條可觀察流程；任一段單獨落地都無法產生可用的 frozen review，因此維持單一 change，並以 TDD 任務分段控制風險。

## Goals / Non-Goals

**Goals:**

- Round 1 對一份凍結、可雜湊的 change patch 執行唯一一次 Standards／Correctness discovery
- Round 2+ 只驗收既有 findings 與 remediation patch 的直接回歸，並以嚴格進展規則保證自動迴圈終止
- 保留 touched 檔案層級契約，以 Host 單一 resolver 同時服務 local 與 remote CLI
- 對 dirty-at-start、跨 active change 重疊、缺 baseline 與 snapshot 漂移一律 fail closed
- 讓既有 review 工單仍可 show、stamp、discard，並將新 frozen patch identity 加入後續輪次

**Non-Goals:**

- 不提供逐 edit 攔截、跨 Host 重播或無 Git provenance
- 不改 touched v1／v2、commit、archive、drift、review badge 或 desktop UI
- 不把 verify 三維度移入 review；verify-station-parity 只消費本變更建立的共用 scope／round 契約
- 不新增固定最大輪數，也不以 finding 數量代表程式品質
- 不自動判定 dirty 同檔中的意圖歸屬

## Decisions

### D1 — Apply baseline 是 host-local sidecar，不是 touched v3

新增 .speclink/review-scopes/<change>/baseline.json，serde 欄位採 camelCase：

- version：目前固定 1
- change：change 名
- baseCommit：Git HEAD 完整 SHA；無 Git 時為 null
- dirtyFilesAtStart：以 repo-root 相對、正斜線、排序去重的路徑陣列
- capturedAt：UTC RFC3339
- confidence：initial、late 或 unavailable

speclink review prepare <change> 由 apply 技能在 in-progress add 之前呼叫，無 stdin、無旗標。change 尚未帶 started_* 時，每次 prepare 都以當下狀態原子取代未正式開工的舊 baseline，confidence=initial；change 已 started 且 baseline 存在時保持 first baseline；已 started 但 baseline 缺失時只記 confidence=late；沒有可用 Git root 時記 unavailable。late／unavailable baseline 只能提供診斷，resolver 不得自動認領 change hunks。

正常 initial capture exit 0 且無 stdout；late／unavailable 以 stderr 警告但仍 exit 0，使 apply 可以繼續。sidecar 寫入採同目錄暫存檔後 rename；寫入失敗為非零 exit，且不呼叫後續 in-progress add。

選擇 sidecar 而非 change metadata／TeamStore，是因資料只對持有 code checkout 的 Host 有意義；remote CLI 仍在本地 checkout 建立同一檔。選擇獨立目錄而非 touched 欄位，是為了讓現有 evidence、commit、archive、drift 消費者完全不變。

### D2 — Host change-diff resolver 是唯一 Git 與 hunk 語意

crates/speclink-host/src/change_diff.rs 定義 baseline capture、candidate resolution、ambiguity adjudication 與 snapshot I/O。CLI local adapter從 fs Store 取得 active changes；remote adapter 從 typed client 取得相同清單與 review ticket；兩者把 host-local touched records、Workspace 與選擇參數送進同一 resolver，不在 CLI／server 各寫一套 Git 演算法。

Discovery candidate 的 tracked 部分等價於：

git diff --find-renames --binary <baseCommit> -- <touchedPaths>

這是 commit tree 到目前 worktree 的雙端比對，包含 staged 與 unstaged；禁止改用 <base>...HEAD。touched 且 untracked 的檔案另建為整檔 addition。resolver 排除 openspec artifacts 與 .speclink work data，但不擴張到 touchedPaths 以外。

文字 patch 解析每個 @@ header 為：

- id：sha256(path identity、old/new range 與 hunk body)
- oldStart、oldLines、newStart、newLines：無號整數
- addition 允許 oldStart=0、oldLines=0；deletion 允許 newStart=0、newLines=0

每個 file delta 另帶 oldPath／newPath、modified／added／deleted／renamed／binary kind，以及 raw bytes 的 beforeHash／afterHash；不存在的一側為 null。patchHash 與 candidateHash 均為 canonical patch bytes 的 sha256。binary delta 無文字 ranges，只能在自動無歧義 scope 中整檔納入；dirty binary 的人工拆 hunk 不在本變更支援。

### D3 — 歧義先輸出 candidate，再以 hash-pinned selection 解鎖

下列任一條件讓 speclink review scope 進入 needsInput，且不得建立 frozen snapshot：

- baseline 缺失、late、unavailable，或 base commit 在目前 repo 不可解析
- touched path 已列在 initial baseline 的 dirtyFilesAtStart
- 另一個 active change 的 host-local touched record 也認領同一路徑
- remediation 續輪所需 snapshot 缺失、patch hash 不符，或修正碰到先前已髒但未保存內容的檔案

review scope <change> 支援 --json、--base <rev>、--candidate-hash <sha256> 與可重複 --include-hunk <id>，無 stdin。--base 只補可信 fixed point，不能自行消除同檔混合 hunks；人工 hunk 選擇必須同時提交前一次 candidateHash，resolver 重算 candidate 後 hash 不同即非零拒絕。所有 include-hunk 必須存在、至少一筆且不得指向 binary；成功 selection 只凍結所選 hunks，但保留其實際檔案 before／after hashes 作漂移錨。

隔離 worktree 能直接消除 dirty／overlap 條件，無需旗標。這三條處置——可信 base、hash-pinned hunk selection、隔離 worktree——都由使用者或 agent 明示；resolver 永不猜測。

### D4 — Frozen review snapshot 只保存審查面

成功 scope 寫入 .speclink/review-scopes/<change>/snapshots/<digest-hex>.json；檔名只取 patchHash 的十六進位 digest，不含 `sha256:` 前綴與冒號。格式 version=1，包含 change、phase、candidateHash、patchHash、baseCommit、createdAt、dirtyFilesAtCapture 的 path/hash 清單、canonical patch、file deltas、hunk ranges，以及文字 scope 檔的 beforeText／afterText；非 UTF-8／binary 僅保留 hashes。snapshot 寫入以暫存檔＋rename 完成，成功前不修改工單。

Discovery snapshot 限於 touched candidate；validation snapshot 限於上輪未解 finding paths、這些路徑自上輪 afterText 至目前內容的差異，以及上輪 capture 後才變髒的新路徑。上輪已髒且不在保存 scope 的檔案若內容 hash 改變，resolver 無法重建 before，依 D3 needsInput。

scope 成功後、review add-round 失敗會留下 orphan snapshot；下一次無對應工單的 scope 會先清除 orphan 後重算。review stamp／discard 成功後清除該 change 的 host-local snapshots；清除失敗只警告，不回滾已完成的 canonical 工單／metadata mutation。工單存在但 referenced snapshot 被刪除時，validation fail closed。

此保存面是「review-time snapshot」，不是 Apply provenance：它只從一次審查開始保存有限 scope，不承諾在另一 Host 重播。

### D5 — 工單以 phase 與 patch hash 綁定 snapshot

新技能寫入的每輪 stdin 依序包含：

- **Phase**: discovery 或 validation
- **Patch**: sha256:<hex>
- **Scope**: 既有 repo-root 相對路徑清單
- 既有分級 findings

core parser 將 phase 與 patchHash 解析為 Round 的 nullable 欄位；缺席代表 legacy round，既有內容仍能 show、stamp、discard。phase token 或 patch hash 格式錯誤時 add-round 非零拒絕且工單零寫入。

review show --json 的 rounds[] 與 lastRound 增加 phase: string|null、patchHash: string|null；既有 change、index、scope、findings 欄位與人眼原文維持。Protocol DTO、remote client 與 server mapping 共用同一 additive shape。stamp 守門與 reviewed_scope 指紋規則不變；phase／patch 不成為章欄位。

### D6 — Round 1 保留 Matt-compatible discovery 形狀

review skill 先呼叫 review scope 取得 frozen discovery patch，再把同一 patch、artifact intent 與 locale 同時送給兩個 read-only axes：

- Standards：repo 文件優先＋既有 smell baseline，只對 change hunks 及其必要上下文判斷
- Correctness：bug hunting，只對 change hunks、呼叫端與測試的直接行為判斷

兩份報告原樣並列，不合併、不跨軸重排；必修／可裁 triage 只在報告之後作 remediation routing。Spec 合規仍由 verify 處理。主線把 Round 1 findings 原文寫入 ticket；後續 validation 若同一 finding 未解，必須原文帶入，避免靠改寫文字假裝集合縮小。

### D7 — Round 2+ 是 remediation validation，嚴格進展決定終止

使用者選擇修正後重審時，主線仍先依 repo 慣例跑完整建置與測試門，再呼叫 review scope 產生 validation patch。Validation brief 只含：

- 上輪未解 findings 與已接受清單
- remediation patch
- 必要的相鄰呼叫端、測試與 artifact intent

對應 axis 只回報每筆原 finding 已解／未解，以及 remediation patch 直接造成的新 regression；不得新增未修改區域的 smell 或 SUGGESTION。未解原 finding 在新 ticket round 原文保留，直接 regression 以新 finding 記錄；已接受事項仍以 (accepted) 原樣前饋。

令 Bn 為第 n 輪 triage 後的未接受必修集合：

- Bn 為空且無 accepted：review stamp，passed clean
- Bn 為空且有 accepted：由使用者明示 review stamp --accept，passed with reservations
- 0 < |Bn| < |Bn-1|：允許再次選擇修正
- |Bn| >= |Bn-1|：記錄本輪後立即 failed，保留工單、不蓋章、不自動再試

因此每次自動續跑都嚴格下降，不需固定最大輪數。與 remediation patch 無關的新問題不進 Bn；若同時具現實觸發路徑、重現／失敗測試／明確 invariant 證據，且影響安全、資料損失或錯誤行為，本站以 scope changed／failed 結束，另開 discovery 或衍生 change。

### D8 — Apply／Review 技能與 local／remote entry points 共用契約

Apply 的 claude／codex 正典模板在 status 成功後、in-progress add 前執行 review prepare；既有 task loop 不變。Review 模板以 review scope 的 patch 取代 touched 整檔清單，並實作 D6／D7。

Local review prepare／scope 直接組合 Workspace、fs Store 與 Host resolver；remote CLI 仍在 code checkout 執行同一 resolver，僅透過 typed client 取得 active changes／ticket並將 add-round／stamp／discard 寫回 TeamStore。Server 不新增 Git endpoint，不保存 baseline／snapshot。Remote revision conflict、離線與認證錯誤仍只發生在既有 store 動詞；scope 的本地 Git 錯誤不偽裝成 remote error。

speclink update 再生 claude 與 codex 的 apply／review 技能；golden 同批更新。根層 workflow 行、其他技能與 UI 不變。

## Implementation Contract

**In scope**

- speclink-host：baseline、Git candidate、ambiguity、selection、snapshot 與 remediation diff 的唯一演算法
- speclink-core：Workspace host-local 路徑 helper、review round phase／patch parser、技能正典與 golden
- speclink-cli：review prepare／scope 參數、local／remote 組裝、human／JSON 呈現與 snapshots cleanup
- speclink-protocol／remote／server：review show additive round 欄位
- review skill：discovery／validation 分流、嚴格進展、重大晚發問題逃生口

**Out of scope**

- touched JSON、TeamStore document schema、review stamp metadata、desktop UI、archive 行為、完整 Apply provenance、verify 實作

**review scope --json resolved payload**

- change: string
- phase: discovery|validation
- state: resolved
- baseCommit: string
- candidateHash: sha256 string
- patchHash: sha256 string
- paths: string[]
- files: array；每項含 oldPath: string|null、newPath: string|null、kind: string、beforeHash: string|null、afterHash: string|null、hunks: array
- hunks[]：id: string、oldStart: number、oldLines: number、newStart: number、newLines: number
- patch: string

歧義時 --json 仍將 state: needsInput、candidateHash、ambiguousPaths、files 與可選 hunk IDs 寫至 stdout，但 process exit 非零；human 路徑將原因與三種處置寫至 stderr。--no-color 下兩條路徑皆無 ANSI。candidate 漂移、無效 hunk ID、binary hunk selection、找不到 change／Git rev 或 snapshot 不符皆非零且不得寫 frozen snapshot。

**review show --json additive fields**

rounds[] 與 lastRound 的 phase、patchHash 為 string|null；其他欄位、型別與排序不變。Local 與 remote payload 必須逐欄同構。

**Acceptance criteria**

1. initial baseline 精確記錄 HEAD、開始前 dirty paths 與 UTC 時間；touched fixture 位元級不變
2. fixed point 至 worktree 的測試同時覆蓋 staged、unstaged、untracked addition、delete、rename、多 hunk、addition oldLines=0、deletion newLines=0 與 binary
3. <base>...HEAD 對同一未提交 fixture 為空時，resolver 仍輸出 worktree patch，釘住正確 Git 語意
4. dirty-at-start、active-change overlap、candidate 漂移與 snapshot 缺失皆零 snapshot effects
5. hash-pinned hunk selection 只輸出被選 hunks，hash 不符時拒絕
6. legacy review ticket 可 show／stamp／discard；新 ticket 的 phase／patchHash 在 local／remote JSON 同構
7. skill golden 明確包含唯一 discovery、validation 不探索新事項、必修集合不縮小即 failed
8. 以 2 筆必修開始的 fixture：2→1 可續跑，1→1 立即停止且無 stamp；1→0 乾淨蓋章；只剩 accepted 走 --accept
9. cargo test -p speclink-host、cargo test -p speclink-core、cargo test -p speclink-cli、cargo test -p speclink-protocol、cargo test -p speclink-remote 與 server review API 測試全綠

## Risks / Trade-offs

- **回歸對照**：review ticket／JSON 增欄會影響 strict fixture → parser 保持 nullable legacy，相容案例與 local／remote parity 測試同批更新
- **跨平台 Git**：rename、line ending、path separator、non-UTF-8 與 git diff --no-index exit 1 容易誤判 → Host 單點包裝 Git、正斜線正規化，Windows／macOS／Linux fixture 覆蓋
- **dirty overlap false positive**：另一 active change 的 touched record 會因歷史殘留而過時 → 只對 active change 啟動守門，並提供 hash-pinned selection；不為降低摩擦而靜默認領
- **snapshot 含原始碼**：review-time text snapshot 增加 host-local code copy → 僅放 gitignored .speclink、只保存 scope、stamp／discard 後清理，不上傳 TeamStore
- **大型 diff**：canonical patch 與 before／after text 會增加 local work data → touched paths 先限界；本變更不加入任意全 repo fallback
- **跨 Host 接手**：remote ticket 在另一 checkout 找不到 snapshot → 明示 fail closed；完整跨 Host 重播留給未來 provenance change
- **兩階段 side effect**：scope snapshot 與 add-round 無法跨 local sidecar／TeamStore 原子提交 → 先原子寫 snapshot、再寫 ticket；orphan 可重算，ticket 指向缺失 snapshot則拒絕 validation

## Migration Plan

- 無資料遷移；既有 touched、metadata 與 review ticket 保持可讀
- 既有 in-progress change 沒有 initial baseline，首次新 review 需明示可信 base；既有 follow-up ticket 沒有 snapshot 時不得自動續輪，使用者可保留工單停止，或明示 discard 後以固定 scope 重新 discovery
- rollback 可移除新技能與 resolver；新增 nullable ticket 行不影響舊 parser 的 Scope／findings，host-local review-scopes 可安全忽略

## Open Questions

無。無 Git／跨 Host／workspace 前進後重播的完整 provenance 已明示 deferred，不阻擋本變更。
