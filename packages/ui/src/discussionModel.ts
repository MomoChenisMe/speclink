// 討論記錄的結構解析（desktop-discussion-doc-views design D1）：區段、輪、結論欄位、分刀、背景收尾行與文中名稱。
// 全部是純函式、不碰 React；形狀不符時回 null、空陣列或整條當一般列（逐項降級），不拋錯——呼叫端照 Markdown 排。
// 記錄格式由引擎（speclink-core discuss）與 discuss 技能擁有：區段與輪標題的切分規則與引擎逐字同源。
import { itemMarkdown, parseListItems, type ListBlock, type ListItem } from "./docModel";

/** 討論記錄的三個標準區段。 */
export interface DiscussionSections {
  context: string;
  rounds: string;
  conclusion: string;
}

/** 結構標題白名單——與引擎（speclink-core discuss）同一條規則：
 * 只有這三個整行標題是區段邊界，輪內文的其他「## 」行不得截斷區段。 */
const STRUCTURAL_HEADERS: readonly string[] = ["## Context", "## Rounds", "## Conclusion"];

/** Fenced code block 圍欄行（``` 或 ~~~ 開頭；容忍前導空白）——圍欄內的行不是結構。 */
function isFenceLine(line: string): boolean {
  const t = line.trimStart();
  return t.startsWith("```") || t.startsWith("~~~");
}

/** 逐行標記是否位於 fenced code block（圍欄行自身視為圍欄內）。 */
function fenceFlags(lines: string[]): boolean[] {
  let fence = false;
  return lines.map((l) => {
    if (isFenceLine(l)) {
      fence = !fence;
      return true;
    }
    return fence;
  });
}

/** 區段邊界：結構標題白名單＋pre-scaffold「## Round 」前綴容忍（與引擎同一條規則）。 */
function isSectionBoundary(line: string): boolean {
  return STRUCTURAL_HEADERS.includes(line) || line.startsWith("## Round ");
}

/**
 * 把討論記錄全文按 `## Context`／`## Rounds`／`## Conclusion` 切分。
 * 任一區段缺失（手寫、pre-scaffold 格式）回 `null`——呼叫端整篇以單一檢視退回。
 */
export function splitDiscussionSections(text: string): DiscussionSections | null {
  const lines = text.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const body = (name: string): string | null => {
    const start = lines.findIndex((l, i) => !inFence[i] && l.trimEnd() === `## ${name}`);
    if (start < 0) return null;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i++) {
      if (!inFence[i] && isSectionBoundary(lines[i].trimEnd())) {
        end = i;
        break;
      }
    }
    return lines.slice(start + 1, end).join("\n");
  };
  const context = body("Context");
  const rounds = body("Rounds");
  const conclusion = body("Conclusion");
  if (context === null || rounds === null || conclusion === null) return null;
  return { context, rounds, conclusion };
}

/** 輪欄位白名單——scaffold 固定四詞，其餘粗體前綴按內文照排。 */
export type RoundLabel = "Focus" | "Position" | "Ruled out" | "Open";
export const ROUND_LABELS: readonly RoundLabel[] = ["Focus", "Position", "Ruled out", "Open"];

/** 討論的一輪：卡頭（輪次/mode/日期）＋欄位（缺席欄位無該鍵）。 */
export interface DiscussionRound {
  round: number;
  mode: string;
  date: string;
  /** 首個欄位標籤前的內文（scaffold 記錄通常為空）。 */
  lead: string;
  fields: Partial<Record<RoundLabel, string>>;
}

/** 欄位切分結果：首個標籤前的內文（lead）＋標籤→內文對應。 */
export interface LabeledFields<L extends string> {
  lead: string;
  fields: Partial<Record<L, string>>;
}

/**
 * 以標籤白名單切欄位（輪與結論共用）：行首「**<Label>**:」起新欄位，
 * 其餘行（含非白名單的粗體前綴行）歸屬當前欄位；首個標籤前的內容歸 lead。
 */
export function splitLabeledFields<L extends string>(lines: string[], labels: readonly L[]): LabeledFields<L> {
  const re = new RegExp(`^\\*\\*(${labels.join("|")})\\*\\*:\\s?(.*)$`);
  const fields: Partial<Record<L, string>> = {};
  let lead = "";
  let field: L | null = null;
  const buf: string[] = [];

  const flush = () => {
    const content = buf.join("\n").trim();
    if (field) fields[field] = content;
    else lead = content;
    buf.length = 0;
  };

  for (const line of lines) {
    const m = re.exec(line);
    if (m) {
      flush();
      field = m[1] as L;
      buf.push(m[2]);
      continue;
    }
    buf.push(line);
  }
  flush();
  return { lead, fields };
}

