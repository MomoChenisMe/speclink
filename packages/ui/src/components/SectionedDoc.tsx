import { useI18n } from "../i18n";
import { Markdown } from "./Markdown";

/** 標籤家族的共用款式——粗體大標題（使用者比對裁定，design D6）：
 * 章節標籤、任務群組標題、輪／結論欄位標籤、色標區段標頭、封存區段標題同源。 */
export const LABEL_CLS = "text-xl font-bold";
/** 次級標籤（Capabilities 內的新增／修改能力）：略小一級的同族款式。 */
export const SUB_LABEL_CLS = "text-base font-bold";

/** 已知模板章節（h2）→ i18n key 對照——涵蓋三型提案模板與設計模板（design D2）。 */
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

/** Capabilities 節內的 h3 模板詞（次級標籤）。 */
const SUB_SECTION_KEYS: Record<string, string> = {
  "New Capabilities": "sections.newCapabilities",
  "Modified Capabilities": "sections.modifiedCapabilities",
};

interface DocSection {
  /** null＝prose 段（未知標題連同內文照排）。 */
  labelKey: string | null;
  sub?: boolean;
  content: string;
}

/** 行掃描切章節（design D1）：h2 白名單命中成標籤段，未知標題併入 prose 段。 */
function splitDocSections(text: string): DocSection[] {
  const sections: DocSection[] = [];
  let cur: DocSection = { labelKey: null, content: "" };
  let buf: string[] = [];
  const flush = () => {
    const content = buf.join("\n").trim();
    if (cur.labelKey !== null || content) sections.push({ ...cur, content });
    buf = [];
  };
  for (const line of text.split(/\r?\n/)) {
    const h2 = /^##\s+(.+?)\s*$/.exec(line);
    if (h2) {
      // 模板附註（如「Non-Goals (optional)」）於比對前剝除。
      const title = h2[1].replace(/\s*\(optional\)\s*$/i, "").trim();
      const key = SECTION_KEYS[title];
      flush();
      cur = { labelKey: key ?? null, content: "" };
      if (!key) buf.push(line);
      continue;
    }
    const h3 = /^###\s+(.+?)\s*$/.exec(line);
    if (h3) {
      const key = SUB_SECTION_KEYS[h3[1].trim()];
      if (key) {
        flush();
        cur = { labelKey: key, sub: true, content: "" };
        continue;
      }
    }
    buf.push(line);
  }
  flush();
  return sections;
}

/**
 * 模板文件的章節標籤檢視（spec「提案與設計章節以中文標籤呈現」）：已知模板章節
 * 渲染為與討論側結論欄位同款的標籤區塊，英文模板標題不直出；未知章節照 prose 排；
 * 整份無白名單命中時整篇單一 markdown 檢視退回。RichDetailDrawer 與 ArchivedList
 * 的提案／設計分頁共用。
 */
export function SectionedDoc({ content, empty }: { content: string | null; empty?: string }) {
  const { t } = useI18n();
  if (!content || !content.trim()) return <Markdown content={content} empty={empty} />;
  const sections = splitDocSections(content);
  if (!sections.some((s) => s.labelKey !== null)) return <Markdown content={content} empty={empty} />;
  return (
    <div>
      {sections.map((s, i) =>
        s.labelKey === null ? (
          <Markdown key={i} content={s.content} />
        ) : (
          <div key={i} data-section={s.labelKey} className={s.sub ? "mt-3" : "mt-5 first:mt-0"}>
            <div className={`${s.sub ? SUB_LABEL_CLS : LABEL_CLS} mb-1`}>{t(s.labelKey)}</div>
            {s.content && <Markdown content={s.content} />}
          </div>
        ),
      )}
    </div>
  );
}
