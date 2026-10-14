import { useEffect, useState } from "react";

// 以 matchMedia 判斷視窗寬度／偏好，據以條件渲染側欄或 Sheet（D6）。
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(
    () => typeof window !== "undefined" && window.matchMedia(query).matches,
  );
  useEffect(() => {
    const mql = window.matchMedia(query);
    const handler = () => setMatches(mql.matches);
    handler();
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, [query]);
  return matches;
}

/** 窄螢幕（<1024px）：殼改用 Sheet 導覽、列表改窄版——全站同一個斷點。 */
export function useNarrow(): boolean {
  return useMediaQuery("(max-width: 1023px)");
}
