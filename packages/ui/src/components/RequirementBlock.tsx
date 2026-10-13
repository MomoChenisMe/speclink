import { useState, type ReactNode } from "react";

import type { DeltaCounts } from "../delta";
import { splitScenarioSteps } from "../docModel";
import { useI18n } from "../i18n";
import { fenceFlags, type SpecRequirement } from "../specSections";
import { Badge } from "./ui/badge";
import { Callout } from "./ui/callout";
import { DELTA_BORDER } from "./DeltaBadges";
import { MoreToggle, SECTION_HEADING_CLS } from "./DocParts";
import { Markdown } from "./Markdown";

/** 收合列的可點提示：共用游標變數（網頁手指、桌面箭頭）、滑過淡底（與文件樹節點同一濃度）。 */
const TOGGLE_HOVER_CLS = "rounded-lg transition-colors hover:bg-foreground/5";

/** 「Requirement」小標：與需求文的 SHALL／MUST 關鍵字同款等寬灰底。 */
const KW_LABEL_CLS = "shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground";

/** delta 檢視的需求段設定；缺席＝規格詳情頁的畫法（不收合、無色條、無移除提示）。 */
export interface RequirementDelta {
  /** 左色條取該操作色。 */
  op: keyof DeltaCounts;
  /** 本需求以 REMOVED-SCENARIO 宣告移除的 scenario 名稱。 */
  removedScenarios: string[];
  defaultOpen: boolean;
}

export interface RequirementBlockProps {
  req: SpecRequirement;
  /** 標題列右端（規格詳情頁的溯源錨點）；放在收合鈕之外，點它不切換收合。 */
  anchor?: ReactNode;
  delta?: RequirementDelta;
  /** 非 delta 畫法的收合模式（規格詳情頁）；缺席＝恆展開、無收合鈕。`onToggle` 在每次展開／收合後
   * 呼叫（頁據以重掃本頁目錄：收起的內文不在 DOM 裡）。 */
  collapse?: { defaultOpen: boolean; onToggle?: () => void };
}

/** 超過這個數量的需求文段落與 Scenario 先收起來。 */
const VISIBLE = 3;

/** 以空行切段（圍欄內的空行不切）。 */
function paragraphs(md: string): string[] {
  const lines = md.split(/\r?\n/);
  const inFence = fenceFlags(lines);
  const out: string[][] = [[]];
  lines.forEach((line, i) => {
    if (!inFence[i] && line.trim() === "") out.push([]);
    else out[out.length - 1].push(line);
  });
  return out.map((p) => p.join("\n")).filter((p) => p.trim());
}

/**
 * 一條需求段（design D6）：「Requirement」小標＋標題＋右端錨點、需求文、左線縮排的 Scenario。
 * 規格詳情頁照原畫法，帶 `collapse` 時整段可收合為一列（desktop-spec-manual-pages design D3）；
 * delta 檢視另加操作色左色條、REMOVED-SCENARIO 提示框、WHEN／THEN 兩欄、超過三段需求文與三個
 * Scenario 時收合，且整段可收合為一列。兩種畫法的標題常駐、皆為本頁目錄錨點。
 */
