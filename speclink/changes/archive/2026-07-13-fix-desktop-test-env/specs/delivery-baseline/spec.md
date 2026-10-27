## ADDED Requirements

### Requirement: 桌面測試套件於乾淨環境全綠

`apps/desktop` 的 vitest 測試套件 SHALL 在乾淨環境下全數通過。其測試執行環境 SHALL 提供瀏覽器 Web Storage 全域（`localStorage` 與 `sessionStorage`），使依賴本機儲存持久化的測試不因環境缺漏而失敗。`apps/desktop` SHALL 直接於自身 package.json 宣告其宣稱使用的測試 DOM 環境（jsdom）為 devDependency，SHALL NOT 僅依賴其他 workspace 的 hoisting。此需求與「root 單一指令全量驗證」互補：後者要求 test:all 於任一面失敗時中止，本需求確保 `apps/desktop` 這一面在乾淨環境下實際通過，使 test:all 為可信的綠燈 gate。測試執行期間 SHALL NOT 有未處理例外（uncaught exception 或 unhandled rejection）使程序非零退出——含在途非同步作業於測試卸載後才觸發者——使 exit 0 為決定性結果而非偶發綠燈。

#### Scenario: 乾淨環境 desktop 測試全綠

- **WHEN** 在乾淨 checkout（無殘留 node_modules 狀態）執行 npm test -w apps/desktop
- **THEN** 指令以 exit code 0 決定性完成（重複執行不偶發非零）、所有測試通過，且測試執行期間無任何未處理例外（含 Web Storage 未定義、以及在途非同步作業於測試卸載後觸發所致的 uncaught 例外）

#### Scenario: 測試環境提供 Web Storage 全域

- **WHEN** desktop vitest 測試存取 localStorage 或 sessionStorage
- **THEN** 兩者皆為可用的 Storage 物件，setItem／getItem／clear 語意正確，且測試檔之間狀態不殘留

#### Scenario: test:all 貫穿 desktop 步驟不中止

- **WHEN** 執行 npm run test:all 且其餘各面測試均通過
- **THEN** 串接鏈不於 apps/desktop 步驟中止，可續行至 crates/speclink-node 步驟
