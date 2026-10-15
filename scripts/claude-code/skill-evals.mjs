#!/usr/bin/env node
// 技能評測的執行與新舊比較（skill-authoring 規格「技能評測套件」）。
//
//   node scripts/claude-code/skill-evals.mjs run --speclink <執行檔> --model <模型 ID> \
//     --label <名稱> --max-cost-usd <金額> [--case <題目>] [--runs <次數>]
//   node scripts/claude-code/skill-evals.mjs compare <舊結果目錄> <新結果目錄>
//
// run 以指定的執行檔渲染受測技能、也讓評測工作區內的 `speclink` 都是它：評測不載入
// 工作區的專案技能，所以技能要放進 plugin 的 skills/。repo 的 manifest 與 evals/ 會複製
// 成 target/skill-evals/.staging/<label>/plugin/，再補上該執行檔寫出的 `.claude/skills/speclink-*`。
//
// compare 的 exit code：0 沒有退步、1 有退步、2 題目集合不同、3 無法比較（用法錯、結果檔
// 不存在或不是 JSON、結果因花費上限而不完整）。
// 需要 claude CLI 與使用者額度，不在 CI 執行；CI 只跑 skill-evals.test.mjs。
import { execFileSync, spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const SUITE_DIR = 'integrations/claude-code/skill-evals';
export const OUT_ROOT = 'target/skill-evals';
export const RESULT_FILE = 'aggregate-result.json';
/// 執行檔與 plugin 副本的暫存根目錄。label 不能以 . 開頭，所以它不會和任何結果目錄同名。
const STAGING = '.staging';
const CANNOT_COMPARE = 3;
const JUDGE_MODEL = 'claude-sonnet-5-5';

/// 逐題比較兩份 aggregate-result.json：題目名在 cases[].name、分數在 cases[].aggregates.score。
/// rows 只含兩邊都有的題目（依名稱排序）；新分數低於舊分數即退步；只在一邊出現的題目列入 missing。
export function compareResults(oldDoc, newDoc) {
  const scores = (doc) => new Map(doc.cases.map((entry) => [entry.name, entry.aggregates.score]));
  const oldScores = scores(oldDoc);
  const newScores = scores(newDoc);
  const names = [...new Set([...oldScores.keys(), ...newScores.keys()])].sort();
  const rows = [];
  const missing = [];
  for (const name of names) {
    if (!newScores.has(name)) missing.push({ name, missingFrom: 'new' });
    else if (!oldScores.has(name)) missing.push({ name, missingFrom: 'old' });
    else rows.push({ name, old: oldScores.get(name), new: newScores.get(name), delta: newScores.get(name) - oldScores.get(name) });
  }
  const regressions = rows.filter((row) => row.new < row.old).map((row) => row.name);
  return { rows, regressions, missing };
}

/// label 成為 target/skill-evals/ 下的目錄名，只收單一路徑段。
const LABEL = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

/// run 用到的目錄：結果寫在 <OUT_ROOT>/<label>/，執行檔與 plugin 副本在 <OUT_ROOT>/.staging/<label>/。
export function runPaths(label, root = ROOT) {
  const staging = path.join(root, OUT_ROOT, STAGING, label);
  return { outDir: path.join(root, OUT_ROOT, label), binDir: path.join(staging, 'bin'), pluginDir: path.join(staging, 'plugin') };
}

/// claude 子行程的啟動設定：不讀 stdin、輸出不帶顏色、PATH 以受測執行檔的目錄開頭。
export function evalSpawnOptions(binDir, env = process.env) {
  return { stdio: ['ignore', 'inherit', 'inherit'], env: { ...env, NO_COLOR: '1', PATH: `${binDir}${path.delimiter}${env.PATH}` } };
}

/// run 的參數。回傳 `{ options, errors }`；errors 非空時不得啟動評測。
/// root 只供測試換成暫存目錄；--speclink 的相對路徑照常以目前目錄解析。
export function parseRunArgs(argv, root = ROOT) {
  let values;
  try {
    ({ values } = parseArgs({
      args: argv,
      options: {
        speclink: { type: 'string' },
        model: { type: 'string' },
        label: { type: 'string' },
        'max-cost-usd': { type: 'string' },
        case: { type: 'string' },
        runs: { type: 'string', default: '3' },
      },
    }));
  } catch (error) {
    return { options: null, errors: [error.message] };
  }
  const errors = ['speclink', 'model', 'label', 'max-cost-usd']
    .filter((name) => values[name] === undefined || values[name] === '')
    .map((name) => `缺少參數 --${name}`);
  if (values.speclink) {
    const binary = path.resolve(values.speclink);
    if (!existsSync(binary) || !statSync(binary).isFile()) errors.push(`--speclink 指向的執行檔不存在：${binary}`);
  }
  if (values.label && !LABEL.test(values.label)) {
    errors.push(`--label 只能用英數字、- 與 _，且以英數字開頭（它是 ${OUT_ROOT}/ 下的目錄名）：${values.label}`);
  } else if (values.label && existsSync(path.join(root, OUT_ROOT, values.label, RESULT_FILE))) {
    errors.push(`${OUT_ROOT}/${values.label}/ 已有評測結果；換一個 --label，或先刪掉該目錄`);
  }
  const cost = Number(values['max-cost-usd']);
  if (values['max-cost-usd'] && !(Number.isFinite(cost) && cost > 0)) {
    errors.push(`--max-cost-usd 必須是正數（美元）：${values['max-cost-usd']}`);
  }
  const runs = Number(values.runs);
  if (!(Number.isInteger(runs) && runs >= 1 && runs <= 50)) errors.push(`--runs 必須是 1 到 50 的整數：${values.runs}`);
  return { options: values, errors };
}

/// 讀結果檔；檔案不存在、不是 JSON，或因花費上限而不完整時丟出指名該檔的錯誤。
/// 不完整＝頂層 partial 為 true，或某次執行略過了付費評分（llm 評分沒跑，分數不能和完整結果比）。
function readResult(dir) {
  const file = path.join(dir, RESULT_FILE);
  if (!existsSync(file)) throw new Error(`找不到結果檔：${file}`);
  let doc;
  try {
    doc = JSON.parse(readFileSync(file, 'utf8'));
  } catch (error) {
    throw new Error(`結果檔不是 JSON：${file}（${error.message}）`);
  }
  const skippedPaid = doc.cases?.some((entry) => entry.arms?.with?.some((run) => run.skippedPaidGraders === true));
  if (doc.partial === true || skippedPaid) throw new Error(`結果因花費上限而不完整，不能比較：${file}`);
  return doc;
}

function compareMain([oldDir, newDir, ...rest]) {
  if (!oldDir || !newDir || rest.length > 0) {
    console.error('用法：node scripts/claude-code/skill-evals.mjs compare <舊結果目錄> <新結果目錄>');
    return CANNOT_COMPARE;
  }
  let result;
  try {
    result = compareResults(readResult(oldDir), readResult(newDir));
  } catch (error) {
    console.error(error.message);
    return CANNOT_COMPARE;
  }
  const width = Math.max('題目'.length, ...result.rows.map((row) => row.name.length));
  const score = (value) => value.toFixed(2);
  console.log(`${'題目'.padEnd(width)}  舊分數  新分數  差值`);
  for (const row of result.rows) {
    const delta = `${row.delta >= 0 ? '+' : ''}${row.delta.toFixed(2)}`;
    console.log(`${row.name.padEnd(width)}  ${score(row.old).padStart(6)}  ${score(row.new).padStart(6)}  ${delta}`);
  }
  for (const name of result.regressions) {
    const row = result.rows.find((entry) => entry.name === name);
    console.log(`退步：${name}（${score(row.old)} → ${score(row.new)}）`);
  }
  for (const entry of result.missing) {
    console.log(`缺少的題目：${entry.name}（${entry.missingFrom === 'new' ? '新' : '舊'}結果沒有）`);
  }
  // 題目集合不同時比較本身不完整，優先於退步回報。
  if (result.missing.length > 0) return 2;
  return result.regressions.length > 0 ? 1 : 0;
}

/// 以指定執行檔在暫存工作區寫出全部技能（worktree 政策開啟才會寫出 apply-with-worktree
/// 與 worktree-merge），連同 repo 的 manifest 與 evals/ 組成受測的 plugin 副本。
function stagePlugin(speclink, pluginDir) {
  rmSync(pluginDir, { recursive: true, force: true });
  mkdirSync(pluginDir, { recursive: true });
  cpSync(path.join(ROOT, SUITE_DIR, '.claude-plugin'), path.join(pluginDir, '.claude-plugin'), { recursive: true });
  cpSync(path.join(ROOT, SUITE_DIR, 'evals'), path.join(pluginDir, 'evals'), { recursive: true });
  const workspace = mkdtempSync(path.join(os.tmpdir(), 'speclink-skill-evals-'));
  try {
    execFileSync(speclink, ['init', workspace, '--tools', 'claude'], { stdio: 'ignore' });
    execFileSync(speclink, ['workflow-config', 'set', 'worktree', 'true'], { cwd: workspace, stdio: 'ignore' });
    execFileSync(speclink, ['update'], { cwd: workspace, stdio: 'ignore' });
    const skillsDir = path.join(workspace, '.claude', 'skills');
    for (const name of readdirSync(skillsDir).filter((entry) => entry.startsWith('speclink-'))) {
      cpSync(path.join(skillsDir, name), path.join(pluginDir, 'skills', name), { recursive: true });
    }
  } finally {
    rmSync(workspace, { recursive: true, force: true });
  }
}

/// macOS 的 /usr/bin/git 是 xcrun 轉接程式：它要把快取寫進系統暫存目錄，評測沙箱不准；
/// 真正的 git 在 /Library/Developer 底下，沙箱也讀不到。所以把真正的 git 複製到受測
/// 執行檔旁邊，受測模型與 speclink 都用這一支（內建子指令不需要 git-core 目錄）。
function copyRealGit(binDir) {
  if (process.platform !== 'darwin') return;
  const git = execFileSync('xcrun', ['--find', 'git'], { encoding: 'utf8' }).trim();
  copyFileSync(git, path.join(binDir, 'git'));
  chmodSync(path.join(binDir, 'git'), 0o755);
}

function runMain(argv) {
  const { options, errors } = parseRunArgs(argv);
  if (errors.length > 0) {
    for (const error of errors) console.error(error);
    return 1;
  }
  const { outDir, binDir, pluginDir } = runPaths(options.label);
  const speclink = path.join(binDir, 'speclink');

  mkdirSync(binDir, { recursive: true });
  copyFileSync(path.resolve(options.speclink), speclink);
  chmodSync(speclink, 0o755);
  copyRealGit(binDir);
  mkdirSync(outDir, { recursive: true });
  writeFileSync(path.join(outDir, 'speclink-version.txt'), execFileSync(speclink, ['--version'], { encoding: 'utf8' }));
  stagePlugin(speclink, pluginDir);

  const args = ['plugin', 'eval', pluginDir, '--scaffold', '--ablation', 'none', '--runs', options.runs, '--model', options.model];
  // 預設的 Haiku 評審對明顯符合的回覆也會投 FAIL，評審改用 Sonnet。
  args.push('--judge-model', JUDGE_MODEL);
  // apply-with-worktree 與 worktree-merge 的 worktree 在工作區旁邊（同一個暫存根目錄下），
  // 只給 Write／Edit 的話寫不進去。
  args.push('--allow-tools', 'Bash', 'Edit', 'Write', 'Edit(//private/tmp/**)', 'Write(//private/tmp/**)', 'Edit(//tmp/**)', 'Write(//tmp/**)');
  args.push('--trust-plugin', '--no-publish', '--threshold', '0');
  args.push('--max-cost-usd', options['max-cost-usd'], '--output-dir', outDir);
  if (options.case !== undefined) args.push('--case', options.case);
  const child = spawnSync('claude', args, evalSpawnOptions(binDir));
  if (child.error) {
    console.error(`無法啟動 claude：${child.error.message}`);
    return 1;
  }
  return child.status ?? 1;
}

function main([command, ...rest]) {
  if (command === 'compare') return compareMain(rest);
  if (command === 'run') return runMain(rest);
  console.error('用法：node scripts/claude-code/skill-evals.mjs <run|compare> ...');
  return 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  process.exit(main(process.argv.slice(2)));
}