export function RequirementBlock({ req, anchor, delta, collapse }: RequirementBlockProps) {
  const { t } = useI18n();
  const [open, setOpen] = useState(delta?.defaultOpen ?? collapse?.defaultOpen ?? true);
  const [allParagraphs, setAllParagraphs] = useState(false);
  const [allScenarios, setAllScenarios] = useState(false);
  if (!delta) {
    const head = (
      <>
        <span data-requirement-label className={`mt-0.5 ${KW_LABEL_CLS}`}>
          {t("sdetail.requirementLabel")}
        </span>
        <h2 data-doc-heading className={`min-w-0 flex-1 ${SECTION_HEADING_CLS}`}>
          {req.title}
        </h2>
      </>
    );
    return (
      <section data-requirement className="border-t border-border/70 pt-4">
        {/* 收合列的淡底蓋整列（含錨點）；左右各外推 8px，文字與未收合的段落對齊不變。 */}
        <div className={`flex min-w-0 items-start gap-2 ${collapse ? `-mx-2 ${TOGGLE_HOVER_CLS}` : ""}`}>
          {collapse ? (
            <button
              type="button"
              aria-expanded={open}
              onClick={() => {
                setOpen((v) => !v);
                collapse.onToggle?.();
              }}
              className="flex min-w-0 flex-1 cursor-(--cursor-interaction) items-start gap-2 px-2 py-1.5 text-left"
            >
              {head}
              {!open && (
                <Badge variant="secondary" className="shrink-0">
                  {t("doc.scenarioCount").replace("{n}", String(req.scenarios.length))}
                </Badge>
              )}
            </button>
          ) : (
            head
          )}
          {collapse ? <div className="shrink-0 py-1.5 pr-2">{anchor}</div> : anchor}
        </div>
        {open && req.body && (
          <div className="mt-2">
            <Markdown content={req.body} specKeywords />
          </div>
        )}
        {open &&
          req.scenarios.map((s, i) => (
            <div key={i} data-scenario className="mt-2.5 border-l-2 border-border pl-4">
              <div className="mb-1 text-xs font-semibold text-muted-foreground">
                {t("sdetail.scenarioLabel").replace("{title}", s.title)}
              </div>
              <Markdown content={s.body} specKeywords />
            </div>
          ))}
      </section>
    );
  }
  const removed = delta.removedScenarios;
  const paras = paragraphs(req.body);
  const shownParas = allParagraphs ? paras : paras.slice(0, VISIBLE);
  const shownScenarios = allScenarios ? req.scenarios : req.scenarios.slice(0, VISIBLE);
  return (
    <section data-requirement className={`border-l-4 ${DELTA_BORDER[delta.op]} py-1 pl-4`}>
      {/* 左側外推 8px 進色條內距，淡底不貼字、文字位置不變。 */}
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
        className={`-ml-2 flex w-[calc(100%+0.5rem)] min-w-0 cursor-(--cursor-interaction) items-start gap-2 px-2 py-1 text-left ${TOGGLE_HOVER_CLS}`}
      >
        <span data-requirement-label className={`mt-0.5 ${KW_LABEL_CLS}`}>
          {t("sdetail.requirementLabel")}
        </span>
        <h3 data-doc-heading className={`min-w-0 flex-1 ${SECTION_HEADING_CLS}`}>
          {req.title}
        </h3>
        {!open && (
          <span className="flex shrink-0 items-center gap-1.5">
            <Badge variant="secondary">{t("doc.scenarioCount").replace("{n}", String(req.scenarios.length))}</Badge>
            {removed.length > 0 && (
              <Badge variant="outline" className="text-destructive">
                {t("doc.removedScenariosChip").replace("{n}", String(removed.length))}
              </Badge>
            )}
          </span>
        )}
      </button>
      {open && (
        <div className="mt-2 flex flex-col gap-2">
          {removed.length > 0 && (
            <Callout
              data-removed-scenarios
              tone="caution"
              title={t("doc.removedScenarios").replace("{n}", String(removed.length))}
              className="my-0"
            >
              <ul className="my-1 flex flex-col gap-0.5">
                {removed.map((name) => (
                  <li key={name}>
                    <del>{name}</del>
                  </li>
                ))}
              </ul>
              <p className="text-sm text-muted-foreground">{t("doc.removedScenariosHint")}</p>
            </Callout>
          )}
          {shownParas.length > 0 && <Markdown content={shownParas.join("\n\n")} specKeywords />}
          {paras.length > shownParas.length && (
            <div>
              <MoreToggle
                label={t("doc.moreParagraphs").replace("{n}", String(paras.length - shownParas.length))}
                onExpand={() => setAllParagraphs(true)}
              />
            </div>
          )}
          {shownScenarios.map((s, i) => (
            <DeltaScenario key={i} title={s.title} body={s.body} />
          ))}
          {req.scenarios.length > shownScenarios.length && (
            <div>
              <MoreToggle
                label={t("doc.moreScenarios").replace("{n}", String(req.scenarios.length - shownScenarios.length))}
                onExpand={() => setAllScenarios(true)}
              />
            </div>
          )}
        </div>
      )}
    </section>
  );
}

/** delta 檢視的一個 Scenario：開頭的 WHEN／THEN 等列點成「標籤／內容」兩欄，其餘照 Markdown 接在下方。 */
function DeltaScenario({ title, body }: { title: string; body: string }) {
  const { t } = useI18n();
  const split = splitScenarioSteps(body);
  return (
    <div data-scenario className="border-l-2 border-border pl-4">
      <div className="mb-1 text-xs font-semibold text-muted-foreground">
        {t("sdetail.scenarioLabel").replace("{title}", title)}
      </div>
      {split ? (
        <>
          <div data-scenario-steps className="grid grid-cols-[4rem_1fr] gap-x-3 gap-y-1">
            {split.steps.map((step, i) => (
              <div key={i} className="contents">
                <span className="pt-1 font-mono text-xs lowercase text-muted-foreground">{step.label}</span>
                <Markdown content={step.text} specKeywords className="min-w-0 [&_p]:my-0" />
              </div>
            ))}
          </div>
          {split.rest && <Markdown content={split.rest} specKeywords />}
        </>
      ) : (
        <Markdown content={body} specKeywords />
      )}
    </div>
  );
}
