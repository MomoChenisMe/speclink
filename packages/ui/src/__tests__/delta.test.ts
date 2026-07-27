import { describe, it, expect } from "vitest";
import { specDeltaCounts, splitDeltaSections, sumDeltaCounts, formatDeltaCounts } from "../delta";

const MD = `## ADDED Requirements

### Requirement: one
body

#### Scenario: s
- **WHEN** x
- **THEN** y

### Requirement: two
body

## MODIFIED Requirements

### Requirement: three
body

## Notes

### Requirement: not counted (outside op section)
`;

describe("specDeltaCounts", () => {
  it("counts requirements per operation section", () => {
    expect(specDeltaCounts(MD)).toEqual({ added: 2, modified: 1, removed: 0, renamed: 0 });
  });
  it("empty input yields zeros", () => {
    expect(specDeltaCounts(null)).toEqual({ added: 0, modified: 0, removed: 0, renamed: 0 });
  });
});

// spec 需求「規格分頁 delta 區段以色標呈現」的切分面（design D4 delta 標題切分）。
describe("splitDeltaSections", () => {
  it("切出各 delta 區段（種類＋內文），delta 標題行不入內文", () => {
    const sections = splitDeltaSections(MD);
    expect(sections.map((s) => s.op)).toEqual(["added", "modified", null]);
    expect(sections[0].content).toContain("### Requirement: one");
    expect(sections[0].content).toContain("### Requirement: two");
    expect(sections[0].content).not.toContain("ADDED Requirements");
    expect(sections[1].content).toContain("### Requirement: three");
    // 非 delta 的 ## 標題結束 delta 區段（與 specDeltaCounts 同界線），自身照排。
    expect(sections[2].content).toContain("## Notes");
  });

  it("四種 delta 區段依序切出", () => {
    const all = ["ADDED", "MODIFIED", "REMOVED", "RENAMED"]
      .map((op) => `## ${op} Requirements\n\n### Requirement: r-${op}\n`)
      .join("\n");
    expect(splitDeltaSections(all).map((s) => s.op)).toEqual([
      "added",
      "modified",
      "removed",
      "renamed",
    ]);
  });

  it("首個 delta 標題前的前導內文為無種類區段", () => {
    const sections = splitDeltaSections(`前導說明。\n\n${MD}`);
    expect(sections[0].op).toBeNull();
    expect(sections[0].content).toContain("前導說明");
    expect(sections[1].op).toBe("added");
  });

  it("不含任何 delta 區段標題時回整篇單段", () => {
    const sections = splitDeltaSections("# 正典規格\n\n### Requirement: plain\n");
    expect(sections.length).toBe(1);
    expect(sections[0].op).toBeNull();
    expect(sections[0].content).toContain("### Requirement: plain");
  });
});

describe("sum + format", () => {
  it("sums and formats the count summary", () => {
    const total = sumDeltaCounts([
      { added: 1, modified: 0, removed: 0, renamed: 0 },
      { added: 0, modified: 2, removed: 0, renamed: 0 },
    ]);
    expect(formatDeltaCounts(total)).toBe("+1 ~2");
  });
});