/**
 * 把 Rounds 區段全文切成輪陣列（行掃描解析 scaffold）。
 * 每個 `### ` 標題必須符合「### Round N — <mode> (<date>)」；任一不符、或首輪前
 * 出現非註解內容即回 `null`——呼叫端整篇以單一 markdown 檢視退回。零輪回空陣列。
 */
export function splitRounds(text: string): DiscussionRound[] | null {
  const heading = /^### Round (\d+) — (.+?) \((.+)\)\s*$/;
  const rounds: { round: number; mode: string; date: string; body: string[] }[] = [];
  let current: { body: string[] } | null = null;

  const lines = text.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  for (const [i, line] of lines.entries()) {
    // 圍欄內的行不是輪標題——歸當前輪內文（首輪前出現則按既有規則整篇退回）。
    if (!inFence[i] && line.startsWith("### ")) {
      const m = heading.exec(line);
      if (!m) return null;
      current = { body: [] };
      rounds.push({ round: Number(m[1]), mode: m[2], date: m[3], body: current.body });
      continue;
    }
    if (!current) {
      // 首輪前僅容許空行與 scaffold 註解；其餘內容代表非標準記錄，整篇退回。
      if (line.trim() === "" || /^<!--.*-->\s*$/.test(line.trim())) continue;
      return null;
    }
    current.body.push(line);
  }
  return rounds.map((r) => ({ round: r.round, mode: r.mode, date: r.date, ...splitLabeledFields(r.body, ROUND_LABELS) }));
}

/** 結論欄位白名單——conclude scaffold 固定六詞。 */
export type ConclusionLabel = "Decision" | "Rationale" | "Rejected alternatives" | "Deferred" | "Capture to" | "Next";
export const CONCLUSION_LABELS: readonly ConclusionLabel[] = [
  "Decision",
  "Rationale",
  "Rejected alternatives",
  "Deferred",
  "Capture to",
  "Next",
];

/** 「none」起頭（`None.`、`none — …`；`none-legacy` 不算）。 */
const NONE_RE = /^none(?![\w-])/i;

/** 背景的 `Prior discussions:` 行列出的先前討論數；缺行、`none` 開頭（`None.`、`none — …`）或切不出名稱為 null。
 * 與背景的先前討論籤同一套名單切法（`parseContextMeta`）。 */
export function countPriorDiscussions(context: string): number | null {
  const prior = parseContextMeta(context).prior;
  return prior && prior.length > 0 ? prior.length : null;
}

const unique = (items: string[]) => [...new Set(items)];

/** 結論六欄位；一個白名單欄位都沒有時回 null（呼叫端整篇 Markdown 退回）。 */
export function parseConclusion(text: string): LabeledFields<ConclusionLabel> | null {
  const parsed = splitLabeledFields(text.split(/\r?\n/), CONCLUSION_LABELS);
  return Object.keys(parsed.fields).length > 0 ? parsed : null;
}

/** Position 與 Decision：第一段散文為定論句（首段即列點時為空）、其後頂層列點依序為細節；
 * 列點之後的散文段也算一條細節，不丟內容。 */
export function splitVerdict(md: string): { verdict: string; details: ListItem[] } {
  const blocks = parseListItems(md);
  const lead = blocks[0]?.kind === "prose" ? blocks[0].text : "";
  const details = blocks
    .slice(lead ? 1 : 0)
    .flatMap((b): ListItem[] => (b.kind === "list" ? b.items : [{ text: b.text, body: "" }]));
  return { verdict: lead, details };
}

/** 分刀卡：識別籤、範圍句、變更名籤、卡內細線列。 */
export interface CutCard {
  id: string;
  title: string;
  names: string[];
  items: string[];
}

