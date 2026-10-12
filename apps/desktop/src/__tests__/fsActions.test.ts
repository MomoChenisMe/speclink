// 「開啟 ▾」的文件定位（desktop-readonly-detail-pages design D7；desktop-spec-manual-pages design D8）：
// 依文件參照型別分派到 core 的定位指令——變更 artifact、討論記錄、正式規格、封存變更 artifact、手冊頁。
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(null) }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { tauriFsActionsAdapter } from "../adapter/fsActions";

describe("tauriFsActionsAdapter.locate", () => {
  it("依參照型別呼叫對應的定位指令", async () => {
    const { locate } = tauriFsActionsAdapter();
    await locate("/r", { change: "add-auth", artifact: "tasks.md" });
    expect(invoke).toHaveBeenLastCalledWith("document_location", { root: "/r", change: "add-auth", artifact: "tasks.md" });
    await locate("/r", { discussion: "alpha" });
    expect(invoke).toHaveBeenLastCalledWith("discussion_location", { root: "/r", slug: "alpha" });
    await locate("/r", { spec: "auth" });
    expect(invoke).toHaveBeenLastCalledWith("spec_location", { root: "/r", capability: "auth" });
    await locate("/r", { archived: "2026-07-04-old", artifact: "specs/auth/spec.md" });
    expect(invoke).toHaveBeenLastCalledWith("archived_document_location", {
      root: "/r",
      datedName: "2026-07-04-old",
      artifact: "specs/auth/spec.md",
    });
    await locate("/r", { manual: "tray" });
    expect(invoke).toHaveBeenLastCalledWith("manual_location", { root: "/r", slug: "tray" });
  });
});
