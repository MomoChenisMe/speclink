// 審查規則頁（spec 需求「設定頁編輯審查規則」）：預設清單卡（模式＋可展開的 12 條名稱）與
// 專案規則卡（CRITICAL／WARNING／SUGGESTION 三個固定節）。編輯沿用產出規則頁的頁標題右側
// 編輯／取消／儲存 ⌘S 與純文字 CodeEditor；預設清單為兩個單選。12 條名稱取自快照（引擎提供）。
import { useState } from "react";
import { ChevronRight } from "lucide-react";
import {
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  SettingsCard,
  SettingsPageHeader,
  cn,
  useI18n,
} from "@speclink/ui";

import type { ReviewBaseline } from "../../adapter/workspace";
import { CodeEditor } from "../../components/CodeEditor";
import { CardEditControls, FieldHelp, PageAlerts, PageNote, useSaveShortcut } from "./parts";
import { REVIEW_LISTS, textToEntries, type ReviewList, type ProjectSettingsState } from "./useProjectSettings";

/** 嚴重度色章：與工單（TicketView）同語意同色——CRITICAL 破壞色、WARNING 琥珀、SUGGESTION 中性。 */
const SEVERITY_CLS: Record<ReviewList, string> = {
  critical: "bg-destructive/15 text-destructive",
  warning: `${SEMANTIC_SURFACE.warning} ${SEMANTIC_TONE.warning}`,
  suggestion: "bg-muted text-muted-foreground",
};

const BASELINES: ReviewBaseline[] = ["extend", "replace"];

function SeverityTag({ list }: { list: ReviewList }) {
  return (
    <span className={cn("rounded px-1.5 py-px font-mono text-[11px] font-semibold", SEVERITY_CLS[list])}>
      {list.toUpperCase()}
    </span>
  );
}

