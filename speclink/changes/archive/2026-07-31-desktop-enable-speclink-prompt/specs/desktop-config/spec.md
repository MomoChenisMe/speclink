## ADDED Requirements

### Requirement: 未啟用資料夾經確認後補齊啟用

所選目錄向上探索命中 workspace、store mode 為本地檔案、且該 workspace root 不存在 `.speclink.yaml` 時，app SHALL 判定為未啟用 speclink，SHALL NOT 逕行寫入亦 SHALL NOT 直接以既有專案開啟，而 SHALL 顯示啟用確認對話框（含 AI 工具多選 claude／codex，預設勾選 claude；文案為啟用語意，遵循 openspec/LANGUAGE.md、不出現工程詞）。判定與寫入 SHALL 錨定向上命中的 workspace root，而非使用者所選的子目錄。

使用者確認後 app SHALL 經引擎的工作區補齊入口執行啟用（補 openspec/ 骨架缺件、專案根 `.speclink.yaml` 記錄所選 tools、為每個所選工具生成指令檔受管區塊與 skills 檔），既有 openspec/ 內容 SHALL 零觸碰，隨即切換至該專案；使用者取消時 app SHALL 維持原專案，目標目錄 SHALL NOT 產生任何寫入。啟用失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。`.speclink.yaml` 存在的專案 SHALL 照舊直接開啟，SHALL NOT 出現啟用對話框；向上探索完全未命中的目錄 SHALL 照舊走初始化確認流程。

#### Scenario: 遷移資料夾確認啟用後補齊並切入

- **WHEN** 使用者選定含 openspec/（內有既有規格文件）但無 .speclink.yaml 的資料夾，於啟用確認對話框保持預設（claude）並確認
- **THEN** 專案根產生 .speclink.yaml（tools 含 claude）、CLAUDE.md 的受管區塊與 .claude/skills/ 技能檔，openspec/ 內既有文件位元級不變，app 切換至該專案並於看板呈現既有內容

#### Scenario: 取消啟用則零寫入

- **WHEN** 使用者於啟用確認對話框取消
- **THEN** app 維持原專案，所選資料夾內容與選擇前完全相同

#### Scenario: 已啟用專案不出現啟用對話框

- **WHEN** 使用者選定專案根含 .speclink.yaml 的資料夾開啟
- **THEN** app 直接開啟該專案進看板，無啟用對話框

#### Scenario: 子目錄開啟錨定專案根

- **WHEN** 使用者選定未啟用專案的子目錄開啟並確認啟用
- **THEN** .speclink.yaml 與工具檔產生於向上命中的專案根，app 切入該根

#### Scenario: 既有工作流設定不被覆蓋

- **WHEN** 未啟用資料夾的 openspec/config.yaml 已存在且含使用者自訂政策，使用者確認啟用
- **THEN** 該檔位元級不變，僅補齊其餘缺件
