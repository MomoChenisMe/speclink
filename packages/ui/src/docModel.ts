// 變更文件的結構解析（desktop-change-doc-views design D1）：章節切分與各章節的列點、路徑、決策、
// 任務三層等形狀。全部是純函式、不碰 React；形狀不符時回 null 或把該列標成一般列（逐項降級），
// 不拋錯——呼叫端照 Markdown 排。逐行掃描且認得 fenced code block（與 specSections 共用 fenceFlags）。
import { fenceFlags } from "./specSections";

/** 已知模板章節（h2）→ i18n key：三型提案模板與設計模板的聯集（白名單留在前端、不讀 schema）。 */
const SECTION_KEYS: Record<string, string> = {
  Why: "sections.why",
  "What Changes": "sections.whatChanges",
  "Non-Goals": "sections.nonGoals",
  Capabilities: "sections.capabilities",
  Impact: "sections.impact",
  Problem: "sections.problem",
  "Root Cause": "sections.rootCause",
  "Proposed Solution": "sections.proposedSolution",
  "Success Criteria": "sections.successCriteria",
  Summary: "sections.summary",
  Motivation: "sections.motivation",
  "Alternatives Considered": "sections.alternatives",
  Context: "sections.context",
  "Goals / Non-Goals": "sections.goalsNonGoals",
  Decisions: "sections.decisions",
  "Implementation Contract": "sections.contract",
  "Risks / Trade-offs": "sections.risks",
  "Migration Plan": "sections.migration",
  "Open Questions": "sections.openQuestions",
};

/** Capabilities 段內的 h3 模板詞（子段）。 */
const SUB_SECTION_KEYS: Record<string, string> = {
  "New Capabilities": "sections.newCapabilities",
  "Modified Capabilities": "sections.modifiedCapabilities",
};

export interface DocSubsection {
  key: string;
  title: string;
  content: string;
}

/** 一個二級標題段：白名單命中時 key 為 i18n 鍵，白名單外為 null（title 為原文）。 */
export interface H2Section {
  key: string | null;
  title: string;
  /** 段內文（不含標題行；Capabilities 段為第一個子段之前的內文）。 */
  content: string;
  /** Capabilities 段的 New／Modified 子段；其他段為空陣列。 */
  subsections: DocSubsection[];
}

export interface SplitDoc {
  /** 第一個 h2 之前的內容。 */
  preamble: string;
  sections: H2Section[];
}

/** 每個 `## ` 行開一段；沒有任何 h2 回 null（呼叫端整篇 Markdown 退回）。 */
export function splitH2Sections(text: string): SplitDoc | null {
  const lines = text.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const preamble: string[] = [];
  const sections: { key: string | null; title: string; lines: string[]; subs: { key: string; title: string; lines: string[] }[] }[] = [];
  lines.forEach((line, i) => {
    const h2 = inFence[i] ? null : /^##\s+(.+?)\s*$/.exec(line);
    if (h2) {
      const title = h2[1].replace(/\s*\(optional\)\s*$/i, "").trim();
      sections.push({ key: SECTION_KEYS[title] ?? null, title, lines: [], subs: [] });
      return;
    }
    const cur = sections[sections.length - 1];
    if (!cur) {
      preamble.push(line);
      return;
    }
    const h3 = cur.key === "sections.capabilities" && !inFence[i] ? /^###\s+(.+?)\s*$/.exec(line) : null;
    const subKey = h3 ? SUB_SECTION_KEYS[h3[1]] : undefined;
    if (h3 && subKey) {
      cur.subs.push({ key: subKey, title: h3[1], lines: [] });
      return;
    }
    (cur.subs[cur.subs.length - 1]?.lines ?? cur.lines).push(line);
  });
  if (sections.length === 0) return null;
  const join = (ls: string[]) => ls.join("\n").trim();
  return {
    preamble: join(preamble),
    sections: sections.map((s) => ({
      key: s.key,
      title: s.title,
      content: join(s.lines),
      subsections: s.subs.map((sub) => ({ key: sub.key, title: sub.title, content: join(sub.lines) })),
    })),
  };
}

/** 頂層列點：首行文字與其後的續行／巢狀行（已去掉共同縮排）。 */
export interface ListItem {
  text: string;
  body: string;
}

