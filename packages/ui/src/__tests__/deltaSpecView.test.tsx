// spec「規格分頁 delta 區段以色標呈現」（desktop-change-doc-views design D6）。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { CapabilityHeader, DeltaSpecView } from "../components/DeltaBadges";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });

const scenario = (title: string) => [`#### Scenario: ${title}`, "", `- **WHEN** ${title} 的動作`, `- **THEN** ${title} 的結果`, ""];

const DELTA_MD = [
  "## ADDED Requirements",
  "",
  "### Requirement: 新需求",
  "",
  "系統 SHALL 新增。",
  "",
  ...scenario("新增情境"),
  "## MODIFIED Requirements",
  "",
  "### Requirement: 面板樣式（macOS）",
  "",
  "<!-- BEFORE: 舊說明。 -->",
  "",
  "第一段需求文。",
  "",
  "第二段需求文。",
  "",
  "第三段需求文。",
  "",
  "第四段需求文。",
  "",
  "第五段需求文。",
  "",
  ...["甲", "乙", "丙", "丁", "戊"].flatMap(scenario),
  "<!-- REMOVED-SCENARIO: ready workspace 的區塊順序與分割線 -->",
  "<!-- REMOVED-SCENARIO: 進度條依階段深淺 -->",
  "",
  "### Requirement: 第二條修改",
  "",
  "第二條需求文。",
  "",
  ...scenario("二之一"),
  "### Requirement: 第三條修改",
  "",
  "第三條需求文。",
  "",
  ...scenario("三之一"),
].join("\n");

const requirement = (container: HTMLElement, title: string) =>
  Array.from(container.querySelectorAll("[data-requirement]")).find((r) => r.querySelector("h3")!.textContent === title)!;

describe("DeltaSpecView", () => {
  it("色標標頭取代機器標題；每段第一條需求展開並帶操作色條，其餘收合但標題留在本頁目錄", () => {
    const { container } = render(<DeltaSpecView markdown={DELTA_MD} />);
    expect(Array.from(container.querySelectorAll("[data-delta-section]")).map((h) => h.textContent)).toEqual(["+新增", "~修改"]);
    expect(screen.queryByText(/MODIFIED Requirements/)).toBeNull();
    const first = requirement(container, "面板樣式（macOS）");
    expect(first.className).toContain("border-status-warning");
    expect(first.querySelector("button")!.getAttribute("aria-expanded")).toBe("true");
    for (const title of ["第二條修改", "第三條修改"]) {
      const row = requirement(container, title);
      expect(row.querySelector("button")!.getAttribute("aria-expanded")).toBe("false");
      expect(row.textContent).toContain("1 個 scenario");
    }
    expect(screen.queryByText("第二條需求文。")).toBeNull();
    const headings = Array.from(container.querySelectorAll("[data-doc-heading]")).map((h) => h.textContent);
    expect(headings).toEqual(expect.arrayContaining(["新需求", "面板樣式（macOS）", "第二條修改", "第三條修改"]));
    expect(requirement(container, "新需求").className).toContain("border-status-success");
  });

  it("REMOVED-SCENARIO 宣告成 caution 提示框，列出刪除線名稱；註解原文不直出", () => {
    const { container } = render(<DeltaSpecView markdown={DELTA_MD} />);
    const box = container.querySelector("[data-removed-scenarios]")!;
    expect(box.textContent).toContain("本變更移除 2 個 scenario");
    expect(Array.from(box.querySelectorAll("del")).map((d) => d.textContent)).toEqual([
      "ready workspace 的區塊順序與分割線",
      "進度條依階段深淺",
    ]);
    expect(container.textContent).not.toContain("REMOVED-SCENARIO");
    expect(container.textContent).not.toContain("舊說明");
    // 收合時列上帶移除籤。
    fireEvent.click(requirement(container, "面板樣式（macOS）").querySelector("button")!);
    expect(requirement(container, "面板樣式（macOS）").textContent).toContain("移除 2 個 scenario");
  });

  it("需求文超過三段只展開前三段，Scenario 超過三個只展開前三個；點鈕後全部出現", () => {
    render(<DeltaSpecView markdown={DELTA_MD} />);
    expect(screen.getByText("第三段需求文。")).toBeTruthy();
    expect(screen.queryByText("第四段需求文。")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "展開其餘 2 段需求文" }));
    expect(screen.getByText("第五段需求文。")).toBeTruthy();
    expect(screen.getByText(/丙 的結果/)).toBeTruthy();
    expect(screen.queryByText(/丁 的動作/)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "還有 2 個 scenario" }));
    expect(screen.getByText(/戊 的結果/)).toBeTruthy();
  });

  it("Scenario 的 WHEN／THEN 成標籤欄與內容欄兩欄", () => {
    const { container } = render(<DeltaSpecView markdown={DELTA_MD} />);
    const steps = container.querySelector("[data-scenario-steps]")!;
    expect(steps.className).toContain("grid-cols-");
    expect(Array.from(steps.querySelectorAll("span")).map((s) => s.textContent)).toEqual(["WHEN", "THEN"]);
  });

  it("REMOVED 區段照 prose 排", () => {
    const md = "## REMOVED Requirements\n\n### Requirement: 舊需求\n\n**Reason**: 不再需要\n";
    const { container } = render(<DeltaSpecView markdown={md} />);
    expect(container.querySelector("[data-requirement]")).toBeNull();
    expect(screen.getByText(/不再需要/)).toBeTruthy();
  });

  it("無 delta 標題的文件整篇照常渲染", () => {
    const { container } = render(<DeltaSpecView markdown={"# 普通規格\n\n內文。\n"} />);
    expect(container.querySelector("[data-delta-section]")).toBeNull();
    expect(screen.getByText("內文。")).toBeTruthy();
  });
});

describe("CapabilityHeader", () => {
  it("等寬能力名＋計數徽章；有回呼時「看正式規格」開該能力的規格", () => {
    const onOpenSpec = vi.fn();
    const { container } = render(
      <CapabilityHeader cap="desktop-app" counts={{ added: 0, modified: 3, removed: 0, renamed: 0 }} onOpenSpec={onOpenSpec} />,
    );
    const header = container.querySelector("[data-capability-header]")!;
    expect(header.querySelector("[data-doc-heading]")!.textContent).toBe("desktop-app");
    expect(header.textContent).toContain("~3");
    fireEvent.click(screen.getByRole("button", { name: "看正式規格" }));
    expect(onOpenSpec).toHaveBeenCalledWith("desktop-app");
  });

  it("沒有回呼時不渲染「看正式規格」", () => {
    render(<CapabilityHeader cap="desktop-app" counts={{ added: 1, modified: 0, removed: 0, renamed: 0 }} />);
    expect(screen.queryByRole("button", { name: "看正式規格" })).toBeNull();
  });
});
