#!/usr/bin/env node
// 從 build artifacts 物化 server 的 npm 套件（server-release spec「npm 套件一行
// 啟動 server」；設計 D9）：主套件 <scope>/server（以 packages/server-npm 為底）加五個平台
// 子套件 <scope>/server-<os>-<cpu>。物化規則與 CLI 共用（npm-platform-package.mjs），這裡
// 只解析參數。缺任一平台即 fail closed。
//
// 用法：npm-server-package.mjs --version X.Y.Z --binaries <dir> --out <dir> [--scope @scope]

import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { materializePackages } from './npm-platform-package.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function arg(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const version = arg('--version');
const binariesDir = arg('--binaries');
const outDir = arg('--out');
const scope = arg('--scope') || '@speclink';
if (!version || !binariesDir || !outDir) {
  process.stderr.write('用法：npm-server-package.mjs --version X.Y.Z --binaries <dir> --out <dir> [--scope @scope]\n');
  process.exit(1);
}

try {
  const count = materializePackages({
    name: 'server',
    binary: 'speclink-server',
    description: 'Speclink server binary',
    sourceDir: path.join(root, 'packages/server-npm'),
    version,
    binariesDir,
    outDir,
    scope,
  });
  process.stdout.write(`已物化 ${count} 個套件至 ${outDir}（scope ${scope}，版本 ${version}）\n`);
} catch (error) {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
}
