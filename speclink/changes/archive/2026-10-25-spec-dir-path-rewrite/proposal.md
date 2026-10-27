## Why

spec-dir-migration 與 desktop-spec-dir-migration 把預設規格資料夾改為 `speclink/`、變更資料檔改為 `.meta.yaml`，但正式規格仍有 42 份規格、151 條需求與 6 段 Purpose 以 `openspec/`、`.openspec.yaml` 或「openspec 樹」描述產品行為，使用者文件（workflow、verb-contract、product-status、sdk-node、development、README 的文件地圖）與 LANGUAGE.md 的詞條定義也一樣。這些文字在遷移上線後就與行為不符，AI agent 讀正式規格時會照舊路徑操作。本變更是討論 positioning-and-process-cost 分刀計畫的第 8 刀，緊接在遷移的兩個變更之後、同一版發布。

目標使用者與情境：閱讀正式規格與使用者文件的開發者與 AI agent（團隊情境下含 PO 與 PM），在 discuss、propose、apply、verify 等會讀正式規格的階段，以及查閱文件的時候。

## What Changes

- 以 MODIFIED delta 改寫 151 條需求裡的舊名稱（含描述 speclink repo 自身檔案的需求）：`openspec/` → `speclink/`、`.openspec.yaml` → `.meta.yaml`、指資料夾的「openspec」字樣 → 「speclink」；行為不變。
- change-plan 的情境「缺 .openspec.yaml 的 change 零寫入拒絕」改名為「缺 .meta.yaml 的 change 零寫入拒絕」，以 REMOVED-SCENARIO 註解宣告。
- 直接修改 change-lifecycle、desktop-manual-page、manual-pages、store-abstraction、ui-copy-vocabulary、workflow-config 六份正式規格 Purpose 段裡的舊名稱（Purpose 不經 delta）。
- 使用者文件（中英）：docs/workflow、docs/verb-contract、docs/product-status、docs/sdk-node、docs/development 裡描述使用者專案的路徑，以及 README 文件地圖的設定檔路徑，改為新名稱；docs/sdk-node 的 `specDir` 預設值改為 `speclink`。
- openspec/LANGUAGE.md 詞條定義裡的產品路徑改為新名稱。
- 由於正式規格可能在本變更封存前被其他變更修改，apply 時以腳本從當下的正式規格重新產生 delta，覆蓋 propose 時的版本。

## Non-Goals

