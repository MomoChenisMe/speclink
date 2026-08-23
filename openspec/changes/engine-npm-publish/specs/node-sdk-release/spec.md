## Purpose

`@speclink/engine` 的 npm 發布通路：五平台 build/pack workflow 的可重用邊界、release tag 版號蓋章與平台子套件物化，以及 npm 發布的閘門與順序。本 capability 保證發布產物版本一律與 release tag 對齊——repo 內版號只是佔位符，不會外流成發布版本。

## ADDED Requirements

### Requirement: engine npm 套件家族與版號蓋章

專案 SHALL 提供 `@speclink/engine` 的 npm 通路：主套件（index.js、index.d.ts、binding.js）與五個平台子套件（Windows x64、macOS x64／arm64、Linux glibc x64／arm64），主套件以 optionalDependencies 指向各平台子套件，安裝時只下載符合平台者。發布產物的版本 SHALL 與 release tag 對齊，由發布管線於打包前蓋章寫入主套件與全部平台子套件——repo 內 `crates/speclink-node/package.json` 的版號 SHALL 視為佔位符，SHALL NOT 作為發布版號來源。蓋章 SHALL 同時把主套件 optionalDependencies 物化為各平台子套件的同版釘定，且平台子套件清單 SHALL 自平台子套件目錄實際內容列舉而非硬編碼。

#### Scenario: 蓋章物化主套件與平台子套件

- **WHEN** 對含五個平台子套件目錄的套件樹執行版號蓋章（如 0.2.0）
- **THEN** 主套件與每個平台子套件的 version 皆為 0.2.0，且主套件 optionalDependencies 恰為五個平台子套件名各釘 0.2.0

#### Scenario: 佔位版號不外流

- **WHEN** 以 0.2.0 蓋章打包後檢視主套件 tarball 內的 package.json
- **THEN** version 為 0.2.0，而非 repo 內佔位的 0.1.0

### Requirement: build/pack workflow 可被 release 管線重用

engine 的五平台建置與打包 workflow SHALL 同時支援 push／PR 觸發與 `workflow_call` 呼叫：`workflow_call` SHALL 接受選填的版號輸入，有值時於打包前執行版號蓋章、無值時維持佔位版號打包（push／PR 情境的既有行為 SHALL 不變）。release 管線 SHALL 經 `workflow_call` 於同一 workflow run 內重用此建置，SHALL NOT 另行複製五平台 build matrix。

#### Scenario: PR 觸發不受版號蓋章影響

- **WHEN** pull request 觸發 build/pack workflow（無 version 輸入）
- **THEN** 五平台 tarball 以 repo 內佔位版號（如 0.1.0）打包上傳，流程與導入 workflow_call 前一致

#### Scenario: release 帶版號呼叫得到蓋章 tarball

- **WHEN** release 管線因 tag v0.2.0 觸發，以 version 輸入 0.2.0 呼叫 build/pack workflow
- **THEN** 產出的主套件與五個平台子套件 tarball 版本皆為 0.2.0，並以 npm-tarballs artifact 供同 run 的後續 job 下載

### Requirement: npm 發布閘門與發布順序

engine 的 npm 發布 SHALL 由 release 管線在 NPM_TOKEN 存在時執行（`npm publish --access public`），缺席時 SHALL 跳過發布步驟且 job 結果為成功、SHALL NOT 影響 Release 其餘產物；token 存在而發布失敗時 SHALL 以紅燈呈現且可單獨重跑。發布順序 SHALL 為平台子套件先發、主套件最後發，使主套件的 optionalDependencies 於發布完成時皆可解析到同版。發布單位 SHALL 為 build/pack workflow 產出的 tarball 檔，發布 job SHALL NOT 重新打包。

#### Scenario: 無 token 時 Release 不受影響

- **WHEN** 在未設定 NPM_TOKEN 的 repo 推送 release tag（如 v0.2.0）
- **THEN** engine 的 npm 發布各步跳過、job 結果為成功，GitHub Release 與 Docker 等其餘產物照常發布

#### Scenario: 主套件最後發布

- **WHEN** NPM_TOKEN 存在且五個平台子套件（如 @speclink/engine-darwin-arm64）的 tarball 皆發布成功
- **THEN** 主套件 @speclink/engine 的 tarball 最後發布，其 optionalDependencies 指向的每個平台子套件皆已存在於 registry 且與主套件同版（如 0.2.0）
