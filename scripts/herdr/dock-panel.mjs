#!/usr/bin/env node
// 複製 Herdr 的 main 版面成 speclink 版面，在最右邊加一欄常駐的 Speclink 面板。
// 也複製 spaces 和 agents 兩個 mod，接上階段徽章和 Speclink 資訊，因為內建的沒有擴充點。
// 內建版面不能停靠，複製版又不會跟著 Herdr 更新，所以 Herdr 更新後要重跑這支。
// 舊的版面會留在 layouts/speclink.json.old-<時間>，和 Herdr 的 layout.reset 同樣做法；
// 舊的 mod 複製版直接刪掉，那些資料夾沒有使用者的資料。
//
// 用法：node scripts/herdr/dock-panel.mjs（Herdr 要開著）
import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const NAME = 'speclink';
// 寬度和 main 內建的 Files／Changes 那一欄相同。
const PANEL = {
  id: 'speclink',
  type: 'block',
  size: { px: 368 },
  min: 280,
  max: 900,
  // 標頭可以拖動視窗，和分頁列一樣；面板內容與按鈕會擋下按壓。
  window_drag: true,
  content: { mod: 'speclink' },
};

// 不畫東西的浮動區塊，負責讓那一欄每個 space 各自記住開關（見 column-sync.js）。
const COLUMN_SYNC = { id: 'speclink-column-sync', content: { mod: 'speclink', props: { view: 'column-sync' } } };

// 接上徽章要改的地方。Herdr 改了這幾行就停下來，不要裝出半套。
const SPACES_EDITS = [
  ['import { View, div, svg } from "gpui-kit";', 'import { View, div, svg } from "gpui-kit";\nimport { rowEnd, watchStages } from "./stage-badge.js";'],
  ['export default class Spaces extends View {\n  init(_props, cx) {', 'export default class Spaces extends View {\n  init(_props, cx) {\n    watchStages(this, cx);'],
  [
    '.child(g ? this.gitTail(g, tree, w.name, t) : div().flex_1())',
    '.child(g ? this.gitTail(g, tree, w.name, t) : div().flex_1())\n          .children(rowEnd(g, tree, t))',
  ],
  // 徽章會佔掉寬度。git 的箭頭搬到 rowEnd，和徽章一起固定在右邊；放不下時讓 space 名字變短，才不會疊在一起。
  [
    '.child(div().flex_shrink(0).max_w(150).truncate().text_color(t.foreground).child(w.name))',
    '.child(div().max_w(150).truncate().text_color(t.foreground).child(w.name))',
  ],
  [
    '.child(where)))\n      .when(g.ahead > 0, (el) => el.child(div().flex_none().text_color(t.ansi2).child(`\\u2191${g.ahead}`)))\n      .when(g.behind > 0, (el) => el.child(div().flex_none().text_color(t.ansi1).child(`\\u2193${g.behind}`)));',
    '.child(where)));',
  ],
];

// Agents 清單要改的地方。規則同上。
const AGENTS_EDITS = [
  [
    'closeOverlay, props, window, layout } from "herdr";',
    'closeOverlay, props, window, layout } from "herdr";\nimport { groupedOrder, statusCounts } from "./model.js";\nimport { changeOf, planMark, watchPlans } from "./speclink-rows.js";',
  ],
  // 預設依 space 分組。時鐘選單還能切回 Priority。
  ['shared("agents.sort")) ?? SORTS[0];', 'shared("agents.sort")) ?? SORTS[1];'],
  ['this.folded = new Set();', 'this.folded = new Set();\n    watchPlans(cx);'],
  // 有自己分頁的 agent 本來就有一列，不再接在開它的 agent 底下。
  ['k.parent === addr && k.state === "live");', 'k.parent === addr && k.state === "live" && !this.listed.has(k.address));'],
  [
    'this.since(y) - this.since(x) || y.seq - x.seq);\n    const pinned',
    'this.since(y) - this.since(x) || y.seq - x.seq);\n    else list = groupedOrder(list, spaces(), rankOf, (a) => this.since(a));\n    const pinned',
  ],
  // 沒取過名的分頁，列上寫在做的變更。
  [
    'return tab ? title(tab, lead(tab)) : a.title || a.program?.name || "";',
    'const change = tab?.named ? null : changeOf(a)?.name;\n      return change ?? (tab ? title(tab, lead(tab)) : a.title || a.program?.name || "");',
  ],
  // 「需要你」那一組沒有 space 標題，所以列上要寫 space。
  ['const full = byPriority || isPinned;', 'const full = byPriority || isPinned || rankOf(a) === RANK.blocked;'],
  ['.child(this.status(t, a, byPriority && !isPinned));', '.child(this.status(t, a, full && !isPinned));'],
  ['.child(`\\u00b7 ${more}`))))', '.child(`\\u00b7 ${more}`)))\n          .children(planMark(a, t)))'],
  // 側欄窄，有進度的列不放時間，名字才看得出是哪個變更。
  ['.when(this.hovered !== a.address && Boolean(when), el =>', '.when(this.hovered !== a.address && Boolean(when) && !changeOf(a), el =>'],
  [
    'const id = byPriority ? `sec-${r}` : `sp-${a.space}`;',
    'const section = byPriority || r === RANK.blocked;\n      const id = section ? `sec-${r}` : `sp-${a.space}`;',
  ],
  [
    'label: byPriority ? SECTION[r] : a.space_name || "", status: byPriority ? SECTION_STATUS[r] : null,',
    'label: byPriority ? SECTION[r] : section ? "需要你" : a.space_name || "", status: section ? SECTION_STATUS[r] : null,',
  ],
  // space 那一組在右邊寫各狀態的數量，其他組照內建寫總數。
  ['      const top = g.members.reduce((m, a) => ((RANK[a.status] ?? 0) > (RANK[m] ?? 0) ? a.status : m), null);\n      return h_flex()\n        .id(g.id)', '      return h_flex()\n        .id(g.id)'],
  ['.when(byPriority || !open, (el) => el.child(div().text_color(t.dim).child(String(g.members.length))))', '.when(!g.id.startsWith("sp-"), (el) => el.child(div().text_color(t.dim).child(String(g.members.length))))'],
  [
    '.when(!open, (el) => el\n          .when(!byPriority && top && top !== "idle", (el) => el.child(this.status(t, { status: top }, true)))\n          .child(svg("mods/workspace-chrome/chevron-right.svg").size(11).flex_none().text_color(t.dim)));',
    '.when(g.id.startsWith("sp-"), (el) => el.children(statusCounts(g.members).map(([status, n]) =>\n          h_flex().gap(3).items_center().text_color(t.dim).child(this.status(t, { status }, true)).child(String(n)))))\n        .when(!open, (el) => el.child(svg("mods/workspace-chrome/chevron-right.svg").size(11).flex_none().text_color(t.dim)));',
  ],
];

