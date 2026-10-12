// 規格詳情頁閱讀卡的推導（desktop-spec-manual-pages design D4）：數字磚、提示框條件與溯源磚，
// 全由需求段與 @trace 推出；形狀缺席回 null、0 或空陣列，不拋錯。

import type { DocTile } from "./components/DetailShell";
import { PURPOSE_TBD_PREFIX, type SpecSections } from "./specSections";

/** 一個 @trace 時點：來源變更與 updated 原字串（純日期或 RFC 3339）。 */
export interface TracePoint {
  source: string;
  updated: string;
}

export interface SpecStats {
  /** `### Requirement:` 段數。 */
  requirements: number;
  /** 各需求段 Scenario 數加總。 */
  scenarios: number;
  /** 全文 @trace 來源變更去重數。 */
  sourceCount: number;
  /** 需求段尾 @trace 中最大／最小的 updated；全無 @trace 為 null。 */
  latest: TracePoint | null;
  earliest: TracePoint | null;
  /** 有 @trace 的需求數。 */
  traced: number;
  /** 沒有 @trace 的需求標題（文件序）。 */
  untraced: string[];
  /** Purpose 以封存佔位前綴起頭。 */
  purposeTbd: boolean;
}

/** 先比前 10 字的日曆日、同日再比全文——純日期與 RFC 3339 混用時以日曆日為準。 */
function compareUpdated(a: string, b: string): number {
  const cmp = (x: string, y: string) => (x < y ? -1 : x > y ? 1 : 0);
  return cmp(a.slice(0, 10), b.slice(0, 10)) || cmp(a, b);
}

/** 數字磚、出處行與提示框的日期只取日曆日。 */
export const calendarDay = (updated: string) => updated.slice(0, 10);

/** 需求段尾 @trace 中最大／最小的 updated；全無 @trace 或 `sections` 為 null 時皆 null。 */
export function traceSpan(sections: SpecSections | null): { latest: TracePoint | null; earliest: TracePoint | null } {
  let latest: TracePoint | null = null;
  let earliest: TracePoint | null = null;
  for (const r of sections?.requirements ?? []) {
    if (!r.trace?.updated) continue;
    const point = { source: r.trace.source, updated: r.trace.updated };
    if (!latest || compareUpdated(point.updated, latest.updated) > 0) latest = point;
    if (!earliest || compareUpdated(point.updated, earliest.updated) < 0) earliest = point;
  }
  return { latest, earliest };
}

/** `sourceCount` 為全文 @trace 來源去重數（頁的溯源列長度）；`sections` 為 null＝規格沒有需求標題。 */
export function specStats(sections: SpecSections | null, sourceCount: number): SpecStats {
  const requirements = sections?.requirements ?? [];
  return {
    requirements: requirements.length,
    scenarios: requirements.reduce((n, r) => n + r.scenarios.length, 0),
    sourceCount,
    ...traceSpan(sections),
    traced: requirements.filter((r) => r.trace).length,
    untraced: requirements.filter((r) => !r.trace).map((r) => r.title),
    purposeTbd: (sections?.purpose ?? "").trim().startsWith(PURPOSE_TBD_PREFIX),
  };
}

/** 「規格」文件的數字磚：需求、Scenario、溯源變更、最近更新（無 @trace 時缺席）；沒有需求段時空陣列。 */
export function specTiles(stats: SpecStats, t: (key: string) => string): DocTile[] {
  if (stats.requirements === 0) return [];
  const tiles: DocTile[] = [
    { value: stats.requirements, label: t("sdetail.tileRequirements") },
    { value: stats.scenarios, label: t("sdetail.tileScenarios") },
    { value: stats.sourceCount, label: t("sdetail.tileSources") },
  ];
  if (stats.latest) {
    tiles.push({
      value: calendarDay(stats.latest.updated),
      label: t("sdetail.tileUpdated").replace("{change}", stats.latest.source),
    });
  }
  return tiles;
}

/** 「溯源」文件的數字磚：來源變更、最早、最近、有來源的需求；沒有來源變更時空陣列。 */
export function traceTiles(stats: SpecStats, t: (key: string) => string): DocTile[] {
  if (stats.sourceCount === 0) return [];
  const point = (p: TracePoint | null, key: string): DocTile[] =>
    p ? [{ value: calendarDay(p.updated), label: t(key).replace("{change}", p.source) }] : [];
  return [
    { value: stats.sourceCount, label: t("sdetail.tileSourceCount") },
    ...point(stats.earliest, "sdetail.tileEarliest"),
    ...point(stats.latest, "sdetail.tileLatest"),
    { value: `${stats.traced} ／ ${stats.requirements}`, label: t("sdetail.tileTraced") },
  ];
}
