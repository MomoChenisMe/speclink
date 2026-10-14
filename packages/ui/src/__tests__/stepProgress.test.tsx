// 多步驟進度列（server-web-shell-alignment design D7）：desktop 工作區精靈與 server 初始設定共用。
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";

import { StepProgress } from "../components/StepProgress";

const filled = () =>
  Array.from(screen.getByTestId("step-bar").children).filter((seg) =>
    (seg.getAttribute("class") ?? "").split(/\s+/).includes("bg-primary"),
  ).length;

describe("StepProgress", () => {
  it("「步驟 n / total」加步驟條，已到的段為主色", () => {
    render(<StepProgress current={2} total={4} />);
    expect(screen.getByText("步驟 2 / 4")).toBeTruthy();
    expect(screen.getByTestId("step-bar").children).toHaveLength(4);
    expect(filled()).toBe(2);
  });

  it("前綴接在步驟文字前，以「 · 」分隔", () => {
    render(<StepProgress prefix="初始設定" current={1} total={2} />);
    expect(screen.getByText("初始設定 · 步驟 1 / 2")).toBeTruthy();
    expect(filled()).toBe(1);
  });
});
