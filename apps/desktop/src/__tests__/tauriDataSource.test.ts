import { describe, it, expect, vi, beforeEach } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

import { RevertBlockedError } from "@speclink/ui";

import { createTauriDataSource } from "../adapter/tauriDataSource";

describe("createTauriDataSource", () => {
  beforeEach(() => invoke.mockReset());

  it("listChanges keeps the envelope: items (restaleFrom carried) plus the top-level planError", async () => {
    // restaleFrom（待重新反映徽章資料源）隨清單項透傳至前端，供看板卡片渲染；spec
    // client-protocol「變更清單的排程欄位」：頂層 planError（成環訊息或 null）隨清單
    // 一起回到前端，同一次 IO。
    invoke.mockResolvedValueOnce({
      changes: [{ name: "a", status: "s", totalTasks: 1, completedTasks: 0, restaleFrom: ["alpha"] }],
      planError: "dependency cycle: a -> b -> a",
    });
    const ds = createTauriDataSource("/r");
    const payload = await ds.listChanges();
    expect(invoke).toHaveBeenCalledWith("list_changes", { root: "/r" });
    expect(payload.changes[0].restaleFrom).toEqual(["alpha"]);
    expect(payload.planError).toBe("dependency cycle: a -> b -> a");
  });

  it("listSpecs unwraps the { specs } envelope and carries the presentation helper fields", async () => {
    // spec「桌面 app 呈現 change 與 spec 的清單與內容」呈現層輔助欄位（design D2、
    // spec-archive-drawer design D4）：modifiedAt 之外，規格卡收合資訊欄位原樣透傳。
    invoke.mockResolvedValueOnce({
      specs: [
        {
          id: "cap-x",
          modifiedAt: "2026-07-08",
          requirementCount: 7,
          purposeExcerpt: "First line.",
          purposeTbd: false,
          traceCount: 3,
        },
        { id: "cap-y" },
      ],
    });
    const ds = createTauriDataSource("/r");
    const specs = await ds.listSpecs();
    expect(invoke).toHaveBeenCalledWith("list_specs", { root: "/r" });
    expect(specs[0].modifiedAt).toBe("2026-07-08");
    expect(specs[0].requirementCount).toBe(7);
    expect(specs[0].purposeExcerpt).toBe("First line.");
    expect(specs[0].purposeTbd).toBe(false);
    expect(specs[0].traceCount).toBe(3);
    expect(specs[1].modifiedAt).toBeUndefined();
  });

  it("listArchived unwraps the { archived } envelope and carries the card info fields", async () => {
    // 封存卡收合資訊（spec-archive-drawer design D4/D5）：specCount／createdBy／
    // fromDiscussions 原樣透傳。
    invoke.mockResolvedValueOnce({
      archived: [
        {
          datedName: "2026-01-01-a",
          date: "2026-01-01",
          name: "a",
          specCount: 2,
          createdBy: "momo",
          fromDiscussions: ["alpha-ux"],
        },
      ],
    });
    const ds = createTauriDataSource("/r");
    const archived = await ds.listArchived();
    expect(invoke).toHaveBeenCalledWith("archived_changes", { root: "/r" });
    expect(archived[0].datedName).toBe("2026-01-01-a");
    expect(archived[0].specCount).toBe(2);
    expect(archived[0].createdBy).toBe("momo");
    expect(archived[0].fromDiscussions).toEqual(["alpha-ux"]);
  });

  it("getDocument passes change + artifact to the document command", async () => {
    invoke.mockResolvedValueOnce("## Why");
    const ds = createTauriDataSource("/r");
    const doc = await ds.getDocument("chg", "proposal.md");
    expect(invoke).toHaveBeenCalledWith("document", { root: "/r", change: "chg", artifact: "proposal.md" });
    expect(doc).toBe("## Why");
  });

  it("revertChangeToProposed invokes the revert command with root and change", async () => {
    invoke.mockResolvedValueOnce(undefined);
    const ds = createTauriDataSource("/r");
    await ds.revertChangeToProposed("chg");
    expect(invoke).toHaveBeenCalledWith("revert_change_to_proposed", { root: "/r", change: "chg" });
  });

  it("revertChangeToProposed 把守門 JSON 錯誤轉為 RevertBlockedError(結構化證據)", async () => {
    invoke.mockRejectedValueOnce(
      '{"kind":"revertBlocked","checkedTasks":2,"touchedFiles":["src/a.rs"]}',
    );
    const ds = createTauriDataSource("/r");
    const err = await ds.revertChangeToProposed("chg").catch((e: unknown) => e);
    expect(err).toBeInstanceOf(RevertBlockedError);
    expect((err as RevertBlockedError).checkedTasks).toBe(2);
    expect((err as RevertBlockedError).touchedFiles).toEqual(["src/a.rs"]);
  });

  it("revertChangeToProposed 非守門錯誤原樣拋出單行訊息", async () => {
    invoke.mockRejectedValueOnce("not a speclink project: /r");
    const ds = createTauriDataSource("/r");
    const err = await ds.revertChangeToProposed("chg").catch((e: unknown) => e);
    expect(err).not.toBeInstanceOf(RevertBlockedError);
    expect(String(err instanceof Error ? err.message : err)).toContain("not a speclink project");
  });

  it("runVerb invokes the verb command by name with the change arg", async () => {
    invoke.mockResolvedValueOnce({ valid: true });
    const ds = createTauriDataSource("/r");
    await ds.runVerb("validate", "chg");
    expect(invoke).toHaveBeenCalledWith("validate", { root: "/r", change: "chg" });
  });

  it("三選項的兩條處置各走自己的 command", async () => {
    // spec「封存入口的未結工單三選項」：放棄審查＝discard_review（不封存），
    // 照樣帶走＝archive_carry（等同 CLI --carry-review／--carry-verify）。
    invoke.mockResolvedValue(undefined);
    const ds = createTauriDataSource("/r");
    await ds.discardReview?.("chg");
    expect(invoke).toHaveBeenCalledWith("discard_review", { root: "/r", change: "chg" });
    await ds.archiveCarry?.("chg", true, false);
    expect(invoke).toHaveBeenCalledWith("archive_carry", {
      root: "/r",
      change: "chg",
      carryReview: true,
      carryVerify: false,
    });
    await ds.discardVerify?.("chg");
    expect(invoke).toHaveBeenCalledWith("discard_verify", { root: "/r", change: "chg" });
  });

  it("setAllTasks invokes set_all_tasks with change and done", async () => {
    // spec「任務分頁提供批次操作工具列」：全部已完成／重置任務走批次指令單次寫回。
    invoke.mockResolvedValue(undefined);
    const ds = createTauriDataSource("/r");
    await ds.setAllTasks("chg", true);
    expect(invoke).toHaveBeenCalledWith("set_all_tasks", { root: "/r", change: "chg", done: true });
    await ds.setAllTasks("chg", false);
    expect(invoke).toHaveBeenCalledWith("set_all_tasks", { root: "/r", change: "chg", done: false });
  });

  it("reorderCard invokes reorder_card with kind, id and neighbor ids (null = column ends)", async () => {
    // design D5：以鄰居識別碼表達落點；null＝欄頂／欄底。
    invoke.mockResolvedValue(undefined);
    const ds = createTauriDataSource("/r");
    await ds.reorderCard("change", "my-change", "prev-c", null);
    expect(invoke).toHaveBeenCalledWith("reorder_card", {
      root: "/r",
      kind: "change",
      id: "my-change",
      prevId: "prev-c",
      nextId: null,
    });
    await ds.reorderCard("discussion", "slug-x", null, "next-s");
    expect(invoke).toHaveBeenCalledWith("reorder_card", {
      root: "/r",
      kind: "discussion",
      id: "slug-x",
      prevId: null,
      nextId: "next-s",
    });
  });

  it("工單分頁的兩個讀取各走自己的 command（drawer-quality-ticket-tab D2／D5）", async () => {
    // 活工單以 change＋station 定址、封存工單以 datedName＋station 定址；payload
    // 原樣透傳（null＝無工單、遠端 404 或格式壞掉，前端一律空態）。
    const ticket = {
      rounds: [{ index: 1, phase: "discovery", patchHash: null, scope: ["src/a.rs"], findings: [] }],
    };
    invoke.mockResolvedValueOnce(ticket);
    const ds = createTauriDataSource("/r");
    expect(await ds.getStationTicket("chg", "review")).toEqual(ticket);
    expect(invoke).toHaveBeenCalledWith("station_ticket", { root: "/r", change: "chg", station: "review" });
    invoke.mockResolvedValueOnce(null);
    expect(await ds.getArchivedStationTicket("2026-01-01-old", "verify")).toBeNull();
    expect(invoke).toHaveBeenCalledWith("archived_station_ticket", {
      root: "/r",
      datedName: "2026-01-01-old",
      station: "verify",
    });
  });

  it("dependsCandidates invokes depends_candidates with root and change（add-change-plan-remote D9）", async () => {
    // 排程分頁的候選名冊：有 worktree 映射時為副本名冊，由桌面 core 定根。
    invoke.mockResolvedValueOnce(["add-auth"]);
    const ds = createTauriDataSource("/r");
    expect(await ds.dependsCandidates!("add-dark-mode")).toEqual(["add-auth"]);
    expect(invoke).toHaveBeenCalledWith("depends_candidates", { root: "/r", change: "add-dark-mode" });
  });

  it("手冊索引與內文各走自己的 command（desktop-manual-page design D1）", async () => {
    // spec desktop-manual-page：索引 `{ present, reason, pages, uncoveredNew, malformed }`
    // 原樣透傳；內文以 slug 定址。command 名與參數對應 src-tauri 的
    // list_manual_pages(root) 與 get_manual_page(root, slug)。
    invoke.mockResolvedValueOnce({
      present: true,
      reason: null,
      pages: [
        {
          slug: "index",
          title: "手冊",
          section: "開始使用",
          order: 10,
          keywords: [],
          sources: [],
          generated: "2026-09-02",
          stale: false,
        },
      ],
      uncoveredNew: ["github-oauth"],
      malformed: ["broken"],
    });
    const ds = createTauriDataSource("/r");
    const index = await ds.listManualPages();
    expect(invoke).toHaveBeenCalledWith("list_manual_pages", { root: "/r" });
    expect(index.present).toBe(true);
    expect(index.pages[0].slug).toBe("index");
    expect(index.uncoveredNew).toEqual(["github-oauth"]);
    expect(index.malformed).toEqual(["broken"]);

    invoke.mockResolvedValueOnce("# 手冊\n\n內文。");
    expect(await ds.getManualPage("index")).toBe("# 手冊\n\n內文。");
    expect(invoke).toHaveBeenCalledWith("get_manual_page", { root: "/r", slug: "index" });
    invoke.mockResolvedValueOnce(null);
    expect(await ds.getManualPage("missing")).toBeNull();
  });

  it("本地後端不提供 claim（RemoteOnly 動詞不在本地偽造入口）", () => {
    const ds = createTauriDataSource("/r");
    expect(ds.claim).toBeUndefined();
  });
});
