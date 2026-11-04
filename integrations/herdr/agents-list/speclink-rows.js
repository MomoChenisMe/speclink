// Agents 清單複製版的 Speclink 部分：每個 agent 在做哪個變更、那個變更做到哪裡。
// 內建的 agents mod 沒有擴充點，所以 dock-panel.mjs 複製一份，把這個檔和面板的 model.js、shared.js
// 放進去並接上。比對用面板的規則，兩邊才不會說法不同。
import { div } from "gpui-kit";
import { h_flex } from "gpui-base";
import { agents, panes, terminals } from "herdr";
import { tr } from "./i18n.js";
import { agentChange, buildBoard, joinAgents } from "./model.js";
import { projectRoot, speclinkJson } from "./shared.js";

const STAGE = {
  proposed: { label: "提案中", tone: "ansi3" },
  "in-progress": { label: "進行中", tone: "ansi4" },
  ready: { label: "已就緒", tone: "ansi2" },
};

// 別台機器的 agent，位址會帶機器名稱（vps/t7）。
const machineOf = (a) => (a.address.includes("/") ? a.address.split("/")[0] : null);
const placeKey = (on, folder) => `${on}\u0000${folder}`;

// 專案資料夾不會變，查過就記住。沒有 Speclink 的專案，board 是 null。
const roots = new Map();
const boards = new Map();

async function look(cx) {
  const projects = new Map();
  for (const a of joinAgents(agents(), terminals(), panes())) {
    if (!a.cwd) continue;
    const on = machineOf(a);
    const place = placeKey(on, a.cwd);
    if (!roots.has(place)) roots.set(place, await projectRoot(a.cwd, on).catch(() => a.cwd));
    const root = roots.get(place);
    projects.set(placeKey(on, root), { on, root });
  }
  for (const [key, { on, root }] of projects) {
    const read = (verb) => speclinkJson([verb], root, on);
    const [list, plan] = await Promise.all([read("list"), read("plan")]).catch(() => [null, null]);
    boards.set(key, list ? buildBoard(list, plan, { discussions: [] }) : null);
  }
  cx.notify();
}

export function watchPlans(cx) {
  cx.timer.after(1000, (cx) => look(cx));
  cx.timer.every(30000, (cx) => look(cx));
}

// 這個 agent 在做的變更。查不到時是 null。
export function changeOf(a) {
  const [agent] = joinAgents([a], terminals(), panes());
  const on = machineOf(a);
  const board = boards.get(placeKey(on, roots.get(placeKey(on, agent.cwd))));
  return board ? agentChange(agent, board) : null;
}

// 列尾的階段顏色和任務進度。回傳陣列，內建的列用 children 接：沒在做變更時什麼都不畫。
export function planMark(a, t) {
  const row = changeOf(a);
  if (!row) return [];
  const { label, tone } = STAGE[row.stage];
  const progress = row.total > 0 ? `${row.completed}/${row.total}` : "";
  return [
    h_flex()
      .id(`plan-${a.address}`)
      .ml_auto()
      .flex_none()
      .gap(3)
      .items_center()
      .text_size(10.5)
      .text_color(t[tone])
      .tooltip(tr("{0}：{1}", row.name, `${tr(label)} ${progress}`.trim()))
      .child(div().size(6).rounded_full().bg(t[tone]))
      .when(Boolean(progress), (el) => el.child(progress)),
  ];
}
