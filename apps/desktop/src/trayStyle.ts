// 系統匣樣式偏好（design D4）：localStorage 單鍵，比照 i18n/locale.ts 的
// app 本機偏好模式；缺鍵或非法值一律視為 native-menu——舊安裝升級後行為不變。
export type TrayStyle = "native-menu" | "panel";

const STORAGE_KEY = "speclink.trayStyle";

/** 讀取偏好；缺鍵或非法值（手改 localStorage）一律視為 native-menu。 */
export function readTrayStylePreference(storage: Storage = localStorage): TrayStyle {
  return storage.getItem(STORAGE_KEY) === "panel" ? "panel" : "native-menu";
}

export function writeTrayStylePreference(style: TrayStyle, storage: Storage = localStorage): void {
  storage.setItem(STORAGE_KEY, style);
}
