## MODIFIED Requirements

### Requirement: 規格面 drift 端點且工作區面不進 wire

server SHALL 提供 change-scoped 的 drift 端點：對單一 store snapshot 回報 (a) 以引擎規格面計算的 spec drift（規格面維度與規格假設）、(b) 該 snapshot 的 basis digests、(c) 該 change 的 store 面輸入（created metadata、design/tasks 內容，及該 change 的 evidence 記錄文字——有記錄才出現，缺席即缺席）——client 的工作區面計算讀這些，缺席與空內容 SHALL 可區別。以上三者 SHALL 出自同一個 snapshot。回應 SHALL 為 protocol 的 drift DTO（camelCase、可匯出 JSON Schema）。

wire DTO SHALL NOT 含任何工作區面（code/git）欄位——broken anchors、工作區維度與 git 事實皆屬 client 本機，server SHALL NOT 執行任何 git 操作。（(c) 是 store 事實而非工作區事實：server 由 snapshot 即知，下行供 client 自算其半，不構成工作區面上行；evidence 記錄亦然——它是 store 保存的歷史事實，client 的 Environment 維度據此取得該 change 的 touched 檔案清單。）

端點沿用 bearer 前置與 binding 裁決；未知 change SHALL 回 404 not_found；計算 SHALL 為唯讀，SHALL NOT 產生事件或取寫入鎖。

#### Scenario: 規格面報告、basis 與 store 面輸入

- **WHEN** 對含 delta specs 與 design 的 change 請求 drift 端點
- **THEN** 回應含規格面維度、規格假設、該 snapshot 的 basis digests，以及 created 與 design/tasks 內容；回應結構無任何工作區/git 欄位（含 broken anchors）

#### Scenario: 缺席的 artifact 不偽裝成空內容

- **WHEN** 對無 design.md 的 change 請求 drift 端點
- **THEN** 回應如實標示 design 缺席，而非回空字串——client 的 Structure 維度據此區別「無 design」與「design 為空」

#### Scenario: 未知 change 拒絕

- **WHEN** 以不存在的 change 名請求 drift 端點
- **THEN** 回 404 且 reason 為 not_found

#### Scenario: store 保存的 evidence 隨回應下行

- **WHEN** 對曾以 task done 落過 evidence 的 change 請求 drift 端點
- **THEN** 回應的 store 面輸入含該 change 的 evidence 記錄文字，與 store 保存的內容一致；對照組：從未落過 evidence 的 change，該欄位缺席而非空字串