- 不改 spec-dir-migration 與 desktop-spec-dir-migration 新增或修改的需求：它們已用新名稱，且刻意提到舊名稱以描述相容與遷移。
- 不改使用者文件與 openspec/LANGUAGE.md 裡描述 speclink repo 自己檔案位置的文字與指向 repo 內檔案的相對連結（例如 README「`openspec/` 是 Speclink 自己的規格」、文件裡指向 `../openspec/specs/...` 的連結、docs/development 的 repo 目錄表）：它們要與 repo 實際搬家放在同一個變更（self-spec-dir-migration）。正式規格不在此限：描述 repo 自身檔案的需求與 Purpose（例如 ui-copy-vocabulary、delivery-baseline 描述的守門範圍）也在本變更改為新名稱，repo 的檔案、工具與文件由 self-spec-dir-migration 跟上；從本變更封存到 self-spec-dir-migration 提交之間，這些需求寫 `speclink/`、repo 仍在 `openspec/`，兩個變更同一版發布。
- 不改已封存的變更與討論。
- 不改 OpenSpec（大寫，指上游專案）的字樣。
- 不改任何程式。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `archive-skill`: 路徑字樣改寫（不改行為）。
- `baseline-skill`: 路徑字樣改寫（不改行為）。
- `board-card-order`: 路徑字樣改寫（不改行為）。
- `capability-naming-guard`: 路徑字樣改寫（不改行為）。
- `change-diff-scope`: 路徑字樣改寫（不改行為）。
- `change-lifecycle`: 路徑字樣改寫（不改行為）。
- `change-plan`: 路徑字樣改寫（不改行為），並把情境「缺 .openspec.yaml 的 change 零寫入拒絕」改名。
- `client-protocol`: 路徑字樣改寫（不改行為）。
- `command-runtime`: 路徑字樣改寫（不改行為）。
- `commit-skill`: 路徑字樣改寫（不改行為）。
- `config-skill`: 路徑字樣改寫（不改行為）。
- `delivery-baseline`: 路徑字樣改寫（不改行為）。
- `desktop-app`: 路徑字樣改寫（不改行為）。
- `desktop-config`: 路徑字樣改寫（不改行為）。
- `desktop-manual-page`: 路徑字樣改寫（不改行為）。
- `dev-harness`: 路徑字樣改寫（不改行為）。
- `discuss-skill`: 路徑字樣改寫（不改行為）。
- `discussion-docs`: 路徑字樣改寫（不改行為）。
- `improve-skill`: 路徑字樣改寫（不改行為）。
- `manual-pages`: 路徑字樣改寫（不改行為）。
- `manual-skill`: 路徑字樣改寫（不改行為）。
- `propose-skill`: 路徑字樣改寫（不改行為）。
- `release-skill`: 路徑字樣改寫（不改行為）。
- `remote-board-order`: 路徑字樣改寫（不改行為）。
- `remote-connection`: 路徑字樣改寫（不改行為）。
- `review-skill`: 路徑字樣改寫（不改行為）。
- `review-station`: 路徑字樣改寫（不改行為）。
- `skill-authoring`: 路徑字樣改寫（不改行為）。
- `skill-routing`: 路徑字樣改寫（不改行為）。
- `trace-verb`: 路徑字樣改寫（不改行為）。
- `tray-status-menu`: 路徑字樣改寫（不改行為）。
- `ui-copy-vocabulary`: 路徑字樣改寫（不改行為）。
- `user-documentation`: 路徑字樣改寫（不改行為）。
- `verb-contract`: 路徑字樣改寫（不改行為）。
- `verify-station`: 路徑字樣改寫（不改行為）。
- `workflow-config`: 路徑字樣改寫（不改行為）。
- `workflow-schemas`: 路徑字樣改寫（不改行為）。
- `workspace-chooser`: 路徑字樣改寫（不改行為）。
- `workspace-migration`: 路徑字樣改寫（不改行為）。
- `workspace-session`: 路徑字樣改寫（不改行為）。
- `workspace-tools`: 路徑字樣改寫（不改行為）。
- `worktree-overlay`: 路徑字樣改寫（不改行為）。

## Impact

- Affected specs: `archive-skill`, `baseline-skill`, `board-card-order`, `capability-naming-guard`, `change-diff-scope`, `change-lifecycle`, `change-plan`, `client-protocol`, `command-runtime`, `commit-skill`, `config-skill`, `delivery-baseline`, `desktop-app`, `desktop-config`, `desktop-manual-page`, `dev-harness`, `discuss-skill`, `discussion-docs`, `improve-skill`, `manual-pages`, `manual-skill`, `propose-skill`, `release-skill`, `remote-board-order`, `remote-connection`, `review-skill`, `review-station`, `skill-authoring`, `skill-routing`, `trace-verb`, `tray-status-menu`, `ui-copy-vocabulary`, `user-documentation`, `verb-contract`, `verify-station`, `workflow-config`, `workflow-schemas`, `workspace-chooser`, `workspace-migration`, `workspace-session`, `workspace-tools`, `worktree-overlay`
- Affected code:
  - Modified: openspec/specs/change-lifecycle/spec.md, openspec/specs/desktop-manual-page/spec.md, openspec/specs/manual-pages/spec.md, openspec/specs/store-abstraction/spec.md, openspec/specs/ui-copy-vocabulary/spec.md, openspec/specs/workflow-config/spec.md, openspec/LANGUAGE.md, README.md, README.zh-TW.md, docs/workflow.md, docs/workflow.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md, docs/sdk-node.md, docs/sdk-node.zh-TW.md, docs/development.md, docs/development.zh-TW.md
- 影響的 crate／app：沒有；只改正式規格與文件文字。
- 設定欄位：沒有。
- 生成的技能：沒有，不升 ASSET_VERSION。
- 相容性影響：沒有行為變動；change-plan 一個情境的名稱改變。
