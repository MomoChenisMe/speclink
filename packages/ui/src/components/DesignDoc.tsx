import { useEffect, useRef, useState } from "react";

import {
  itemMarkdown,
  listItemsOf,
  parseContractRows,
  parseDecisions,
  parseGoalsNonGoals,
  parseListItems,
  splitH2Sections,
  splitRisk,
  type DecisionData,
  type H2Section,
} from "../docModel";
import { useI18n } from "../i18n";
import { Callout } from "./ui/callout";
import { Card } from "./ui/card";
import {
  DocSection,
  ExpandToggle,
  InlineMarks,
  KeyValueRows,
  ListBlocks,
  ListSection,
  NAME_CHIP_CLS,
  RowList,
  SECTION_HEADING_CLS,
  SUB_LABEL_CLS,
} from "./DocParts";
import { Markdown } from "./Markdown";

/** 跳到某張決策卡的請求：每次點 D 籤 seq 遞增，同一張卡可重跳。 */
export interface DecisionJump {
  id: string;
  seq: number;
}

export interface DesignDocProps {
  content: string | null;
  empty?: string;
  jump: DecisionJump | null;
  /** 跳轉處理完（命中或沒命中）——頁據此清掉請求。 */
  onJumped: () => void;
}

/**
 * 設計渲染器（spec「提案與設計章節以中文標籤呈現」；design D4）：背景細線列、目標／非目標兩欄、
 * 決策可收合卡、實作契約鍵值列、風險逐條 warning 提示框；其餘章節照 Markdown。
 */
export function DesignDoc({ content, empty, jump, onJumped }: DesignDocProps) {
  const doc = content ? splitH2Sections(content) : null;
  const decisionIds = (doc?.sections ?? [])
    .filter((s) => s.key === "sections.decisions")
    .flatMap((s) => parseDecisions(s.content).cards.map((c) => c.id));
  const hit = jump !== null && decisionIds.includes(jump.id);
  // 沒命中：只回報完成（頁已切到設計文件、自頂端呈現）；命中由該卡自己的 effect 處理。
  useEffect(() => {
    if (jump && !hit) onJumped();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [jump]);
  if (!doc) return <Markdown content={content} empty={empty} />;
  return (
    <div data-design-doc>
      {doc.preamble && <Markdown content={doc.preamble} />}
      {doc.sections.map((s, i) => (
        <DesignSection key={i} section={s} jump={hit ? jump : null} onJumped={onJumped} />
      ))}
    </div>
  );
}

function DesignSection({ section, jump, onJumped }: { section: H2Section; jump: DecisionJump | null; onJumped: () => void }) {
  const { t } = useI18n();
  const { key, content } = section;
  const title = key ? t(key) : section.title;
  if (key === "sections.context") {
    return <ListSection section={section} title={title} renderList={(items) => <RowList items={items.map(itemMarkdown)} />} />;
  }
  if (key === "sections.risks") {
    return (
      <ListSection
        section={section}
        title={title}
        renderList={(items) => (
          <div className="flex flex-col gap-2.5">
            {items.map((item, i) => {
              const { risk, mitigation } = splitRisk(itemMarkdown(item));
              return (
                <Callout key={i} data-risk tone="warning" title={<InlineMarks text={risk} />} className="my-0">
                  {mitigation && <Markdown content={mitigation} className="[&_p]:my-0" />}
                </Callout>
              );
            })}
          </div>
        )}
      />
    );
  }
  if (key === "sections.goalsNonGoals") {
    const split = parseGoalsNonGoals(content);
    if (split) {
      const columns = [
        { mark: "goals", label: "doc.goals", md: split.goals },
        { mark: "nonGoals", label: "doc.nonGoals", md: split.nonGoals },
      ];
      return (
        <DocSection sectionKey={key} title={title}>
          {split.intro && <Markdown content={split.intro} />}
          <div className="grid grid-cols-2 gap-4">
            {columns.map((col) => {
              const blocks = parseListItems(col.md);
              return (
                <div key={col.mark} data-goals-column={col.mark} className="flex min-w-0 flex-col gap-2">
                  <div className={SUB_LABEL_CLS}>{t(col.label).replace("{n}", String(listItemsOf(blocks).length))}</div>
                  <ListBlocks blocks={blocks} renderList={(items) => <RowList items={items.map(itemMarkdown)} />} />
                </div>
              );
            })}
          </div>
        </DocSection>
      );
    }
  }
  if (key === "sections.decisions") {
    const { intro, cards } = parseDecisions(content);
    return (
      <DocSection sectionKey={key} title={title} count={cards.length > 0 ? cards.length : undefined}>
        <div className="flex flex-col gap-2.5">
          {intro && <Markdown content={intro} />}
          {cards.map((card, i) => (
            <DecisionCard key={i} card={card} mark={card.id ?? String(i + 1)} jump={jump} onJumped={onJumped} />
          ))}
        </div>
      </DocSection>
    );
  }
  if (key === "sections.contract") {
    const rows = parseContractRows(content);
    if (rows) {
      return (
        <DocSection sectionKey={key} title={title}>
          <KeyValueRows rows={rows} />
        </DocSection>
      );
    }
  }
  return (
    <DocSection sectionKey={key} title={title}>
      {content && <Markdown content={content} />}
    </DocSection>
  );
}

/**
 * 可收合的決策卡：預設收合，點卡頭或 D 籤跳轉才展開。收合只藏內文、標題常駐——本頁目錄的
 * 錨點集合不隨展開收合改變。捲動放在卡自己的 effect：Radix 的 TabsContent 切換後下一輪才掛內文。
 */
function DecisionCard({
  card,
  mark,
  jump,
  onJumped,
}: {
  card: DecisionData;
  mark: string;
  jump: DecisionJump | null;
  onJumped: () => void;
}) {
  const { t, tn } = useI18n();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!jump || jump.id !== card.id) return;
    setOpen(true);
    ref.current?.scrollIntoView?.({ block: "start" });
    onJumped();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [jump]);
  return (
    <Card ref={ref} size="nested" data-decision={mark} className="overflow-hidden">
      <ExpandToggle open={open} onToggle={() => setOpen((v) => !v)} className="gap-2 px-4 py-3">
        {card.id && <span className={NAME_CHIP_CLS}>{card.id}</span>}
        <h3 data-doc-heading className={`min-w-0 flex-1 ${SECTION_HEADING_CLS}`}>
          <InlineMarks text={card.title} />
        </h3>
        <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
          {tn("doc.points", card.bullets)}
        </span>
      </ExpandToggle>
      {open && card.body && (
        <div className="border-t border-border/70 px-4 py-3">
          <Markdown content={card.body} />
        </div>
      )}
    </Card>
  );
}
