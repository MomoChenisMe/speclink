/** delta spec 各操作的 Requirement 計數。 */
export interface DeltaCounts {
  added: number;
  modified: number;
  removed: number;
  renamed: number;
}

const SECTION_RE = /^##\s+(ADDED|MODIFIED|REMOVED|RENAMED)\s+Requirements\s*$/;
const REQ_RE = /^###\s+Requirement:/;

/** 解析 delta spec markdown：各 `## <OP> Requirements` 區段內 `### Requirement:` 的數量。 */
export function specDeltaCounts(markdown: string | null | undefined): DeltaCounts {
  const counts: DeltaCounts = { added: 0, modified: 0, removed: 0, renamed: 0 };
  if (!markdown) return counts;
  let section: keyof DeltaCounts | null = null;
  for (const line of markdown.split(/\r?\n/)) {
    const m = line.match(SECTION_RE);
    if (m) {
      section = m[1].toLowerCase() as keyof DeltaCounts;
      continue;
    }
    if (/^##\s/.test(line)) {
      section = null;
      continue;
    }
    if (section && REQ_RE.test(line)) counts[section]++;
  }
  return counts;
}

/** delta spec 的一個渲染區段：op 為 delta 種類，null＝非 delta 內文（照常渲染）。 */
export interface DeltaSection {
  op: keyof DeltaCounts | null;
  content: string;
}

/**
 * 把 delta spec 切成渲染區段（design D4）：`## <OP> Requirements` 開新 delta 區段
 * （標題行不入內文，由呼叫端畫色標標頭）；其他 `## ` 標題結束 delta 區段（與
 * specDeltaCounts 同界線）、自身歸入無種類區段照常渲染。無任何 delta 標題＝整篇單段。
 */
export function splitDeltaSections(markdown: string): DeltaSection[] {
  const sections: DeltaSection[] = [];
  let op: keyof DeltaCounts | null = null;
  let buf: string[] = [];
  const flush = () => {
    const content = buf.join("\n");
    if (op !== null || content.trim()) sections.push({ op, content });
    buf = [];
  };
  for (const line of markdown.split(/\r?\n/)) {
    const m = line.match(SECTION_RE);
    if (m) {
      flush();
      op = m[1].toLowerCase() as keyof DeltaCounts;
      continue;
    }
    if (/^##\s/.test(line) && op !== null) {
      flush();
      op = null;
    }
    buf.push(line);
  }
  flush();
  return sections.length > 0 ? sections : [{ op: null, content: markdown }];
}

/** 合併多個 delta spec 的計數。 */
export function sumDeltaCounts(list: DeltaCounts[]): DeltaCounts {
  return list.reduce(
    (a, b) => ({
      added: a.added + b.added,
      modified: a.modified + b.modified,
      removed: a.removed + b.removed,
      renamed: a.renamed + b.renamed,
    }),
    { added: 0, modified: 0, removed: 0, renamed: 0 },
  );
}

/** 顯示為 Spectra 風的 `+1 ~2 -0` 摘要（省略為 0 的項）。 */
export function formatDeltaCounts(c: DeltaCounts): string {
  const parts: string[] = [];
  if (c.added) parts.push(`+${c.added}`);
  if (c.modified) parts.push(`~${c.modified}`);
  if (c.removed) parts.push(`-${c.removed}`);
  if (c.renamed) parts.push(`→${c.renamed}`);
  return parts.join(" ");
}
