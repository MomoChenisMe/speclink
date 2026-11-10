// 專案設定分類欄（desktop-settings-mode design D2）：專案設定模式下取代專案欄——「工作流程」組
// （上層分類「品質關卡」底下有審查規則、子代理模型兩個子項）與「專案」組 AI 工具（遠端專案無此組）。
// 組名與分類名一律是概念名，與應用程式設定相同；設定檔路徑只在各頁標題下的註記。解析失敗的檔所掛
// 分類項帶警示點（上層分類沒有自己的頁，不掛）。底部 notice 槽放技能檔提示卡（提示跨頁常駐，專案
// 設定頁也在）。
import type { ReactNode } from "react";
import { NavItem, SEMANTIC_TONE, SettingsGroupHeader, cn, useI18n } from "@speclink/ui";

import {
  PROJECT_SETTINGS_CATEGORIES,
  QUALITY_PARENT,
  type ProjectSettingsCategory,
  type ProjectSettingsSection,
} from "../settingsCatalog";

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
  /** 工作流設定檔解析失敗：「工作流程」組的分類項與子項帶警示點。 */
  configParseError: boolean;
  /** .speclink.yaml 解析失敗：AI 工具項帶警示點。 */
  speclinkParseError: boolean;
  onSelect: (section: ProjectSettingsSection) => void;
  /** 底部提示卡（null＝無）。 */
  notice?: ReactNode;
}) {
  const { t } = useI18n();
  const groups = [
    { group: "config", label: t("settings.group.workflow"), parseError: configParseError },
    { group: "project", label: t("settings.group.project"), parseError: speclinkParseError },
  ] as const;
  return (
    <nav aria-label={t("settings.projectColumnLabel")} className="flex w-[200px] shrink-0 flex-col gap-0.5 p-2 pb-3">
      {groups.map(({ group, label, parseError }) => {
        const items = PROJECT_SETTINGS_CATEGORIES.filter((c) => c.group === group && (c.remote || !remote));
        if (items.length === 0) return null;
        const item = ({ id, labelKey, icon: Icon, parent }: ProjectSettingsCategory) => (
          <NavItem
            key={id}
            variant={parent ? "sub" : "default"}
            icon={Icon && <Icon />}
            label={t(labelKey)}
            active={section === id}
            badge={parseError ? <WarningDot /> : undefined}
            onClick={() => onSelect(id)}
          />
        );
        return (
          <div key={group} className="flex flex-col gap-0.5 pb-2">
            <div className="pb-1 pt-1">
              <SettingsGroupHeader label={label} />
            </div>
            {items.map((category, index) => {
              if (!category.parent) return item(category);
              // 子項在目錄裡相鄰：第一個子項帶出上層分類與整組子項，其餘略過。
              if (items[index - 1]?.parent === category.parent) return null;
              const parentLabel = t(QUALITY_PARENT.labelKey);
              return (
                <div key={category.parent} role="group" aria-label={parentLabel} className="flex flex-col gap-0.5">
                  <NavItem icon={<QUALITY_PARENT.icon />} label={parentLabel} onClick={() => onSelect(QUALITY_PARENT.opens)} />
                  {items.filter((c) => c.parent === category.parent).map(item)}
                </div>
              );
            })}
          </div>
        );
      })}
      {notice && <div className="mt-auto">{notice}</div>}
    </nav>
  );
}
