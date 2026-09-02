import type { ManualPageItem } from "./adapter";

/** 側欄樹的一個分區：標籤與其下依閱讀序排列的頁。 */
export interface Section {
  label: string;
  pages: ManualPageItem[];
}

/** 以 section 對閱讀序中連續的頁分組（core 已把同分區的頁排在一起）；缺 section
 * 的頁歸「其他」。 */
export function groupSections(pages: ManualPageItem[], otherLabel: string): Section[] {
  const sections: Section[] = [];
  for (const page of pages) {
    const label = page.section ?? otherLabel;
    const last = sections[sections.length - 1];
    if (last && last.label === label) last.pages.push(page);
    else sections.push({ label, pages: [page] });
  }
  return sections;
}

// 頁尾出處行（manual-pages 契約）：最後一個非空段落以 `**出處**：` 開頭。出處名的
// 單一真相是索引 frontmatter 的 sources（契約規定兩者一致），內文這行只剝掉、不重解。
const SOURCES_LINE_RE = /^\*\*出處\*\*[：:]/;

/** 剝掉內文尾端的出處行；沒有該行時原樣回傳。 */
export function stripSourcesLine(body: string): string {
  const lines = body.split("\n");
  let last = lines.length - 1;
  while (last >= 0 && lines[last].trim() === "") last--;
  if (last < 0 || !SOURCES_LINE_RE.test(lines[last].trim())) return body;
  return lines.slice(0, last).join("\n");
}

/** 從內文開頭抽出 H1：第一個非空行為 `# 標題` 時回傳（去掉該行的內文、標題文字），
 * 否則標題為 null。標題進固定頁首，內文不重複呈現。 */
export function splitLeadingHeading(body: string): { body: string; heading: string | null } {
  const lines = body.split("\n");
  let first = 0;
  while (first < lines.length && lines[first].trim() === "") first++;
  const match = first < lines.length ? /^#\s+(.+?)\s*#*\s*$/.exec(lines[first]) : null;
  if (!match) return { body, heading: null };
  return { body: lines.slice(first + 1).join("\n"), heading: match[1] };
}

// 跨頁連結（manual-pages 契約「內文慣例」：相對檔名 `[認識畫面](layout.md)`，可帶錨點）。
const MANUAL_LINK_RE = /^([A-Za-z0-9._-]+)\.md(?:#.*)?$/;

/** 相對檔名連結 → 目標頁 slug；外部網址、純錨點等其他 href 回 null。 */
export function manualLinkSlug(href: string): string | null {
  return MANUAL_LINK_RE.exec(href)?.[1] ?? null;
}
