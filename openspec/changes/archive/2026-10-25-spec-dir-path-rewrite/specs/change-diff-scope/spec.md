## MODIFIED Requirements

### Requirement: Git-backed discovery scope 解析完整 worktree patch

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

speclink review scope <change> SHALL 以 touchedFiles 聯集作自動候選路徑，使用可信 baseCommit 到目前 worktree 的 Git diff，涵蓋 staged、unstaged、rename、delete 與 touched untracked additions；SHALL NOT 使用只比較 commit graph 的 `<base>...HEAD`。規格資料夾內的 artifacts 與 `.speclink` work data SHALL NOT 成為 review target。

成功的文字 file delta SHALL 回報 oldPath、newPath、kind、beforeHash、afterHash 與 hunks；每個 hunk SHALL 含 id、oldStart、oldLines、newStart、newLines。addition SHALL 允許 oldStart=0、oldLines=0；deletion SHALL 允許 newStart=0、newLines=0；多段修改 SHALL 保留多筆 hunk。rename SHALL 保留 oldPath 與 newPath。binary SHALL 回報 file hashes 與 kind=binary，hunks 為空。

#### Scenario: 未提交 staged 與 unstaged 內容都在 scope

- **WHEN** touched path 同時有 staged 與 unstaged 修改，baseCommit 等於 Apply 開始時 HEAD
- **THEN** review scope 的 patch 同時含兩部分修改，即使 `git diff <base>...HEAD` 對該 fixture 為空

#### Scenario: untracked touched file 是整檔 addition

- **WHEN** touchedFiles 含一個未追蹤 UTF-8 檔案
- **THEN** resolved payload 將它列為 added，beforeHash 為 null，首個 hunk 的 oldStart=0、oldLines=0

#### Scenario: delete 與 rename 保留雙端語意

- **WHEN** touched scope 含一個刪除檔與一個 Git 可辨識 rename
- **THEN** delete 的 afterHash 為 null 且 newLines=0，rename 同時輸出 oldPath 與 newPath
