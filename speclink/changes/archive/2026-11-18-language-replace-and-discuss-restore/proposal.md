## Why

規格存在 Server（宿主或示範用 server）時，有兩件事今天完全做不到，兩項都列在專案能力狀態的已知缺口：

1. 詞彙表（`LANGUAGE.md`）不能寫。只有 `speclink language show` 可以讀。本機專案可以直接改檔案；遠端專案與內嵌宿主沒有檔案可以改，所以詞彙表永遠是空的，或停在匯入當時的內容。discuss 技能在結論發現用詞偏移時會要求把它記進詞彙表，這一步在遠端沒有辦法完成。
2. 封存錯的討論搬不回來。例如最後一刀的旗標下錯，討論記錄跟著變更一起被封存。本機專案可以手動把檔案搬回來；遠端沒有任何動詞可以做，技能只能停下來請使用者到 Server 那一端處理。

受影響的人：在宿主裡經 Agent 做 discuss 的非工程角色（結論的用詞要落到詞彙表）、連到遠端專案的工程師、以及維護規格的人（討論封存錯了要挽回）。對應的技能是 discuss 與 propose。來源討論 embedded-host-needs-2 裁定：新增兩個動詞，兩個入口都可用，並經過執行前掛鉤。

掃過的相關正式規格：verb-contract（「store 文件讀取動詞」規定 `language show`）、discussion-docs（討論動詞與封存）、discuss-skill（「正式規格與變更文件一律經動詞讀取」裡有搬回記錄的條件步驟）、propose-skill（「從討論轉出最後一刀時帶 --last」裡有標錯的救援）、server-verb-api（端點）、host-kit（每個引擎命令都要有線上的歸屬）。

## What Changes

- 新增動詞 `speclink language replace --stdin`：把 stdin 的內容整份寫成詞彙表。
  - 詞彙表不存在時建立，存在時整份取代。內容是空的（或只有空白）時拒絕。
  - 用一般的寫入權限，不算政策鍵。
  - 人眼輸出：`✓ Replaced the LANGUAGE document`（新建時是 `✓ Created the LANGUAGE document`）。`--json` 輸出 `{ "created": <布林> }`。
  - 引擎命令名稱 `language-replace`，事件 `language-replaced`。
- 新增動詞 `speclink discuss restore <slug>`：把已封存的討論記錄搬回進行中。
  - 同一個 slug 有多份封存時，搬回最新的一份。記錄的內容逐字元不變。
  - 同一個 slug 已經有進行中的記錄時拒絕；沒有封存的記錄時回報找不到。
  - 人眼輸出：`✓ Restored discussion: <slug>`，下一行是 `  From: <封存時的名稱>`。`--json` 輸出 `{ "slug", "restoredFrom" }`。
  - 引擎命令名稱 `discuss-restore`，事件 `discussion-restored`。
- 線上格式新增兩個端點：`PUT /language`（本文 `{ "content" }`）與 `POST /discussions/{slug}/restore`。CLI 連到遠端專案時經這兩個端點執行。
- 四個「討論已封存」的拒絕訊息（繼續討論、轉出、連結、seal）改成指向新動詞：原本寫「move it out of discussions/archive/」，改成「run `speclink discuss restore <slug>`」。
- 技能的原稿：
  - discuss：結論把用詞偏移記進詞彙表的步驟，改成用 `language show` 讀出、修改後用 `language replace --stdin` 整份寫回，不再分「規格在不在這個 checkout」。「封存錯了」的說明改成執行 `discuss restore`，同樣不再分兩種情況。
  - propose：最後一刀旗標標錯的救援改成執行 `discuss restore`。
  - 兩支技能的原稿改動會讓內嵌資產的版本號前進，golden 與資產鎖定檔同批更新。
- 使用者文件：專案能力狀態把兩項已知缺口移到「已經解決」的清單；動詞契約文件補上兩個動詞、兩個端點與兩個事件；中英兩版同步。

相容性影響：

