// getting-started 與 workflow 的指令面查核（user-documentation spec「文件準確性具可重複
// 驗證清單」）：寫成可直接使用的 `speclink` 子指令與旗標要出現在目前 CLI 的 help，
// 文件明說「沒有」的指令要確實不存在；技能名要存在於產生的技能目錄。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { NO_BUILT_CLI, builtCli } from './built-cli.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const read = (relative) => readFileSync(path.join(ROOT, relative), 'utf8');

const DOCS = [
  'docs/getting-started.zh-TW.md',
  'docs/getting-started.md',
  'docs/workflow.zh-TW.md',
  'docs/workflow.md',
];

/// 文件寫明不存在的指令：「沒有 `speclink baseline` 指令」／「No `speclink baseline` command exists」。
const ABSENT = /沒有 `(speclink [^`]+)` 指令|No `(speclink [^`]+)` command exists/g;

/// 會寫指令的 code block 語言。
const SHELL_FENCE = /```(?:bash|sh|shell|zsh|console)\n([\s\S]*?)```/g;

/// 行內 code 與 shell 區塊裡的指令。區塊內以 `\` 續行的先接回一行，console 的 `$ `
/// 提示符號先剝掉，管線、`&&`、`;` 後面的 `speclink …` 也算。`speclink 0.8.0 (…)`
/// 這類輸出範例後面接的不是子指令或旗標，不算指令。
function commandsIn(markdown) {
  const absent = new Set([...markdown.matchAll(ABSENT)].map((match) => match[1] ?? match[2]));
  const spans = [...markdown.matchAll(/`(speclink [^`\n]+)`/g)].map((match) => match[1]);
  const lines = [...markdown.matchAll(SHELL_FENCE)].flatMap((match) =>
    match[1]
      .replace(/\\\n\s*/g, '')
      .split('\n')
      .map((line) => line.replace(/\s+#.*$/, '').replace(/^\s*\$\s+/, ''))
      .flatMap((line) => line.split(/\|\||&&|[|;]/))
      .map((segment) => segment.trim())
      .filter((segment) => segment.startsWith('speclink ')),
  );
  const present = [...spans, ...lines].filter(
    (command) => /^speclink (?:-|[a-z])/.test(command) && !absent.has(command),
  );
  return { present, absent: [...absent] };
}

const cli = builtCli();
const helpCache = new Map();

function helpOf(words) {
  const key = words.join(' ');
  if (!helpCache.has(key)) {
    const result = spawnSync(cli, [...words, '--help'], { cwd: ROOT, encoding: 'utf8' });
    assert.equal(result.status, 0, `speclink ${key} --help 失敗：${result.stderr}`);
    helpCache.set(key, result.stdout);
  }
  return helpCache.get(key);
}

const subcommandsIn = (help) =>
  new Set(
    (/\nCommands:\n([\s\S]*?)(?:\n\n|$)/.exec(help)?.[1] ?? '')
      .split('\n')
      .map((line) => line.trim().split(/\s+/)[0])
      .filter(Boolean),
  );

/// help 的 Options 區段：旗標只認這裡列的，子指令說明裡順口提到的不算。
const optionsOf = (help) => /\nOptions:\n([\s\S]*)$/.exec(help)?.[1] ?? '';

/// 沿著 help 的 Commands 清單走到最深的子指令。clap 的 `help` 子指令後面接的是要查的
/// 子指令路徑（`speclink help archive` 查的是 archive），所以先剝掉；子指令前的旗標
/// （`speclink --no-color list`）略過繼續走；遇到參數就停。
/// 回傳走到的子指令路徑，以及第一個不在清單裡的子指令名（沒有則為 null）。
function walkSubcommands(command) {
  const tokens = command.split(/\s+/).slice(1);
  if (tokens[0] === 'help') tokens.shift();
  const words = [];
  for (const token of tokens) {
    if (token.startsWith('-')) continue;
    if (!/^[a-z][a-z-]*$/.test(token)) break;
    const subcommands = subcommandsIn(helpOf(words));
    if (subcommands.has(token)) {
      words.push(token);
      continue;
    }
    // 有子指令清單卻對不上：寫錯的子指令。沒有清單：這是位置參數（變更名稱等）。
    return { words, unknown: subcommands.size > 0 ? token : null };
  }
  return { words, unknown: null };
}

const FLAG = /(?<![\w-])(--?[a-zA-Z][\w-]*)/g;

test(
  'getting-started／workflow 的 speclink 子指令與旗標都在目前的 help 裡',
  { skip: !cli && NO_BUILT_CLI },
  () => {
    const problems = [];
    for (const doc of DOCS) {
      for (const command of commandsIn(read(doc)).present) {
        const { words, unknown } = walkSubcommands(command);
        if (unknown) {
          problems.push(`${doc}：\`${command}\` 的子指令「${unknown}」不存在`);
          continue;
        }
        const options = optionsOf(helpOf(words));
        for (const [, flag] of command.matchAll(FLAG)) {
          if (!new RegExp(`(?<![\\w-])${flag}(?![\\w-])`).test(options)) {
            problems.push(`${doc}：\`${command}\` 的旗標 ${flag} 不在 \`${['speclink', ...words, '--help'].join(' ')}\``);
          }
        }
      }
    }
    assert.deepEqual(problems, []);
  },
);

