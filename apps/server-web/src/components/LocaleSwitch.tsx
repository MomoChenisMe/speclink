import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  useI18n,
  type LocalePreference,
} from "@speclink/ui";
import { useLocalePreference } from "../i18n/LocaleContext";

// header 的介面語言切換（server-web-console「介面語言支援中文與英文」）：中文／English／
// 跟隨系統三選，與 Desktop 設定頁同一組選項。語言名稱維持母語寫法——把「English」翻成
// 「英文」只會讓看不懂當前語言的人找不到自己的語言。

/** 跟隨系統在 LocalePreference 裡是 null，但 Radix Select 的 item 不接受空字串。 */
const SYSTEM = "__system__";

export function LocaleSwitch() {
  const { t } = useI18n();
  const { pref, setPref } = useLocalePreference();

  return (
    <Select
      value={pref ?? SYSTEM}
      onValueChange={(v) => setPref(v === SYSTEM ? null : (v as LocalePreference))}
    >
      <SelectTrigger aria-label={t("locale.label")} className="w-auto gap-1.5 border-none px-2">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={SYSTEM}>{t("locale.followSystem")}</SelectItem>
        <SelectItem value="zh-TW">繁體中文</SelectItem>
        <SelectItem value="en">English</SelectItem>
      </SelectContent>
    </Select>
  );
}
