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

/** 就地改寫第 ordinal（1-based，僅計 checkbox 行）個任務的勾選標記——勾選樂觀
 * 更新用，序數判定與 parseTaskDoc 同一正則對齊。找不到該序數回 null。 */
export function setTaskMark(markdown: string, ordinal: number, done: boolean): string | null {
  let n = 0;
  let hit = false;
  const lines = markdown.split(/\r?\n/).map((line) => {
    const m = line.match(/^(\s*-\s*\[)[ xX](\]\s+.*\S\s*)$/);
    if (!m) return line;
    n += 1;
    if (n !== ordinal) return line;
    hit = true;
    return `${m[1]}${done ? "x" : " "}${m[2]}`;
  });
  return hit ? lines.join("\n") : null;
}

/** 拖放落點解析結果：以第 to 個任務為錨；before=true＝插錨前（組首落點）。 */
export interface DropTarget {
  to: number;
  before?: boolean;
}

/**
 * 把 dnd 的 over id 解析為寫回用的落點（design D6）：
 * - over 為任務 ordinal → `{ to }`（側別留給後端方向推斷）
 * - over 為群組標題 id（`g-<items 索引>`）→ 標題是「組界槽」，依 active 相對
 *   標題的位置雙向解析：上方來＝成為該群組組首（組首任務為錨、before=true）；
 *   下方來＝移到標題之前、成為上一群組末任務（標題前最近任務為錨、before=false）
 *   ——否則組首任務永遠拖不回上一群組末位。
 * - 該側無任務可錨定（空群組、檔首）、錨即自己、落點即自己 → null（不觸發寫回）
 */
export function resolveDropTarget(
  items: TaskDocItem[],
  activeOrdinal: number,
  overId: number | string,
): DropTarget | null {
  if (typeof overId === "number") {
    return overId === activeOrdinal ? null : { to: overId };
  }
  const headingIndex = Number(overId.replace(/^g-/, ""));
  if (!Number.isInteger(headingIndex) || items[headingIndex]?.kind !== "group") return null;
  const activeIndex = items.findIndex((it) => it.kind === "task" && it.ordinal === activeOrdinal);
  if (activeIndex < 0) return null;
  if (activeIndex < headingIndex) {
    // 上方來：錨定標題後第一個任務（同群組組首），插其前。
    for (let i = headingIndex + 1; i < items.length; i++) {
      const item = items[i];
      if (item.kind === "group") return null; // 標題下無任務（空群組）
      return item.ordinal === activeOrdinal ? null : { to: item.ordinal, before: true };
    }
    return null;
  }
  // 下方來：錨定標題前最近的任務（上一群組末位），插其後。
  for (let i = headingIndex - 1; i >= 0; i--) {
    const item = items[i];
    if (item.kind === "task") {
      return item.ordinal === activeOrdinal ? null : { to: item.ordinal, before: false };
    }
  }
  return null; // 標題之前無任務（檔首）
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
