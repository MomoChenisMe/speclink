# 離線來源核對

2026-10-20 核對 OWASP 官方 [CheatSheetSeries 固定 commit](https://github.com/OWASP/CheatSheetSeries/tree/c366393214da5ae186a7a06a27dade7617987304)。

- 採用 18 份原文：Authentication、Authorization、Cryptographic Storage、Logging、Transport Layer Security、REST Security、SQL Injection、XSS、OS Command Injection、Deserialization、XXE、SSRF、Prototype Pollution、Webhook、DoS、DotNet、Java、Nodejs；逐檔成功讀取並計算 SHA-256，值記於 spec_registry.json。
- 授權檔實際為 LICENSE.md，保留 CC-BY-SA-4.0 與作者歸屬；包內使用 raw/CheatSheetSeries_LICENSE.md 避免覆蓋標準本身的 LICENSE。測試 ZIP 保留這份原始授權與未修改原文。
- 標準身分、版本與原解析器繼續引用 sources.json，不另維護第二份標準清單；CWE 對照由已選標準的官方控制項產生，guidance 不增加檢核標準。
- 不採用附件中的 installed／not_installed 作本機狀態，不採用 master／LTS 等浮動版本。不假設 Python_Security／Rust_Security 檔名存在；Python 與 Rust 的修正依據使用共同主題指南與程式證據。
- 以上是發行資料來源與完整性核對，並非官方對產品的安全認證。執行審查只讀已安裝內容，不使用這些連結抓資料。
