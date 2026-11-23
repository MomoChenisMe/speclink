// 在終端機 Cmd+點變更或討論底下的路徑：用 Speclink 桌面 app 開它的詳情頁。
// 哪些路徑會送到這裡由 mod.json 的 link_handlers 決定，dock-panel 會把這個 mod 加進版面。
import { exec } from "herdr";
import { lastOpenError, linkTarget, speclinkUrl } from "./model.js";
import { projectRoot } from "./shared.js";

export async function handleLink({ link, source }) {
  // 桌面 app 在這台機器上，別台機器的路徑開不了。
  const target = link.kind === "file" && link.path && !source.machine ? linkTarget(link.path) : null;
  if (!target) return "pass";
  // 在 worktree 裡也要用主資料夾，桌面 app 認得的是主資料夾。
  const root = await projectRoot(target.dir, null);
  // 沒裝桌面 app，或 app 舊到不認得 speclink 網址時，交回 Herdr 照原本的方式開。
  const error = await lastOpenError([[speclinkUrl(target.kind, root, target.name)]], exec);
  return error === null ? "handled" : "pass";
}
