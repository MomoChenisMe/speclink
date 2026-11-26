/**
 * 看板與已封存頁共用的搜尋比對規則（spec「與已封存頁的搜尋規則一致」的單一真相）：
 * query 去頭尾空白、不分大小寫、以子字串命中任一欄位；空（或僅空白）query 恆命中。
 */
export function matchesQuery(query: string, ...fields: (string | null | undefined)[]): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return fields.some((f) => (f ?? "").toLowerCase().includes(q));
}

/**
 * 名稱層模糊比對（design D7）：查詢字元依序出現於目標即命中（subsequence，
 * 不分大小寫、去頭尾空白）。僅套用於變更卡名稱與討論卡 slug——摘要與主題
 * 維持子字串（spec「名稱層模糊比對」）。空 query 恆命中。
 */
export function matchesFuzzy(query: string, target: string | null | undefined): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  const t = (target ?? "").toLowerCase();
  let i = 0;
  for (const ch of t) {
    if (ch === q[i]) i++;
    if (i === q.length) return true;
  }
  return false;
}

/** 看板篩選狀態（design D5）：三維度、null＝未啟用；多 chip 與搜尋字串 AND 交集。 */
export interface BoardFilters {
  /** 選定的建立者（{@link creatorOwnership} 的選項 key）。 */
  creator: CreatorKey | null;
  /** 建立時間窗：近 7 天／近 30 天／更早（超過 30 天）。 */
  createdWithin: "7d" | "30d" | "earlier" | null;
  /** 來源討論 slug：命中該討論卡自身與來源討論含該 slug 的變更卡。 */
  fromDiscussion: string | null;
}

export const EMPTY_FILTERS: BoardFilters = {
  creator: null,
  createdWithin: null,
  fromDiscussion: null,
};

/**
 * created 日期（YYYY-MM-DD）是否落在時間窗內（以 today 為基準）。
 * 未啟用（range null）恆命中；日期缺席或不可解析時不命中任何啟用中的窗。
 */
export function matchesCreatedRange(
  created: string | null | undefined,
  range: BoardFilters["createdWithin"],
  today: string,
): boolean {
  if (!range) return true;
  if (!created) return false;
  const days = (new Date(today).getTime() - new Date(created).getTime()) / 86_400_000;
  if (Number.isNaN(days)) return false;
  if (range === "7d") return days <= 7;
  if (range === "30d") return days <= 30;
  return days > 30;
}

/** 建立者篩選讀的兩個欄位（變更卡與討論卡同名）。 */
export interface CreatorFields {
  createdBy?: string | null;
  createdById?: string | null;
}

/** 建立者篩選的一個人：有識別時是識別，沒有識別時是名稱——兩種在結構上分開，不會撞在一起。 */
export type CreatorKey = { id: string } | { name: string };

/** 兩個歸屬是不是同一個人。 */
export function sameCreator(a: CreatorKey | null, b: CreatorKey | null): boolean {
  if (!a || !b) return a === b;
  return "id" in a ? "id" in b && a.id === b.id : "name" in b && a.name === b.name;
}

/** 建立者選單的一個選項：key 存進 {@link BoardFilters.creator}，label 顯示給人看。 */
export interface CreatorOption {
  key: CreatorKey;
  label: string;
}

/** 卡片自己看得出的歸屬：有識別依識別，否則依名稱。 */
function ownKey(card: CreatorFields): CreatorKey | null {
  if (card.createdById) return { id: card.createdById };
  return card.createdBy ? { name: card.createdBy } : null;
}