/** 一段內容依出現序切成列點群與 prose 段。 */
export type ListBlock = { kind: "list"; items: ListItem[] } | { kind: "prose"; text: string };

const ITEM_RE = /^(?:[-*+]|\d+[.)])\s+(.*)$/;

function dedent(lines: string[]): string {
  const indents = lines.filter((l) => l.trim()).map((l) => /^\s*/.exec(l)![0].length);
  const cut = indents.length > 0 ? Math.min(...indents) : 0;
  return lines.map((l) => l.slice(cut)).join("\n").trim();
}

/** 頂層列點（`- `／`* `／`1. `）與 prose 段依序切開；巢狀與縮排續行歸入該項 body。 */
export function parseListItems(md: string): ListBlock[] {
  const lines = md.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const blocks: ListBlock[] = [];
  let prose: string[] = [];
  let list: { text: string; body: string[] }[] | null = null;
  let lastBlank = false;
  // 圍欄跟著開頭行走：開頭行縮排且在列點內＝歸該項，否則歸 prose。
  let fenceTarget: string[] | null = null;
  const flushProse = () => {
    const text = prose.join("\n").trim();
    if (text) blocks.push({ kind: "prose", text });
    prose = [];
  };
  const flushList = () => {
    if (list) blocks.push({ kind: "list", items: list.map((it) => ({ text: it.text, body: dedent(it.body) })) });
    list = null;
  };
  const current = () => (list ? list[list.length - 1] : null);
  lines.forEach((line, i) => {
    const blank = line.trim() === "";
    const indented = /^\s/.test(line);
    if (inFence[i]) {
      if (!fenceTarget) {
        if (current() && indented) fenceTarget = current()!.body;
        else {
          flushList();
          fenceTarget = prose;
        }
      }
      fenceTarget.push(line);
      lastBlank = false;
      return;
    }
    fenceTarget = null;
    const item = indented ? null : ITEM_RE.exec(line);
    if (item) {
      flushProse();
      list = list ?? [];
      list.push({ text: item[1].trim(), body: [] });
    } else if (blank) {
      (current()?.body ?? prose).push(line);
    } else if (current() && (indented || !lastBlank)) {
      current()!.body.push(line);
    } else {
      flushList();
      prose.push(line);
    }
    lastBlank = blank;
  });
  flushList();
  flushProse();
  return blocks;
}

/** 各段列點收成一串（計數與逐列渲染用）。 */
export function listItemsOf(blocks: ListBlock[]): ListItem[] {
  return blocks.flatMap((b) => (b.kind === "list" ? b.items : []));
}

/** 列點的完整內文：首行加續行。 */
export function itemMarkdown(item: ListItem): string {
  return item.body ? `${item.text}\n${item.body}` : item.text;
}

/** 替代方案一列：首個「——」切方案與落敗理由；切不到時整條為方案。 */
export function splitAlternative(text: string): { option: string; reason: string | null } {
  const i = text.indexOf("——");
  if (i < 0) return { option: text, reason: null };
  return { option: text.slice(0, i).trim(), reason: text.slice(i + 2).trim() || null };
}

/** 「(none)」「無」「（無）」這類空值標記。 */
const NONE_RE = /^[（(]?\s*(none|無)\s*[)）]?[。.]?$/i;
const CAP_NAME_RE = /^[a-z0-9][a-z0-9-]*$/;

