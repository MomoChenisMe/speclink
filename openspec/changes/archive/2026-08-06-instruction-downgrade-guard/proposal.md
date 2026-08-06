## Why

2026-08-05 實際事故：repo 檔案已再生為 v1.14.0，但安裝中的 desktop app 引擎停在 v1.11.0，橫幅把這個狀態標成「舊版」並提供「更新」主動作，按下後 30 個受管檔被靜默改寫回 v1.11 內容。根因有二：(1) 過期判準只有「不等於」（正典明文「SHALL NOT 解析版本語意」），分不出檔案比引擎新還是舊，降級被包裝成「更新」；(2) 本機安裝的新鮮度靠代理紀律與記憶，無任何查詢面可驗證「這顆 binary 的引擎是哪版」，過期的 app 就這樣裝了進來。目標使用者是在本 repo 以 AI 代理開發、透過 desktop 或 CLI 維護多專案指令檔的開發者（討論 instruction-downgrade-guard 定案）。

## What Changes

- **過期探測加方向感**（speclink-core）：探測狀態由四態擴為五態——新增「較新」（任一工具檔案的標記版號比引擎新）；方向以版號拆段數值比較（vX.Y.Z），任一版號無法解析時該工具退回現行字串相等判定（不等即過期），不硬排序；聚合優先序改為 較新 > 缺失 > 過期（任一工具較新即整體較新，寧可不提供破壞性動作）。
- **desktop 較新提示**（apps/desktop）：探測回報較新時，提示改示「你的 app 是舊版」語意、引導使用者更新 app 本體，不提供任何改寫檔案的動作；「保留現狀」與略過記憶照既有機制沿用（同鍵值、同版不再提示）。
- **受管檔再生的降級守門**（speclink-core，CLI 與 desktop 共用）：守門落在引擎的再生入口，判定目標取自該次的實際寫入集（tools 選集、無清單時的目錄偵測、自訂描述子），領先即拒絕——單行說明含工作區版號與引擎版號、非零 exit code、零寫入。凡經該入口的路徑一體適用：update、init --force 的重建、工具選集收斂、workflow-config 的技能足跡同步（CLI 與桌面設定頁）、桌面的更新動作。新增旗標 --allow-downgrade 明示越過（不共用 --force，避免慣性帶旗標把守門靜默穿透；同理 --force 的重建也不算同意降級）。缺失、過期、現版的行為不變。
- **引擎版號查詢面**（speclink-cli）：--version 輸出加印產物層版號，格式如 0.1.0 (arm64, engine v1.14.0)——任何 binary 的新舊變成一條指令可斷言。
- **本機安裝腳本**（scripts/desktop-install.mjs，新增）：把「sidecar 佈署 → 前端建置 → tauri bundle → 斷言 bundle 內 CLI 引擎版號等於源碼 MARKER_VERSION → 安裝後再斷言安裝版同版」收成單一入口；任一斷言失敗以非零結束並指出版號差。LLM 安裝只跑這個 script，新鮮度由建構保證、兩道斷言證明。

相容性影響：--version 人眼輸出改變（刻意，附 engine 版號；若有 CLI 測試釘住舊格式同批更新）；探測 JSON 的 status 欄位新增 "newer" 值（消費端僅 desktop，前端與引擎同 bundle 出貨、無版本錯開）；update 僅在「較新」情境從靜默改寫變為拒絕，其餘情境行為不變；不涉及技能內文與 marker 模板 render，MARKER_VERSION 不推進、golden 零變動。

## Non-Goals

- 不動 desktop 自動更新（updater）流程本身；較新提示以文案引導，不內建「更新 app」動作
- 不做降級的自動備份或回復機制（守門的目的是讓降級不再靜默發生）
- 不改缺失、過期、現版的既有語意與略過記憶機制
- 不保證「源碼樹等於 origin 最新」——安裝腳本印出 HEAD 與引擎版號供操作者確認，保證的是「安裝版等於這棵樹」
- CLI 訊息不新增本地化（維持現行英文輸出）
- 不處理 .evidence.json 任務歸屬等本次事故無關的已知項

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-tools`: 「指令檔過期探測」由四態改五態並加方向判定與聚合優先序；新增「受管檔再生的降級守門」與「引擎版號查詢面」兩條需求。
- `desktop-app`: 「指令檔過期提示」新增較新形態——不提供改寫檔案動作、文案引導更新 app。
- `desktop-release`: 新增「本機安裝的新鮮度斷言」需求——安裝腳本與其兩道版號斷言。

## Impact

- Affected specs: workspace-tools、desktop-app、desktop-release
- Affected code:
  - Modified: crates/speclink-core/src/init.rs、crates/speclink-cli/src/main.rs、apps/desktop/core/src/project.rs、apps/desktop/src/instructionPrompt.ts、apps/desktop/src/components/InstructionUpdatePrompt.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx、apps/desktop/src/adapter/workspace.ts
  - New: scripts/desktop-install.mjs、crates/speclink-cli/tests/update_downgrade_guard.rs
  - Removed: (none)
