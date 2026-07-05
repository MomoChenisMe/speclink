import { describe, it, expect } from "vitest";
import { changeStage } from "../stage";
import type { ChangeItem } from "../adapter";

function ci(total: number, done: number): ChangeItem {
  return { name: "c", status: "x", totalTasks: total, completedTasks: done };
}

describe("changeStage", () => {
  it.each([
    [0, 0, "proposed"],
    [21, 0, "in-progress"],
    [21, 13, "in-progress"],
    [21, 21, "ready"],
    [3, 3, "ready"],
  ] as const)("total=%i done=%i → %s", (total, done, expected) => {
    expect(changeStage(ci(total, done))).toBe(expected);
  });
});
