// React 之外（zustand store）的 t 橋：App 於 Provider 內把當前 t 同步進來，
// store 組使用者可見訊息時經 appT 取字串。切語言後既有瞬時訊息不重譯（可接受）。
import { MESSAGES } from "@speclink/ui";

import { APP_MESSAGES } from "./messages";

let currentT: (key: string) => string = (key) =>
  APP_MESSAGES["zh-TW"][key] ?? MESSAGES["zh-TW"][key] ?? key;

/** App 於 I18nProvider 內同步當前 t（useEffect）。 */
export function setAppT(t: (key: string) => string): void {
  currentT = t;
}

/** store 層取使用者可見字串。 */
export function appT(key: string): string {
  return currentT(key);
}
