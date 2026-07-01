# speclink

@Spectra-OpenSpec-SDD-完整功能邏輯分析.md
這份是關於spectra+OpenSpec的比較分析，我想要基於這兩個製作一個屬於自己的SDD規格驅動引擎，我想保留某些spectra的特性和某些openspec的特性並且依據
這些特性再延伸更進階的設計：
1. 實作語言Rust
2. 工作流想選擇spectra的模式discuss? → propose → apply ⇄ ingest → archive
3. 目前spectra的discuss比較鬆散，我想要有讓使用discuss時可以有一個延續性的感覺，目前discuss不會留下任何的文件，導致有時候討論一個需求時，討論到後面會越來越偏離主題，所以我希望discuss時是可以文件記錄這些迭代討論的過程和演進，但本質一樣是要和discuss一樣的步驟邏輯
4. propose時，spectra可以選擇從對話、plan文件，再增加一個選擇是從discuss的文件中產生propose
5. 保留完整的config.yaml和.spectra.yaml的部分功能：locale、tdd、audit
6. 不需要的功能有：spectra的debug、ask、向量搜尋、worktree、park/unpark、parallel_tasks、claude_effort
7. 第一階段一樣先實作出完整的cli所有指令，init時可以初始化專案（包含產出技能、資料夾、config.yaml、.spectra.yaml等，基本跟spectra完全一樣），然後可以用技能完整測試SDD的流程
8. 唯一稍微不同的流程是discuss，但其它的流程包括技能內容、功能結構、流程邏輯、cli引擎邏輯、CLI指令輸出結果，跑起來全部都要跟spectra有相同的結果
9. 完成整個speclink的專案，確保cli的功能、流程、輸出結果、技能內容、功能結構、流程邏輯、cli引擎邏輯都有完成並且跟spectra一致
10. 完成speclink cli後，你需要在這個專案中執行init並確保skill和相關產生的文件都有，同時用speclink完整跑過整個SDD流程，目標是用html建立一個彈珠檯遊戲
11. 測試過程中，你需要同時測試spectra和speclink的cli邏輯、輸出是否一致，並且在speclink中加入discuss的文件記錄功能，讓討論過程可以被完整記錄下來，並且可以從這些討論文件中產生propose。
12. init初始化後你還需要填寫config.yaml和.speclink.yaml(.spectra.yaml的speclink版本)，確保測試時有東西可以取得，也要確保cli指令的instructions有正確取得資訊如同spectra一樣
13. 最終請你進行比較spectra和speclink並功能分析結果報告，若分析上有任何不一致的地方就重回到第9點重跑一次


以上是跟Openspec與spectra的基本設計需求
接下來要說我要延伸的功能：
目前不管是Openspec或spectra，規格文件都是跟隨著git儲存庫，雖然openspec有store的概念，但他的store比較像是把規格抽離出來，我希望的是提供一套規格
驅動引擎的概念，文件怎麼存放、管理我不管，想讓使用者自己決定怎麼存放這些文件（你要寫成md文件、要儲存在資料庫、要存成json、要存成yaml都可以、串
接個人的系統、JIRA等都可以），我只要提供一個規格驅動引擎的概念，甚至希望可以達到：由PO/PM在客製化系統中執行discuss + propose + ingest +
archive，再交由RD/QA在本地git儲存庫中執行apply和verify