/**
 * 建立者以人為單位（design D2）：輸入看板上全部的變更卡與討論卡（清單順序），
 * 回傳每張卡的歸屬、選單的選項，與重載後找回選定的人的 resolve。
 * - 有識別的卡依識別歸屬，名稱只當標籤。
 * - 沒有識別的卡：名稱在有識別的卡裡只對應一個識別時併進那個人；零個或多個時依名稱。
 * - 兩者都沒有的卡沒有歸屬（keyOf 回 null）。
 * - 選項依卡片出現的順序；標籤取該人最先出現的名稱；標籤相同的帶識別選項各接識別的
 *   最後 4 個字元，最後 4 個字元也相同時接完整識別。
 * - resolve：選定的人還是選項之一時原樣回傳；選定的名稱在這份資料裡併進一個識別時
 *   回那個識別；都不是時回 null（那個人已不在看板上）。
 * 本機專案沒有識別，每張卡都落在「依名稱」，選項與比對結果和名稱全等比對時相同。
 */
export function creatorOwnership(cards: readonly CreatorFields[]): {
  keyOf: (card: CreatorFields) => CreatorKey | null;
  options: CreatorOption[];
  resolve: (key: CreatorKey) => CreatorKey | null;
} {
  const idsByName = new Map<string, Set<string>>();
  for (const c of cards) {
    if (!c.createdById || !c.createdBy) continue;
    const ids = idsByName.get(c.createdBy) ?? new Set<string>();
    ids.add(c.createdById);
    idsByName.set(c.createdBy, ids);
  }
  const keyOf = (card: CreatorFields): CreatorKey | null => {
    if (!card.createdById && card.createdBy) {
      const ids = idsByName.get(card.createdBy);
      if (ids?.size === 1) return { id: [...ids][0] };
    }
    return ownKey(card);
  };
  // 每個人第一次出現時記下歸屬；標籤取最先出現的名稱（依名稱歸屬的人，名稱就是標籤）。
  const owners: { key: CreatorKey; name: string | null }[] = [];
  for (const c of cards) {
    const key = keyOf(c);
    if (!key) continue;
    const owner = owners.find((o) => sameCreator(o.key, key));
    if (!owner) owners.push({ key, name: c.createdBy || null });
    else owner.name ??= c.createdBy || null;
  }
  const labelled = owners.map(({ key, name }) => ({ key, label: "name" in key ? key.name : (name ?? key.id) }));
  const options = labelled.map(({ key, label }) => {
    if (!("id" in key)) return { key, label };
    const twins = labelled.filter((o) => o.label === label && !sameCreator(o.key, key));
    if (twins.length === 0) return { key, label };
    const short = key.id.slice(-4);
    const clash = twins.some((o) => "id" in o.key && o.key.id.slice(-4) === short);
    return { key, label: `${label} (${clash ? key.id : short})` };
  });
  const resolve = (key: CreatorKey): CreatorKey | null => {
    const listed = (k: CreatorKey | null) => options.find((o) => sameCreator(o.key, k))?.key ?? null;
    return listed(key) ?? ("name" in key ? listed(keyOf({ createdBy: key.name })) : null);
  };
  return { keyOf, options, resolve };
}

/**
 * 篩選 chips 的 AND 交集比對（design D5）：卡片（變更或討論）須同時滿足所有
 * 啟用中的維度。建立者比對卡片的歸屬：看板以 creator 傳入 {@link creatorOwnership}
 * 的結果；沒有傳時取卡片自己看得出的歸屬。變更卡以 fromDiscussions 命中來源討論；
 * 討論卡以自身 slug 命中（spec「來源討論篩選」：該討論卡自身與其衍生變更卡）。
 */
export function matchesFilters(
  filters: BoardFilters,
  card: CreatorFields & {
    creator?: CreatorKey | null;
    created?: string | null;
    fromDiscussions?: string[];
    slug?: string;
  },
  today: string,
): boolean {
  if (filters.creator) {
    const key = card.creator !== undefined ? card.creator : ownKey(card);
    if (!sameCreator(key, filters.creator)) return false;
  }
  if (!matchesCreatedRange(card.created, filters.createdWithin, today)) return false;
  if (filters.fromDiscussion) {
    const hit =
      card.slug === filters.fromDiscussion ||
      (card.fromDiscussions ?? []).includes(filters.fromDiscussion);
    if (!hit) return false;
  }
  return true;
}
