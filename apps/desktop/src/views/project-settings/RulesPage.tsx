// 產出規則頁（desktop-settings-mode design D8；spec「設定頁編輯專案說明與產出規則」）：頁標題右側
// 編輯／取消／儲存；唯讀只列有條目的固定鍵（等寬鍵名＋條數），編輯為每個固定鍵一個文字區。
import { Card, SettingsPageHeader, Textarea, useI18n } from "@speclink/ui";

import { CardEditControls, FieldHelp, PageAlerts, PageNote } from "./parts";
import type { ProjectSettingsState } from "./useProjectSettings";

export function RulesPage({ ps }: { ps: ProjectSettingsState }) {
  const { t } = useI18n();
  return (
    <div data-testid="rules-page" className="flex flex-col gap-6">
      <SettingsPageHeader
        title={t("settings.projectSection.rules")}
        description={t("settings.rulesPageDesc")}
        actions={
          <CardEditControls
            editing={ps.rulesEditing}
            disabled={ps.wfDisabled}
            msg={ps.rulesMsg}
            testPrefix="rules"
            onEdit={ps.beginRulesEdit}
            onCancel={() => ps.setRulesEditing(false)}
            onSave={() => void ps.saveRules()}
          />
        }
        note={<PageNote remote={ps.remoteNote} fileKey="openspec/config.yaml · rules" testId="file-note-config" />}
      />
      <PageAlerts ps={ps} parseError={ps.snap.workflow.parseError} />
      <Card className="flex flex-col gap-3 p-4">
        {ps.rulesEditing ? (
          <>
            {/* 編輯面以凍結鍵集渲染（review N4），不跟 snap 的固定鍵換集。 */}
            {ps.editingRuleKeys.map((id) => (
              <div key={id} className="flex flex-col gap-1">
                <label htmlFor={`rules-input-${id}`} className="font-mono text-sm font-medium">
                  {id}
                </label>
                <Textarea
                  id={`rules-input-${id}`}
                  data-testid={`rules-input-${id}`}
                  value={ps.draftRules[id] ?? ""}
                  disabled={ps.wfDisabled}
                  rows={3}
                  className="font-mono"
                  onChange={(e) => ps.setDraftRules((prev) => ({ ...prev, [id]: e.target.value }))}
                />
              </div>
            ))}
            <FieldHelp>{t("settings.rulesHelp")}</FieldHelp>
          </>
        ) : ps.populatedRuleKeys.length === 0 ? (
          <p className="m-0 py-4 text-sm text-muted-foreground">{t("common.noContent")}</p>
        ) : (
          ps.populatedRuleKeys.map((id) => {
            const entries = ps.rules[id] ?? [];
            return (
              <div key={id} data-testid={`rules-readonly-${id}`} className="flex flex-col gap-1">
                <span className="flex items-baseline gap-2">
                  <span className="font-mono text-sm font-medium">{id}</span>
                  <span className="text-xs text-muted-foreground">
                    {t("settings.rulesCount").replace("{count}", String(entries.length))}
                  </span>
                </span>
                <ul className="m-0 flex list-disc flex-col gap-0.5 pl-5 text-sm text-muted-foreground">
                  {entries.map((entry, i) => (
                    <li key={i}>{entry}</li>
                  ))}
                </ul>
              </div>
            );
          })
        )}
      </Card>
    </div>
  );
}
