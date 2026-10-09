export type Platform = "macos" | "windows" | "linux";

/** 平台偵測（design D1）：以 UA 判定——無 os plugin，前端自足且可測；非瀏覽器環境回 linux。 */
export function detectPlatform(ua?: string): Platform {
  const source =
    ua ??
    (typeof navigator === "undefined"
      ? ""
      : navigator.userAgent || (navigator as { platform?: string }).platform || "");
  if (/Macintosh|Mac OS/i.test(source)) return "macos";
  if (/Windows/i.test(source)) return "windows";
  return "linux";
}
