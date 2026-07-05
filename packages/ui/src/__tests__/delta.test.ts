import { describe, it, expect } from "vitest";
import { specDeltaCounts, sumDeltaCounts, formatDeltaCounts } from "../delta";

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

describe("sum + format", () => {
  it("sums and formats Spectra-style", () => {
    const total = sumDeltaCounts([
      { added: 1, modified: 0, removed: 0, renamed: 0 },
      { added: 0, modified: 2, removed: 0, renamed: 0 },
    ]);
    expect(formatDeltaCounts(total)).toBe("+1 ~2");
  });
});
