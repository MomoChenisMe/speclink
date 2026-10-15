// 文件守門測試用來對照 help 的 CLI（remote-docs.test.mjs、docs-surface.test.mjs 共用）。
import { existsSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

/// 沒有建出 CLI 時的 skip 理由。
export const NO_BUILT_CLI = '尚未建置 CLI（target/{debug,release} 皆無）';

/// 這個 repo 建出來的 CLI。刻意不走 PATH 上的 `speclink`：那可能是使用者安裝的舊版
/// （於是文件比對的是過期的 help surface），而 CI 上根本沒有，spawn 直接 ENOENT。
/// debug 與 release 都在時取較新的那個：本機開發者平常建的是 debug，但 `cargo test`
/// 會把 target/debug/speclink 換回舊的快取產物；CI 只建 release，於是只有它。
export function builtCli(root = ROOT) {
  const exe = process.platform === 'win32' ? 'speclink.exe' : 'speclink';
  const candidates = ['debug', 'release']
    .map((profile) => path.join(root, 'target', profile, exe))
    .filter((candidate) => existsSync(candidate));
  candidates.sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs);
  return candidates[0] ?? null;
}
