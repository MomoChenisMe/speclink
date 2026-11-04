// 面板的純邏輯：CLI 輸出怎麼變成面板的列、每一列有哪些技能鈕、技能要送給誰。
// 不碰 Herdr API，scripts/herdr/panel-model.test.mjs 直接測。
//
// 階段、波次、依賴、尚未轉出的刀都取自 CLI 的 --json，這裡不重算。

// 有 speclink 技能的 agent（program.key）與叫用技能的前綴。
const PREFIX = { claude: "/speclink-", copilot: "/speclink-", codex: "$speclink-" };
// sending：hc send 要等 agent 開工才回來（約兩秒），這段時間再按一次會重複送出。
const BUSY = { working: "正在做事", blocked: "在等你回答", error: "出錯了", sending: "正在收指令" };

// 這些技能要在主資料夾跑；其餘技能跟著變更走，有 worktree 就在 worktree 裡跑。
const MAIN_FOLDER = new Set(["archive", "worktree-merge", "apply-with-worktree"]);
const CLAUDE_ONLY = new Set(["analyze"]);

const STAGE_SKILLS = {
  proposed: ["apply", "analyze", "ingest"],
  "in-progress": ["apply", "ingest", "drift", "verify"],
  ready: ["quality", "review", "verify", "archive", "commit"],
  "ready-worktree": ["quality", "review", "verify", "commit", "worktree-merge"],
};
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
  { skill: "worktree-merge", group: "收尾", pair: "變更", about: "把 worktree 合回主分支" },
  { skill: "commit", group: "收尾", pair: "變更", about: "只提交這個變更的檔案" },
  { skill: "improve", group: "專案", pair: null, about: "掃描程式碼找改善點" },
  { skill: "audit", group: "專案", pair: null, about: "找出程式碼的安全問題" },
  { skill: "trace", group: "專案", pair: null, about: "查一項能力的來龍去脈" },
  { skill: "manual", group: "專案", pair: null, about: "從規格產生操作手冊" },
  { skill: "baseline", group: "專案", pair: null, about: "從現有程式碼產生初始規格" },
  { skill: "config", group: "專案", pair: null, about: "整理專案說明與產出規則" },
];

export const ALL_SKILLS = new Set(PICKER_SKILLS.map((w) => w.skill));

