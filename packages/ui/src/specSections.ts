// 正式規格 spec.md 的分段解析（desktop-readonly-detail-pages design D1）：Purpose 區段、
// 逐條需求的 body／scenarios／段尾 @trace，與 SHALL／MUST 關鍵字的 rehype 標示外掛。
// 逐行掃描且認得 fenced code block——圍欄內的 `## `、`### Requirement:`、`---` 都不是結構。

/** 需求段的一個 Scenario：標題（去掉 `#### Scenario:` 前綴）與內文（含所屬 `##### Example`）。 */
export interface SpecScenario {
  title: string;
  body: string;
}

/** 一條需求：標題、需求文、Scenario 與段尾 @trace（缺席為 null；updated 只取日期部分）。 */
export interface SpecRequirement {
  title: string;
  body: string;
  scenarios: SpecScenario[];
  trace: { source: string; updated: string | null } | null;
}

export interface SpecSections {
  /** `## Purpose` 區段內文；缺席為空字串。 */
  purpose: string;
  /** 第一條需求之前、Purpose 之外的內容（如 `## Requirements` 下的引言）；缺席為空字串。 */
  intro: string;
  requirements: SpecRequirement[];
  /** 第一條需求之後、不屬任何需求段的內容（自 `## ` 標題起的其他區段）；缺席為空字串。 */
  appendix: string;
}

const REQUIREMENT_RE = /^### Requirement:\s*(.*?)\s*$/;
const SCENARIO_RE = /^#### Scenario:\s*(.*?)\s*$/;
const DATE_RE = /\d{4}-\d{2}-\d{2}/;

/** 圍欄行：3 個以上 ` 或 ~（容忍前導空白），其後為 info 字串。 */
const FENCE_RE = /^\s*(`{3,}|~{3,})(.*)$/;

/** 逐行標記是否位於 fenced code block（圍欄行自身視為圍欄內）。收尾圍欄須與開頭同字元、
 * 不短於開頭且其後只有空白——較短或另一種字元的圍欄行是內容（巢狀圍欄）。docModel 的
 * 章節與列點解析共用。 */
export function fenceFlags(lines: string[]): boolean[] {
  let open: string | null = null;
  return lines.map((l) => {
    const m = FENCE_RE.exec(l);
    if (open === null) {
      if (m) open = m[1];
      return m !== null;
    }
    if (m && m[1][0] === open[0] && m[1].length >= open.length && m[2].trim() === "") open = null;
    return true;
  });
}

/** `<!-- @trace … -->` 區塊內的 `key: value` 行。 */
function traceField(block: string[], key: string): string | null {
  for (const line of block) {
    const m = new RegExp(`^\\s*${key}:\\s*(\\S.*?)\\s*$`).exec(line);
    if (m) return m[1];
  }
  return null;
}

/** 一條需求的行（標題行之後到段尾）切成 body、scenarios 與 trace。 */
function parseRequirement(title: string, lines: string[], inFence: boolean[]): SpecRequirement {
  const body: string[] = [];
  const scenarios: { title: string; lines: string[] }[] = [];
  let trace: SpecRequirement["trace"] = null;
  let traceBlock: string[] | null = null;
  lines.forEach((line, i) => {
    if (!traceBlock && !inFence[i] && /^\s*<!--\s*@trace\b/.test(line)) traceBlock = [];
    if (traceBlock) {
      traceBlock.push(line);
      if (line.includes("-->")) {
        const source = traceField(traceBlock, "source");
        const updated = traceField(traceBlock, "updated");
        if (source) trace = { source, updated: updated ? (DATE_RE.exec(updated)?.[0] ?? updated) : null };
        traceBlock = null;
      }
      return;
    }
    if (!inFence[i]) {
      if (line.trim() === "---") return;
      // 只認 `#### Scenario:` 為邊界——正式規格的 h4 皆為 Scenario，其他 h4 原樣留在所在段。
      const sc = SCENARIO_RE.exec(line);
      if (sc) {
        scenarios.push({ title: sc[1], lines: [] });
        return;
      }
    }
    (scenarios[scenarios.length - 1]?.lines ?? body).push(line);
  });
  return {
    title,
    body: body.join("\n").trim(),
    scenarios: scenarios.map((s) => ({ title: s.title, body: s.lines.join("\n").trim() })),
    trace,
  };
}

