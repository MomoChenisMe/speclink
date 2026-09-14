#!/usr/bin/env node
// 從 build artifacts 物化 CLI 的 npm 套件（cli-distribution spec「CLI 以 npm 套件發布」；
// release-assets-trim 設計 D4）：主套件 @speclink/cli（以 packages/cli-npm 為底）加五個平台
// 子套件 @speclink/cli-<os>-<cpu>。物化規則與 server 共用（npm-platform-package.mjs），這裡
// 只解析參數。缺任一平台即 fail closed，且不留下半套輸出。
//
// 用法：npm-cli-package.mjs --version X.Y.Z --binaries <dir> --out <dir>

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
if (!version || !binariesDir || !outDir) {
  process.stderr.write('用法：npm-cli-package.mjs --version X.Y.Z --binaries <dir> --out <dir>\n');
  process.exit(1);
}

try {
  const count = materializePackages({
    name: 'cli',
    binary: 'speclink',
    description: 'Speclink CLI binary',
    sourceDir: path.join(root, 'packages/cli-npm'),
    version,
    binariesDir,
    outDir,
    scope: '@speclink',
  });
  process.stdout.write(`已物化 ${count} 個套件至 ${outDir}（版本 ${version}）\n`);
} catch (error) {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
}
