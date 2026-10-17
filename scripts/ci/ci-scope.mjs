import { execFileSync } from 'node:child_process';
import { appendFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// 明確列出的桌面交付設定才可省略應用程式測試；未知路徑一律跑完整 CI。
const DESKTOP_DELIVERY_FILES = new Set([
  '.github/workflows/desktop-build.yml', '.github/workflows/ci.yml',
  'scripts/desktop/desktop-release-version.mjs', 'scripts/desktop/desktop-release.test.mjs',
  'scripts/ci/ci-scope.mjs', 'scripts/ci/ci-scope.test.mjs',
  'docs/development.md', 'docs/development.zh-TW.md', 'docs/development/windows-portable.md',
  'openspec/specs/delivery-baseline/spec.md',
]);

export function requiresFullCI(files) {
  return files.length === 0 || files.some((file) => !DESKTOP_DELIVERY_FILES.has(file));
}

if (resolve(process.argv[1] ?? '') === fileURLToPath(import.meta.url)) {
  let full = true;
  const { BASE_SHA: base, HEAD_SHA: head } = process.env;
  if (base && head && !/^0+$/.test(base)) {
    try {
      const separator = process.env.GITHUB_EVENT_NAME === 'pull_request' ? '...' : '..';
      // 不啟用 rename 偵測，讓移到其他範圍的檔案同時留下舊、新路徑。
      const files = execFileSync('git', ['diff', '--no-renames', '--name-only', '-z', `${base}${separator}${head}`], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] })
        .split('\0').filter(Boolean);
      full = requiresFullCI(files);
    } catch {
      console.warn('無法判斷變更範圍，改跑完整 CI。');
    }
  }
  appendFileSync(process.env.GITHUB_OUTPUT, `full=${full}\n`);
  console.log(full ? '執行完整 CI' : '僅桌面發布設定變更：執行 scripts 檢查');
}
