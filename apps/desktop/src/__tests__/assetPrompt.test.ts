// 技能檔提示承載規格資料夾遷移（desktop-app spec「技能檔提示承載規格資料夾遷移」；
// desktop-spec-dir-migration design D2）：探測回報的遷移狀態也會觸發提示，略過照舊只記這一版。
import { describe, expect, it } from "vitest";

import type { AssetProbeResult, MigrationProbe } from "../adapter/workspace";
import { assetPrompt } from "../assetPrompt";

const PENDING: MigrationProbe = { state: "pending", folder: "openspec", metaFiles: 4 };
const DEFERRED: MigrationProbe = {
  state: "deferred",
  folder: "openspec",
  reason: "otherWorktrees",
  worktrees: ["/work/side"],
};

function probe(over: Partial<AssetProbeResult> = {}): AssetProbeResult {
  return {
    status: "current",
    currentVersion: "v1.3.0",
    tools: [{ tool: "claude", workspaceVersion: "v1.3.0", stale: false, newer: false, missing: false }],
    differingFiles: [],
    migration: { state: "none" },
    ...over,
  };
}

const STALE_TOOLS = [{ tool: "claude", workspaceVersion: "v0.9.0", stale: true, newer: false, missing: false }];

describe("assetPrompt：規格資料夾遷移", () => {
  it("技能檔不過期但待遷移：照樣出現提示", () => {
    expect(assetPrompt(probe({ migration: PENDING }), "A", {})).toEqual({
      kind: "migrate",
      fileCount: 0,
      version: "v1.3.0",
      projectVersion: "v1.3.0",
      migration: PENDING,
    });
  });

  it("技能檔不過期而暫緩：不提示——略過會壓掉排除原因後的待遷移提示，那時才提示", () => {
    expect(assetPrompt(probe({ migration: DEFERRED }), "A", {})).toBeNull();
  });

  it("技能檔過期且暫緩：維持過期提示並帶暫緩原因與 worktree 路徑", () => {
    const prompt = assetPrompt(probe({ status: "stale", tools: STALE_TOOLS, migration: DEFERRED }), "A", {});
    expect(prompt?.kind).toBe("stale");
    expect(prompt?.migration).toEqual(DEFERRED);
  });

  it("技能檔過期且待遷移：維持過期提示並帶遷移內容", () => {
    const prompt = assetPrompt(
      probe({ status: "stale", tools: STALE_TOOLS, differingFiles: ["a", "b"], migration: PENDING }),
      "A",
      {},
    );
    expect(prompt?.kind).toBe("stale");
    expect(prompt?.fileCount).toBe(2);
    expect(prompt?.migration).toEqual(PENDING);
  });

  it("較新態不帶遷移：引擎會拒絕這次更新，資料夾不會動", () => {
    const prompt = assetPrompt(
      probe({
        status: "newer",
        tools: [{ tool: "claude", workspaceVersion: "v1.4.0", stale: false, newer: true, missing: false }],
        migration: PENDING,
      }),
      "A",
      {},
    );
    expect(prompt?.kind).toBe("newer");
    expect(prompt?.migration).toEqual({ state: "none" });
  });

  it("沒有遷移：現版不提示", () => {
    expect(assetPrompt(probe(), "A", {})).toBeNull();
  });

  it("略過只記這一版：同版不再提示，換版後再出現", () => {
    expect(assetPrompt(probe({ migration: PENDING }), "A", { A: "v1.3.0" })).toBeNull();
    expect(assetPrompt(probe({ migration: PENDING, currentVersion: "v1.4.0" }), "A", { A: "v1.3.0" })?.kind).toBe(
      "migrate",
    );
  });
});
