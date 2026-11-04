// 介面文字跟隨系統語系：中文系統用中文，其他系統用英文。
// 面板自己的文字以中文為原文，複製來的 Herdr mod 以英文為原文，所以 tr() 查的表看語系決定，查不到就用原文。
// 這支不碰 Herdr API，scripts/herdr/panel-model.test.mjs 直接測，也會檢查每句中文都有英文。

let lang = "zh";

export const setLang = (next) => {
  lang = next;
};

// macOS 的 AppleLanguages 長這樣：( "zh-Hant-TW" )。其他系統是 zh_TW.UTF-8。第一個語言算數。
export const langFrom = (text) => (text.match(/[A-Za-z]{2,}/)?.[0].toLowerCase() === "zh" ? "zh" : "en");

const READ = 'defaults read -g AppleLanguages 2>/dev/null || echo "$LC_ALL $LC_MESSAGES $LANG"';

// Herdr 的 JS 環境沒有 Intl 和 navigator，要問系統。exec 由呼叫的人傳進來，這支才不用碰 Herdr API。
// 先用上次的結果，畫面才不會先閃另一種語言。
export function watchLang(exec, cx) {
  lang = localStorage.getItem("lang") ?? lang;
  cx.spawn(async (cx) => {
    const r = await exec("sh", ["-c", READ]).catch(() => null);
    const next = langFrom(r?.stdout ?? "");
    localStorage.setItem("lang", next);
    if (next === lang) return;
    lang = next;
    cx.notify();
  });
}

export const TO_ZH = {
  Spaces: "工作區",
  "This computer": "這台電腦",
  "New space there": "在這裡新增工作區",
  "Loading…": "載入中…",
  Connect: "連線",
  "Connecting…": "連線中…",
  "rebuilt ·": "已重建 ·",
  "This machine was rebuilt or replaced. Click to connect to the new one.": "這台機器重建或換過了。點這裡連到新的機器。",
  Agents: "Agent",
  "Sort agents": "排序方式",
  "Sort agents: {0}": "排序方式：{0}",
  Priority: "依狀態",
  "Grouped by space": "依工作區分組",
  Pinned: "已釘選",
  "Needs you": "需要你",
  Done: "已完成",
  Working: "執行中",
  Idle: "閒置",
  Unpin: "取消釘選",
  "Pin to the top": "釘到最上面",
  "No agent status yet. Agents show up here when they report their status. Older versions and agents inside tmux may not.":
    "還沒有 agent 回報狀態。agent 回報後會顯示在這裡。舊版或在 tmux 裡的 agent 可能不會回報。",
  Settings: "設定",
  "Add machine": "新增機器",
};

