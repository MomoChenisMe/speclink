## ADDED Requirements

### Requirement: 手動任務與待手動以 LANGUAGE.md 詞條承載

`openspec/LANGUAGE.md` 的「詞彙」段 SHALL 含「手動任務」與「待手動」兩個詞條,各含 definition、avoid 與 why,並於 why 載明裁定日期;名為「手動測試」與「待手測」的詞條 SHALL NOT 再存在。「手動任務」詞條的 definition SHALL 載明 `[M]` 標記語意為「agent 無法代行、需使用者親手操作的任務,不限於測試」,其 avoid SHALL 含「手動測試(此概念上)」與原「手動測試」詞條承接的存量 avoid 詞——「此概念上」的語境限定使指真測試的正當用法(如 docs 內「每日手動測試」)不被機械守門誤傷。「待手動」詞條的 avoid SHALL 含「待手測」且不帶語境限定——自造詞無其他正當用法,納入機械守門。

#### Scenario: 詞條改名完成

- **WHEN** 讀取 `openspec/LANGUAGE.md` 的「詞彙」段
- **THEN** 存在「手動任務」與「待手動」兩個詞條且 why 皆載明裁定日期;不存在名為「手動測試」或「待手測」的詞條;前者 avoid 含「手動測試(此概念上)」,後者 avoid 含「待手測」

#### Scenario: 指真測試的正當用法不受機械守門誤傷

- **WHEN** `docs/` 內某檔含「每日手動測試」一類指真測試的敘述,執行守門測試
- **THEN** 守門測試通過——「手動測試(此概念上)」屬其他語境限定條目,不入機械守門詞集

#### Scenario: 待手測納入機械守門

- **WHEN** 於使用者可見文案面某檔的繁中文案植入「待手測」後執行守門測試
- **THEN** 測試失敗且 exit code 非 0,訊息含該檔案路徑、行號與命中的詞「待手測」

## MODIFIED Requirements

### Requirement: 使用者可見文案面的範圍與詞彙約束

「使用者可見文案面」SHALL 涵蓋下列檔案集合:兩個前端 app 的 i18n 訊息檔(`apps/desktop/src/i18n/messages.ts`、`apps/server-web/src/i18n/messages.ts`)、共用看板元件的 i18n 訊息檔(`packages/ui/src/i18n.tsx`)、`crates/speclink-core/assets/skills/` 底下全部 `.md` 技能資產、`docs/` 底下全部 `.md`、以及 `README.md` 與 `README.en.md`。

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