const integration = (file) => fileURLToPath(new URL(`../../integrations/herdr/${file}`, import.meta.url));
// 版面裡的區塊換成哪個複製版。files 是複製版裡的檔名對應 repo 裡的檔案。
const CLONES = [
  {
    block: 'spaces',
    as: 'speclink-spaces',
    edits: SPACES_EDITS,
    files: { 'stage-badge.js': 'spaces-badge/stage-badge.js' },
  },
  {
    block: 'agents',
    as: 'speclink-agents',
    edits: AGENTS_EDITS,
    files: { 'speclink-rows.js': 'agents-list/speclink-rows.js', 'model.js': 'mods/speclink/model.js', 'shared.js': 'mods/speclink/shared.js' },
  },
];

const home = process.env.HERDR_APP_HOME ?? path.join(homedir(), '.config/herdr-app');
const file = path.join(home, 'layouts', `${NAME}.json`);
const herdr = (command, args) => execFileSync('herdr-app', ['do', command, JSON.stringify(args)], { stdio: 'pipe' });

// 先改好再動任何東西：找不到要改的行時，現在的版面還完好。
for (const clone of CLONES) {
  clone.source = readFileSync(path.join(home, 'mods', clone.block, 'main.js'), 'utf8');
  for (const [from, to] of clone.edits) {
    if (clone.source.split(from).length !== 2) {
      throw new Error(`Herdr 的 ${clone.block} mod 改了，找不到這一行，這支腳本要跟著改：${from}`);
    }
    clone.source = clone.source.replace(from, () => to);
  }
}

// 版面裡指定 id 的區塊。
function findBlock(node, id) {
  if (node.id === id) return node;
  for (const child of node.children ?? []) {
    const found = findBlock(child, id);
    if (found) return found;
  }
  return null;
}

if (existsSync(file)) {
  // 先切回 main，才不會把顯示中的版面抽走。
  herdr('layout.switch', { name: 'main' });
  renameSync(file, `${file}.old-${Date.now()}`);
}

for (const clone of CLONES) {
  const dir = path.join(home, 'mods', clone.as);
  rmSync(dir, { recursive: true, force: true });
  herdr('mod.clone', { name: clone.block, as: clone.as });
  writeFileSync(path.join(dir, 'main.js'), clone.source);
  for (const [name, from] of Object.entries(clone.files)) copyFileSync(integration(from), path.join(dir, name));
}

herdr('layout.clone', { name: 'main', as: NAME });
const layout = JSON.parse(readFileSync(file, 'utf8'));
if (layout.type !== 'row') {
  throw new Error(`main 版面的最外層不再是 row（現在是 ${layout.type}），這支腳本要跟著改`);
}
for (const clone of CLONES) {
  const block = findBlock(layout, clone.block);
  if (block?.content?.mod !== clone.block) {
    throw new Error(`main 版面找不到 ${clone.block} 區塊，這支腳本要跟著改`);
  }
  block.content.mod = clone.as;
}
layout.children.push(PANEL);
layout.floating = [...(layout.floating ?? []), COLUMN_SYNC];
writeFileSync(file, `${JSON.stringify(layout, null, 2)}\n`);
console.log(`已停靠：${file}`);
