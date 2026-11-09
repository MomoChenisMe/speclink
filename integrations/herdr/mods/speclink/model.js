// 面板的純邏輯：CLI 輸出怎麼變成面板的列、每一列有哪些技能鈕、技能要送給誰。
// 不碰 Herdr API，scripts/herdr/panel-model.test.mjs 直接測。
//
// 階段、波次、依賴、尚未轉出的刀都取自 CLI 的 --json，這裡不重算。

import { tr } from "./i18n.js";

// 有 speclink 技能的 agent（program.key）與叫用技能的前綴。Pi 讀 .agents/skills，和 Codex 共用技能檔。
const SKILL_PREFIX = { claude: "/", copilot: "/", codex: "$", pi: "/skill:" };
const PREFIX = Object.fromEntries(Object.entries(SKILL_PREFIX).map(([key, prefix]) => [key, `${prefix}speclink-`]));
// speclink init --tools 的名稱。
const SPECLINK_TOOL = { claude: "claude", copilot: "copilot", codex: "codex", pi: "codex" };
// sending：hc send 要等 agent 開工才回來（約兩秒），這段時間再按一次會重複送出。
const BUSY = { working: "正在做事", blocked: "在等你回答", error: "出錯了", sending: "正在收指令" };

// 這些技能要在主資料夾跑；其餘技能跟著變更走，有 worktree 就在 worktree 裡跑。
const MAIN_FOLDER = new Set(["archive", "archive+commit", "worktree-merge", "apply-with-worktree"]);
const CLAUDE_ONLY = new Set(["analyze"]);

// 第一個是建議的下一步。這裡只放合理的選項，技能本身沒有階段限制的（ingest、drift、commit）每個階段都有。
const STAGE_SKILLS = {
  proposed: ["apply", "analyze", "ingest", "drift", "commit"],
  "in-progress": ["apply", "ingest", "drift", "verify", "quality", "review", "analyze", "commit"],
  ready: ["quality", "review", "verify", "archive", "archive+commit", "ingest", "drift", "commit"],
};
// 這些技能要等程式任務都做完才能跑，手動任務（[M]）不算。沒做完時技能也會自己停下來。
const NEEDS_CODE_DONE = new Set(["quality", "review"]);
// 技能跳窗的技能。前四組依工作流程，pair 是可以搭配的東西；「專案」用在整個專案，不搭配。
export const PICKER_SKILLS = [
  { skill: "discuss", group: "規劃", pair: "討論", about: "把模糊的需求談清楚" },
  { skill: "propose", group: "規劃", pair: "討論", about: "建立變更與文件" },
  { skill: "ingest", group: "規劃", pair: "變更", about: "把新需求併進變更" },
  { skill: "apply", group: "實作", pair: "變更", about: "照任務清單實作" },
  { skill: "apply-with-worktree", group: "實作", pair: "變更", about: "在獨立的 worktree 裡實作" },
  { skill: "analyze", group: "實作", pair: "變更", about: "實作前交叉檢查文件" },
  { skill: "drift", group: "實作", pair: "變更", about: "查程式碼離變更的假設多遠" },
  { skill: "quality", group: "品質", pair: "變更", about: "跑 review 和 verify 兩站" },
  { skill: "review", group: "品質", pair: "變更", about: "檢查實作的品質" },
  { skill: "verify", group: "品質", pair: "變更", about: "對照文件檢查實作" },
  { skill: "archive", group: "收尾", pair: "變更", about: "把變更併進規格並封存" },
  { skill: "archive+commit", group: "收尾", pair: "變更", about: "封存後接著提交這個變更" },
  { skill: "worktree-merge", group: "收尾", pair: "變更", about: "把 worktree 合回主分支" },
  { skill: "commit", group: "收尾", pair: "變更", about: "只提交這個變更的檔案" },
  { skill: "improve", group: "專案", pair: null, about: "掃描程式碼找改善點" },
  { skill: "audit", group: "專案", pair: null, about: "找出程式碼的安全問題" },
  { skill: "trace", group: "專案", pair: null, about: "查一項能力的來龍去脈" },
  { skill: "manual", group: "專案", pair: null, about: "從規格產生操作手冊" },
  { skill: "baseline", group: "專案", pair: null, about: "從現有程式碼產生初始規格" },
  { skill: "config", group: "專案", pair: null, about: "整理專案說明與產出規則" },
];

// 真正存在的技能。疊加的 archive+commit 是兩個技能。
export const ALL_SKILLS = new Set(PICKER_SKILLS.map((w) => w.skill).filter((skill) => !skill.includes("+")));

// 跳窗清單裡合成一列的技能：同一件事的不同做法，選中後在右邊挑。第一個是預設，也是那一列的名字。
export const PICKER_VARIANTS = {
  apply: [
    { label: "主資料夾", skill: "apply" },
    { label: "worktree", skill: "apply-with-worktree" },
  ],
  quality: [
    { label: "兩站都跑", skill: "quality" },
    { label: "只 review", skill: "review" },
    { label: "只 verify", skill: "verify" },
  ],
  archive: [
    { label: "只封存", skill: "archive" },
    { label: "封存後提交", skill: "archive+commit" },
  ],
};
const MERGED = new Set(Object.values(PICKER_VARIANTS).flatMap((list) => list.slice(1).map((v) => v.skill)));
export const PICKER_ROWS = PICKER_SKILLS.filter((w) => !MERGED.has(w.skill));
const PICKER_GROUPS = [...new Set(PICKER_ROWS.map((w) => w.group))];

// 左邊清單每一組怎麼顯示：closed 收合，open 只列現在能做的，all 全列。choices 是使用者點過的。
// 沒有變更可以搭配的技能現在做不了，所以藏起來；discuss、propose 沒有討論也能打字開工；「專案」不常用，預設收合。
export function pickerGroups(countOf, choices = {}) {
  const canDo = (w) => w.pair === "討論" || (w.pair === "變更" && countOf(w.skill) > 0);
  return PICKER_GROUPS.map((group) => {
    const items = PICKER_ROWS.filter((w) => w.group === group);
    const state = choices[group] ?? (items.some(canDo) ? "open" : "closed");
    return { group, items, state, rows: state === "closed" ? [] : state === "all" ? items : items.filter(canDo) };
  });
}

