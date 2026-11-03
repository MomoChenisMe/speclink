// speclink 網址（desktop-app「以 speclink 網址開啟詳情頁」）：解析表逐列對照 spec 的
// 「網址解析」Example。
import { describe, it, expect, vi, beforeEach } from "vitest";

import defaultCapsRaw from "../../src-tauri/capabilities/default.json?raw";
import baseConfRaw from "../../src-tauri/tauri.conf.json?raw";
import { installDeepLinks, parseSpeclinkUrl } from "../deepLink";
import { APP_MESSAGES } from "../i18n/messages";

describe("parseSpeclinkUrl：合法網址", () => {
  it.each([
    {
      url: "speclink://change/add-search?project=%2FUsers%2Fdev%2Fspeclink",
      link: { kind: "change", name: "add-search", project: "/Users/dev/speclink" },
    },
    {
      url: "speclink://discussion/board-search-bar?project=C%3A%5Cwork%5Capp",
      link: { kind: "discussion", name: "board-search-bar", project: "C:\\work\\app" },
    },
    {
      url: "speclink://change/add-search?project=C%3A%2Fwork%2Fapp",
      link: { kind: "change", name: "add-search", project: "C:/work/app" },
    },
  ])("$url", ({ url, link }) => {
    expect(parseSpeclinkUrl(url)).toEqual({ ok: true, link });
  });
});

describe("parseSpeclinkUrl：格式錯誤", () => {
  it.each([
    { url: "speclink://spec/desktop-app?project=%2Fa", reason: "kind" }, // 未知種類
    { url: "speclink://change/a/b?project=%2Fa", reason: "name" }, // 名稱多於一段
    { url: "speclink://change/?project=%2Fa", reason: "name" }, // 名稱為空
    { url: "speclink://change/add-search", reason: "project" }, // 缺 project
    { url: "speclink://change/add-search?project=work%2Fapp", reason: "project" }, // 不是絕對路徑
    { url: "https://change/add-search?project=%2Fa", reason: "scheme" }, // 不是 speclink 網址
  ])("$url → $reason", ({ url, reason }) => {
    expect(parseSpeclinkUrl(url)).toEqual({ ok: false, reason });
  });
});

// sharp-edges 稽核（網址來自任何本機程式或網頁，一律視為不可信）。
describe("parseSpeclinkUrl：不可信網址", () => {
  it.each([
    { url: "speclink://change/%E0%A4?project=%2Fa", reason: "name" }, // 無效的百分比編碼不得拋例外
    { url: "speclink://change/x?project=%2Fa%E0%A4", reason: "project" }, // 不得靜默換成替代字元
    { url: "speclink://change/a%00b?project=%2Fa", reason: "name" }, // 控制字元
    { url: "speclink://change/x?project=%2Fa%0Ab", reason: "project" }, // 控制字元
    { url: "speclink://change/x?project=%2Fa&project=%2Fb", reason: "project" }, // 重複 project 有歧義，不挑一個
    // 判定：host 大小寫不放寬——spec 只收 change／discussion，CHANGE 照未知種類出 toast，不會靜默誤開。
    { url: "speclink://CHANGE/x?project=%2Fa", reason: "kind" },
    // 判定：.. 不構成缺口——URL 解析先把它收成空名稱；名稱只拿來比對清單，從不當檔案路徑。
    { url: "speclink://change/..?project=%2Fa", reason: "name" },
  ])("$url → $reason", ({ url, reason }) => {
    expect(parseSpeclinkUrl(url)).toEqual({ ok: false, reason });
  });

  // + 照字面保留（RFC 3986 的 query 不是表單編碼）：路徑 /a+b 不得被讀成 /a b。
  it("project 內的 + 不轉成空白", () => {
    expect(parseSpeclinkUrl("speclink://change/x?project=%2Fa+b")).toEqual({
      ok: true,
      link: { kind: "change", name: "x", project: "/a+b" },
    });
  });
});

