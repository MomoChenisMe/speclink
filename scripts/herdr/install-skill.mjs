#!/usr/bin/env node
// 把 herdr-speclink 技能複製到 Claude Code 的個人技能資料夾。裝了以後，Herdr 裡的 Claude Code 才知道
// 怎麼用 hc 開分頁派 speclink 的工作，並貼上面板認得的標籤。Claude Code 本身不認得 hc。
// 再跑一次就是更新。移除：刪掉印出來的那個資料夾。
//
// 用法：node scripts/herdr/install-skill.mjs
import { cpSync, existsSync, readFileSync, rmSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const NAME = 'herdr-speclink';
const from = fileURLToPath(new URL(`../../integrations/herdr/skills/${NAME}/`, import.meta.url));
// Claude Code 的設定資料夾可以用 CLAUDE_CONFIG_DIR 換位置。
const to = path.join(process.env.CLAUDE_CONFIG_DIR ?? path.join(homedir(), '.claude'), 'skills', NAME);

const read = (dir) => (existsSync(path.join(dir, 'SKILL.md')) ? readFileSync(path.join(dir, 'SKILL.md'), 'utf8') : null);
const before = read(to);
if (before === read(from)) {
  console.log(`已是最新：${to}`);
} else {
  rmSync(to, { recursive: true, force: true });
  cpSync(from, to, { recursive: true });
  console.log(`${before === null ? '已安裝' : '已更新'}：${to}`);
  console.log('新開的 Claude Code session 會載入它。');
}
