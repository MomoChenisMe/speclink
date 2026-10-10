// 設定分類欄（desktop-settings-mode design D2；desktop-about-page design D4）：設定模式下取代
// 專案欄——頂部全圓搜尋框，其下「應用程式」群組五個分類與「關於」群組；搜尋以分類名與各頁
// 列標籤過濾，只過濾分類、不跳到列，沒有命中的群組連標題一起隱藏。
import { useState } from "react";
import { NavItem, SearchField, SettingsGroupHeader, useI18n } from "@speclink/ui";

import { matchAppSettings, type AppSettingsCategory, type SettingsSection } from "../settingsCatalog";

const GROUPS: { id: AppSettingsCategory["group"]; labelKey: string }[] = [
  { id: "app", labelKey: "settings.group.app" },
  { id: "about", labelKey: "settings.group.about" },
];

export function SettingsColumn({
  section,
  onSelect,
}: {
  section: SettingsSection;
  onSelect: (section: SettingsSection) => void;
}) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const categories = matchAppSettings(query, t);
  return (
    <nav aria-label={t("settings.columnLabel")} className="flex w-[200px] shrink-0 flex-col gap-0.5 p-2 pb-3">
      <div className="pb-2">
        <SearchField value={query} onChange={setQuery} placeholder={t("settings.searchPlaceholder")} />
      </div>
      {GROUPS.map((group, index) => {
        const items = categories.filter((category) => category.group === group.id);
        if (items.length === 0) return null;
        return (
          <div key={group.id} className="flex flex-col gap-0.5">
            <div className={index === 0 ? "pb-1" : "pb-1 pt-3"}>
              <SettingsGroupHeader label={t(group.labelKey)} />
            </div>
            {items.map(({ id, labelKey, icon: Icon }) => (
              <NavItem
                key={id}
                icon={<Icon />}
                label={t(labelKey)}
                active={section === id}
                onClick={() => onSelect(id)}
              />
            ))}
          </div>
        );
      })}
      {categories.length === 0 && (
        <p className="m-0 px-2.5 py-1.5 text-xs text-muted-foreground">{t("settings.noMatch")}</p>
      )}
    </nav>
  );
}
