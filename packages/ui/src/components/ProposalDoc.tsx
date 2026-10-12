import type { ReactNode } from "react";
import { Ban } from "lucide-react";

import {
  itemMarkdown,
  parseCapabilities,
  parseImpact,
  splitAlternative,
  splitH2Sections,
  type H2Section,
  type ImpactData,
  type ListItem,
} from "../docModel";
import { useI18n } from "../i18n";
import {
  DocSection,
  Lead,
  ListSection,
  NameChip,
  NumberedSteps,
  OptionReasonRows,
  PathChip,
  ROW_MD_CLS,
  RowList,
  SUB_LABEL_CLS,
  type PathReveal,
} from "./DocParts";
import { Markdown } from "./Markdown";

export interface ProposalDocProps {
  content: string | null;
  empty?: string;
  /** 本變更已載入 delta 規格的能力名：在其中的能力籤才可點。 */
  deltaCaps: string[];
  onOpenCapability?: (capability: string) => void;
  /** 宿主的本機路徑動作（文案依平台）；缺席時路徑籤只有複製鈕。 */
  pathReveal?: PathReveal;
  /** 接在能力段尾的內容（頁推導的改寫提示框）。 */
  afterCapabilities?: ReactNode;
}

/** 列式章節：列點成細線列，prose 照排。 */
const ROW_SECTIONS = new Set([
  "sections.why",
  "sections.motivation",
  "sections.problem",
  "sections.rootCause",
  "sections.successCriteria",
]);
const STEP_SECTIONS = new Set(["sections.whatChanges", "sections.proposedSolution"]);

const CODE_GROUP_KEYS = { new: "doc.codeNew", modified: "doc.codeModified", removed: "doc.codeRemoved" } as const;

/**
 * 提案渲染器（spec「提案與設計章節以中文標籤呈現」；design D3）：每個二級標題一段——白名單命中
 * 翻譯並換專屬畫法，白名單外以原文為標題照 Markdown 排；整份沒有二級標題才整篇退回 Markdown。
 */
export function ProposalDoc({ content, empty, deltaCaps, onOpenCapability, pathReveal, afterCapabilities }: ProposalDocProps) {
  const doc = content ? splitH2Sections(content) : null;
  if (!doc) return <Markdown content={content} empty={empty} />;
  const capChip = (name: string) => (
    <NameChip
      key={name}
      name={name}
      onClick={onOpenCapability && deltaCaps.includes(name) ? () => onOpenCapability(name) : undefined}
    />
  );
  return (
    <div data-proposal-doc>
      {doc.preamble && <Markdown content={doc.preamble} />}
      {doc.sections.map((s, i) => (
        <ProposalSection
          key={i}
          section={s}
          capChip={capChip}
          pathReveal={pathReveal}
          afterCapabilities={afterCapabilities}
        />
      ))}
    </div>
  );
}

