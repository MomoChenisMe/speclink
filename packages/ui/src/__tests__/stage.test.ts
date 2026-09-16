// spec 需求「看板欄位由生命週期標記驅動」：全完成＝已就緒 ＞ started_at 或任務
// 完成數>0＝進行中 ＞ 其餘＝提案中。矩陣值取自 spec 的 Example「欄位判定矩陣」表。
import { describe, it, expect } from "vitest";
import { awaitingManualCount, changeStage, planBlockedBy, planBlockedLabel, planWave, planWaveLabel, STAGE_BAR, STAGE_ICON } from "../stage";
import type { ChangeItem } from "../adapter";

function ci(total: number, done: number, startedAt?: string): ChangeItem {
  return { name: "c", status: "x", totalTasks: total, completedTasks: done, startedAt };
}

describe("changeStage（標記驅動）", () => {
  it.each([
    // [totalTasks, completedTasks, startedAt, expected]
    [0, 0, undefined, "proposed"], // 無 | 0 任務 | 提案中
    [28, 0, undefined, "proposed"], // 無 | 0/28 | 提案中——剛 propose 完不錯置
    [28, 3, undefined, "in-progress"], // 無 | 3/28 | 進行中——任務進度涵蓋繞過工具的寫入路徑
    [28, 0, "2026-07-06", "in-progress"], // 有 | 0/28 | 進行中
    [28, 13, "2026-07-06", "in-progress"], // 有 | 13/28 | 進行中
    [28, 28, undefined, "ready"], // 無 | 28/28 | 已就緒（全完成優先）
    [28, 28, "2026-07-06", "ready"], // 有 | 28/28 | 已就緒
    [0, 0, "2026-07-06", "in-progress"], // 有標記、0 任務——已開工
  ] as const)("total=%i done=%i started=%s → %s", (total, done, startedAt, expected) => {
    expect(changeStage(ci(total, done, startedAt))).toBe(expected);
  });
});

// 進度條與圖示的階段色階單一來源（與 STAGE_BADGE 同模式）：看板與系統匣
// 面板共用，防兩處 magic value 漂移。值即現行看板深淺階梯（50→75→100）。
describe("STAGE_BAR / STAGE_ICON（單一 teal 深淺階梯）", () => {
  it("進度條填色依階段遞深", () => {
    expect(STAGE_BAR).toEqual({
      proposed: "bg-primary/50",
      "in-progress": "bg-primary/75",
      ready: "bg-primary",
    });
  });

  it("圖示色依階段遞深", () => {
    expect(STAGE_ICON).toEqual({
      proposed: "text-primary/50",
      "in-progress": "text-primary/75",
      ready: "text-primary",
    });
  });
});

// spec desktop-app「看板卡片的待手動標示」:判定收斂於階段派生模組單一入口。
describe("awaitingManualCount(待手動判定)", () => {
  const c = (codeTotal: number | undefined, codeRemaining: number | undefined, remaining: number): ChangeItem => ({
    name: "c",
    status: "in-progress",
    totalTasks: 10,
    completedTasks: 10 - remaining,
    ...(codeTotal !== undefined ? { codeTotal, codeComplete: codeTotal - (codeRemaining ?? 0), codeRemaining } : {}),
  });

  it("spec Example「浮現判定」表逐列", () => {
    // | codeTotal | codeRemaining | remaining | 待手動章 |
    const rows: [number, number, number, number][] = [
      [9, 0, 1, 1],
      [7, 0, 3, 3],
      [8, 2, 3, 0],
      [10, 0, 0, 0],
      [0, 0, 2, 0], // 尚無寫碼任務:空真值不浮現
    ];
    for (const [codeTotal, codeRemaining, remaining, want] of rows) {
      expect(
        awaitingManualCount(c(codeTotal, codeRemaining, remaining)),
        `codeTotal=${codeTotal} codeRemaining=${codeRemaining} remaining=${remaining}`,
      ).toBe(want);
    }
  });

  it("remote 缺寫碼進度欄位一律 0(章缺席)", () => {
    expect(awaitingManualCount(c(undefined, undefined, 1))).toBe(0);
  });
});

// spec desktop-app「看板卡片的波次與阻擋標示」：判定收斂於單一入口，欄位缺席
// （remote 摘要、plan 成環、壞 meta）一律回 null／空陣列，卡片只讀結果。
describe("planWave / planBlockedBy（排程欄位讀取入口）", () => {
  const base: ChangeItem = { name: "c", status: "proposed", totalTasks: 3, completedTasks: 0 };

  it("wave 存在時回傳波次，缺席回 null", () => {
    expect(planWave({ ...base, wave: 2, blockedBy: [], dependsOn: [], overlaps: [] })).toBe(2);
    expect(planWave({ ...base, wave: 1 })).toBe(1);
    expect(planWave(base)).toBeNull();
    expect(planWave({ ...base, wave: undefined })).toBeNull();
  });

  it("blockedBy 只在 wave 存在時回傳，其餘回空陣列", () => {
    expect(planBlockedBy({ ...base, wave: 2, blockedBy: ["a", "b"] })).toEqual(["a", "b"]);
    expect(planBlockedBy({ ...base, wave: 1, blockedBy: [] })).toEqual([]);
    expect(planBlockedBy({ ...base, wave: 1 })).toEqual([]);
    expect(planBlockedBy(base)).toEqual([]);
    // 缺 wave 的不合法組合：blockedBy 不單獨成立。
    expect(planBlockedBy({ ...base, blockedBy: ["a"] })).toEqual([]);
  });
});

describe("planWaveLabel / planBlockedLabel（排程文字的單一組裝點）", () => {
  // 審查 Round 1：卡片 tooltip、面板列首與排程分頁各自 replace 佔位符——收成一處。
  const dict: Record<string, string> = {
    "card.wave": "第 {n} 波",
    "card.blockedTitle": "等待：{names}",
    "common.listSeparator": "、",
  };
  const t = (key: string) => dict[key] ?? key;

  it("波次文字套 card.wave 的 {n}", () => {
    expect(planWaveLabel(2, t)).toBe("第 2 波");
  });

  it("前置文字以語系分隔符相連", () => {
    expect(planBlockedLabel(["add-a", "add-b"], t)).toBe("等待：add-a、add-b");
    expect(planBlockedLabel(["add-a"], t)).toBe("等待：add-a");
  });
});
