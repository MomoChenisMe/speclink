// 看板拖排的落點解析（design D6）——與 tasks.ts 的 resolveDropTarget 同款
// 「純函式＋薄 dragEnd 接線」模式：jsdom 測純函式、真實拖曳留真視窗驗證。

import type { CardKind } from "./adapter";

/** 一欄的可見卡識別碼（視覺序）。變更卡＝變更名、討論卡＝slug。 */
export interface ColumnCards {
  kind: CardKind;
  ids: string[];
}

/** dnd id 前綴：變更名與討論 slug 可能撞名，入 DndContext 前加種類前綴。 */
export function cardDndId(kind: CardKind, id: string): string {
  return kind === "change" ? `chg:${id}` : `disc:${id}`;
}

/** 自 dnd id 還原種類與識別碼；非卡片 id（欄容器、封存落點）回 null。 */
export function parseCardDndId(dndId: string): { kind: CardKind; id: string } | null {
  if (dndId.startsWith("chg:")) return { kind: "change", id: dndId.slice(4) };
  if (dndId.startsWith("disc:")) return { kind: "discussion", id: dndId.slice(5) };
  return null;
}

/**
 * 封存落點浮層的浮現條件（spec「拖曳封存落點以浮層呈現」）：僅拖曳**已就緒**
 * 變更卡（名列 `readyIds`）時浮現——非就緒變更卡的拖曳＝純排序，討論卡不可
 * 拖曳封存，兩者皆不得造成任何佈局變動。
 */
export function archiveZoneVisible(
  activeDndId: string | null,
  readyIds: ReadonlySet<string>,
): boolean {
  if (activeDndId === null) return false;
  const card = parseCardDndId(activeDndId);
  return card?.kind === "change" && readyIds.has(card.id);
}

/**
 * dragEnd 落點解析：active 與 over 屬**同一欄**的卡片時，回傳 arrayMove 後
 * 的前後鄰居（欄頂／欄底為 null）；跨欄、欄容器、封存落點、原位放開一律
 * 回 null——呼叫端不觸發任何寫回（spec「跨欄拖曳不改變變更階段」）。
 */
export function resolveCardDrop(
  columns: ColumnCards[],
  activeDndId: string,
  overDndId: string,
): { kind: CardKind; id: string; prevId: string | null; nextId: string | null } | null {
  const active = parseCardDndId(activeDndId);
  const over = parseCardDndId(overDndId);
  if (!active || !over) return null;
  const col = columns.find((c) => c.kind === active.kind && c.ids.includes(active.id));
  if (!col || col.kind !== over.kind || !col.ids.includes(over.id)) return null;
  const from = col.ids.indexOf(active.id);
  const to = col.ids.indexOf(over.id);
  if (from === to) return null;
  const moved = col.ids.filter((x) => x !== active.id);
  moved.splice(to, 0, active.id);
  return {
    kind: active.kind,
    id: active.id,
    prevId: moved[to - 1] ?? null,
    nextId: moved[to + 1] ?? null,
  };
}
