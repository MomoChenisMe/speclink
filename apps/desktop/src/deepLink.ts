// speclink 網址（desktop-app「以 speclink 網址開啟詳情頁」）：外部程式以
// speclink://change/<name>?project=<path> 或 speclink://discussion/<slug>?project=<path>
// 叫桌面 app 開啟詳情頁，以 speclink://roadmap?project=<path> 開啟該專案的路線圖。
// 解析與派送都在這裡；Tauri 殼只註冊外掛。
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { toast } from "sonner";

import { appT } from "./i18n/runtime";
import { findChange, findDiscussion, uncancelableOverlayOpen, type AppState } from "./store";
import { openMainWindow } from "./tray";

export type SpeclinkLinkKind = "change" | "discussion";

/** 解析後的網址；名稱與路徑都已百分比解碼，`project` 為專案資料夾的絕對路徑。 */
export type SpeclinkLink =
  | {
      kind: SpeclinkLinkKind;
      /** 變更名稱或討論 slug。 */
      name: string;
      project: string;
    }
  | { kind: "roadmap"; project: string };

export type ParsedSpeclinkUrl =
  | { ok: true; link: SpeclinkLink }
  | { ok: false; reason: "scheme" | "kind" | "name" | "project" };

const KINDS: readonly string[] = ["change", "discussion", "roadmap"];
/** POSIX 以 / 開頭；Windows 以磁碟代號加 :\ 或 :/ 開頭。 */
const ABSOLUTE_PATH = /^(\/|[A-Za-z]:[\\/])/;

/** 嚴格解碼：無效的百分比編碼或解出控制字元時回 null。不用 URLSearchParams——
 * 它把無效編碼靜默換成替代字元、把 + 讀成空白，路徑會悄悄變成另一個。 */
function decodeStrict(raw: string): string | null {
  let text: string;
  try {
    text = decodeURIComponent(raw);
  } catch {
    return null;
  }
  return /\p{Cc}/u.test(text) ? null : text;
}

export function parseSpeclinkUrl(raw: string): ParsedSpeclinkUrl {
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    return { ok: false, reason: "scheme" };
  }
  if (url.protocol !== "speclink:") return { ok: false, reason: "scheme" };
  if (!KINDS.includes(url.host)) return { ok: false, reason: "kind" };
  const segment = url.pathname.replace(/^\//, "");
  let name: string | null;
  if (url.host === "roadmap") {
    // 路線圖不帶名稱：path 只能為空或單一 /。
    name = segment === "" ? "" : null;
  } else {
    name = segment === "" || segment.includes("/") ? null : decodeStrict(segment);
  }
  if (name === null) return { ok: false, reason: "name" };
  // 恰好一個 project 參數；重複時有歧義，不挑一個用。
  const values = url.search
    .slice(1)
    .split("&")
    .filter((pair) => pair.startsWith("project="))
    .map((pair) => pair.slice("project=".length));
  const project = values.length === 1 ? decodeStrict(values[0]) : null;
  if (project === null || !ABSOLUTE_PATH.test(project)) return { ok: false, reason: "project" };
  if (url.host === "roadmap") return { ok: true, link: { kind: "roadmap", project } };
  return { ok: true, link: { kind: url.host as SpeclinkLinkKind, name, project } };
}

/** 派送所需的最小 store 介面（Zustand bound store 相容——AppState 的結構子集）。 */
export interface DeepLinkStore {
  getState: () => Pick<
    AppState,
    | "changes"
    | "discussions"
    | "sessions"
    | "migrationRoot"
    | "pendingRemoteConflict"
    | "openProjectAt"
    | "openDetail"
    | "openDiscussion"
    | "setBoardView"
    | "setBoardMode"
  >;
}

type DeepLinkState = ReturnType<DeepLinkStore["getState"]>;

/** 兩種網址目標各自的查找、開啟與「找不到」訊息。 */
const TARGETS: Record<
  SpeclinkLinkKind,
  { exists: (s: DeepLinkState, name: string) => boolean; open: (s: DeepLinkState, name: string) => void; notFound: string }
> = {
  change: {
    exists: (s, name) => findChange(s, name) !== undefined,
    open: (s, name) => s.openDetail(name),
    notFound: "deepLink.changeNotFound",
  },
  discussion: {
    exists: (s, slug) => findDiscussion(s, slug) !== undefined,
    open: (s, slug) => s.openDiscussion(slug),
    notFound: "deepLink.discussionNotFound",
  },
};

/** 訂閱啟動網址與執行中的網址，ready（分頁列還原完成）之後依到達順序一次處理一個。
 * 回傳卸載函式：解除訂閱，尚未處理的網址也不再處理（StrictMode 會先卸載再重裝一次）。 */
export function installDeepLinks(store: DeepLinkStore, ready: Promise<unknown>): () => void {
  let disposed = false;
  // 每個網址接在前一個之後；鏈頭是就緒訊號。單一網址失敗不擋後面的。
  let queue: Promise<unknown> = ready.catch(() => {});
  const enqueue = (urls: string[] | null) => {
    for (const url of urls ?? []) {
      queue = queue.then(() => (disposed ? undefined : openLink(store, url))).catch(() => {});
    }
  };
  void getCurrent().then(enqueue, () => {});
  const unlisten = onOpenUrl(enqueue).catch(() => null);
  return () => {
    disposed = true;
    void unlisten.then((stop) => stop?.());
  };
}

/** 處理一個網址：先喚起主視窗（後續的確認框才在使用者眼前），再開專案，載入完才開詳情。
 * 只開啟與檢視，不呼叫任何寫入動作。 */
async function openLink(store: DeepLinkStore, url: string): Promise<void> {
  // 喚起失敗不吞掉網址：視窗也許已在前景，照常處理。
  await openMainWindow().catch(() => {});
  const parsed = parseSpeclinkUrl(url);
  if (!parsed.ok) {
    toast.error(appT("deepLink.invalid"));
    return;
  }
  // 不可取消的浮層開著：不切專案、不開詳情（與系統匣同語意）。
  if (uncancelableOverlayOpen(store.getState())) return;
  const { link } = parsed;
  // null＝現在不能拿這個專案的清單來用：確認框、選擇器或錯誤已由開啟專案流程接手，
  // 或載入失敗、載入期間已切走。remote 標記一律走選擇器，確認前零寫入。
  const key = await store.getState().openProjectAt(link.project, { remoteViaChooser: true });
  if (key === null) return;
  const s = store.getState();
  if (link.kind === "roadmap") {
    // 切回看板頁即關閉開著的詳情頁；路線圖沒有找不到目標的情形。
    s.setBoardView("board");
    s.setBoardMode("roadmap");
    return;
  }
  const { kind, name, project } = link;
  const target = TARGETS[kind];
  if (!target.exists(s, name)) {
    s.setBoardView("board");
    const projectName = s.sessions[key]?.descriptor.name ?? project;
    // 一次代入、以函式代入：名稱來自外部網址，不得被當成佔位符或 $& 取代樣式。
    const message = appT(target.notFound).replace(/\{(project|name)\}/g, (_match, slot: string) =>
      slot === "project" ? projectName : name,
    );
    toast.error(message);
    return;
  }
  target.open(s, name);
}