export function ReviewRulesPage({ ps }: { ps: ProjectSettingsState }) {
  const { t } = useI18n();
  const [showSmells, setShowSmells] = useState(false);
  const save = () => void ps.saveReview();
  useSaveShortcut(ps.reviewEditing, save);
  const countLabel = (n: number) => t("settings.rulesCount").replace("{count}", String(n));
  const smells = ps.snap.workflow.reviewBaselineSmells;
  const disabled = ps.wfDisabled || ps.reviewSaving;
  const total = REVIEW_LISTS.reduce((n, list) => n + ps.review[list].length, 0);

  return (
    <div data-testid="review-page" className="flex flex-col gap-6">
      <SettingsPageHeader
        title={t("settings.projectSection.review")}
        description={t("settings.reviewPageDesc")}
        actions={
          <CardEditControls
            editing={ps.reviewEditing}
            disabled={disabled}
            msg={ps.reviewMsg}
            testPrefix="review"
            onEdit={ps.beginReviewEdit}
            onCancel={() => ps.setReviewEditing(false)}
            onSave={save}
          />
        }
        note={<PageNote remote={ps.remoteNote} fileKey="openspec/config.yaml · review" testId="file-note-config" />}
      />
      <PageAlerts ps={ps} parseError={ps.snap.workflow.parseError} />

      <SettingsCard title={t("settings.reviewBaselineCard")}>
        {ps.reviewEditing ? (
          <fieldset className="m-0 flex flex-col gap-2 border-0 p-4">
            <legend className="sr-only">{t("settings.reviewBaselineCard")}</legend>
            {BASELINES.map((baseline) => {
              const checked = ps.draftReview.baseline === baseline;
              return (
                <label
                  key={baseline}
                  className={cn(
                    "flex items-start gap-2.5 rounded-lg border px-3 py-2.5",
                    checked ? "border-primary/50 bg-primary/5" : "border-border",
                  )}
                >
                  <input
                    type="radio"
                    name="review-baseline"
                    value={baseline}
                    checked={checked}
                    disabled={disabled}
                    onChange={() => ps.setDraftReview((prev) => ({ ...prev, baseline }))}
                    className="mt-1 h-4 w-4 accent-primary"
                  />
                  <span className="flex flex-col gap-0.5">
                    <span className="text-sm font-medium">{t(`settings.reviewBaseline.${baseline}`)}</span>
                    <span className="text-xs text-muted-foreground">{t(`settings.reviewBaseline.${baseline}Desc`)}</span>
                  </span>
                </label>
              );
            })}
          </fieldset>
        ) : (
          <div className="flex flex-col gap-2 p-4">
            <span
              data-testid="review-baseline-mode"
              className="self-start rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary"
            >
              {t(`settings.reviewBaseline.${ps.review.baseline}`)}
            </span>
            <p className="m-0 text-sm text-muted-foreground">{t(`settings.reviewBaseline.${ps.review.baseline}Desc`)}</p>
            <button
              type="button"
              aria-expanded={showSmells}
              onClick={() => setShowSmells((open) => !open)}
              className="-ml-1 flex items-center gap-1 self-start rounded px-1 text-sm text-primary hover:underline"
            >
              <ChevronRight aria-hidden className={cn("h-3.5 w-3.5 transition-transform", showSmells && "rotate-90")} />
              {t("settings.reviewSmellsShow").replace("{count}", String(smells.length))}
            </button>
            {showSmells && (
              <ul data-testid="review-smells" className="m-0 flex list-none flex-wrap gap-1.5 p-0">
                {smells.map((name) => (
                  <li key={name} className="rounded-md border border-border bg-muted/40 px-2 py-0.5 font-mono text-xs">
                    {name}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </SettingsCard>

      <SettingsCard title={t("settings.reviewRulesCard")}>
        {ps.reviewEditing ? (
          <div className="flex flex-col gap-4 p-4">
            {ps.reviewReplaceEmpty && (
              <p
                data-testid="review-replace-empty"
                role="note"
                className="m-0 rounded-lg border border-border bg-muted/40 px-3 py-2 text-sm"
              >
                {t("settings.reviewReplaceEmpty")}
              </p>
            )}
            {REVIEW_LISTS.map((list) => (
              <section key={list} data-testid={`review-input-${list}`} aria-label={list.toUpperCase()} className="flex flex-col gap-2">
                <div className="flex flex-wrap items-center gap-1.5 px-1 text-sm font-medium">
                  <SeverityTag list={list} />
                  <span
                    data-testid={`review-count-${list}`}
                    className="inline-flex h-4 items-center rounded-full bg-muted px-1.5 text-[11px] font-normal tabular-nums text-muted-foreground"
                  >
                    {countLabel(textToEntries(ps.draftReview[list]).length)}
                  </span>
                  <span className="text-xs font-normal text-muted-foreground">{t(`settings.reviewEffect.${list}`)}</span>
                </div>
                <CodeEditor
                  value={ps.draftReview[list]}
                  onChange={(text) => ps.setDraftReview((prev) => ({ ...prev, [list]: text }))}
                  mode="plain"
                  readOnly={disabled}
                  onSave={save}
                  minLines={3}
                  ariaLabel={list.toUpperCase()}
                />
              </section>
            ))}
            <FieldHelp>{t("settings.reviewHelp")}</FieldHelp>
          </div>
        ) : total === 0 ? (
          <p data-testid="review-empty" className="m-0 px-4 py-4 text-sm text-muted-foreground">
            {t("settings.reviewEmpty")}
          </p>
        ) : (
          <div className="flex flex-col gap-4 p-4">
            {REVIEW_LISTS.map((list) => (
              <div key={list} data-testid={`review-section-${list}`} className="flex flex-col gap-1">
                <div className="flex flex-wrap items-center gap-2">
                  <SeverityTag list={list} />
                  <span className="text-xs text-muted-foreground">{t(`settings.reviewEffect.${list}`)}</span>
                  <span className="ml-auto text-xs text-muted-foreground">{countLabel(ps.review[list].length)}</span>
                </div>
                {ps.review[list].length > 0 && (
                  <ul className="m-0 flex list-disc flex-col gap-0.5 pl-5 text-sm text-muted-foreground">
                    {ps.review[list].map((rule, i) => (
                      <li key={i}>{rule}</li>
                    ))}
                  </ul>
                )}
              </div>
            ))}
          </div>
        )}
      </SettingsCard>

      {!ps.reviewEditing && <FieldHelp>{t("settings.reviewNote")}</FieldHelp>}
    </div>
  );
}
