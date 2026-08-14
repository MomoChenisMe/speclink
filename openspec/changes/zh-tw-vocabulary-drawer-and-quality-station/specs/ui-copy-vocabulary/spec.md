## Purpose

使用者可見繁中文案的正典詞彙約束：界定哪些檔案屬於「使用者可見文案面」，要求該面不出現 `openspec/LANGUAGE.md` 列為 avoid 的詞，並以自動化守門測試釘死。邊界在於此 capability 只管**使用者看得到的字**——規格散文、程式碼註解、測試名稱與識別符不在其範圍內。

## ADDED Requirements

### Requirement: 使用者可見文案面的範圍與詞彙約束

「使用者可見文案面」SHALL 涵蓋下列檔案集合：兩個前端 app 的 i18n 訊息檔（`apps/desktop/src/i18n/messages.ts`、`apps/server-web/src/i18n/messages.ts`）、`crates/speclink-core/assets/skills/` 底下全部 `.md` 技能資產、`docs/` 底下全部 `.md`、以及 `README.md` 與 `README.en.md`。

此面內的繁體中文文案 SHALL NOT 出現 `openspec/LANGUAGE.md` 各詞條 `avoid` 欄列出的詞。約束標的 SHALL 限於中日韓文字構成的詞彙字串；ASCII 識別符（型別名、函式名、變數名、CSS 類名、i18n 訊息鍵名）SHALL NOT 因此受限，亦 SHALL NOT 因本約束而更名。

英文文案 SHALL NOT 受此約束——`openspec/LANGUAGE.md` 的適用範圍本即排除英文 CLI 輸出與英文介面字串。

#### Scenario: 繁中文案使用正典詞

- **WHEN** 讀取 `apps/desktop/src/i18n/messages.ts` 的 `zh-TW` 區塊
- **THEN** 品質關卡相關文案以「品質關卡」稱之，且全檔不含「品質站」

#### Scenario: 同鍵的英文文案不受約束

- **WHEN** 讀取同一檔案 `en` 區塊中與 `zh-TW` 相同鍵名的訊息
- **THEN** 英文字面維持不變，SHALL NOT 因繁中詞彙收斂而改動

#### Scenario: ASCII 識別符不受詞彙約束影響

- **WHEN** 檢視使用者可見文案面所在檔案及其相依模組的識別符
- **THEN** `RichDetailDrawer`、`SpecDrawer`、`ArchivedDrawer`、`archivedDrawerBase` 等含 `Drawer` 的識別符維持原名

##### Example: 約束標的判定

| 字串 | 是否受約束 | 理由 |
| --- | --- | --- |
| `"此工作區不支援品質站工單處置"` | 是 | 面內繁中文案且含 avoid 詞 |
| `"this workspace cannot settle quality-station tickets"` | 否 | 英文文案不在適用範圍 |
| `RichDetailDrawer` | 否 | ASCII 識別符，非繁中詞彙 |
| `"tour.navUsers.hint"` | 否 | i18n 鍵名為 ASCII 識別符 |

### Requirement: 詞彙守門測試釘死使用者可見文案面

專案 SHALL 提供詞彙守門測試，掃描使用者可見文案面的全部檔案，比對 `openspec/LANGUAGE.md` 各詞條 `avoid` 欄的繁中詞。守門測試 SHALL 掛入既有的 `node --test "scripts/**/*.test.mjs"` 套件，並隨 `npm run test:all` 執行。

面內任一檔案出現 avoid 詞時，守門測試 SHALL 失敗並以非零 exit code 結束，訊息 SHALL 載明違規的檔案路徑、行號與命中的詞。面內無 avoid 詞時 SHALL 通過。

守門測試讀檔 SHALL 以 UTF-8 解碼，比對 SHALL NOT 依賴行尾字元形式，於 Windows、macOS 與 Linux SHALL 得到相同結果。

#### Scenario: 面內乾淨時通過

- **WHEN** 使用者可見文案面不含任何 avoid 詞，執行 `node --test "scripts/**/*.test.mjs"`
- **THEN** 守門測試通過，exit code 為 0

#### Scenario: 面內出現 avoid 詞時響亮失敗

- **WHEN** 於 `apps/server-web/src/i18n/messages.ts` 的繁中文案植入「抽屜」後執行守門測試
- **THEN** 測試失敗且 exit code 非 0，訊息含該檔案路徑、植入所在行號與命中的詞「抽屜」

#### Scenario: CRLF 行尾不影響判定

- **WHEN** 面內檔案以 CRLF 行尾儲存且不含 avoid 詞
- **THEN** 守門測試通過，判定結果與 LF 行尾時相同

### Requirement: 兩個收斂詞以 LANGUAGE.md 詞條承載，且正典本身排除於守門之外

`openspec/LANGUAGE.md` 的「詞彙」段 SHALL 含「詳情面板」與「品質關卡」兩個詞條，各含 definition、avoid 與 why，並於 why 載明裁定日期。「詳情面板」詞條的 avoid SHALL 列「抽屜」；「品質關卡」詞條的 avoid SHALL 列「品質站」。

`openspec/LANGUAGE.md` 本身 SHALL NOT 納入守門測試的掃描範圍——詞條的 avoid 欄依設計即需寫出舊詞，納入掃描將使正典自身違規。

`openspec/specs/` 的規格散文、程式碼註解、測試名稱與 `openspec/changes/archive/` 的封存內容 SHALL NOT 納入守門測試的掃描範圍；此三者的既有舊詞依 `openspec/LANGUAGE.md`「舊文案陸續汰換、歷史 artifacts 不回改」原則處理。

#### Scenario: 詞條齊備且列出舊詞為 avoid

- **WHEN** 讀取 `openspec/LANGUAGE.md` 的「詞彙」段
- **THEN** 存在「詳情面板」與「品質關卡」兩個詞條，前者 avoid 含「抽屜」、後者 avoid 含「品質站」，兩者 why 皆載明裁定日期

#### Scenario: 正典自身不被守門判為違規

- **WHEN** `openspec/LANGUAGE.md` 的 avoid 欄寫有「抽屜」與「品質站」，執行守門測試
- **THEN** 守門測試通過——該檔不在掃描範圍內

#### Scenario: 規格散文的存量舊詞不觸發守門

- **WHEN** `openspec/specs/desktop-app/spec.md` 散文仍使用「抽屜」，執行守門測試
- **THEN** 守門測試通過——規格散文不在掃描範圍內
