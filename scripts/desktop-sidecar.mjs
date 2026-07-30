// 桌面建置前置：把 speclink CLI binary 佈到 Tauri externalBin 要求的
// target-triple 命名位置（desktop-release spec「桌面安裝檔內含同版 CLI」，design D5）。
//
// 用法：node scripts/desktop-sidecar.mjs [--target <triple>]
//   有 --target：cargo build --release -p speclink-cli --target <triple>（交叉編譯）
//   無 --target：host 編譯，triple 取自 rustc -vV 的 host
// 產出：apps/desktop/src-tauri/binaries/speclink-<triple>[.exe]
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit' });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} 以非零結束（${result.status}）`);
  }
}

function hostTriple() {
  const result = spawnSync('rustc', ['-vV'], { encoding: 'utf8' });
  const host = result.stdout?.match(/^host: (\S+)$/m)?.[1];
  if (!host) throw new Error('無法從 rustc -vV 取得 host triple');
  return host;
}

function main() {
  const argv = process.argv.slice(2);
  const targetIndex = argv.indexOf('--target');
  const target = targetIndex === -1 ? null : argv[targetIndex + 1];
  if (targetIndex !== -1 && !target) throw new Error('--target 後必須接 triple');

  const triple = target ?? hostTriple();
  run('cargo', ['build', '--release', '-p', 'speclink-cli', ...(target ? ['--target', target] : [])]);

  const exe = triple.includes('windows') ? '.exe' : '';
  const built = path.join(root, 'target', ...(target ? [target] : []), 'release', `speclink${exe}`);
  const destDir = path.join(root, 'apps/desktop/src-tauri/binaries');
  mkdirSync(destDir, { recursive: true });
  const dest = path.join(destDir, `speclink-${triple}${exe}`);
  copyFileSync(built, dest);
  console.log(`sidecar 佈署完成：${path.relative(root, dest)}`);
}

try {
  main();
} catch (error) {
  console.error(`desktop-sidecar: ${error.message}`);
  process.exit(1);
}