/**
 * 以 `### Requirement:` 行切段（design D1）。第一段之前取 `## Purpose` 區段；每段自標題行
 * 到下一個需求標題或 `## ` 標題為止。不屬 Purpose 與需求段的內容不丟：`# ` 標題與
 * `## Requirements` 標題行之外，第一段之前的收進 intro、之後的收進 appendix。沒有任何需求
 * 標題回 null——呼叫端整篇以單一 markdown 檢視退回。
 */
export function parseSpecSections(markdown: string): SpecSections | null {
  const lines = markdown.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const isRequirement = (i: number) => !inFence[i] && REQUIREMENT_RE.test(lines[i]);
  // 區段終點：下一個需求標題或 `## ` 標題（圍欄外）。
  const sectionEnd = (from: number) => {
    for (let i = from; i < lines.length; i++) {
      if (isRequirement(i) || (!inFence[i] && lines[i].startsWith("## "))) return i;
    }
    return lines.length;
  };
  const starts = lines.flatMap((_, i) => (isRequirement(i) ? [i] : []));
  if (starts.length === 0) return null;
  const p = lines.findIndex((l, i) => !inFence[i] && l.trimEnd() === "## Purpose");
  // 已歸屬的行：文件標題、`## Requirements` 標題行、Purpose 區段與各需求段。
  const used = lines.map((l, i) => !inFence[i] && (/^# /.test(l) || l.trimEnd() === "## Requirements"));
  const claim = (from: number, to: number) => used.fill(true, from, to);
  const purposeEnd = p < 0 ? -1 : sectionEnd(p + 1);
  if (p >= 0) claim(p, purposeEnd);
  const requirements = starts.map((s) => {
    const end = sectionEnd(s + 1);
    claim(s, end);
    const title = REQUIREMENT_RE.exec(lines[s])![1];
    return parseRequirement(title, lines.slice(s + 1, end), inFence.slice(s + 1, end));
  });
  const rest = (from: number, to: number) =>
    lines.slice(from, to).filter((_, i) => !used[from + i]).join("\n").trim();
  return {
    purpose: p < 0 ? "" : lines.slice(p + 1, purposeEnd).join("\n").trim(),
    intro: rest(0, starts[0]),
    requirements,
    appendix: rest(starts[0], lines.length),
  };
}

/** 最小 hast 節點視圖（不引入型別套件相依）。 */
interface HastNode {
  type: string;
  value?: string;
  tagName?: string;
  properties?: Record<string, unknown>;
  children?: HastNode[];
}

const KEYWORD_RE = /\b(SHALL NOT|SHALL|MUST NOT|MUST)\b/g;

/** 一個文字節點切成文字與 `<code class="spec-kw">` 交錯的節點；無關鍵字回 null。 */
function splitKeywords(value: string): HastNode[] | null {
  const out: HastNode[] = [];
  let last = 0;
  for (const m of value.matchAll(KEYWORD_RE)) {
    if (m.index > last) out.push({ type: "text", value: value.slice(last, m.index) });
    out.push({
      type: "element",
      tagName: "code",
      properties: { className: ["spec-kw"] },
      children: [{ type: "text", value: m[0] }],
    });
    last = m.index + m[0].length;
  }
  if (out.length === 0) return null;
  if (last < value.length) out.push({ type: "text", value: value.slice(last) });
  return out;
}

/**
 * rehype 外掛（design D1）：把文字節點中的 `SHALL NOT`／`SHALL`／`MUST NOT`／`MUST`（整詞）包成
 * `<code class="spec-kw">`；行內 code 與程式碼區塊內不處理。只標示、不改來源。
 */
export function rehypeSpecKeywords() {
  const walk = (node: HastNode) => {
    if (!node.children) return;
    if (node.type === "element" && (node.tagName === "code" || node.tagName === "pre")) return;
    node.children = node.children.flatMap((child) => {
      if (child.type === "text" && child.value) return splitKeywords(child.value) ?? [child];
      walk(child);
      return [child];
    });
  };
  return walk;
}
