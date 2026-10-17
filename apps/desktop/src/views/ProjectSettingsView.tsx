// 專案設定（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」；
// desktop-settings-mode design D8）：依專案設定分類欄的所在分類渲染一頁——專案說明、產出規則、
// 審查規則、產出流程、.speclink.yaml（remote 無此頁）。狀態與動作在 useProjectSettings，各頁共用、
// 切頁不重載。
import { useEffect, useId, useState } from "react";
import { SettingsCard, useI18n } from "@speclink/ui";

import { PROJECT_COLORS, type ProjectColor } from "../projectColors";
import { projectTileClass } from "../components/projectTile";
import type { WorkspaceSettingsProvider } from "../session";
import type { ProjectSettingsSection } from "../settingsCatalog";
import { ContextPage } from "./project-settings/ContextPage";
import { ReviewRulesPage } from "./project-settings/ReviewRulesPage";
import { RulesPage } from "./project-settings/RulesPage";
import { SpeclinkPage } from "./project-settings/SpeclinkPage";
import { WorkflowPage } from "./project-settings/WorkflowPage";
import { useProjectSettings } from "./project-settings/useProjectSettings";

export interface ProjectSettingsViewProps {
  projectDirectory?: string | null;
  projectColor?: ProjectColor;
  onProjectColorChange?: (color: ProjectColor | null) => void | Promise<void>;
  /** 活躍 session 的設定面（root 已綁定；workspace-session 決策 3）。 */
  settings: WorkspaceSettingsProvider;
  /** 專案設定分類欄的所在分類（remote 專案的 .speclink.yaml 已由 App 退回專案說明）。 */
  section: ProjectSettingsSection;
  /** 回報兩層設定檔的解析失敗——專案設定分類欄據此掛警示點（desktop-settings-mode design D2）。 */
  onParseErrors?: (errors: { config: boolean; speclink: boolean }) => void;
  /** 回報有無未存修改——App 的返回據此先問要不要捨棄。 */
  onDirtyChange?: (dirty: boolean) => void;
}

export function ProjectSettingsView({ settings, section, onParseErrors, onDirtyChange, projectDirectory = null, projectColor, onProjectColorChange }: ProjectSettingsViewProps) {
  const { t } = useI18n();
  const ps = useProjectSettings(settings);
  const colorGroup = useId();
  const [colorError, setColorError] = useState<string | null>(null);
  const [colorSaving, setColorSaving] = useState(false);
  useEffect(() => { setColorError(null); }, [settings]);
  async function chooseColor(color: ProjectColor | null) {
    setColorSaving(true); setColorError(null);
    try { await onProjectColorChange?.(color); }
    catch (error) { setColorError(error instanceof Error ? error.message : String(error)); }
    finally { setColorSaving(false); }
  }
  const configParseError = ps !== null && ps.snap.workflow.parseError !== null;
  const speclinkParseError = ps !== null && ps.snap.app.parseError !== null;
  const dirty = ps !== null && ps.dirty;
  useEffect(() => {
    onParseErrors?.({ config: configParseError, speclink: speclinkParseError });
  }, [onParseErrors, configParseError, speclinkParseError]);
  useEffect(() => {
    onDirtyChange?.(dirty);
  }, [onDirtyChange, dirty]);
  // 卸載（離開專案設定、切到設定頁還沒就緒的分頁）時清掉回報：分類欄不沿用上一個專案的
  // 警示點，返回也不再問。
  useEffect(
    () => () => {
      onParseErrors?.({ config: false, speclink: false });
      onDirtyChange?.(false);
    },
    [onParseErrors, onDirtyChange],
  );

  if (!ps) return null;
  return (
    <div className="mx-auto flex w-full max-w-[768px] flex-col gap-6 px-6 pb-10 pt-7">
      <SettingsCard title={t("settings.projectDirectory")}>
        <span data-testid="project-directory" className="select-text font-mono text-sm text-muted-foreground [overflow-wrap:anywhere]" style={{ whiteSpace: "pre-wrap" }}>
          {projectDirectory ?? t("settings.noLocalDirectory")}
        </span>
      </SettingsCard>
      {onProjectColorChange && <SettingsCard title={t("settings.projectColor")}>
        <fieldset disabled={colorSaving} className="space-y-3 p-4">
          <legend className="sr-only">{t("settings.projectColor")}</legend>
          <p className="text-xs text-muted-foreground">{t("settings.projectColorHelp")}</p>
          <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
            {([null, ...PROJECT_COLORS] as const).map((color) => <label key={color ?? "default"} className={`flex items-center gap-2 rounded-md border p-2 text-sm focus-within:ring-2 focus-within:ring-ring ${projectColor === (color ?? undefined) ? "border-primary" : "border-border"}`}>
              <input type="radio" name={colorGroup} value={color ?? "default"} className="sr-only" checked={projectColor === (color ?? undefined)} onChange={() => void chooseColor(color)} />
              <span aria-hidden="true" className={`flex h-7 w-7 items-center justify-center rounded-md ${projectTileClass(color ?? undefined, projectColor === (color ?? undefined))}`}>A</span>
              {t(`settings.projectColor.${color ?? "default"}`)}
            </label>)}
          </div>
          {colorError && <p role="alert" className="text-sm text-destructive">{t("settings.projectColorSaveFailed")}: {colorError}</p>}
        </fieldset>
      </SettingsCard>}
      {section === "context" && <ContextPage ps={ps} />}
      {section === "rules" && <RulesPage ps={ps} />}
      {section === "review" && <ReviewRulesPage ps={ps} />}
      {section === "workflow" && <WorkflowPage ps={ps} />}
      {section === "speclink" && <SpeclinkPage ps={ps} />}
    </div>
  );
}