// ↑↓ 的下一列。收合的組也走得到，走進去就展開（open 是要展開的組），這樣用鍵盤也摸得到每個技能。
export function pickerStep(groups, skill, step) {
  const path = groups.flatMap((g) => {
    if (g.state !== "closed") return g.rows.map((w) => ({ skill: w.skill, open: null }));
    return [{ skill: g.items[step > 0 ? 0 : g.items.length - 1].skill, open: g.group }];
  });
  const at = path.findIndex((x) => x.skill === skill);
  const from = at === -1 && step < 0 ? 0 : at;
  return path[(from + step + path.length) % path.length];
}

// 品質關卡的兩站，值是 .meta.yaml 裡欄位的開頭。
const STATIONS = { review: "reviewed", verify: "verified" };

function readStamp(meta, prefix) {
  const field = (name) => meta.match(new RegExp(`^${prefix}_${name}:[ \\t]*(\\S.*)$`, "m"))?.[1];
  const block = meta.match(new RegExp(`^${prefix}_scope:\\n((?:[ \\t-].*\\n?)*)`, "m"))?.[1] ?? "";
  const scope = [...block.matchAll(/path:[ \t]*(.+)\n\s*hash:[ \t]*(\S+)/g)].map(([, path, hash]) => ({
    path: path.replace(/^["']|["']$/g, ""),
    hash,
  }));
  const total = Number(field("tasks_total"));
  // 欄位不全不算章，和引擎的 is_stamped 相同。
  return field("at") && Number.isInteger(total) && scope.length > 0 ? { total, scope } : null;
}

// 讀 shared.js 的 stationFacts 拿到的文字：.meta.yaml 加上 open_ticket: 列。CLI 的 list --json 刻意不帶章，所以要自己讀。
export function parseStations(text) {
  return Object.fromEntries(
    Object.entries(STATIONS).map(([station, prefix]) => [
      station,
      { ticket: new RegExp(`^open_ticket: ${station}$`, "m").test(text), stamp: readStamp(text, prefix) },
    ]),
  );
}

export const stampedPaths = (stations) => [
  ...new Set(Object.keys(STATIONS).flatMap((station) => stations[station].stamp?.scope.map((f) => f.path) ?? [])),
];

// 每一站的狀態：none、open（工單開著）、stamped（章有效）或 stale（章過期）。判斷和引擎的封存守門相同：
// 工單開著的站不看章；章要任務數沒變、程式任務都做完、範圍內的檔案內容沒變。hashes 查不到時只看任務。
export function stationStates(facts, row) {
  const state = (station) => {
    const { ticket, stamp } = facts?.[station] ?? {};
    if (ticket) return "open";
    if (!stamp) return "none";
    if (stamp.total !== row.total || row.codeRemaining > 0) return "stale";
    return facts.hashes && stamp.scope.some((f) => facts.hashes[f.path] !== f.hash) ? "stale" : "stamped";
  };
  return Object.fromEntries(Object.keys(STATIONS).map((station) => [station, state(station)]));
}

// remaining 是每個變更還剩幾個程式任務（見 shared.js 的 codeRemaining），facts 是章和工單（見 stationFacts），
// 查不到的不放。
export function buildBoard(list, plan, discussions, remaining = {}, facts = {}) {
  const listed = new Map(list.changes.map((c) => [c.name, c]));
  const stages = { proposed: [], "in-progress": [], ready: [] };
  for (const c of plan.changes) {
    const l = listed.get(c.name);
    const row = {
      name: c.name,
      stage: c.stage,
      wave: c.wave,
      blockedBy: c.blockedBy,
      completed: l?.completedTasks ?? 0,
      total: l?.totalTasks ?? 0,
      codeRemaining: c.stage === "ready" ? 0 : remaining[c.name] ?? null,
      worktree: l?.worktree ?? null,
      restale: (l?.restaleFrom ?? []).length > 0,
    };
    stages[c.stage].push({ ...row, stations: stationStates(facts[c.name], row) });
  }
  const cuts = new Map();
  for (const cut of plan.unpromotedCuts ?? []) {
    cuts.set(cut.discussion, [...(cuts.get(cut.discussion) ?? []), ...cut.unpromoted]);
  }
  const rows = discussions.discussions.map((d) => ({
    slug: d.slug,
    topic: d.topic,
    status: d.status,
    cuts: cuts.get(d.slug) ?? [],
  }));
  // 已轉出但還有刀沒轉出的討論留在討論卡，下一步的 propose 才不會收在預設收合的卡裡。
  const finished = (d) => d.status === "promoted" && d.cuts.length === 0;
  return {
    stages,
    discussions: rows.filter((d) => !finished(d)),
    promoted: rows.filter(finished),
    skipped: plan.skipped ?? [],
  };
}

// 資料夾沒有 Speclink 專案時，CLI 所有動詞都回這句，而且不是 JSON。設定檔壞掉是另一句，要照常當錯誤。
export const isUninitialized = (message) => message.includes("Not initialized");

// speclink 網址（正式規格 desktop-app「以 speclink 網址開啟詳情頁」）。kind 是 change、
// discussion、roadmap 或 project；roadmap 與 project 不帶名稱。
export function speclinkUrl(kind, project, name) {
  const path = name ? `/${encodeURIComponent(name)}` : "";
  return `speclink://${kind}${path}?project=${encodeURIComponent(project)}`;
}

// 桌面 app 的 bundle id。
export const APP_ID = "app.speclink.desktop";

// 這個 space 的專案能不能用網址在桌面 app 開：讀得到專案，而且專案在這台機器上。
// Logo 和「路線圖」鈕用同一個門檻。
export const canOpenProject = ({ hasBoard, on }) => hasBoard && !on;

// Logo 依序嘗試的 open 參數。能用網址開時先送網址切過去；沒有註冊 speclink 網址的舊版 app 會讓
// 網址送不出去，這時退回只叫出 app。
export function logoOpenAttempts({ root, hasBoard, on }) {
  const bringUp = ["-b", APP_ID];
  return canOpenProject({ hasBoard, on }) ? [[speclinkUrl("project", root)], bringUp] : [bringUp];
}

// 依序跑 open，成功就停並回傳 null；全部失敗回傳最後一次有內容的錯誤。exec 由呼叫端傳入。
export async function lastOpenError(attempts, exec) {
  let error = "";
  for (const args of attempts) {
    const r = await exec("open", args).catch((e) => ({ code: 1, stderr: String(e.message ?? e) }));
    if (r.code === 0) return null;
    error = r.stderr || error;
  }
  return error;
}

// 疊加的技能（archive+commit）依序跑，標籤和 hook 一樣算第一個。
const skillLabel = (skill) => `speclink.skill=speclink-${skill.split("+")[0]}`;

// 這個 agent 接下來在做什麼。name 是技能搭配的變更或討論。變更的技能沒指定時不動 speclink.change，
// 那通常是同一個變更的下一步；其他情況清掉用不到的標籤（hc label k= 是移除）。
export function skillLabels(skill, name) {
  const pair = PICKER_SKILLS.find((w) => w.skill === skill)?.pair;
  const labels = [skillLabel(skill)];
  if (pair === "變更") return [...labels, ...(name ? [`speclink.change=${name}`] : []), "speclink.discussion="];
  return [...labels, "speclink.change=", `speclink.discussion=${pair === "討論" && name ? name : ""}`];
}

// hook 用：送出的訊息是 speclink 技能指令時，回傳要貼的標籤，其他訊息回傳 null。名稱只認第一個參數。
// 變更名稱只有英數，所以中文不算；討論的 slug 可能有中文。疊加的指令（/speclink-archive + /speclink-commit x）算第一個技能。
export function promptLabels(prompt) {
  const match = prompt.trimStart().match(/^(?:\/skill:|\/|\$)speclink-([a-z-]+)(?:\s+\+\s+(?:\/skill:|\/|\$)speclink-[a-z-]+)?(?:\s+(?:--from-discussion\s+)?(\S+))?/);
  if (!match || !ALL_SKILLS.has(match[1])) return null;
  const [, skill, name] = match;
  const isChange = PICKER_SKILLS.find((w) => w.skill === skill).pair === "變更";
  return skillLabels(skill, isChange && !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(name ?? "") ? undefined : name);
}

// 從 speclink 指令看出 agent 在做哪個變更或討論。只認會動到它的指令，show、status 這類可能只是查一下。
// 指令要在開頭，或接在 ; & | ( 後面，所以 commit 訊息裡提到的不算；heredoc 的內文也不看。
const NAME = String.raw`"?([A-Za-z0-9][^\s"';&|]*)"?`;
// speclink 可能帶路徑（~/.local/bin/speclink）。
const BIN = String.raw`["']?(?:[^\s;&|()"']*/)?speclink["']?`;
const speclinkAt = (verb) => new RegExp(String.raw`(?:^|[;&|(]\s*)(?:\w+=\S*\s+)*${BIN}\s+${verb}\s+${NAME}`, "m");
const CHANGE_COMMANDS = [
  "new\\s+change",
  "instructions\\s+apply\\s+--change",
  "task\\s+done\\s+--change",
  "in-progress\\s+add",
  "(?:review|verify)\\s+prepare",
  "archive",
].map(speclinkAt);
const DISCUSSION_COMMANDS = ["discuss\\s+new\\s+.*--slug", "discuss\\s+(?:context|add-round|conclude)"].map(speclinkAt);

// 也可能先存進變數：S=~/.local/bin/speclink; $S discuss …
function expandAlias(text) {
  const alias = text.match(new RegExp(String.raw`(?:^|[;&|(\s])(\w+)=${BIN}(?=[\s;&|)]|$)`, "m"))?.[1];
  return alias ? text.replace(new RegExp(String.raw`["']?\$\{?${alias}\}?["']?(?=\s)`, "g"), "speclink") : text;
}

export function commandLabels(command) {
  const text = expandAlias(command.split("<<")[0]);
  const change = CHANGE_COMMANDS.map((pattern) => text.match(pattern)?.[1]).find(Boolean);
  if (change) return [`speclink.change=${change}`, "speclink.discussion="];
  const discussion = DISCUSSION_COMMANDS.map((pattern) => text.match(pattern)?.[1]).find(Boolean);
  return discussion ? ["speclink.change=", `speclink.discussion=${discussion}`] : null;
}

// Claude Code hook 收到的事件要貼哪些標籤，不用貼時是 null。
// /clear 和 /resume 換了對話，舊的標籤不算數。模型自己叫用技能時，Skill 工具帶著技能和參數。
export function hookLabels({ hook_event_name: event, source, prompt, tool_name: tool, tool_input: input }) {
  if (event === "UserPromptSubmit") return promptLabels(prompt ?? "");
  if (event === "SessionStart") {
    return source === "clear" || source === "resume" ? ["speclink.change=", "speclink.discussion=", "speclink.skill="] : null;
  }
  if (tool === "Skill") return promptLabels(`/${input?.skill} ${input?.args ?? ""}`);
  if (tool === "Bash") return commandLabels(input?.command ?? "");
  return null;
}

// 建議的下一步。有 worktree 時，程式任務做完就先合併，回主資料夾再跑品質關卡。
// 沒有 worktree 時，兩站都蓋章就是收尾；手動任務還沒做完時還不能封存，交給 apply 處理。
// 沒蓋章時，只剩手動任務的下一步是品質關卡，不是 apply。
function suggestedSkill(row, skills) {
  if (row.restale) return "ingest";
  if (row.worktree && (row.stage === "ready" || row.codeRemaining === 0)) return "worktree-merge";
  const stamped = row.stations?.review === "stamped" && row.stations?.verify === "stamped";
  if (stamped && row.stage === "ready") return "archive+commit";
  if (!stamped && row.stage === "in-progress" && row.codeRemaining === 0) return "quality";
  return skills[0];
}

// 沒被擋、還有程式任務、討論也沒有重新定案，才能開始做。
const canStart = (row) => row.stage !== "ready" && row.codeRemaining !== 0 && row.blockedBy.length === 0 && !row.restale;

// 還沒開始做的變更，可以選在主資料夾或 worktree 做。已有 worktree 時，apply 鈕會接續它。
const choosesWorktree = (row, worktreePolicy) => worktreePolicy && row.stage === "proposed" && !row.worktree && canStart(row);

export function changeSkills(row, worktreePolicy = false) {
  // 有 worktree 就要先合併才能封存。合併不看任務做到哪裡，所以每個階段都能選。
  const stageSkills = row.worktree
    ? [...STAGE_SKILLS[row.stage].filter((skill) => !skill.startsWith("archive")), "worktree-merge"]
    : STAGE_SKILLS[row.stage];
  const skills = choosesWorktree(row, worktreePolicy) ? [stageSkills[0], "apply-with-worktree", ...stageSkills.slice(1)] : stageSkills;
  const suggested = suggestedSkill(row, skills);
  return skills.map((skill) => {
    const waiting = skill === "apply" && row.blockedBy.length > 0;
    return {
      skill,
      args: row.name,
      primary: skill === suggested && !waiting,
      ...(waiting ? { waitingFor: row.blockedBy } : {}),
    };
  });
}

// 列上放建議的那一顆，其他的按 ⋯ 才展開。apply 被擋時沒有建議，放第一顆。
// apply-with-worktree 是 apply 的另一種做法，也放列上。有開工鈕時不放，因為開工已經能選 worktree。
export function rowChoices(actions, start = null) {
  const next = actions.find((a) => a.primary) ?? actions[0];
  const beside = start ? [] : actions.filter((a) => a.skill === "apply-with-worktree");
  return { next, beside, others: actions.filter((a) => a !== next && !beside.includes(a)) };
}

export function discussionSkills(d) {
  const discuss = (primary) => ({ skill: "discuss", args: d.slug, primary });
  const propose = { skill: "propose", args: `--from-discussion ${d.slug}`, primary: true };
  if (d.status === "open") return [discuss(true)];
  if (d.status === "concluded" || d.cuts.length > 0) return [propose, discuss(false)];
  return [discuss(false)];
}

// 疊加的技能和 Claude Code mod 的寫法相同：/speclink-archive + /speclink-commit x。
// agent 只把第一個當指令，後面的是它的參數，模型做完第一個會接著做。
export const commandText = (key, skill, args) => {
  const command = skill.split("+").map((part) => `${PREFIX[key]}${part}`).join(" + ");
  return `${command}${args ? ` ${args}` : ""}`;
};

// `--if idle` 只認 idle：agent 做完一輪後 core 的狀態是 done（Herdr 看過後才顯示 idle），
// 所以 done 改用不帶旗標的 send。不帶旗標時 hc 仍會拒絕 blocked，但 working 會照送，所以
// working 在這裡就要擋下來。
export function sendArgs(agent, text) {
  if (agent.status === "idle") return ["send", "--if", "idle", agent.address, text];
  if (agent.status === "done") return ["send", agent.address, text];
  return null;
}


export const inside = (path, folder) => Boolean(path) && (path === folder || path.startsWith(`${folder}/`));

// agents() 的 done 看過就變 idle，送達要看 core 的原始狀態（terminals()）。
// 這個 agent 在做 board 裡的哪個變更，比對規則和面板的 changeAgents 相同。查不到時是 null。
export function agentChange(agent, board) {
  return Object.values(board.stages).flat().find((row) => changeAgents(row, [agent]).length > 0) ?? null;
}

// 內建 agents mod 的 RANK：等你回答的最急。
const NEEDS_YOU = 4;
const STATUS_ORDER = ["blocked", "done", "working", "idle"];

// Agents 清單的順序：等你回答的不分 space 放最前面，其他照 Spaces 清單的順序。
// 同一個 space 裡，急的在前面，再來是最近變動的。
export function groupedOrder(list, spaces, rankOf, since) {
  const at = new Map(spaces.map((s, i) => [s.id, i]));
  const key = (a) => (rankOf(a) === NEEDS_YOU ? -1 : at.get(a.space) ?? spaces.length);
  return [...list].sort((x, y) => key(x) - key(y) || rankOf(y) - rankOf(x) || since(y) - since(x) || y.seq - x.seq);
}

// 一組 agent 裡各狀態有幾個，最急的在前面，沒有的不列。收起的 space 標題用它。
export function statusCounts(members) {
  return STATUS_ORDER.map((status) => [status, members.filter((a) => a.status === status).length]).filter(([, n]) => n > 0);
}

// 把一個 space 的 agent 依狀態分段。members 已經排好，狀態相同的會相鄰。
export function statusSections(members, rankOf) {
  const sections = [];
  for (const a of members) {
    const rank = rankOf(a);
    if (sections.at(-1)?.rank === rank) sections.at(-1).members.push(a);
    else sections.push({ rank, members: [a] });
  }
  return sections;
}

export function joinAgents(agents, terminals, panes) {
  const byAddress = new Map(terminals.map((t) => [t.address, t]));
  const byPane = new Map(panes.map((p) => [p.pane, p]));
  return agents.map((a) => {
    const t = byAddress.get(a.address);
    return {
      ...a,
      status: t?.status ?? a.status,
      view: a.status,
      name: t?.name ?? null,
      key: a.program?.key,
      cwd: byPane.get(a.pane)?.cwd ?? t?.cwd ?? null,
      labels: t?.labels ?? {},
    };
  });
}

// 正在做這個變更的 agent，依標籤、worktree 路徑、外掛回報的焦點的順序。
// speclink.change 由開工、面板、技能跳窗和 hook 貼上。貼了標籤的 agent 專做那個變更，焦點提到別的變更不算。
// 正在做這場討論的 agent。討論沒有 worktree，只看標籤。
export const discussionAgents = (d, agents) => agents.filter((a) => a.labels["speclink.discussion"] === d.slug);

// 在討論、但討論還沒記錄的 agent：discuss 技能先查資料、問完問題才建立記錄，這段期間沒有那一列可以放它。
// 標籤上的名稱對不到任何一場討論就算，只看這個專案資料夾裡的 agent。
export function unrecordedDiscussions(board, agents, root) {
  const slugs = new Set([...board.discussions, ...board.promoted].map((d) => d.slug));
  return agents.filter(
    (a) => a.labels["speclink.skill"] === "speclink-discuss" && inside(a.cwd, root) && !slugs.has(a.labels["speclink.discussion"]),
  );
}

export function changeAgents(row, agents) {
  const byLabel = agents.filter((a) => a.labels["speclink.change"] === row.name);
  const byPath = row.worktree ? agents.filter((a) => inside(a.cwd, row.worktree.path)) : [];
  const byFocus = agents.filter((a) => !a.labels["speclink.change"] && a.labels["speclink.focus"] === row.name);
  return [...new Set([...byLabel, ...byPath, ...byFocus])];
}

const STATION_SKILLS = {
  "speclink-quality": Object.keys(STATIONS),
  "speclink-review": ["review"],
  "speclink-verify": ["verify"],
};

// 列上要畫的站。agent 正在跑的站是 running（第一輪做完前還沒有工單），蓋好章的站不管 agent 還在不在跑都算蓋好了。
export function stationViews(row, agents) {
  const running = new Set(
    changeAgents(row, agents)
      .filter((a) => a.status === "working")
      .flatMap((a) => STATION_SKILLS[a.labels["speclink.skill"]] ?? []),
  );
  return Object.keys(STATIONS)
    .map((station) => {
      const state = row.stations?.[station] ?? "none";
      return { station, state: state !== "stamped" && running.has(station) ? "running" : state };
    })
    .filter((view) => view.state !== "none");
}

const sendable = (a) => a.status === "idle" || a.status === "done";

function pickAgent({ change, discussion, folder, root, agents, focus }) {
  const pool = agents.filter((a) => Object.hasOwn(PREFIX, a.key));
  if (discussion) {
    const mine = discussionAgents(discussion, pool);
    if (mine.length > 0) return { agent: mine.find(sendable) ?? mine[0], where: "discussion" };
  }
  if (change) {
    const mine = changeAgents(change, pool);
    const here = mine.filter((a) => inside(a.cwd, folder));
    // agent 用 cd 進 worktree 時，終端機回報的資料夾還是主資料夾；標籤已標明它在做這個變更。
    const moved = folder === root ? [] : mine.filter((a) => inside(a.cwd, root));
    const group = here.length > 0 ? here : moved;
    if (group.length > 0) return { agent: group.find(sendable) ?? group[0], where: "change" };
  }
  const tab = pool.filter((a) => a.tab === focus?.tab && inside(a.cwd, folder));
  const current = tab.find((a) => a.address === focus?.terminal) ?? tab[0];
  return current ? { agent: current, where: "tab" } : null;
}

export function planDelivery({ skill, change = null, discussion = null, args, root, agents, focus, known, preferred }) {
  let name = skill;
  let folder = change?.worktree && !MAIN_FOLDER.has(skill) ? change.worktree.path : root;
  let target = pickAgent({ change, discussion, folder, root, agents, focus });
  // 有 worktree 但 agent 不在裡面：apply 改走 apply-with-worktree，它會接續原本的 worktree。
  if (skill === "apply" && change?.worktree && !inside(target?.agent.cwd, folder)) {
    name = "apply-with-worktree";
    folder = root;
    target ??= pickAgent({ change, folder, root, agents, focus });
  }
  if (target) {
    const { agent, where } = target;
    if (!sendable(agent)) return { kind: "busy", where, agent, reason: BUSY[agent.status] ?? agent.status };
    const text = commandText(agent.key, name, args);
    return { kind: "send", where, agent, text, hc: sendArgs(agent, text), labels: skillLabels(name, change?.name ?? discussion?.slug) };
  }
  return newAgent({ name, args, folder, change, discussion, known, preferred });
}

// 已安裝、有 speclink 技能的 agent，Herdr 依最常用排序。
export const usableAgents = (known) => (known ?? []).filter((a) => a.installed && Object.hasOwn(PREFIX, a.key));

// 跳窗一開啟要給哪個 agent：目前分頁的 agent，再來是上次選的，最後是最常用的。
export function defaultAgent({ known, agents, focus, preferred }) {
  const keys = usableAgents(known).map((a) => a.key);
  return [inputAgent(agents, focus)?.key, preferred].find((key) => keys.includes(key)) ?? keys[0] ?? null;
}

// hc --skill：Codex 要帶 --no-daemon，否則它的指令跑在 hc 之外。
const agentArgv = (pick) => (pick.key === "codex" ? [...pick.run, "--no-daemon"] : pick.run);

// 新開一個 agent：選使用者選的那一個，沒有就選最常用的。
function newAgent({ name, args, folder, change, discussion, known, preferred }) {
  if (!known) return { kind: "none", where: "none", reason: tr("還在確認可用的 agent") };
  const usable = usableAgents(known);
  const pick = usable.find((a) => a.key === preferred) ?? usable[0];
  if (!pick) return { kind: "none", where: "none", reason: tr("找不到有 speclink 技能的 agent") };
  return {
    kind: "new",
    where: "new",
    key: pick.key,
    folder,
    argv: agentArgv(pick),
    text: commandText(pick.key, name, args),
    tabName: change || discussion ? `${name} · ${change?.name ?? discussion.slug}` : name,
    change: change?.name ?? null,
    skill: name,
    labels: skillLabels(name, change?.name ?? discussion?.slug),
  };
}

// 開工：開一個專門做這個變更的新 agent，開在目前的工作區。沒被擋、任務還沒做完、沒有 agent 在做才有這個鈕；
// 討論重新定案的變更要先 ingest，不直接開工。回傳可以選的做法，沒有鈕時是 null。
// 已有 worktree 就接續，沒開 worktree 流程就只能直接 apply，這兩種不用問；其他情況要問用 worktree 還是直接 apply。
export function startChoices(row, ctx) {
  if (!canStart(row) || ctx.starting?.has(row.name) || changeAgents(row, ctx.agents).length > 0) return null;
  const target = { kind: "change", name: row.name, args: row.name, row };
  const choice = (label, skill) => ({ label, plan: newTabPlan({ ...ctx, skill, target, note: "" }) });
  if (row.worktree) return [choice("開工", "apply-with-worktree")];
  if (!ctx.worktreePolicy) return [choice("開工", "apply")];
  return [choice("用 worktree", "apply-with-worktree"), choice("直接 apply", "apply")];
}

// 整波開工：最前面那一波能開工的變更。一律用 worktree，因為主資料夾一次只能跑一個 apply。
export function waveToStart(stages, ctx) {
  if (!ctx.worktreePolicy) return null;
  const startable = Object.values(stages)
    .flat()
    .map((row) => ({ wave: row.wave, plan: startChoices(row, ctx)?.[0].plan }))
    .filter((s) => s.plan?.kind === "tab");
  const wave = Math.min(...startable.map((s) => s.wave));
  const plans = startable.filter((s) => s.wave === wave).map((s) => s.plan);
  return plans.length >= 2 ? { wave, plans } : null;
}

const deliveryKey = (d) => d?.agent?.key ?? d?.key;

export function changeActions(row, ctx) {
  return changeSkills(row, ctx.worktreePolicy)
    .map((s) => ({
      ...s,
      delivery: s.waitingFor ? null : planDelivery({ ...ctx, skill: s.skill, change: row, args: s.args }),
    }))
    .filter((a) => !CLAUDE_ONLY.has(a.skill) || !deliveryKey(a.delivery) || deliveryKey(a.delivery) === "claude");
}

export function discussionActions(d, ctx) {
  return discussionSkills(d).map((s) => ({ ...s, delivery: planDelivery({ ...ctx, skill: s.skill, discussion: d, args: s.args }) }));
}

const WHERE = { change: "這個變更", discussion: "這場討論" };
const SEND_TO = { change: "送給這個變更的 agent", discussion: "送給這場討論的 agent" };

// 技能鈕旁邊的說明：會送去哪裡，或為什麼不能送。
export function deliveryHint(delivery) {
  if (delivery.kind === "busy") {
    const where = WHERE[delivery.where];
    return where ? tr("{0}的 agent {1}", tr(where), tr(delivery.reason)) : tr("目前分頁的 agent {0}", tr(delivery.reason));
  }
  if (delivery.kind === "none") return delivery.reason;
  if (delivery.kind === "new") return tr("開新分頁");
  return tr(SEND_TO[delivery.where] ?? "送給目前分頁的 agent");
}

// 技能跳窗可以搭配的變更與討論。哪個階段能用哪個技能，規則和面板的技能鈕相同。
export function skillTargets(skill, board) {
  const name = skill === "apply-with-worktree" ? "apply" : skill;
  const changes = Object.values(board.stages)
    .flat()
    .filter((row) => changeSkills(row).some((s) => s.skill === name))
    .map((row) => ({ kind: "change", name: row.name, args: row.name, row }));
  const discussions = board.discussions.flatMap((d) =>
    discussionSkills(d)
      .filter((s) => s.skill === skill)
      .map((s) => ({ kind: "discussion", name: d.slug, args: s.args, topic: d.topic })),
  );
  return [...changes, ...discussions];
}

const none = (reason, ...args) => ({ kind: "none", reason: tr(reason, ...args) });
export const onlyClaude = (skill, key) => CLAUDE_ONLY.has(skill) && key !== "claude";
const joinArgs = (target, note) => [target?.args, note].filter(Boolean).join(" ");
// 控制字元會被終端機當成按鍵（換行就送出了），一律換成空白。
const pasteable = (text) => text.replace(/[\u0000-\u001f\u007f]/g, " ");
const codeLeft = (skill, change) => (NEEDS_CODE_DONE.has(skill) && change?.codeRemaining > 0 ? change.codeRemaining : 0);

// 放進輸入框的對象：目前分頁的 agent，焦點所在的優先。
function inputAgent(agents, focus) {
  const here = agents.filter((a) => a.tab === focus?.tab && Object.hasOwn(PREFIX, a.key));
  return here.find((a) => a.address === focus?.terminal) ?? here[0] ?? null;
}

// agent 在等回答時不放，貼上可能答掉對話框。在做事可以放，因為不會送出。
const WAITING = "目前分頁的 agent 在等你回答";

// 技能跳窗：把指令放到輸入框。
export function placePlan({ skill, target, note, agents, focus, agentKey }) {
  const left = codeLeft(skill, target?.row);
  if (left) return none("還有 {0} 個程式任務沒做完", left);
  const agent = placeTarget(agents, focus, agentKey);
  if (agent.kind === "none") return agent;
  if (onlyClaude(skill, agent.key)) return none("{0} 只有 Claude Code 能用", skill);
  return {
    kind: "place",
    agent,
    text: pasteable(`${commandText(agent.key, skill, joinArgs(target, note))} `),
    labels: skillLabels(skill, target?.name),
  };
}

// 能放進輸入框的 agent。跳窗選了別的 agent 時不放：指令的寫法和技能都對不上。
function placeTarget(agents, focus, agentKey) {
  const agent = inputAgent(agents, focus);
  if (!agent) return none("目前分頁沒有 agent。可以改開新分頁");
  if (agent.status === "blocked") return none(WAITING);
  if (agentKey && agent.key !== agentKey) return none("目前分頁是 {0}，要放到輸入框請選它", agent.program?.name ?? agent.key);
  return agent;
}

// 面板的附加鈕：把變更或討論的名稱接在輸入框現有的文字後面。讀不到輸入框，所以前後都留空白。
export function attachPlan({ name, agents, focus }) {
  const agent = inputAgent(agents, focus);
  if (!agent) return none("目前分頁沒有 agent");
  if (agent.status === "blocked") return none(WAITING);
  return { kind: "place", agent, text: pasteable(` ${name} `) };
}

// 主資料夾哪個變更的 apply 還沒做完。標籤在分頁關掉前都留著，所以要看變更本身：
// 任務做完或已經封存就不算，不管那個分頁還開著。查不到時是 null。
function unfinishedApply(agents, board) {
  const open = Object.values(board?.stages ?? {}).flat().filter((row) => row.stage !== "ready").map((row) => row.name);
  const applying = agents.filter((a) => a.labels["speclink.skill"] === "speclink-apply").map((a) => a.labels["speclink.change"]);
  return applying.find((name) => open.includes(name)) ?? null;
}

// 開新分頁：做法和檢查都跟 herdr-dispatch 技能相同。分頁開在目前的 space，畫面留在原地。
export function newTabPlan({ skill, target, note, root, agents, known, preferred, worktreePolicy, space, on, board }) {
  const change = target?.kind === "change" ? target.row : null;
  const applying = skill === "apply" || skill === "apply-with-worktree";
  // 只擋正在忙的 agent。標籤在分頁關掉前都留著，做完停著的不算，例如 apply 做完後開新分頁 merge。
  if (change && changeAgents(change, agents).some((a) => !sendable(a))) return none("已經有 agent 在做");
  if (change && applying && change.blockedBy.length > 0) return none("要等 {0} 落地", change.blockedBy.join(tr("、")));
  const left = codeLeft(skill, change);
  if (left) return none("還有 {0} 個程式任務沒做完", left);
  // 有 worktree 的變更，apply 要從主資料夾接續 worktree。
  const name = skill === "apply" && change?.worktree ? "apply-with-worktree" : skill;
  // 政策沒開時這個技能會自己停下來。
  if (name === "apply-with-worktree" && !worktreePolicy) return none("這個專案沒開 worktree 流程");
  // 主資料夾一次只能跑一個 apply，兩個一起跑會互相踩到檔案。
  const unfinished = name === "apply" ? unfinishedApply(agents, board) : null;
  if (unfinished) return none("{0} 的 apply 還沒做完", unfinished);
  const folder = change?.worktree && !MAIN_FOLDER.has(name) ? change.worktree.path : root;
  const discussion = target?.kind === "discussion" ? { slug: target.name } : null;
  const d = newAgent({ name, args: joinArgs(target, note), folder, change, discussion, known, preferred });
  if (d.kind === "none") return d;
  if (onlyClaude(skill, d.key)) return none("{0} 只有 Claude Code 能用", skill);
  return {
    ...d,
    kind: "tab",
    hc: [
      "run",
      "--bg",
      "--json",
      "--no-parent",
      ...(on ? ["--on", on] : []),
      "--cwd",
      folder,
      "--label",
      `herdr.space=${space}`,
      // 只有變更才取名。hc 的名稱不能重複（name_taken），已經被佔用時就不取名。
      ...(change
        ? [
            ...(agents.some((a) => a.name === change.name.slice(0, 32)) ? [] : ["--name", change.name.slice(0, 32)]),
            "--label",
            `speclink.change=${change.name}`,
            "--label",
            skillLabel(name),
          ]
        : []),
      ...(discussion ? ["--label", `speclink.discussion=${discussion.slug}`, "--label", `speclink.skill=speclink-${name}`] : []),
      ...d.argv,
      d.text,
    ],
  };
}

// 一次開多個分頁，一個變更一個分頁。不能開的變更略過，兩個都要在主資料夾跑的 apply 整批擋下。
export function batchPlan({ skill, targets, ...context }) {
  const plans = targets.map((target) => newTabPlan({ ...context, skill, target, note: "" })).filter((p) => p.kind === "tab");
  if (plans.length === 0) return none("勾選要開工的變更");
  if (plans.filter((p) => p.skill === "apply").length > 1) {
    return none("主資料夾一次只能跑一個 apply。可以改選 apply-with-worktree");
  }
  if (plans.filter((p) => p.skill === "archive+commit").length > 1) {
    return none("一次只能跑一個 archive+commit，同時提交會互相影響");
  }
  return { kind: "tabs", plans, text: plans.map((p) => p.text).join("　") };
}

// 這個專案沒有這個 agent 用的 Speclink 技能時，技能鈕送出去也沒有用。
export const missingSkillsPlan = (agentName, key) =>
  none("這個專案沒有 {0} 的 Speclink 技能。先執行 speclink init --tools {1}", agentName, SPECLINK_TOOL[key]);

// 我的技能：使用者自己裝的技能。Speclink 的在另一頁；內建的不是檔案，也不會掃到。

// 每個 agent 讀哪些資料夾，取自各自的文件。外掛另外算，見 pluginFolders。
const SKILL_FOLDERS = {
  claude: [
    ["project", ".claude/skills"],
    ["personal", "~/.claude/skills"],
  ],
  copilot: [["project", ".github/skills"]],
  codex: [
    ["project", ".agents/skills"],
    ["personal", "~/.agents/skills"],
    ["personal", "~/.codex/skills"],
  ],
  pi: [
    ["project", ".pi/skills"],
    ["project", ".agents/skills"],
    ["personal", "~/.pi/agent/skills"],
    ["personal", "~/.agents/skills"],
  ],
};
export const skillFolders = (key) => (SKILL_FOLDERS[key] ?? []).map(([source, dir]) => ({ source, prefix: "", dir }));

// Claude Code 外掛的技能資料夾。快取裡有舊版本，所以照安裝紀錄和啟用設定找，不能掃快取。
// settings 由低到高：使用者、專案、本機。專案範圍的外掛只算這個專案。
export function pluginFolders(installed, settings, root) {
  const enabled = Object.assign({}, ...settings.map((s) => s?.enabledPlugins ?? {}));
  return Object.entries(installed?.plugins ?? {}).flatMap(([key, entries]) => {
    if (!enabled[key]) return [];
    const entry = entries.find((e) => e.scope === "user" || e.projectPath === root);
    return entry ? [{ source: "plugin", prefix: `${key.split("@")[0]}:`, dir: `${entry.installPath}/skills` }] : [];
  });
}

// 依名稱猜用途，畫面用來分組。比對順序照這張表（remotion 含 motion，影片要排在動畫前面），對不到的是「其他」。
const SKILL_USES = [
  ["3d", /three|webgl/],
  ["video", /remotion|video|film|youtube|shotcraft/],
  ["anim", /gsap|motion|cinematic|cinetic|anim/],
  ["content", /social|publish|writing|article|blog/],
  ["design", /design|\bui\b|\bux\b|theme|style|shadcn|image/],
  ["ship", /commit|release|changelog/],
  ["dev", /tdd|test|debug|refactor|lint/],
  ["tool", /skill-creator|dispatch|mcp|automation/],
];
export const skillUse = (name) => SKILL_USES.find(([, pattern]) => pattern.test(name))?.[0] ?? "other";
export const SKILL_GROUPS = [
  { id: "dev", label: "開發", tone: "ansi4" },
  { id: "design", label: "設計", tone: "ansi5" },
  { id: "ship", label: "提交與發版", tone: "ansi2" },
  { id: "video", label: "影片", tone: "ansi1" },
  { id: "anim", label: "動畫", tone: "ansi3" },
  { id: "3d", label: "3D", tone: "ansi6" },
  { id: "content", label: "內容", tone: "foreground" },
  { id: "tool", label: "工具", tone: "dim" },
  { id: "other", label: "其他", tone: "dim" },
];

const SKILL_ABOUT_LIMIT = 200;
const isSpeclinkSkill = (name) => name.startsWith("speclink-");

// 掃到的 SKILL.md 變成清單。不列 Speclink 的和不能手動叫用的，同名的只留第一個。
export function skillEntries(found) {
  const seen = new Set();
  const skills = [];
  for (const x of found) {
    if (isSpeclinkSkill(x.name) || x.invocable === "false" || seen.has(x.name)) continue;
    seen.add(x.name);
    const about = (x.description ?? "").replace(/^[>|][-+]?\s*/, "").replace(/^["']|["']$/g, "").trim();
    skills.push({
      name: x.name,
      source: x.source,
      path: x.source === "plugin" ? x.name.split(":")[0] : x.path,
      about: about.length > SKILL_ABOUT_LIMIT ? `${about.slice(0, SKILL_ABOUT_LIMIT).trimEnd()}…` : about,
      use: skillUse(x.name),
    });
  }
  return skills.sort((a, b) => (a.name < b.name ? -1 : 1));
}

export const hasSpeclinkSkills = (found) => found.some((x) => isSpeclinkSkill(x.name));

// 同一組裡三個以上開頭相同的技能收成一列（系列）。整組都同一系列時不用多一層，但整組太長就預設收合。
const FAMILY_MIN = 3;
const GROUP_OPEN_LIMIT = 12;
const skillStem = (name) => name.split(/[-:]/)[0];

function familiesIn(items) {
  const count = new Map();
  for (const x of items) count.set(skillStem(x.name), (count.get(skillStem(x.name)) ?? 0) + 1);
  return new Set([...count].filter(([, n]) => n >= FAMILY_MIN && n < items.length).map(([stem]) => stem));
}

// 我的技能頁的列。有搜尋字時全部攤開，不分系列。
export function skillRows({ skills, query = "", groupOpen = {}, familyOpen = {} }) {
  const words = query.trim().toLowerCase();
  const matched = words ? skills.filter((x) => `${x.name} ${x.about}`.toLowerCase().includes(words)) : skills;
  const rows = [];
  const skillRow = (skill, nested) => ({ kind: "skill", key: `s:${skill.name}`, skill, nested });
  for (const group of SKILL_GROUPS) {
    const items = matched.filter((x) => x.use === group.id);
    if (items.length === 0) continue;
    const families = words ? new Set() : familiesIn(items);
    const inFamily = (x) => families.has(skillStem(x.name));
    const lines = items.filter((x) => !inFamily(x)).length + families.size;
    const open = words ? true : groupOpen[group.id] ?? lines <= GROUP_OPEN_LIMIT;
    rows.push({ kind: "group", key: `g:${group.id}`, id: group.id, group, count: items.length, open });
    if (!open) continue;
    const done = new Set();
    for (const x of items) {
      if (!inFamily(x)) {
        rows.push(skillRow(x, false));
        continue;
      }
      const stem = skillStem(x.name);
      if (done.has(stem)) continue;
      done.add(stem);
      const members = items.filter((y) => skillStem(y.name) === stem);
      const id = `${group.id}/${stem}`;
      const hint = members.slice(0, 4).map((y) => y.name.slice(stem.length + 1)).join(tr("、"));
      rows.push({ kind: "family", key: `f:${id}`, id, name: stem, count: members.length, open: Boolean(familyOpen[id]), hint: members.length > 4 ? `${hint}…` : hint });
      if (familyOpen[id]) rows.push(...members.map((y) => skillRow(y, true)));
    }
  }
  return rows;
}

const customCommand = (key, name, note) => `${SKILL_PREFIX[key]}${name}${note ? ` ${note}` : ""}`;

export function customPlacePlan({ name, note, agents, focus, agentKey }) {
  const agent = placeTarget(agents, focus, agentKey);
  if (agent.kind === "none") return agent;
  return { kind: "place", agent, text: pasteable(`${customCommand(agent.key, name, note)} `) };
}

export function customTabPlan({ name, note, root, known, agentKey, space, on }) {
  if (!known || !root) return none("還在確認可用的 agent");
  const usable = usableAgents(known);
  const pick = usable.find((a) => a.key === agentKey) ?? usable[0];
  if (!pick) return none("找不到有 speclink 技能的 agent");
  const text = customCommand(pick.key, name, note);
  return {
    kind: "tab",
    key: pick.key,
    text,
    tabName: name,
    hc: ["run", "--bg", "--json", "--no-parent", ...(on ? ["--on", on] : []), "--cwd", root, "--label", `herdr.space=${space}`, ...agentArgv(pick), text],
  };
}