/** 以「、」「,」「，」切項，括號與引號內的分隔符不切。 */
function splitTopLevel(value: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let cur = "";
  for (const ch of value) {
    if ("（(「".includes(ch)) depth += 1;
    else if ("）)」".includes(ch)) depth = Math.max(0, depth - 1);
    if (depth === 0 && "、,，".includes(ch)) {
      out.push(cur);
      cur = "";
    } else cur += ch;
  }
  out.push(cur);
  return out.map((s) => s.replace(/`/g, "").trim().replace(/。$/, "")).filter(Boolean);
}

/** 一個項：本體與緊接的括號附註（如「a.ts（整合測試）」）。本體含空白或其後還有文字時回 null。 */
function withNote(token: string): { head: string; note: string | null } | null {
  const m = /^([^\s（(]+)\s*(?:[（(](.*)[）)])?$/.exec(token);
  return m ? { head: m[1], note: m[2]?.trim() || null } : null;
}

export interface ImpactPath {
  path: string;
  note: string | null;
}

export interface ImpactData {
  /** 「Affected specs:」列：names 為能力名（不成形為 null，照 raw 原文顯示）；整列缺席為 null。 */
  specs: { names: string[] | null; raw: string } | null;
  code: { new: ImpactPath[]; modified: ImpactPath[]; removed: ImpactPath[] };
  /** 兩列之外的其他列點（照一般列排）。 */
  notes: ListItem[];
}

/** Impact 段：「Affected code:」之下的 New／Modified／Removed 三組路徑是必要形狀，缺了或路徑不成形回 null。 */
export function parseImpact(md: string): ImpactData | null {
  let specs: ImpactData["specs"] = null;
  let code: ImpactData["code"] | null = null;
  const notes: ListItem[] = [];
  for (const block of parseListItems(md)) {
    if (block.kind === "prose") {
      notes.push({ text: block.text, body: "" });
      continue;
    }
    for (const item of block.items) {
      const spec = /^Affected specs\s*[:：]\s*(.*)$/i.exec(item.text);
      const affected = /^Affected code\s*[:：]\s*$/i.exec(item.text);
      if (spec) {
        const raw = spec[1].trim();
        if (NONE_RE.test(raw)) specs = { names: [], raw };
        else {
          const heads = splitTopLevel(raw).map(withNote);
          const ok = heads.length > 0 && heads.every((h) => h !== null && CAP_NAME_RE.test(h.head));
          specs = { names: ok ? heads.map((h) => h!.head) : null, raw };
        }
      } else if (affected) {
        const groups: ImpactData["code"] = { new: [], modified: [], removed: [] };
        const subs = listItemsOf(parseListItems(item.body));
        if (subs.length === 0) return null;
        for (const sub of subs) {
          const m = /^(New|Modified|Removed)\s*[:：]\s*(.*)$/i.exec(sub.text);
          if (!m) return null;
          const value = [m[2], sub.body].filter(Boolean).join(" ").trim();
          if (NONE_RE.test(value)) continue;
          for (const token of splitTopLevel(value)) {
            const p = withNote(token);
            if (!p) return null;
            groups[m[1].toLowerCase() as keyof ImpactData["code"]].push({ path: p.head, note: p.note });
          }
        }
        code = groups;
      } else notes.push(item);
    }
  }
  return code ? { specs, code, notes } : null;
}

/** 能力列：name 為能力名（不成形的列為 null，整列當一般列）。 */
export interface CapabilityRow {
  name: string | null;
  text: string;
}

/** Capabilities 子段：每個「`name`: 說明」列點；「(none)」為空。 */
export function parseCapabilities(md: string): CapabilityRow[] {
  return parseListItems(md).flatMap((block): CapabilityRow[] => {
    if (block.kind === "prose") return NONE_RE.test(block.text) ? [] : [{ name: null, text: block.text }];
    return block.items.map((item) => {
      const m = /^`?([a-z0-9][a-z0-9-]*)`?\s*[:：]\s*(.*)$/.exec(item.text);
      if (!m) return { name: null, text: itemMarkdown(item) };
      return { name: m[1], text: [m[2], item.body].filter(Boolean).join("\n") };
    });
  });
}

/** 提案 Capabilities 段「New Capabilities」列出的能力名——作用中變更裡它們還沒有正式規格。 */
export function newCapabilities(proposal: string): string[] {
  const caps = splitH2Sections(proposal)?.sections.find((s) => s.key === "sections.capabilities");
  const sub = caps?.subsections.find((s) => s.key === "sections.newCapabilities");
  return parseCapabilities(sub?.content ?? "").flatMap((r) => (r.name ? [r.name] : []));
}

const GOALS_RE = /^\s*\*\*Goals\s*[:：]?\*\*\s*[:：]?\s*(.*)$/i;
const NON_GOALS_RE = /^\s*\*\*Non-Goals\s*[:：]?\*\*\s*[:：]?\s*(.*)$/i;

/** Goals / Non-Goals 段：以兩個粗體標籤切兩欄（各為 Markdown 原文）；缺任一標籤回 null。 */
export function parseGoalsNonGoals(md: string): { intro: string; goals: string; nonGoals: string } | null {
  const lines = md.split(/\r?\n/);
  const g = lines.findIndex((l) => GOALS_RE.test(l));
  const n = lines.findIndex((l) => NON_GOALS_RE.test(l));
  if (g < 0 || n < 0 || n < g) return null;
  const column = (labelLine: string, re: RegExp, rest: string[]) =>
    [re.exec(labelLine)![1], ...rest].join("\n").trim();
  return {
    intro: lines.slice(0, g).join("\n").trim(),
    goals: column(lines[g], GOALS_RE, lines.slice(g + 1, n)),
    nonGoals: column(lines[n], NON_GOALS_RE, lines.slice(n + 1)),
  };
}

export interface DecisionData {
  /** 標題開頭的 D 編號；無則 null。 */
  id: string | null;
  /** 去掉 D 編號的標題。 */
  title: string;
  /** 卡內頂層列點數。 */
  bullets: number;
  body: string;
}

/** Decisions 段：每個 `### ` 開一張卡；第一張之前的內容為 intro。 */
export function parseDecisions(md: string): { intro: string; cards: DecisionData[] } {
  const lines = md.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const intro: string[] = [];
  const cards: { heading: string; lines: string[] }[] = [];
  lines.forEach((line, i) => {
    const h3 = inFence[i] ? null : /^###\s+(.+?)\s*$/.exec(line);
    if (h3) cards.push({ heading: h3[1], lines: [] });
    else (cards[cards.length - 1]?.lines ?? intro).push(line);
  });
  return {
    intro: intro.join("\n").trim(),
    cards: cards.map((c) => {
      const m = /^(D\d+)\b\s*[:：.、—-]?\s*(.*)$/.exec(c.heading);
      const body = c.lines.join("\n").trim();
      return {
        id: m ? m[1] : null,
        title: m && m[2] ? m[2] : c.heading,
        bullets: listItemsOf(parseListItems(body)).length,
        body,
      };
    }),
  };
}

/** Implementation Contract：「- **鍵**：值」列點成鍵值列；有 prose 或任一列不符回 null。 */
export function parseContractRows(md: string): { key: string; value: string }[] | null {
  const blocks = parseListItems(md);
  if (blocks.length === 0 || blocks.some((b) => b.kind === "prose")) return null;
  const rows: { key: string; value: string }[] = [];
  for (const item of listItemsOf(blocks)) {
    const m = /^\*\*(.+?)\*\*\s*[:：]\s*(.*)$/.exec(item.text);
    if (!m) return null;
    rows.push({ key: m[1], value: [m[2], item.body].filter(Boolean).join("\n") });
  }
  return rows;
}

/**
 * 風險一列：首個「→」切風險與對策（風險外圍的方括號剝除）；切不到時整條為風險。風險只取首行當標題，
 * 其餘行（巢狀子項、續行）接到對策之前。
 */
export function splitRisk(text: string): { risk: string; mitigation: string | null } {
  const i = text.indexOf("→");
  const head = (i < 0 ? text : text.slice(0, i)).trim();
  const bracketed = /^\[([\s\S]*)\]$/.exec(head);
  const [risk, ...rest] = (bracketed ? bracketed[1].trim() : head).split("\n");
  const mitigation = [rest.join("\n").trim(), i < 0 ? "" : text.slice(i + 1).trim()].filter(Boolean).join("\n");
  return { risk: risk.trim(), mitigation: mitigation || null };
}

export interface TaskTextParts {
  /** 描述開頭的「X.Y」序號；無則 null。 */
  number: string | null;
  title: string;
  body: string | null;
  verify: string | null;
  /** 標題緊接的 design 括號裡的 D 編號。 */
  decisions: string[];
  /** design 括號裡 D 編號以外的文字（歸入內文首行）。 */
  note: string | null;
}

/** 標題尾的「（design D1、D3；spec「…」）」或「(design D1, D3)」。 */
const DESIGN_PAREN_RE = /[（(]\s*design\s+([^）)]*)[）)]\s*$/i;

/** 描述開頭的「X.Y」序號（tasks.md 的寫法）。 */
const TASK_NUMBER_RE = /^(\d+(?:\.\d+)*)\s+([\s\S]*)$/;

/** 任務描述拆三層：先剝開頭序號，再以首個「驗證：」切驗證，再以首個全形「：」切標題與內文。 */
export function splitTaskText(raw: string): TaskTextParts {
  const numbered = TASK_NUMBER_RE.exec(raw);
  const text = numbered ? numbered[2] : raw;
  const v = text.indexOf("驗證：");
  const front = v < 0 ? text : text.slice(0, v);
  const verify = v < 0 ? null : text.slice(v + 3).trim() || null;
  const c = front.indexOf("：");
  let title = (c < 0 ? front : front.slice(0, c)).trim();
  const body = c < 0 ? null : front.slice(c + 1).trim() || null;
  const decisions: string[] = [];
  const notes: string[] = [];
  const paren = DESIGN_PAREN_RE.exec(title);
  if (paren) {
    title = title.slice(0, paren.index).trim();
    paren[1].split(/[；;]/).forEach((seg, i) => {
      if (i === 0) decisions.push(...(seg.match(/D\d+/g) ?? []));
      else if (seg.trim()) notes.push(seg.trim());
    });
  }
  return { number: numbered ? numbered[1] : null, title, body, verify, decisions, note: notes.join("；") || null };
}

const REMOVED_SCENARIO_RE = /^\s*<!--\s*REMOVED-SCENARIO:\s*(.*?)\s*-->\s*$/;
const BEFORE_RE = /^\s*<!--\s*BEFORE:.*-->\s*$/;

/** 取 REMOVED-SCENARIO 宣告的名稱，並回去掉這些行與 BEFORE 註解行的內文。 */
export function removedScenarios(block: string): { names: string[]; content: string } {
  const lines = block.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const names: string[] = [];
  const kept = lines.filter((line, i) => {
    if (inFence[i]) return true;
    const m = REMOVED_SCENARIO_RE.exec(line);
    if (m) names.push(m[1]);
    return !m && !BEFORE_RE.test(line);
  });
  return { names, content: kept.join("\n").trim() };
}

export type ScenarioStepLabel = "GIVEN" | "WHEN" | "THEN" | "AND";

const STEP_RE = /^[-*]\s+\*\*(GIVEN|WHEN|THEN|AND)\*\*\s*(.*)$/;

/** Scenario 開頭的「- **WHEN** …」等列點成步驟，其後的內容（如 Example 區塊）為 rest；沒有步驟回 null。 */
export function splitScenarioSteps(
  body: string,
): { steps: { label: ScenarioStepLabel; text: string }[]; rest: string } | null {
  const lines = body.split(/\r?\n/);
  const steps: { label: ScenarioStepLabel; text: string }[] = [];
  let i = 0;
  for (; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim()) continue;
    const m = STEP_RE.exec(line);
    if (m) steps.push({ label: m[1] as ScenarioStepLabel, text: m[2].trim() });
    else if (/^\s/.test(line) && steps.length > 0) steps[steps.length - 1].text += `\n${line.trim()}`;
    else break;
  }
  if (steps.length === 0) return null;
  return { steps, rest: lines.slice(i).join("\n").trim() };
}

export type FindingAxis = "Correctness" | "Standards" | "Coherence";

/** 工單描述開頭的軸向前綴（「Correctness:」等）：取出軸向並去前綴；無前綴時 axis 為 null。 */
export function findingAxis(text: string): { axis: FindingAxis | null; text: string } {
  const m = /^(Correctness|Standards|Coherence)\s*[:：]\s*/.exec(text);
  return m ? { axis: m[1] as FindingAxis, text: text.slice(m[0].length) } : { axis: null, text };
}

export type ProposalShape = "bugFix" | "refactor" | "feature";

/** 提案模板形狀：Problem＋Root Cause、Summary＋Motivation、Why＋What Changes；皆非為 null。 */
export function proposalShape(doc: SplitDoc): ProposalShape | null {
  const keys = new Set(doc.sections.map((s) => s.key));
  const has = (...names: string[]) => names.every((n) => keys.has(`sections.${n}`));
  if (has("problem", "rootCause")) return "bugFix";
  if (has("summary", "motivation")) return "refactor";
  if (has("why", "whatChanges")) return "feature";
  return null;
}
