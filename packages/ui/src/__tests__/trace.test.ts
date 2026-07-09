// D1／D2：前端解析正典 spec.md 內 @trace 區塊的 source，聚合至 spec 層級去重保序。
// @trace 格式比照 archive.rs 的 trace_block（source/updated/code）。
import { describe, it, expect } from "vitest";

import { parseTraceSources } from "../trace";

/** 依 archive.rs trace_block 格式組一個 @trace HTML 註解區塊（source 為 null＝畸形、略去該行）。 */
function traceBlock(source: string | null, code: string[] = ["a.rs"]): string {
  const lines = ["<!-- @trace"];
  if (source !== null) lines.push(`source: ${source}`);
  lines.push("updated: 2026-07-09");
  lines.push("code:");
  for (const f of code) lines.push(`  - ${f}`);
  lines.push("-->");
  return lines.join("\n");
}

/** 組多 requirement 的 spec.md：每塊需求後接一個 @trace（比照正典排版）。 */
function specWith(sources: (string | null)[]): string {
  return sources
    .map((s, i) => `### Requirement: R${i}\n\n某段內文。\n\n${traceBlock(s)}`)
    .join("\n\n");
}

describe("parseTraceSources（@trace source 解析）", () => {
  it("單一 @trace 回傳其 source", () => {
    expect(parseTraceSources(specWith(["alpha-change"]))).toEqual(["alpha-change"]);
  });

  // spec Example 表：source 出現序去重且依首次出現保序。
  it.each([
    { seq: ["A", "A", "B"], expected: ["A", "B"] },
    { seq: ["B", "A", "B"], expected: ["B", "A"] },
  ])("去重保序：$seq → $expected", ({ seq, expected }) => {
    expect(parseTraceSources(specWith(seq))).toEqual(expected);
  });

  it("無 @trace 的純 markdown 回空陣列", () => {
    expect(parseTraceSources("# 標題\n\n沒有任何 trace 註解。")).toEqual([]);
  });

  it("畸形區塊（缺 source）靜默略過，只回有效 source", () => {
    // 中間一塊缺 source 行 → 該塊不計；前後有效塊照收。
    expect(parseTraceSources(specWith(["A", null, "B"]))).toEqual(["A", "B"]);
  });

  it("null／undefined／空字串回空陣列", () => {
    expect(parseTraceSources(null)).toEqual([]);
    expect(parseTraceSources(undefined)).toEqual([]);
    expect(parseTraceSources("")).toEqual([]);
  });
});
