import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const zhPath = path.join(root, 'docs/remote-getting-started.zh-TW.md');
const enPath = path.join(root, 'docs/remote-getting-started.md');

function read(relativePath) {
  return readFileSync(path.join(root, relativePath), 'utf8');
}

function h2s(markdown) {
  return [...markdown.matchAll(/^## (.+)$/gm)].map((match) => match[1]);
}

function localMarkdownLinks(markdown) {
  return [...markdown.matchAll(/!?\[[^\]]*]\(([^)]+)\)/g)]
    .map((match) => match[1].trim().replace(/^<|>$/g, ''))
    .filter((target) => !/^(?:https?:|mailto:|#)/.test(target));
}

function literal(text) {
  return new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
}

/// 取出以 prefix 開頭的 h2 章節內文，讓斷言綁在該章節而不是整份文件。
function sectionStartingWith(markdown, prefix, relativePath) {
  const section = markdown.split(/^## /m).slice(1).find((body) => body.startsWith(prefix));
  assert.ok(section, `${relativePath}: 找不到以「${prefix}」開頭的章節`);
  return section;
}

const remoteGuides = [
  'docs/remote-getting-started.zh-TW.md',
  'docs/remote-getting-started.md',
];

test('remote getting-started guides exist with matching section order', () => {
  assert.equal(existsSync(zhPath), true, 'missing Traditional Chinese remote guide');
  assert.equal(existsSync(enPath), true, 'missing English remote guide');

  const zh = read('docs/remote-getting-started.zh-TW.md');
  const en = read('docs/remote-getting-started.md');
  assert.deepEqual(h2s(zh), h2s(en));
});

test('development guides exist with matching section order', () => {
  const zh = read('docs/development.zh-TW.md');
  const en = read('docs/development.md');
  assert.deepEqual(h2s(zh), h2s(en));

  // 五個一鍵入口各有一節（規格「開發者入口文件雙語對」）。
  for (const entry of [
    'npm run dev`',
    'npm run dev:server`',
    'npm run dev:desktop`',
    'npm run dev:reset`',
    'npm run cli -- <args>`',
  ]) {
    assert.equal(
      h2s(zh).some((heading) => heading.includes(entry)),
      true,
      `development 文件缺少 ${entry} 章節`,
    );
  }
});

test('both guides cover the remote setup, authorization, and recovery contract', () => {
  for (const relativePath of remoteGuides) {
    const guide = read(relativePath);
    for (const required of [
      '/account',
      'POST `/api/speclink/v1/web/account/tokens`',
      '/admin/users',
      '404',
      'membership',
      'project-scoped URL',
      'spec-only',
      'checkout',
      'offline',
      'npm run dev:reset',
    ]) {
      assert.match(guide, literal(required));
    }
  }
});

test('existing user entry points link to the matching remote guide', () => {
  assert.match(read('README.md'), /docs\/remote-getting-started\.zh-TW\.md/);
  assert.match(read('README.en.md'), /docs\/remote-getting-started\.md/);
  assert.match(read('docs/product-status.zh-TW.md'), /remote-getting-started\.zh-TW\.md/);
  assert.match(read('docs/product-status.md'), /remote-getting-started\.md/);
  assert.match(read('docs/server-deployment.zh-TW.md'), /remote-getting-started\.zh-TW\.md/);
});

test('documented browser routes reflect the SPA + browser-API surface', () => {
  const routes = read('crates/speclink-server/src/app.rs');
  // After the SPA migration the server-rendered /account and /admin/users HTML
  // pages are gone; every browser route is served by the SPA shell fallback.
  assert.match(routes, /\.fallback\(assets::spa_fallback\)/);
  assert.doesNotMatch(routes, /web::account_page/);
  assert.doesNotMatch(routes, /admin::users_page/);
  // PAT creation is a same-origin browser-API POST, never a browsable GET page.
  const webApi = read('crates/speclink-server/src/web.rs');
  assert.match(webApi, /\.route\("\/account\/tokens", post\(api_create_pat\)\)/);

  for (const relativePath of remoteGuides) {
    const guide = read(relativePath);
    // The guides create a PAT through the account SPA page's browser API.
    assert.match(guide, /\/api\/speclink\/v1\/web\/account\/tokens/);
    assert.equal(
      localMarkdownLinks(guide).some((target) => target.endsWith('/account/tokens')),
      false,
      `${relativePath} must not link to the POST-only action as a browser page`,
    );
  }
});

test('both guides tie the dev startup step to the checkout CLI build', () => {
  for (const relativePath of remoteGuides) {
    const startup = sectionStartingWith(read(relativePath), '2.', relativePath);
    assert.match(startup, literal('speclink-cli'));
  }
});

test('both guides verify the CLI through the checkout wrapper', () => {
  for (const relativePath of remoteGuides) {
    const guide = read(relativePath);
    for (const required of [
      'npm run cli -- ',
      'npm run --silent cli -- ',
      'npm --prefix ',
    ]) {
      assert.match(guide, literal(required), `${relativePath}: 缺少 ${required}`);
    }
    assert.doesNotMatch(
      guide,
      /path\/to\/speclink\/target\/debug\/speclink/,
      `${relativePath}: 不應再教使用者手打 binary 絕對路徑`,
    );
  }
});

test('both guides troubleshoot a stale speclink on PATH', () => {
  for (const relativePath of remoteGuides) {
    const troubleshooting = sectionStartingWith(read(relativePath), '10.', relativePath);
    assert.match(troubleshooting, literal('PATH'));
    assert.match(troubleshooting, literal('npm run cli'));
  }
});

test('platform architecture records the dev CLI build gate', () => {
  const doc = read('docs/platform-architecture.zh-TW.md');
  assert.match(doc, literal('speclink-cli'));
  assert.match(doc, literal('npm run cli'));
});

/// 這個 repo 建出來的 CLI。刻意不走 PATH 上的 `speclink`：那可能是使用者安裝的舊版
/// （於是文件比對的是過期的 help surface），而 CI 上根本沒有，spawn 直接 ENOENT。
/// debug 優先——本機開發者用的就是它；CI 只建 release，於是落到第二順位。
function builtCli() {
  const exe = process.platform === 'win32' ? 'speclink.exe' : 'speclink';
  for (const profile of ['debug', 'release']) {
    const candidate = path.join(root, 'target', profile, exe);
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

const cli = builtCli();

test(
  'documented CLI commands are present in the current help surface',
  { skip: !cli && '尚未建置 CLI（target/{debug,release} 皆無）' },
  () => {
    // clap 的 Usage 行用的是執行檔名——Windows 上是 speclink.exe，故程式名後
    // 容許 .exe 尾碼；文件教的指令寫法不受影響。
    const cases = [
      [[], /Commands:[\s\S]*\blink\b[\s\S]*\bauth\b/],
      [['link', '--help'], /Usage: speclink(?:\.exe)? link \[OPTIONS\] <URL>[\s\S]*--repo <REPO>/],
      [['auth', 'login', '--help'], /Usage: speclink(?:\.exe)? auth login \[OPTIONS\][\s\S]*--token-stdin/],
    ];

    for (const [args, expected] of cases) {
      const result = spawnSync(cli, args.length === 0 ? ['--help'] : args, {
        cwd: root,
        encoding: 'utf8',
      });
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, expected);
    }
  },
);

test('relative Markdown links in the changed documentation resolve', () => {
  const documents = [
    'README.md',
    'README.en.md',
    'docs/remote-getting-started.zh-TW.md',
    'docs/remote-getting-started.md',
    'docs/product-status.zh-TW.md',
    'docs/product-status.md',
    'docs/server-deployment.zh-TW.md',
    'docs/platform-architecture.zh-TW.md',
    'docs/development.zh-TW.md',
    'docs/development.md',
  ];

  for (const relativePath of documents) {
    const markdown = read(relativePath);
    for (const target of localMarkdownLinks(markdown)) {
      const withoutFragment = target.split('#', 1)[0];
      if (withoutFragment === '') continue;
      const resolved = path.resolve(root, path.dirname(relativePath), withoutFragment);
      assert.equal(existsSync(resolved), true, `${relativePath}: broken link ${target}`);
    }
  }
});
