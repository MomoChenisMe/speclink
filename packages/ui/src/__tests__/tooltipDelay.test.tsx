import { describe, it, expect, afterEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup, act } from "@testing-library/react";
import { readFileSync, readdirSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../components/ui/tooltip";

// spec「主題化提示統一延遲」：延遲預設下沉共用元件層（300ms），個別介面不得再自訂。
const DELAY_MS = 300;

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

const hoverTrigger = () => {
  render(
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <button>觸發</button>
        </TooltipTrigger>
        <TooltipContent>提示內容</TooltipContent>
      </Tooltip>
    </TooltipProvider>,
  );
  fireEvent.pointerMove(screen.getByRole("button", { name: "觸發" }), { pointerType: "mouse" });
};

describe("主題化提示統一延遲", () => {
  it("停留未達 300ms 不顯示、達 300ms 顯示", () => {
    vi.useFakeTimers();
    hoverTrigger();

    act(() => {
      vi.advanceTimersByTime(DELAY_MS - 1);
    });
    // 氣泡以 [data-side] 辨識（與 ui.test.tsx 同慣例）——開啟時 Radix 另渲染一份
    // 供輔助技術朗讀的同文字節點，用文字查詢會撞到兩個。
    expect(document.querySelector("[data-side]")).toBeNull();

    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(document.querySelector("[data-side]")?.textContent).toContain("提示內容");
  });
});

// 純檔案系統斷言：以測試檔自身路徑推出 repo root（jsdom 環境下 import.meta.url
// 會被換成 http location，fileURLToPath 失敗——見 theme.test.ts 的同款註記）。
const REPO_ROOT = resolve(dirname(expect.getState().testPath ?? ""), "../../../..");
const SCAN_ROOTS = ["packages/ui/src", "apps/desktop/src", "apps/server-web/src"];
/** 唯一得持有延遲值的檔案：共用元件層自己。 */
const DELAY_OWNER = "packages/ui/src/components/ui/tooltip.tsx";

const scannedSources = () => {
  const files: string[] = [];
  const walk = (path: string) => {
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      if (entry.name === "__tests__" || entry.name === "dist" || entry.name === "node_modules")
        continue;
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else if (entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) files.push(child);
    }
  };
  for (const root of SCAN_ROOTS) walk(`${REPO_ROOT}/${root}`);
  return files.map((f) => [f.slice(REPO_ROOT.length + 1), readFileSync(f, "utf8")] as const);
};

describe("延遲覆寫守門", () => {
  it("共用元件層以外沒有任何 delayDuration 覆寫", () => {
    const overrides = scannedSources()
      .filter(([file, source]) => file !== DELAY_OWNER && source.includes("delayDuration"))
      .map(([file]) => file);
    expect(overrides).toEqual([]);
  });

  it("共用元件層釘住 300ms 預設", () => {
    const source = readFileSync(`${REPO_ROOT}/${DELAY_OWNER}`, "utf8");
    expect(source).toContain("delayDuration = 300");
  });
});