- 既有動詞的人眼輸出與 `--json` 輸出不變，只有上面四個拒絕訊息的後半句是刻意變更（exit code 不變）。
- 一個邊界情況的讀取結果刻意變更：封存名稱帶 `-N` 序號（同一天重複封存的寫法）、而那份記錄自己的 frontmatter 寫明是另一個 slug 時，不再把它當成這個 slug 的封存記錄。例如 `embedded-host-needs-2` 封存之後，`discuss show embedded-host-needs` 與 `discuss restore embedded-host-needs` 讀到的是 `embedded-host-needs` 自己的那一份，不是 `-2` 那一份。沒有寫 slug 的記錄照原本的方式依名稱判斷。
- 引擎新增兩個命令與兩個事件：命令名稱與參數的 golden 刻意更新。事件 `discussion-restored` 歸到該討論，`language-replaced` 歸到「其他」（用戶端重新讀取全部）。
- 技能的渲染 golden 與資產版本號刻意更新；使用者執行 `speclink update` 後拿到新的技能文字。影響 discuss 與 propose 兩支技能，全部工具（claude、codex 與其他已支援的工具）都一樣。
- 線上只新增端點，不改既有回應。新版 CLI 連到舊版 Server 執行這兩個動詞時，Server 回 404；CLI 以非零結束，訊息說明這台 Server 不支援這個動詞、請更新 Server（比照 `claim --release` 的做法）。

三種形態：

- 本機 repo：兩個動詞都可用。`language replace` 寫規格資料夾裡的 `LANGUAGE.md`；`discuss restore` 把檔案從封存資料夾搬回來並拿掉檔名的日期。
- 連到遠端的 checkout：兩個動詞都可用，經新的端點執行。
- 沒有本機規格檔的內嵌宿主：兩個動詞都可用，經 `host.dispatch` 執行，先問執行前掛鉤。

不涉及設定欄位。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 新增一條需求——詞彙文件的寫入動詞。
- `discussion-docs`: 新增一條需求——以 restore 動詞把已封存的討論搬回進行中，以及「已封存」拒絕訊息的指路。
- `server-verb-api`: 新增一條需求——詞彙文件寫入與討論搬回的兩個端點。
- `discuss-skill`: 「正式規格與變更文件一律經動詞讀取」改寫——搬回記錄的步驟改用動詞、不再帶條件；另新增一條需求——用詞偏移以動詞寫回詞彙表。
- `propose-skill`: 「從討論轉出最後一刀時帶 --last」改寫——標錯的救援改用 restore 動詞。

## Impact

- Affected specs: verb-contract、discussion-docs、server-verb-api、discuss-skill、propose-skill
- Affected crates: speclink-core（兩個命令、兩個事件、儲存介面的兩個方法、技能原稿與資產版本）、speclink-fs（本機儲存的實作）、speclink-verbs（解析與輸出）、speclink-host（儲存接口上的實作、路由、事件歸類）、speclink-protocol（請求與回應型別）、speclink-remote（用戶端方法）、speclink-cli（遠端分支）
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/src/store.rs
    - crates/engine/speclink-core/src/teststore.rs
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/command/typed.rs
    - crates/engine/speclink-core/src/command/workflow_config.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/assets/skills/discuss.md
    - crates/engine/speclink-core/assets/skills/propose.md
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/engine/speclink-core/tests/it/command_describe.rs
    - crates/engine/speclink-core/tests/it/skill_verbization.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-remote.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/cursor.snapshot.md
    - crates/engine/speclink-core/tests/golden/pi.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call-remote.snapshot.md
    - crates/engine/speclink-fs/src/lib.rs
    - crates/engine/speclink-fs/tests/store_fs.rs
    - crates/adapters/speclink-verbs/src/parse.rs
    - crates/adapters/speclink-verbs/src/inputs.rs
    - crates/adapters/speclink-verbs/src/tool.rs
    - crates/adapters/speclink-verbs/src/families/mod.rs
    - crates/adapters/speclink-verbs/src/families/documents.rs
    - crates/adapters/speclink-verbs/src/families/discuss.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/documents.rs
    - crates/adapters/speclink-verbs/src/render/discuss.rs
    - crates/protocol/speclink-protocol/src/command.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/host/speclink-host/src/port_view.rs
    - crates/host/speclink-host/src/worktree.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/drift.rs
    - crates/host/speclink-host/src/read_only.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/coverage.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-host/tests/port_dual_path.rs
    - crates/host/speclink-host/tests/common/mod.rs
    - crates/host/speclink-server/src/api/events.rs
    - crates/host/speclink-server/tests/it/api/write_guard.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/verbs/discuss.rs
    - crates/adapters/speclink-cli/src/verbs/documents.rs
    - crates/adapters/speclink-cli/tests/it/doc_verbs.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed: (none)
  - 另外：`speclink update` 會重新產生本 repo 已安裝的技能檔（`.claude/skills` 與 `.agents/skills` 底下的 discuss 與 propose 等），這些是衍生物，收尾時一併提交
