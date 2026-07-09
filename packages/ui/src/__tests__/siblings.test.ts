import { describe, it, expect } from "vitest";

import type { ChangeItem } from "../adapter";
import { siblingChangesOf } from "../siblings";

function change(name: string, fromDiscussions: string[]): ChangeItem {
  return { name, status: "in-progress", totalTasks: 0, completedTasks: 0, fromDiscussions };
}

describe("siblingChangesOf（同源以來源討論交集判定）", () => {
  // spec「變更的來源討論多值呈現」的 Example 交集判定表：
  // A 的來源、B 的來源、是否同源。
  it.each([
    { a: ["d1", "d2"], b: ["d2"], sibling: true },
    { a: ["d1"], b: ["d1"], sibling: true },
    { a: ["d1", "d2"], b: ["d3"], sibling: false },
    { a: [] as string[], b: ["d1"], sibling: false },
  ])("A=$a B=$b → sibling=$sibling", ({ a, b, sibling }) => {
    const changes = [change("A", a), change("B", b)];
    const result = siblingChangesOf(changes, a, "A");
    expect(result.includes("B")).toBe(sibling);
  });

  it("排除自己、可收多個同源", () => {
    const changes = [
      change("A", ["d1", "d2"]),
      change("B", ["d2"]),
      change("C", ["d1"]),
      change("D", ["d9"]),
    ];
    expect(siblingChangesOf(changes, ["d1", "d2"], "A").sort()).toEqual(["B", "C"]);
  });
});
