// 面板和技能跳窗共用的 Herdr 小工具。
import { exec, focused, hc, panes, run, spaces } from "herdr";
import { tr } from "./i18n.js";
import { inside, parseStations, stampedPaths } from "./model.js";

export const firstLine = (text) => text.trim().split("\n")[0];

// 目前 space 所在的資料夾：焦點所在的終端機，否則該 space 第一個終端機。spaces() 的 folder
// 只是開新終端機的起點，不一定是它現在的位置。
export function location() {
  const space = spaces().find((s) => s.active);
  const f = focused();
  const ps = panes();
  const pane = f?.pane ? ps.find((p) => p.pane === f.pane) : null;
  const tabs = new Set(space?.tabs.map((t) => t.id));
  const first = ps.find((p) => tabs.has(p.tab) && p.cwd)?.cwd;
  return {
    cwd: pane?.cwd || f?.cwd || first || space?.folder || null,
    on: space?.machine ?? null,
    branch: pane?.branch ?? space?.branch ?? null,
  };
}

// 主資料夾：在 worktree 裡也要回到主 checkout，變更清單才完整。
export async function projectRoot(cwd, on) {
  const r = await exec("git", ["rev-parse", "--path-format=absolute", "--git-common-dir"], { cwd, on });
  return r.code === 0 ? r.stdout.trim().replace(/\/\.git\/?$/, "") : cwd;
}

// 新分頁開在已經待在這個資料夾的 space，目前的 space 優先。
export function spaceFor(folder) {
  const all = spaces();
  const ps = panes();
  const holds = (s) => ps.some((p) => s.tabs.some((t) => t.id === p.tab) && inside(p.cwd, folder));
  return all.find((s) => s.active && holds(s)) ?? all.find(holds);
}

// 跑 speclink 動詞並讀它的 JSON。失敗時丟出 CLI 的第一行訊息。
export async function speclinkJson(verb, cwd, on) {
  const r = await exec("speclink", [...verb, "--json"], { cwd, on });
  if (r.code !== 0) throw new Error(firstLine(r.stderr) || tr("speclink {0} 結束碼 {1}", verb.join(" "), r.code));
  return JSON.parse(r.stdout);
}

// 進行中的變更還剩幾個程式任務，手動任務（[M]）不算。list 和 plan 沒有這個數字，要問 instructions。
// 有 worktree 的要到 worktree 裡問，主資料夾的副本不會更新。查不到的不放進結果。
export async function codeRemaining(list, plan, root, on) {
  const folders = new Map(list.changes.map((c) => [c.name, c.worktree?.path ?? root]));
  const names = plan.changes.filter((c) => c.stage === "in-progress").map((c) => c.name);
  const answers = await Promise.all(
    names.map((name) =>
      speclinkJson(["instructions", "apply", "--change", name], folders.get(name) ?? root, on).catch(() => null),
    ),
  );
  return Object.fromEntries(names.flatMap((name, i) => (answers[i]?.progress ? [[name, answers[i].progress.codeRemaining]] : [])));
}

// 這個變更的 .meta.yaml，加上開著的工單。
const READ_STATIONS = `for d in speclink openspec; do
  c="$d/changes/$1"
  [ -d "$c" ] || continue
  cat "$c/.meta.yaml" 2>/dev/null; echo
  for s in review verify; do [ -e "$c/$s.md" ] && echo "open_ticket: $s"; done
  exit 0
done`;

// 檔案的指紋，規則和引擎的 content_fingerprint_bytes 相同：UTF-8 文字先把 CRLF 換成 LF，其他檔案看原始位元組。
// 不見的檔案不輸出。
const FINGERPRINT = String.raw`for my $f (@ARGV) { open(my $h, "<:raw", $f) or next; my $b = <$h> // ""; my $u = $b; $b =~ s/\r\n/\n/g if utf8::decode($u); print sha256_hex($b), "  $f\n" }`;

async function fingerprints(paths, cwd, on) {
  const r = await exec("perl", ["-MDigest::SHA=sha256_hex", "-0777", "-e", FINGERPRINT, "--", ...paths], { cwd, on }).catch(() => null);
  if (r?.code !== 0) return null;
  return Object.fromEntries(r.stdout.split("\n").filter(Boolean).map((line) => [line.slice(66), line.slice(0, 64)]));
}

