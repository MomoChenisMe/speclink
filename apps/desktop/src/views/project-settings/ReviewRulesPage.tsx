// 審查規則頁（spec 需求「設定頁編輯審查規則」）：預設清單卡（模式＋可展開的 12 條名稱）與
// 專案規則卡（CRITICAL／WARNING／SUGGESTION 三個固定節）。編輯沿用產出規則頁的頁標題右側
// 編輯／取消／儲存 ⌘S 與純文字 CodeEditor；預設清單是列右側的共用下拉（與產出流程頁同一元件），
// 展開鈕與產出流程頁的「檢視內容」同一種按鈕。12 條名稱取自快照（引擎提供）。
import { useState } from "react";
import {
  BaselineSmellList,
  Button,
  REVIEW_LISTS,
  SettingsCard,
  SettingsChoice,
  SettingsPageHeader,
  SettingsRow,
  SeverityTag,
  useI18n,
  type ReviewBaseline,
} from "@speclink/ui";

import { CodeEditor } from "../../components/CodeEditor";
import { CardEditControls, FieldHelp, PageAlerts, PageNote, useSaveShortcut } from "./parts";
import { textToEntries, type ProjectSettingsState } from "./useProjectSettings";

const BASELINES: ReviewBaseline[] = ["extend", "replace"];

export function ReviewRulesPage({ ps }: { ps: ProjectSettingsState }) {
  const { t, tn } = useI18n();
  const [showSmells, setShowSmells] = useState(false);
  const save = () => void ps.saveReview();
  useSaveShortcut(ps.reviewEditing, save);
  const countLabel = (n: number) => tn("settings.rulesCount", n);
  const smells = ps.snap.workflow.reviewBaselineSmells;
  const disabled = ps.wfDisabled || ps.reviewSaving;
  const total = REVIEW_LISTS.reduce((n, list) => n + ps.review[list].length, 0);
  const baseline = ps.reviewEditing ? ps.draftReview.baseline : ps.review.baseline;

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
        note={<PageNote remote={ps.remoteNote} fileKey={`${ps.configFile} · review`} testId="file-note-config" />}
      />
      <PageAlerts ps={ps} parseError={ps.snap.workflow.parseError} />

      <SettingsCard title={t("settings.reviewBaselineCard")}>
        <SettingsRow
          label={tn("settings.reviewBaselineLabel", smells.length)}
          htmlFor={ps.reviewEditing ? "review-baseline" : undefined}
          description={t(`settings.reviewBaseline.${baseline}Desc`)}
          control={
            ps.reviewEditing ? (
              <SettingsChoice
                id="review-baseline"
                value={baseline}
                options={BASELINES.map((value) => ({ value, label: t(`settings.reviewBaseline.${value}`) }))}
                disabled={disabled}
                onChange={(value) => ps.setDraftReview((prev) => ({ ...prev, baseline: value as ReviewBaseline }))}
              />
            ) : (
              <span
                data-testid="review-baseline-mode"
                className="rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary"
              >
                {t(`settings.reviewBaseline.${baseline}`)}
              </span>
            )
          }
        >
          <div className="flex flex-col items-start gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              aria-expanded={showSmells}
              className="text-sm font-normal text-muted-foreground hover:text-foreground"
              onClick={() => setShowSmells((open) => !open)}
            >
              {showSmells
                ? t("settings.reviewSmellsHide")
                : tn("settings.reviewSmellsShow", smells.length)}
            </Button>
            {showSmells && <BaselineSmellList data-testid="review-smells" names={smells} />}
          </div>
        </SettingsRow>
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
                  <SeverityTag severity={list.toUpperCase()} />
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
                  <SeverityTag severity={list.toUpperCase()} />
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
