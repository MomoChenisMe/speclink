## MODIFIED Requirements

### Requirement: 使用者可見文案面的範圍與詞彙約束

「使用者可見文案面」SHALL 涵蓋下列檔案集合:兩個前端 app 的 i18n 訊息檔(`apps/desktop/src/i18n/messages.ts`、`apps/server-web/src/i18n/messages.ts`)、共用看板元件的 i18n 訊息檔(`packages/ui/src/i18n.tsx`)、`crates/engine/speclink-core/assets/skills/` 底下全部 `.md` 技能資產、`docs/` 底下全部 `.md`、以及 `README.md`（英文）與 `README.zh-TW.md`（繁中）。

此面內的繁體中文文案 SHALL NOT 出現 `openspec/LANGUAGE.md` 各詞條 `avoid` 欄列出的詞。約束標的 SHALL 限於中日韓文字構成的詞彙字串;ASCII 識別符(型別名、函式名、變數名、CSS 類名、i18n 訊息鍵名)SHALL NOT 因此受限,亦 SHALL NOT 因本約束而更名。

`avoid` 欄帶括號語境限定的條目 SHALL 依限定語境分流:限定為「使用者可見文案中」者,其限定範圍即守門面本身,SHALL 剝掉限定後納入約束;其他語境限定(如「此概念上」「分頁名中」「pagination 語意上」「中文散文中」)綁在比守門面更窄的語境,機械比對必然誤命中,SHALL NOT 納入機械守門——該類條目仍為正典的一部分,由撰稿時人工判斷。

英文文案 SHALL NOT 受此約束——`openspec/LANGUAGE.md` 的適用範圍本即排除英文 CLI 輸出與英文介面字串。

#### Scenario: 繁中文案使用正典詞

- **WHEN** 讀取 `apps/desktop/src/i18n/messages.ts` 的 `zh-TW` 區塊
- **THEN** 品質關卡相關文案以「品質關卡」稱之,且全檔不含「品質站」

#### Scenario: 共用看板元件的繁中文案受約束

- **WHEN** 讀取 `packages/ui/src/i18n.tsx` 的 `zh-TW` 區塊
- **THEN** 待手動相關文案以「待手動」稱之,且全檔繁中文案不含「待手測」

#### Scenario: 同鍵的英文文案不受約束

- **WHEN** 讀取同一檔案 `en` 區塊中與 `zh-TW` 相同鍵名的訊息
- **THEN** 英文字面維持不變,SHALL NOT 因繁中詞彙收斂而改動

#### Scenario: ASCII 識別符不受詞彙約束影響

- **WHEN** 檢視使用者可見文案面所在檔案及其相依模組的識別符
- **THEN** `RichDetailDrawer`、`SpecDrawer`、`ArchivedDrawer`、`archivedDrawerBase` 等含 `Drawer` 的識別符維持原名

##### Example: 約束標的判定

| 字串 | 是否受約束 | 理由 |
| --- | --- | --- |
| `"此工作區不支援品質站工單處置"` | 是 | 面內繁中文案且含 avoid 詞 |
| `"this workspace cannot settle quality-station tickets"` | 否 | 英文文案不在適用範圍 |
| `RichDetailDrawer` | 否 | ASCII 識別符,非繁中詞彙 |
| `"tour.navUsers.hint"` | 否 | i18n 鍵名為 ASCII 識別符 |
| `"待手測"`(於 packages/ui/src/i18n.tsx) | 是 | 面內繁中文案且含 avoid 詞 |

#### Scenario: 語境限定的 avoid 條目依限定分流

- **WHEN** 解析 `openspec/LANGUAGE.md` 各詞條 `avoid` 欄組成守門詞集
- **THEN** 限定為「使用者可見文案中」的條目(如「覆審(使用者可見文案中)」)剝掉限定後入集,其他語境限定的條目(如「背景(此概念上)」「分頁(pagination 語意上)」)不入集

#### Scenario: 繁中 README 在守門面內

- **WHEN** 詞彙守門測試組成使用者可見文案面的檔案清單
- **THEN** 清單含 `README.md` 與 `README.zh-TW.md`，不含已不存在的 `README.en.md`；`README.zh-TW.md` 的繁中散文出現 avoid 詞時測試失敗並點名該檔
