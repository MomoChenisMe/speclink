// 受保護 route 導向登入時保留的站內返回路徑。白名單由 Server 再驗一次，
// 這裡只確保 SPA 不會把外部 URL 塞進 returnTo（server-web-console
// 「導向遵守伺服器裁決與安全優先序」）。route guard 與 401 session 失效
// 共用同一份判定，避免兩處白名單分歧。

const ALLOWED_FIRST_SEGMENTS = ["account", "activate", "admin"];

/** 通過白名單的站內路徑，否則 null。 */
export function safeReturnTo(pathname: string): string | null {
  const first = pathname.replace(/^\//, "").split(/[/?#]/)[0];
  return ALLOWED_FIRST_SEGMENTS.includes(first) ? pathname : null;
}

/** 登入頁路徑，帶上通過白名單的 returnTo（沒有就不帶）。 */
export function loginRedirect(pathname: string): string {
  const rt = safeReturnTo(pathname);
  return rt ? `/login?returnTo=${encodeURIComponent(rt)}` : "/login";
}
