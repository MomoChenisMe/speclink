// 第三方授權清單（desktop-about-page design D3）：關於頁「第三方授權」對話框的資料來源，
// 建置時重生、不入版本控制（apps/desktop/package.json 的 predev 與 prebuild 串接本腳本）。
//
// 用法：node scripts/desktop/third-party-licenses.mjs
//   npm 節：package-lock.json 中 apps/desktop 與 packages/ui 的生產相依（遞迴，不含 dev），
//     授權取 lock 條目的 license 欄（npm 寫入 lock 時照抄套件的 package.json）
//   cargo 節：cargo metadata 中非 workspace 成員的套件；cargo 無法執行時警告並輸出空節
// 產出：apps/desktop/src/generated/third-party-licenses.json（{ npm, cargo }，各依名稱排序；
//   授權缺席為 null，由 UI 以介面語言顯示「未標示」）
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

/** 打包進桌面 app 的兩個工作區。 */
const WORKSPACES = ['apps/desktop', 'packages/ui'];

const byNameThenVersion = (a, b) =>
  a.name.localeCompare(b.name) || a.version.localeCompare(b.version, undefined, { numeric: true });

/** 從 `from` 位置解析相依 `name` 的 lock 鍵：由近而遠找 node_modules，找不到為 null。 */
function resolveKey(packages, from, name) {
  let base = from;
  for (;;) {
    const key = base ? `${base}/node_modules/${name}` : `node_modules/${name}`;
    if (packages[key]) return key;
    if (!base) return null;
    const index = base.lastIndexOf('/node_modules/');
    base = index === -1 ? '' : base.slice(0, index);
  }
}

/** lock 中兩個工作區的生產相依（遞迴走 dependencies），工作區連結只穿過、不列入。 */
export function npmLicenses(lock) {
  const packages = lock.packages ?? {};
  const seen = new Set();
  const entries = new Map();
  const visit = (key) => {
    if (seen.has(key)) return;
    seen.add(key);
    const entry = packages[key];
    if (entry.link) return visit(entry.resolved);
    if (key.includes('node_modules/')) {
      const name = key.slice(key.lastIndexOf('node_modules/') + 'node_modules/'.length);
      entries.set(`${name}@${entry.version}`, { name, version: entry.version, license: entry.license ?? null });
    }
    for (const dep of Object.keys(entry.dependencies ?? {})) {
      const depKey = resolveKey(packages, key, dep);
      if (depKey) visit(depKey);
    }
  };
  for (const workspace of WORKSPACES) if (packages[workspace]) visit(workspace);
  return [...entries.values()].sort(byNameThenVersion);
}

/** cargo metadata 中非 workspace 成員的套件。 */
export function cargoLicenses(metadata) {
  const members = new Set(metadata.workspace_members);
  return metadata.packages
    .filter((pkg) => !members.has(pkg.id))
    .map(({ name, version, license }) => ({ name, version, license: license ?? null }))
    .sort(byNameThenVersion);
}

export function buildLicenseList({ lock, metadata }) {
  return { npm: npmLicenses(lock), cargo: metadata ? cargoLicenses(metadata) : [] };
}

/** 執行 cargo metadata；無法執行或非零結束時警告並回 null（建置不因此失敗）。 */
export function readCargoMetadata({ spawn = spawnSync, warn = console.warn } = {}) {
  const args = [
    'metadata',
    '--format-version',
    '1',
    '--manifest-path',
    path.join(root, 'apps/desktop/src-tauri/Cargo.toml'),
  ];
  const result = spawn('cargo', args, { cwd: root, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
  if (result.error || result.status !== 0) {
    const reason = result.error?.message ?? result.stderr?.trim().split('\n')[0] ?? `status ${result.status}`;
    warn(`third-party-licenses: 略過 cargo 節——cargo metadata 失敗（${reason}）`);
    return null;
  }
  return JSON.parse(result.stdout);
}

function main() {
  const lock = JSON.parse(readFileSync(path.join(root, 'package-lock.json'), 'utf8'));
  const list = buildLicenseList({ lock, metadata: readCargoMetadata() });
  const out = path.join(root, 'apps/desktop/src/generated/third-party-licenses.json');
  mkdirSync(path.dirname(out), { recursive: true });
  writeFileSync(out, `${JSON.stringify(list, null, 2)}\n`);
  console.log(`third-party-licenses: npm ${list.npm.length}、cargo ${list.cargo.length} 筆 → ${path.relative(root, out)}`);
}

// node --test 匯入本模組時只取函式，不寫檔。
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main();
}
