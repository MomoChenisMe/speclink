// 「主套件＋五平台子套件」的 npm 物化共用層：CLI（cli-distribution spec「CLI 以 npm 套件發布」；
// release-assets-trim 設計 D4）與 server（server-release spec「npm 套件一行啟動 server」；
// 設計 D9）的套件形狀相同——repo 內只維護帶 private 的主套件，發布時產出去掉 private、蓋上
// 版本、補 optionalDependencies 的主套件副本，以及五個只含對應平台 binary 的子套件。兩支
// 入口腳本（npm-cli-package.mjs、npm-server-package.mjs）只負責解析參數，物化規則在這裡一份。
//
// 缺任一平台 binary 或版號不合形即丟錯，且在寫任何檔之前——不留下半套輸出。

import { chmodSync, copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

// release build artifact（<name>-<target>，內容為 raw binary）→ npm 平台子套件（os／cpu）的對映契約。
const TARGETS = [
  { target: 'aarch64-apple-darwin', os: 'darwin', cpu: 'arm64' },
  { target: 'x86_64-apple-darwin', os: 'darwin', cpu: 'x64' },
  { target: 'x86_64-unknown-linux-gnu', os: 'linux', cpu: 'x64' },
  { target: 'aarch64-unknown-linux-gnu', os: 'linux', cpu: 'arm64' },
  { target: 'x86_64-pc-windows-msvc', os: 'win32', cpu: 'x64' },
];

/// 物化套件。name 是套件家族名（cli／server）：artifact 目錄叫 <name>-<target>，主套件叫
/// <scope>/<name>，子套件叫 <scope>/<name>-<os>-<cpu>。binary 是 binary 的基底檔名（Windows
/// 自動加 .exe）。sourceDir 是 repo 內的主套件目錄，其 package.json 的 files 就是要一併帶進
/// 副本的檔案清單（bin 內的檔設為可執行）。回傳物化的套件數。
export function materializePackages({ name, binary, description, sourceDir, version, binariesDir, outDir, scope }) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error(`版本「${version}」不符 X.Y.Z 格式（不帶 v 前綴）`);

  const binaryFor = (os) => (os === 'win32' ? `${binary}.exe` : binary);
  const artifactBinary = ({ target, os }) => path.join(binariesDir, `${name}-${target}`, binaryFor(os));

  // fail closed：先驗五個 binary 全數到位，再開始物化。
  const missing = TARGETS.filter((entry) => {
    try {
      readFileSync(artifactBinary(entry));
      return false;
    } catch {
      return true;
    }
  });
  if (missing.length > 0) throw new Error(`缺少平台 binary：${missing.map((m) => m.target).join('、')}`);

  // 平台子套件：os/cpu 圈定，內容物只有 binary。
  for (const entry of TARGETS) {
    const { os, cpu } = entry;
    const pkgDir = path.join(outDir, `${name}-${os}-${cpu}`);
    mkdirSync(pkgDir, { recursive: true });
    writeJson(path.join(pkgDir, 'package.json'), {
      name: `${scope}/${name}-${os}-${cpu}`,
      version,
      description: `${description} for ${os}/${cpu}`,
      license: 'MIT',
      os: [os],
      cpu: [cpu],
    });
    const dest = path.join(pkgDir, binaryFor(os));
    copyFileSync(artifactBinary(entry), dest);
    chmodSync(dest, 0o755);
  }

  // 主套件副本：以 repo 的主套件為底，去 private 與說明欄、蓋版本與 scope、補五組同版
  // optionalDependencies；files 列的檔原樣帶上。
  const source = JSON.parse(readFileSync(path.join(sourceDir, 'package.json'), 'utf8'));
  delete source.private;
  delete source['//'];
  source.name = `${scope}/${name}`;
  source.version = version;
  source.optionalDependencies = Object.fromEntries(TARGETS.map(({ os, cpu }) => [`${scope}/${name}-${os}-${cpu}`, version]));
  const mainDir = path.join(outDir, name);
  mkdirSync(mainDir, { recursive: true });
  writeJson(path.join(mainDir, 'package.json'), source);
  for (const file of source.files) {
    const dest = path.join(mainDir, file);
    mkdirSync(path.dirname(dest), { recursive: true });
    copyFileSync(path.join(sourceDir, file), dest);
    if (file.startsWith('bin/')) chmodSync(dest, 0o755);
  }

  return TARGETS.length + 1;
}

function writeJson(file, value) {
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}
