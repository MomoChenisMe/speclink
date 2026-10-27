## Why

透過 AI 代理執行 SDD、並在本機驗證 Remote workflow 的開發者、PO 與 PM，目前以 npm run dev 啟動 Server／Desktop 時，仍可能因未安裝 CLI 或 PATH 指向其他版本，而用錯 CLI 驗證目前 checkout。開發 harness 需要保證三個入口都來自同一份原始碼，讓 remote 測試結果可重現。

## What Changes

- npm run dev SHALL 先建置目前 checkout 的 speclink-cli；只有建置成功後才啟動既有的 Server 與 Desktop 長時間程序。
- CLI 建置失敗時，npm run dev SHALL 以非零 exit code 結束，且 SHALL NOT 啟動任何 Server／Desktop 長時間程序。
- 新增根目錄 npm run cli -- <args> 測試入口，直接執行目前 checkout 的 debug CLI binary，不依賴 PATH 中已安裝的 speclink。
- wrapper SHALL 跨 Windows、macOS 與 Linux 解析 binary，並透明轉送 argv、stdin、stdout、stderr、呼叫端工作目錄與 CLI exit code。
- 補齊 dev harness 自動化測試與 Remote 操作文件，使「啟動環境」和「以同版 CLI 驗證」成為一條可重現流程。

## Non-Goals

- 不執行 cargo install、不覆寫或移除使用者已安裝的 speclink，也不修改 PATH 或 shell profile。
- 不把 CLI 當成 npm run dev 的第三個長時間 child process；dev lifecycle 仍只管理 Server 與 Desktop。
- 不修改 speclink CLI 的子指令、旗標、stdin 語意、人眼輸出、--json shape 或既有 exit code。
- 不改變 Server／Desktop runtime、Remote Protocol、認證、membership 或 workspace 行為。
- 不建立 release 安裝、版本管理器或全域 CLI 切換機制。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `dev-harness`: 將一鍵啟動契約擴充為先完成目前 checkout 的 CLI build gate，並提供固定執行該 binary 的 npm CLI 測試入口。

## Impact

- 可能修改：`package.json`、`scripts/dev.mjs`、`scripts/dev.test.mjs`。
- 可能新增：`scripts/cli.mjs`、`scripts/cli.test.mjs`。
- 文件：`docs/remote-getting-started.zh-TW.md`、`docs/remote-getting-started.md`，以及必要的開發啟動架構說明。
- Crate 影響：`speclink-cli` 只會由 harness 建置與呼叫，CLI 實作與契約不變；`speclink-core` 不變。
- API／設定／技能：不新增或變更 `.speclink.yaml`、`openspec/config.yaml`、CLAUDE.md、AGENTS.md、Speclink skills 或 protocol API。
- CLI 介面：新增的是 package-level npm wrapper，不是 speclink 子指令；`<args>`、stdin 與 CLI exit code 原樣轉送。
- 相容性影響：npm run dev 會多一道 CLI 建置前置條件，首次啟動時間可能增加，建置失敗會提早中止。既有 CLI 人眼輸出、--json 輸出與 Spectra 回歸對照不變，既有使用者不需遷移，已安裝 CLI 也不受影響。