/** 粗體開頭的分刀識別：`cut <編號>`（編號以數字起頭：`cut 3a`；`cutover`、`Cut scope` 不算）、`刀…`、`第 N 刀`。 */
const CUT_ID_RE = /^(?:cut\s*\d[^\s，,：:（(]*|第\s*[^\s刀]+?\s*刀|刀[^\s，,：:（(]*)/i;

/** 反引號片段（不含空白的才算名稱：`speclink plan` 這類指令不是變更名）。 */
const tickNames = (text: string) =>
  unique([...text.matchAll(/`([^`]+)`/g)].map((m) => m[1].trim()).filter((s) => /^\S+$/.test(s)));

/** 外圍整組括號（`（…）`／`(…)`，含巢狀）拿掉；不是一整組時原樣。 */
function unwrapParens(text: string): string {
  if (!/^[（(]/.test(text)) return text;
  let depth = 0;
  for (let i = 0; i < text.length; i++) {
    if ("（(".includes(text[i])) depth += 1;
    else if ("）)".includes(text[i]) && --depth === 0) return i === text.length - 1 ? text.slice(1, -1).trim() : text;
  }
  return text;
}

/** 首行為一張分刀卡時回卡，否則 null。 */
function cutCard(item: ListItem): CutCard | null {
  if (!item.text.startsWith("**")) return null;
  const close = item.text.indexOf("**", 2);
  if (close < 0) return null;
  const bold = item.text.slice(2, close);
  const after = item.text.slice(close + 2);
  const plain = bold.replace(/`[^`]*`/g, " ").replace(/\s+/g, " ").trim();
  const id = CUT_ID_RE.exec(plain);
  if (!id) return null;
  const scope = plain.slice(id[0].length).replace(/^[\s:：、—–-]+/, "").trim();
  const tail = after.replace(/^\s*[:：]\s*/, "").trim();
  // 範圍句取自粗體時（「**cut 2 外殼**（…）」）粗體之後全部是說明；否則以第一個「——」切範圍句與說明。
  const dash = scope ? -1 : tail.indexOf("——");
  const title = scope || (dash < 0 ? tail : tail.slice(0, dash).trim());
  const note = scope ? unwrapParens(tail) : dash < 0 ? "" : tail.slice(dash + 2).trim();
  const inBold = tickNames(bold);
  return {
    id: id[0],
    title,
    names: inBold.length > 0 ? inBold : tickNames(after),
    items: [...(note ? [note] : []), ...blockRows(parseListItems(item.body))],
  };
}

/** 決定細節中抽出分刀卡；其餘留在 rest。沒有命中時 cuts 為空陣列。 */
export function parseCuts(details: ListItem[]): { cuts: CutCard[]; rest: ListItem[] } {
  const cuts: CutCard[] = [];
  const rest: ListItem[] = [];
  for (const item of details) {
    const card = cutCard(item);
    if (card) cuts.push(card);
    else rest.push(item);
  }
  return { cuts, rest };
}

/** 否決與擱置的一列：第一個「——」或前後帶空白的「 — 」（discuss 技能檔寫「option — why it lost」）切方案與
 * 落敗理由；切不到時整條為方案。提案的替代方案走 docModel 的 splitAlternative，不經這裡。 */
export function splitOption(text: string): { option: string; reason: string | null } {
  const m = /——|\s—\s/.exec(text);
  if (!m) return { option: text.trim(), reason: null };
  return { option: text.slice(0, m.index).trim(), reason: text.slice(m.index + m[0].length).trim() || null };
}

/** 以頂層（反引號與括號之外）的分隔符切段。 */
function splitTopLevel(text: string, separators: string): string[] {
  const out: string[] = [];
  let cur = "";
  let depth = 0;
  let tick = false;
  for (const ch of text) {
    if (ch === "`") tick = !tick;
    else if (!tick && "（(「".includes(ch)) depth += 1;
    else if (!tick && "）)」".includes(ch)) depth = Math.max(0, depth - 1);
    if (!tick && depth === 0 && separators.includes(ch)) {
      out.push(cur);
      cur = "";
    } else cur += ch;
  }
  out.push(cur);
  return out.map((s) => s.trim()).filter(Boolean);
}

/** 每條列點與其間的散文段各一列（不丟內容）。 */
function blockRows(blocks: ListBlock[]): string[] {
  return blocks.flatMap((b) => (b.kind === "list" ? b.items.map(itemMarkdown) : [b.text]));
}

/** Ruled out、Open、Rejected alternatives、Deferred 的逐條：有列點時每條列點（與其間的散文段）一列，
 * 否則以頂層的「；」／「;」切段。 */
export function splitRows(md: string): string[] {
  const blocks = parseListItems(md);
  if (blocks.some((b) => b.kind === "list")) return blockRows(blocks);
  return splitTopLevel(md, "；;");
}

/** 記錄去向的籤：以頂層（括號之外）的頓號與逗號切項、去掉反引號；任一項含句號或換行（整段散文）時回 null。 */
export function captureTargets(md: string): string[] | null {
  const items = splitTopLevel(md, "、,，").map((s) => s.replace(/`/g, "").trim());
  return items.length === 0 || items.some((s) => /[。\n]/.test(s)) ? null : items;
}

/** 收斂標記：「無」或 none 起頭、其後緊接標點、空白或結尾（「無法…」「none-legacy」不算）。 */
const OPEN_NONE_RE = /^(?:無|none)(?![\p{L}\p{N}_-])/iu;

/** 待決：收斂標記時整條原文一列（「無：寫入結論」），否則逐條。 */
export function openItems(md: string): { none: boolean; items: string[] } {
  const text = md.trim();
  if (OPEN_NONE_RE.test(text)) return { none: true, items: [text] };
  return { none: false, items: splitRows(text) };
}

/** 輪的 mode（首次出現序、字串透傳）與日期統計。 */
export function roundStats(rounds: DiscussionRound[]): {
  modes: { mode: string; count: number }[];
  days: number;
  first: string | null;
  last: string | null;
} {
  const modes: { mode: string; count: number }[] = [];
  for (const r of rounds) {
    const hit = modes.find((m) => m.mode === r.mode);
    if (hit) hit.count += 1;
    else modes.push({ mode: r.mode, count: 1 });
  }
  return {
    modes,
    days: new Set(rounds.map((r) => r.date)).size,
    first: rounds[0]?.date ?? null,
    last: rounds[rounds.length - 1]?.date ?? null,
  };
}

/** 背景收尾四行：名單行缺席為 null、`none` 為空陣列；來源文件缺席為 null、`none` 為空字串。 */
export interface ContextMeta {
  specs: string[] | null;
  changes: string[] | null;
  prior: string[] | null;
  sourceDoc: string | null;
  /** 剝掉四行之後的內文。 */
  body: string;
}

const META_LINES = {
  "Related specs:": "specs",
  "Related changes:": "changes",
  "Prior discussions:": "prior",
} as const;
const SOURCE_DOC = "Source doc:";

/** 英數起頭的名稱：以連字號串起英數段或純漢字段（`config-context-與-rules-gui-編輯`）；英數段後緊接的中文
 * 不算進名稱（`desktop-detail-doc-tree定義了…`）。名單與指令參數共用。 */
const NAME_SRC = String.raw`[A-Za-z0-9][\w.]*(?:-(?:[\w.]+|\p{Script=Han}+))*`;
const NAME_HEAD_RE = new RegExp(`^${NAME_SRC}`, "u");

/** 名單：括號備註換成空白（名稱止於括號），以逗號、頓號、分號切項，每項取冒號前的第一個 token
 * （`add-copilot-tool: desktop-config`）；英數起頭時只取名稱段，句尾的句點拿掉。 */
function nameList(value: string): string[] {
  const list = value.replace(/[（(][^）)]*[）)]/g, " ").trim();
  if (NONE_RE.test(list)) return [];
  return unique(
    list
      .split(/[,，、;；]/)
      .map((item) => {
        const token = item.split(/[:：]/)[0].trim().split(/\s+/)[0];
        return (NAME_HEAD_RE.exec(token)?.[0] ?? token).replace(/[。.]+$/, "");
      })
      .filter(Boolean),
  );
}

/** 剝掉圍欄外、行首（容許前導空白）的 `Related specs:`／`Related changes:`／`Prior discussions:`／`Source doc:` 四行。 */
export function parseContextMeta(context: string): ContextMeta {
  const lines = context.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const meta: ContextMeta = { specs: null, changes: null, prior: null, sourceDoc: null, body: "" };
  const kept = lines.filter((line, i) => {
    if (inFence[i]) return true;
    const t = line.trimStart();
    if (t.startsWith(SOURCE_DOC)) {
      const value = t.slice(SOURCE_DOC.length).trim();
      meta.sourceDoc = NONE_RE.test(value) ? "" : value;
      return false;
    }
    const prefix = (Object.keys(META_LINES) as (keyof typeof META_LINES)[]).find((p) => t.startsWith(p));
    if (!prefix) return true;
    meta[META_LINES[prefix]] = nameList(t.slice(prefix.length));
    return false;
  });
  meta.body = kept.join("\n").trim();
  return meta;
}

/** 宿主清單裡的名稱：比對背景內文與收尾行用。 */
export interface ContextNames {
  specs: string[];
  active: string[];
  archived: string[];
  /** 其他討論的 slug（本身由 `self` 排除）。 */
  discussions: string[];
  self?: string;
}

/** 背景的關聯：收尾行與內文命中的聯集（在途與已封存變更只收宿主清單裡有的名稱）。 */
export interface ContextRelations {
  specs: string[];
  changes: string[];
  archived: string[];
  prior: string[];
  /** 內文命中（名稱籤用），依種類。 */
  hits: { specs: string[]; changes: string[]; archived: string[]; discussions: string[] };
}

/** 背景關聯的單一推導：ContextDoc 的關聯列與名稱籤、頁的數字磚共用。 */
export function contextRelations(meta: ContextMeta, names: ContextNames): ContextRelations {
  const discussions = names.discussions.filter((slug) => slug !== names.self);
  const hits = {
    specs: matchNames(meta.body, names.specs),
    changes: matchNames(meta.body, names.active),
    archived: matchNames(meta.body, names.archived),
    discussions: matchNames(meta.body, discussions),
  };
  const listed = meta.changes ?? [];
  return {
    specs: unique([...(meta.specs ?? []), ...hits.specs]),
    changes: unique([...listed.filter((n) => names.active.includes(n)), ...hits.changes]),
    archived: unique([...listed.filter((n) => names.archived.includes(n)), ...hits.archived]),
    prior: unique([...(meta.prior ?? []), ...hits.discussions]),
    hits,
  };
}

/** 背景文件的單一推導（ContextDoc 與頁的數字磚共用）：收尾行、關聯與比對所用的名稱清單。 */
export interface ContextModel {
  meta: ContextMeta;
  rel: ContextRelations;
  names: ContextNames;
}

export function contextModel(context: string, names: ContextNames): ContextModel {
  const meta = parseContextMeta(context);
  return { meta, rel: contextRelations(meta, names), names };
}

/** 引言段：圍欄外第一個空行之前的散文；第一段是列點、標題、圍欄、引言或表格時引言為空、全文進 rest。 */
export function splitLead(body: string): { lead: string; rest: string } {
  const lines = body.split(/\r?\n/);
  let start = 0;
  while (start < lines.length && !lines[start].trim()) start += 1;
  if (start === lines.length || /^\s*(?:[-*+]\s|\d+[.)]\s|#|>|\||```|~~~)/.test(lines[start])) {
    return { lead: "", rest: body };
  }
  let end = start;
  while (end < lines.length && lines[end].trim()) end += 1;
  return { lead: lines.slice(start, end).join("\n").trim(), rest: lines.slice(end).join("\n").trim() };
}

/** 名稱邊界：前後皆非英數、連字號與底線（中文名稱也用這條）。 */
const isBoundary = (ch: string | undefined) => ch === undefined || !/[A-Za-z0-9_-]/.test(ch);

/** 一段純文字中名稱的命中位置：長名優先、已被長名佔走的範圍不再命中，依出現序。Markdown 的名稱籤外掛共用。 */
export function nameSpans(text: string, names: string[]): { name: string; start: number; end: number }[] {
  const taken = new Array<boolean>(text.length).fill(false);
  const spans: { name: string; start: number; end: number }[] = [];
  for (const name of unique(names.filter(Boolean)).sort((a, b) => b.length - a.length)) {
    for (let at = text.indexOf(name); at >= 0; at = text.indexOf(name, at + 1)) {
      const end = at + name.length;
      if (!isBoundary(text[at - 1]) || !isBoundary(text[end]) || taken.slice(at, end).some(Boolean)) continue;
      taken.fill(true, at, end);
      spans.push({ name, start: at, end });
    }
  }
  return spans.sort((a, b) => a.start - b.start);
}

/** 圍欄與行內反引號之外的名稱命中，依首次出現序去重。 */
export function matchNames(text: string, names: string[]): string[] {
  const lines = text.split("\n");
  const inFence = fenceFlags(lines);
  // 程式碼換成等長空白：位置不動、邊界照算。
  const plain = lines
    .map((l, i) => (inFence[i] ? " ".repeat(l.length) : l.replace(/`[^`]*`/g, (m) => " ".repeat(m.length))))
    .join("\n");
  return unique(nameSpans(plain, names).map((s) => s.name));
}

const COMMAND_RE = new RegExp(String.raw`/speclink-[a-z-]+(?:\s+(?:--[a-z-]+|${NAME_SRC}))*`, "gu");

/** 下一步裡的 `/speclink-*` 指令（含旗標與參數；反引號內外皆算；句尾句點不算），去重保序。 */
export function commandChips(md: string): string[] {
  return unique((md.match(COMMAND_RE) ?? []).map((command) => command.replace(/\.+$/, "")));
}