// 品質關卡的章和工單。CLI 沒有給，只好讀檔；有 worktree 的要到 worktree 裡讀，主資料夾的副本不會更新。
// 提案中的變更還沒有章。讀不到時沒有章，封存時引擎還會再查一次。
export async function stationFacts(list, plan, root, on) {
  const folders = new Map(list.changes.map((c) => [c.name, c.worktree?.path ?? root]));
  const names = plan.changes.filter((c) => c.stage !== "proposed").map((c) => c.name);
  const facts = await Promise.all(
    names.map(async (name) => {
      const cwd = folders.get(name) ?? root;
      const r = await exec("sh", ["-c", READ_STATIONS, "sh", name], { cwd, on }).catch(() => null);
      const stations = parseStations(r?.stdout ?? "");
      const paths = stampedPaths(stations);
      return [name, { ...stations, hashes: paths.length > 0 ? await fingerprints(paths, cwd, on) : null }];
    }),
  );
  return Object.fromEntries(facts);
}

// hc run 回來時分頁可能還沒出現，等它出現才能改名。
const nameTabs = (waiting, cx, tries = 20) =>
  new Promise((done) => {
    const left = waiting.filter(({ address, name }) => {
      const pane = panes().find((p) => p.terminal === address);
      if (pane) run("tab.rename", { id: pane.tab, name });
      return !pane;
    });
    if (left.length === 0 || tries === 0) return done();
    cx.timer.after(150, (cx) => nameTabs(left, cx, tries - 1).then(done));
  });

// 依序開新分頁並取名。中途失敗就停下來，已經開好的分頁留著，回傳失敗的結果；全部成功時回傳 null。
export async function openTabs(plans, cx) {
  const opened = [];
  let failed = null;
  for (const plan of plans) {
    const r = await hc(plan.hc);
    if (r.code !== 0) {
      failed = r;
      break;
    }
    opened.push({ address: JSON.parse(r.stdout).address, name: plan.tabName });
  }
  await nameTabs(opened, cx);
  return failed;
}

// Herdr 沒有「只貼上不按 Enter」的指令，所以暫時當一個看不見的檢視者：hc attach 要真的終端機才肯跑，
// script 給它一個。送出括號貼上（程式會當成貼上，不是打字），再按 ctrl+\ 離開。
// 大小用終端機現在的大小，它才不會跳動。ctrl+\ 要和貼上分開送，同一次讀到時 hc 不會當成離開；
// alarm 確保失敗時不會一直掛著。script 的用法是 macOS 的，其他系統會失敗並回報。
const PASTE = `{ sleep 0.2; printf '\\033[200~%s\\033[201~' "$4"; sleep 0.15; printf '\\034'; } | perl -e 'alarm 3; exec @ARGV' script -q /dev/null sh -c 'stty rows "$2" cols "$3"; exec hc attach "$1"' sh "$1" "$2" "$3" >/dev/null 2>&1`;

// 照 placePlan 或 attachPlan 的結果貼進輸入框，再把焦點切過去。失敗時回傳結果，sendError 讀得懂。
export async function placeInInput(plan) {
  const { address } = plan.agent;
  const got = await hc(["get", address, "--json"]);
  if (got.code !== 0) return got;
  const { rows, cols } = JSON.parse(got.stdout);
  const r = await exec("sh", ["-c", PASTE, "sh", address, String(rows), String(cols), plan.text]);
  if (r.code !== 0) return { ...r, stderr: tr("貼不進 {0}（結束碼 {1}）", address, r.code) };
  // 附加名稱不算開始做，所以只有技能指令帶標籤。
  if (plan.labels) await hc(["label", address, ...plan.labels]);
  run("pane.focus", { terminal: address });
  return null;
}

export function sendError(result) {
  try {
    return JSON.parse(result.stderr).error.message;
  } catch {
    return firstLine(result.stderr) || tr("hc 結束碼 {0}", result.code);
  }
}
