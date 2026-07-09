## ADDED Requirements

### Requirement: 討論記錄蓋建立者章

建立討論記錄（discuss new）時，引擎 SHALL 於記錄 frontmatter 蓋建立者 created_by，取自 git 身分（user.name 與 email）；git 身分不可得時 SHALL 省略該欄位。discuss list 與 show 的 --json 輸出 SHALL 以 camelCase createdBy 曝露該值，缺席時省略。既有無 created_by 的討論記錄 SHALL 照常運作，其 createdBy 缺席、不報錯。

#### Scenario: discuss new 於有 git 身分時蓋建立者

- **WHEN** 於設有 git user.name 與 email 的專案執行 discuss new
- **THEN** 產生的討論記錄 frontmatter 含 created_by 為該 git 身分，且 discuss show --json 的 createdBy 為同值

#### Scenario: 無 git 身分時省略建立者

- **WHEN** 於無可解析 git 身分的環境執行 discuss new
- **THEN** 討論記錄不含 created_by，--json 的 createdBy 缺席，記錄仍正常建立

#### Scenario: 既有無建立者記錄照常運作

- **WHEN** list 或 show 一筆 frontmatter 無 created_by 的既有討論
- **THEN** 指令正常輸出、其 createdBy 缺席，不報錯
