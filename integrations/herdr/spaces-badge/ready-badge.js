// Spaces 側欄的待收尾徽章：這個專案有幾個已就緒的變更。
// Herdr 內建的 spaces mod 沒有擴充點，所以 dock-panel.mjs 複製一份，把這個檔放進去並接上三行。
import { div } from "gpui-kit";
import { exec } from "herdr";

// 內建 spaces 的 git 資訊裡，repo 是 JSON.stringify([機器, .git 資料夾])。
const counts = new Map();

async function look(view, cx) {
  const repos = new Set(Object.values(view.git).map((g) => g?.repo).filter(Boolean));
  for (const repo of repos) {
    const [on, gitDir] = JSON.parse(repo);
    // 沒有 Speclink 的專案和連不上的機器都當成沒有徽章。
    const r = await exec("speclink", ["plan", "--json"], { cwd: gitDir.replace(/\/\.git$/, ""), on }).catch(() => null);
    const ready = r?.code === 0 ? JSON.parse(r.stdout).changes.filter((c) => c.stage === "ready").length : 0;
    if (counts.get(repo) !== ready) {
      counts.set(repo, ready);
      cx.notify();
    }
  }
}

export function watchReady(view, cx) {
  // 內建 spaces 剛啟動時還在問 git，稍等才有 repo 可以看。
  cx.timer.after(2000, (cx) => look(view, cx));
  cx.timer.every(30000, (cx) => look(view, cx));
}

// 回傳陣列，內建 spaces 用 children 接：沒有待收尾時什麼都不畫。worktree 那幾列屬於同一個專案，不重複畫。
export function readyBadge(g, tree, t) {
  const n = counts.get(g?.repo);
  if (tree || !n) return [];
  return [
    div()
      .id(`speclink-ready-${g.repo}`)
      .flex_none()
      .px(5)
      .rounded_full()
      .bg(`${t.ansi2}26`)
      .text_size(10.5)
      .font_semibold()
      .text_color(t.ansi2)
      .tooltip(`${n} 個變更待收尾`)
      .child(`✓${n}`),
  ];
}
