// 前端解析正典 spec.md 內封存時注入的 @trace 註解（archive.rs trace_block 產生），
// 抽出各區塊的 source 變更名，聚合至 spec 層級去重保序（design D1／D2）。

// 一個 @trace HTML 註解區塊：`<!-- @trace ... -->`（跨行、非貪婪至最近的 -->）。
const TRACE_BLOCK_RE = /<!--\s*@trace\b([\s\S]*?)-->/g;
// 區塊內的 `source: <名>` 行（前後空白容忍；名取到行尾去空白）。
const SOURCE_RE = /^\s*source:\s*(\S.*?)\s*$/m;

/**
 * 解析 raw markdown：回傳所有 @trace 區塊的 source 名，去重且依文件首次出現順序。
 * 缺 source 的畸形區塊靜默略過；無 @trace 或空輸入回空陣列。
 */
export function parseTraceSources(markdown: string | null | undefined): string[] {
  if (!markdown) return [];
  const seen = new Set<string>();
  const sources: string[] = [];
  for (const block of markdown.matchAll(TRACE_BLOCK_RE)) {
    const m = block[1].match(SOURCE_RE);
    if (!m) continue;
    const source = m[1];
    if (seen.has(source)) continue;
    seen.add(source);
    sources.push(source);
  }
  return sources;
}
