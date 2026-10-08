// spec 需求「看板欄位由生命週期標記驅動」：全完成＝已就緒 ＞ started_at 或任務
// 完成數>0＝進行中 ＞ 其餘＝提案中。矩陣值取自 spec 的 Example「欄位判定矩陣」表。
import { describe, it, expect } from "vitest";
import {
  awaitingManualCount,
  changeStage,
  planArchiveAfter,
  planBlockedBy,
  planBlockedLabel,
  planRequirementOverlap,
  planWave,
  planWaveLabel,
  DISCUSSION_TONE,
  STAGE_BADGE,
  STAGE_BAR,
  STAGE_ICON,
} from "../stage";
import type { ChangeItem } from "../adapter";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { REVIEW_TONE } from "../components/reviewStyle";
import { VERIFY_TONE } from "../components/verifyStyle";
import { IMPROVE_CHIP_TONE, IMPROVE_TONE } from "../components/improveStyle";
import { DELTA_COLORS } from "../components/DeltaBadges";

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

// 生命週期與討論的色相單一來源（design D4）：每階一色相，色值只在 theme.css 的
// stage-* token；看板與系統匣共用這幾張表，表內只保存「階段→token class」對照。
describe("STAGE_* / DISCUSSION_TONE（生命週期每階一色相）", () => {
  it("計數徽章：提案中與進行中為淡底實色字、已就緒為實心", () => {
    expect(STAGE_BADGE).toEqual({
      proposed: "bg-stage-proposed/10 text-stage-proposed",
      "in-progress": "bg-stage-in-progress/10 text-stage-in-progress",
      ready: "bg-stage-ready text-primary-foreground",
    });
  });

  it("進度條填色取各階段 token", () => {
    expect(STAGE_BAR).toEqual({
      proposed: "bg-stage-proposed",
      "in-progress": "bg-stage-in-progress",
      ready: "bg-stage-ready",
    });
  });

  it("圖示色取各階段 token", () => {
    expect(STAGE_ICON).toEqual({
      proposed: "text-stage-proposed",
      "in-progress": "text-stage-in-progress",
      ready: "text-stage-ready",
    });
  });

  it("討論欄與系統匣討論分區共用桃紫 token", () => {
    expect(DISCUSSION_TONE).toEqual({
      icon: "text-stage-discussion",
      badge: "bg-stage-discussion/10 text-stage-discussion",
    });
  });
});

// 狀態→token class 對照（design D1）：集中常數檔只保存對照，深色由 theme.css 的
// 深色 token 承擔，表內不得出現 dark: 成對寫法或原生色階。
describe("語意色對照表只含 token class", () => {
  it("SEMANTIC_TONE／SEMANTIC_SURFACE 取 status token 與 destructive", () => {
    expect(SEMANTIC_TONE).toEqual({
      inProgress: "text-status-progress",
      success: "text-status-success",
      warning: "text-status-warning",
      danger: "text-destructive",
    });
    expect(SEMANTIC_SURFACE).toEqual({
      inProgress: "border-status-progress/40 bg-status-progress/10",
      success: "border-status-success/40 bg-status-success/10",
      warning: "border-status-warning/40 bg-status-warning/10",
      danger: "border-destructive/40 bg-destructive/10",
    });
  });

  it("審查與驗證章：蓋章取 stamp，其餘三態取 status token", () => {
    expect(REVIEW_TONE).toEqual({
      inReview: "text-status-progress",
      reviewed: "text-stamp",
      reviewedStale: "text-status-warning",
      reviewedNotPassed: "text-destructive",
    });
    expect(VERIFY_TONE).toEqual({
      inVerify: "text-status-progress",
      verified: "text-stamp",
      verifiedStale: "text-status-warning",
      verifiedNotPassed: "text-destructive",
    });
  });

  it("改進標示取 improve token", () => {
    expect(IMPROVE_TONE).toBe("text-improve");
    expect(IMPROVE_CHIP_TONE).toBe("bg-improve/10");
  });

  it("delta 四色：新增＝成功、修改＝警示、移除＝destructive、更名＝進行中", () => {
    expect(DELTA_COLORS).toEqual({
      added: "text-status-success",
      modified: "text-status-warning",
      removed: "text-destructive",
      renamed: "text-status-progress",
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

// spec desktop-app「詳情抽屜的排程分頁」：requirement 級重疊與封存順序同走單一入口——
// wave 缺席（remote 未取得 plan、plan 成環、壞 meta）即空陣列；wave 在而新欄位缺席
// （舊 server）同為空陣列，分頁據此顯示空態。
describe("planRequirementOverlap / planArchiveAfter（排程新欄位讀取入口）", () => {
  const base: ChangeItem = { name: "c", status: "proposed", totalTasks: 3, completedTasks: 0 };
  const row = {
    change: "add-b",
    capability: "desktop-app",
    requirement: "看板與任務",
    ownOperation: "MODIFIED",
    otherOperation: "MODIFIED",
    conflict: false,
  } as const;

  it("requirementOverlap 只在 wave 存在時回傳，其餘回空陣列", () => {
    expect(planRequirementOverlap({ ...base, wave: 1, requirementOverlap: [row] })).toEqual([row]);
    expect(planRequirementOverlap({ ...base, wave: 1 })).toEqual([]);
    expect(planRequirementOverlap(base)).toEqual([]);
    expect(planRequirementOverlap({ ...base, requirementOverlap: [row] })).toEqual([]);
  });

  it("archiveAfter 只在 wave 存在時回傳，其餘回空陣列", () => {
    expect(planArchiveAfter({ ...base, wave: 2, archiveAfter: ["add-b"] })).toEqual(["add-b"]);
    expect(planArchiveAfter({ ...base, wave: 2 })).toEqual([]);
    expect(planArchiveAfter(base)).toEqual([]);
    expect(planArchiveAfter({ ...base, archiveAfter: ["add-b"] })).toEqual([]);
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
