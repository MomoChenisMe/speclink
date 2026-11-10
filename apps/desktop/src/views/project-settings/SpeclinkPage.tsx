// AI 工具頁（desktop-settings-mode design D8；spec「設定頁圖形化讀寫兩層設定」）：讀寫 .speclink.yaml
// 的 tools——「AI 工具」卡裡內建工具每個一列附開關，自訂工具描述子為不可編輯列。檔名只在頁標題下
// 的等寬註記。remote 專案沒有這一頁（tools 屬本機 checkout 概念）。
import {
  Button,
  SettingsCard,
  SettingsPageHeader,
  SettingsRow,
  Switch,
  useI18n,
} from "@speclink/ui";

import { BUILTIN_TOOLS } from "../../builtinTools";
import { PageAlerts } from "./parts";
import type { ProjectSettingsState } from "./useProjectSettings";

export function SpeclinkPage({ ps }: { ps: ProjectSettingsState }) {
  const { t } = useI18n();
  return (
    <>
      <SettingsPageHeader
        title={t("settings.projectSection.speclink")}
        description={t("settings.speclinkPageDesc")}
        note={
          <span data-testid="file-note-speclink" className="font-mono">
            .speclink.yaml · tools
          </span>
        }
      />
      <PageAlerts ps={ps} parseError={ps.snap.app.parseError} />
      <div data-testid="tools-card">
        <SettingsCard title={t("settings.toolsLabel")}>
          {BUILTIN_TOOLS.map((tool) => (
            <SettingsRow
              key={tool}
              label={tool}
              htmlFor={`tool-${tool}`}
              control={
                <Switch
                  id={`tool-${tool}`}
                  checked={ps.tools.includes(tool)}
                  disabled={ps.appDisabled}
                  onCheckedChange={(on) => ps.toggleTool(tool, on)}
                />
              }
            />
          ))}
          {ps.snap.app.customTools.map((name) => (
            <SettingsRow
              key={name}
              label={name}
              description={t("settings.customToolsLabel")}
              control={<span className="text-xs text-muted-foreground">{t("settings.readOnly")}</span>}
            />
          ))}
          <div className="flex items-center justify-end gap-2 px-4 py-3">
            <span className="mr-auto text-xs text-muted-foreground">{ps.appMsg ?? t("settings.toolsHelp")}</span>
            <Button type="button" size="sm" data-testid="save-app" disabled={ps.appDisabled} onClick={() => void ps.saveApp()}>
              {t("settings.save")}
            </Button>
          </div>
        </SettingsCard>
      </div>
    </>
  );
}
