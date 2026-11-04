#!/usr/bin/env node
// 複製 Herdr 的 main 版面成 speclink 版面，在最右邊加一欄常駐的 Speclink 面板。
// 內建版面不能停靠，複製版又不會跟著 Herdr 更新，所以 Herdr 更新後要重跑這支。
// 舊的複製版會留在 layouts/speclink.json.old-<時間>，和 Herdr 的 layout.reset 同樣做法。
//
// 用法：node scripts/herdr/dock-panel.mjs（Herdr 要開著）
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';

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

const home = process.env.HERDR_APP_HOME ?? path.join(homedir(), '.config/herdr-app');
const file = path.join(home, 'layouts', `${NAME}.json`);
const herdr = (command, args) => execFileSync('herdr-app', ['do', command, JSON.stringify(args)], { stdio: 'pipe' });

if (existsSync(file)) {
  // 先切回 main，才不會把顯示中的版面抽走。
  herdr('layout.switch', { name: 'main' });
  renameSync(file, `${file}.old-${Date.now()}`);
}

herdr('layout.clone', { name: 'main', as: NAME });
const layout = JSON.parse(readFileSync(file, 'utf8'));
if (layout.type !== 'row') {
  throw new Error(`main 版面的最外層不再是 row（現在是 ${layout.type}），這支腳本要跟著改`);
}
layout.children.push(PANEL);
layout.floating = [...(layout.floating ?? []), COLUMN_SYNC];
writeFileSync(file, `${JSON.stringify(layout, null, 2)}\n`);
console.log(`已停靠：${file}`);
