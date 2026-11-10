// 內建 AI 工具勾選的共用斷言：初始化確認、啟用確認與 checkout 步驟都依同一個順序列出五個工具。
// 期望的順序寫在這裡，不讀被測的 BUILTIN_TOOLS。
import { expect } from "vitest";
import { within } from "@testing-library/react";

const TOOL_ORDER = ["claude", "codex", "copilot", "pi", "cursor"] as const;

/** `container` 內恰好依 claude、codex、copilot、pi、cursor 順序出現五個勾選，且只有 `checked` 列出的工具是勾選狀態。 */
export function expectToolCheckboxes(container: HTMLElement, checked: readonly (typeof TOOL_ORDER)[number][]) {
  const boxes = within(container).getAllByRole("checkbox");
  expect(TOOL_ORDER.map((name) => boxes.indexOf(within(container).getByRole("checkbox", { name })))).toEqual([
    0, 1, 2, 3, 4,
  ]);
  expect(boxes.map((box) => box.getAttribute("aria-checked"))).toEqual(
    TOOL_ORDER.map((name) => String(checked.includes(name))),
  );
}
