目前不管是openspec、spectra或是以spectra為主的speclink，SDD的規格驅動開發（文件、狀態、流程、產物）大多是跟隨著git儲存庫在走，比較偏向個人，而我希望這個speclink除了個人可以使用，維持現有的做法，並讓使用者自行開發GUI之外，也可以將SDD作為團隊用的，但是我不想強制規定文件的存放方式，因為每個團隊的文件管理方式都不一樣，我的理念是：引擎邏輯要保持，甚至引擎可以作為SDK提供使用者自行串接到系統（甚至是AI Agent），而文件的存放方式、管理方式、UI呈現方式則由團隊自行決定，但這樣就要思考init時，寫入claude.md、agents.md以及技能的安裝等等，這部分我沒有太多的想法，請你幫我想看看怎麼做比較好？

我的使用情境比較像是：
1. 由PO/PM在客製化的AI Agent系統中搭配指定的儲存庫來執行discuss + propose + ingest + archive，產出的change儲存的文件也是在這套AI Agent系統中，用類似看板的功能來呈現，而RD則會依據這些Change在本地的git儲存庫搭配Claude Code執行apply、drift、verify等功能
2. 則是：PO/PM/RD/QA都在同一個客製化的AI Agent系統中執行SDD的流程
3. 是：RD/QA也可以在本地的git儲存庫搭配Claude Code執行完整的SDD流程，只是規格文件會儲存在客製化的AI Agent所開發的看板功能中
4. 就是完全本地（現狀speclink的做法），PO/PM/RD/QA都在同一個本地的git儲存庫搭配Claude Code執行完整的SDD流程，change全部跟隨git儲存庫的版本控制
