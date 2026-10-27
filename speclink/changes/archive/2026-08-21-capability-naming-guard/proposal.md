## Why

capability 的取名靠 AI 判斷，引擎的 archive 只按資料夾路徑合併：既有 `auth` 卻另建 `authentication` 時，會靜默產出兩份語意重複的正典規格。上游 OpenSpec 沒有確定性防護（catalog 提案 #901 懸置、路線只走指令強化），而本專案已有 71 個正典 capability，正處於「規格一多、AI 掃描就漏」的風險區。目標使用者是透過 AI 代理跑 SDD 的開發者；情境對應 propose 與 ingest 兩個階段——AI 替 change 建立 delta spec、決定 capability 名字的時刻。

## What Changes

- **BREAKING** 建立點主閘：`speclink new artifact spec <capability> --change <name>` 遇到正典規格沒有的 capability 名稱時，從「靜默建立」改為「預設拒絕」。錯誤訊息列出最多三個近似既有名（來源含正典規格與其他進行中 change 的 delta capabilities，各附來源標注與 Purpose 首行），並指引兩條路：沿用既有名，或帶新旗標 `--new` 重跑以顯性宣告「這是新 capability」。拒絕歸類為既有錯誤碼 `refused`（前置條件拒絕），exit code 非零，stdin 內容不落盤。
- 近似名單排序工具：token 完全包含 > kebab token 交集 > 編輯距離，取前三；只排序建議、不設擋人的相似度門檻。實作放在 speclink-core 內部模組，不另立 crate、不改 store trait（組合既有的正典列舉、delta 列舉與規格讀取）。
- validate 第二網：`speclink validate` 對「正典無同名」的新 capability，若名稱與既有名（正典＋其他進行中 change 的 delta）高度相似，報 warning 級 lint，訊息附「同一 capability 就改用既有名；確為新 capability 可忽略」。propose 收尾自動跑 validate，故提案階段即可見；此網同時涵蓋 ingest 或手寫檔案繞過 CLI 建立點的入口。
- AI 指令側（技能資產）：`assets/skills/propose.md` 加三項——既有規格掃描結果須留痕於 proposal、New Capabilities 每項附一句「為何既有規格不涵蓋」、寫明 `--new` 旗標的語意與使用時機；`assets/skills/ingest.md` 加一項——新增 delta capability 前先對照既有名。影響 claude 與 codex 兩個工具的生成技能；資產變更連動 MARKER_VERSION 版號、golden snapshots 與 assets.lock，並由 `speclink update` 再生各專案技能檔。
- archive 不加新防護：維持既有的新 capability Purpose 守門與 ADDED-only 限制。

## Non-Goals

- 不做相似度門檻硬擋（正典命名家族如 archive-merge／archive-skill 會被誤殺；相似度只用於排序建議）。
- 不在 archive 增加命名檢查（實作完成後才擋代價最高，且引擎直跑無互動確認）。
- 不為 `--new` 另留 metadata 痕跡（新 capability 已被 Purpose 守門強制說明用途）。
- 不處理純語意重複（`login` vs `auth` 字面不像，字串比對抓不到；Purpose 文字留作未來語意比對素材）。
- 不處理 worktree 內新建 delta 對主 checkout 不可見的跨 change 比對盲區（接受為已知限制，於 design 記載）。
- 不涉及設定欄位（openspec/config.yaml／.speclink.yaml 無新增）。

## Capabilities

### New Capabilities

- `capability-naming-guard`: capability 命名守門機制——建立點確認制主閘、近似名單排序規則、propose／ingest 技能資產的命名守門指令面。既有規格不涵蓋的原因：spec-validation 管 change 驗證的結果語意、command-runtime 管動詞跨入口共通語意與錯誤碼分類，皆不含「capability 名稱與既有規格的一致性」這個關注點；目前沒有任何 capability 規範 new artifact spec 的名稱守門行為。

### Modified Capabilities

- `spec-validation`: 新增「新 capability 近似名 warning lint」需求——validate 對正典無同名且與既有名相似的 delta capability 報 warning。

## Impact

- Affected specs: `capability-naming-guard`（新增）、`spec-validation`（修改）
- Affected code:
  - New: `crates/speclink-core/src/capname.rs`（近似名排序與名單組裝）
  - Modified: `crates/speclink-core/src/newcmd.rs`（建立點主閘與 --new 參數）、`crates/speclink-cli/src/verbs/new.rs`（CLI 旗標）、`crates/speclink-core/src/command/mod.rs`（命令層 argv 傳遞與 refused 分類）、`crates/speclink-core/src/validate.rs`（warning lint）、`crates/speclink-core/assets/skills/propose.md`、`crates/speclink-core/assets/skills/ingest.md`、`crates/speclink-core/src/init.rs`（MARKER_VERSION 版號）、`crates/speclink-core/tests/golden/assets.lock` 與同目錄 golden snapshots（資產連動再生）
  - Removed: （無）
- 相容性影響：`new artifact spec` 對正典未收錄名稱的行為由成功轉為 refused 拒絕——人眼輸出新增錯誤訊息與近似名單，exit code 由 0 變非零；帶 `--new` 或名稱命中正典時行為與輸出不變。既有回歸對照中「以全新名稱建 spec」的路徑需補 `--new`。使用者遷移：錯誤訊息自帶兩條路的指引，且技能資產同步更新後，AI 代理照新指令自然帶旗標。`--json` 的成功 payload（artifact、change、path、status、validated、warnings 欄位）維持現行形狀；主閘拒絕走既有錯誤路徑，`--json` 下的錯誤呈現與此指令現行其他錯誤（如缺 capability 名稱）一致。
