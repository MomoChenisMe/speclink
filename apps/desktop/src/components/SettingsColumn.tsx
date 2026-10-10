// 設定分類欄（desktop-settings-mode design D2）：設定模式下取代專案欄——頂部全圓搜尋框，
// 其下「應用程式」群組五個分類；搜尋以分類名與各頁列標籤過濾，只過濾分類、不跳到列。
import { useState } from "react";
import { NavItem, SearchField, SettingsGroupHeader, useI18n } from "@speclink/ui";

import { matchAppSettings, type SettingsSection } from "../settingsCatalog";

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
      <div className="pb-1">
        <SettingsGroupHeader label={t("settings.group.app")} />
      </div>
      {categories.map(({ id, labelKey, icon: Icon }) => (
        <NavItem
          key={id}
          icon={<Icon />}
          label={t(labelKey)}
          active={section === id}
          onClick={() => onSelect(id)}
        />
      ))}
      {categories.length === 0 && (
        <p className="m-0 px-2.5 py-1.5 text-xs text-muted-foreground">{t("settings.noMatch")}</p>
      )}
    </nav>
  );
}
