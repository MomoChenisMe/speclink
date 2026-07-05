/** 一行任務：是否勾選 + 文字（去掉 checkbox 標記）。 */
export interface TaskLine {
  done: boolean;
  text: string;
}

/** 任務文件的一個節點：群組標題或任務（帶 1-based 序數，供寫回 tasks.md 定位）。 */
export type TaskDocItem =
  | { kind: "group"; text: string }
  | { kind: "task"; ordinal: number; done: boolean; text: string };

/** 解析 tasks.md 為群組標題＋任務序列（序數僅計 checkbox 行，與桌面寫入 command 對齊）。 */
export function parseTaskDoc(markdown: string | null | undefined): TaskDocItem[] {
  if (!markdown) return [];
  const items: TaskDocItem[] = [];
  let ordinal = 0;
  for (const line of markdown.split(/\r?\n/)) {
    const g = line.match(/^##\s+(.*\S)\s*$/);
    if (g) {
      items.push({ kind: "group", text: g[1] });
      continue;
    }
    const t = line.match(/^\s*-\s*\[([ xX])\]\s+(.*\S)\s*$/);
    if (t) {
      ordinal += 1;
      items.push({ kind: "task", ordinal, done: t[1].toLowerCase() === "x", text: t[2] });
    }
  }
  return items;
}

/** 解析 tasks.md 的 checkbox 行（`- [ ]` / `- [x]`）為任務清單。非 checkbox 行忽略。 */
export function parseTasks(markdown: string | null | undefined): TaskLine[] {
  if (!markdown) return [];
  const out: TaskLine[] = [];
  for (const line of markdown.split(/\r?\n/)) {
    const m = line.match(/^\s*-\s*\[([ xX])\]\s+(.*\S)\s*$/);
    if (m) out.push({ done: m[1].toLowerCase() === "x", text: m[2] });
  }
  return out;
}
