// 語言與外觀頁（desktop-settings-mode design D5）：卡「介面」兩列。顯示語言列與外觀列由一般頁
// 的「介面語言」卡共用——同一組控制元件，兩處不分岔。
import {
  SettingsCard,
  SettingsChoice,
  SettingsPageHeader,
  SettingsRow,
  useI18n,
  type ThemePreference,
} from "@speclink/ui";

import type { LocalePreference } from "../../i18n/locale";

/** 「跟隨系統」在偏好裡是 null，Radix Select 的值不接受空值：以哨兵值往返。 */
const FOLLOW_SYSTEM = "system";

export function LocaleRow({
  localePref,
  onLocalePrefChange,
}: {
  localePref: LocalePreference;
  onLocalePrefChange: (pref: LocalePreference) => void;
}) {
  const { t } = useI18n();
  const label = t("settings.displayLanguage");
  return (
    <SettingsRow
      label={label}
      description={t("settings.uiLocaleHelp")}
      control={
        <SettingsChoice
          ariaLabel={label}
          value={localePref ?? FOLLOW_SYSTEM}
          options={[
            { value: FOLLOW_SYSTEM, label: t("settings.followSystem") },
            { value: "zh-TW", label: "繁體中文" },
            { value: "en", label: "English" },
          ]}
          onChange={(v) => onLocalePrefChange(v === "zh-TW" || v === "en" ? v : null)}
        />
      }
    />
  );
}

export function ThemeRow({
  themePref,
  onThemePrefChange,
}: {
  themePref: ThemePreference;
  onThemePrefChange: (pref: ThemePreference) => void;
}) {
  const { t } = useI18n();
  const label = t("settings.appearance");
  return (
    <SettingsRow
      label={label}
      description={t("settings.appearanceHelp")}
      control={
        <SettingsChoice
          ariaLabel={label}
          value={themePref ?? FOLLOW_SYSTEM}
          options={[
            { value: FOLLOW_SYSTEM, label: t("settings.followSystem") },
            { value: "light", label: t("settings.theme.light") },
            { value: "dark", label: t("settings.theme.dark") },
          ]}
          onChange={(v) => onThemePrefChange(v === "light" || v === "dark" ? v : null)}
        />
      }
    />
  );
}

export interface InterfacePrefsProps {
  localePref: LocalePreference;
  onLocalePrefChange: (pref: LocalePreference) => void;
  themePref: ThemePreference;
  onThemePrefChange: (pref: ThemePreference) => void;
}

export function AppearancePage({ localePref, onLocalePrefChange, themePref, onThemePrefChange }: InterfacePrefsProps) {
  const { t } = useI18n();
  return (
    <>
      <SettingsPageHeader title={t("settings.section.appearance")} note={t("settings.pageNote")} />
      <SettingsCard title={t("settings.interfaceCard")}>
        <LocaleRow localePref={localePref} onLocalePrefChange={onLocalePrefChange} />
        <ThemeRow themePref={themePref} onThemePrefChange={onThemePrefChange} />
      </SettingsCard>
    </>
  );
}
