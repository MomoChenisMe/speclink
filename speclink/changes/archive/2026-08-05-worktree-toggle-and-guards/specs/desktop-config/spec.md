## ADDED Requirements

### Requirement: 產出政策的 worktree 開關

local workspace 的設定頁產出政策區 SHALL 顯示 worktree 開關，文案於 zh-TW 與 en 介面語言下均直出「worktree」一詞（LANGUAGE.md 明文例外）。存檔 SHALL 以開關的畫面實值寫入 openspec/config.yaml 的 worktree 鍵（汰換「UI 無此欄位時恆回填原值」的保值行為），寫入成功後 SHALL 觸發與 CLI workflow-config set 同一技能足跡同步；設定頁載入時 SHALL 反映 config 現值。由開改關存檔且存在活躍 linked worktree 時，SHALL 拒絕寫入並浮出擋下訊息——列出各活躍 worktree 的 change 名、分支與路徑，提示先執行 worktree-merge 收尾——config 維持不變。remote 工作區的設定頁 SHALL NOT 顯示此開關。

#### Scenario: 開關顯示且存檔生效

- **WHEN** local workspace 於設定頁產出政策區將 worktree 開關切為開啟並存檔
- **THEN** openspec/config.yaml 的 worktree 鍵為 true，技能足跡出現兩顆 worktree 技能，設定頁重新載入後開關維持開啟

#### Scenario: 關閉遇活躍 worktree 浮出擋下

- **WHEN** 存在活躍 linked worktree 時於設定頁將 worktree 開關切為關閉並存檔
- **THEN** 存檔失敗浮出，訊息列出各 worktree 的 change 名、分支與路徑及先收尾的指引，openspec/config.yaml 不變，開關回復為開啟

#### Scenario: CLI 先行寫入後設定頁不吃鍵

- **WHEN** 以 CLI 將 worktree 設為 true 後，開啟設定頁再存檔
- **THEN** 設定頁載入時開關即呈現開啟，存檔後 openspec/config.yaml 的 worktree 鍵維持 true

#### Scenario: remote 工作區不顯示開關

- **WHEN** remote 工作區開啟設定頁的產出政策區
- **THEN** 不顯示 worktree 開關，其餘政策欄位照常
