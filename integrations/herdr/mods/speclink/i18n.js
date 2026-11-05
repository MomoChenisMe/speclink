// 介面文字跟隨系統語系：中文系統用中文，其他系統用英文。
// 面板自己的文字以中文為原文，Herdr 的文字以英文為原文，所以 tr() 查的表看語系決定，查不到就用原文。
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
export function watchLang(exec, cx, ui) {
  lang = localStorage.getItem("lang") ?? lang;
  if (ui) translateHerdr(ui);
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
  // Herdr 內建畫面：分頁列、狀態列
  Files: "檔案",
  Changes: "變更",
  commands: "指令",
  // 設定頁
  Look: "外觀",
  Keyboard: "鍵盤",
  Machines: "機器",
  MACHINES: "機器",
  Alerts: "提醒",
  Updates: "更新",
  Privacy: "隱私",
  Terminals: "終端機",
  System: "跟隨系統",
  Light: "淺色",
  Dark: "深色",
  "Type to search fonts": "輸入文字搜尋字型",
  "This layout sets its own; yours shows in the others.": "這個版面有自己的設定，你的設定會用在其他版面。",
  LAYOUT: "版面",
  THEME: "主題",
  Appearance: "外觀模式",
  "Light or dark, or as your system is now.": "淺色、深色，或跟隨系統目前的設定。",
  "For the layout you're in: each layout keeps its own light and dark theme.": "只影響目前的版面：每個版面各自記住淺色和深色主題。",
  "SIZE AND FONTS": "大小與字型",
  Zoom: "縮放",
  "Everything: terminals, panels, text.": "全部一起縮放：終端機、面板和文字。",
  Reset: "重設",
  "Terminal font": "終端機字型",
  "Interface font": "介面字型",
  "Automatic (Alt on US keyboards)": "自動（美式鍵盤當 Alt）",
  "Types characters": "輸入字元",
  "Left Option is Alt": "左 Option 當 Alt",
  "Right Option is Alt": "右 Option 當 Alt",
  "OPTION KEY": "Option 鍵",
  "Use Option for terminal shortcuts or to type accents and special characters.": "Option 鍵可以當終端機快捷鍵，也可以用來輸入重音和特殊字元。",
  "Couldn't read machines.": "讀不到機器清單。",
  "It didn't work.": "沒有成功。",
  "Updating…": "更新中…",
  "Renaming…": "改名中…",
  "Removing…": "移除中…",
  Update: "更新",
  Rename: "重新命名",
  Remove: "移除",
  "Copying this computer's herdr-core. Its terminals keep running.": "正在複製這台電腦的 herdr-core。它的終端機會繼續執行。",
  Cancel: "取消",
  "Remove machine": "移除機器",
  "New name (addresses using the old name stop working)": "新名稱（用舊名稱的位址會失效）",
  Save: "儲存",
  "Your computers, reached over ssh. Their terminals keep running when you disconnect.": "透過 ssh 連線的電腦。中斷連線後，它們的終端機會繼續執行。",
  "Connected · hc version unknown": "已連線 · 不知道 hc 版本",
  "Reading machines…": "讀取機器中…",
  "No other machines yet.": "還沒有其他機器。",
  "Add a machine": "新增機器",
  "Herdr's sound": "Herdr 的音效",
  SOUNDS: "音效",
  Sounds: "音效",
  "Play sounds": "播放音效",
  "When an agent needs you, and when one finishes while you're not looking. A note shows in the corner either way.": "agent 需要你時，以及你沒在看時有 agent 做完時。不管有沒有開，角落都會跳通知。",
  "An agent needs you": "agent 需要你",
  Play: "播放",
  "An agent finished": "agent 做完了",
  "Want your own sounds? Ask your agent to set sounds.done and sounds.blocked in settings.json to an MP3.": "想用自己的音效嗎？請 agent 把 settings.json 的 sounds.done 和 sounds.blocked 設成 MP3。",
  "Checking…": "檢查中…",
  "Up to date.": "已是最新版本。",
  "No successful check. Check your connection and try again.": "還沒有檢查成功。請檢查網路連線後再試一次。",
  "✓ Up to date": "✓ 已是最新版本",
  "Check now": "立即檢查",
  "A build from source: it updates with git, not here.": "這是從原始碼建置的版本，要用 git 更新，不在這裡更新。",
  "Restart to update": "重新啟動並更新",
  "Updating… your terminals keep running.": "更新中…你的終端機會繼續執行。",
  "Try again": "再試一次",
  "Update herdr-core": "更新 herdr-core",
  "Not checked: a build from source looks for no updates.": "沒有檢查：從原始碼建置的版本不會找更新。",
  "What's new": "新功能",
  "Herdr looks for both every hour and tells you in the corner. herdr-core runs your terminals, so they keep running through its updates.": "Herdr 每小時檢查這兩個，有更新時會在角落通知。herdr-core 負責執行你的終端機，所以更新時終端機會繼續執行。",
  "USAGE DATA": "使用資料",
  "Share anonymous usage data": "分享匿名使用資料",
  "Once a day: which agents and features you use, how many spaces and tabs, your system and Herdr's version. Never what's in your terminals, never names of files, folders or projects.": "每天一次：你用哪些 agent 和功能、有幾個工作區和分頁、你的系統和 Herdr 版本。絕不包含終端機內容，也絕不包含檔案、資料夾或專案的名稱。",
  "What we send": "我們傳送的內容",
  "This build never sends: it's built from source. You can still see what a release would send.": "這個版本從原始碼建置，不會傳送。你還是可以看到正式版會傳送什麼。",
  "Exactly what leaves your computer, word for word.": "一字不差，就是離開你電腦的內容。",
  Hide: "隱藏",
  Show: "顯示",
  "Nothing is collected or sent while it's off.": "關閉時不會收集也不會傳送任何資料。",
  "Sent as is, soon.": "很快會原封不動傳送。",
  "Today so far": "今天到目前為止",
  "A draft: it changes until the day ends (UTC), then goes out once.": "這是草稿：當天（UTC）結束前都會變動，之後傳送一次。",
  "Send feedback": "傳送意見回饋",
  // 檔案面板
  "Cannot list this folder": "無法列出這個資料夾",
  "This file is no longer available": "這個檔案已經不存在",
  "Binary file · no text preview": "二進位檔案 · 沒有文字預覽",
  "Cannot preview this file (binary, too large, or unreadable)": "無法預覽這個檔案（二進位、太大或讀不到）",
  "Loading files…": "載入檔案中…",
  "No files": "沒有檔案",
  "No folder": "沒有資料夾",
  "File list limited to 500 entries per folder and 5,000 total": "檔案清單最多顯示每個資料夾 500 個、總共 5,000 個",
  "Loading file…": "載入檔案中…",
  // 變更面板
  "Commit message": "提交訊息",
  "Not a git repository": "不是 git 儲存庫",
  "Cannot preview this diff (too large or unreadable)": "無法預覽這個差異（太大或讀不到）",
  "Binary file · no text diff": "二進位檔案 · 沒有文字差異",
  "Binary file changed": "二進位檔案有變更",
  "No text diff available": "沒有文字差異",
  "Loading changes…": "載入變更中…",
  "No uncommitted changes": "沒有未提交的變更",
  "Showing the first 1,000 changed files": "只顯示前 1,000 個變更的檔案",
  "Select a file to view its changes": "選一個檔案來看它的變更",
  "Committing…": "提交中…",
  Commit: "提交",
  // 右鍵和新增分頁的選單
  "Split right": "向右分割",
  "Split down": "向下分割",
  "Use Herdr right-click menu": "使用 Herdr 的右鍵選單",
  "Send right-clicks to pane": "把右鍵交給窗格裡的程式",
  "Close pane": "關閉窗格",
  "New tab": "新分頁",
  Close: "關閉",
  "New worktree": "新增 worktree",
  "Open worktree": "開啟 worktree",
  "Remove worktree": "移除 worktree",
  "New space": "新增工作區",
  Keybinds: "快捷鍵",
  Terminal: "終端機",
  "Start an agent": "開一個 agent",
  "not installed": "未安裝",
  "Could not load agents": "載入不了 agent",
  "Checking installed agents…": "檢查已安裝的 agent…",
  "New worktree…": "新增 worktree…",
  "Open worktree…": "開啟 worktree…",
  // 複製版的側欄
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
  我的技能: "My skills",
  給: "To",
  主資料夾: "Main folder",
  兩站都跑: "Both stations",
  "只 review": "Review only",
  "只 verify": "Verify only",
  只封存: "Archive only",
  封存後提交: "Archive, then commit",
  開發: "Development",
  設計: "Design",
  提交與發版: "Commit and release",
  影片: "Video",
  動畫: "Animation",
  內容: "Content",
  工具: "Tools",
  其他: "Other",
  個人: "Personal",
  外掛: "Plugin",
  "外掛 {0}": "Plugin {0}",
  "{0} 個": "{0}",
  "找技能：名稱或說明": "Find a skill by name or description",
  找不到符合的技能: "No matching skill",
  "{0} 目前沒有 Speclink 以外的技能": "{0} has no skills other than Speclink",
  選一個技能: "Select a skill",
  "目前分頁是 {0}，要放到輸入框請選它": "The current tab runs {0}. Select it to put the command in the input box",
  "這個專案沒有 {0} 的 Speclink 技能。先執行 speclink init --tools {1}": "This project has no Speclink skills for {0}. Run speclink init --tools {1} first",
  目前分頁: "Current tab",
  "perl 結束碼 {0}": "perl exit code {0}",
  "↑↓ 移動　→ 補充說明　Tab 換頁": "↑↓ Move　→ Note　Tab Switch page",
  "↑↓ 技能": "↑↓ Skill",
  "←→ 搭配": "←→ Pair",
  "Space 勾選": "Space Check",
  "⇧←→ 做法": "⇧←→ Method",
  "Tab 換頁": "Tab Switch page",
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
  封存後接著提交這個變更: "Archive the change, then commit it",
  "一次只能跑一個 archive+commit，同時提交會互相影響": "Run one archive+commit at a time. Commits at the same time affect each other",
  審查中: "In review",
  已審查: "Reviewed",
  "已審查·其後有變動": "Reviewed · changed since",
  驗證中: "In verification",
  已驗證: "Verified",
  "已驗證·其後有變動": "Verified · changed since",
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
  "目前分頁的 agent {0}": "Agent on the current tab: {0}",
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

// Herdr 內建畫面裡帶變數的句子。要整句符合，名稱是使用者取的。{n} 是第 n 個括號。
const ZH_PATTERNS = [
  [/^(\d+) agents$/, "{0} 個 agent"],
  [/^· (\d+) needs you$/, "· {0} 個需要你"],
  [/^(.+) {2}commands$/, "{0}  指令"],
  [/^([\d,]+) files$/, "{0} 個檔案"],
  [/^Theme: (.+)$/, "主題：{0}"],
  [/^Send back to (.+)$/, "送回 {0}"],
  [/^Default \((.+)\)$/, "預設（{0}）"],
  [/^Updated (.+) to hc (.+); its terminals kept running\.$/, "已更新 {0} 到 hc {1}，它的終端機繼續執行。"],
  [/^Updated (.+); its terminals kept running\.$/, "已更新 {0}，它的終端機繼續執行。"],
  [/^Renamed (.+) to (.+)\.$/, "已把 {0} 改名為 {1}。"],
  [/^Removed (.+); its terminals keep running there\.$/, "已移除 {0}，它的終端機會在那台機器上繼續執行。"],
  [/^Remove (.+)\? Its terminals keep running there\.$/, "要移除 {0} 嗎？它的終端機會在那台機器上繼續執行。"],
  [/^Connected · (.+)$/, "已連線 · {0}"],
  [/^Up to date\. Checked at (.+)\.$/, "已是最新版本。檢查時間 {0}。"],
  [/^(.+) is ready\. Restart to switch now; it also switches when you quit\.$/, "{0} 已經準備好。重新啟動就會切換，結束 Herdr 時也會切換。"],
  [/^(.+) is ready\. Your terminals keep running through it\.$/, "{0} 已經準備好。更新時你的終端機會繼續執行。"],
  [/^Managed by (.+): update it there\.$/, "由 {0} 管理，請在那裡更新。"],
  [/^The update failed: (.+)$/, "更新失敗：{0}"],
  [/^Off for now: (.+) is set where Herdr started\.$/, "目前關閉：啟動 Herdr 的環境設定了 {0}。"],
  [/^Waiting to send: (.+)$/, "等待傳送：{0}"],
  [/^Last sent: (.+)$/, "上次傳送：{0}"],
  [/^Sent (.+)\.$/, "傳送於 {0}。"],
  [/^Cannot preview this diff \(too large or unreadable\): (.+)$/, "無法預覽這個差異（太大或讀不到）：{0}"],
  [/^Commit failed: (.+)$/, "提交失敗：{0}"],
];

// 只查對照表和上面的句子，查不到就是原文。英文系統不用翻，內建畫面本來就是英文。
export function autoTr(text) {
  if (lang !== "zh") return text;
  if (Object.hasOwn(TO_ZH, text)) return TO_ZH[text];
  for (const [pattern, zh] of ZH_PATTERNS) {
    const found = text.match(pattern);
    if (found) return zh.replace(/\{(\d)\}/g, (_match, i) => found[Number(i) + 1]);
  }
  return text;
}

const TRANSLATE = Symbol.for("speclink.translate");

// Herdr 所有 mod 共用同一份 JS 環境，所以改一次元件的原型，內建畫面（設定頁、選單、分頁列、狀態列）的文字也會查對照表，
// 不用複製那些 mod，從哪個入口開都一樣。文件沒有保證這件事：Herdr 改掉後這段不會生效，畫面回到英文，不會壞。
// 使用者的內容不能經過這裡：檔案樹和差異由 dock-panel.mjs 的複製版排除。ui 由呼叫的人傳進來，這支才不用碰 Herdr API。
export function translateHerdr({ div, InputState }) {
  const proto = Object.getPrototypeOf(div());
  const installed = TRANSLATE in proto;
  // mod 重新載入後要用新的這份 lang，但不能包兩層。
  proto[TRANSLATE] = autoTr;
  if (installed) return;
  const text = (value) => (typeof value === "string" ? proto[TRANSLATE](value) : value);
  const { child, children, tooltip } = proto;
  proto.child = function (value) {
    return child.call(this, text(value));
  };
  proto.children = function (values) {
    return children.call(this, Array.from(values, text));
  };
  proto.tooltip = function (value) {
    return tooltip.call(this, text(value));
  };
  // 輸入框的提示文字不是 child。
  const make = InputState?.new;
  if (!make) return;
  InputState.new = function (options) {
    return make.call(this, options?.placeholder ? { ...options, placeholder: text(options.placeholder) } : options);
  };
}

export function tr(text, ...args) {
  const table = lang === "zh" ? TO_ZH : TO_EN;
  return (table[text] ?? text).replace(/\{(\d)\}/g, (_match, i) => args[i]);
}
