## ADDED Requirements

### Requirement: commit 確認閘門所見即所簽

內嵌 speclink-commit 技能（事實來源 crates/speclink-core/assets/skills/commit.md，經 init 與 update 渲染至各工具技能目錄）SHALL 規定以下確認流程：commit 訊息 SHALL 於使用者確認之前生成；commit 計畫（分組檔案清單）與 commit 訊息 SHALL 在呼叫確認工具之前以可見文字一次輸出；確認問題的文字 SHALL NOT 指涉對話中未曾輸出的內容。本能力屬 Speclink 自身延伸、非 Spectra parity 對照面；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物將訊息生成排在確認之前

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-commit 技能檔 SHALL 將「生成 commit 訊息」步驟置於使用者確認步驟之前，且確認步驟 SHALL 引用先前已輸出的計畫與訊息

#### Scenario: 渲染產物含可見性守則

- **WHEN** 檢視渲染產出的 speclink-commit 技能檔的 Guardrails 段落
- **THEN** 該段落 SHALL 含以下守則：呼叫確認工具前，commit 計畫與 commit 訊息必須已作為可見文字輸出；確認問題不得指涉未曾輸出的內容

#### Scenario: archive 子流程後重新確認

- **WHEN** 使用者於確認閘門選擇 Archive first, then commit together 且 archive 子流程執行完成
- **THEN** 技能檔 SHALL 規定重新輸出更新後的 commit 計畫與含 Archived: yes 的 commit 訊息，並再次經確認工具確認後才執行暫存與提交

#### Scenario: 確認工具不可用時的降級

- **WHEN** 執行環境沒有 AskUserQuestion 工具
- **THEN** 技能檔 SHALL 規定以純文字提出相同的確認問題並等待使用者回覆，且同樣 SHALL 先輸出計畫與訊息後才提問
