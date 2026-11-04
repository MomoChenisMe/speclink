#!/usr/bin/env node
// 複製 Herdr 的 main 版面成 speclink 版面，在最右邊加一欄常駐的 Speclink 面板。
// 也複製 spaces mod，接上階段徽章（integrations/herdr/spaces-badge），因為內建的沒有擴充點。
// 內建版面不能停靠，複製版又不會跟著 Herdr 更新，所以 Herdr 更新後要重跑這支。
// 舊的版面會留在 layouts/speclink.json.old-<時間>，和 Herdr 的 layout.reset 同樣做法；
// 舊的 spaces 複製版直接刪掉，那份資料夾沒有使用者的資料。
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

const SPACES = 'speclink-spaces';
const BADGE = fileURLToPath(new URL('../../integrations/herdr/spaces-badge/stage-badge.js', import.meta.url));
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

const home = process.env.HERDR_APP_HOME ?? path.join(homedir(), '.config/herdr-app');
const file = path.join(home, 'layouts', `${NAME}.json`);
const herdr = (command, args) => execFileSync('herdr-app', ['do', command, JSON.stringify(args)], { stdio: 'pipe' });
const spacesDir = path.join(home, 'mods', SPACES);

// 先改好再動任何東西：找不到要改的行時，現在的版面還完好。
let spacesSource = readFileSync(path.join(home, 'mods/spaces/main.js'), 'utf8');
for (const [from, to] of SPACES_EDITS) {
  if (spacesSource.split(from).length !== 2) {
    throw new Error(`Herdr 的 spaces mod 改了，找不到這一行，這支腳本要跟著改：${from}`);
  }
  spacesSource = spacesSource.replace(from, () => to);
}

// 版面裡 id 是 spaces 的區塊。
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

rmSync(spacesDir, { recursive: true, force: true });
herdr('mod.clone', { name: 'spaces', as: SPACES });
writeFileSync(path.join(spacesDir, 'main.js'), spacesSource);
copyFileSync(BADGE, path.join(spacesDir, 'stage-badge.js'));

herdr('layout.clone', { name: 'main', as: NAME });
const layout = JSON.parse(readFileSync(file, 'utf8'));
if (layout.type !== 'row') {
  throw new Error(`main 版面的最外層不再是 row（現在是 ${layout.type}），這支腳本要跟著改`);
}
const spacesBlock = findBlock(layout, 'spaces');
if (spacesBlock?.content?.mod !== 'spaces') {
  throw new Error('main 版面找不到 spaces 區塊，這支腳本要跟著改');
}
spacesBlock.content.mod = SPACES;
layout.children.push(PANEL);
layout.floating = [...(layout.floating ?? []), COLUMN_SYNC];
writeFileSync(file, `${JSON.stringify(layout, null, 2)}\n`);
console.log(`已停靠：${file}`);
