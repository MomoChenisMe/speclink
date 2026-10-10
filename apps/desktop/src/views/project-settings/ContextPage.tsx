// 專案說明頁（desktop-settings-mode design D8；spec「設定頁編輯專案說明與產出規則」）：頁標題右側
// 編輯／取消／儲存；唯讀為 markdown 渲染（超長收合、顯示更多），編輯為多行文字區。
import { Button, Card, Markdown, SettingsPageHeader, Textarea, cn, useI18n } from "@speclink/ui";

import { CardEditControls, FieldHelp, PageAlerts, PageNote } from "./parts";
import type { ProjectSettingsState } from "./useProjectSettings";

export function ContextPage({ ps }: { ps: ProjectSettingsState }) {
  const { t } = useI18n();
  return (
    <div data-testid="context-page" className="flex flex-col gap-6">
      <SettingsPageHeader
        title={t("settings.projectSection.context")}
        description={t("settings.contextPageDesc")}
        actions={
          <CardEditControls
            editing={ps.ctxEditing}
            disabled={ps.wfDisabled}
            msg={ps.ctxMsg}
            testPrefix="context"
            onEdit={ps.beginCtxEdit}
            onCancel={() => ps.setCtxEditing(false)}
            onSave={() => void ps.saveContext()}
          />
        }
        note={<PageNote remote={ps.remoteNote} fileKey="openspec/config.yaml · context" testId="file-note-config" />}
      />
      <PageAlerts ps={ps} parseError={ps.snap.workflow.parseError} />
      <Card className="flex flex-col gap-2 p-4">
        {ps.ctxEditing ? (
          <>
            <Textarea
              data-testid="context-input"
              value={ps.draftContext}
              disabled={ps.wfDisabled}
              rows={12}
              className="font-mono"
              onChange={(e) => ps.setDraftContext(e.target.value)}
            />
            <FieldHelp>{t("settings.contextHelp")}</FieldHelp>
          </>
        ) : (
          <>
            <div data-testid="context-readonly" className={cn(ps.contextCollapsed && "max-h-52 overflow-hidden")}>
              <Markdown content={ps.contextText || null} empty={t("settings.contextEmpty")} />
            </div>
            {ps.contextCollapsed && (
              <Button
                type="button"
                variant="link"
                data-testid="context-show-more"
                className="h-auto self-start p-0 text-xs font-normal"
                onClick={() => ps.setContextExpanded(true)}
              >
                {t("settings.showMore")}
              </Button>
            )}
          </>
        )}
      </Card>
    </div>
  );
}