// 派送（design「啟動時的網址排隊到分頁還原之後」「專案一律走 openProjectAt，詳情等載入後再開」）：
// 外掛、sonner 與 tray.ts 的 openMainWindow 換成假模組，假 store 記下呼叫順序。
const { plugin, toastError, openMainWindow } = vi.hoisted(() => ({
  plugin: {
    launch: null as string[] | null,
    emit: (_urls: string[]) => {},
    unlisten: (() => {}) as () => void,
  },
  toastError: vi.fn(),
  openMainWindow: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-deep-link", () => ({
  getCurrent: async () => plugin.launch,
  onOpenUrl: async (handler: (urls: string[]) => void) => {
    plugin.emit = handler;
    return () => plugin.unlisten();
  },
}));
vi.mock("sonner", () => ({ toast: { error: toastError } }));
vi.mock("../tray", () => ({ openMainWindow }));

const PROJECT = "/Users/dev/speclink";
const PROJECT_KEY = `local:${PROJECT}`;
const CHANGE_URL = "speclink://change/add-search?project=%2FUsers%2Fdev%2Fspeclink";
const DISCUSSION_URL = "speclink://discussion/board-search-bar?project=%2FUsers%2Fdev%2Fspeclink";

const flush = () => new Promise((r) => setTimeout(r, 0));

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

let log: string[] = [];
beforeEach(() => {
  log = [];
  plugin.launch = null;
  plugin.emit = () => {};
  plugin.unlisten = vi.fn();
  openMainWindow.mockReset().mockImplementation(async () => {
    log.push("openMainWindow");
  });
  toastError.mockReset().mockImplementation((message: string) => log.push(`toast:${message}`));
});

function harness(
  over: {
    ready?: Promise<void>;
    launch?: string[];
    openKey?: string | null;
    projectName?: string;
    migrationRoot?: string | null;
  } = {},
) {
  plugin.launch = over.launch ?? null;
  const state = {
    changes: [{ name: "add-search" }],
    discussions: { active: [{ slug: "board-search-bar" }], archived: [] },
    sessions: { [PROJECT_KEY]: { descriptor: { name: over.projectName ?? "speclink" } } },
    migrationRoot: over.migrationRoot ?? null,
    pendingRemoteConflict: null,
    openProjectAt: vi.fn(async (path: string, _options?: { remoteViaChooser?: boolean }) => {
      log.push(`openProjectAt:${path}`);
      return over.openKey === undefined ? PROJECT_KEY : over.openKey;
    }),
    openDetail: vi.fn((name: string) => log.push(`openDetail:${name}`)),
    openDiscussion: vi.fn((slug: string) => log.push(`openDiscussion:${slug}`)),
    setBoardView: vi.fn((view: string) => log.push(`setBoardView:${view}`)),
  };
  const dispose = installDeepLinks({ getState: () => state as never }, over.ready ?? Promise.resolve());
  return { state, dispose, emit: (urls: string[]) => plugin.emit(urls) };
}