test(
  'getting-started／workflow 寫明「沒有」的 speclink 指令確實不存在',
  { skip: !cli && NO_BUILT_CLI },
  () => {
    const problems = [];
    for (const doc of DOCS) {
      for (const command of commandsIn(read(doc)).absent) {
        if (walkSubcommands(command).unknown === null) problems.push(`${doc}：文件說沒有 \`${command}\`，但 CLI 有`);
      }
    }
    assert.deepEqual(problems, []);
  },
);

/// Claude 寫 `/speclink-x`、Codex 寫 `$speclink-x`，各自對到自己的技能目錄。
/// 前面緊接字元、`.`、`/` 或 `-` 的是路徑（`crates/adapters/speclink-cli`、`./speclink-data`），不是技能。
const SKILL = /(?<![\w./-])([/$])(speclink-[a-z]+(?:-[a-z]+)*)/g;
const SKILL_DIRS = { '/': '.claude/skills', $: '.agents/skills' };

test('getting-started／workflow 的技能名稱都存在於產生的技能目錄', () => {
  const problems = [];
  for (const doc of DOCS) {
    for (const [, sigil, name] of read(doc).matchAll(SKILL)) {
      if (!existsSync(path.join(ROOT, SKILL_DIRS[sigil], name, 'SKILL.md'))) {
        problems.push(`${doc}：${sigil}${name} 不在 ${SKILL_DIRS[sigil]}/`);
      }
    }
  }
  assert.deepEqual([...new Set(problems)], []);
});

test('抽取：管線、提示符號、續行與 shell／console 區塊裡的指令都算', () => {
  const md = [
    '```bash',
    "printf 'x' | speclink discuss context demo --stdin",
    'speclink validate --specs \\',
    '  --strict',
    '```',
    '',
    '```console',
    '$ speclink list --json',
    'speclink 0.8.0 (arm64, engine v1.42.0)',
    '```',
    '',
    '```shell',
    'cd repo && speclink plan --json',
    '```',
  ].join('\n');
  assert.deepEqual(commandsIn(md).present, [
    'speclink discuss context demo --stdin',
    'speclink validate --specs --strict',
    'speclink list --json',
    'speclink plan --json',
  ]);
});

test('旗標比對只看 help 的 Options 區段，不看子指令說明裡提到的旗標', () => {
  const help = [
    'Usage: speclink [OPTIONS] <COMMAND>',
    '',
    'Commands:',
    '  discard  Discard a change (--force required once work has started)',
    '',
    'Options:',
    '      --no-color  Disable colored output',
    '  -h, --help      Print help',
  ].join('\n');
  assert.equal(optionsOf(help).includes('--force'), false);
  assert.equal(optionsOf(help).includes('--no-color'), true);
});

test(
  '子指令路徑：help 子指令、前置的全域旗標與多層的「沒有」指令',
  { skip: !cli && NO_BUILT_CLI },
  () => {
    assert.deepEqual(walkSubcommands('speclink help archive'), { words: ['archive'], unknown: null });
    assert.deepEqual(walkSubcommands('speclink --no-color list --json'), { words: ['list'], unknown: null });
    assert.deepEqual(walkSubcommands('speclink discuss nonexistent-verb'), {
      words: ['discuss'],
      unknown: 'nonexistent-verb',
    });
  },
);
