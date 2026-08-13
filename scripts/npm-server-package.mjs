#!/usr/bin/env node
// 從 build artifacts 物化 server 的 npm 套件（server-release spec「npm 套件一行
// 啟動 server」；設計 D9）。repo 內只維護主套件（packages/server-npm，帶 private
// 擋誤發布）；本腳本在發布時產出去掉 private、蓋上版本、補 optionalDependencies
// 的主套件副本，以及五個只含對應平台 binary 的子套件。缺任一平台即 fail closed。

import { chmodSync, copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

// release build artifact（server-<target>）→ npm 平台子套件的對映契約。
const TARGETS = [
  { target: 'aarch64-apple-darwin', os: 'darwin', cpu: 'arm64', binary: 'speclink-server' },
  { target: 'x86_64-apple-darwin', os: 'darwin', cpu: 'x64', binary: 'speclink-server' },
  { target: 'x86_64-unknown-linux-gnu', os: 'linux', cpu: 'x64', binary: 'speclink-server' },
  { target: 'aarch64-unknown-linux-gnu', os: 'linux', cpu: 'arm64', binary: 'speclink-server' },
  { target: 'x86_64-pc-windows-msvc', os: 'win32', cpu: 'x64', binary: 'speclink-server.exe' },
];

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

function arg(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const version = arg('--version');
const binariesDir = arg('--binaries');
const outDir = arg('--out');
const scope = arg('--scope') || '@speclink';
if (!version || !binariesDir || !outDir) {
  fail('用法：npm-server-package.mjs --version X.Y.Z --binaries <dir> --out <dir> [--scope @scope]');
}
if (!/^\d+\.\d+\.\d+$/.test(version)) fail(`版本「${version}」不符 X.Y.Z 格式（不帶 v 前綴）`);

// fail closed：先驗五個 binary 全數到位，再開始物化——不產生缺平台的半套輸出。
const missing = TARGETS.filter(
  ({ target, binary }) => {
    try {
      readFileSync(path.join(binariesDir, `server-${target}`, binary));
      return false;
    } catch {
      return true;
    }
  },
);
if (missing.length > 0) {
  fail(`缺少平台 binary：${missing.map((m) => m.target).join('、')}`);
}

// 平台子套件：os/cpu 圈定，內容物只有 binary。
for (const { target, os, cpu, binary } of TARGETS) {
  const pkgName = `server-${os}-${cpu}`;
  const pkgDir = path.join(outDir, pkgName);
  mkdirSync(pkgDir, { recursive: true });
  writeFileSync(
    path.join(pkgDir, 'package.json'),
    `${JSON.stringify(
      {
        name: `${scope}/${pkgName}`,
        version,
        description: `Speclink server binary for ${os}/${cpu}`,
        license: 'MIT',
        os: [os],
        cpu: [cpu],
      },
      null,
      2,
    )}\n`,
  );
  const dest = path.join(pkgDir, binary);
  copyFileSync(path.join(binariesDir, `server-${target}`, binary), dest);
  chmodSync(dest, 0o755);
}

// 主套件副本：以 repo 的 packages/server-npm 為底，去 private、蓋版本與 scope、
// 補五組同版 optionalDependencies。
const source = JSON.parse(readFileSync(path.join(root, 'packages/server-npm/package.json'), 'utf8'));
delete source.private;
delete source['//'];
source.name = `${scope}/server`;
source.version = version;
source.optionalDependencies = Object.fromEntries(
  TARGETS.map(({ os, cpu }) => [`${scope}/server-${os}-${cpu}`, version]),
);
const mainDir = path.join(outDir, 'server');
mkdirSync(path.join(mainDir, 'bin'), { recursive: true });
writeFileSync(path.join(mainDir, 'package.json'), `${JSON.stringify(source, null, 2)}\n`);
copyFileSync(
  path.join(root, 'packages/server-npm/bin/speclink-server.mjs'),
  path.join(mainDir, 'bin/speclink-server.mjs'),
);

process.stdout.write(`已物化 ${TARGETS.length + 1} 個套件至 ${outDir}（scope ${scope}，版本 ${version}）\n`);