function ProposalSection({
  section,
  capChip,
  pathReveal,
  afterCapabilities,
}: {
  section: H2Section;
  capChip: (name: string) => ReactNode;
  pathReveal?: PathReveal;
  afterCapabilities?: ReactNode;
}) {
  const { t } = useI18n();
  const { key, content } = section;
  const title = key ? t(key) : section.title;
  const listSection = (renderList: (items: ListItem[]) => ReactNode) => (
    <ListSection section={section} title={title} renderList={renderList} />
  );
  if (key === "sections.summary") {
    return (
      <DocSection sectionKey={key} title={title}>
        {content && (
          <Lead>
            <Markdown content={content} />
          </Lead>
        )}
      </DocSection>
    );
  }
  if (key && ROW_SECTIONS.has(key)) return listSection((items) => <RowList items={items.map(itemMarkdown)} />);
  if (key && STEP_SECTIONS.has(key)) return listSection((items) => <NumberedSteps items={items.map(itemMarkdown)} />);
  if (key === "sections.nonGoals") {
    return listSection((items) => <RowList items={items.map(itemMarkdown)} icon={<Ban className="h-3.5 w-3.5" />} />);
  }
  if (key === "sections.alternatives") {
    return listSection((items) => <OptionReasonRows items={items.map((it) => splitAlternative(itemMarkdown(it)))} />);
  }
  if (key === "sections.capabilities") {
    const groups = section.subsections.map((sub) => ({ sub, rows: parseCapabilities(sub.content) }));
    const named = groups.reduce((n, g) => n + g.rows.filter((r) => r.name).length, 0);
    return (
      <DocSection sectionKey={key} title={title} count={named > 0 ? named : undefined}>
        <div className="flex flex-col gap-4">
          {content && <Markdown content={content} />}
          {groups.map(({ sub, rows }) => (
            <div key={sub.key} data-capability-group={sub.key} className="flex flex-col gap-2">
              <h3 data-doc-heading className={SUB_LABEL_CLS}>
                {t(sub.key)}
              </h3>
              {rows.length === 0 ? (
                <p className="text-muted-foreground">{t("doc.none")}</p>
              ) : (
                <ul className="divide-y divide-border/70">
                  {rows.map((row, i) => (
                    <li key={i} className="flex items-start gap-2.5 py-2 first:pt-0 last:pb-0">
                      {row.name && <span className="mt-0.5">{capChip(row.name)}</span>}
                      {row.text && <Markdown content={row.text} className={ROW_MD_CLS} />}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          ))}
          {afterCapabilities}
        </div>
      </DocSection>
    );
  }
  if (key === "sections.impact") {
    const impact = parseImpact(content);
    if (!impact) return listSection((items) => <RowList items={items.map(itemMarkdown)} />);
    return (
      <DocSection sectionKey={key} title={title} count={impactFileCount(impact)}>
        <ImpactBody impact={impact} capChip={capChip} pathReveal={pathReveal} />
      </DocSection>
    );
  }
  return (
    <DocSection sectionKey={key} title={title}>
      {content && <Markdown content={content} />}
    </DocSection>
  );
}

/** 影響段的路徑總數（數字磚「影響的檔案」同一個數）。 */
export function impactFileCount(impact: ImpactData): number {
  return impact.code.new.length + impact.code.modified.length + impact.code.removed.length;
}

function ImpactBody({
  impact,
  capChip,
  pathReveal,
}: {
  impact: ImpactData;
  capChip: (name: string) => ReactNode;
  pathReveal?: PathReveal;
}) {
  const { t } = useI18n();
  const label = "w-20 shrink-0 pt-0.5 text-sm text-muted-foreground";
  return (
    <div className="flex flex-col gap-3">
      {impact.specs && (
        <div data-impact-specs className="flex items-start gap-3">
          <span className={label}>{t("doc.specsRow")}</span>
          <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
            {impact.specs.names === null ? (
              <Markdown content={impact.specs.raw} className="[&_p]:my-0" />
            ) : impact.specs.names.length === 0 ? (
              <span className="text-muted-foreground">{t("doc.none")}</span>
            ) : (
              impact.specs.names.map(capChip)
            )}
          </div>
        </div>
      )}
      {(["new", "modified", "removed"] as const).map((group) => {
        const paths = impact.code[group];
        const n = paths.length === 0 ? t("doc.none") : String(paths.length);
        return (
          <div key={group} data-impact-group={group} className="flex items-start gap-3">
            <span className={label}>{t(CODE_GROUP_KEYS[group]).replace("{n}", n)}</span>
            <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
              {/* 「移除」組的檔案在 apply 後已不存在，不給顯示鈕。 */}
              {paths.map((p) => (
                <PathChip key={p.path} path={p.path} note={p.note} reveal={group === "removed" ? undefined : pathReveal} />
              ))}
            </div>
          </div>
        );
      })}
      {impact.notes.length > 0 && <RowList items={impact.notes.map(itemMarkdown)} />}
    </div>
  );
}