export function buildBoard(list, plan, discussions) {
  const listed = new Map(list.changes.map((c) => [c.name, c]));
  const stages = { proposed: [], "in-progress": [], ready: [] };
  for (const c of plan.changes) {
    const l = listed.get(c.name);
    stages[c.stage].push({
      name: c.name,
      stage: c.stage,
      wave: c.wave,
      blockedBy: c.blockedBy,
      completed: l?.completedTasks ?? 0,
      total: l?.totalTasks ?? 0,
      worktree: l?.worktree ?? null,
      restale: (l?.restaleFrom ?? []).length > 0,
    });
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
// discussion 或 roadmap；roadmap 不帶名稱。
export function speclinkUrl(kind, project, name) {
  const path = name ? `/${encodeURIComponent(name)}` : "";
  return `speclink://${kind}${path}?project=${encodeURIComponent(project)}`;
}

export function changeSkills(row) {
  const key = row.stage === "ready" && row.worktree ? "ready-worktree" : row.stage;
  const skills = STAGE_SKILLS[key];
  const shown = row.restale && !skills.includes("ingest") ? ["ingest", ...skills] : skills;
  const suggested = row.restale ? "ingest" : shown[0];
  return shown.map((skill) => {
    const waiting = skill === "apply" && row.blockedBy.length > 0;
    return {
      skill,
      args: row.name,
      primary: skill === suggested && !waiting,
      ...(waiting ? { waitingFor: row.blockedBy } : {}),
    };
  });
}

export function discussionSkills(d) {
  const discuss = (primary) => ({ skill: "discuss", args: d.slug, primary });
  const propose = { skill: "propose", args: `--from-discussion ${d.slug}`, primary: true };
  if (d.status === "open") return [discuss(true)];
  if (d.status === "concluded" || d.cuts.length > 0) return [propose, discuss(false)];
  return [discuss(false)];
}

export const commandText = (key, skill, args) => `${PREFIX[key]}${skill}${args ? ` ${args}` : ""}`;

// `--if idle` 只認 idle：agent 做完一輪後 core 的狀態是 done（Herdr 看過後才顯示 idle），
// 所以 done 改用不帶旗標的 send。不帶旗標時 hc 仍會拒絕 blocked，但 working 會照送，所以
// working 在這裡就要擋下來。
export function sendArgs(agent, text) {
  if (agent.status === "idle") return ["send", "--if", "idle", agent.address, text];
  if (agent.status === "done") return ["send", agent.address, text];
  return null;
}

// Claude Code 外掛把終端機標題設成「<步驟> · <變更>」，封存後多一個 ✓。
export function titleChange(title) {
  return title?.match(/ · (\S+?)(?: ✓)?$/)?.[1] ?? null;
}

export const inside = (path, folder) => Boolean(path) && (path === folder || path.startsWith(`${folder}/`));

// agents() 的 done 看過就變 idle，送達要看 core 的原始狀態（terminals()）。
export function joinAgents(agents, terminals, panes) {
  const byAddress = new Map(terminals.map((t) => [t.address, t]));
  const byPane = new Map(panes.map((p) => [p.pane, p]));
  return agents.map((a) => {
    const t = byAddress.get(a.address);
    return {
      ...a,
      status: t?.status ?? a.status,
      view: a.status,
      key: a.program?.key,
      cwd: byPane.get(a.pane)?.cwd ?? t?.cwd ?? null,
      labels: t?.labels ?? {},
    };
  });
}

// 正在做這個變更的 agent，依開工時貼的標籤、worktree 路徑、外掛回報的焦點、終端機標題的順序。
// 開工時貼了標籤的 agent 專做那個變更，焦點和標題提到別的變更都不算。
// 外掛回報的焦點（speclink.focus）和技能列相同，而且是即時的，有回報就不看標題。標題只有送出訊息時才
// 更新，是舊版外掛或沒重新載入的 session 才用得到的線索；討論和提案的標題多半只是提到那個變更，不算。
export function changeAgents(row, agents) {
  const byLabel = agents.filter((a) => a.labels["speclink.change"] === row.name);
  const byPath = row.worktree ? agents.filter((a) => inside(a.cwd, row.worktree.path)) : [];
  const free = agents.filter((a) => !a.labels["speclink.change"]);
  const byFocus = free.filter((a) => a.labels["speclink.focus"] === row.name);
  const byTitle = free.filter(
    (a) => !a.labels["speclink.focus"] && !/^(discuss|propose) · /.test(a.title ?? "") && titleChange(a.title) === row.name,
  );
  return [...new Set([...byLabel, ...byPath, ...byFocus, ...byTitle])];
}

const sendable = (a) => a.status === "idle" || a.status === "done";

function pickAgent({ change, folder, root, agents, focus }) {
  const pool = agents.filter((a) => Object.hasOwn(PREFIX, a.key));
  if (change) {
    const mine = changeAgents(change, pool);
    const here = mine.filter((a) => inside(a.cwd, folder));
    // agent 用 cd 進 worktree 時，終端機回報的資料夾還是主資料夾；標籤與標題已標明它在做這個變更。
    const moved = folder === root ? [] : mine.filter((a) => inside(a.cwd, root));
    const group = here.length > 0 ? here : moved;
    if (group.length > 0) return { agent: group.find(sendable) ?? group[0], where: "change" };
  }
  const tab = pool.filter((a) => a.tab === focus?.tab && inside(a.cwd, folder));
  const current = tab.find((a) => a.address === focus?.terminal) ?? tab[0];
  return current ? { agent: current, where: "tab" } : null;
}

export function planDelivery({ skill, change = null, args, root, agents, focus, known }) {
  let name = skill;
  let folder = change?.worktree && !MAIN_FOLDER.has(skill) ? change.worktree.path : root;
  let target = pickAgent({ change, folder, root, agents, focus });
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
    return { kind: "send", where, agent, text, hc: sendArgs(agent, text) };
  }
  return newAgent({ name, args, folder, change, known });
}

// 新開一個 agent：選最常用且已安裝、有 speclink 技能的那一個。
function newAgent({ name, args, folder, change, known }) {
  if (!known) return { kind: "none", where: "none", reason: "還在確認可用的 agent" };
  const pick = known.find((a) => a.installed && Object.hasOwn(PREFIX, a.key));
  if (!pick) return { kind: "none", where: "none", reason: "找不到有 speclink 技能的 agent" };
  return {
    kind: "new",
    where: "new",
    key: pick.key,
    folder,
    // hc --skill：Codex 要帶 --no-daemon，否則它的指令跑在 hc 之外。
    argv: pick.key === "codex" ? [...pick.run, "--no-daemon"] : pick.run,
    text: commandText(pick.key, name, args),
    tabName: change ? `${name} · ${change.name}` : name,
    change: change?.name ?? null,
    skill: name,
  };
}

// 開工：開一個專門做這個變更的新 agent。沒被擋、任務還沒做完、沒有 agent 在做才有這個鈕；
// 討論重新定案的變更要先 ingest，不直接開工。開 worktree 政策時開新 space，技能在主資料夾
// 建好或接續 worktree；沒開時在主資料夾開新分頁跑 apply。
export function startAction(row, ctx) {
  const open = row.stage !== "ready" && row.blockedBy.length === 0 && !row.restale;
  if (!open || ctx.starting?.has(row.name) || changeAgents(row, ctx.agents).length > 0) return null;
  const name = ctx.worktreePolicy ? "apply-with-worktree" : "apply";
  const delivery = newAgent({ name, args: row.name, folder: ctx.root, change: row, known: ctx.known });
  return {
    skill: "開工",
    primary: false,
    delivery: ctx.worktreePolicy && delivery.kind === "new" ? { ...delivery, kind: "space", where: "space" } : delivery,
  };
}

// 整波開工：最前面那一波裡能開工的變更。波次由引擎決定，這裡只分組。不到兩個就不必整波。
export function waveToStart(stages, ctx) {
  if (!ctx.worktreePolicy) return null;
  const startable = Object.values(stages)
    .flat()
    .map((row) => ({ wave: row.wave, delivery: startAction(row, ctx)?.delivery }))
    .filter((s) => s.delivery?.kind === "space");
  const wave = Math.min(...startable.map((s) => s.wave));
  const deliveries = startable.filter((s) => s.wave === wave).map((s) => s.delivery);
  return deliveries.length >= 2 ? { wave, deliveries } : null;
}

const deliveryKey = (d) => d?.agent?.key ?? d?.key;

export function changeActions(row, ctx) {
  return changeSkills(row)
    .map((s) => ({
      ...s,
      delivery: s.waitingFor ? null : planDelivery({ ...ctx, skill: s.skill, change: row, args: s.args }),
    }))
    .filter((a) => !CLAUDE_ONLY.has(a.skill) || !deliveryKey(a.delivery) || deliveryKey(a.delivery) === "claude");
}

export function discussionActions(d, ctx) {
  return discussionSkills(d).map((s) => ({ ...s, delivery: planDelivery({ ...ctx, skill: s.skill, args: s.args }) }));
}

// 技能鈕旁邊的說明：會送去哪裡，或為什麼不能送。
export function deliveryHint(delivery) {
  if (delivery.kind === "busy") return `${delivery.where === "change" ? "這個變更" : "目前分頁"}的 agent ${delivery.reason}`;
  if (delivery.kind === "none") return delivery.reason;
  if (delivery.kind === "new") return "開新分頁";
  if (delivery.kind === "space") return "開新 space";
  return delivery.where === "change" ? "送給這個變更的 agent" : "送給目前分頁的 agent";
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

const none = (reason) => ({ kind: "none", reason });
const onlyClaude = (skill, key) => CLAUDE_ONLY.has(skill) && key !== "claude";
const joinArgs = (target, note) => [target?.args, note].filter(Boolean).join(" ");
// hc keys 打得出來的字元。打不出來的有中文、大寫和 Codex 的 $。
const TYPABLE = /^[a-z0-9/. -]+$/;

// 放進輸入框的對象：目前分頁的 agent，焦點所在的優先。
function inputAgent(agents, focus) {
  const here = agents.filter((a) => a.tab === focus?.tab && Object.hasOwn(PREFIX, a.key));
  return here.find((a) => a.address === focus?.terminal) ?? here[0] ?? null;
}

function typeInto(agent, text) {
  const plan = { kind: "place", agent, text };
  if (TYPABLE.test(text)) {
    return { ...plan, hc: ["keys", agent.address, ...[...text].map((c) => (c === " " ? "space" : c))] };
  }
  // 其他字元只有 hc send 貼得進去，而它一定按 Enter。Claude Code 把結尾的「\」加 Enter 當成換行，
  // 不會送出（閒置和做事中都實測過）。輸入框會多一個空行。不再按退格去掉它：退格萬一排到 Enter 之前，
  // 刪掉的是「\」，文字就送出去了。
  if (agent.key === "claude") return { ...plan, hc: ["send", agent.address, `${text}\\`] };
  return none("這段文字有打不出來的字元");
}

// agent 在等回答時不放，打字可能答掉對話框。在做事可以放，因為不會送出。
const WAITING = "目前分頁的 agent 在等你回答";
const TRY_NEW_TAB = "。可以勾選「開新分頁」";

// 技能跳窗：把指令放到輸入框。
export function placePlan({ skill, target, note, agents, focus }) {
  const agent = inputAgent(agents, focus);
  if (!agent) return none(`目前分頁沒有 agent${TRY_NEW_TAB}`);
  if (agent.status === "blocked") return none(WAITING);
  if (onlyClaude(skill, agent.key)) return none(`${skill} 只有 Claude Code 能用`);
  const plan = typeInto(agent, `${commandText(agent.key, skill, joinArgs(target, note))} `);
  return plan.kind === "none" ? none(`${plan.reason}${TRY_NEW_TAB}`) : plan;
}

// 面板的附加鈕：把變更或討論的名稱接在輸入框現有的文字後面。讀不到輸入框，所以前後都留空白。
export function attachPlan({ name, agents, focus }) {
  const agent = inputAgent(agents, focus);
  if (!agent) return none("目前分頁沒有 agent");
  if (agent.status === "blocked") return none(WAITING);
  return typeInto(agent, ` ${name} `);
}

// 開新分頁：做法和檢查都跟 herdr-dispatch 技能相同。分頁開在目前的 space，畫面留在原地。
export function newTabPlan({ skill, target, note, root, agents, known, worktreePolicy, space, on }) {
  const change = target?.kind === "change" ? target.row : null;
  const applying = skill === "apply" || skill === "apply-with-worktree";
  if (change && changeAgents(change, agents).length > 0) return none("已經有 agent 在做");
  if (change && applying && change.blockedBy.length > 0) return none(`要等 ${change.blockedBy.join("、")} 落地`);
  // 有 worktree 的變更，apply 要從主資料夾接續 worktree。
  const name = skill === "apply" && change?.worktree ? "apply-with-worktree" : skill;
  // 政策沒開時這個技能會自己停下來。
  if (name === "apply-with-worktree" && !worktreePolicy) return none("這個專案沒開 worktree 流程");
  // 主資料夾一次只能跑一個 apply，兩個一起跑會互相踩到檔案。
  if (name === "apply" && agents.some((a) => a.labels["speclink.skill"] === "speclink-apply")) {
    return none("主資料夾已經有 apply 在跑");
  }
  const folder = change?.worktree && !MAIN_FOLDER.has(name) ? change.worktree.path : root;
  const d = newAgent({ name, args: joinArgs(target, note), folder, change, known });
  if (d.kind === "none") return d;
  if (onlyClaude(skill, d.key)) return none(`${skill} 只有 Claude Code 能用`);
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
      // hc 的名稱不能重複，所以只有變更才取名：同一個變更只會有一個 agent。
      ...(change
        ? ["--name", change.name.slice(0, 32), "--label", `speclink.change=${change.name}`, "--label", `speclink.skill=speclink-${name}`]
        : []),
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
  return { kind: "tabs", plans, text: plans.map((p) => p.text).join("　") };
}
