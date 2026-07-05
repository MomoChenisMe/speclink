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
