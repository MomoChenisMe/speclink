// 專案說明頁（desktop-settings-mode design D8；desktop-settings-editors design D2；spec「設定頁編輯
// 專案說明與產出規則」）：唯讀為 markdown 渲染全文（自成一頁，不收合）；編輯為 markdown 模式的
// CodeEditor，頁標題右側「編輯／預覽」切換＋取消＋儲存 ⌘S，預覽渲染草稿、編輯器維持掛載。
import { useState } from "react";
import {
  Card,
  Markdown,
  SettingsPageHeader,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  useI18n,
} from "@speclink/ui";

import { CodeEditor } from "../../components/CodeEditor";
import { CardEditControls, FieldHelp, PageAlerts, PageNote, useEditorSave } from "./parts";
import type { ProjectSettingsState } from "./useProjectSettings";

type EditView = "edit" | "preview";

const TOGGLE_TRIGGER =
  "h-6 rounded-md border-0 mb-0 px-3 text-xs data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm";

export function ContextPage({ ps }: { ps: ProjectSettingsState }) {
  const { t } = useI18n();
  const [view, setView] = useState<EditView>("edit");
  const { saving, save } = useEditorSave(ps.ctxEditing, ps.saveContext);
  return (
    <Tabs
      data-testid="context-page"
      value={view}
      onValueChange={(next) => setView(next as EditView)}
      className="flex flex-col gap-6"
    >
      <SettingsPageHeader
        title={t("settings.projectSection.context")}
        description={t("settings.contextPageDesc")}
        actions={
          <CardEditControls
            editing={ps.ctxEditing}
            disabled={ps.wfDisabled || saving}
            msg={ps.ctxMsg}
            testPrefix="context"
            toggle={
              <TabsList variant={null} className="h-7 w-auto rounded-lg bg-muted p-0.5">
                <TabsTrigger value="edit" className={TOGGLE_TRIGGER}>
                  {t("settings.editor.edit")}
                </TabsTrigger>
                <TabsTrigger value="preview" className={TOGGLE_TRIGGER}>
                  {t("settings.editor.preview")}
                </TabsTrigger>
              </TabsList>
            }
            onEdit={() => {
              setView("edit");
              ps.beginCtxEdit();
            }}
            onCancel={() => ps.setCtxEditing(false)}
            onSave={() => void save()}
          />
        }
        note={<PageNote remote={ps.remoteNote} fileKey="openspec/config.yaml · context" testId="file-note-config" />}
      />
      <PageAlerts ps={ps} parseError={ps.snap.workflow.parseError} />
      {ps.ctxEditing ? (
        <>
          <TabsContent
            value="edit"
            forceMount
            hidden={view !== "edit"}
            data-testid="context-input"
            className="flex flex-col gap-2"
          >
            <CodeEditor
              value={ps.draftContext}
              onChange={ps.setDraftContext}
              mode="markdown"
              readOnly={ps.wfDisabled}
              onSave={() => void save()}
              minLines={12}
              ariaLabel={t("settings.projectSection.context")}
              autoFocus
            />
            <FieldHelp>{t("settings.contextHelp")}</FieldHelp>
          </TabsContent>
          <TabsContent value="preview" data-testid="context-preview">
            <Card className="p-4">
              <Markdown content={ps.draftContext || null} empty={t("settings.contextEmpty")} />
            </Card>
          </TabsContent>
        </>
      ) : (
        <Card data-testid="context-readonly" className="p-4">
          <Markdown content={ps.contextText || null} empty={t("settings.contextEmpty")} />
        </Card>
      )}
    </Tabs>
  );
}
