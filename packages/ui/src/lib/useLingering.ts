import { useRef } from "react";

/** 抽屜關閉動畫期間沿用最後的主體：宿主關抽屜時同步把主體（capability、change、
 * target、discussion）設為 null，元件若因此 return null 會整棵卸載，Radix Presence
 * 沒機會跑滑出動畫（desktop-app「抽屜與浮層的開關動畫」）。主體為 null 時回傳上一個
 * 非 null 值；下次開啟帶新主體即刻覆蓋。 */
export function useLingering<T>(value: T | null): T | null {
  const last = useRef<T | null>(value);
  if (value !== null) last.current = value;
  return value ?? last.current;
}
