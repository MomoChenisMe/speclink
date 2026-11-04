// 面板和技能跳窗共用的 Herdr 小工具。
import { exec, focused, hc, panes, run, spaces } from "herdr";
import { inside } from "./model.js";

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
  if (r.code !== 0) throw new Error(firstLine(r.stderr) || `speclink ${verb.join(" ")} 結束碼 ${r.code}`);
  return JSON.parse(r.stdout);
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
  if (r.code !== 0) return { ...r, stderr: `貼不進 ${address}（結束碼 ${r.code}）` };
  run("pane.focus", { terminal: address });
  return null;
}

export function sendError(result) {
  try {
    return JSON.parse(result.stderr).error.message;
  } catch {
    return firstLine(result.stderr) || `hc 結束碼 ${result.code}`;
  }
}
