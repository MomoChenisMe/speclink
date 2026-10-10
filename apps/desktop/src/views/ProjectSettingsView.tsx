// 專案設定（spec 需求「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」；
// desktop-settings-mode design D8）：依專案設定分類欄的所在分類渲染一頁——專案說明、產出規則、
// 產出流程、.speclink.yaml（remote 無此頁）。狀態與動作在 useProjectSettings，四頁共用、切頁不重載。
import { useEffect } from "react";

import type { WorkspaceSettingsProvider } from "../session";
import type { ProjectSettingsSection } from "../settingsCatalog";
import { ContextPage } from "./project-settings/ContextPage";
import { RulesPage } from "./project-settings/RulesPage";
import { SpeclinkPage } from "./project-settings/SpeclinkPage";
import { WorkflowPage } from "./project-settings/WorkflowPage";
import { useProjectSettings } from "./project-settings/useProjectSettings";

export interface ProjectSettingsViewProps {
  /** 活躍 session 的設定面（root 已綁定；workspace-session 決策 3）。 */
  settings: WorkspaceSettingsProvider;
  /** 專案設定分類欄的所在分類（remote 專案的 .speclink.yaml 已由 App 退回專案說明）。 */
  section: ProjectSettingsSection;
  /** 回報兩層設定檔的解析失敗——專案設定分類欄據此掛警示點（desktop-settings-mode design D2）。 */
  onParseErrors?: (errors: { config: boolean; speclink: boolean }) => void;
  /** 回報有無未存修改——App 的返回據此先問要不要捨棄。 */
  onDirtyChange?: (dirty: boolean) => void;
}

export function ProjectSettingsView({ settings, section, onParseErrors, onDirtyChange }: ProjectSettingsViewProps) {
  const ps = useProjectSettings(settings);
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
      {section === "context" && <ContextPage ps={ps} />}
      {section === "rules" && <RulesPage ps={ps} />}
      {section === "workflow" && <WorkflowPage ps={ps} />}
      {section === "speclink" && <SpeclinkPage ps={ps} />}
    </div>
  );
}
