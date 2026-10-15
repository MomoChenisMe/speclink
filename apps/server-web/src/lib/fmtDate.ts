/** ISO 時間戳的日期部分；沒有值時回 fallback（例如「—」「永久」「從未」）。 */
export function fmtDate(iso: string | null, fallback: string): string {
  return iso ? iso.slice(0, 10) : fallback;
}
