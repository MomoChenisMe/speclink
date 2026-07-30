## ADDED Requirements

### Requirement: 開發者入口文件雙語對

docs/development.md（英文）與 docs/development.zh-TW.md（繁體中文）SHALL 存在且章節結構與事實內容對等（僅語言不同），並受既有「中英文文件保持結構與事實對等」與「文件準確性具可重複驗證清單」需求約束。內容 SHALL 涵蓋：npm run dev、npm run dev:server、npm run dev:desktop、npm run dev:reset、npm run cli 五個入口各自的用途、前置條件與預期可觀察結果；下載安裝檔的未簽章放行步驟（macOS 系統設定放行、Windows SmartScreen 仍要執行），放行一節的產物名稱 SHALL 與 desktop-release 規格一致。文件中出現的指令 SHALL 全部為已驗證可執行的入口。README.md 與 README.en.md SHALL 各含一處指向對應語言 development 文件的連結。

#### Scenario: 一鍵入口涵蓋完整

- **WHEN** 檢視 development 文件任一語言版
- **THEN** 五個 npm 入口各有一節，敘明用途、前置條件與預期可觀察結果（如 dev:server 會印出 /setup 連結、dev:desktop 會開啟本地模式視窗）

#### Scenario: 雙語結構對等

- **WHEN** 對照 docs/development.md 與 docs/development.zh-TW.md 的章節骨架
- **THEN** 兩檔章節一一對應、事實一致，僅語言不同

#### Scenario: README 導流

- **WHEN** 檢視 README.md 與 README.en.md
- **THEN** README.md 含指向 docs/development.zh-TW.md 的連結、README.en.md 含指向 docs/development.md 的連結

#### Scenario: 未簽章放行教學可依循

- **WHEN** 使用者依放行一節在 macOS 開啟未簽章 dmg 安裝的 app、或在 Windows 執行未簽章安裝器
- **THEN** 文件步驟足以完成放行（macOS：系統設定 > 隱私權與安全性的強制打開；Windows：SmartScreen 其他資訊 > 仍要執行），且步驟中的產物名稱與實際 Release assets 相符
