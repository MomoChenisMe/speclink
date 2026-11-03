// spec desktop-roadmap「路線圖依波次排列變更」「路線圖畫出依賴線並亮起路線」：分組、
// 連線與路線的判定是純函式（design「路線圖模型為純函式」），版面量測不在這裡。
import { describe, it, expect } from "vitest";
import { roadmapModel, roadmapRoute } from "../roadmapModel";
import type { ChangeItem } from "../adapter";

/** 帶排程欄位的清單項；`wave` 為 undefined 表示 plan 沒排到（壞 metadata）。 */
function item(
  name: string,
  wave: number | undefined,
  tasks: [done: number, total: number],
  dependsOn: string[] = [],
  blockedBy: string[] = [],
): ChangeItem {
  const [completedTasks, totalTasks] = tasks;
  const base: ChangeItem = { name, status: "x", totalTasks, completedTasks };
  return wave === undefined ? base : { ...base, wave, dependsOn, blockedBy };
}

describe("roadmapModel", () => {
  it("依波次升冪分組、波內維持清單順序，next 只標在同名站（Scenario「兩波與下一個」）", () => {
    // 清單順序即 plan 的配置序：add-a、add-c 在第 1 波，add-b 在第 2 波。
    const model = roadmapModel(
      [
        item("add-b", 2, [0, 3], ["add-a"], ["add-a"]),
        item("add-a", 1, [0, 4]),
        item("add-c", 1, [2, 5]),
      ],
      "add-a",
    );
    expect(model.waves.map((w) => w.wave)).toEqual([1, 2]);
    expect(model.waves[0].stations.map((s) => s.name)).toEqual(["add-a", "add-c"]);
    expect(model.waves[1].stations.map((s) => s.name)).toEqual(["add-b"]);
    const flagged = model.waves.flatMap((w) => w.stations).filter((s) => s.next);
    expect(flagged.map((s) => s.name)).toEqual(["add-a"]);
    expect(model.unplaced).toEqual([]);
  });

  it("next 為 null 時沒有站帶下一個標記", () => {
    const model = roadmapModel([item("add-a", 1, [0, 4])], null);
    expect(model.waves[0].stations[0].next).toBe(false);
  });

  // Example「站的階段文字」：階段沿用 changeStage、阻擋沿用 planBlockedBy。
  it.each([
    { name: "add-a", tasks: [0, 4] as [number, number], blockedBy: [], stage: "proposed" },
    { name: "add-c", tasks: [2, 5] as [number, number], blockedBy: [], stage: "in-progress" },
    { name: "add-d", tasks: [6, 6] as [number, number], blockedBy: [], stage: "ready" },
    { name: "add-b", tasks: [0, 3] as [number, number], blockedBy: ["add-a"], stage: "proposed" },
  ])("站的階段：$name → $stage", ({ name, tasks, blockedBy, stage }) => {
    const model = roadmapModel([item(name, 1, tasks, blockedBy, blockedBy)], null);
    const station = model.waves[0].stations[0];
    expect(station).toEqual({
      name,
      stage,
      completedTasks: tasks[0],
      totalTasks: tasks[1],
      blockedBy,
      next: false,
    });
  });

  it("沒有 wave 的變更進 unplaced、不成為站（Scenario「無法排程的變更不成為站」）", () => {
    const model = roadmapModel(
      [item("good-a", 1, [0, 1]), item("broken-meta", undefined, [0, 1]), item("good-b", 1, [0, 1])],
      null,
    );
    expect(model.waves.flatMap((w) => w.stations).map((s) => s.name)).toEqual(["good-a", "good-b"]);
    expect(model.unplaced).toEqual(["broken-meta"]);
  });

  it("連線只收兩端都在路線圖上的 dependsOn：前置連到依賴者", () => {
    const model = roadmapModel(
      [
        item("add-a", 1, [0, 1]),
        item("add-x", 1, [0, 1]),
        item("broken-meta", undefined, [0, 1]),
        item("add-b", 2, [0, 1], ["add-a"], ["add-a"]),
        // 前置一個無法排程、一個已封存（不在清單）——兩條都不畫。
        item("add-c", 3, [0, 1], ["add-b", "broken-meta", "archived-one"], ["add-b"]),
      ],
      null,
    );
    expect(model.edges).toEqual([
      { from: "add-a", to: "add-b" },
      { from: "add-b", to: "add-c" },
    ]);
  });

  it("前置已封存時沒有線（Scenario「前置已封存時不畫線」）", () => {
    // 引擎的 blockedBy 只含作用中的前置：add-a 已封存，add-b 不再被擋。
    const model = roadmapModel([item("add-b", 1, [0, 3], ["add-a"], [])], null);
    expect(model.edges).toEqual([]);
    expect(model.waves[0].stations[0].blockedBy).toEqual([]);
  });

  it("空清單沒有波次、線與無法排程項", () => {
    expect(roadmapModel([], null)).toEqual({ waves: [], edges: [], unplaced: [] });
  });
});

describe("roadmapRoute", () => {
  // Scenario「停在中間的站亮起整條路線」：add-b depends_on add-a、add-c depends_on add-b，
  // add-x 與三者無依賴。
  const model = roadmapModel(
    [
      item("add-a", 1, [0, 1]),
      item("add-x", 1, [0, 1]),
      item("add-b", 2, [0, 1], ["add-a"], ["add-a"]),
      item("add-c", 3, [0, 1], ["add-b"], ["add-b"]),
    ],
    null,
  );

  // Example「路線的範圍」。
  it.each([
    { hover: "add-b", stations: ["add-a", "add-b", "add-c"], edges: ["add-a→add-b", "add-b→add-c"] },
    { hover: "add-a", stations: ["add-a", "add-b", "add-c"], edges: ["add-a→add-b", "add-b→add-c"] },
    { hover: "add-c", stations: ["add-a", "add-b", "add-c"], edges: ["add-a→add-b", "add-b→add-c"] },
    { hover: "add-x", stations: ["add-x"], edges: [] },
  ])("停在 $hover：遞移前置、遞移依賴者與其間的線", ({ hover, stations, edges }) => {
    const route = roadmapRoute(model, hover);
    expect([...route.stations].sort()).toEqual(stations);
    expect([...route.edges].sort()).toEqual(edges);
  });

  it("兩條分支只亮起停留站所在的那條", () => {
    // add-d 與 add-e 都依賴 add-a；停在 add-d 時 add-e 與 add-a→add-e 不在路線上。
    const forked = roadmapModel(
      [
        item("add-a", 1, [0, 1]),
        item("add-d", 2, [0, 1], ["add-a"], ["add-a"]),
        item("add-e", 2, [0, 1], ["add-a"], ["add-a"]),
      ],
      null,
    );
    const route = roadmapRoute(forked, "add-d");
    expect([...route.stations].sort()).toEqual(["add-a", "add-d"]);
    expect([...route.edges]).toEqual(["add-a→add-d"]);
  });
});
