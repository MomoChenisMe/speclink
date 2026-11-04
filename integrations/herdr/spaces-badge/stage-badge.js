// Spaces 側欄的階段徽章：這個專案的變更在各階段有幾個，順序和顏色都和面板的卡片相同。
// Herdr 內建的 spaces mod 沒有擴充點，所以 dock-panel.mjs 複製一份，把這個檔放進去並接上。
import { div } from "gpui-kit";
import { h_flex } from "gpui-base";
import { exec } from "herdr";
import { tr } from "./i18n.js";

const STAGES = [
  { id: "proposed", label: "提案中", tone: "ansi3" },
  { id: "in-progress", label: "進行中", tone: "ansi4" },
  { id: "ready", label: "已就緒", tone: "ansi2" },
];

// repo 對應各階段的數量。內建 spaces 的 git 資訊裡，repo 是 JSON.stringify([機器, .git 資料夾])。
const counts = new Map();

async function look(view, cx) {
  const repos = new Set(Object.values(view.git).map((g) => g?.repo).filter(Boolean));
  for (const repo of repos) {
    const [on, gitDir] = JSON.parse(repo);
    // 沒有 Speclink 的專案和連不上的機器都當成沒有徽章。
    const r = await exec("speclink", ["plan", "--json"], { cwd: gitDir.replace(/\/\.git$/, ""), on }).catch(() => null);
    const changes = r?.code === 0 ? JSON.parse(r.stdout).changes : [];
    const now = STAGES.map((s) => changes.filter((c) => c.stage === s.id).length);
    if (String(counts.get(repo)) !== String(now)) {
      counts.set(repo, now);
      cx.notify();
    }
  }
}

export function watchStages(view, cx) {
  // 內建 spaces 剛啟動時還在問 git，稍等才有 repo 可以看。
  cx.timer.after(2000, (cx) => look(view, cx));
  cx.timer.every(30000, (cx) => look(view, cx));
}

// 已就緒是下一步要收尾的變更，所以用底色和勾勾畫得比較明顯。
function stageMark({ id, tone, n }, t) {
  if (id === "ready") return div().px(5).rounded_full().bg(`${t[tone]}26`).text_color(t[tone]).child(`✓${n}`);
  return h_flex().gap(3).items_center().text_color(t[tone]).child(div().size(6).rounded_full().bg(t[tone])).child(String(n));
}

// 列尾固定不縮的部分：git 的箭頭（從內建的 gitTail 搬來）和階段徽章。回傳陣列，內建 spaces 用 children 接。
// 徽章只畫在專案那一列，worktree 那幾列屬於同一個專案。
export function rowEnd(g, tree, t) {
  if (!g) return [];
  const shown = STAGES.map((s, i) => ({ ...s, n: counts.get(g.repo)?.[i] ?? 0 })).filter((s) => s.n > 0 && !tree);
  const arrow = (mark, n, color) => div().flex_none().text_size(11.5).text_color(color).child(`${mark}${n}`);
  return [
    ...(g.ahead > 0 ? [arrow("↑", g.ahead, t.ansi2)] : []),
    ...(g.behind > 0 ? [arrow("↓", g.behind, t.ansi1)] : []),
    ...(shown.length > 0
      ? [
          h_flex()
            .id(`speclink-stages-${g.repo}`)
            .flex_none()
            .gap(5)
            .items_center()
            .text_size(10.5)
            .font_semibold()
            .tooltip(shown.map((s) => `${tr(s.label)} ${s.n}`).join("　"))
            .children(shown.map((s) => stageMark(s, t))),
        ]
      : []),
  ];
}
