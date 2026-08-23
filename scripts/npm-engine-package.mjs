#!/usr/bin/env node
// engine 的 npm 發布版號蓋章（node-sdk-release spec「engine npm 套件家族與版號蓋章」；
// 設計 D2）。repo 內 crates/speclink-node/package.json 的版號只是佔位符；發布版本一律
// 由 release tag 決定，於 CI 的拋棄式 checkout 上蓋章：主套件與 napi create-npm-dir
// 產生的每個平台子套件同版，主套件 optionalDependencies 物化為各子套件名釘同版。
// 平台子套件清單自 npm/ 目錄實際內容列舉（napi.triples 是唯一來源），不硬編碼 triple。

import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

function arg(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const version = arg('--version');
const dir = arg('--dir');
if (!version || !dir) {
  fail('用法：npm-engine-package.mjs --version X.Y.Z --dir <套件根>');
}
if (!/^\d+\.\d+\.\d+$/.test(version)) fail(`版本「${version}」不符 X.Y.Z 格式（不帶 v 前綴）`);

function readPkg(file) {
  return JSON.parse(readFileSync(file, 'utf8'));
}

function writePkg(file, pkg) {
  writeFileSync(file, `${JSON.stringify(pkg, null, 2)}\n`);
}

// fail closed：沒有平台子套件目錄代表 napi create-npm-dir 未跑，此時發布的主套件會
// 帶空 optionalDependencies——安裝得到沒有 binary 的套件，比紅燈更糟。
const npmDir = path.join(dir, 'npm');
let platforms;
try {
  platforms = readdirSync(npmDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
} catch {
  fail(`找不到平台子套件目錄 ${npmDir}——napi create-npm-dir 尚未執行？`);
}
if (platforms.length === 0) fail(`${npmDir} 沒有任何平台子套件目錄`);

// 平台子套件：只蓋版號，名稱與 os／cpu 欄位維持 napi 產出的原樣。
const subNames = platforms.map((platform) => {
  const file = path.join(npmDir, platform, 'package.json');
  const pkg = readPkg(file);
  pkg.version = version;
  writePkg(file, pkg);
  return pkg.name;
});

// 主套件：蓋版號並把 optionalDependencies 整組換成本次列舉到的子套件（舊清單不留殘留）。
// 佔位符註解只對 repo 內的版號成立，發布產物不帶（與 npm-server-package.mjs 同慣例）。
const mainFile = path.join(dir, 'package.json');
const main = readPkg(mainFile);
delete main['//'];
main.version = version;
main.optionalDependencies = Object.fromEntries(subNames.map((name) => [name, version]));
writePkg(mainFile, main);

process.stdout.write(`已蓋章 ${main.name} 與 ${subNames.length} 個平台子套件為 ${version}\n`);
