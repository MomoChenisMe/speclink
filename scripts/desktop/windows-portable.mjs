// 在 Windows Tauri release 建置後，整理免安裝包；壓縮與執行檢查由 workflow 負責。
import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function stageWindowsPortable(root) {
  const tauri = path.join(root, 'apps/desktop/src-tauri');
  const config = JSON.parse(readFileSync(path.join(tauri, 'tauri.conf.json'), 'utf8'));
  const release = path.join(root, 'target/x86_64-pc-windows-msvc/release');
  const folder = path.join(release, 'bundle/portable', `Speclink_${config.version}_x64-portable`);
  // 只清理本腳本的產物，避免重跑時把舊檔案帶進 ZIP。
  rmSync(folder, { recursive: true, force: true });
  mkdirSync(folder, { recursive: true });
  writeFileSync(path.join(folder, 'speclink-portable'), 'Speclink portable mode\n');
  // Windows 不分大小寫，GUI 不能改成 Speclink.exe，否則會覆蓋 CLI。
  copyFileSync(path.join(release, 'speclink-desktop.exe'), path.join(folder, 'speclink-desktop.exe'));
  copyFileSync(path.join(tauri, 'binaries/speclink-x86_64-pc-windows-msvc.exe'), path.join(folder, 'speclink.exe'));
  for (const [source, destination] of Object.entries(config.bundle.resources)) {
    const target = path.join(folder, destination);
    mkdirSync(path.dirname(target), { recursive: true });
    copyFileSync(path.join(tauri, source), target);
  }
  copyFileSync(path.join(root, 'LICENSE'), path.join(folder, 'LICENSE'));
  copyFileSync(path.join(root, 'docs/development/windows-portable.md'), path.join(folder, 'README.md'));
  writeFileSync(path.join(folder, 'build-info.json'), `${JSON.stringify({
    version: config.version,
    repository: process.env.GITHUB_REPOSITORY ?? null,
    ref: process.env.GITHUB_REF ?? null,
    commit: process.env.GITHUB_SHA ?? null,
  }, null, 2)}\n`);
  return folder;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(stageWindowsPortable(fileURLToPath(new URL('../../', import.meta.url))));
}