export const TO_EN = {
  "、": ", ",
  "{0}：{1}": "{0}: {1}",
  // 階段與狀態
  提案中: "Proposed",
  進行中: "In progress",
  已就緒: "Ready",
  執行中: "Working",
  在等你回答: "Waiting for you",
  已完成: "Done",
  閒置: "Idle",
  正在做事: "Working",
  出錯了: "Error",
  正在收指令: "Taking a command",
  // 面板
  技能沒有送出: "The skill was not sent",
  "開不了 Speclink": "Could not open Speclink",
  "等 {0} 先完成": "Waiting for {0} to finish",
  沒有放進輸入框: "Not put in the input box",
  附加到目前分頁的輸入框: "Attach to the input box of the current tab",
  其他技能: "Other skills",
  "切到這個 agent（{0}）": "Go to this agent ({0})",
  "在 Speclink 開路線圖": "Open the roadmap in Speclink",
  "等 {0}": "Waiting for {0}",
  複製名稱: "Copy name",
  "複製 slug": "Copy slug",
  "還有 {0} 刀沒轉出": "{0} cuts not promoted yet",
  收合: "Show less",
  "還有 {0} 個…": "{0} more…",
  "第 {0} 波可以一起開工：{1}": "Wave {0} can start together: {1}",
  "開 {0} 個新分頁，各跑一個 agent": "Open {0} new tabs, one agent each",
  開工失敗: "Could not start",
  "用 worktree": "Use a worktree",
  "直接 apply": "Apply directly",
  取消: "Cancel",
  "選擇用 worktree 或直接 apply": "Choose a worktree or a direct apply",
  整波開工: "Start the wave",
  "這個專案還沒有 Speclink": "This project has no Speclink yet",
  "在新分頁執行 speclink init": "Run speclink init in a new tab",
  初始化: "Initialize",
  討論: "Discussions",
  已轉出: "Promoted",
  "這些變更讀不到 metadata：{0}": "Could not read metadata for these changes: {0}",
  "開啟 Speclink app": "Open the Speclink app",
  "在 Speclink app 開啟這個專案": "Open this project in the Speclink app",
  目前的工作區: "Current space",
  路線圖: "Roadmap",
  技能: "Skills",
  "載入中…": "Loading…",
  // 技能跳窗
  "輸入框一次只能放一個。勾兩個以上請開新分頁": "The input box takes one at a time. With two or more checks, open new tabs",
  主題: "Topic",
  "補充說明（可留空）": "Notes (optional)",
  "這個技能用在整個專案，不用搭配": "This skill works on the whole project. It needs no pairing",
  目前沒有可以搭配的討論: "No discussion to pair with",
  目前沒有可以搭配的變更: "No change to pair with",
  搭配的討論: "Pair with a discussion",
  搭配的變更: "Pair with a change",
  "全選 ⌘A": "All ⌘A",
  不指定: "None",
  "處理中…": "Working on it…",
  "開 {0} 個分頁並開工 ⌘⏎": "Open {0} tabs and start ⌘⏎",
  "開新分頁並開工 ⌘⏎": "Open a new tab and start ⌘⏎",
  "在目前的工作區開新分頁並執行：{0}": "Open a new tab in the current space and run: {0}",
  "放到輸入框 ⏎": "Put in input box ⏎",
  "↑↓ 技能　←→ 搭配　Space 勾選": "↑↓ Skill   ←→ Pair   Space Check",
  "↑↓ 技能　←→ 搭配": "↑↓ Skill   ←→ Pair",
  "這個專案還沒有 Speclink。到面板按「初始化」後再用。": "This project has no Speclink yet. Press Initialize in the panel first.",
  // 共用
  "speclink {0} 結束碼 {1}": "speclink {0} exit code {1}",
  "貼不進 {0}（結束碼 {1}）": "Could not paste into {0} (exit code {1})",
  "hc 結束碼 {0}": "hc exit code {0}",
  // 技能表
  規劃: "Plan",
  實作: "Build",
  品質: "Quality",
  收尾: "Finish",
  專案: "Project",
  變更: "Change",
  把模糊的需求談清楚: "Talk through a fuzzy requirement",
  建立變更與文件: "Create a change and its documents",
  把新需求併進變更: "Add new requirements to a change",
  照任務清單實作: "Implement from the task list",
  "在獨立的 worktree 裡實作": "Implement in a separate worktree",
  實作前交叉檢查文件: "Cross-check the documents before you implement",
  查程式碼離變更的假設多遠: "Check how far the code moved from the change",
  "跑 review 和 verify 兩站": "Run the review and verify stations",
  檢查實作的品質: "Check the quality of the implementation",
  對照文件檢查實作: "Check the implementation against the documents",
  把變更併進規格並封存: "Merge the change into the specs and archive it",
  "把 worktree 合回主分支": "Merge the worktree back into the main branch",
  只提交這個變更的檔案: "Commit only the files of this change",
  掃描程式碼找改善點: "Scan the code for improvements",
  找出程式碼的安全問題: "Find security problems in the code",
  查一項能力的來龍去脈: "Trace how a capability came to be",
  從規格產生操作手冊: "Generate a manual from the specs",
  從現有程式碼產生初始規格: "Generate initial specs from the code",
  整理專案說明與產出規則: "Organize project context and artifact rules",
  // 送達與開新分頁的原因
  "還在確認可用的 agent": "Checking which agents are available",
  "找不到有 speclink 技能的 agent": "No agent has the speclink skills",
  開工: "Start",
  這個變更: "this change",
  這場討論: "this discussion",
  "送給這場討論的 agent": "Send to the agent on this discussion",
  目前分頁: "the current tab",
  "{0}的 agent {1}": "Agent on {0}: {1}",
  開新分頁: "Open a new tab",
  "送給這個變更的 agent": "Send to the agent on this change",
  "送給目前分頁的 agent": "Send to the agent in the current tab",
  "目前分頁的 agent 在等你回答": "The agent in the current tab waits for your answer",
  "目前分頁沒有 agent。可以改開新分頁": "The current tab has no agent. You can open a new tab",
  "目前分頁沒有 agent": "The current tab has no agent",
  "{0} 只有 Claude Code 能用": "{0} works only in Claude Code",
  "已經有 agent 在做": "An agent already works on it",
  "要等 {0} 落地": "Waiting for {0} to land",
  "還有 {0} 個程式任務沒做完": "{0} code tasks are not done",
  "這個專案沒開 worktree 流程": "This project has no worktree flow",
  "{0} 的 apply 還沒做完": "The apply of {0} is not finished",
  勾選要開工的變更: "Check the changes to start",
  "主資料夾一次只能跑一個 apply。可以改選 apply-with-worktree": "The main folder runs one apply at a time. You can use apply-with-worktree",
};

export function tr(text, ...args) {
  const table = lang === "zh" ? TO_ZH : TO_EN;
  return (table[text] ?? text).replace(/\{(\d)\}/g, (_match, i) => args[i]);
}
