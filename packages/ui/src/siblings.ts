import type { ChangeItem } from "./adapter";

/**
 * 同源變更：與選定變更共享至少一份來源討論的其他變更（不含自己）。
 * 判定為「雙方來源討論集合交集非空」——一份討論可扇出多個變更，這些變更
 * 互為同源；一個變更亦可連結多份討論，任一交集即成立。
 */
export function siblingChangesOf(
  changes: ChangeItem[],
  fromDiscussions: string[],
  selfName: string,
): string[] {
  if (fromDiscussions.length === 0) return [];
  return changes
    .filter(
      (c) =>
        c.name !== selfName &&
        (c.fromDiscussions ?? []).some((s) => fromDiscussions.includes(s)),
    )
    .map((c) => c.name);
}
