// UI 語言偏好（design D8）已提升到 @speclink/ui 供 Desktop 與 Server 共用。
// 此檔保留為 re-export，讓既有的 `../i18n/locale` import 面維持不變。
export {
  detectSystemLocale,
  readLocalePreference,
  writeLocalePreference,
  resolveUiLocale,
  type LocalePreference,
} from "@speclink/ui";
