## Purpose

/speclink-ingest 技能的收尾行為：artifacts 更新並通過驗證後，重判本變更相對其他作用中變更的軟依賴並以 change depends 落檔，讓中途改需求產生的新前置進入 plan 的守門。邊界：只涵蓋 ingest 收尾的依賴判定；ingest 的來源解析、artifacts 更新與 seal 仍由技能本文承載，propose 側的同一判定屬 propose-skill。

## ADDED Requirements

### Requirement: ingest 收尾重判本變更的軟依賴

技能檔 SHALL 規定：ingest 的 artifacts 更新完成並通過 validate 後、給出下一步建議之前，代理人 SHALL 以 list 動詞的 JSON 輸出列出作用中變更名；作用中變更只有本次更新者時 SHALL 跳過本步驟、SHALL NOT 執行 change depends。有兩個以上時 SHALL 只針對本次更新的變更判定軟依賴——以更新後的 artifacts 為準，讀其他變更的 proposal Impact，判定本變更是否建立在某變更的成果上；有則 SHALL 對每個前置各執行一次 `speclink change depends <本變更> --on <前置>` 落檔（動詞對同一次呼叫的多個前置全寫或全不寫，逐一呼叫才不會因一個被拒而連帶遺失其他前置），SHALL NOT 只口頭報告；邊已存在時由動詞冪等處理，動詞拒絕（自依賴、未知或已封存名稱、成環）時 SHALL 回報拒絕訊息並續行下一個前置，SHALL NOT 重試。僅動到同一段程式碼 SHALL NOT 記為前置——本變更若已在進行中，等待尚未開工的變更會卡住它——代理人 SHALL 改為向使用者提出該重疊。落檔段落（每個前置各一次呼叫、拒絕時的處置、硬信號交引擎）SHALL 與 propose 技能收尾的同一段逐字一致。硬信號（delta capability 重疊）SHALL 由引擎計算，代理人 SHALL NOT 自行判定。本步驟 SHALL NOT 移除任何既有 depends_on（刪邊是使用者的決定），SHALL NOT 自動呼叫 apply；出邊維持回 apply，由 apply 第 1 步的 plan 守門讀取新宣告。

#### Scenario: 技能檔含重判軟依賴指示

- **WHEN** 技能再生後讀取 speclink-ingest 的 SKILL.md
- **THEN** 技能檔含「更新完成後讀其他作用中變更的 proposal Impact 判定本變更是否建立在其成果上、有則以 speclink change depends 逐一落檔」的指示，且該指示位於 validate 步驟之後、下一步建議之前

#### Scenario: 單一作用中變更時跳過

- **WHEN** ingest 更新的變更是唯一的作用中變更
- **THEN** 技能檔指示跳過重判、不執行 change depends，維持既有下一步建議

#### Scenario: 動詞拒絕時續行

- **WHEN** 代理人判定的前置已封存或會形成依賴環，change depends 以非零 exit code 拒絕
- **THEN** 技能檔指示回報拒絕訊息並續行下一個前置（其餘前置照常各自落檔），不重試、不改寫他方變更的 depends_on

#### Scenario: 僅動到同一段程式碼時不記為前置

- **WHEN** 代理人判定本變更只與另一作用中變更動到同一段程式碼，並未建立在其成果上
- **THEN** 技能檔指示不執行 change depends，改向使用者提出該重疊

##### Example: 中途改需求後新增前置

| 作用中變更 | 本次 ingest 的變更 | 更新後的判定 | 落檔指令 | 後續 |
| --- | --- | --- | --- | --- |
| add-a、add-b | add-b | 新需求要用 add-a 新增的動詞 | speclink change depends add-b --on add-a | 下一次 /speclink-apply add-b 由 plan 守門擋到 add-a 封存 |
| add-a、add-b | add-b | 新需求與 add-a 無關 | （不執行） | 出邊回 apply，順序不變 |
| add-a、add-b | add-b | 新需求只和 add-a 動到同一段程式碼 | （不執行） | 向使用者提出與 add-a 的重疊，出邊回 apply |
| add-a、add-b、add-c | add-b | 新需求要用 add-a 與 add-c 的成果，但 add-c 已依賴 add-b | speclink change depends add-b --on add-a；speclink change depends add-b --on add-c（成環被拒） | add-a 照常落檔；回報 add-c 的拒絕訊息，不重試 |
| add-b | add-b | （唯一作用中變更） | （跳過本步驟） | 出邊回 apply |
