// 專案設定分類欄（desktop-settings-mode design D2）：專案設定模式下取代專案欄——群組
// 「openspec/config.yaml」三項與「專案」組 .speclink.yaml（遠端專案無此組）；解析失敗的檔
// 所掛分類項帶警示點。底部 notice 槽放技能檔提示卡（提示跨頁常駐，專案設定頁也在）。
import type { ReactNode } from "react";
import { NavItem, SEMANTIC_TONE, SettingsGroupHeader, cn, useI18n } from "@speclink/ui";

import { PROJECT_SETTINGS_CATEGORIES, type ProjectSettingsSection } from "../settingsCatalog";

/** 分類項的解析失敗警示點。 */
function WarningDot() {
  return (
    <span
      data-testid="tab-warning"
      aria-hidden="true"
      className={cn("inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-current", SEMANTIC_TONE.warning)}
    />
  );
}

export function ProjectSettingsColumn({
  section,
  remote,
  configParseError,
  speclinkParseError,
  onSelect,
  notice,
}: {
  section: ProjectSettingsSection;
  remote: boolean;
  /** openspec/config.yaml 解析失敗：專案說明、產出規則、產出流程三項帶警示點。 */
  configParseError: boolean;
  /** .speclink.yaml 解析失敗：.speclink.yaml 項帶警示點。 */
  speclinkParseError: boolean;
  onSelect: (section: ProjectSettingsSection) => void;
  /** 底部提示卡（null＝無）。 */
  notice?: ReactNode;
}) {
  const { t } = useI18n();
  const groups = [
    { group: "config", labelKey: "settings.group.configFile", parseError: configParseError },
    { group: "project", labelKey: "settings.group.project", parseError: speclinkParseError },
  ] as const;
  return (
    <nav aria-label={t("settings.projectColumnLabel")} className="flex w-[200px] shrink-0 flex-col gap-0.5 p-2 pb-3">
      {groups.map(({ group, labelKey, parseError }) => {
        const items = PROJECT_SETTINGS_CATEGORIES.filter((c) => c.group === group && (c.remote || !remote));
        if (items.length === 0) return null;
        return (
          <div key={group} className="flex flex-col gap-0.5 pb-2">
            <div className="pb-1 pt-1">
              <SettingsGroupHeader label={t(labelKey)} />
            </div>
            {items.map(({ id, labelKey: itemLabel, icon: Icon }) => (
              <NavItem
                key={id}
                icon={<Icon />}
                label={t(itemLabel)}
                active={section === id}
                badge={parseError ? <WarningDot /> : undefined}
                onClick={() => onSelect(id)}
              />
            ))}
          </div>
        );
      })}
      {notice && <div className="mt-auto">{notice}</div>}
    </nav>
  );
}
