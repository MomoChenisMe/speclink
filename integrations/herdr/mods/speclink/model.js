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
const NEW_WORK = ["discuss", "propose", "improve", "config", "audit", "trace", "manual", "baseline"];

// 全部技能都要有放處。apply-with-worktree 沒有自己的鈕：apply 鈕接續 worktree 時才送它。
export const ALL_SKILLS = new Set([...Object.values(STAGE_SKILLS).flat(), ...NEW_WORK, "apply-with-worktree"]);

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

// 正在做這個變更的 agent，依標籤、worktree 路徑、終端機標題的順序。
export function changeAgents(row, agents) {
  const byLabel = agents.filter((a) => a.labels["speclink.change"] === row.name);
  const byPath = row.worktree ? agents.filter((a) => inside(a.cwd, row.worktree.path)) : [];
  const byTitle = agents.filter((a) => titleChange(a.title) === row.name);
  return [...new Set([...byLabel, ...byPath, ...byTitle])];
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

export function newWorkActions(ctx) {
  return NEW_WORK.map((skill) => ({ skill, primary: false, delivery: planDelivery({ ...ctx, skill }) }));
}

// 技能鈕旁邊的說明：會送去哪裡，或為什麼不能送。
export function deliveryHint(delivery) {
  if (delivery.kind === "busy") return `agent ${delivery.reason}`;
  if (delivery.kind === "none") return delivery.reason;
  if (delivery.kind === "new") return "開新分頁";
  if (delivery.kind === "space") return "開新 space";
  return delivery.where === "change" ? "送給這個變更的 agent" : "送給目前分頁的 agent";
}