describe("installDeepLinks", () => {
  it("(a) 就緒前的啟動網址與事件網址都排隊，就緒後依到達順序逐一處理", async () => {
    const ready = deferred<void>();
    const h = harness({ ready: ready.promise, launch: [CHANGE_URL] });
    await flush();
    h.emit([DISCUSSION_URL]);
    await flush();
    expect(log).toEqual([]);

    // 第一個網址的開專案卡住時，第二個網址不得開始。
    const slow = deferred<string | null>();
    h.state.openProjectAt.mockImplementationOnce(async (path: string) => {
      log.push(`openProjectAt:${path}`);
      return slow.promise;
    });
    ready.resolve();
    await flush();
    expect(log).toEqual(["openMainWindow", `openProjectAt:${PROJECT}`]);

    slow.resolve(PROJECT_KEY);
    await flush();
    expect(log).toEqual([
      "openMainWindow",
      `openProjectAt:${PROJECT}`,
      "openDetail:add-search",
      "openMainWindow",
      `openProjectAt:${PROJECT}`,
      "openDiscussion:board-search-bar",
    ]);
  });

  it("(b) 變更網址依序喚起主視窗、開專案（remote 標記改走選擇器）、開變更詳情", async () => {
    const h = harness({ launch: [CHANGE_URL] });
    await flush();
    expect(log).toEqual(["openMainWindow", `openProjectAt:${PROJECT}`, "openDetail:add-search"]);
    expect(h.state.openProjectAt).toHaveBeenCalledWith(PROJECT, { remoteViaChooser: true });
    expect(toastError).not.toHaveBeenCalled();
  });

  it("(c) 討論網址開討論詳情", async () => {
    const h = harness();
    await flush();
    h.emit([DISCUSSION_URL]);
    await flush();
    expect(log).toEqual(["openMainWindow", `openProjectAt:${PROJECT}`, "openDiscussion:board-search-bar"]);
  });

  it("(d) openProjectAt 回傳 null（對話框或選擇器接手）時不開詳情、不出 toast", async () => {
    harness({ launch: [CHANGE_URL], openKey: null });
    await flush();
    expect(log).toEqual(["openMainWindow", `openProjectAt:${PROJECT}`]);
  });

  it.each([
    ["變更", "speclink://change/no-such-change?project=%2FUsers%2Fdev%2Fspeclink", "deepLink.changeNotFound", "no-such-change"],
    ["討論", "speclink://discussion/no-such-topic?project=%2FUsers%2Fdev%2Fspeclink", "deepLink.discussionNotFound", "no-such-topic"],
  ])("(e) 找不到%s：主區切回看板（關掉詳情頁），toast 含專案名稱與目標名稱", async (_label, url, key, target) => {
    const h = harness({ launch: [url] });
    await flush();
    const message = APP_MESSAGES["zh-TW"][key].replace("{project}", "speclink").replace("{name}", target);
    expect(log).toEqual(["openMainWindow", `openProjectAt:${PROJECT}`, "setBoardView:board", `toast:${message}`]);
    expect(h.state.openDetail).not.toHaveBeenCalled();
    expect(h.state.openDiscussion).not.toHaveBeenCalled();
  });

  it("(e2) 專案名稱含 {name} 字樣時照原字顯示，不被當成佔位符", async () => {
    harness({ launch: ["speclink://change/x?project=%2FUsers%2Fdev%2Fspeclink"], projectName: "a{name}b" });
    await flush();
    expect(toastError).toHaveBeenCalledWith("在 a{name}b 找不到變更 x");
  });

  it("(f) 格式錯誤：只喚起主視窗並出 toast，不開專案", async () => {
    const h = harness({ launch: ["speclink://spec/desktop-app?project=%2FUsers%2Fdev%2Fspeclink"] });
    await flush();
    expect(log).toEqual(["openMainWindow", `toast:${APP_MESSAGES["zh-TW"]["deepLink.invalid"]}`]);
    expect(h.state.openProjectAt).not.toHaveBeenCalled();
  });

  it("(g) 卸載函式解除 onOpenUrl 訂閱，之後排隊的網址也不再處理", async () => {
    const ready = deferred<void>();
    const h = harness({ ready: ready.promise, launch: [CHANGE_URL] });
    await flush();
    h.dispose();
    await flush();
    expect(plugin.unlisten).toHaveBeenCalledTimes(1);
    ready.resolve();
    await flush();
    expect(log).toEqual([]);
  });

  it("(h) 不可取消的浮層開著：喚起主視窗，但不切專案、不開詳情、不出 toast", async () => {
    const h = harness({ launch: [CHANGE_URL], migrationRoot: "/Users/dev/legacy" });
    await flush();
    expect(log).toEqual(["openMainWindow"]);
    expect(h.state.openProjectAt).not.toHaveBeenCalled();
  });

  it("(i) 喚起主視窗失敗時照常處理網址", async () => {
    openMainWindow.mockRejectedValueOnce(new Error("window gone"));
    harness({ launch: [CHANGE_URL] });
    await flush();
    expect(log).toEqual([`openProjectAt:${PROJECT}`, "openDetail:add-search"]);
  });
});

// 網址格式的註冊與權限（design「網址格式的註冊與權限」）：bundler 依 tauri.conf.json 寫進
// Info.plist／NSIS 登錄；主視窗要有外掛預設權限才能呼叫 getCurrent、onOpenUrl。
describe("deep-link 外掛設定", () => {
  it("tauri.conf.json 的 desktop schemes 含 speclink", () => {
    const conf = JSON.parse(baseConfRaw);
    expect(conf.plugins["deep-link"].desktop.schemes).toContain("speclink");
  });

  it("主視窗的 capabilities 含 deep-link 預設權限", () => {
    const caps = JSON.parse(defaultCapsRaw);
    expect(caps.windows).toContain("main");
    expect(caps.permissions).toContain("deep-link:default");
  });
});
