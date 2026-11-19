#!/usr/bin/env node
// npm run clean:target：清掉本機累積的 Rust 編譯產物，再補回 debug CLI。
// 存在的理由：target/ 只增不減（兩週約 50 GB，其中三分之二是增量編譯的快取），
// cargo 沒有內建的清理。發版後版號一升，workspace 的 crate 全部要重編、舊產物
// 全部作廢，所以 /release 的收尾固定跑這支。
//
// 清兩個資料夾：
//   - target/                               由 cargo clean 清
//   - crates/adapters/speclink-node/target/ 該目錄的 .cargo/config.toml 把 napi 的
//                                           產物隔離在這裡，根目錄的 cargo clean 清不到
// 清完跑 cargo build -p speclink-cli：scripts/ 的文件守門會讀 target/debug/speclink。
//
// 有 cargo、rustc 或 tauri 在跑時不清任何東西、以非零結束：清掉別的 session 正在
// 用的產物會讓它編譯失敗。只支援 macOS 與 Linux（用到 ps 與 du）。
import { spawnSync } from 'node:child_process';
import { rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const NODE_TARGET = path.join('crates', 'adapters', 'speclink-node', 'target');

/// `ps -axo comm=` 的輸出裡正在編譯的程序（cargo、rustc、tauri），回傳它們的原字串。
/// 只比對執行檔的檔名：rust-analyzer 常駐不算，它叫出來的 cargo check 才算。
export function busyBuildProcesses(psOutput) {
  return psOutput
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => ['cargo', 'rustc', 'tauri'].includes(path.basename(line)));
}

/// KiB 轉成人看的大小：1 GiB 以上用 GB（一位小數），以下用 MB（整數）。
export function formatSize(kib) {
  return kib >= 1024 * 1024 ? `${(kib / 1024 / 1024).toFixed(1)} GB` : `${Math.round(kib / 1024)} MB`;
}

function sizeKib(dir) {
  const out = spawnSync('du', ['-sk', dir], { cwd: ROOT, encoding: 'utf8' });
  return out.status === 0 ? Number(out.stdout.split('\t')[0]) : 0; // 資料夾不存在＝0
}

function run(command, args) {
  const out = spawnSync(command, args, { cwd: ROOT, stdio: 'inherit' });
  if (out.status !== 0) {
    console.error(`${command} ${args.join(' ')} 失敗（exit ${out.status}）`);
    process.exit(1);
  }
}

function main() {
  const ps = spawnSync('ps', ['-axo', 'comm='], { encoding: 'utf8' });
  if (ps.status !== 0) {
    console.error('讀不到程序清單（ps 失敗），不清理');
    process.exit(1);
  }
  const busy = busyBuildProcesses(ps.stdout);
  if (busy.length > 0) {
    console.error(`有編譯正在進行，不清理：${[...new Set(busy)].join('、')}\n等它結束後再跑一次。`);
    process.exit(1);
  }

  const before = sizeKib('target') + sizeKib(NODE_TARGET);
  console.log(`清理前：${formatSize(before)}（target ＋ ${NODE_TARGET}）`);

  run('cargo', ['clean']);
  // Finder 可能在刪除途中補寫 .DS_Store，讓 rm 報 Directory not empty；重試即可。
  rmSync(path.join(ROOT, NODE_TARGET), { recursive: true, force: true, maxRetries: 3, retryDelay: 200 });
  run('cargo', ['build', '-p', 'speclink-cli']);

  const after = sizeKib('target') + sizeKib(NODE_TARGET);
  console.log(`清理後：${formatSize(after)}，釋放 ${formatSize(before - after)}。debug CLI 已重建。`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) main();
