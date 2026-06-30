# speclink

@Spectra-OpenSpec-SDD-完整功能邏輯分析.md
這份是關於spectra+OpenSpec的比較分析，我想要基於這兩個製作一個屬於自己的SDD規格驅動引擎，我想保留某些spectra的特性和某些openspec的特性並且依據
這些特性再延伸更進階的設計：
1. 實作語言Rust
2. 工作流想選擇spectra的模式discuss? → propose → apply ⇄ ingest → archive
3. 目前spectra的discuss比較鬆散，我想要有讓使用discuss時可以有一個延續性的感覺，目前discuss不會留下任何的文件，導致有時候討論一個需求時，討論到後面會越來越偏離主題，所以我希望discuss時是可以文件記錄這些迭代討論的過程和演進，但本質一樣是要和discuss一樣的步驟邏輯
4. 保留完整的config.yaml和.spectra.yaml的部分功能：locale、tdd、audit
5. 不需要的功能有：spectra的debug、ask、向量搜尋、worktree、park/unpark、parallel_tasks、claude_effort
6. 第一階段一樣先實作出完整的cli所有指令，init時可以初始化專案（包含產出技能、資料夾、config.yaml、.spectra.yaml等，基本跟spectra完全一樣），然後可以用技能完整測試SDD的流程
7. 唯一稍微不同的流程是discuss，但其它的流程包括技能內容、功能結構、流程邏輯、cli引擎邏輯、CLI指令輸出結果，跑起來全部都要跟spectra有相同的結果



以上是跟Openspec與spectra的基本設計需求
接下來要說我要延伸的功能：
目前不管是Openspec或spectra，規格文件都是跟隨著git儲存庫，雖然openspec有store的概念，但他的store比較像是把規格抽離出來，我希望的是提供一套規格
驅動引擎的概念，文件怎麼存放、管理我不管，想讓使用者自己決定怎麼存放這些文件（你要寫成md文件、要儲存在資料庫、要存成json、要存成yaml都可以、串
接個人的系統、JIRA等都可以），我只要提供一個規格驅動引擎的概念，甚至希望可以達到：由PO/PM在客製化系統中執行discuss + propose + ingest +
archive，再交由RD/QA在本地git儲存庫中執行apply和verify