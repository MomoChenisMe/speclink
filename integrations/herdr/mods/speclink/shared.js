// 面板和新工作跳窗共用的 Herdr 小工具。
import { exec, focused, panes, spaces } from "herdr";
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

export function sendError(result) {
  try {
    return JSON.parse(result.stderr).error.message;
  } catch {
    return firstLine(result.stderr) || `hc 結束碼 ${result.code}`;
  }
}
