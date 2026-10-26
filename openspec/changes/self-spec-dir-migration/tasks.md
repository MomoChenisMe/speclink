## 1. 前置與搬移（D1）

- [x] 1.1 確認可以搬移：spec-dir-migration、desktop-spec-dir-migration 與 spec-dir-path-rewrite 都已封存（不必等第一批發版，本變更封存後一次發版）；`git worktree list` 只有主目錄；`cargo build -p speclink-cli` 後 `./target/debug/speclink --version` 的 engine 版號不低於 spec-dir-migration 封存時的版號。驗證：上述三個指令的輸出符合條件，記在本任務完成註記 <!-- speclink-task:tsk_01M5J3RE5VYCF7YV4NHZMCVA2Y -->
- [ ] 1.2 把本變更目前的文件與勾選狀態 commit，讓規格資料夾沒有未提交修改。驗證：`git status --porcelain --untracked-files=all -- openspec` 沒有輸出 <!-- speclink-task:tsk_01M5J3RE5VYRW4GF1K62CBSDWQ -->
- [ ] 1.3 在主目錄執行 `./target/debug/speclink update`，把 `openspec/` 搬到 `speclink/`：stdout 顯示改名結果與提醒，改名的資料檔數等於搬移前所有變更資料夾（進行中與已封存）的數量；`openspec/` 不存在，每個變更資料夾只有 `.meta.yaml`；`.speclink.yaml` 沒有新增 `spec_dir`；repo 自用技能檔的規格路徑改為 `speclink/`。驗證：搬移前後以 `find openspec/changes -name .openspec.yaml | wc -l` 與 `find speclink/changes -name .meta.yaml | wc -l` 比對數量；`./target/debug/speclink list` 的 stderr 沒有佈局警告 <!-- speclink-task:tsk_01M5J3RE5V8AHTCJEVFTYC88WP -->

## 2. repo 工具（D2）

- [ ] 2.1 文件守門與發版守門改用新路徑：scripts/docs/vocabulary-guard.test.mjs 讀 `speclink/LANGUAGE.md`、排除 `speclink/` 下的規格散文與封存內容；scripts/release/delivery-gate.test.mjs 的略過路徑改為 `speclink/changes` 與 `speclink/discussions`；scripts/docs/docs-screenshots.mjs 的示範工作區路徑與資料檔名改為 `speclink/` 與 `.meta.yaml`。驗證：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過 <!-- speclink-task:tsk_01M5J3RE5V9TDVA9Y8KX9FPADR -->
- [ ] 2.2 版控與打包的排除項指向新資料夾：`.gitignore` 的向量搜尋資料庫改為 `speclink/.vector-search.db*`，`.dockerignore` 的排除項改為 `speclink`。驗證：`git status` 沒有出現向量搜尋資料庫檔；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 的發版守門通過 <!-- speclink-task:tsk_01M5J3RE5VQXE6DWTDGATHQX0N -->
- [ ] 2.3 協作入口指向新資料夾：.github/PULL_REQUEST_TEMPLATE.md、CONTRIBUTING.md 與 .claude/skills/release/SKILL.md 裡指本 repo 規格資料夾的路徑改為 `speclink/`。驗證：`git grep -n "openspec/" -- .github CONTRIBUTING.md .claude/skills/release` 沒有命中 <!-- speclink-task:tsk_01M5J3RE5VD2JCN2ST05YPCRVD -->
- [ ] 2.4 技能評測以新版 `speclink init` 建立的 `speclink/` 示範專案為準：integrations/claude-code/skill-evals 下的 fixtures.sh、各 scaffold.sh、評分規則與提示裡的 `openspec/` 改為 `speclink/`、`.openspec.yaml` 改為 `.meta.yaml`；integrations/claude-code/speclink-skills 的兩份 README 同步。驗證：`git grep -n -E "openspec/|\\.openspec\\.yaml" -- integrations` 沒有命中；`bash -n integrations/claude-code/skill-evals/evals/fixtures.sh` 通過 <!-- speclink-task:tsk_01M5J3RE5VKW3VC0J9X6Q266HW -->

## 3. 文件裡的 repo 自身描述（D2）

- [ ] 3.1 文件描述本 repo 自身的句子與指向 repo 內檔案的相對連結改為新資料夾：README.md 與 README.zh-TW.md（遠端契約所在位置、「Speclink 自己的規格」一段）、docs/server-deployment、docs/sdk-node、docs/product-status、docs/roadmap、docs/development（中英）裡的 `openspec/` 與 `../openspec/...` 改為 `speclink/` 與 `../speclink/...`；CHANGELOG 不改。驗證：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 的連結檢查通過；`git grep -n "openspec/" -- README.md README.zh-TW.md docs` 沒有命中 <!-- speclink-task:tsk_01M5J3RE5VQDTNQVE029J04NWQ -->

## 4. 收尾

- [ ] [M] 4.1 用桌面 app 開啟本 repo：看板與已封存頁正常顯示，專案設定頁的檔案註記為 `speclink/config.yaml`，沒有遷移提示 <!-- speclink-task:tsk_01M5J3RE5V8TEAEY9J0BSAPP33 -->
- [ ] 4.2 確認行為未變：產品程式碼沒有任何改動，既有測試照常通過，歷史記錄保留原文。驗證：`git diff --stat` 在 crates/、apps/、packages/ 下沒有變動；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過；`./target/debug/speclink list --json` 列出的變更與搬移前相同；`git grep -n "openspec/"` 只命中已封存的變更與討論、CHANGELOG 與描述舊佈局相容的正式規格；`./target/debug/speclink validate self-spec-dir-migration` 通過 <!-- speclink-task:tsk_01M5J3RE5VCCY57FKYYABNHW9T -->
